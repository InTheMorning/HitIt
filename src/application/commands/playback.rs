//! Playback command family.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::application::events::playback::PlaybackEvent;
use crate::application::events::ApplicationEvent;
use crate::db;
use crate::playback::NowPlayingUpdate;
use crate::playback_driver::PlaybackDriver;
use crate::playback_owner::PlaybackOwner;

type SharedConnection = Arc<Mutex<Connection>>;
type SharedPlaybackOwner<D> = Arc<Mutex<PlaybackOwner<D>>>;

/// Command result for playback transport actions.
#[derive(Clone, Debug)]
pub struct PlaybackCommandResult {
    update: Option<NowPlayingUpdate>,
    message: String,
}

impl PlaybackCommandResult {
    /// Creates a playback command result with a now-playing update.
    #[must_use]
    pub fn with_update(update: NowPlayingUpdate, message: impl Into<String>) -> Self {
        Self {
            update: Some(update),
            message: message.into(),
        }
    }

    /// Creates a playback command result without a now-playing update.
    #[must_use]
    pub fn without_update(message: impl Into<String>) -> Self {
        Self {
            update: None,
            message: message.into(),
        }
    }

    /// Returns the now-playing update when the command produced one.
    #[must_use]
    pub const fn update(&self) -> Option<&NowPlayingUpdate> {
        self.update.as_ref()
    }

    /// Returns the user-facing playback status message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Plays one local library track.
pub struct PlayTrack<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
    track_id: i64,
    start_ms: u64,
}

impl<D> PlayTrack<D> {
    /// Creates a local track playback command.
    #[must_use]
    pub fn new(
        conn: SharedConnection,
        playback_owner: SharedPlaybackOwner<D>,
        track_id: i64,
        start_ms: u64,
    ) -> Self {
        Self {
            conn,
            playback_owner,
            track_id,
            start_ms,
        }
    }
}

impl<D> ApplicationCommand for PlayTrack<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        context.validate_retry_subject(&conn)?;
        let mut owner = self
            .playback_owner
            .lock()
            .map_err(|_| playback_error("playback owner lock poisoned"))?;
        let update = owner
            .load_track(&conn, self.track_id, self.start_ms)
            .map_err(playback_error)?;
        Ok(CommandOutcome::new(
            PlaybackCommandResult::with_update(update.clone(), playing_message(&update)),
            playback_changed_events(),
        ))
    }
}

/// Moves to the next Show cue entry.
pub struct NextShowCueEntry<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
}

impl<D> NextShowCueEntry<D> {
    /// Creates a next Show cue entry command.
    #[must_use]
    pub fn new(conn: SharedConnection, playback_owner: SharedPlaybackOwner<D>) -> Self {
        Self {
            conn,
            playback_owner,
        }
    }
}

impl<D> ApplicationCommand for NextShowCueEntry<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        context.validate_retry_subject(&conn)?;
        let mut owner = self
            .playback_owner
            .lock()
            .map_err(|_| playback_error("playback owner lock poisoned"))?;
        let update = owner.move_show_cue(&conn, true).map_err(playback_error)?;
        Ok(CommandOutcome::new(
            update.as_ref().map_or_else(
                || PlaybackCommandResult::without_update("Selected the next Show cue entry"),
                |update| {
                    PlaybackCommandResult::with_update(update.clone(), playing_message(update))
                },
            ),
            playback_changed_events(),
        ))
    }
}

/// Moves to the previous Show cue entry.
pub struct PreviousShowCueEntry<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
}

impl<D> PreviousShowCueEntry<D> {
    /// Creates a previous Show cue entry command.
    #[must_use]
    pub fn new(conn: SharedConnection, playback_owner: SharedPlaybackOwner<D>) -> Self {
        Self {
            conn,
            playback_owner,
        }
    }
}

impl<D> ApplicationCommand for PreviousShowCueEntry<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        context.validate_retry_subject(&conn)?;
        let mut owner = self
            .playback_owner
            .lock()
            .map_err(|_| playback_error("playback owner lock poisoned"))?;
        let update = owner.move_show_cue(&conn, false).map_err(playback_error)?;
        Ok(CommandOutcome::new(
            update.as_ref().map_or_else(
                || PlaybackCommandResult::without_update("Selected the previous Show cue entry"),
                |update| {
                    PlaybackCommandResult::with_update(update.clone(), playing_message(update))
                },
            ),
            playback_changed_events(),
        ))
    }
}

/// Pauses current playback.
pub struct PausePlayback<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
}

impl<D> PausePlayback<D> {
    /// Creates a pause command.
    #[must_use]
    pub fn new(conn: SharedConnection, playback_owner: SharedPlaybackOwner<D>) -> Self {
        Self {
            conn,
            playback_owner,
        }
    }
}

impl<D> ApplicationCommand for PausePlayback<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        set_paused(&self.conn, &self.playback_owner, true, context)
    }
}

/// Resumes current playback.
pub struct ResumePlayback<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
}

impl<D> ResumePlayback<D> {
    /// Creates a resume command.
    #[must_use]
    pub fn new(conn: SharedConnection, playback_owner: SharedPlaybackOwner<D>) -> Self {
        Self {
            conn,
            playback_owner,
        }
    }
}

impl<D> ApplicationCommand for ResumePlayback<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        set_paused(&self.conn, &self.playback_owner, false, context)
    }
}

/// Seeks current playback to an absolute position.
pub struct SeekPlayback<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
    position_ms: u64,
}

impl<D> SeekPlayback<D> {
    /// Creates a seek command.
    #[must_use]
    pub fn new(
        conn: SharedConnection,
        playback_owner: SharedPlaybackOwner<D>,
        position_ms: u64,
    ) -> Self {
        Self {
            conn,
            playback_owner,
            position_ms,
        }
    }
}

impl<D> ApplicationCommand for SeekPlayback<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        context.validate_retry_subject(&conn)?;
        let mut owner = self
            .playback_owner
            .lock()
            .map_err(|_| playback_error("playback owner lock poisoned"))?;
        let update = owner
            .seek(&conn, self.position_ms)
            .map_err(playback_error)?;
        Ok(CommandOutcome::new(
            PlaybackCommandResult::with_update(update.clone(), playing_message(&update)),
            playback_changed_events(),
        ))
    }
}

/// Stops current playback.
pub struct StopPlayback<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
}

/// Starts the selected Show cue entry or the named entry.
pub struct StartShow<D> {
    conn: SharedConnection,
    playback_owner: SharedPlaybackOwner<D>,
    entry_id: Option<i64>,
}

impl<D> StartShow<D> {
    #[must_use]
    pub fn new(
        conn: SharedConnection,
        playback_owner: SharedPlaybackOwner<D>,
        entry_id: Option<i64>,
    ) -> Self {
        Self {
            conn,
            playback_owner,
            entry_id,
        }
    }
}

impl<D: PlaybackDriver + 'static> ApplicationCommand for StartShow<D> {
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        let mut owner = self
            .playback_owner
            .lock()
            .map_err(|_| playback_error("playback owner lock poisoned"))?;
        let update = owner
            .start_show_entry(&conn, self.entry_id)
            .map_err(playback_error)?;
        Ok(CommandOutcome::new(
            PlaybackCommandResult::with_update(
                update.clone(),
                format!("Starting {}", update.title),
            ),
            playback_changed_events(),
        ))
    }
}

/// Selects a cue entry while Show is stopped.
pub struct SelectShowCueEntry {
    conn: SharedConnection,
    entry_id: i64,
}

impl SelectShowCueEntry {
    #[must_use]
    pub fn new(conn: SharedConnection, entry_id: i64) -> Self {
        Self { conn, entry_id }
    }
}

impl ApplicationCommand for SelectShowCueEntry {
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        db::select_show_cue_entry(&conn, self.entry_id).map_err(playback_error)?;
        Ok(CommandOutcome::new(
            PlaybackCommandResult::without_update("Show cue entry selected"),
            playback_changed_events(),
        ))
    }
}

impl<D> StopPlayback<D> {
    /// Creates a stop command.
    #[must_use]
    pub fn new(conn: SharedConnection, playback_owner: SharedPlaybackOwner<D>) -> Self {
        Self {
            conn,
            playback_owner,
        }
    }
}

/// Show-specific name for the pause command.
pub type PauseShow<D> = PausePlayback<D>;
/// Show-specific name for the resume command.
pub type ResumeShow<D> = ResumePlayback<D>;
/// Show-specific name for the stop command.
pub type StopShow<D> = StopPlayback<D>;

impl<D> ApplicationCommand for StopPlayback<D>
where
    D: PlaybackDriver + 'static,
{
    type Output = PlaybackCommandResult;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        ensure_not_cancelled(context)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| playback_error("database lock poisoned"))?;
        context.validate_retry_subject(&conn)?;
        let mut owner = self
            .playback_owner
            .lock()
            .map_err(|_| playback_error("playback owner lock poisoned"))?;
        owner.stop(&conn).map_err(playback_error)?;
        Ok(CommandOutcome::new(
            PlaybackCommandResult::without_update("Playback stopped"),
            playback_changed_events(),
        ))
    }
}

fn set_paused<D>(
    conn: &SharedConnection,
    playback_owner: &SharedPlaybackOwner<D>,
    paused: bool,
    context: &CommandContext,
) -> CommandResult<PlaybackCommandResult>
where
    D: PlaybackDriver + 'static,
{
    ensure_not_cancelled(context)?;
    let conn = conn
        .lock()
        .map_err(|_| playback_error("database lock poisoned"))?;
    context.validate_retry_subject(&conn)?;
    let mut owner = playback_owner
        .lock()
        .map_err(|_| playback_error("playback owner lock poisoned"))?;
    let update = owner.pause(&conn, paused).map_err(playback_error)?;
    let message = if paused {
        format!("Paused {}", update.title)
    } else {
        playing_message(&update)
    };
    Ok(CommandOutcome::new(
        PlaybackCommandResult::with_update(update, message),
        playback_changed_events(),
    ))
}

fn ensure_not_cancelled(context: &CommandContext) -> Result<(), CommandError> {
    if context.cancellation().is_cancelled() {
        return Err(CommandError::Cancelled);
    }
    Ok(())
}

fn playing_message(update: &NowPlayingUpdate) -> String {
    format!("Playing {}", update.title)
}

fn playback_changed_events() -> Vec<ApplicationEvent> {
    vec![ApplicationEvent::Playback(PlaybackEvent::Changed)]
}

fn playback_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Playback(format!("{error:#}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::application::command_bus::CommandBus;
    use crate::application::command_context::{CancellationToken, OperationId, TraceId};
    use crate::db;
    use crate::playback;
    use crate::playback_driver::NullDriver;

    fn setup_test_db() -> anyhow::Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
    }

    fn shared_conn() -> anyhow::Result<SharedConnection> {
        Ok(Arc::new(Mutex::new(setup_test_db()?)))
    }

    fn shared_owner() -> SharedPlaybackOwner<NullDriver> {
        Arc::new(Mutex::new(PlaybackOwner::new(
            NullDriver::new(),
            playback::DEFAULT_SESSION_ID,
            PathBuf::from("/"),
        )))
    }

    fn cancelled_context() -> CommandContext {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        CommandContext::new(OperationId::new(1), cancellation, TraceId::new(1))
    }

    fn create_feed(conn: &Connection) -> anyhow::Result<i64> {
        conn.execute(
            "INSERT INTO feeds (feed_url, feed_guid, title)
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://example.test/feed.xml", "feed-guid", "Feed Title"],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_track(
        conn: &Connection,
        feed_id: i64,
        item_guid: &str,
        path: &str,
    ) -> anyhow::Result<i64> {
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
        let relative_path = crate::library_path::LibraryRelativePath::for_test(path);
        db::mark_track_downloaded(conn, track_id, &relative_path, None)?;
        Ok(track_id)
    }

    fn create_playlist_track(conn: &Connection) -> anyhow::Result<(i64, i64)> {
        let feed_id = create_feed(conn)?;
        let track_id = create_track(conn, feed_id, "item-guid", "tmp/track.mp3")?;
        let playlist_id = db::playlist_create(conn, "Phase 2")?;
        db::playlist_append(conn, playlist_id, track_id)?;
        Ok((playlist_id, track_id))
    }

    #[test]
    fn pause_and_resume_playback_emit_events() -> anyhow::Result<()> {
        let conn = shared_conn()?;
        let owner = shared_owner();
        let track_id = {
            let conn = conn.lock().expect("test database lock");
            create_playlist_track(&conn)?.1
        };
        CommandBus::new().execute(
            PlayTrack::new(Arc::clone(&conn), Arc::clone(&owner), track_id, 0),
            &CommandContext::next(),
        )?;

        let pause = CommandBus::new().execute(
            PausePlayback::new(Arc::clone(&conn), Arc::clone(&owner)),
            &CommandContext::next(),
        )?;
        let resume = CommandBus::new().execute(
            ResumePlayback::new(Arc::clone(&conn), Arc::clone(&owner)),
            &CommandContext::next(),
        )?;

        assert_eq!(pause.events(), playback_changed_events());
        assert_eq!(pause.value().message(), "Paused Track item-guid");
        assert_eq!(resume.events(), playback_changed_events());
        assert_eq!(resume.value().message(), "Playing Track item-guid");
        Ok(())
    }

    #[test]
    fn stop_playback_emits_changed_without_update() -> anyhow::Result<()> {
        let conn = shared_conn()?;
        let owner = shared_owner();
        let track_id = {
            let conn = conn.lock().expect("test database lock");
            create_playlist_track(&conn)?.1
        };
        CommandBus::new().execute(
            PlayTrack::new(Arc::clone(&conn), Arc::clone(&owner), track_id, 0),
            &CommandContext::next(),
        )?;

        let stop = CommandBus::new().execute(
            StopPlayback::new(Arc::clone(&conn), Arc::clone(&owner)),
            &CommandContext::next(),
        )?;

        assert_eq!(stop.events(), playback_changed_events());
        assert!(stop.value().update().is_none());
        assert_eq!(stop.value().message(), "Playback stopped");
        Ok(())
    }

    #[test]
    fn play_track_honors_cancelled_context() {
        let error = CommandBus::new()
            .execute(
                PlayTrack::new(shared_conn().expect("db"), shared_owner(), 1, 0),
                &cancelled_context(),
            )
            .expect_err("cancelled playback should fail before local lookup");

        assert_eq!(error, CommandError::Cancelled);
    }
}
