//! Queue/Now Playing projection adapter.
//!
//! This module bridges application playback/session state into the queue
//! display contract. ADR 0060 renders that projection inside `Show`; the
//! detailed queue and transport controls are still projected here.

use gpui::{App, ClickEvent, Context, Entity, Window};
use std::path::Path;

use crate::application::ApplicationServices;
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

pub(super) fn queue_now_playing_vm(
    services: &ApplicationServices,
    conn: &rusqlite::Connection,
    music_dir: &Path,
    text_filter: Option<String>,
) -> QueueNowPlayingPageVm {
    let session = db::playback_session(conn, playback::DEFAULT_SESSION_ID)
        .ok()
        .flatten()
        .filter(|session| session.state != "stopped");
    let transport_state = session.as_ref().map_or(TransportState::Stopped, |session| {
        if session.state == "paused" {
            TransportState::Paused
        } else {
            TransportState::Playing
        }
    });
    let queue = session.as_ref().map_or_else(
        || queue_tracks_for_stored_cue(conn, music_dir),
        |session| queue_tracks_for_session(services, conn, session),
    );

    let mut vm = QueueNowPlayingPageVm::builder()
        .tracks(queue.tracks)
        .transport_state(transport_state)
        .skip_availability(queue.can_skip_previous, queue.can_skip_next)
        .build();

    if let Some(filter) = text_filter {
        vm.set_text_filter(Some(filter));
    }

    vm
}

fn queue_tracks_for_stored_cue(conn: &rusqlite::Connection, music_dir: &Path) -> QueueProjection {
    let Ok(Some(cue)) = db::show_cue(conn) else {
        return QueueProjection::default();
    };
    QueueProjection {
        tracks: cue
            .entries
            .into_iter()
            .map(|entry| {
                let file_exists = entry
                    .track
                    .local_path
                    .as_ref()
                    .is_some_and(|path| path.resolve(music_dir).is_file());
                let mut input = queue_track_input(entry.track, i64::MIN, Some(entry.id));
                input.unavailable_reason = entry.unavailable_reason.map(|reason| match reason {
                    db::CueUnavailableReason::NotInLibrary => QueueUnavailableReason::NotInLibrary,
                    db::CueUnavailableReason::MissingFile => QueueUnavailableReason::MissingFile,
                });
                if input.unavailable_reason.is_none() && !file_exists {
                    input.unavailable_reason = Some(QueueUnavailableReason::MissingFile);
                }
                input
            })
            .collect(),
        can_skip_previous: false,
        can_skip_next: false,
    }
}

#[derive(Default)]
struct QueueProjection {
    tracks: Vec<QueueTrackInput>,
    can_skip_previous: bool,
    can_skip_next: bool,
}

fn queue_tracks_for_session(
    services: &ApplicationServices,
    conn: &rusqlite::Connection,
    session: &db::PlaybackSessionRow,
) -> QueueProjection {
    if let Some(playlist_id) = session.playlist_id {
        let rows = services
            .query_service()
            .playlist_tracks(conn, playlist_id)
            .unwrap_or_default();
        return playlist_queue_projection(rows, session.local_track_id);
    }

    db::track_row_by_id(conn, session.local_track_id)
        .ok()
        .flatten()
        .map_or_else(QueueProjection::default, |row| QueueProjection {
            tracks: vec![queue_track_input(row, session.local_track_id, None)],
            can_skip_previous: false,
            can_skip_next: false,
        })
}

fn playlist_queue_projection(rows: Vec<db::TrackRow>, active_track_id: i64) -> QueueProjection {
    let active_index = rows.iter().position(|row| row.id == active_track_id);
    let can_skip_previous =
        active_index.is_some_and(|index| rows[..index].iter().rev().any(queue_track_can_play));
    let can_skip_next = active_index.is_some_and(|index| {
        rows[index.saturating_add(1)..]
            .iter()
            .any(queue_track_can_play)
    });
    QueueProjection {
        tracks: rows
            .into_iter()
            .map(|row| queue_track_input(row, active_track_id, None))
            .collect(),
        can_skip_previous,
        can_skip_next,
    }
}

fn queue_track_can_play(row: &db::TrackRow) -> bool {
    row.is_in_library && row.local_path.is_some()
}

fn queue_track_input(
    row: db::TrackRow,
    active_track_id: i64,
    entry_id: Option<i64>,
) -> QueueTrackInput {
    QueueTrackInput {
        id: row.id,
        entry_id,
        unavailable_reason: None,
        title: row.track_title.or(row.feed_title),
        artist: row.artist_name.or(row.album_artist_name),
        duration_seconds: row.duration_seconds,
        now_playing: row.id == active_track_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let projection = queue_tracks_for_stored_cue(&conn, music_dir.path());

        let reasons = projection
            .tracks
            .iter()
            .map(|track| track.unavailable_reason)
            .collect::<Vec<_>>();
        assert_eq!(reasons, [None, Some(QueueUnavailableReason::MissingFile)]);
        Ok(())
    }
}
