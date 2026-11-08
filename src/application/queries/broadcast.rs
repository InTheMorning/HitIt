//! Broadcast readiness read model for ADR 0059.
//!
//! The query is read-only: it checks local library rows and embedded audio tag
//! state so Show and CLI surfaces can report payment-route readiness without
//! mutating files.
//!
//! ADR 0076 Decisions 7 and 9 add two not-ready states. A file route that
//! differs from the stored route is `RouteOutOfDate`. A track with an
//! unconfirmed "removed from feed" mark is `RemovedFromFeed`, and that state
//! comes before each route state.

#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::Serialize;

use crate::api::PaymentRoute;
use crate::application::application_query_service::ApplicationQueryService;
use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::audio_tags::{read_audio_tags, AudioTags};
use crate::db::{self, LocalMetadataOwner, LocalMetadataValue, TrackRow};
use crate::metadata::{
    audio_tags_have_ready_value_routes, audio_tags_value_routes, parse_value_routes,
    payment_routes_equal, MUSICINDEX_METADATA_SOURCE, MUSICINDEX_PAYMENT_ROUTES_ABSENT_FACT_KEY,
};
use crate::{config, library_service};

/// Readiness report for local library tracks before a show.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct BroadcastReadinessReport {
    /// Count summary by readiness state.
    pub(crate) summary: BroadcastReadinessSummary,
    /// Every scanned library track with its readiness state.
    pub(crate) tracks: Vec<BroadcastReadinessTrack>,
}

impl BroadcastReadinessReport {
    /// Returns tracks that are not ready for broadcast payment routing.
    #[must_use]
    pub(crate) fn problem_tracks(&self) -> Vec<&BroadcastReadinessTrack> {
        self.tracks
            .iter()
            .filter(|track| !matches!(track.state, BroadcastReadinessState::Ready))
            .collect()
    }

    /// Returns the number of tracks that need curator attention.
    #[must_use]
    pub(crate) const fn problem_count(&self) -> usize {
        self.summary.no_route_tag
            + self.summary.no_routes_upstream
            + self.summary.route_out_of_date
            + self.summary.removed_from_feed
            + self.summary.file_missing
            + self.summary.not_downloaded
    }
}

/// Count summary by readiness state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct BroadcastReadinessSummary {
    /// Tracks with a non-empty parsed payment-route array.
    pub(crate) ready: usize,
    /// Tracks whose local file lacks a usable value-routes tag.
    pub(crate) no_route_tag: usize,
    /// Tracks whose upstream `MusicIndex` data has no payment routes.
    pub(crate) no_routes_upstream: usize,
    /// Tracks whose recorded local path cannot be read.
    pub(crate) file_missing: usize,
    /// Tracks in the library with no downloaded file.
    pub(crate) not_downloaded: usize,
    /// Tracks whose file route differs from the stored route.
    pub(crate) route_out_of_date: usize,
    /// Tracks with an unconfirmed "removed from feed" mark.
    pub(crate) removed_from_feed: usize,
}

impl BroadcastReadinessSummary {
    fn record(&mut self, state: BroadcastReadinessState) {
        match state {
            BroadcastReadinessState::Ready => self.ready += 1,
            BroadcastReadinessState::NoRouteTag => self.no_route_tag += 1,
            BroadcastReadinessState::NoRoutesUpstream => self.no_routes_upstream += 1,
            BroadcastReadinessState::FileMissing => self.file_missing += 1,
            BroadcastReadinessState::NotDownloaded => self.not_downloaded += 1,
            BroadcastReadinessState::RouteOutOfDate => self.route_out_of_date += 1,
            BroadcastReadinessState::RemovedFromFeed => self.removed_from_feed += 1,
        }
    }
}

/// Per-track broadcast payment-route readiness state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BroadcastReadinessState {
    /// Local file contains a usable embedded payment-route array.
    Ready,
    /// Local file has no usable embedded payment-route tag.
    NoRouteTag,
    /// `MusicIndex` has no payment routes for this track or its feed.
    NoRoutesUpstream,
    /// The library row has no downloaded file at all.
    NotDownloaded,
    /// Local path is missing or does not point to a readable file.
    FileMissing,
    /// The file carries a route array that differs from the stored route.
    RouteOutOfDate,
    /// The track has the "removed from feed" mark, and the operator has not
    /// confirmed it.
    RemovedFromFeed,
}

impl BroadcastReadinessState {
    /// Returns the compact row label for this state.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::NoRouteTag => "Missing routes",
            Self::NoRoutesUpstream => "No upstream routes",
            Self::FileMissing => "Missing file",
            Self::NotDownloaded => "Not downloaded",
            Self::RouteOutOfDate => "Route out of date",
            Self::RemovedFromFeed => "Removed from feed",
        }
    }
}

/// The reason of a `RouteOutOfDate` track (ADR 0076 Decision 9).
pub(crate) const ROUTE_OUT_OF_DATE_REASON: &str = "Payment route in file is out of date.";

/// The reason of a track whose stored route has no recipient.
pub(crate) const NO_RSS_RECIPIENTS_REASON: &str =
    "RSS has no payment recipients for this track or feed.";

const NO_UPSTREAM_ROUTES_REASON: &str = "Music Index has no payment routes for this track or feed.";

/// The reason of a `RemovedFromFeed` track (ADR 0076 Decision 7), with the
/// recorded time of the check that set the mark, in UTC.
fn removed_from_feed_reason(removed_at_us: i64) -> String {
    let at = chrono::DateTime::from_timestamp_micros(removed_at_us).map_or_else(
        || "an unknown time".to_owned(),
        |at| at.format("%Y-%m-%d %H:%M UTC").to_string(),
    );
    format!("Removed from feed on {at}. Confirm to play it, or remove it from the playlist.")
}

/// One local library track classified for broadcast readiness.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct BroadcastReadinessTrack {
    /// Local track database id.
    pub(crate) track_id: i64,
    /// Listener-facing track title fallback.
    pub(crate) title: String,
    /// Artist display fallback.
    pub(crate) artist: Option<String>,
    /// Album display fallback.
    pub(crate) album: Option<String>,
    /// Recorded local path, if the database has one.
    pub(crate) path: Option<String>,
    /// Readiness classification.
    pub(crate) state: BroadcastReadinessState,
    /// Curator-facing reason for this classification.
    pub(crate) reason: String,
}

impl ApplicationQueryService {
    /// Builds the broadcast readiness report from local library rows and files.
    ///
    /// # Errors
    ///
    /// Returns an error when local library rows cannot be read. Unreadable tag
    /// data is classified as not ready.
    #[expect(
        clippy::unused_self,
        reason = "ApplicationQueryService uses an instance receiver across query families"
    )]
    pub(crate) fn broadcast_readiness_report(
        &self,
        conn: &Connection,
        music_dir: &Path,
    ) -> Result<BroadcastReadinessReport, CommandError> {
        broadcast_readiness_report_for_music_dir(conn, music_dir)
            .map_err(|error| CommandError::Query(format!("{error:#}")))
    }
}

/// Builds the broadcast readiness report from local library rows and files.
///
/// # Errors
///
/// Returns an error when local library rows cannot be read. Unreadable tag data
/// is classified as not ready.
/// Plans the readiness report with the configured music directory. Call it
/// under the database lock, then read the files with the lock released
/// (task 039, step 1).
///
/// # Errors
///
/// Returns an error when the configuration or the stored rows cannot be read.
pub(crate) fn broadcast_readiness_plan_from_config(
    conn: &Connection,
) -> Result<BroadcastReadinessPlan> {
    let cfg_path = config::config_path()?;
    let music_dir = config::ConfigSnapshot::read_existing(&cfg_path)?.music_dir?;
    broadcast_readiness_plan(conn, &music_dir)
}

/// Builds the broadcast readiness report with a configured music directory.
///
/// # Errors
///
/// Returns an error when local library rows cannot be read. Unreadable tag data
/// is classified as not ready.
pub(crate) fn broadcast_readiness_report_for_music_dir(
    conn: &Connection,
    music_dir: &Path,
) -> Result<BroadcastReadinessReport> {
    Ok(broadcast_readiness_plan(conn, music_dir)?.read_files())
}

/// The database part of the readiness report (task 039, step 2). The plan
/// reads the rows and the stored facts. The file part of each track is
/// read later, with no database connection.
#[derive(Debug)]
pub(crate) struct BroadcastReadinessPlan {
    tracks: Vec<PlannedReadiness>,
}

/// One track of the plan. A track that needs no file read is decided
/// already.
#[derive(Debug)]
enum PlannedReadiness {
    Decided(BroadcastReadinessTrack),
    TagRead(Box<TagReadPlan>),
}

/// The stored facts of one track whose embedded tags must be read.
#[derive(Debug)]
struct TagReadPlan {
    track: TrackRow,
    title: String,
    path: PathBuf,
    stored: Option<Vec<PaymentRoute>>,
    no_routes: Option<&'static str>,
}

/// Plans the broadcast readiness report of the Library. Call it under the
/// database lock, then release the lock before [`BroadcastReadinessPlan::read_files`].
///
/// # Errors
///
/// Returns an error when local library rows or stored facts cannot be read.
pub(crate) fn broadcast_readiness_plan(
    conn: &Connection,
    music_dir: &Path,
) -> Result<BroadcastReadinessPlan> {
    let tracks = library_service::library_tracks(conn).context("load local library tracks")?;
    let tracks = tracks
        .iter()
        .map(|track| plan_readiness_track(conn, track, music_dir))
        .collect::<Result<Vec<_>>>()?;
    Ok(BroadcastReadinessPlan { tracks })
}

impl BroadcastReadinessPlan {
    /// Reads the embedded tags of each planned file. It takes no database
    /// connection, so it runs with the database lock free (task 039, step 2).
    #[must_use]
    pub(crate) fn read_files(self) -> BroadcastReadinessReport {
        let mut report = BroadcastReadinessReport::default();
        for planned in self.tracks {
            let item = match planned {
                PlannedReadiness::Decided(item) => item,
                PlannedReadiness::TagRead(plan) => plan.read(),
            };
            report.summary.record(item.state);
            report.tracks.push(item);
        }
        report
    }
}

impl TagReadPlan {
    fn read(self) -> BroadcastReadinessTrack {
        let Self {
            track,
            title,
            path,
            stored,
            no_routes,
        } = self;
        let no_routes_upstream = no_routes.is_some();
        match read_audio_tags(&path).context("read embedded audio tags") {
            Ok(tags) if audio_tags_have_ready_value_routes(&tags) => {
                readiness_track_from_tags(&track, title, path.as_path(), &tags, stored.as_deref())
            }
            Ok(tags) => {
                readiness_track_from_missing_tag(&track, title, path.as_path(), no_routes, &tags)
            }
            Err(error) => BroadcastReadinessTrack {
                track_id: track.id,
                title,
                artist: track.artist_name.clone(),
                album: track.album_title.clone(),
                path: Some(path.display().to_string()),
                state: if no_routes_upstream {
                    BroadcastReadinessState::NoRoutesUpstream
                } else {
                    BroadcastReadinessState::NoRouteTag
                },
                reason: no_routes.map_or_else(
                    || format!("Value routes tag could not be read: {error:#}"),
                    str::to_owned,
                ),
            },
        }
    }
}

fn plan_readiness_track(
    conn: &Connection,
    track: &TrackRow,
    music_dir: &Path,
) -> Result<PlannedReadiness> {
    let title = track
        .track_title
        .clone()
        .or_else(|| track.feed_title.clone())
        .unwrap_or_else(|| "Untitled".to_string());
    if let Some(removed_at_us) = db::rss_field_holds::unconfirmed_removed_at(conn, track.id)? {
        return Ok(PlannedReadiness::Decided(BroadcastReadinessTrack {
            track_id: track.id,
            title,
            artist: track.artist_name.clone(),
            album: track.album_title.clone(),
            path: track
                .local_path
                .as_ref()
                .map(|path| path.resolve(music_dir).display().to_string()),
            state: BroadcastReadinessState::RemovedFromFeed,
            reason: removed_from_feed_reason(removed_at_us),
        }));
    }
    let Some(path) = track
        .local_path
        .as_ref()
        .map(|path| path.resolve(music_dir))
    else {
        return Ok(PlannedReadiness::Decided(BroadcastReadinessTrack {
            track_id: track.id,
            title,
            artist: track.artist_name.clone(),
            album: track.album_title.clone(),
            path: None,
            // `library_tracks` uses a LEFT JOIN, so a library row with no
            // download reaches here. The file is not missing. It was never
            // downloaded, and the operator fixes it with a download.
            state: BroadcastReadinessState::NotDownloaded,
            reason: "Track is in the library and has no downloaded file.".to_owned(),
        }));
    };

    if !path.is_file() {
        return Ok(PlannedReadiness::Decided(BroadcastReadinessTrack {
            track_id: track.id,
            title,
            artist: track.artist_name.clone(),
            album: track.album_title.clone(),
            path: Some(path.display().to_string()),
            state: BroadcastReadinessState::FileMissing,
            reason: "Recorded local file is missing.".to_owned(),
        }));
    }

    let stored = db::payment_routes::stored_route(conn, track.id)?;
    let no_routes = match stored.as_deref() {
        Some([]) => Some(NO_RSS_RECIPIENTS_REASON),
        Some(_) => None,
        None => {
            track_has_no_upstream_payment_routes(conn, track)?.then_some(NO_UPSTREAM_ROUTES_REASON)
        }
    };
    Ok(PlannedReadiness::TagRead(Box::new(TagReadPlan {
        track: track.clone(),
        title,
        path,
        stored,
        no_routes,
    })))
}

/// The readiness report and the show playlist track ids, for the Inbox
/// (task 039, step 2). The command reads the rows under a short lock and
/// the file tags with no lock.
#[derive(Clone, Debug)]
pub(crate) struct LoadBroadcastReadiness {
    conn: Arc<Mutex<Connection>>,
    music_dir: PathBuf,
}

impl LoadBroadcastReadiness {
    /// Creates the readiness load for the Library connection and `music_dir`.
    #[must_use]
    pub(crate) const fn new(conn: Arc<Mutex<Connection>>, music_dir: PathBuf) -> Self {
        Self { conn, music_dir }
    }
}

/// The output of [`LoadBroadcastReadiness`].
#[derive(Clone, Debug)]
pub(crate) struct BroadcastReadinessLoad {
    pub(crate) report: BroadcastReadinessReport,
    /// The track ids of the show playlist, in its sequence. Empty with no
    /// playback session, or when the session names no playlist.
    pub(crate) source_track_ids: Vec<i64>,
}

impl ApplicationCommand for LoadBroadcastReadiness {
    type Output = BroadcastReadinessLoad;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let (plan, source_track_ids) = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| CommandError::Other("database lock poisoned".to_string()))?;
            let plan = broadcast_readiness_plan(&conn, &self.music_dir)
                .map_err(|error| CommandError::Query(format!("{error:#}")))?;
            (plan, show_source_track_ids(&conn))
        };
        Ok(CommandOutcome::without_events(BroadcastReadinessLoad {
            report: plan.read_files(),
            source_track_ids,
        }))
    }
}

/// The track ids of the show playlist, in its sequence (ADR 0089 Decision 3,
/// Answer 3). The show playlist is the playlist of the playback session that
/// is not stopped.
fn show_source_track_ids(conn: &Connection) -> Vec<i64> {
    let playlist_id = db::playback_session(conn, crate::playback::DEFAULT_SESSION_ID)
        .ok()
        .flatten()
        .filter(|session| session.state != "stopped")
        .and_then(|session| session.playlist_id);
    let Some(playlist_id) = playlist_id else {
        return Vec::new();
    };
    ApplicationQueryService::new()
        .playlist_tracks(conn, playlist_id)
        .map(|rows| rows.into_iter().map(|row| row.id).collect())
        .unwrap_or_default()
}

fn readiness_track_from_tags(
    track: &TrackRow,
    title: String,
    path: &Path,
    tags: &AudioTags,
    stored: Option<&[PaymentRoute]>,
) -> BroadcastReadinessTrack {
    // ADR 0076 Decision 9: every recipient field counts, and the order does
    // not. Without a stored route the app has nothing to compare.
    let out_of_date = stored.is_some_and(|stored| {
        audio_tags_value_routes(tags)
            .and_then(parse_value_routes)
            .is_some_and(|file| !payment_routes_equal(&file, stored))
    });
    let (state, reason) = if out_of_date {
        (
            BroadcastReadinessState::RouteOutOfDate,
            ROUTE_OUT_OF_DATE_REASON,
        )
    } else {
        (
            BroadcastReadinessState::Ready,
            "Embedded value routes are present.",
        )
    };
    BroadcastReadinessTrack {
        track_id: track.id,
        title,
        artist: track.artist_name.clone(),
        album: track.album_title.clone(),
        path: Some(path.display().to_string()),
        state,
        reason: reason.to_owned(),
    }
}

fn readiness_track_from_missing_tag(
    track: &TrackRow,
    title: String,
    path: &Path,
    no_routes: Option<&str>,
    tags: &AudioTags,
) -> BroadcastReadinessTrack {
    let (state, reason) = if let Some(reason) = no_routes {
        (BroadcastReadinessState::NoRoutesUpstream, reason.to_owned())
    } else if audio_tags_value_routes(tags).is_some() {
        (
            BroadcastReadinessState::NoRouteTag,
            "Embedded value routes are empty or invalid.".to_owned(),
        )
    } else {
        (
            BroadcastReadinessState::NoRouteTag,
            "Embedded MusicIndex Value Routes tag is missing.".to_owned(),
        )
    };

    BroadcastReadinessTrack {
        track_id: track.id,
        title,
        artist: track.artist_name.clone(),
        album: track.album_title.clone(),
        path: Some(path.display().to_string()),
        state,
        reason,
    }
}

pub(crate) fn track_has_no_upstream_payment_routes(
    conn: &Connection,
    track: &TrackRow,
) -> Result<bool> {
    Ok(
        local_payment_routes_absent(conn, LocalMetadataOwner::Track(track.id))?
            || local_payment_routes_absent(conn, LocalMetadataOwner::Feed(track.feed_id))?,
    )
}

fn local_payment_routes_absent(conn: &Connection, owner: LocalMetadataOwner) -> Result<bool> {
    let fact = db::local_metadata_fact(
        conn,
        owner,
        MUSICINDEX_METADATA_SOURCE,
        MUSICINDEX_PAYMENT_ROUTES_ABSENT_FACT_KEY,
    )?;
    Ok(fact.is_some_and(|fact| matches!(fact.value, LocalMetadataValue::Boolean(true))))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use rusqlite::Connection;

    use super::*;
    use crate::audio_tags::{write_id3v24_edits, Id3v24Edit};
    use crate::db;

    fn setup_test_db() -> anyhow::Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
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
        music_dir: &Path,
        feed_id: i64,
        title: &str,
        path: &Path,
    ) -> anyhow::Result<i64> {
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, track_title, artist_name, album_title)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                feed_id,
                format!("item-guid-{title}"),
                title,
                "Track Artist",
                "Track Album"
            ],
        )?;
        let track_id = conn.last_insert_rowid();
        let relative_path =
            crate::library_path::LibraryRelativePath::from_absolute(music_dir, path)?;
        library_service::mark_track_downloaded(conn, track_id, &relative_path, None)?;
        Ok(track_id)
    }

    fn tagged_audio_file(
        temp: &tempfile::TempDir,
        name: &str,
        routes: &str,
    ) -> anyhow::Result<PathBuf> {
        let path = temp.path().join(name);
        fs::write(&path, b"not really an mp3")?;
        write_id3v24_edits(
            &path,
            &[Id3v24Edit {
                frame_label: crate::metadata::MUSICINDEX_VALUE_ROUTES_FRAME.to_owned(),
                value: routes.to_owned(),
            }],
        )?;
        Ok(path)
    }

    fn untagged_audio_file(temp: &tempfile::TempDir, name: &str) -> anyhow::Result<PathBuf> {
        let path = temp.path().join(name);
        fs::write(&path, b"not really an mp3")?;
        Ok(path)
    }

    #[test]
    fn broadcast_readiness_report_marks_non_empty_value_routes_ready() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = tagged_audio_file(
            &temp,
            "ready.mp3",
            r#"[{"recipient_name":"Artist","route_type":"node","split":100.0}]"#,
        )?;
        let track_id = create_track(&conn, temp.path(), feed_id, "Ready Track", &path)?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(report.summary.ready, 1);
        assert_eq!(report.summary.no_route_tag, 0);
        assert_eq!(report.summary.file_missing, 0);
        assert_eq!(report.tracks[0].track_id, track_id);
        assert_eq!(report.tracks[0].state, BroadcastReadinessState::Ready);
        Ok(())
    }

    #[test]
    fn broadcast_readiness_report_marks_missing_value_routes_tag() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = untagged_audio_file(&temp, "missing-tag.mp3")?;
        create_track(&conn, temp.path(), feed_id, "Missing Tag", &path)?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(report.summary.ready, 0);
        assert_eq!(report.summary.no_route_tag, 1);
        assert_eq!(report.summary.no_routes_upstream, 0);
        assert_eq!(report.summary.file_missing, 0);
        assert_eq!(report.tracks[0].state, BroadcastReadinessState::NoRouteTag);
        Ok(())
    }

    #[test]
    fn broadcast_readiness_report_counts_no_routes_upstream_separately() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = untagged_audio_file(&temp, "no-upstream-routes.mp3")?;
        let track_id = create_track(&conn, temp.path(), feed_id, "No Upstream Routes", &path)?;
        db::replace_local_metadata_fact(
            &conn,
            LocalMetadataOwner::Track(track_id),
            MUSICINDEX_METADATA_SOURCE,
            &db::LocalMetadataFactInput {
                fact_key: MUSICINDEX_PAYMENT_ROUTES_ABSENT_FACT_KEY.to_owned(),
                value: LocalMetadataValue::Boolean(true),
                extraction_path: Some("$.payment_routes".to_owned()),
                observed_at: None,
                raw_json: None,
            },
        )?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(report.summary.ready, 0);
        assert_eq!(report.summary.no_route_tag, 0);
        assert_eq!(report.summary.no_routes_upstream, 1);
        assert_eq!(report.summary.file_missing, 0);
        assert_eq!(report.problem_count(), 1);
        assert_eq!(
            report.tracks[0].state,
            BroadcastReadinessState::NoRoutesUpstream
        );
        Ok(())
    }

    #[test]
    fn broadcast_readiness_report_marks_empty_route_array_not_ready() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = tagged_audio_file(&temp, "empty-routes.mp3", "[]")?;
        create_track(&conn, temp.path(), feed_id, "Empty Routes", &path)?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(report.summary.ready, 0);
        assert_eq!(report.summary.no_route_tag, 1);
        assert_eq!(report.summary.file_missing, 0);
        assert_eq!(report.tracks[0].state, BroadcastReadinessState::NoRouteTag);
        Ok(())
    }

    /// A library row with no download is not a missing file. An operator saw
    /// "54 missing files" on 2026-09-08 while every file was present, because
    /// `library_tracks` LEFT JOINs `local_files` and both cases shared one state.
    #[test]
    fn broadcast_readiness_report_separates_not_downloaded_from_missing_file() -> anyhow::Result<()>
    {
        let conn = setup_test_db()?;
        let feed_id = create_feed(&conn)?;
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, track_title, artist_name, album_title, is_in_library)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            rusqlite::params![
                feed_id,
                "item-guid-never-downloaded",
                "Never Downloaded",
                "Track Artist",
                "Track Album"
            ],
        )?;

        let report = ApplicationQueryService::new()
            .broadcast_readiness_report(&conn, std::path::Path::new("/tmp"))?;

        assert_eq!(report.summary.not_downloaded, 1);
        assert_eq!(
            report.summary.file_missing, 0,
            "a library row with no download is not a missing file"
        );
        assert_eq!(
            report.tracks[0].state,
            BroadcastReadinessState::NotDownloaded
        );
        Ok(())
    }

    #[test]
    fn broadcast_readiness_report_marks_recorded_missing_file() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = temp.path().join("missing.mp3");
        create_track(&conn, temp.path(), feed_id, "Missing File", &path)?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(report.summary.ready, 0);
        assert_eq!(report.summary.no_route_tag, 0);
        assert_eq!(report.summary.file_missing, 1);
        assert_eq!(report.tracks[0].state, BroadcastReadinessState::FileMissing);
        Ok(())
    }

    #[test]
    fn broadcast_readiness_report_allows_empty_library() -> anyhow::Result<()> {
        let conn = setup_test_db()?;

        let report = ApplicationQueryService::new()
            .broadcast_readiness_report(&conn, std::path::Path::new("/tmp"))?;

        assert_eq!(report.summary, BroadcastReadinessSummary::default());
        assert!(report.tracks.is_empty());
        Ok(())
    }

    const STORED: &str = r#"[{"recipient_name":"Band","route_type":"node","split":90.0,"fee":false,"address":"a"},{"recipient_name":"App","route_type":"node","split":10.0,"fee":false,"address":"b"}]"#;

    fn set_stored_route(conn: &Connection, track_id: i64, routes: &str) -> anyhow::Result<()> {
        conn.execute(
            "UPDATE tracks SET payment_routes_json = ?2 WHERE id = ?1",
            rusqlite::params![track_id, routes],
        )?;
        Ok(())
    }

    fn state_of(report: &BroadcastReadinessReport, track_id: i64) -> BroadcastReadinessState {
        report
            .tracks
            .iter()
            .find(|track| track.track_id == track_id)
            .map(|track| track.state)
            .expect("track in report")
    }

    /// R3-06: a split difference and a name-only difference are
    /// `RouteOutOfDate`. An equal route in another recipient order is `Ready`.
    #[test]
    fn adr_0076_route_readiness_route_comparison_counts_every_field_and_ignores_order(
    ) -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let cases = [
            (
                "split.mp3",
                r#"[{"recipient_name":"Band","route_type":"node","split":95.0,"address":"a"},{"recipient_name":"App","route_type":"node","split":5.0,"address":"b"}]"#,
                BroadcastReadinessState::RouteOutOfDate,
            ),
            (
                "name.mp3",
                r#"[{"recipient_name":"The Band","route_type":"node","split":90.0,"address":"a"},{"recipient_name":"App","route_type":"node","split":10.0,"address":"b"}]"#,
                BroadcastReadinessState::RouteOutOfDate,
            ),
            (
                "order.mp3",
                r#"[{"recipient_name":"App","route_type":"node","split":10.0,"fee":false,"address":"b"},{"recipient_name":"Band","route_type":"node","split":90.0,"fee":false,"address":"a"}]"#,
                BroadcastReadinessState::Ready,
            ),
        ];
        let mut expected = Vec::new();
        for (name, routes, state) in cases {
            let path = tagged_audio_file(&temp, name, routes)?;
            let track_id = create_track(&conn, temp.path(), feed_id, name, &path)?;
            set_stored_route(&conn, track_id, STORED)?;
            expected.push((track_id, state));
        }

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        for (track_id, state) in expected {
            assert_eq!(state_of(&report, track_id), state, "track {track_id}");
        }
        let out_of_date = report
            .tracks
            .iter()
            .find(|track| track.state == BroadcastReadinessState::RouteOutOfDate)
            .expect("out-of-date track");
        assert_eq!(out_of_date.reason, ROUTE_OUT_OF_DATE_REASON);
        assert_eq!(report.summary.route_out_of_date, 2);
        assert_eq!(report.summary.ready, 1);
        Ok(())
    }

    /// R3-07 and R3-08: an unconfirmed removed track is `RemovedFromFeed`
    /// before its route state. After the confirmation it shows its route
    /// state.
    #[test]
    fn adr_0076_route_readiness_removed_track_comes_before_route_state() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = tagged_audio_file(
            &temp,
            "removed.mp3",
            r#"[{"recipient_name":"Other","route_type":"node","split":100.0,"address":"z"}]"#,
        )?;
        let track_id = create_track(&conn, temp.path(), feed_id, "Removed", &path)?;
        set_stored_route(&conn, track_id, STORED)?;
        // 2026-09-24 10:00:00 UTC, the recorded time of a check.
        conn.execute(
            "UPDATE tracks SET removed_from_feed_at = 1790244000000000 WHERE id = ?1",
            [track_id],
        )?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;
        assert_eq!(
            report.tracks[0].state,
            BroadcastReadinessState::RemovedFromFeed
        );
        assert_eq!(
            report.tracks[0].reason,
            "Removed from feed on 2026-09-24 10:00 UTC. Confirm to play it, or remove it from the playlist."
        );
        assert_eq!(report.summary.removed_from_feed, 1);
        assert_eq!(report.summary.route_out_of_date, 0);

        assert!(db::rss_field_holds::confirm_removed_track(
            &conn,
            track_id,
            1_790_244_100_000_000
        )?);
        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;
        assert_eq!(
            report.tracks[0].state,
            BroadcastReadinessState::RouteOutOfDate
        );
        assert_eq!(report.summary.removed_from_feed, 0);
        Ok(())
    }

    /// R3-09: the summary and `problem_count` include both new states.
    #[test]
    fn adr_0076_route_readiness_summary_counts_both_new_states() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let old = tagged_audio_file(
            &temp,
            "old.mp3",
            r#"[{"recipient_name":"Other","route_type":"node","split":100.0,"address":"z"}]"#,
        )?;
        let old_id = create_track(&conn, temp.path(), feed_id, "Old", &old)?;
        set_stored_route(&conn, old_id, STORED)?;
        let removed = tagged_audio_file(&temp, "removed.mp3", STORED)?;
        let removed_id = create_track(&conn, temp.path(), feed_id, "Removed", &removed)?;
        set_stored_route(&conn, removed_id, STORED)?;
        conn.execute(
            "UPDATE tracks SET removed_from_feed_at = 1790244000000000 WHERE id = ?1",
            [removed_id],
        )?;
        let ready = tagged_audio_file(&temp, "ready.mp3", STORED)?;
        let ready_id = create_track(&conn, temp.path(), feed_id, "Ready", &ready)?;
        set_stored_route(&conn, ready_id, STORED)?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(report.summary.route_out_of_date, 1);
        assert_eq!(report.summary.removed_from_feed, 1);
        assert_eq!(report.summary.ready, 1);
        assert_eq!(report.problem_count(), 2);
        assert_eq!(report.problem_tracks().len(), 2);
        Ok(())
    }

    /// A stored route with no recipient means that RSS has no route. A file
    /// without a route tag is then "no upstream routes", with an RSS reason.
    #[test]
    fn adr_0076_route_readiness_empty_stored_route_needs_publisher_routes() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = untagged_audio_file(&temp, "none.mp3")?;
        let track_id = create_track(&conn, temp.path(), feed_id, "None", &path)?;
        set_stored_route(&conn, track_id, "[]")?;

        let report =
            ApplicationQueryService::new().broadcast_readiness_report(&conn, temp.path())?;

        assert_eq!(
            report.tracks[0].state,
            BroadcastReadinessState::NoRoutesUpstream
        );
        assert_eq!(report.tracks[0].reason, NO_RSS_RECIPIENTS_REASON);
        Ok(())
    }

    /// ADR 0084 Decision 10 as amended 2026-10-24, task 039 (step 2). The
    /// file part of the readiness report reads the tags with the database
    /// lock held by this test. Only the plan step reads the rows.
    #[test]
    fn adr_0084_task_039_readiness_files_are_read_with_the_lock_free() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let temp = tempfile::tempdir()?;
        let feed_id = create_feed(&conn)?;
        let path = tagged_audio_file(
            &temp,
            "ready.mp3",
            r#"[{"recipient_name":"Artist","route_type":"node","split":100.0}]"#,
        )?;
        create_track(&conn, temp.path(), feed_id, "Ready Track", &path)?;
        let shared = Mutex::new(conn);

        let plan = broadcast_readiness_plan(&shared.lock().unwrap(), temp.path())?;
        let _held = shared.lock().unwrap();
        let report = plan.read_files();

        assert_eq!(report.summary.ready, 1, "the file read needs no connection");
        Ok(())
    }
}
