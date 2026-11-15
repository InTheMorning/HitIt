//! Queue/Now Playing projection adapter.
//!
//! This module bridges application playback/session state into the queue
//! display contract. ADR 0060 renders that projection inside `Show`; the
//! detailed queue and transport controls are still projected here.

use gpui::{App, ClickEvent, Context, Entity, Window};
use std::path::Path;

use crate::view_models::queue_now_playing::{
    QueueNowPlayingPageVm, QueueTrackInput, QueueUnavailableReason, TransportState,
};
use crate::{db, playback};

use super::TopApp;

pub(super) type QueueTransportAction = fn(&mut TopApp, &mut Context<TopApp>);

pub(super) fn queue_transport_action(
    entity: Entity<TopApp>,
    action: QueueTransportAction,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    move |_, _, cx| {
        entity.update(cx, action);
    }
}

pub(super) fn queue_select_entry_action(
    entity: Entity<TopApp>,
) -> impl Fn(i64, &ClickEvent, &mut Window, &mut App) + 'static {
    move |entry_id, _, _, cx| entity.update(cx, |app, cx| app.select_show_cue_entry(entry_id, cx))
}

pub(super) struct QueueNowPlayingInput {
    session: Option<db::PlaybackSessionRow>,
    queue: QueueProjection,
    text_filter: Option<String>,
    has_cue: bool,
}

pub(super) fn queue_now_playing_input(
    conn: &rusqlite::Connection,
    text_filter: Option<String>,
) -> QueueNowPlayingInput {
    let session = db::playback_session(conn, playback::DEFAULT_SESSION_ID)
        .ok()
        .flatten()
        .filter(|session| session.state != "stopped");
    let queue = queue_tracks_for_stored_cue(
        conn,
        session.as_ref().and_then(|row| row.cue_entry_id),
        session.is_none(),
    );

    QueueNowPlayingInput {
        session,
        queue,
        text_filter,
        has_cue: db::show_cue_exists(conn).unwrap_or(false),
    }
}

pub(super) fn queue_now_playing_vm(
    input: QueueNowPlayingInput,
    music_dir: &Path,
) -> QueueNowPlayingPageVm {
    let QueueNowPlayingInput {
        session,
        mut queue,
        text_filter,
        has_cue,
    } = input;
    for (track, path) in queue.tracks.iter_mut().zip(queue.cue_paths) {
        if let Some(path) = path {
            if track.unavailable_reason.is_none() && !path.resolve(music_dir).is_file() {
                track.unavailable_reason = Some(QueueUnavailableReason::MissingFile);
            }
        }
    }
    let transport_state = session.as_ref().map_or_else(
        || {
            if has_cue {
                TransportState::Stopped
            } else {
                TransportState::Empty
            }
        },
        |session| {
            if session.state == "paused" {
                TransportState::Paused
            } else {
                TransportState::Playing
            }
        },
    );

    let mut vm = QueueNowPlayingPageVm::builder()
        .tracks(queue.tracks)
        .transport_state(transport_state)
        .skip_availability(queue.can_skip_previous, queue.can_skip_next)
        .has_cue(has_cue)
        .build();

    if let Some(filter) = text_filter {
        vm.set_text_filter(Some(filter));
    }

    vm
}

fn queue_tracks_for_stored_cue(
    conn: &rusqlite::Connection,
    active_entry_id: Option<i64>,
    can_select: bool,
) -> QueueProjection {
    let Ok(Some(cue)) = db::show_cue(conn) else {
        return QueueProjection::default();
    };
    let cursor_entry_id = active_entry_id.or(cue.selected_entry_id);
    let cursor_index =
        cursor_entry_id.and_then(|id| cue.entries.iter().position(|entry| entry.id == id));
    let can_skip_previous = cursor_index.is_some_and(|index| index > 0);
    let can_skip_next = cursor_index.is_some_and(|index| index + 1 < cue.entries.len());
    let tracks = cue
        .entries
        .into_iter()
        .map(|entry| {
            let file_path = entry.track.local_path.clone();
            let mut input = queue_track_input(
                entry.track,
                active_entry_id,
                cue.selected_entry_id,
                Some(entry.id),
                can_select,
            );
            input.unavailable_reason = entry.unavailable_reason.map(|reason| match reason {
                db::CueUnavailableReason::NotInLibrary => QueueUnavailableReason::NotInLibrary,
                db::CueUnavailableReason::NotDownloaded => QueueUnavailableReason::NotDownloaded,
            });
            (input, file_path)
        })
        .collect::<Vec<_>>();
    let (tracks, cue_paths) = tracks.into_iter().unzip();
    QueueProjection {
        tracks,
        cue_paths,
        can_skip_previous,
        can_skip_next,
    }
}

#[derive(Default)]
struct QueueProjection {
    tracks: Vec<QueueTrackInput>,
    cue_paths: Vec<Option<crate::library_path::LibraryRelativePath>>,
    can_skip_previous: bool,
    can_skip_next: bool,
}

fn queue_track_input(
    row: db::TrackRow,
    active_entry_id: Option<i64>,
    selected_entry_id: Option<i64>,
    entry_id: Option<i64>,
    selectable: bool,
) -> QueueTrackInput {
    QueueTrackInput {
        id: row.id,
        entry_id,
        unavailable_reason: None,
        title: row.track_title.or(row.feed_title),
        artist: row.artist_name.or(row.album_artist_name),
        duration_seconds: row.duration_seconds,
        now_playing: entry_id.is_some() && entry_id == active_entry_id,
        selected: entry_id.is_some() && entry_id == selected_entry_id,
        selectable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleared_cue_projects_an_empty_queue_without_clear_availability() -> anyhow::Result<()> {
        let mut conn = rusqlite::Connection::open_in_memory()?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        let playlist_id = db::playlist_create(&conn, "Empty cue")?;
        conn.execute(
            "INSERT INTO show_cue(id, loaded_at_us, source_playlist_id, source_playlist_name) VALUES (1, 1, ?1, 'Empty cue')",
            [playlist_id],
        )?;
        let music_dir = tempfile::tempdir()?;

        let before = queue_now_playing_vm(queue_now_playing_input(&conn, None), music_dir.path());
        assert!(before.can_clear_show_cue());
        db::clear_show_cue(&mut conn)?;
        let after = queue_now_playing_vm(queue_now_playing_input(&conn, None), music_dir.path());

        assert_eq!(after.transport.play_pause_state, TransportState::Empty);
        assert!(!after.can_clear_show_cue());
        assert!(after.rows.is_empty());
        Ok(())
    }

    // ADR 0068 task 002 R2: a bound cue file that is not on the disk is
    // unavailable, and a bound file that is on the disk is available.
    #[test]
    fn stored_cue_marks_a_bound_file_absent_from_the_disk_as_missing() -> anyhow::Result<()> {
        let mut conn = rusqlite::Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        conn.execute(
            "INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/disk', 'disk-feed', 'Disk feed')",
            [],
        )?;
        let feed_id = conn.last_insert_rowid();
        let mut track_ids = Vec::new();
        for guid in ["on-disk", "absent"] {
            conn.execute(
                "INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, ?2, ?2, 1)",
                rusqlite::params![feed_id, guid],
            )?;
            let track_id = conn.last_insert_rowid();
            db::mark_track_downloaded(
                &conn,
                track_id,
                &crate::library_path::LibraryRelativePath::for_test(&format!("{guid}.mp3")),
                None,
            )?;
            track_ids.push(track_id);
        }
        let playlist_id = db::playlist_create(&conn, "Disk")?;
        for track_id in &track_ids {
            db::playlist_append(&conn, playlist_id, *track_id)?;
        }
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let music_dir = tempfile::tempdir()?;
        std::fs::write(music_dir.path().join("on-disk.mp3"), b"audio")?;

        let input = queue_now_playing_input(&conn, None);
        let projection = queue_now_playing_vm(input, music_dir.path());

        let reasons = projection
            .rows
            .iter()
            .map(|track| track.unavailable_label)
            .collect::<Vec<_>>();
        assert_eq!(reasons, [None, Some("File missing")]);
        Ok(())
    }

    #[test]
    fn skip_availability_follows_the_cue_boundaries_and_is_off_without_a_cue() -> anyhow::Result<()>
    {
        let mut conn = rusqlite::Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        let skips = |conn: &rusqlite::Connection| {
            let vm = queue_now_playing_vm(
                queue_now_playing_input(conn, None),
                Path::new("/nonexistent"),
            );
            (vm.can_skip_previous(), vm.can_skip_next())
        };
        assert_eq!(skips(&conn), (false, false));

        conn.execute("INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/skip', 'skip', 'Skip')", [])?;
        let feed_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, 'track', 'Track', 1)", [feed_id])?;
        let track_id = conn.last_insert_rowid();
        let playlist_id = db::playlist_create(&conn, "Skip")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let entries = db::show_cue(&conn)?.expect("cue").entries;

        assert_eq!(skips(&conn), (false, true));
        db::select_show_cue_entry(&conn, entries[1].id)?;
        assert_eq!(skips(&conn), (true, false));
        Ok(())
    }

    #[test]
    fn stored_cue_reports_not_downloaded_without_a_file_binding() -> anyhow::Result<()> {
        let mut conn = rusqlite::Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        conn.execute("INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://cue.test/no-file', 'no-file', 'No file')", [])?;
        let feed_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, 'track', 'Track', 1)", [feed_id])?;
        let track_id = conn.last_insert_rowid();
        let playlist_id = db::playlist_create(&conn, "No file")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;

        let input = queue_now_playing_input(&conn, None);
        let vm = queue_now_playing_vm(input, Path::new("/nonexistent"));
        assert_eq!(vm.rows[0].unavailable_label, Some("Not downloaded"));
        Ok(())
    }
}
