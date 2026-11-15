//! Long-running playback owner for live driver mode.
//!
//! One-shot CLI commands keep using `playback.rs` directly. This owner is for
//! processes that can hold a driver, poll it, and reconcile observed state back
//! into the canonical playback session.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::broadcast::producer::DropFileProducer;
use crate::playback_driver::{DriverStatus, PlaybackDriver};
use crate::{db, playback, track_identity};

#[derive(Clone, Debug)]
pub enum PollOutcome {
    NoSession,
    Reconciled(Option<playback::NowPlayingUpdate>),
    Advanced(playback::NowPlayingUpdate),
    /// The session changed to the state `stopped` after the owner loaded
    /// its track. The owner stopped the driver. A later poll of the same
    /// stopped session gives `Reconciled(None)`. ADR 0076 packet 005 starts
    /// the tag update scan on this outcome.
    Stopped,
}

/// The reason why a Show cue entry cannot play (ADR 0068).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShowCueEntryBlock {
    NotInLibrary,
    NotDownloaded,
    FileMissing,
}

impl From<db::CueUnavailableReason> for ShowCueEntryBlock {
    fn from(reason: db::CueUnavailableReason) -> Self {
        match reason {
            db::CueUnavailableReason::NotInLibrary => Self::NotInLibrary,
            db::CueUnavailableReason::NotDownloaded => Self::NotDownloaded,
        }
    }
}

/// A Show command named a cue entry that cannot play. The session did not change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShowCueEntryCannotPlay {
    pub entry_id: i64,
    pub reason: ShowCueEntryBlock,
}

impl std::fmt::Display for ShowCueEntryCannotPlay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self.reason {
            ShowCueEntryBlock::NotInLibrary => "the track is not in the library",
            ShowCueEntryBlock::NotDownloaded => "the track is not downloaded",
            ShowCueEntryBlock::FileMissing => "the file is missing",
        };
        write!(f, "Show cue entry {} cannot play: {reason}", self.entry_id)
    }
}

impl std::error::Error for ShowCueEntryCannotPlay {}

#[derive(Debug)]
pub struct PlaybackOwner<D> {
    driver: D,
    session_id: String,
    music_dir: PathBuf,
    eof_armed: bool,
    loaded_track_id: Option<i64>,
    generation: u64,
    active_cue_entry_id: Option<i64>,
    prepared_cue_entry_id: Option<i64>,
    drop_file_producer: Option<DropFileProducer>,
}

impl<D: PlaybackDriver> PlaybackOwner<D> {
    pub fn new(driver: D, session_id: impl Into<String>, music_dir: impl Into<PathBuf>) -> Self {
        Self {
            driver,
            session_id: session_id.into(),
            music_dir: music_dir.into(),
            eof_armed: true,
            loaded_track_id: None,
            generation: 0,
            active_cue_entry_id: None,
            prepared_cue_entry_id: None,
            drop_file_producer: None,
        }
    }

    #[must_use]
    pub fn with_drop_file_producer(mut self, producer: Option<DropFileProducer>) -> Self {
        self.drop_file_producer = producer;
        self
    }

    pub fn driver(&self) -> &D {
        &self.driver
    }

    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn active_cue_entry_id(&self) -> Option<i64> {
        self.active_cue_entry_id
    }

    /// Replace idle optional resources without loading audio or publishing metadata (ADR 0066).
    pub(crate) fn replace_idle_driver(&mut self, driver: D) -> Result<()> {
        if self.loaded_track_id.is_some() {
            anyhow::bail!("Playback still owns a loaded track. End the app session before changing its driver.");
        }
        self.driver = driver;
        self.advance_generation();
        Ok(())
    }

    pub(crate) fn replace_idle_producer(
        &mut self,
        producer: Option<DropFileProducer>,
    ) -> Result<()> {
        if self.loaded_track_id.is_some() {
            anyhow::bail!("Playback still owns a loaded track. End the app session before changing its publication target.");
        }
        self.drop_file_producer = producer;
        Ok(())
    }

    pub fn set_music_dir(&mut self, music_dir: impl Into<PathBuf>) {
        self.music_dir = music_dir.into();
    }

    #[must_use]
    pub fn drop_file_producer(&self) -> Option<&DropFileProducer> {
        self.drop_file_producer.as_ref()
    }

    #[must_use]
    pub fn broadcast_drop_file_shutdown_warning(&self) -> Option<&'static str> {
        self.drop_file_producer
            .as_ref()
            .map(DropFileProducer::shutdown_warning)
    }

    pub fn clear_broadcast_drop_file(&mut self) -> Result<bool> {
        self.clear_drop_file()
    }

    pub fn load_track(
        &mut self,
        conn: &Connection,
        track_id: i64,
        start_ms: u64,
    ) -> Result<playback::NowPlayingUpdate> {
        let identity = track_identity::local_track_identity(conn, track_id)?;
        let path = identity.local_path.resolve(&self.music_dir);
        self.load_track_path(conn, track_id, &path, start_ms)
    }

    pub fn load_track_path(
        &mut self,
        conn: &Connection,
        track_id: i64,
        path: &Path,
        start_ms: u64,
    ) -> Result<playback::NowPlayingUpdate> {
        self.driver.load(path, start_ms)?;
        self.advance_generation();
        self.active_cue_entry_id = None;
        self.prepared_cue_entry_id = None;
        self.eof_armed = true;
        self.loaded_track_id = Some(track_id);
        let update = playback::set_track(conn, track_id, &self.session_id)?;
        self.sync_drop_file_for_update(conn, &update)?;
        if start_ms == 0 {
            return Ok(update);
        }
        let update = playback::update_position(conn, start_ms, &self.session_id)?;
        self.sync_drop_file_for_update(conn, &update)?;
        Ok(update)
    }

    pub fn start_show_entry(
        &mut self,
        conn: &Connection,
        entry_id: Option<i64>,
    ) -> Result<playback::NowPlayingUpdate> {
        anyhow::ensure!(
            db::playback_session(conn, &self.session_id)?
                .is_none_or(|session| session.state == "stopped"),
            "Show must be stopped before it starts a cue entry"
        );
        let cue = db::show_cue(conn)?.context("No Show cue is loaded")?;
        let entry_id = entry_id
            .or(cue.selected_entry_id)
            .context("No Show cue entry is selected")?;
        let entry = cue
            .entries
            .iter()
            .find(|entry| entry.id == entry_id)
            .with_context(|| format!("Show cue entry {entry_id} does not exist"))?;
        if let Some(reason) = entry.unavailable_reason {
            return Err(ShowCueEntryCannotPlay {
                entry_id,
                reason: reason.into(),
            }
            .into());
        }
        let path = entry
            .track
            .local_path
            .as_ref()
            .map(|path| path.resolve(&self.music_dir))
            .filter(|path| path.is_file())
            .ok_or(ShowCueEntryCannotPlay {
                entry_id,
                reason: ShowCueEntryBlock::FileMissing,
            })?;

        self.driver
            .load(&path, 0)
            .with_context(|| format!("load Show cue entry {entry_id}"))?;
        self.advance_generation();
        self.loaded_track_id = Some(entry.track_id);
        self.active_cue_entry_id = Some(entry_id);
        self.prepared_cue_entry_id = None;
        self.eof_armed = true;
        let started_at = chrono::Utc::now().to_rfc3339();
        db::set_playback_session_track_with_cue(
            conn,
            &self.session_id,
            db::PlaybackSessionTrackUpdate {
                local_track_id: entry.track_id,
                playlist_id: None,
                playlist_position: None,
                started_at: &started_at,
                position_ms: 0,
                cue_entry_id: Some(entry_id),
            },
        )?;
        let update = playback::now_playing_update(conn, &self.session_id)?
            .context("Show session did not start")?;
        self.sync_drop_file_for_update(conn, &update)?;
        Ok(update)
    }

    fn advance_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn load_current_session(&mut self, conn: &Connection) -> Result<Option<DriverStatus>> {
        let Some(session) = db::playback_session(conn, &self.session_id)? else {
            self.clear_drop_file()?;
            return Ok(None);
        };
        if session.state == "stopped" {
            self.loaded_track_id = None;
            self.active_cue_entry_id = None;
            self.prepared_cue_entry_id = None;
            self.clear_drop_file()?;
            return Ok(None);
        }
        let identity = track_identity::local_track_identity(conn, session.local_track_id)?;
        let path = identity.local_path.resolve(&self.music_dir);
        self.driver.load(&path, session.position_ms)?;
        self.advance_generation();
        if session.state == "paused" {
            self.driver.pause(true)?;
        }
        self.eof_armed = true;
        self.loaded_track_id = Some(session.local_track_id);
        self.active_cue_entry_id = session.cue_entry_id;
        self.prepared_cue_entry_id = None;
        let status = self.driver.poll()?;
        if session.state == "paused" {
            self.clear_drop_file()?;
        } else if let Some(update) = playback::now_playing_update(conn, &self.session_id)? {
            self.sync_drop_file_for_update(conn, &update)?;
        }
        Ok(Some(status))
    }

    pub fn seek(
        &mut self,
        conn: &Connection,
        position_ms: u64,
    ) -> Result<playback::NowPlayingUpdate> {
        self.driver.seek(position_ms)?;
        let update = playback::update_position(conn, position_ms, &self.session_id)?;
        self.sync_drop_file_for_update(conn, &update)?;
        Ok(update)
    }

    pub fn pause(&mut self, conn: &Connection, paused: bool) -> Result<playback::NowPlayingUpdate> {
        let update = if !paused {
            if let Some(entry_id) = self.prepared_cue_entry_id {
                let cue = db::show_cue(conn)?.context("No Show cue is loaded")?;
                let entry = cue
                    .entries
                    .iter()
                    .find(|entry| entry.id == entry_id)
                    .context("Prepared Show cue entry does not exist")?;
                if let Some(reason) = entry.unavailable_reason {
                    return Err(ShowCueEntryCannotPlay {
                        entry_id,
                        reason: reason.into(),
                    }
                    .into());
                }
                let path = entry
                    .track
                    .local_path
                    .as_ref()
                    .map(|path| path.resolve(&self.music_dir))
                    .filter(|path| path.is_file())
                    .ok_or(ShowCueEntryCannotPlay {
                        entry_id,
                        reason: ShowCueEntryBlock::FileMissing,
                    })?;
                self.driver.load(&path, 0)?;
                self.advance_generation();
                self.eof_armed = true;
                self.prepared_cue_entry_id = None;
                db::update_playback_session_paused(conn, &self.session_id, false)?;
                playback::now_playing_update(conn, &self.session_id)?
                    .context("Show session did not resume")?
            } else {
                self.driver.pause(false)?;
                playback::update_paused(conn, false, &self.session_id)?
            }
        } else {
            self.driver.pause(true)?;
            playback::update_paused(conn, true, &self.session_id)?
        };
        if paused {
            self.clear_drop_file()?;
        } else {
            self.sync_drop_file_for_update(conn, &update)?;
        }
        Ok(update)
    }

    pub fn move_show_cue(
        &mut self,
        conn: &Connection,
        next: bool,
    ) -> Result<Option<playback::NowPlayingUpdate>> {
        let session = db::playback_session(conn, &self.session_id)?;
        let cue = db::show_cue(conn)?.context("No Show cue is loaded")?;
        let current_entry_id = session
            .as_ref()
            .filter(|session| session.state != "stopped")
            .and_then(|session| session.cue_entry_id)
            .or(cue.selected_entry_id)
            .context("No Show cue entry is selected")?;
        let target = if next {
            db::show_cue_entry_after(conn, current_entry_id)?
        } else {
            db::show_cue_entry_before(conn, current_entry_id)?
        }
        .context("No adjacent Show cue entry")?;
        let path = target
            .track
            .local_path
            .as_ref()
            .map(|path| path.resolve(&self.music_dir))
            .filter(|path| path.is_file());
        let reason = target
            .unavailable_reason
            .map(Into::into)
            .or_else(|| path.is_none().then_some(ShowCueEntryBlock::FileMissing));
        if let Some(reason) = reason {
            return Err(ShowCueEntryCannotPlay {
                entry_id: target.id,
                reason,
            }
            .into());
        }
        let path = path.context("Show cue entry has no local file")?;
        let Some(session) = session.filter(|session| session.state != "stopped") else {
            db::select_show_cue_entry(conn, target.id)?;
            return Ok(None);
        };
        let paused = session.state == "paused";
        if paused {
            self.driver.stop()?;
        } else {
            self.driver.load(&path, 0)?;
        }
        self.advance_generation();
        let started_at = chrono::Utc::now().to_rfc3339();
        db::set_playback_session_track_with_cue(
            conn,
            &self.session_id,
            db::PlaybackSessionTrackUpdate {
                local_track_id: target.track_id,
                playlist_id: None,
                playlist_position: None,
                started_at: &started_at,
                position_ms: 0,
                cue_entry_id: Some(target.id),
            },
        )?;
        if paused {
            db::update_playback_session_paused(conn, &self.session_id, true)?;
        }
        self.active_cue_entry_id = Some(target.id);
        self.loaded_track_id = Some(target.track_id);
        self.prepared_cue_entry_id = paused.then_some(target.id);
        self.eof_armed = !paused;
        let update = playback::now_playing_update(conn, &self.session_id)?
            .context("Show session did not move")?;
        self.sync_drop_file_for_update(conn, &update)?;
        Ok(Some(update))
    }

    /// Finish existing playback state during managed session maintenance (ADR 0066).
    pub(crate) fn finish_session_playback(&mut self, conn: &Connection) -> Result<()> {
        if db::playback_session(conn, &self.session_id)?.is_some() {
            self.stop(conn)?;
        }
        Ok(())
    }

    pub fn stop(&mut self, conn: &Connection) -> Result<db::PlaybackSessionRow> {
        self.driver.stop()?;
        self.advance_generation();
        self.eof_armed = false;
        self.loaded_track_id = None;
        self.active_cue_entry_id = None;
        self.prepared_cue_entry_id = None;
        let session = playback::stop(conn, &self.session_id)?;
        self.clear_drop_file()?;
        Ok(session)
    }

    pub fn poll(&mut self, conn: &Connection) -> Result<PollOutcome> {
        let Some(session) = db::playback_session(conn, &self.session_id)? else {
            if self.loaded_track_id.take().is_some() {
                self.driver.stop()?;
                self.advance_generation();
            }
            self.active_cue_entry_id = None;
            self.prepared_cue_entry_id = None;
            self.clear_drop_file()?;
            return Ok(PollOutcome::NoSession);
        };
        if session.state == "stopped" {
            let was_loaded = self.loaded_track_id.take().is_some();
            if was_loaded {
                self.driver.stop()?;
                self.advance_generation();
            }
            self.active_cue_entry_id = None;
            self.prepared_cue_entry_id = None;
            self.clear_drop_file()?;
            return Ok(if was_loaded {
                PollOutcome::Stopped
            } else {
                PollOutcome::Reconciled(None)
            });
        }
        if session.state == "paused"
            && self.prepared_cue_entry_id.is_some()
            && self.prepared_cue_entry_id == session.cue_entry_id
        {
            return Ok(PollOutcome::Reconciled(None));
        }
        if self.loaded_track_id != Some(session.local_track_id) {
            let identity = track_identity::local_track_identity(conn, session.local_track_id)?;
            let path = identity.local_path.resolve(&self.music_dir);
            self.driver.load(&path, session.position_ms)?;
            self.advance_generation();
            if session.state == "paused" {
                self.driver.pause(true)?;
            }
            self.loaded_track_id = Some(session.local_track_id);
            self.active_cue_entry_id = session.cue_entry_id;
            let update = playback::now_playing_update(conn, &self.session_id)?;
            match &update {
                Some(update) => self.sync_drop_file_for_update(conn, update)?,
                None => {
                    self.clear_drop_file()?;
                }
            }
            return Ok(PollOutcome::Reconciled(update));
        }
        let poll_generation = self.generation;
        let poll_entry_id = self.active_cue_entry_id;
        let status = self.driver.poll()?;
        if let Some(error) = &status.error {
            anyhow::bail!("playback driver error: {error}");
        }
        if status.eof {
            if poll_generation != self.generation
                || poll_entry_id != self.active_cue_entry_id
                || session.cue_entry_id != self.active_cue_entry_id
            {
                return Ok(PollOutcome::Reconciled(None));
            }
            if !self.eof_armed {
                return Ok(PollOutcome::Reconciled(None));
            }
            self.eof_armed = false;
            let target = match session.cue_entry_id {
                Some(entry_id) => db::show_cue_entry_after(conn, entry_id)?,
                None => None,
            };
            let Some(target) = target else {
                self.driver.stop()?;
                self.advance_generation();
                self.loaded_track_id = None;
                self.active_cue_entry_id = None;
                playback::stop(conn, &self.session_id)?;
                self.clear_drop_file()?;
                return Ok(PollOutcome::Stopped);
            };
            let path = target
                .track
                .local_path
                .as_ref()
                .map(|path| path.resolve(&self.music_dir))
                .filter(|path| path.is_file());
            if target.unavailable_reason.is_some() || path.is_none() {
                self.driver.stop()?;
                self.advance_generation();
                self.loaded_track_id = None;
                self.active_cue_entry_id = None;
                playback::stop(conn, &self.session_id)?;
                self.clear_drop_file()?;
                let reason = target
                    .unavailable_reason
                    .map(Into::into)
                    .unwrap_or(ShowCueEntryBlock::FileMissing);
                return Err(ShowCueEntryCannotPlay {
                    entry_id: target.id,
                    reason,
                }
                .into());
            }
            self.driver
                .load(path.as_deref().expect("checked path"), 0)?;
            self.advance_generation();
            let started_at = chrono::Utc::now().to_rfc3339();
            db::set_playback_session_track_with_cue(
                conn,
                &self.session_id,
                db::PlaybackSessionTrackUpdate {
                    local_track_id: target.track_id,
                    playlist_id: None,
                    playlist_position: None,
                    started_at: &started_at,
                    position_ms: 0,
                    cue_entry_id: Some(target.id),
                },
            )?;
            self.loaded_track_id = Some(target.track_id);
            self.active_cue_entry_id = Some(target.id);
            self.prepared_cue_entry_id = None;
            self.eof_armed = false;
            let update = playback::now_playing_update(conn, &self.session_id)?
                .context("Show session did not advance")?;
            self.sync_drop_file_for_update(conn, &update)?;
            return Ok(PollOutcome::Advanced(update));
        }
        self.eof_armed = true;
        let update = playback::reconcile_driver_status(conn, &status, &self.session_id)?;
        match &update {
            Some(update) => self.sync_drop_file_for_update(conn, update)?,
            None => {
                self.clear_drop_file()?;
            }
        }
        Ok(PollOutcome::Reconciled(update))
    }

    fn sync_drop_file_for_update(
        &mut self,
        conn: &Connection,
        update: &playback::NowPlayingUpdate,
    ) -> Result<()> {
        let Some(producer) = self.drop_file_producer.as_mut() else {
            return Ok(());
        };
        let Some(session) = db::playback_session(conn, &self.session_id)? else {
            producer.clear()?;
            return Ok(());
        };
        if session.state != "playing" {
            producer.clear()?;
            return Ok(());
        }
        let identity = track_identity::local_track_identity(conn, update.local_track_id)?;
        let path = identity.local_path.resolve(&self.music_dir);
        producer.publish(update, &path)?;
        Ok(())
    }

    fn clear_drop_file(&mut self) -> Result<bool> {
        self.drop_file_producer
            .as_mut()
            .map_or(Ok(false), DropFileProducer::clear)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use anyhow::Result;

    use super::*;
    use crate::application::ApplicationQueryService;
    use crate::audio_tags::{write_id3v24_edits, Id3v24Edit};
    use crate::broadcast::producer::{DropFileProducer, SHUTDOWN_WARNING};
    use crate::playback_driver::NullDriver;

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
    }

    fn create_feed(conn: &Connection) -> Result<i64> {
        conn.execute(
            "INSERT INTO feeds (
                 feed_url, feed_guid, title, album_image_href, podcast_value_json
             )
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "https://example.test/feed.xml",
                "feed-guid",
                "Feed Title",
                "https://example.test/feed.png",
                r#"{"feed":"value"}"#
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_track(
        conn: &Connection,
        feed_id: i64,
        item_guid: &str,
        music_dir: &Path,
        path: &Path,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO tracks (
                 feed_id, item_guid, track_title, artist_name, album_title,
                 duration_seconds, item_value_json, extra_json
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                feed_id,
                item_guid,
                format!("Track {item_guid}"),
                "Artist Name",
                "Album Title",
                123_i64,
                r#"{"item":"value"}"#,
                r#"{"source":"rss"}"#
            ],
        )?;
        let track_id = conn.last_insert_rowid();
        let relative_path =
            crate::library_path::LibraryRelativePath::from_absolute(music_dir, path)?;
        db::mark_track_downloaded(conn, track_id, &relative_path, None)?;
        Ok(track_id)
    }

    fn tagged_audio_file(temp: &tempfile::TempDir, name: &str) -> Result<PathBuf> {
        let path = temp.path().join(name);
        fs::write(&path, b"not really an mp3")?;
        write_id3v24_edits(
            &path,
            &[Id3v24Edit {
                frame_label: "TXXX:MusicIndex Value Routes".to_owned(),
                value: r#"[{"recipient_name":"Embedded Route","route_type":"node","split":77.0}]"#
                    .to_owned(),
            }],
        )?;
        Ok(path)
    }

    #[test]
    fn owner_seek_updates_driver_and_session_immediately() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(
            &conn,
            feed_id,
            "item-guid",
            Path::new("/"),
            Path::new("/tmp/track.mp3"),
        )?;
        playback::set_track(&conn, track_id, playback::DEFAULT_SESSION_ID)?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            PathBuf::from("/"),
        );
        owner.load_current_session(&conn)?;

        let update = owner.seek(&conn, 12_000)?;
        let snap = owner.driver().snapshot();

        assert_eq!(update.position_ms, 12_000);
        assert_eq!(snap.position_ms, 12_000);
        Ok(())
    }

    #[test]
    fn owner_pause_persists_paused_state() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(
            &conn,
            feed_id,
            "item-guid",
            Path::new("/"),
            Path::new("/tmp/track.mp3"),
        )?;
        playback::set_track(&conn, track_id, playback::DEFAULT_SESSION_ID)?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            PathBuf::from("/"),
        );
        owner.load_current_session(&conn)?;

        owner.pause(&conn, true)?;
        let row = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");

        assert_eq!(row.state, "paused");
        assert!(owner.driver().snapshot().paused);
        Ok(())
    }

    #[test]
    fn show_start_pause_resume_and_stop_keep_the_cue_entry() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let audio_path = temp.path().join("show.mp3");
        fs::write(&audio_path, b"audio")?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(&conn, feed_id, "show", temp.path(), &audio_path)?;
        let playlist_id = db::playlist_create(&conn, "Show")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let selected = db::show_cue(&conn)?
            .expect("cue")
            .selected_entry_id
            .expect("entry");
        let mut owner =
            PlaybackOwner::new(NullDriver::new(), playback::DEFAULT_SESSION_ID, temp.path());

        let update = owner.start_show_entry(&conn, None)?;
        let started = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        assert_eq!(update.local_track_id, track_id);
        assert_eq!(started.cue_entry_id, Some(selected));
        owner.pause(&conn, true)?;
        assert_eq!(
            db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?
                .expect("session")
                .cue_entry_id,
            Some(selected)
        );
        owner.pause(&conn, false)?;
        owner.stop(&conn)?;
        let stopped = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        assert_eq!(stopped.state, "stopped");
        assert_eq!(stopped.position_ms, 0);
        assert_eq!(
            db::show_cue(&conn)?.expect("cue").selected_entry_id,
            Some(selected)
        );
        Ok(())
    }

    #[test]
    fn show_start_failure_keeps_the_session_unchanged() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let missing_path = temp.path().join("missing-show.mp3");
        let track_id = create_track(&conn, feed_id, "missing", temp.path(), &missing_path)?;
        let playlist_id = db::playlist_create(&conn, "Show")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let before = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?;
        let mut owner =
            PlaybackOwner::new(NullDriver::new(), playback::DEFAULT_SESSION_ID, temp.path());

        let error = owner
            .start_show_entry(&conn, None)
            .expect_err("missing cue file must fail");

        let entry_id = db::show_cue(&conn)?
            .expect("cue")
            .selected_entry_id
            .expect("entry");
        let typed = error
            .downcast_ref::<ShowCueEntryCannotPlay>()
            .expect("typed start error");
        assert_eq!(typed.entry_id, entry_id);
        assert_eq!(owner.generation(), 0);
        assert_eq!(owner.active_cue_entry_id(), None);
        assert!(before.is_none());
        assert!(db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.is_none());
        Ok(())
    }

    #[test]
    fn cue_eof_stops_once_and_preserves_the_cue() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let audio_path = temp.path().join("show.mp3");
        fs::write(&audio_path, b"audio")?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(&conn, feed_id, "eof", temp.path(), &audio_path)?;
        let playlist_id = db::playlist_create(&conn, "Show")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let selected = db::show_cue(&conn)?.expect("cue").selected_entry_id;
        let mut owner =
            PlaybackOwner::new(EndOfTrackDriver, playback::DEFAULT_SESSION_ID, temp.path());
        owner.start_show_entry(&conn, None)?;

        assert!(matches!(owner.poll(&conn)?, PollOutcome::Stopped));
        let stopped = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        let generation = owner.generation();
        assert_eq!(stopped.state, "stopped");
        assert_eq!(
            db::show_cue(&conn)?.expect("cue").selected_entry_id,
            selected
        );
        assert!(matches!(owner.poll(&conn)?, PollOutcome::Reconciled(None)));
        assert_eq!(owner.generation(), generation);
        assert_eq!(
            db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?
                .expect("session")
                .sequence,
            stopped.sequence
        );
        Ok(())
    }

    #[test]
    fn cue_eof_advances_once_and_ignores_a_duplicate_observation() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let audio_path = temp.path().join("show.mp3");
        fs::write(&audio_path, b"audio")?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(&conn, feed_id, "eof-two", temp.path(), &audio_path)?;
        let playlist_id = db::playlist_create(&conn, "Two entry cue")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let entries = db::show_cue(&conn)?.expect("cue").entries;
        let mut owner =
            PlaybackOwner::new(EndOfTrackDriver, playback::DEFAULT_SESSION_ID, temp.path());
        owner.start_show_entry(&conn, Some(entries[0].id))?;

        assert!(matches!(owner.poll(&conn)?, PollOutcome::Advanced(_)));
        let advanced_generation = owner.generation();
        let advanced = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        assert_eq!(advanced.cue_entry_id, Some(entries[1].id));
        assert!(matches!(owner.poll(&conn)?, PollOutcome::Reconciled(None)));
        assert_eq!(owner.generation(), advanced_generation);
        assert_eq!(
            db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?
                .expect("session")
                .cue_entry_id,
            Some(entries[1].id)
        );
        Ok(())
    }

    struct EndOfTrackDriver;

    impl PlaybackDriver for EndOfTrackDriver {
        fn load(&self, _path: &Path, _start_ms: u64) -> Result<()> {
            Ok(())
        }
        fn seek(&self, _position_ms: u64) -> Result<()> {
            Ok(())
        }
        fn pause(&self, _paused: bool) -> Result<()> {
            Ok(())
        }
        fn stop(&self) -> Result<()> {
            Ok(())
        }
        fn poll(&self) -> Result<DriverStatus> {
            Ok(DriverStatus {
                eof: true,
                ..DriverStatus::default()
            })
        }
    }

    #[test]
    fn owner_pause_removes_drop_file_but_keeps_paused_now_playing() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let audio_path = tagged_audio_file(&temp, "track.mp3")?;
        let track_id = create_track(&conn, feed_id, "item-guid", temp.path(), &audio_path)?;
        let producer = DropFileProducer::new(temp.path(), "default")?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            temp.path().to_path_buf(),
        )
        .with_drop_file_producer(Some(producer));

        owner.load_track_path(&conn, track_id, &audio_path, 0)?;
        let drop_path = owner
            .drop_file_producer()
            .expect("producer")
            .path()
            .to_owned();
        assert!(drop_path.exists());

        owner.pause(&conn, true)?;
        let row = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        let current = playback::now_playing_update(&conn, playback::DEFAULT_SESSION_ID)?
            .expect("paused sessions still have display state");
        let snapshot = ApplicationQueryService::new()
            .playback_snapshot(&conn, playback::DEFAULT_SESSION_ID)
            .expect("playback VM snapshot");

        assert_eq!(row.state, "paused");
        assert_eq!(current.local_track_id, track_id);
        assert!(snapshot.is_active());
        assert!(snapshot.is_paused());
        assert_eq!(snapshot.title(), Some("Track item-guid"));
        assert!(!drop_path.exists());
        Ok(())
    }

    #[test]
    fn owner_shutdown_cleanup_warns_and_removes_drop_file() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let audio_path = tagged_audio_file(&temp, "track.mp3")?;
        let track_id = create_track(&conn, feed_id, "item-guid", temp.path(), &audio_path)?;
        let producer = DropFileProducer::new(temp.path(), "default")?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            temp.path().to_path_buf(),
        )
        .with_drop_file_producer(Some(producer));

        assert_eq!(
            owner.broadcast_drop_file_shutdown_warning(),
            Some(SHUTDOWN_WARNING)
        );
        owner.load_track_path(&conn, track_id, &audio_path, 0)?;
        let drop_path = owner
            .drop_file_producer()
            .expect("producer")
            .path()
            .to_owned();
        assert!(drop_path.exists());

        assert!(owner.clear_broadcast_drop_file()?);
        assert!(!drop_path.exists());
        Ok(())
    }

    #[test]
    fn show_next_moves_by_cue_entry_identity() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let first_path = temp.path().join("first.mp3");
        fs::write(&first_path, b"audio")?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let first_track_id = create_track(&conn, feed_id, "same-guid", temp.path(), &first_path)?;
        let playlist_id = db::playlist_create(&conn, "Show cue")?;
        db::playlist_append(&conn, playlist_id, first_track_id)?;
        db::playlist_append(&conn, playlist_id, first_track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let entries = db::show_cue(&conn)?.expect("cue").entries;
        let mut owner =
            PlaybackOwner::new(NullDriver::new(), playback::DEFAULT_SESSION_ID, temp.path());
        owner.start_show_entry(&conn, Some(entries[0].id))?;

        let update = owner.move_show_cue(&conn, true)?.expect("active move");

        assert_eq!(update.local_track_id, first_track_id);
        assert_eq!(owner.active_cue_entry_id(), Some(entries[1].id));
        assert_eq!(
            db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?
                .expect("session")
                .cue_entry_id,
            Some(entries[1].id)
        );
        Ok(())
    }

    #[test]
    fn paused_show_move_stops_silently_and_resume_loads_prepared_entry() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let audio_path = temp.path().join("track.mp3");
        fs::write(&audio_path, b"audio")?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(&conn, feed_id, "same", temp.path(), &audio_path)?;
        let playlist_id = db::playlist_create(&conn, "Repeated cue")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        let entries = db::show_cue(&conn)?.expect("cue").entries;
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut owner = PlaybackOwner::new(
            RecordingDriver(Arc::clone(&calls)),
            playback::DEFAULT_SESSION_ID,
            temp.path(),
        );
        owner.start_show_entry(&conn, Some(entries[0].id))?;
        owner.pause(&conn, true)?;
        calls.lock().expect("calls").clear();

        owner.move_show_cue(&conn, true)?;
        assert_eq!(*calls.lock().expect("calls"), ["stop"]);
        let paused = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        assert_eq!(paused.state, "paused");
        assert_eq!(paused.cue_entry_id, Some(entries[1].id));
        assert!(matches!(owner.poll(&conn)?, PollOutcome::Reconciled(None)));
        assert_eq!(*calls.lock().expect("calls"), ["stop"]);

        owner.pause(&conn, false)?;
        assert_eq!(*calls.lock().expect("calls"), ["stop", "load"]);
        assert_eq!(
            db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?
                .expect("session")
                .state,
            "playing"
        );
        Ok(())
    }

    #[test]
    fn stopped_show_move_changes_only_the_selection_and_stays_inside_the_cue() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let audio_path = temp.path().join("track.mp3");
        fs::write(&audio_path, b"audio")?;
        let mut conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(&conn, feed_id, "same", temp.path(), &audio_path)?;
        let playlist_id = db::playlist_create(&conn, "Stopped cue")?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::playlist_append(&conn, playlist_id, track_id)?;
        db::load_show_cue(&mut conn, playlist_id, 1, false)?;
        // The source playlist changes after the load. The cue does not follow it.
        db::playlist_append(&conn, playlist_id, track_id)?;
        let entries = db::show_cue(&conn)?.expect("cue").entries;
        assert_eq!(entries.len(), 2);
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut owner = PlaybackOwner::new(
            RecordingDriver(Arc::clone(&calls)),
            playback::DEFAULT_SESSION_ID,
            temp.path(),
        );
        let selected = |conn: &Connection| -> Result<Option<i64>> {
            Ok(db::show_cue(conn)?.expect("cue").selected_entry_id)
        };

        assert!(owner.move_show_cue(&conn, false).is_err());
        assert_eq!(selected(&conn)?, Some(entries[0].id));
        assert!(owner.move_show_cue(&conn, true)?.is_none());
        assert_eq!(selected(&conn)?, Some(entries[1].id));
        assert!(owner.move_show_cue(&conn, true).is_err());
        assert_eq!(selected(&conn)?, Some(entries[1].id));

        assert!(db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.is_none());
        assert_eq!(owner.generation(), 0);
        assert!(calls.lock().expect("calls").is_empty());
        Ok(())
    }

    fn cue_with_a_missing_second_file(
        conn: &mut Connection,
        music_dir: &Path,
    ) -> Result<Vec<db::CueEntry>> {
        let audio_path = music_dir.join("first.mp3");
        fs::write(&audio_path, b"audio")?;
        let feed_id = create_feed(conn)?;
        let first = create_track(conn, feed_id, "first", music_dir, &audio_path)?;
        let second = create_track(
            conn,
            feed_id,
            "second",
            music_dir,
            &music_dir.join("missing.mp3"),
        )?;
        let playlist_id = db::playlist_create(conn, "Missing second")?;
        db::playlist_append(conn, playlist_id, first)?;
        db::playlist_append(conn, playlist_id, second)?;
        db::load_show_cue(conn, playlist_id, 1, false)?;
        Ok(db::show_cue(conn)?.expect("cue").entries)
    }

    #[test]
    fn show_move_to_an_entry_that_cannot_play_keeps_the_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut conn = setup_test_db()?;
        let entries = cue_with_a_missing_second_file(&mut conn, temp.path())?;
        let mut owner =
            PlaybackOwner::new(NullDriver::new(), playback::DEFAULT_SESSION_ID, temp.path());
        owner.start_show_entry(&conn, Some(entries[0].id))?;
        let generation = owner.generation();

        let error = owner
            .move_show_cue(&conn, true)
            .expect_err("the target file is missing");

        let typed = error
            .downcast_ref::<ShowCueEntryCannotPlay>()
            .expect("typed move error");
        assert_eq!(typed.entry_id, entries[1].id);
        assert_eq!(typed.reason, ShowCueEntryBlock::FileMissing);
        assert_eq!(owner.generation(), generation);
        let session = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        assert_eq!(session.state, "playing");
        assert_eq!(session.cue_entry_id, Some(entries[0].id));
        Ok(())
    }

    #[test]
    fn cue_eof_before_an_entry_that_cannot_play_stops_and_names_the_entry() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut conn = setup_test_db()?;
        let entries = cue_with_a_missing_second_file(&mut conn, temp.path())?;
        let mut owner =
            PlaybackOwner::new(EndOfTrackDriver, playback::DEFAULT_SESSION_ID, temp.path());
        owner.start_show_entry(&conn, Some(entries[0].id))?;

        let error = owner.poll(&conn).expect_err("the next file is missing");

        let typed = error
            .downcast_ref::<ShowCueEntryCannotPlay>()
            .expect("typed end of track error");
        assert_eq!(typed.entry_id, entries[1].id);
        let session = db::playback_session(&conn, playback::DEFAULT_SESSION_ID)?.expect("session");
        assert_eq!(session.state, "stopped");
        assert_eq!(db::show_cue(&conn)?.expect("cue").entries.len(), 2);
        Ok(())
    }

    struct RecordingDriver(Arc<Mutex<Vec<&'static str>>>);

    impl PlaybackDriver for RecordingDriver {
        fn load(&self, _path: &Path, _start_ms: u64) -> Result<()> {
            self.0.lock().expect("calls").push("load");
            Ok(())
        }
        fn seek(&self, _position_ms: u64) -> Result<()> {
            self.0.lock().expect("calls").push("seek");
            Ok(())
        }
        fn pause(&self, _paused: bool) -> Result<()> {
            self.0.lock().expect("calls").push("pause");
            Ok(())
        }
        fn stop(&self) -> Result<()> {
            self.0.lock().expect("calls").push("stop");
            Ok(())
        }
        fn poll(&self) -> Result<DriverStatus> {
            self.0.lock().expect("calls").push("poll");
            Ok(DriverStatus::default())
        }
    }

    #[test]
    fn owner_load_current_uses_stored_path_and_position() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(
            &conn,
            feed_id,
            "item-guid",
            Path::new("/"),
            Path::new("/tmp/track.mp3"),
        )?;
        playback::set_track(&conn, track_id, playback::DEFAULT_SESSION_ID)?;
        playback::update_position(&conn, 9_000, playback::DEFAULT_SESSION_ID)?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            PathBuf::from("/"),
        );

        owner.load_current_session(&conn)?;
        let snap = owner.driver().snapshot();

        assert_eq!(snap.loaded_path, Some(PathBuf::from("/tmp/track.mp3")));
        assert_eq!(snap.position_ms, 9_000);
        Ok(())
    }

    #[test]
    fn owner_poll_loads_new_active_session_before_reconciling() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(
            &conn,
            feed_id,
            "item-guid",
            Path::new("/"),
            Path::new("/tmp/track.mp3"),
        )?;
        playback::set_track(&conn, track_id, playback::DEFAULT_SESSION_ID)?;
        playback::update_position(&conn, 5_000, playback::DEFAULT_SESSION_ID)?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            PathBuf::from("/"),
        );

        let outcome = owner.poll(&conn)?;
        let snap = owner.driver().snapshot();

        assert!(matches!(outcome, PollOutcome::Reconciled(Some(_))));
        assert_eq!(snap.loaded_path, Some(PathBuf::from("/tmp/track.mp3")));
        assert_eq!(snap.position_ms, 5_000);
        Ok(())
    }

    #[test]
    fn owner_poll_stops_driver_when_session_is_stopped() -> Result<()> {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(
            &conn,
            feed_id,
            "item-guid",
            Path::new("/"),
            Path::new("/tmp/track.mp3"),
        )?;
        playback::set_track(&conn, track_id, playback::DEFAULT_SESSION_ID)?;
        let mut owner = PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            PathBuf::from("/"),
        );
        owner.load_current_session(&conn)?;
        playback::stop(&conn, playback::DEFAULT_SESSION_ID)?;

        let outcome = owner.poll(&conn)?;

        assert!(matches!(outcome, PollOutcome::Stopped));
        assert!(owner.driver().snapshot().stopped);
        assert!(matches!(owner.poll(&conn)?, PollOutcome::Reconciled(None)));
        Ok(())
    }
}
