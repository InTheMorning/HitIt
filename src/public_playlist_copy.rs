//! Copies a public playlist of the Index into a local playlist (ADR 0089
//! Decision 1, packet 010).
//!
//! The copy downloads no file. Each remote feed is read once through the
//! RSS path of a subscribe: `rss::fetch_feed_document`, then
//! `rss::subscribe_feed_body`. That path makes the `feeds` and `tracks`
//! rows, with `is_in_library` false (ADR 0075 provenance). Requests to one
//! host keep the minimum interval of ADR 0084 task 039. A host that answers
//! HTTP 429 gets no more requests in the same copy.
//!
//! The remote items come from `MusicIndex`, so they are untrusted input.
//! The database lock is never held during an HTTP request.

#![warn(clippy::pedantic)]

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context as _, Result};
use rusqlite::Connection;

use crate::api::RemoteItem;
use crate::config::MusicIndexEndpoint;
use crate::db;
use crate::playlist_service;
use crate::rss::{self, HostThrottled};

/// The minimum wait between two RSS requests to one host (ADR 0084 task
/// 039).
pub const RSS_HOST_INTERVAL: Duration = Duration::from_secs(2);

/// Why a copy did not finish.
#[derive(Debug)]
pub enum CopyError {
    /// A playlist with the name already exists. Nothing was written.
    NameTaken(String),
    /// The copy failed. The error names the cause.
    Failed(anyhow::Error),
}

/// The counts of a copy that finished.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyOutcome {
    /// The id of the new local playlist.
    pub playlist_id: i64,
    /// The trimmed name of the new local playlist.
    pub name: String,
    /// The tracks that were appended, in `position` sequence.
    pub copied: usize,
    /// The remote items that were offered.
    pub total: usize,
}

impl CopyOutcome {
    /// The remote items that did not become a track of the playlist.
    #[must_use]
    pub const fn skipped(&self) -> usize {
        self.total - self.copied
    }
}

/// Makes the local playlist `name` with the tracks of `remote_items`.
///
/// `fetch` reads one RSS document. It is `rss::fetch_feed_document` in the
/// app. A test passes a stub. `min_interval` is the wait between two
/// requests to one host.
///
/// # Errors
///
/// Returns [`CopyError::NameTaken`] before any write when the name exists,
/// and [`CopyError::Failed`] for a database failure or an empty name.
pub fn copy_public_playlist(
    conn: &Arc<Mutex<Connection>>,
    endpoint: &MusicIndexEndpoint,
    name: &str,
    remote_items: &[RemoteItem],
    fetch: impl FnMut(&str) -> Result<Vec<u8>>,
    min_interval: Duration,
) -> Result<CopyOutcome, CopyError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CopyError::Failed(anyhow!("Playlist name cannot be empty")));
    }
    if name_taken(conn, name).map_err(CopyError::Failed)? {
        return Err(CopyError::NameTaken(name.to_owned()));
    }

    let mut items: Vec<&RemoteItem> = remote_items.iter().collect();
    items.sort_by_key(|item| item.position.unwrap_or(i64::MAX));
    let total = items.len();

    let mut reader = FeedReader::new(fetch, min_interval);
    let mut track_ids = Vec::new();
    for item in items {
        let resolved =
            resolve_item(conn, endpoint, item, &mut reader).map_err(CopyError::Failed)?;
        track_ids.extend(resolved);
    }

    let playlist_id = {
        let db = lock(conn).map_err(CopyError::Failed)?;
        match playlist_service::create(&db, name) {
            Ok(id) => id,
            Err(error) => {
                if db::playlist_name_taken(&db, name).unwrap_or(false) {
                    return Err(CopyError::NameTaken(name.to_owned()));
                }
                return Err(CopyError::Failed(error));
            }
        }
    };
    {
        let db = lock(conn).map_err(CopyError::Failed)?;
        for track_id in &track_ids {
            playlist_service::append_track(&db, playlist_id, *track_id)
                .context("append copied track")
                .map_err(CopyError::Failed)?;
        }
    }

    Ok(CopyOutcome {
        playlist_id,
        name: name.to_owned(),
        copied: track_ids.len(),
        total,
    })
}

fn lock(conn: &Arc<Mutex<Connection>>) -> Result<std::sync::MutexGuard<'_, Connection>> {
    conn.lock().map_err(|_| anyhow!("database lock poisoned"))
}

fn name_taken(conn: &Arc<Mutex<Connection>>, name: &str) -> Result<bool> {
    let db = lock(conn)?;
    db::playlist_name_taken(&db, name)
}

/// The track of one remote item, or `None` when the item names no track
/// that the copy can read. An item with no item GUID is skipped.
fn resolve_item<F: FnMut(&str) -> Result<Vec<u8>>>(
    conn: &Arc<Mutex<Connection>>,
    endpoint: &MusicIndexEndpoint,
    item: &RemoteItem,
    reader: &mut FeedReader<F>,
) -> Result<Option<i64>> {
    let Some(item_guid) = non_empty(item.remote_item_guid.as_deref()) else {
        return Ok(None);
    };
    if let Some(feed_guid) = non_empty(item.remote_feed_guid.as_deref()) {
        let db = lock(conn)?;
        if let Some(id) = db::find_track_id_by_feed_guid_and_item_guid(&db, feed_guid, item_guid)? {
            return Ok(Some(id));
        }
    }
    let Some(feed_url) = non_empty(item.remote_feed_url.as_deref()) else {
        return Ok(None);
    };
    reader.read_once(conn, endpoint, feed_url)?;
    let db = lock(conn)?;
    db::find_track_id(&db, Some(feed_url), Some(item_guid), None)
}

/// Reads each feed URL once per copy, and keeps the host limits.
struct FeedReader<F> {
    fetch: F,
    min_interval: Duration,
    /// The result of each feed URL that was read: `true` when its rows
    /// were stored.
    outcomes: HashMap<String, bool>,
    last_request: HashMap<String, Instant>,
    /// Hosts that answered HTTP 429. They get no more requests.
    stopped_hosts: BTreeSet<String>,
}

impl<F: FnMut(&str) -> Result<Vec<u8>>> FeedReader<F> {
    fn new(fetch: F, min_interval: Duration) -> Self {
        Self {
            fetch,
            min_interval,
            outcomes: HashMap::new(),
            last_request: HashMap::new(),
            stopped_hosts: BTreeSet::new(),
        }
    }

    /// Reads `feed_url` unless it was read already in this copy. Gives
    /// `true` when its rows are stored.
    fn read_once(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        endpoint: &MusicIndexEndpoint,
        feed_url: &str,
    ) -> Result<bool> {
        if let Some(stored) = self.outcomes.get(feed_url) {
            return Ok(*stored);
        }
        let host = rss::feed_host(feed_url).unwrap_or_else(|| feed_url.to_owned());
        if self.stopped_hosts.contains(&host) {
            self.outcomes.insert(feed_url.to_owned(), false);
            return Ok(false);
        }
        if let Some(last) = self.last_request.get(&host) {
            let wait = self.min_interval.checked_sub(last.elapsed());
            if let Some(wait) = wait {
                thread::sleep(wait);
            }
        }
        self.last_request.insert(host.clone(), Instant::now());

        let stored = match (self.fetch)(feed_url) {
            Ok(body) => store_document(conn, endpoint, feed_url, &body)?,
            Err(error) => {
                if error.downcast_ref::<HostThrottled>().is_some() {
                    self.stopped_hosts.insert(host);
                }
                false
            }
        };
        self.outcomes.insert(feed_url.to_owned(), stored);
        Ok(stored)
    }
}

/// Stores one RSS document, as a subscribe does. The feed stays out of
/// the subscriptions when it was not subscribed before. A document that
/// does not parse gives `false`, so its items count as skipped.
fn store_document(
    conn: &Arc<Mutex<Connection>>,
    endpoint: &MusicIndexEndpoint,
    feed_url: &str,
    body: &[u8],
) -> Result<bool> {
    let mut db = lock(conn)?;
    let prior_subscribed = db::feed_is_subscribed_by_url(&db, feed_url).unwrap_or(false);
    match rss::subscribe_feed_body(&mut db, feed_url, body, endpoint) {
        Ok(()) => {
            if !prior_subscribed {
                db::set_feed_subscribed_by_url(&db, feed_url, false)?;
            }
            Ok(true)
        }
        Err(error) => {
            eprintln!("copy of a public playlist did not store {feed_url}: {error:#}");
            Ok(false)
        }
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const FEED_URL: &str = "https://feed.test/album.xml";

    fn rss_body(item_guids: &[&str]) -> Vec<u8> {
        let items: Vec<String> = item_guids
            .iter()
            .map(|guid| {
                format!(
                    r#"<item><guid>{guid}</guid><title>Song {guid}</title><enclosure url="https://feed.test/{guid}.mp3" type="audio/mpeg" length="123"/></item>"#
                )
            })
            .collect();
        let items = items.concat();
        format!(
            r#"<rss version="2.0"><channel><title>Album</title><link>https://feed.test</link><description>Fixture</description>{items}</channel></rss>"#
        )
        .into_bytes()
    }

    fn setup() -> Result<Arc<Mutex<Connection>>> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    fn remote(position: i64, feed_url: Option<&str>, item: Option<&str>) -> RemoteItem {
        RemoteItem {
            position: Some(position),
            remote_feed_guid: Some("album-guid".to_owned()),
            remote_feed_url: feed_url.map(str::to_owned),
            remote_item_guid: item.map(str::to_owned),
            ..RemoteItem::default()
        }
    }

    fn endpoint() -> MusicIndexEndpoint {
        "invalid endpoint".into()
    }

    /// ADR 0089 packet 010: the copy makes the playlist in `position`
    /// sequence, and it writes no file.
    #[test]
    fn adr_0089_copy_to_my_playlists_keeps_position_order_and_writes_no_file() -> Result<()> {
        let conn = setup()?;
        let items = [
            remote(2, Some(FEED_URL), Some("second")),
            remote(1, Some(FEED_URL), Some("first")),
        ];
        let outcome = copy_public_playlist(
            &conn,
            &endpoint(),
            "  Road Trip  ",
            &items,
            |_| Ok(rss_body(&["first", "second"])),
            Duration::ZERO,
        );
        let outcome = outcome.map_err(|error| anyhow!("{error:?}"))?;
        assert_eq!(outcome.name, "Road Trip");
        assert_eq!(outcome.total, 2);
        assert_eq!(outcome.copied, 2);
        assert_eq!(outcome.skipped(), 0);

        let db = conn.lock().map_err(|_| anyhow!("lock"))?;
        let order: Vec<String> = {
            let mut statement = db.prepare(
                "SELECT t.item_guid FROM playlist_tracks pt
                 JOIN tracks t ON t.id = pt.track_id
                 WHERE pt.playlist_id = ?1 ORDER BY pt.position",
            )?;
            let rows = statement.query_map([outcome.playlist_id], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<String>>>()?
        };
        assert_eq!(order, ["first", "second"]);
        let is_in_library: i64 = db.query_row(
            "SELECT is_in_library FROM tracks WHERE item_guid = 'first'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(is_in_library, 0);
        let files: i64 = db.query_row("SELECT COUNT(*) FROM local_files", [], |row| row.get(0))?;
        assert_eq!(files, 0, "the copy writes no local file row");
        Ok(())
    }

    /// ADR 0089 packet 010: a second copy with the same name is a typed
    /// conflict, and it writes nothing.
    #[test]
    fn adr_0089_copy_to_my_playlists_same_name_conflicts_and_writes_nothing() -> Result<()> {
        let conn = setup()?;
        let items = [remote(1, Some(FEED_URL), Some("first"))];
        let first = copy_public_playlist(
            &conn,
            &endpoint(),
            "Road Trip",
            &items,
            |_| Ok(rss_body(&["first"])),
            Duration::ZERO,
        );
        assert!(first.is_ok());
        let feeds_before: i64 = conn.lock().map_err(|_| anyhow!("lock"))?.query_row(
            "SELECT COUNT(*) FROM feeds",
            [],
            |row| row.get(0),
        )?;

        let reads = Cell::new(0_usize);
        let second = copy_public_playlist(
            &conn,
            &endpoint(),
            "Road Trip",
            &items,
            |_| {
                reads.set(reads.get() + 1);
                Ok(rss_body(&["first"]))
            },
            Duration::ZERO,
        );
        assert!(matches!(second, Err(CopyError::NameTaken(ref name)) if name == "Road Trip"));
        assert_eq!(reads.get(), 0, "a conflict reads no feed");
        let db = conn.lock().map_err(|_| anyhow!("lock"))?;
        let playlists: i64 =
            db.query_row("SELECT COUNT(*) FROM playlists", [], |row| row.get(0))?;
        let feeds_after: i64 = db.query_row("SELECT COUNT(*) FROM feeds", [], |row| row.get(0))?;
        assert_eq!(playlists, 1);
        assert_eq!(feeds_after, feeds_before);
        Ok(())
    }

    /// ADR 0089 packet 010: an item with no item GUID is skipped and
    /// counted. A feed named by several items is read once.
    #[test]
    fn adr_0089_copy_to_my_playlists_counts_skipped_and_reads_each_feed_once() -> Result<()> {
        let conn = setup()?;
        let items = [
            remote(1, Some(FEED_URL), Some("first")),
            remote(2, Some(FEED_URL), None),
            remote(3, Some(FEED_URL), Some("first")),
        ];
        let reads = Cell::new(0_usize);
        let outcome = copy_public_playlist(
            &conn,
            &endpoint(),
            "Mixed",
            &items,
            |_| {
                reads.set(reads.get() + 1);
                Ok(rss_body(&["first"]))
            },
            Duration::ZERO,
        )
        .map_err(|error| anyhow!("{error:?}"))?;
        assert_eq!(reads.get(), 1, "one read for one feed URL");
        assert_eq!(outcome.total, 3);
        assert_eq!(outcome.copied, 2);
        assert_eq!(outcome.skipped(), 1);
        Ok(())
    }

    /// ADR 0089 packet 010: after HTTP 429 from a host, the copy sends no
    /// more requests to that host, and it counts the items as skipped.
    #[test]
    fn adr_0089_copy_to_my_playlists_stops_a_throttled_host() -> Result<()> {
        let conn = setup()?;
        let items = [
            remote(1, Some("https://busy.test/a.xml"), Some("a")),
            remote(2, Some("https://busy.test/b.xml"), Some("b")),
        ];
        let reads = Cell::new(0_usize);
        let outcome = copy_public_playlist(
            &conn,
            &endpoint(),
            "Throttled",
            &items,
            |url| {
                reads.set(reads.get() + 1);
                Err(HostThrottled {
                    host: url.to_owned(),
                    retry_after: None,
                }
                .into())
            },
            Duration::ZERO,
        )
        .map_err(|error| anyhow!("{error:?}"))?;
        assert_eq!(reads.get(), 1);
        assert_eq!(outcome.copied, 0);
        assert_eq!(outcome.skipped(), 2);
        Ok(())
    }
}
