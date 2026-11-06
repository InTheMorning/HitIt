//! Saved search new-results actor (ADR 0040, ADR 0089 Decision 4, packet
//! 007 Part C).
//!
//! The actor runs each saved search in sequence, with its first page only.
//! It counts the result keys that the seen set does not hold. It owns its
//! own database connection, so no UI lock is held during a read or an Index
//! request. A run starts at most once in `SAVED_SEARCH_COUNT_INTERVAL`. The
//! actor stops with the session.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rusqlite::Connection;
use tokio::sync::{mpsc, watch};

use crate::application::application_query_service::ApplicationQueryService;
use crate::application::command_bus::ApplicationCommand;
use crate::application::command_context::CommandContext;
use crate::application::queries::search::FetchIndexSearchResults;
use crate::application::session_lifecycle::SessionLifecycle;
use crate::config::MusicIndexEndpoint;
use crate::db::saved_search_seen;
use crate::view_models::library_search_facts::LibrarySearchFacts;
use crate::view_models::search_results::{
    new_result_count, ArtistSearchFacts, IndexSearchPlan, NewResultCount, SavedSearchId,
    SearchResultsInspectorPageVm, SearchRun,
};

/// The least time between two runs of the saved search counts (ADR 0089
/// Decision 4).
pub(crate) const SAVED_SEARCH_COUNT_INTERVAL: Duration = Duration::from_mins(10);

/// Tells if a run of the saved search counts is due. `last_run` is the start
/// of the last run, or `None` when no run has started.
#[must_use]
pub(crate) fn count_run_due(last_run: Option<Instant>, now: Instant) -> bool {
    match last_run {
        None => true,
        Some(last) => now.saturating_duration_since(last) >= SAVED_SEARCH_COUNT_INTERVAL,
    }
}

/// One saved search to count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearchCountJob {
    /// The saved search.
    pub(crate) id: SavedSearchId,
    /// The stored query text of the saved search.
    pub(crate) query: String,
}

/// One request to count the saved searches. The facts come from the Library
/// tree at the time of the request.
#[derive(Clone, Debug)]
pub(crate) struct SavedSearchCountRequest {
    /// The saved searches, in the order of the Search page.
    pub(crate) jobs: Vec<SavedSearchCountJob>,
    /// The Library facts that the Library part of a run reads.
    pub(crate) library: LibrarySearchFacts,
    /// The artist facts that the rows of a run read, with the name matches
    /// that the Index part uses.
    pub(crate) artists: ArtistSearchFacts,
}

/// The counts of the saved searches after the last run. A saved search that
/// has no entry shows no count.
#[derive(Clone, Debug, Default)]
pub(crate) struct SavedSearchCountsSnapshot {
    pub(crate) counts: BTreeMap<SavedSearchId, NewResultCount>,
}

/// Caller-side handle of the saved search counts actor. Dropping the handle
/// closes the request channel, and the actor then ends.
pub(crate) struct SavedSearchCountsHandle {
    snapshot: watch::Receiver<SavedSearchCountsSnapshot>,
    requests: mpsc::UnboundedSender<SavedSearchCountRequest>,
}

impl SavedSearchCountsHandle {
    /// Subscribes to the counts. Each run publishes one snapshot for each
    /// saved search it counts.
    #[must_use]
    pub(crate) fn subscribe(&self) -> watch::Receiver<SavedSearchCountsSnapshot> {
        self.snapshot.clone()
    }

    /// Sends `request` to the actor. Gives `false` when the actor has
    /// stopped. The actor ignores a request that comes within the interval.
    pub(crate) fn request(&self, request: SavedSearchCountRequest) -> bool {
        self.requests.send(request).is_ok()
    }
}

/// Spawns the actor on the session. The actor uses `conn` for its own
/// reads and writes.
#[must_use]
pub(crate) fn spawn(
    conn: Connection,
    endpoint: MusicIndexEndpoint,
    session: &SessionLifecycle,
) -> SavedSearchCountsHandle {
    let (snapshot_tx, snapshot_rx) = watch::channel(SavedSearchCountsSnapshot::default());
    let (request_tx, mut request_rx) = mpsc::unbounded_channel::<SavedSearchCountRequest>();
    let stop = session.stop_token();
    session.spawn_actor("Saved search new results", async move {
        let mut conn = conn;
        let mut last_run: Option<Instant> = None;
        let mut counts: BTreeMap<SavedSearchId, NewResultCount> = BTreeMap::new();
        loop {
            tokio::select! {
                biased;
                () = stop.cancelled() => break,
                request = request_rx.recv() => {
                    let Some(request) = request else { break };
                    let now = Instant::now();
                    if !count_run_due(last_run, now) {
                        continue;
                    }
                    last_run = Some(now);
                    let shared = Arc::new(request);
                    for index in 0..shared.jobs.len() {
                        if stop.is_cancelled() {
                            break;
                        }
                        let id = shared.jobs[index].id;
                        let job_request = Arc::clone(&shared);
                        let job_endpoint = endpoint.clone();
                        let job_conn = conn;
                        let joined = tokio::task::spawn_blocking(move || {
                            let count = count_saved_search(
                                &job_conn,
                                &job_endpoint,
                                &job_request,
                                index,
                            );
                            (job_conn, count)
                        })
                        .await;
                        let Ok((returned, count)) = joined else {
                            return;
                        };
                        conn = returned;
                        counts.insert(id, count);
                        let _ = snapshot_tx.send(SavedSearchCountsSnapshot {
                            counts: counts.clone(),
                        });
                    }
                }
            }
        }
    });
    SavedSearchCountsHandle {
        snapshot: snapshot_rx,
        requests: request_tx,
    }
}

/// Counts the new results of job `index` of `request`. The Library part
/// reads `conn`. The Index part sends its first page, and no lock is held
/// while it waits. A failed read or request gives `Unavailable`, and no seen
/// row changes. A saved search with no seen set stores its keys and gives
/// zero.
fn count_saved_search(
    conn: &Connection,
    endpoint: &MusicIndexEndpoint,
    request: &SavedSearchCountRequest,
    index: usize,
) -> NewResultCount {
    let job = &request.jobs[index];
    let run = SearchRun::new(&job.query);
    if !run.is_runnable() {
        return NewResultCount::Unavailable;
    }
    let Ok(rows) = ApplicationQueryService::new().search_local_library_tracks(
        conn,
        run.query(),
        &request.library,
        None,
    ) else {
        return NewResultCount::Unavailable;
    };
    let mut page =
        SearchResultsInspectorPageVm::from_library_search(run.text(), &rows, &request.artists);
    if let IndexSearchPlan::Send(index_request) = run.index_plan() {
        let command = FetchIndexSearchResults::new(
            endpoint.clone(),
            index_request.clone(),
            request.artists.matches.clone(),
        );
        let Ok(outcome) = command.execute(&CommandContext::next()) else {
            return NewResultCount::Unavailable;
        };
        let (index_rows, _events) = outcome.into_parts();
        page.replace_index_results(index_rows);
    }
    let result_keys = page.result_keys();
    let Ok(seen) = saved_search_seen::read_seen(conn, job.id.0) else {
        return NewResultCount::Unavailable;
    };
    match seen {
        None => {
            if saved_search_seen::mark_seen(conn, job.id.0, &result_keys).is_err() {
                return NewResultCount::Unavailable;
            }
            NewResultCount::Count(new_result_count(&result_keys, None))
        }
        Some(seen) => NewResultCount::Count(new_result_count(&result_keys, Some(&seen))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::saved_searches::{insert_saved_search, StoredStartTab};
    use crate::db::{upgrades, CURRENT_VERSION};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        conn
    }

    fn insert_saved(conn: &Connection, query: &str) -> i64 {
        insert_saved_search(conn, query, StoredStartTab::Releases, 1).unwrap()
    }

    fn insert_track(conn: &Connection, item_guid: &str, title: &str) {
        conn.execute(
            "INSERT OR IGNORE INTO feeds (id, feed_url, feed_guid, title) VALUES (1, 'https://example.test/feed.xml', 'feed-guid', 'Feed')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, track_title, artist_name, album_title, duration_seconds, item_value_json, extra_json, is_in_library)
             VALUES (1, ?1, ?2, 'Artist', 'Album', 60, '{}', '{}', 1)",
            rusqlite::params![item_guid, title],
        )
        .unwrap();
    }

    fn request_for(id: i64, query: &str) -> SavedSearchCountRequest {
        SavedSearchCountRequest {
            jobs: vec![SavedSearchCountJob {
                id: SavedSearchId(id),
                query: query.to_owned(),
            }],
            library: LibrarySearchFacts::default(),
            artists: ArtistSearchFacts::default(),
        }
    }

    fn endpoint() -> MusicIndexEndpoint {
        MusicIndexEndpoint::from("http://127.0.0.1:9")
    }

    #[test]
    fn count_run_is_due_when_no_run_started_or_the_interval_passed() {
        let start = Instant::now();
        assert!(count_run_due(None, start));
        assert!(!count_run_due(
            Some(start),
            start + Duration::from_secs(599)
        ));
        assert!(count_run_due(Some(start), start + Duration::from_mins(10)));
    }

    #[test]
    fn first_run_stores_the_keys_and_gives_zero_then_counts_only_new_keys() {
        let conn = setup_db();
        let id = insert_saved(&conn, "in:library ambient");
        insert_track(&conn, "item-1", "Ambient One");
        let request = request_for(id, "in:library ambient");

        let first = count_saved_search(&conn, &endpoint(), &request, 0);
        assert_eq!(first, NewResultCount::Count(0));
        let stored = saved_search_seen::read_seen(&conn, id).unwrap();
        assert!(stored.is_some(), "the first run stores its seen set");

        insert_track(&conn, "item-2", "Ambient Two");
        let second = count_saved_search(&conn, &endpoint(), &request, 0);
        assert_eq!(second, NewResultCount::Count(1));
    }

    /// ADR 0089 Decision 4 (operator decision of 2026-11-06): two saved
    /// searches that hold one query keep their own seen sets. Each count
    /// follows its own id.
    #[test]
    fn two_saved_searches_with_one_query_count_separately() {
        let conn = setup_db();
        let query = "in:library ambient";
        let first = insert_saved(&conn, query);
        let second = insert_saved(&conn, query);
        insert_track(&conn, "item-1", "Ambient One");
        let first_request = request_for(first, query);
        let second_request = request_for(second, query);

        assert_eq!(
            count_saved_search(&conn, &endpoint(), &first_request, 0),
            NewResultCount::Count(0)
        );
        assert_eq!(
            count_saved_search(&conn, &endpoint(), &second_request, 0),
            NewResultCount::Count(0),
            "the first saved search does not initialise the second one"
        );

        insert_track(&conn, "item-2", "Ambient Two");
        assert_eq!(
            count_saved_search(&conn, &endpoint(), &first_request, 0),
            NewResultCount::Count(1)
        );
        assert_eq!(
            count_saved_search(&conn, &endpoint(), &second_request, 0),
            NewResultCount::Count(1)
        );
    }

    #[test]
    fn a_failed_index_request_gives_no_count_and_stores_nothing() {
        let conn = setup_db();
        let id = insert_saved(&conn, "ambient");
        let request = request_for(id, "ambient");

        let count = count_saved_search(&conn, &endpoint(), &request, 0);
        assert_eq!(count, NewResultCount::Unavailable);
        assert_eq!(saved_search_seen::read_seen(&conn, id).unwrap(), None);
    }
}
