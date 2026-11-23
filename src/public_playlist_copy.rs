//! Copies a public playlist of the Index into a local playlist (ADR 0089
//! Decision 1, packet 010).
//!
//! The copy downloads no file. The `FeedReader` of `musicindex-fetch` reads
//! each remote feed one time and finds the item (musicindex ADR 0092
//! R-STATION-1 item 5). When an item has no feed URL, the reader gets it
//! from `MusicIndex` by the feed GUID (R-STATION-1 item 3). The reader keeps
//! the minimum interval between two requests to one host, and it sends no
//! more requests to a host that answers HTTP 429.
//!
//! `HitIt` then stores the body through the RSS path of a subscribe,
//! `rss::subscribe_feed_body`. That path makes the `feeds` and `tracks`
//! rows, with `is_in_library` false (ADR 0075 provenance).
//!
//! The remote items come from `MusicIndex`, so they are untrusted input.
//! The database lock is never held during an HTTP request.

#![warn(clippy::pedantic)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Context as _, Result};
use musicindex_fetch::{FeedReader, FetchDocument, ReaderConfig, RemoteItemRef};
use rusqlite::Connection;

use crate::api::RemoteItem;
use crate::config::MusicIndexEndpoint;
use crate::db;
use crate::playlist_service;
use crate::rss;

/// Why a copy did not finish.
#[derive(Debug)]
pub enum CopyError {
    /// A playlist with the name already exists. Nothing was written.
    NameTaken(String),
    /// No item of the playlist gave a readable track, so no playlist was
    /// made. An empty copy gives the curator nothing to use.
    NoReadableTracks,
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
/// `fetch` sends one GET request. It is `http_client::feed_fetch()` in the
/// app. A test gives a closure. `min_interval` is the wait between two
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
    fetch: impl FetchDocument,
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

    let musicindex_base_url = endpoint.require().ok().map(str::to_owned);
    let config = ReaderConfig {
        min_interval,
        musicindex_base_url: musicindex_base_url.clone().unwrap_or_default(),
        user_agent: crate::http_client::USER_AGENT.to_owned(),
    };
    let mut copy = Copy {
        conn,
        endpoint,
        can_look_up_feed_urls: musicindex_base_url.is_some(),
        reader: FeedReader::with_fetch(fetch, config),
        stored: HashMap::new(),
    };
    let mut track_ids = Vec::new();
    for item in items {
        let resolved = copy.resolve_item(item).map_err(CopyError::Failed)?;
        track_ids.extend(resolved);
    }

    if track_ids.is_empty() {
        return Err(CopyError::NoReadableTracks);
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

/// The state of one copy: the crate reader and the store result of each
/// feed URL.
struct Copy<'a, F> {
    conn: &'a Arc<Mutex<Connection>>,
    endpoint: &'a MusicIndexEndpoint,
    /// `false` when the `MusicIndex` endpoint is not valid. Then an item with
    /// no feed URL is skipped.
    can_look_up_feed_urls: bool,
    reader: FeedReader<F>,
    /// `true` for a feed URL whose rows were stored in this copy.
    stored: HashMap<String, bool>,
}

impl<F: FetchDocument> Copy<'_, F> {
    /// The track of one remote item, or `None` when the item names no
    /// track that the copy can read. An item with no item GUID is skipped.
    fn resolve_item(&mut self, item: &RemoteItem) -> Result<Option<i64>> {
        let Some(item_guid) = non_empty(item.remote_item_guid.as_deref()) else {
            return Ok(None);
        };
        let feed_guid = non_empty(item.remote_feed_guid.as_deref());
        if let Some(feed_guid) = feed_guid {
            // The database lookup comes before each request.
            let db = lock(self.conn)?;
            if let Some(id) =
                db::find_track_id_by_feed_guid_and_item_guid(&db, feed_guid, item_guid)?
            {
                return Ok(Some(id));
            }
        }
        let feed_url = non_empty(item.remote_feed_url.as_deref());
        if feed_url.is_none() && (feed_guid.is_none() || !self.can_look_up_feed_urls) {
            return Ok(None);
        }
        let reference = RemoteItemRef {
            feed_guid: feed_guid.unwrap_or_default().to_owned(),
            feed_url: feed_url.map(str::to_owned),
            item_guid: Some(item_guid.to_owned()),
            title: None,
        };
        let Ok(resolved) = self.reader.resolve(&reference) else {
            return Ok(None);
        };
        if !self.store_once(&resolved.feed_url)? {
            return Ok(None);
        }
        let db = lock(self.conn)?;
        db::find_track_id(&db, Some(&resolved.feed_url), Some(item_guid), None)
    }

    /// Stores the feed at `feed_url` one time in this copy. Gives `true`
    /// when its rows are stored.
    fn store_once(&mut self, feed_url: &str) -> Result<bool> {
        if let Some(stored) = self.stored.get(feed_url) {
            return Ok(*stored);
        }
        // The reader keeps each document, so this read sends no request.
        let stored = match self.reader.read(feed_url) {
            Ok(document) => store_document(self.conn, self.endpoint, feed_url, &document.body)?,
            Err(_) => false,
        };
        self.stored.insert(feed_url.to_owned(), stored);
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
    // The MusicIndex read runs before the lock (ADR 0084 Decision 10 as
    // amended 2026-10-24, task 039).
    let updated_at = rss::musicindex_updated_at_for_body(body, endpoint);
    let mut db = lock(conn)?;
    let prior_subscribed = db::feed_is_subscribed_by_url(&db, feed_url).unwrap_or(false);
    match rss::subscribe_feed_body(&mut db, feed_url, body, updated_at) {
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
    use musicindex_fetch::{FeedError, HostThrottled};
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
    /// Walk of 2026-11-06: a copy with no readable track makes no playlist.
    #[test]
    fn adr_0089_copy_with_no_readable_track_makes_no_playlist() -> Result<()> {
        let conn = setup()?;
        let result = copy_public_playlist(
            &conn,
            &endpoint(),
            "Nothing",
            &[remote(1, None, None)],
            |url: &str| {
                Err(FeedError::Transport {
                    url: url.to_owned(),
                    message: "no fetch expected".to_owned(),
                })
            },
            Duration::ZERO,
        );
        assert!(matches!(result, Err(CopyError::NoReadableTracks)));
        assert!(!name_taken(&conn, "Nothing")?);
        Ok(())
    }

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
            |_: &str| Ok(rss_body(&["first", "second"])),
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
            |_: &str| Ok(rss_body(&["first"])),
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
            |_: &str| {
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
            |_: &str| {
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
        let result = copy_public_playlist(
            &conn,
            &endpoint(),
            "Throttled",
            &items,
            |_: &str| {
                reads.set(reads.get() + 1);
                Err(FeedError::Throttled(HostThrottled {
                    host: "busy.test".to_owned(),
                    retry_after: None,
                }))
            },
            Duration::ZERO,
        );
        assert_eq!(reads.get(), 1);
        assert!(matches!(result, Err(CopyError::NoReadableTracks)));
        assert!(!name_taken(&conn, "Throttled")?);
        Ok(())
    }

    /// musicindex ADR 0092 R-STATION-1 items 3 and 5: an item with no feed
    /// URL gets the URL from `MusicIndex` by the feed GUID. Then the copy
    /// reads and stores that feed.
    #[test]
    fn adr_0092_copy_gets_a_missing_feed_url_from_musicindex() -> Result<()> {
        let conn = setup()?;
        let lookup = "https://index.test/v1/feeds/album-guid";
        let requests = std::cell::RefCell::new(Vec::new());
        let outcome = copy_public_playlist(
            &conn,
            &MusicIndexEndpoint::from("https://index.test"),
            "Looked Up",
            &[remote(1, None, Some("first"))],
            |url: &str| {
                requests.borrow_mut().push(url.to_owned());
                if url == lookup {
                    Ok(format!(r#"{{"data":{{"feed_url":"{FEED_URL}"}}}}"#).into_bytes())
                } else {
                    Ok(rss_body(&["first"]))
                }
            },
            Duration::ZERO,
        )
        .map_err(|error| anyhow!("{error:?}"))?;
        assert_eq!(outcome.copied, 1);
        assert_eq!(*requests.borrow(), [lookup, FEED_URL]);
        Ok(())
    }

    /// musicindex ADR 0092 R-STATION-1 item 5: when the feed has no item
    /// with the item GUID, the item is skipped.
    #[test]
    fn adr_0092_copy_skips_an_item_that_is_not_in_its_feed() -> Result<()> {
        let conn = setup()?;
        let result = copy_public_playlist(
            &conn,
            &endpoint(),
            "Missing",
            &[remote(1, Some(FEED_URL), Some("absent"))],
            |_: &str| Ok(rss_body(&["first"])),
            Duration::ZERO,
        );
        assert!(matches!(result, Err(CopyError::NoReadableTracks)));
        Ok(())
    }
}
