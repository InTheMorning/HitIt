//! UI-agnostic planning for removals from the local library.
//!
//! All user-facing "remove from library" affordances should resolve to this
//! plan before mutating state so playlist playback impact is handled
//! consistently across Library and Discover.

#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

use crate::{audio_cache, db, library_service};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LibraryRemovalIntent {
    TrackId(i64),
    FeedId(i64),
    TrackMatch {
        feed_url: Option<String>,
        item_guid: Option<String>,
        enclosure_url: Option<String>,
    },
    FeedUrl(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LibraryRemovalTarget {
    Track(i64),
    Feed(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LibraryRemovalImpact {
    Track {
        playlist_reference_count: i64,
        show_cue_reference_count: i64,
    },
    Feed {
        playlist_track_count: i64,
        show_cue_track_count: i64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LibraryRemovalPlan {
    target: LibraryRemovalTarget,
    impact: LibraryRemovalImpact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryRemovalExecution {
    target: LibraryRemovalTarget,
    message: &'static str,
    feed_changed: bool,
}

impl LibraryRemovalPlan {
    #[must_use]
    pub const fn new(target: LibraryRemovalTarget, impact: LibraryRemovalImpact) -> Self {
        Self { target, impact }
    }

    #[must_use]
    pub const fn target(self) -> LibraryRemovalTarget {
        self.target
    }

    #[must_use]
    pub const fn impact(self) -> LibraryRemovalImpact {
        self.impact
    }

    #[must_use]
    pub const fn requires_confirmation(self) -> bool {
        match self.impact {
            LibraryRemovalImpact::Track {
                playlist_reference_count,
                show_cue_reference_count,
            } => playlist_reference_count > 0 || show_cue_reference_count > 0,
            LibraryRemovalImpact::Feed {
                playlist_track_count,
                show_cue_track_count,
            } => playlist_track_count > 0 || show_cue_track_count > 0,
        }
    }
}

impl LibraryRemovalExecution {
    #[must_use]
    pub const fn new(
        target: LibraryRemovalTarget,
        message: &'static str,
        feed_changed: bool,
    ) -> Self {
        Self {
            target,
            message,
            feed_changed,
        }
    }

    #[must_use]
    pub const fn target(&self) -> LibraryRemovalTarget {
        self.target
    }

    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.message
    }

    #[must_use]
    pub const fn feed_changed(&self) -> bool {
        self.feed_changed
    }
}

/// Resolve a removal intent to a canonical local target and playlist impact.
///
/// # Errors
///
/// Returns an error when the local target cannot be resolved or playlist
/// impact cannot be queried.
pub fn plan_library_removal(
    conn: &Connection,
    intent: &LibraryRemovalIntent,
) -> Result<LibraryRemovalPlan> {
    match intent {
        LibraryRemovalIntent::TrackId(track_id) => plan_track_removal(conn, *track_id),
        LibraryRemovalIntent::FeedId(feed_id) => plan_feed_removal(conn, *feed_id),
        LibraryRemovalIntent::TrackMatch {
            feed_url,
            item_guid,
            enclosure_url,
        } => {
            let track_id = library_service::find_track_id(
                conn,
                feed_url.as_deref(),
                item_guid.as_deref(),
                enclosure_url.as_deref(),
            )?
            .context("track is not present in the local library")?;
            plan_track_removal(conn, track_id)
        }
        LibraryRemovalIntent::FeedUrl(feed_url) => {
            let feed_id = db::feed_id_by_url(conn, feed_url)?
                .with_context(|| format!("feed is not present in the local library: {feed_url}"))?;
            plan_feed_removal(conn, feed_id)
        }
    }
}

fn plan_track_removal(conn: &Connection, track_id: i64) -> Result<LibraryRemovalPlan> {
    let playlist_reference_count =
        library_service::playlist_reference_count_for_track(conn, track_id)?;
    let show_cue_reference_count = conn.query_row(
        "SELECT COUNT(*) FROM show_cue_entries WHERE track_id = ?1",
        [track_id],
        |row| row.get(0),
    )?;
    Ok(LibraryRemovalPlan::new(
        LibraryRemovalTarget::Track(track_id),
        LibraryRemovalImpact::Track {
            playlist_reference_count,
            show_cue_reference_count,
        },
    ))
}

fn plan_feed_removal(conn: &Connection, feed_id: i64) -> Result<LibraryRemovalPlan> {
    let playlist_track_count =
        library_service::playlist_referenced_library_track_count_for_feed(conn, feed_id)?;
    let show_cue_track_count = conn.query_row(
        "SELECT COUNT(DISTINCT t.id) FROM show_cue_entries e JOIN tracks t ON t.id = e.track_id WHERE t.feed_id = ?1 AND t.is_in_library = 1",
        [feed_id],
        |row| row.get(0),
    )?;
    Ok(LibraryRemovalPlan::new(
        LibraryRemovalTarget::Feed(feed_id),
        LibraryRemovalImpact::Feed {
            playlist_track_count,
            show_cue_track_count,
        },
    ))
}

/// Names each playlist that refers to `target` (ADR 0084 Decision 10):
/// the selection removal confirmation names each one.
///
/// # Errors
///
/// Returns an error when the playlist names cannot be queried.
pub fn removal_referring_playlist_names(
    conn: &Connection,
    target: LibraryRemovalTarget,
) -> Result<Vec<String>> {
    match target {
        LibraryRemovalTarget::Track(track_id) => {
            library_service::playlist_names_referencing_track(conn, track_id)
        }
        LibraryRemovalTarget::Feed(feed_id) => {
            library_service::playlist_names_referencing_feed(conn, feed_id)
        }
    }
}

/// Execute a planned local-library removal target.
///
/// # Errors
///
/// Returns an error when the library target cannot be mutated.
pub fn execute_library_removal(
    conn: &Connection,
    music_dir: &Path,
    audio_cache_dir: &Path,
    target: LibraryRemovalTarget,
) -> Result<LibraryRemovalExecution> {
    match target {
        LibraryRemovalTarget::Track(track_id) => {
            execute_track_removal(conn, music_dir, audio_cache_dir, track_id)
        }
        LibraryRemovalTarget::Feed(feed_id) => {
            execute_feed_removal(conn, music_dir, audio_cache_dir, feed_id)
        }
    }
}

fn execute_track_removal(
    conn: &Connection,
    music_dir: &Path,
    audio_cache_dir: &Path,
    track_id: i64,
) -> Result<LibraryRemovalExecution> {
    let feed_url = match library_service::track_row_by_id(conn, track_id)? {
        Some(track) => db::feed_url_by_id(conn, track.feed_id)?,
        None => None,
    };
    let rows = db::local_audio_files_for_track(conn, track_id)?;
    let paths = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
    let moves = audio_cache::put_files(
        music_dir,
        audio_cache_dir,
        &paths,
        audio_cache::ExistingDestination::Replace,
    )?;
    let result = (|| -> Result<bool> {
        let transaction = conn.unchecked_transaction()?;
        transaction.execute(
            "UPDATE tracks SET is_in_library = 0 WHERE id = ?1",
            [track_id],
        )?;
        transaction.execute(
            "DELETE FROM tag_update_skips WHERE track_id = ?1",
            [track_id],
        )?;
        db::move_local_file_rows_to_cache(&transaction, &rows, |row| {
            moves.iter().any(|moved| moved.relative_path() == &row.path)
        })?;
        if let Some(feed_url) = feed_url.as_deref() {
            db::reconcile_feed_subscription_by_url(&transaction, feed_url)?;
        }
        transaction.commit()?;
        Ok(feed_url.is_some())
    })();
    let feed_changed = match result {
        Ok(changed) => changed,
        Err(error) => {
            audio_cache::undo_put(&moves);
            return Err(error);
        }
    };
    if let Err(error) = audio_cache::remove_sources(&moves, music_dir) {
        eprintln!("ADR 0100: could not remove a moved Library file: {error:#}");
    }
    Ok(LibraryRemovalExecution::new(
        LibraryRemovalTarget::Track(track_id),
        "Removed track",
        feed_changed,
    ))
}

fn execute_feed_removal(
    conn: &Connection,
    music_dir: &Path,
    audio_cache_dir: &Path,
    feed_id: i64,
) -> Result<LibraryRemovalExecution> {
    let track_ids = {
        let mut statement = conn.prepare("SELECT id FROM tracks WHERE feed_id = ?1 ORDER BY id")?;
        let rows = statement
            .query_map([feed_id], |row| row.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    let rows = track_ids
        .iter()
        .map(|id| db::local_audio_files_for_track(conn, *id))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let paths = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
    let moves = audio_cache::put_files(
        music_dir,
        audio_cache_dir,
        &paths,
        audio_cache::ExistingDestination::Replace,
    )?;
    let result = (|| -> Result<()> {
        let transaction = conn.unchecked_transaction()?;
        db::set_feed_subscribed(&transaction, feed_id, false)?;
        db::unsubscribe_feed_tracks(&transaction, feed_id)?;
        transaction.execute(
            "DELETE FROM tag_update_skips WHERE track_id IN (SELECT id FROM tracks WHERE feed_id = ?1)",
            [feed_id],
        )?;
        db::move_local_file_rows_to_cache(&transaction, &rows, |row| {
            moves.iter().any(|moved| moved.relative_path() == &row.path)
        })?;
        transaction.commit()?;
        Ok(())
    })();
    if let Err(error) = result {
        audio_cache::undo_put(&moves);
        return Err(error);
    }
    if let Err(error) = audio_cache::remove_sources(&moves, music_dir) {
        eprintln!("ADR 0100: could not remove a moved Library file: {error:#}");
    }
    Ok(LibraryRemovalExecution::new(
        LibraryRemovalTarget::Feed(feed_id),
        "Removed feed",
        true,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library_path::LibraryRelativePath;

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
    }

    #[test]
    fn removal_plan_resolves_match_to_track_impact() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn, "https://example.test/feed.xml")?;
        let track_id = create_track(&conn, feed_id, "item", "https://example.test/audio.mp3")?;
        library_service::set_track_in_library(&conn, track_id, true)?;
        let playlist_id = db::playlist_create(&conn, "Mix")?;
        db::playlist_append(&conn, playlist_id, track_id)?;

        let plan = plan_library_removal(
            &conn,
            &LibraryRemovalIntent::TrackMatch {
                feed_url: Some("https://example.test/feed.xml".into()),
                item_guid: Some("item".into()),
                enclosure_url: None,
            },
        )?;

        assert_eq!(plan.target(), LibraryRemovalTarget::Track(track_id));
        assert_eq!(
            plan.impact(),
            LibraryRemovalImpact::Track {
                playlist_reference_count: 1,
                show_cue_reference_count: 0
            }
        );
        assert!(plan.requires_confirmation());
        Ok(())
    }

    #[test]
    fn removal_plan_resolves_feed_url_to_feed_impact() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn, "https://example.test/feed.xml")?;
        let track_id = create_track(&conn, feed_id, "item", "https://example.test/audio.mp3")?;
        library_service::set_track_in_library(&conn, track_id, true)?;
        let playlist_id = db::playlist_create(&conn, "Mix")?;
        db::playlist_append(&conn, playlist_id, track_id)?;

        let plan = plan_library_removal(
            &conn,
            &LibraryRemovalIntent::FeedUrl("https://example.test/feed.xml".into()),
        )?;

        assert_eq!(plan.target(), LibraryRemovalTarget::Feed(feed_id));
        assert_eq!(
            plan.impact(),
            LibraryRemovalImpact::Feed {
                playlist_track_count: 1,
                show_cue_track_count: 0
            }
        );
        assert!(plan.requires_confirmation());
        Ok(())
    }

    #[test]
    fn adr_0100_track_removal_moves_file_and_updates_file_rows() -> Result<()> {
        let conn = setup_test_db()?;
        let music = tempfile::tempdir()?;
        let cache = tempfile::tempdir()?;
        let feed_id = create_feed(&conn, "https://example.test/adr-0100-track.xml")?;
        let track_id = create_track(&conn, feed_id, "track", "https://example.test/track.mp3")?;
        library_service::set_track_in_library(&conn, track_id, true)?;
        let relative = LibraryRelativePath::from_stored("artists/a/release/track.mp3")?;
        let source = relative.resolve(music.path());
        std::fs::create_dir_all(source.parent().unwrap())?;
        std::fs::write(&source, b"ADR 0100 audio")?;
        db::mark_track_downloaded(&conn, track_id, &relative, Some(14))?;
        db::tag_update_skips::record_tag_update_skips(
            &conn,
            &[db::tag_update_skips::TagUpdateSkip {
                track_id,
                frame_label: "TIT2".into(),
                stored_value: "Track Title".into(),
            }],
        )?;

        execute_library_removal(
            &conn,
            music.path(),
            cache.path(),
            LibraryRemovalTarget::Track(track_id),
        )?;

        let destination = relative.resolve(cache.path());
        assert!(!source.exists(), "ADR 0100 removes the file from music_dir");
        assert_eq!(std::fs::read(destination)?, b"ADR 0100 audio");
        let local_count: i64 = conn.query_row(
            "SELECT count(*) FROM local_files WHERE track_id = ?1",
            [track_id],
            |row| row.get(0),
        )?;
        let cached_count: i64 = conn.query_row(
            "SELECT count(*) FROM cached_files WHERE track_id = ?1",
            [track_id],
            |row| row.get(0),
        )?;
        assert_eq!(local_count, 0, "ADR 0100 leaves no local_files row");
        assert_eq!(cached_count, 1, "ADR 0100 adds one cached_files row");
        assert!(db::tag_update_skips::tag_update_skips(&conn)?.is_empty());
        Ok(())
    }

    #[test]
    fn adr_0100_feed_removal_moves_each_file_and_cleans_empty_folders() -> Result<()> {
        let conn = setup_test_db()?;
        let music = tempfile::tempdir()?;
        let cache = tempfile::tempdir()?;
        let feed_id = create_feed(&conn, "https://example.test/adr-0100-release.xml")?;
        let paths = ["artists/a/release/one.mp3", "artists/a/release/two.mp3"];
        for (index, value) in paths.iter().enumerate() {
            let track_id = create_track(
                &conn,
                feed_id,
                &format!("track-{index}"),
                &format!("https://example.test/{index}.mp3"),
            )?;
            library_service::set_track_in_library(&conn, track_id, true)?;
            let relative = LibraryRelativePath::from_stored(*value)?;
            let source = relative.resolve(music.path());
            std::fs::create_dir_all(source.parent().unwrap())?;
            std::fs::write(&source, [u8::try_from(index)?])?;
            db::mark_track_downloaded(&conn, track_id, &relative, Some(1))?;
        }

        execute_library_removal(
            &conn,
            music.path(),
            cache.path(),
            LibraryRemovalTarget::Feed(feed_id),
        )?;

        for value in paths {
            let relative = LibraryRelativePath::from_stored(value)?;
            assert!(
                !relative.resolve(music.path()).exists(),
                "ADR 0100 removes each source"
            );
            assert!(
                relative.resolve(cache.path()).is_file(),
                "ADR 0100 caches each file"
            );
        }
        assert!(
            !music.path().join("artists").exists(),
            "ADR 0100 removes empty parents"
        );
        let cached_count: i64 =
            conn.query_row("SELECT count(*) FROM cached_files", [], |row| row.get(0))?;
        assert_eq!(cached_count, 2, "ADR 0100 caches both release tracks");
        Ok(())
    }

    #[test]
    fn adr_0100_failed_put_keeps_rows_and_source_file() -> Result<()> {
        let conn = setup_test_db()?;
        let music = tempfile::tempdir()?;
        let blocker = tempfile::NamedTempFile::new()?;
        let feed_id = create_feed(&conn, "https://example.test/adr-0100-failure.xml")?;
        let track_id = create_track(&conn, feed_id, "track", "https://example.test/track.mp3")?;
        library_service::set_track_in_library(&conn, track_id, true)?;
        let relative = LibraryRelativePath::from_stored("track.mp3")?;
        let source = relative.resolve(music.path());
        std::fs::write(&source, b"audio")?;
        db::mark_track_downloaded(&conn, track_id, &relative, Some(5))?;

        let error = execute_library_removal(
            &conn,
            music.path(),
            blocker.path(),
            LibraryRemovalTarget::Track(track_id),
        )
        .expect_err("ADR 0100 rejects a cache path that is a file");

        assert!(error.to_string().contains("create audio-cache folder"));
        assert!(
            source.is_file(),
            "ADR 0100 keeps the source after step 1 fails"
        );
        assert!(db::track_row_by_id(&conn, track_id)?.unwrap().is_in_library);
        let local_count: i64 =
            conn.query_row("SELECT count(*) FROM local_files", [], |row| row.get(0))?;
        let cached_count: i64 =
            conn.query_row("SELECT count(*) FROM cached_files", [], |row| row.get(0))?;
        assert_eq!(
            local_count, 1,
            "ADR 0100 keeps local_files after step 1 fails"
        );
        assert_eq!(
            cached_count, 0,
            "ADR 0100 adds no cached_files row after failure"
        );
        Ok(())
    }

    #[test]
    fn adr_0100_missing_source_deletes_local_row_without_cache_row() -> Result<()> {
        let conn = setup_test_db()?;
        let music = tempfile::tempdir()?;
        let cache = tempfile::tempdir()?;
        let feed_id = create_feed(&conn, "https://example.test/adr-0100-missing.xml")?;
        let track_id = create_track(&conn, feed_id, "track", "https://example.test/track.mp3")?;
        library_service::set_track_in_library(&conn, track_id, true)?;
        let relative = LibraryRelativePath::from_stored("missing.mp3")?;
        db::mark_track_downloaded(&conn, track_id, &relative, None)?;

        execute_library_removal(
            &conn,
            music.path(),
            cache.path(),
            LibraryRemovalTarget::Track(track_id),
        )?;

        let local_count: i64 =
            conn.query_row("SELECT count(*) FROM local_files", [], |row| row.get(0))?;
        let cached_count: i64 =
            conn.query_row("SELECT count(*) FROM cached_files", [], |row| row.get(0))?;
        assert_eq!(local_count, 0, "ADR 0100 deletes a missing local_files row");
        assert_eq!(
            cached_count, 0,
            "ADR 0100 adds no cache row for a missing file"
        );
        Ok(())
    }

    /// R30-08 (ADR 0084 task 030, decision D4): the count of the removal
    /// message equals the number of playlist names that the dialog lists.
    #[test]
    fn adr_0084_music_open_defects_r30_08_count_equals_listed_names() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn, "https://example.test/feed.xml")?;
        let twice = create_track(&conn, feed_id, "twice", "https://example.test/1.mp3")?;
        let spread = create_track(&conn, feed_id, "spread", "https://example.test/2.mp3")?;
        let none = create_track(&conn, feed_id, "none", "https://example.test/3.mp3")?;
        let first = db::playlist_create(&conn, "First")?;
        let second = db::playlist_create(&conn, "Second")?;
        db::playlist_append(&conn, first, twice)?;
        db::playlist_append(&conn, first, twice)?;
        db::playlist_append(&conn, first, spread)?;
        db::playlist_append(&conn, second, spread)?;

        for (track_id, expected) in [(twice, 1), (spread, 2), (none, 0)] {
            let plan = plan_track_removal(&conn, track_id)?;
            let LibraryRemovalImpact::Track {
                playlist_reference_count,
                ..
            } = plan.impact()
            else {
                panic!("a track removal has a track impact");
            };
            let names =
                removal_referring_playlist_names(&conn, LibraryRemovalTarget::Track(track_id))?;
            assert_eq!(playlist_reference_count, expected);
            assert_eq!(usize::try_from(playlist_reference_count)?, names.len());
        }
        Ok(())
    }

    fn create_feed(conn: &Connection, feed_url: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO feeds (feed_url, feed_guid, title)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![feed_url, "feed-guid", "Feed Title"],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_track(
        conn: &Connection,
        feed_id: i64,
        item_guid: &str,
        enclosure_url: &str,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, enclosure_url, track_title)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![feed_id, item_guid, enclosure_url, "Track Title"],
        )?;
        Ok(conn.last_insert_rowid())
    }
}
