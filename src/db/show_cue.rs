//! The stored Show cue (ADR 0068 task 002).

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

const DDL: &str = r"
CREATE TABLE show_cue (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    selected_entry_id INTEGER,
    loaded_at_us INTEGER NOT NULL,
    source_playlist_id INTEGER NOT NULL,
    source_playlist_name TEXT NOT NULL
);
CREATE TABLE show_cue_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    position INTEGER NOT NULL,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE RESTRICT,
    source_playlist_id INTEGER NOT NULL,
    source_position INTEGER NOT NULL,
    UNIQUE(position)
);
ALTER TABLE playback_sessions ADD COLUMN cue_entry_id INTEGER;
";

pub(super) const PLAYBACK_SESSION_COLUMNS: &(&str, &[&str]) = &(
    "playback_sessions",
    &[
        "session_id",
        "sequence",
        "local_track_id",
        "playlist_id",
        "playlist_position",
        "started_at",
        "position_ms",
        "state",
        "updated_at",
        "cue_entry_id",
    ],
);

pub(super) const COLUMNS: &[(&str, &[&str])] = &[
    (
        "show_cue",
        &[
            "id",
            "selected_entry_id",
            "loaded_at_us",
            "source_playlist_id",
            "source_playlist_name",
        ],
    ),
    (
        "show_cue_entries",
        &[
            "id",
            "position",
            "track_id",
            "source_playlist_id",
            "source_position",
        ],
    ),
];

pub(super) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
        .context("Create the stored Show cue")?;
    migrate_rss_check_triggers(conn)
}

/// Adds `cue_load` to the CHECK of `rss_check_runs`. SQLite changes a CHECK
/// only by a rebuild. The registry transaction keeps foreign keys on, so the
/// drop of `rss_check_runs` deletes or clears the rows of its child tables.
/// The rebuild sets each child row aside first and restores it after, as
/// migration 23 does for `saved_search_seen`.
fn migrate_rss_check_triggers(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TEMP TABLE rss_check_differences_25 AS SELECT * FROM rss_check_differences;
         CREATE TEMP TABLE rss_field_hold_runs_25 AS
             SELECT rowid AS hold_rowid, run_id FROM rss_field_holds WHERE run_id IS NOT NULL;
         CREATE TEMP TABLE rss_check_feed_results_25 AS SELECT * FROM rss_check_feed_results;
         DROP TABLE rss_check_feed_results;
         DROP INDEX rss_check_runs_playlist;
         CREATE TABLE rss_check_runs_new (
             id INTEGER NOT NULL PRIMARY KEY,
             playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
             trigger TEXT NOT NULL CHECK (trigger IN ('button', 'playback_start', 'cue_load')),
             started_at_us INTEGER NOT NULL,
             finished_at_us INTEGER NULL,
             document_count INTEGER NOT NULL DEFAULT 0 CHECK (document_count >= 0),
             not_modified_count INTEGER NOT NULL DEFAULT 0 CHECK (not_modified_count >= 0),
             failed_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_count >= 0),
             not_checked_count INTEGER NOT NULL DEFAULT 0 CHECK (not_checked_count >= 0),
             CHECK (finished_at_us IS NULL OR finished_at_us >= started_at_us)
         );
         INSERT INTO rss_check_runs_new SELECT * FROM rss_check_runs;
         DROP TABLE rss_check_runs;
         ALTER TABLE rss_check_runs_new RENAME TO rss_check_runs;
         CREATE INDEX rss_check_runs_playlist ON rss_check_runs(playlist_id, id);
         CREATE TABLE rss_check_feed_results (
             run_id INTEGER NOT NULL REFERENCES rss_check_runs(id) ON DELETE CASCADE,
             feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
             outcome TEXT NOT NULL CHECK (outcome IN ('document', 'not_modified', 'failed', 'not_checked')),
             http_status INTEGER NULL CHECK (http_status IS NULL OR http_status BETWEEN 100 AND 599),
             observation_id INTEGER NULL REFERENCES metadata_observations(id) ON DELETE RESTRICT,
             message TEXT NULL,
             PRIMARY KEY (run_id, feed_id)
         );
         INSERT INTO rss_check_feed_results SELECT * FROM rss_check_feed_results_25;
         DROP TABLE rss_check_feed_results_25;
         DELETE FROM rss_check_differences;
         INSERT INTO rss_check_differences SELECT * FROM rss_check_differences_25;
         DROP TABLE rss_check_differences_25;
         UPDATE rss_field_holds
             SET run_id = (SELECT run_id FROM rss_field_hold_runs_25 WHERE hold_rowid = rss_field_holds.rowid)
             WHERE rowid IN (SELECT hold_rowid FROM rss_field_hold_runs_25);
         DROP TABLE rss_field_hold_runs_25;",
    )
    .context("Add the Show cue RSS trigger and retain check history")
}

#[derive(Clone, Debug)]
pub struct CueEntry {
    pub id: i64,
    pub position: i64,
    pub track_id: i64,
    pub source_playlist_id: i64,
    pub source_position: i64,
    pub track: super::TrackRow,
    pub unavailable_reason: Option<CueUnavailableReason>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CueUnavailableReason {
    NotInLibrary,
    NotDownloaded,
}

#[derive(Clone, Debug)]
pub struct ShowCue {
    pub selected_entry_id: Option<i64>,
    pub loaded_at_us: i64,
    pub source_playlist_id: i64,
    pub source_playlist_name: String,
    pub entries: Vec<CueEntry>,
}

pub fn show_cue(conn: &Connection) -> Result<Option<ShowCue>> {
    let Some((selected_entry_id, loaded_at_us, source_playlist_id, source_playlist_name)) = conn
        .query_row(
            "SELECT selected_entry_id, loaded_at_us, source_playlist_id, source_playlist_name FROM show_cue WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .context("Read the stored Show cue")?
    else {
        return Ok(None);
    };
    let mut statement = conn.prepare(
        "SELECT e.id, e.position, e.track_id, e.source_playlist_id, e.source_position,
                t.is_in_library
         FROM show_cue_entries e
         JOIN tracks t ON t.id = e.track_id
         ORDER BY e.position",
    )?;
    let entries = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, bool>(5)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let entries = entries
        .into_iter()
        .map(
            |(id, position, track_id, source_playlist_id, source_position, in_library)| {
                // R1: the file of a cue entry is the file that
                // `track_row_by_id` selects for the track.
                let track = super::track_row_by_id(conn, track_id)?
                    .context("Show cue track disappeared")?;
                let unavailable_reason = if !in_library {
                    Some(CueUnavailableReason::NotInLibrary)
                } else if track.local_path.is_none() {
                    Some(CueUnavailableReason::NotDownloaded)
                } else {
                    None
                };
                Ok(CueEntry {
                    id,
                    position,
                    track_id,
                    source_playlist_id,
                    source_position,
                    track,
                    unavailable_reason,
                })
            },
        )
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(ShowCue {
        selected_entry_id,
        loaded_at_us,
        source_playlist_id,
        source_playlist_name,
        entries,
    }))
}

pub fn show_cue_exists(conn: &Connection) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM show_cue WHERE id = 1)",
        [],
        |row| row.get(0),
    )
    .context("Check whether a Show cue exists")
}

/// Returns whether ADR 0068 allows a Show cue change in the current session.
pub fn show_cue_can_change(conn: &Connection) -> Result<bool> {
    let session_state: Option<String> = conn
        .query_row(
            "SELECT state FROM playback_sessions WHERE session_id = 'default'",
            [],
            |row| row.get(0),
        )
        .optional()
        .context("Read the playback state for a Show cue change")?;
    Ok(session_state.is_none_or(|state| state == "stopped"))
}

pub fn load_show_cue(
    conn: &mut Connection,
    playlist_id: i64,
    loaded_at_us: i64,
    replace: bool,
) -> Result<bool> {
    let tx = conn.transaction().context("Begin the Show cue load")?;
    if replace && !show_cue_can_change(&tx)? {
        anyhow::bail!("The Show cue cannot change while playback is active");
    }
    let playlist = tx
        .query_row(
            "SELECT name FROM playlists WHERE id = ?1",
            [playlist_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .context("Playlist does not exist")?;
    let rows = {
        let mut statement = tx.prepare("SELECT track_id, position FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position")?;
        let rows = statement
            .query_map([playlist_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    if rows.is_empty() {
        tx.rollback()?;
        return Ok(false);
    }
    if !replace && show_cue_exists(&tx)? {
        anyhow::bail!("A Show cue already exists");
    }
    tx.execute("DELETE FROM show_cue_entries", [])?;
    tx.execute("DELETE FROM show_cue", [])?;
    tx.execute(
        "UPDATE playback_sessions SET cue_entry_id = NULL WHERE cue_entry_id IS NOT NULL",
        [],
    )?;
    tx.execute("INSERT INTO show_cue(id, loaded_at_us, source_playlist_id, source_playlist_name) VALUES (1, ?1, ?2, ?3)", params![loaded_at_us, playlist_id, playlist])?;
    for (position, (track_id, source_position)) in rows.iter().enumerate() {
        tx.execute("INSERT INTO show_cue_entries(position, track_id, source_playlist_id, source_position) VALUES (?1, ?2, ?3, ?4)", params![i64::try_from(position)?, track_id, playlist_id, source_position])?;
    }
    let first_entry: i64 = tx.query_row(
        "SELECT id FROM show_cue_entries ORDER BY position LIMIT 1",
        [],
        |row| row.get(0),
    )?;
    tx.execute(
        "UPDATE show_cue SET selected_entry_id = ?1 WHERE id = 1",
        [first_entry],
    )?;
    tx.commit().context("Commit the Show cue load")?;
    Ok(true)
}

pub fn clear_show_cue(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction().context("Begin the Show cue clear")?;
    anyhow::ensure!(
        show_cue_can_change(&tx)?,
        "The Show cue cannot clear while playback is active"
    );
    tx.execute("DELETE FROM show_cue_entries", [])?;
    tx.execute("DELETE FROM show_cue", [])?;
    tx.execute(
        "UPDATE playback_sessions SET cue_entry_id = NULL WHERE cue_entry_id IS NOT NULL",
        [],
    )?;
    tx.commit().context("Commit the Show cue clear")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
    }

    fn playlist_with_repeated_track(conn: &Connection) -> Result<(i64, i64)> {
        conn.execute(
            "INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/feed', 'cue-feed', 'Cue feed')",
            [],
        )?;
        let feed_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, 'entry', 'Cue track', 1)",
            [feed_id],
        )?;
        let track_id = conn.last_insert_rowid();
        let playlist_id = db::playlist_create(conn, "Cue source")?;
        db::playlist_append(conn, playlist_id, track_id)?;
        db::playlist_append(conn, playlist_id, track_id)?;
        Ok((playlist_id, track_id))
    }

    /// Migration 25 rebuilds `rss_check_runs`. The rebuild keeps each row of
    /// its child tables, with foreign keys on as in the registry.
    #[test]
    fn migration_25_keeps_the_rss_check_history() -> Result<()> {
        let conn = setup()?;
        let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
        let feed_id: i64 = conn.query_row(
            "SELECT feed_id FROM tracks WHERE id = ?1",
            [track_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO rss_check_runs(id, playlist_id, trigger, started_at_us) VALUES (7, ?1, 'button', 1)",
            [playlist_id],
        )?;
        conn.execute(
            "INSERT INTO rss_check_feed_results(run_id, feed_id, outcome) VALUES (7, ?1, 'document')",
            [feed_id],
        )?;
        conn.execute(
            "INSERT INTO rss_check_differences(run_id, feed_id, field, kind, recorded_at_us) VALUES (7, ?1, 'title', 'changed', 1)",
            [feed_id],
        )?;
        conn.execute(
            "INSERT INTO rss_field_holds(owner_kind, feed_id, field, run_id, checked_at_us) VALUES ('feed', ?1, 'title', 7, 1)",
            [feed_id],
        )?;

        let transaction = conn.unchecked_transaction()?;
        migrate_rss_check_triggers(&transaction)?;
        transaction.commit()?;

        let count = |table: &str| -> Result<i64> {
            Ok(conn.query_row(
                &format!("SELECT count(*) FROM {table} WHERE run_id = 7"),
                [],
                |row| row.get(0),
            )?)
        };
        let runs: i64 = conn.query_row(
            "SELECT count(*) FROM rss_check_runs WHERE id = 7",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(runs, 1);
        assert_eq!(count("rss_check_feed_results")?, 1);
        assert_eq!(count("rss_check_differences")?, 1);
        assert_eq!(count("rss_field_holds")?, 1);
        Ok(())
    }

    #[test]
    fn cue_load_keeps_duplicate_entries_and_survives_source_delete() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
        assert!(load_show_cue(&mut conn, playlist_id, 123, false)?);
        let cue = show_cue(&conn)?.expect("the cue is stored");
        assert_eq!(cue.entries.len(), 2);
        assert_ne!(cue.entries[0].id, cue.entries[1].id);
        assert_eq!(cue.entries[0].track_id, track_id);
        assert_eq!(cue.entries[1].track_id, track_id);
        db::playlist_delete(&conn, playlist_id)?;
        assert_eq!(
            show_cue(&conn)?
                .expect("source deletion keeps cue")
                .entries
                .len(),
            2
        );
        Ok(())
    }

    #[test]
    fn empty_and_failed_loads_keep_the_existing_cue() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, _) = playlist_with_repeated_track(&conn)?;
        load_show_cue(&mut conn, playlist_id, 123, false)?;
        let empty_id = db::playlist_create(&conn, "Empty")?;
        assert!(!load_show_cue(&mut conn, empty_id, 456, true)?);
        assert!(load_show_cue(&mut conn, i64::MAX, 456, true).is_err());
        assert_eq!(
            show_cue(&conn)?.expect("cue remains").source_playlist_id,
            playlist_id
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM playback_sessions", [], |row| row
                .get::<_, i64>(0))?,
            0
        );
        Ok(())
    }

    #[test]
    fn replace_and_clear_require_stopped_playback() -> Result<()> {
        for state in ["playing", "paused"] {
            let mut conn = setup()?;
            let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
            load_show_cue(&mut conn, playlist_id, 123, false)?;
            db::set_playback_session_track(
                &conn,
                crate::playback::DEFAULT_SESSION_ID,
                track_id,
                None,
                None,
                "started",
                0,
            )?;
            conn.execute(
                "UPDATE playback_sessions SET state = ?1 WHERE session_id = ?2",
                params![state, crate::playback::DEFAULT_SESSION_ID],
            )?;
            assert!(load_show_cue(&mut conn, playlist_id, 456, true).is_err());
            assert!(clear_show_cue(&mut conn).is_err());
            assert!(show_cue(&conn)?.is_some());
        }
        Ok(())
    }

    #[test]
    fn stopped_replace_succeeds_and_clear_removes_the_cue() -> Result<()> {
        let mut conn = setup()?;
        let (first_playlist, _) = playlist_with_repeated_track(&conn)?;
        let second_playlist = db::playlist_create(&conn, "Second")?;
        let track_id: i64 =
            conn.query_row("SELECT id FROM tracks LIMIT 1", [], |row| row.get(0))?;
        db::playlist_append(&conn, second_playlist, track_id)?;

        assert!(load_show_cue(&mut conn, first_playlist, 123, false)?);
        assert!(load_show_cue(&mut conn, second_playlist, 456, true)?);
        assert_eq!(
            show_cue(&conn)?.expect("cue replaced").source_playlist_id,
            second_playlist
        );
        clear_show_cue(&mut conn)?;
        assert!(show_cue(&conn)?.is_none());
        Ok(())
    }

    #[test]
    fn replace_and_clear_remove_the_session_cue_entry() -> Result<()> {
        for clear in [false, true] {
            let mut conn = setup()?;
            let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
            load_show_cue(&mut conn, playlist_id, 1, false)?;
            let entry_id = show_cue(&conn)?.expect("cue stored").entries[0].id;
            db::set_playback_session_track_with_cue(
                &conn,
                "default",
                db::PlaybackSessionTrackUpdate {
                    local_track_id: track_id,
                    playlist_id: None,
                    playlist_position: None,
                    started_at: "started",
                    position_ms: 0,
                    cue_entry_id: Some(entry_id),
                },
            )?;
            conn.execute(
                "UPDATE playback_sessions SET state = 'stopped' WHERE session_id = 'default'",
                [],
            )?;

            if clear {
                clear_show_cue(&mut conn)?;
            } else {
                load_show_cue(&mut conn, playlist_id, 2, true)?;
            }

            let session = db::playback_session(&conn, "default")?.expect("session stored");
            assert_eq!(session.cue_entry_id, None);
            assert_eq!(session.local_track_id, track_id);
        }
        Ok(())
    }

    #[test]
    fn track_write_without_a_cue_entry_clears_the_session_cue_entry() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
        load_show_cue(&mut conn, playlist_id, 1, false)?;
        let entry_id = show_cue(&conn)?.expect("cue stored").entries[0].id;
        db::set_playback_session_track_with_cue(
            &conn,
            "default",
            db::PlaybackSessionTrackUpdate {
                local_track_id: track_id,
                playlist_id: None,
                playlist_position: None,
                started_at: "started",
                position_ms: 0,
                cue_entry_id: Some(entry_id),
            },
        )?;
        let session = db::playback_session(&conn, "default")?.expect("session stored");
        assert_eq!(session.cue_entry_id, Some(entry_id));

        // The CLI simulator writes a track with no cue entry.
        db::set_playback_session_track(&conn, "default", track_id, None, None, "later", 0)?;
        let session = db::playback_session(&conn, "default")?.expect("session stored");
        assert_eq!(session.cue_entry_id, None);
        Ok(())
    }

    #[test]
    fn playback_session_reader_hides_a_missing_cue_entry() -> Result<()> {
        let conn = setup()?;
        let (_, track_id) = playlist_with_repeated_track(&conn)?;
        db::set_playback_session_track(&conn, "default", track_id, None, None, "started", 0)?;
        conn.execute(
            "UPDATE playback_sessions SET cue_entry_id = 999999 WHERE session_id = 'default'",
            [],
        )?;

        let session = db::playback_session(&conn, "default")?.expect("session stored");
        assert_eq!(session.cue_entry_id, None);
        Ok(())
    }

    #[test]
    fn restart_stop_preserves_track_position_and_observation_time() -> Result<()> {
        let conn = setup()?;
        let (_, track_id) = playlist_with_repeated_track(&conn)?;
        db::set_playback_session_track(&conn, "default", track_id, None, None, "started", 812)?;
        conn.execute(
            "UPDATE playback_sessions SET state = 'paused', updated_at = 'fixed observation' WHERE session_id = 'default'",
            [],
        )?;

        let session = db::stop_playback_session_after_restart(&conn, "default")?
            .expect("session remains stored");
        assert_eq!(session.state, "stopped");
        assert_eq!(session.local_track_id, track_id);
        assert_eq!(session.position_ms, 812);
        assert_eq!(session.started_at, "started");
        let observed_at: String = conn.query_row(
            "SELECT updated_at FROM playback_sessions WHERE session_id = 'default'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(observed_at, "fixed observation");
        Ok(())
    }

    #[test]
    fn cue_order_survives_source_edits_and_database_reopen() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("show-cue.sqlite");
        let mut conn = Connection::open(&path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        conn.execute(
            "INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/order', 'order-feed', 'Order feed')",
            [],
        )?;
        let feed_id = conn.last_insert_rowid();
        let first = insert_track(&conn, feed_id, "first")?;
        let second = insert_track(&conn, feed_id, "second")?;
        let playlist = db::playlist_create(&conn, "Order")?;
        db::playlist_append(&conn, playlist, first)?;
        db::playlist_append(&conn, playlist, second)?;
        load_show_cue(&mut conn, playlist, 100, false)?;

        db::playlist_rename(&conn, playlist, "Renamed")?;
        db::playlist_reorder(&mut conn, playlist, 0, 1)?;
        db::playlist_delete(&conn, playlist)?;
        drop(conn);

        let conn = Connection::open(&path)?;
        let cue = show_cue(&conn)?.expect("cue survives database reopen");
        assert_eq!(
            cue.entries
                .iter()
                .map(|entry| entry.track_id)
                .collect::<Vec<_>>(),
            [first, second]
        );
        Ok(())
    }

    #[test]
    fn removal_and_show_usage_count_each_cue_entry_without_a_session() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
        load_show_cue(&mut conn, playlist_id, 123, false)?;

        let plan = crate::application::library_removal::plan_library_removal(
            &conn,
            &crate::application::library_removal::LibraryRemovalIntent::TrackId(track_id),
        )?;
        assert_eq!(
            plan.impact(),
            crate::application::library_removal::LibraryRemovalImpact::Track {
                playlist_reference_count: 1,
                show_cue_reference_count: 2,
            }
        );
        assert_eq!(db::tracks_in_use_by_show(&conn)?, [track_id].into());
        assert!(db::playback_session(&conn, crate::playback::DEFAULT_SESSION_ID)?.is_none());
        Ok(())
    }

    // ADR 0068 task 002 review: a cue track counts one time in the album
    // removal plan, also when the cue holds it two times.
    #[test]
    fn album_removal_counts_each_cue_track_one_time() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
        load_show_cue(&mut conn, playlist_id, 123, false)?;
        let feed_id: i64 = conn.query_row(
            "SELECT feed_id FROM tracks WHERE id = ?1",
            [track_id],
            |row| row.get(0),
        )?;

        let plan = crate::application::library_removal::plan_library_removal(
            &conn,
            &crate::application::library_removal::LibraryRemovalIntent::FeedId(feed_id),
        )?;

        assert_eq!(
            plan.impact(),
            crate::application::library_removal::LibraryRemovalImpact::Feed {
                playlist_track_count: 1,
                show_cue_track_count: 1,
            }
        );
        Ok(())
    }

    // ADR 0068: each cue entry has its own identity. A Replace gives new
    // entry ids, so a stored `cue_entry_id` cannot point to a new entry.
    #[test]
    fn replace_never_gives_an_old_entry_id_again() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, _) = playlist_with_repeated_track(&conn)?;
        load_show_cue(&mut conn, playlist_id, 1, false)?;
        let old_max: i64 =
            conn.query_row("SELECT MAX(id) FROM show_cue_entries", [], |row| row.get(0))?;

        load_show_cue(&mut conn, playlist_id, 2, true)?;

        let new_min: i64 =
            conn.query_row("SELECT MIN(id) FROM show_cue_entries", [], |row| row.get(0))?;
        assert!(new_min > old_max);
        Ok(())
    }

    fn insert_track(conn: &Connection, feed_id: i64, guid: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, ?2, ?2, 1)",
            params![feed_id, guid],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[test]
    fn cue_entries_report_missing_local_files() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, _) = playlist_with_repeated_track(&conn)?;
        load_show_cue(&mut conn, playlist_id, 123, false)?;
        let cue = show_cue(&conn)?.expect("the cue is stored");
        assert_eq!(
            cue.entries[0].unavailable_reason,
            Some(CueUnavailableReason::NotDownloaded)
        );
        Ok(())
    }

    #[test]
    fn show_cue_exists_tracks_load_and_clear() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, _) = playlist_with_repeated_track(&conn)?;
        assert!(!show_cue_exists(&conn)?);
        load_show_cue(&mut conn, playlist_id, 123, false)?;
        assert!(show_cue_exists(&conn)?);
        clear_show_cue(&mut conn)?;
        assert!(!show_cue_exists(&conn)?);
        Ok(())
    }

    #[test]
    fn failed_replace_rolls_back_cue_rows_and_entry_ids() -> Result<()> {
        let mut conn = setup()?;
        let (first_playlist, first_track) = playlist_with_repeated_track(&conn)?;
        let second_playlist = db::playlist_create(&conn, "Second cue")?;
        let feed_id: i64 = conn.query_row(
            "SELECT feed_id FROM tracks WHERE id = ?1",
            [first_track],
            |row| row.get(0),
        )?;
        let second_track = insert_track(&conn, feed_id, "second")?;
        db::playlist_append(&conn, second_playlist, second_track)?;
        db::playlist_append(&conn, second_playlist, second_track)?;
        load_show_cue(&mut conn, first_playlist, 1, false)?;
        let before = show_cue(&conn)?.expect("first cue is stored");
        conn.execute_batch("CREATE TEMP TRIGGER abort_second_cue_entry BEFORE INSERT ON show_cue_entries WHEN NEW.position = 1 BEGIN SELECT RAISE(ABORT, 'forced cue insert failure'); END;")?;

        assert!(load_show_cue(&mut conn, second_playlist, 2, true).is_err());

        let after = show_cue(&conn)?.expect("first cue remains");
        let sequence = |cue: &ShowCue| {
            cue.entries
                .iter()
                .map(|entry| (entry.id, entry.position, entry.track_id))
                .collect::<Vec<_>>()
        };
        assert_eq!(after.source_playlist_id, before.source_playlist_id);
        assert_eq!(sequence(&after), sequence(&before));
        Ok(())
    }

    #[test]
    fn cue_sequence_and_source_positions_survive_source_reorder() -> Result<()> {
        let mut conn = setup()?;
        conn.execute("INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/reorder', 'reorder-feed', 'Reorder')", [])?;
        let feed_id = conn.last_insert_rowid();
        let tracks = [
            insert_track(&conn, feed_id, "one")?,
            insert_track(&conn, feed_id, "two")?,
            insert_track(&conn, feed_id, "three")?,
        ];
        let playlist = db::playlist_create(&conn, "Reorder")?;
        for track in tracks {
            db::playlist_append(&conn, playlist, track)?;
        }
        load_show_cue(&mut conn, playlist, 10, false)?;
        db::playlist_reorder(&mut conn, playlist, 0, 2)?;
        let cue = show_cue(&conn)?.expect("cue remains stored");
        assert_eq!(
            cue.entries
                .iter()
                .map(|entry| (entry.track_id, entry.source_position))
                .collect::<Vec<_>>(),
            [(tracks[0], 0), (tracks[1], 1), (tracks[2], 2)]
        );
        Ok(())
    }

    #[test]
    fn cue_entries_select_one_local_file_for_each_cue_entry() -> Result<()> {
        let mut conn = setup()?;
        let (playlist_id, track_id) = playlist_with_repeated_track(&conn)?;
        conn.execute(
            "INSERT INTO local_files(path, track_id) VALUES ('first.mp3', ?1), ('second.mp3', ?1)",
            [track_id],
        )?;
        load_show_cue(&mut conn, playlist_id, 123, false)?;

        let cue = show_cue(&conn)?.expect("the cue is stored");

        assert_eq!(cue.entries.len(), 2);
        assert!(cue
            .entries
            .iter()
            .all(|entry| entry.unavailable_reason.is_none()));
        Ok(())
    }

    #[test]
    fn migration_25_keeps_sessions_and_does_not_make_a_cue() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema_to(&conn, 24)?;
        conn.execute(
            "INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/old', 'old-feed', 'Old feed')",
            [],
        )?;
        let feed_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, 'old-entry', 'Old track', 1)",
            [feed_id],
        )?;
        let track_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO playlists(id, name) VALUES (1, 'Old show')", [])?;
        db::playlist_append(&conn, 1, track_id)?;
        db::mark_track_downloaded(
            &conn,
            track_id,
            &crate::library_path::LibraryRelativePath::for_test("artists/old-track.mp3"),
            None,
        )?;
        conn.execute(
            "INSERT INTO rss_check_runs(id, playlist_id, trigger, started_at_us) VALUES (1, 1, 'playback_start', 10)",
            [],
        )?;
        conn.execute(
            "INSERT INTO rss_check_feed_results(run_id, feed_id, outcome) VALUES (1, ?1, 'document')",
            [feed_id],
        )?;
        conn.execute(
            "INSERT INTO playback_sessions (
                 session_id, sequence, local_track_id, playlist_id, playlist_position,
                 started_at, position_ms, state, updated_at
             ) VALUES ('default', 1, ?1, NULL, NULL, 'before', 0, 'playing', datetime('now'))",
            [track_id],
        )?;
        let migration_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        db::migrate_schema_to(&conn, 25)?;
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row
                .get::<_, i64>(0))?,
            migration_count + 1
        );
        assert!(show_cue(&conn)?.is_none());
        assert_eq!(
            conn.query_row(
                "SELECT trigger FROM rss_check_runs WHERE id = 1",
                [],
                |row| row.get::<_, String>(0)
            )?,
            "playback_start"
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM rss_check_feed_results WHERE run_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )?,
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT track_id FROM playlist_tracks WHERE playlist_id = 1",
                [],
                |row| row.get::<_, i64>(0)
            )?,
            track_id
        );
        assert_eq!(
            conn.query_row(
                "SELECT path FROM local_files WHERE track_id = ?1",
                [track_id],
                |row| row.get::<_, String>(0)
            )?,
            "artists/old-track.mp3"
        );
        assert_eq!(
            db::playback_session(&conn, crate::playback::DEFAULT_SESSION_ID)?
                .expect("legacy session remains")
                .local_track_id,
            track_id
        );
        conn.execute(
            "INSERT INTO rss_check_runs(playlist_id, trigger, started_at_us) VALUES (1, 'cue_load', 20)",
            [],
        )?;
        assert_eq!(
            db::playback_session(&conn, "default")?
                .expect("session remains")
                .local_track_id,
            track_id
        );
        Ok(())
    }
}
