//! Queue and now-playing frame display contracts.
//!
//! ADR 0046 Phase 4 moves detailed playback controls out of the global toolbar
//! and into the Queue workspace frame. ADR 0060 moves that queue into Show and
//! removes output controls that have no command path. This module keeps those
//! display contracts GPUI-free so the shell can bind them to primitives without
//! owning playback state.

#![warn(clippy::pedantic)]

use crate::view_models::format::fmt_duration;
use crate::view_models::text_filter::{contains_normalized, normalize};
use crate::view_models::workspace::FrameChromeButtonDisplay;

/// Transport state for the queue frame.
///
/// The state intentionally stores only command availability facts. Playback
/// engines and session handles remain in the application layer that dispatches
/// commands.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum TransportState {
    /// No playable queue item is active.
    #[default]
    Empty,
    /// A cue is loaded and playback is stopped.
    Stopped,
    /// A start command was accepted, but playback is not yet observed.
    Starting,
    /// Playback is currently advancing.
    Playing,
    /// Playback is active but paused.
    Paused,
    /// Playback failed and needs an explicit retry.
    Failed,
}

impl TransportState {
    /// Returns whether transport commands should be enabled.
    #[must_use]
    pub(crate) const fn is_active(self) -> bool {
        matches!(self, Self::Starting | Self::Playing | Self::Paused)
    }

    /// Returns the visible play/pause command label.
    #[must_use]
    pub(crate) const fn play_pause_label(self) -> &'static str {
        match self {
            Self::Playing => "Pause",
            Self::Paused => "Resume",
            Self::Stopped | Self::Empty => "Start",
            Self::Failed => "Retry",
            Self::Starting => "Starting",
        }
    }

    /// Returns the play/pause command accessibility label.
    #[must_use]
    pub(crate) const fn play_pause_a11y_label(self) -> &'static str {
        match self {
            Self::Playing => "Pause Show",
            Self::Paused => "Resume Show",
            Self::Stopped | Self::Failed => "Start Show",
            Self::Empty => "No Show cue is loaded",
            Self::Starting => "Starting Show",
        }
    }
}

/// The Show intent of the play and pause control and of its key (ADR 0068).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShowToggleIntent {
    Start,
    Pause,
    Resume,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShowActionAvailability {
    Available,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShowActionDisplay {
    pub(crate) availability: ShowActionAvailability,
    pub(crate) label: &'static str,
    pub(crate) a11y_label: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShowActionsDisplay {
    pub(crate) start: ShowActionDisplay,
    pub(crate) pause: ShowActionDisplay,
    pub(crate) resume: ShowActionDisplay,
    pub(crate) stop: ShowActionDisplay,
    pub(crate) select: ShowActionDisplay,
    pub(crate) clear: ShowActionDisplay,
}

impl ShowActionsDisplay {
    fn for_state(state: TransportState, cue_can_clear: bool) -> Self {
        let available = |condition| {
            if condition {
                ShowActionAvailability::Available
            } else {
                ShowActionAvailability::Unavailable
            }
        };
        let (start, pause, resume, stop, select) = match state {
            TransportState::Empty | TransportState::Starting => (false, false, false, false, false),
            TransportState::Stopped => (true, false, false, false, true),
            TransportState::Playing => (false, true, false, true, false),
            TransportState::Paused => (false, false, true, true, false),
            TransportState::Failed => (true, false, false, true, false),
        };
        Self {
            start: ShowActionDisplay {
                availability: available(start),
                label: "Start",
                a11y_label: "Start Show",
            },
            pause: ShowActionDisplay {
                availability: available(pause),
                label: "Pause",
                a11y_label: "Pause Show",
            },
            resume: ShowActionDisplay {
                availability: available(resume),
                label: "Resume",
                a11y_label: "Resume Show",
            },
            stop: ShowActionDisplay {
                availability: available(stop),
                label: "Stop",
                a11y_label: "Stop Show",
            },
            select: ShowActionDisplay {
                availability: available(select),
                label: "Select",
                a11y_label: "Select Show cue entry",
            },
            clear: ShowActionDisplay {
                availability: available(
                    cue_can_clear
                        && matches!(state, TransportState::Stopped | TransportState::Failed),
                ),
                label: "Clear",
                a11y_label: "Clear Show cue",
            },
        }
    }
}

/// Plain queue-track input projected by the application layer.
///
/// This input prevents DB rows from becoming part of the queue-frame VM public
/// surface. Callers extract only the facts the frame needs and the VM owns the
/// resulting display labels.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct QueueTrackInput {
    /// Stable local track identifier.
    pub(crate) id: i64,
    /// Cue entry identifier when the row belongs to a stored cue.
    pub(crate) entry_id: Option<i64>,
    /// Whether this is the selected cue entry.
    pub(crate) selected: bool,
    /// Whether a stopped Show can select this entry.
    pub(crate) selectable: bool,
    /// Why a stored cue entry cannot play, when known.
    pub(crate) unavailable_reason: Option<QueueUnavailableReason>,
    /// Primary track title.
    pub(crate) title: Option<String>,
    /// Secondary artist label.
    pub(crate) artist: Option<String>,
    /// Track duration in seconds.
    pub(crate) duration_seconds: Option<i64>,
    /// Whether this row represents the active playback item.
    pub(crate) now_playing: bool,
    /// Play position of the active entry in milliseconds, from the session.
    pub(crate) position_ms: Option<i64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The database reports Library and download facts. This enum adds disk facts.
pub(crate) enum QueueUnavailableReason {
    NotInLibrary,
    NotDownloaded,
    MissingFile,
}

impl QueueUnavailableReason {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::NotInLibrary => "Not in Library",
            Self::NotDownloaded => "Not downloaded",
            Self::MissingFile => "File missing",
        }
    }
}

/// Display-ready queue row.
///
/// The shell renders rows without checking playback state or formatting
/// duration values. Unavailable or now-playing state is already named here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueueRowDisplay {
    pub(crate) track_id: i64,
    pub(crate) entry_id: Option<i64>,
    pub(crate) selected: bool,
    /// Whether a stopped Show can select this entry.
    pub(crate) selectable: bool,
    /// Stable row element identifier.
    pub(crate) id: String,
    /// Primary row title.
    pub(crate) title: String,
    /// Optional secondary artist label.
    pub(crate) artist: Option<String>,
    /// Optional clock-style duration label.
    pub(crate) duration_label: Option<String>,
    /// Whether this row is the active now-playing item.
    pub(crate) now_playing: bool,
    /// Why this cue entry cannot play, when known.
    pub(crate) unavailable_label: Option<&'static str>,
    /// Accessibility label summarizing the row.
    pub(crate) a11y_label: String,
    /// Place of the entry in the whole cue, from 0 (ADR 0106 Decision 2).
    /// A text filter does not change it.
    pub(crate) cue_index: usize,
    /// "Played", or "Playing" with the time left, while the Show plays or is
    /// paused (ADR 0106 Decision 1).
    pub(crate) state_label: Option<String>,
    /// Whether a drag, Alt+Up and Alt+Down or "Play next" can move this entry
    /// now (ADR 0106 Decisions 1 and 2).
    pub(crate) cue_move: CueRowMove,
    /// Track duration in seconds, for the time left of the playing entry.
    pub(crate) duration_seconds: Option<i64>,
}

/// The move availability of one cue row (ADR 0106 Decisions 1 and 2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CueRowMove {
    /// The row does not move: it played, it plays, or it has no cue entry.
    Locked,
    /// The row moves. It is the next row, so "Play next" does not show.
    Movable,
    /// The row moves, and "Play next" shows on it.
    MovableWithPlayNext,
}

impl QueueRowDisplay {
    /// Whether a drag or Alt+Up and Alt+Down can move this entry now.
    #[must_use]
    pub(crate) const fn can_move(&self) -> bool {
        !matches!(self.cue_move, CueRowMove::Locked)
    }

    /// Whether "Play next" shows on this row: a row after the next row,
    /// while the Show plays or is paused.
    #[must_use]
    pub(crate) const fn play_next(&self) -> bool {
        matches!(self.cue_move, CueRowMove::MovableWithPlayNext)
    }

    /// Projects a queue-track input into display-ready row data.
    #[must_use]
    pub(crate) fn from_track(input: QueueTrackInput) -> Self {
        let title = input
            .title
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Untitled track".to_string());
        let duration_label = input
            .duration_seconds
            .filter(|seconds| *seconds > 0)
            .map(fmt_duration);
        let a11y_label = queue_row_a11y_label(
            &title,
            input.artist.as_deref(),
            duration_label.as_deref(),
            input.now_playing,
        );
        let unavailable_label = input.unavailable_reason.map(QueueUnavailableReason::label);
        let a11y_label = unavailable_label.map_or(a11y_label.clone(), |reason| {
            format!("{a11y_label}. Unavailable: {reason}")
        });

        Self {
            track_id: input.id,
            entry_id: input.entry_id,
            selected: input.selected,
            selectable: input.selectable,
            id: input.entry_id.map_or_else(
                || format!("queue-row-{}", input.id),
                |entry_id| format!("queue-entry-{entry_id}"),
            ),
            title,
            artist: input.artist,
            duration_label,
            now_playing: input.now_playing,
            unavailable_label,
            a11y_label,
            cue_index: 0,
            state_label: None,
            cue_move: CueRowMove::Locked,
            duration_seconds: input.duration_seconds,
        }
    }
}

/// The text below the cue while the Show plays or is paused (ADR 0106
/// Decision 2).
pub(crate) const CUE_MOVE_HINT: &str =
    "Drag a row that did not play, or select \"Play next\". The playing track does not stop.";

/// Where a moved cue entry goes (ADR 0106 Decision 2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CueMoveTarget {
    /// The place of the row with this cue index.
    Index(usize),
    /// The place directly after the current entry ("Play next").
    AfterCurrent,
}

/// Gives each cue row its place, its play state and its move availability
/// (ADR 0106 Decisions 1 and 2). `position_ms` is the play position of the
/// current entry.
fn mark_cue_rows(rows: &mut [QueueRowDisplay], state: TransportState, position_ms: Option<i64>) {
    let current = rows.iter().position(|row| row.now_playing);
    let active = matches!(state, TransportState::Playing | TransportState::Paused);
    for (index, row) in rows.iter_mut().enumerate() {
        row.cue_index = index;
        if row.entry_id.is_none() {
            continue;
        }
        if !active {
            if matches!(state, TransportState::Stopped) {
                row.cue_move = CueRowMove::Movable;
            }
            continue;
        }
        let Some(current) = current else {
            continue;
        };
        row.cue_move = match index.checked_sub(current) {
            Some(0) | None => CueRowMove::Locked,
            Some(1) => CueRowMove::Movable,
            Some(_) => CueRowMove::MovableWithPlayNext,
        };
        if index < current {
            row.state_label = Some("Played".to_owned());
        } else if index == current {
            row.state_label = Some(playing_label(row.duration_seconds, position_ms));
        }
    }
}

fn playing_label(duration_seconds: Option<i64>, position_ms: Option<i64>) -> String {
    match (duration_seconds.filter(|seconds| *seconds > 0), position_ms) {
        (Some(duration), Some(position)) => {
            let left = (duration - position / 1000).max(0);
            format!("Playing, {} left", fmt_duration(left))
        }
        _ => "Playing".to_owned(),
    }
}

/// Display-ready transport command state.
///
/// The queue shell maps these display facts to icon buttons. Command handlers
/// are supplied separately by the application shell so this VM stays pure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransportDisplay {
    /// Stable play/pause control identifier.
    pub(crate) play_pause_id: String,
    /// Visible play/pause label.
    pub(crate) play_pause_label: &'static str,
    /// Accessibility label for the play/pause command.
    pub(crate) play_pause_a11y_label: &'static str,
    /// Current transport state.
    pub(crate) play_pause_state: TransportState,
    /// Previous-track command display.
    pub(crate) skip_previous: FrameChromeButtonDisplay,
    /// Next-track command display.
    pub(crate) skip_next: FrameChromeButtonDisplay,
    /// Stop command display.
    pub(crate) stop: FrameChromeButtonDisplay,
    /// Whether transport commands should render unavailable.
    pub(crate) disabled: bool,
}

impl TransportDisplay {
    /// Creates a transport display with independent skip availability.
    #[must_use]
    pub(crate) fn from_state_with_skip_availability(
        state: TransportState,
        can_skip_previous: bool,
        can_skip_next: bool,
    ) -> Self {
        let disabled = matches!(state, TransportState::Empty | TransportState::Starting);
        Self {
            play_pause_id: "queue-transport-playpause".to_string(),
            play_pause_label: state.play_pause_label(),
            play_pause_a11y_label: state.play_pause_a11y_label(),
            play_pause_state: state,
            skip_previous: FrameChromeButtonDisplay::new(
                "queue-transport-previous",
                "Previous track",
                !can_skip_previous,
            ),
            skip_next: FrameChromeButtonDisplay::new(
                "queue-transport-next",
                "Next track",
                !can_skip_next,
            ),
            stop: FrameChromeButtonDisplay::new(
                "queue-transport-stop",
                "Stop Show",
                !matches!(
                    state,
                    TransportState::Playing | TransportState::Paused | TransportState::Failed
                ),
            ),
            disabled,
        }
    }
}

/// Display-ready Queue/Now Playing page.
///
/// The page groups the queue rows with transport controls. It contains no GPUI
/// values and no playback engine handles.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct QueueNowPlayingPageVm {
    /// Ordered queue rows.
    pub(crate) rows: Vec<QueueRowDisplay>,
    all_rows: Vec<QueueRowDisplay>,
    text_filter: Option<String>,
    can_skip_previous: bool,
    can_skip_next: bool,
    /// Transport control display state.
    pub(crate) transport: TransportDisplay,
    pub(crate) actions: ShowActionsDisplay,
    /// A cue is stored, and the stored session is stopped. The store permits a Clear
    /// only then. A state that the app shows in place of the stored state does not change it.
    cue_can_clear: bool,
    /// The text below the cue while the Show plays or is paused (ADR 0106
    /// Decision 2).
    pub(crate) cue_move_hint: Option<&'static str>,
    /// Empty-state label for the queue list.
    pub(crate) empty_label: &'static str,
}

impl QueueNowPlayingPageVm {
    pub(crate) fn set_transport_state(&mut self, state: TransportState) {
        self.transport = TransportDisplay::from_state_with_skip_availability(
            state,
            self.can_skip_previous,
            self.can_skip_next,
        );
        self.actions = ShowActionsDisplay::for_state(state, self.cue_can_clear);
    }

    pub(crate) fn track_ids(&self) -> Vec<i64> {
        self.all_rows.iter().map(|row| row.track_id).collect()
    }

    pub(crate) fn current_track_id(&self) -> Option<i64> {
        self.all_rows
            .iter()
            .find(|row| row.now_playing)
            .map(|row| row.track_id)
    }

    /// Gives the available Show intent of the toggle with its accessibility label.
    /// It gives `None` when no cue is loaded and while a start is in progress.
    pub(crate) fn toggle_intent(&self) -> Option<(ShowToggleIntent, &'static str)> {
        [
            (ShowToggleIntent::Start, &self.actions.start),
            (ShowToggleIntent::Pause, &self.actions.pause),
            (ShowToggleIntent::Resume, &self.actions.resume),
        ]
        .into_iter()
        .find(|(_, action)| action.availability == ShowActionAvailability::Available)
        .map(|(intent, action)| (intent, action.a11y_label))
    }

    pub(crate) fn can_stop_show(&self) -> bool {
        self.actions.stop.availability == ShowActionAvailability::Available
    }

    pub(crate) fn can_clear_show_cue(&self) -> bool {
        self.actions.clear.availability == ShowActionAvailability::Available
    }

    pub(crate) fn can_skip_previous(&self) -> bool {
        self.can_skip_previous
    }

    pub(crate) fn can_skip_next(&self) -> bool {
        self.can_skip_next
    }

    /// Gives the cue index that Alt+Up or Alt+Down moves the entry to. It gives
    /// `None` when the entry cannot move now or the move leaves the movable
    /// rows (ADR 0106 Decision 1).
    pub(crate) fn key_move_index(&self, entry_id: i64, up: bool) -> Option<usize> {
        let row = self
            .all_rows
            .iter()
            .find(|row| row.entry_id == Some(entry_id) && row.can_move())?;
        let to = if up {
            row.cue_index.checked_sub(1)?
        } else {
            row.cue_index + 1
        };
        self.all_rows
            .get(to)
            .filter(|target| target.can_move())
            .map(|target| target.cue_index)
    }

    /// Whether the cue has no entry. A text filter does not change it
    /// (ADR 0106 Decision 4).
    #[must_use]
    pub(crate) fn cue_is_empty(&self) -> bool {
        self.all_rows.is_empty()
    }

    /// Whether each cue entry is ready to play. A text filter does not change it.
    #[must_use]
    pub(crate) fn cue_all_available(&self) -> bool {
        self.all_rows
            .iter()
            .all(|row| row.unavailable_label.is_none())
    }

    /// The title of the selected cue entry. A text filter does not change it.
    #[must_use]
    pub(crate) fn selected_cue_title(&self) -> Option<&str> {
        self.all_rows
            .iter()
            .find(|row| row.selected)
            .map(|row| row.title.as_str())
    }

    pub(crate) fn selected_entry_id(&self) -> Option<i64> {
        self.all_rows
            .iter()
            .find(|row| row.selected)
            .and_then(|row| row.entry_id)
    }
    pub(crate) fn apply_playback_availability(
        &mut self,
        availability: Result<(), crate::application::capability::ExecutionUnavailable>,
    ) {
        if availability.is_err() {
            self.transport.play_pause_label = "Repair playback";
            self.transport.play_pause_a11y_label = "Repair playback for the original track";
            self.transport.skip_previous.a11y_label =
                "Repair playback before the original previous-track action";
            self.transport.skip_next.a11y_label =
                "Repair playback before the original next-track action";
        }
    }
    /// Creates a queue page builder.
    pub(crate) fn builder() -> QueueNowPlayingPageVmBuilder {
        QueueNowPlayingPageVmBuilder::default()
    }

    /// Returns the active queue text filter, if any.
    ///
    /// Only this type's own tests read the filter back; the live queue shell
    /// renders from `rows` instead. Compiled only for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn text_filter(&self) -> Option<&str> {
        self.text_filter.as_deref()
    }

    /// Updates the queue text filter and visible row projection.
    pub(crate) fn set_text_filter(&mut self, filter: Option<String>) {
        self.text_filter = normalize(filter);
        self.rows = match self.text_filter.as_deref() {
            Some(filter) => self
                .all_rows
                .iter()
                .filter(|row| queue_row_matches_text_filter(row, filter))
                .cloned()
                .collect(),
            None => self.all_rows.clone(),
        };
    }
}

/// Builder for [`QueueNowPlayingPageVm`].
#[derive(Clone, Debug)]
#[must_use]
pub(crate) struct QueueNowPlayingPageVmBuilder {
    tracks: Vec<QueueTrackInput>,
    transport_state: TransportState,
    can_skip_previous: bool,
    can_skip_next: bool,
    has_cue: bool,
}

impl Default for QueueNowPlayingPageVmBuilder {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            transport_state: TransportState::Empty,
            can_skip_previous: false,
            can_skip_next: false,
            has_cue: false,
        }
    }
}

impl QueueNowPlayingPageVmBuilder {
    /// Supplies ordered queue-track inputs.
    pub(crate) fn tracks(mut self, tracks: impl IntoIterator<Item = QueueTrackInput>) -> Self {
        self.tracks = tracks.into_iter().collect();
        self
    }

    /// Supplies current transport state.
    pub(crate) const fn transport_state(mut self, state: TransportState) -> Self {
        self.transport_state = state;
        self
    }

    /// Supplies skip command availability.
    pub(crate) const fn skip_availability(
        mut self,
        can_skip_previous: bool,
        can_skip_next: bool,
    ) -> Self {
        self.can_skip_previous = can_skip_previous;
        self.can_skip_next = can_skip_next;
        self
    }

    pub(crate) const fn has_cue(mut self, has_cue: bool) -> Self {
        self.has_cue = has_cue;
        self
    }

    /// Projects the builder input into a display-ready page.
    #[must_use]
    pub(crate) fn build(self) -> QueueNowPlayingPageVm {
        let position_ms = self
            .tracks
            .iter()
            .find(|track| track.now_playing)
            .and_then(|track| track.position_ms);
        let mut rows = self
            .tracks
            .into_iter()
            .map(QueueRowDisplay::from_track)
            .collect::<Vec<_>>();
        mark_cue_rows(&mut rows, self.transport_state, position_ms);
        let cue_move_hint = (self.has_cue
            && matches!(
                self.transport_state,
                TransportState::Playing | TransportState::Paused
            ))
        .then_some(CUE_MOVE_HINT);
        let cue_can_clear = self.has_cue && matches!(self.transport_state, TransportState::Stopped);
        QueueNowPlayingPageVm {
            all_rows: rows.clone(),
            rows,
            text_filter: None,
            can_skip_previous: self.can_skip_previous,
            can_skip_next: self.can_skip_next,
            transport: TransportDisplay::from_state_with_skip_availability(
                self.transport_state,
                self.can_skip_previous,
                self.can_skip_next,
            ),
            actions: ShowActionsDisplay::for_state(self.transport_state, cue_can_clear),
            cue_can_clear,
            cue_move_hint,
            empty_label: "Queue is empty",
        }
    }
}

fn queue_row_matches_text_filter(row: &QueueRowDisplay, filter: &str) -> bool {
    contains_normalized(&row.title, filter)
        || row
            .artist
            .as_deref()
            .is_some_and(|artist| contains_normalized(artist, filter))
        || row
            .duration_label
            .as_deref()
            .is_some_and(|duration| contains_normalized(duration, filter))
        || contains_normalized(&row.a11y_label, filter)
}

fn queue_row_a11y_label(
    title: &str,
    artist: Option<&str>,
    duration: Option<&str>,
    now_playing: bool,
) -> String {
    let mut parts = Vec::new();
    if now_playing {
        parts.push("Now playing".to_string());
    }
    parts.push(title.to_string());
    if let Some(artist) = artist {
        if !artist.trim().is_empty() {
            parts.push(format!("by {artist}"));
        }
    }
    if let Some(duration) = duration {
        parts.push(duration.to_string());
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_reasons_have_distinct_labels() {
        assert_eq!(
            QueueUnavailableReason::NotDownloaded.label(),
            "Not downloaded"
        );
        assert_eq!(QueueUnavailableReason::MissingFile.label(), "File missing");
    }

    fn track(id: i64, title: &str, now_playing: bool) -> QueueTrackInput {
        QueueTrackInput {
            id,
            entry_id: None,
            selected: false,
            selectable: false,
            unavailable_reason: None,
            title: Some(title.to_string()),
            artist: Some("Artist".to_string()),
            duration_seconds: Some(245),
            now_playing,
            position_ms: None,
        }
    }

    #[test]
    fn empty_queue_disables_transport_controls() {
        let vm = QueueNowPlayingPageVm::builder().build();

        assert!(vm.rows.is_empty());
        assert!(vm.transport.disabled);
        assert!(vm.transport.skip_previous.disabled);
        assert!(vm.transport.skip_next.disabled);
        assert_eq!(vm.empty_label, "Queue is empty");
    }

    #[test]
    fn show_actions_match_each_transport_state() {
        let cases = [
            (TransportState::Empty, false, false, false, false, false),
            (TransportState::Stopped, true, false, false, false, true),
            (TransportState::Starting, false, false, false, false, false),
            (TransportState::Playing, false, true, false, true, false),
            (TransportState::Paused, false, false, true, true, false),
            (TransportState::Failed, true, false, false, true, false),
        ];
        for (state, start, pause, resume, stop, select) in cases {
            let vm = QueueNowPlayingPageVm::builder()
                .transport_state(state)
                .build();
            assert_eq!(
                vm.actions.start.availability == ShowActionAvailability::Available,
                start
            );
            assert_eq!(
                vm.actions.pause.availability == ShowActionAvailability::Available,
                pause
            );
            assert_eq!(
                vm.actions.resume.availability == ShowActionAvailability::Available,
                resume
            );
            assert_eq!(
                vm.actions.stop.availability == ShowActionAvailability::Available,
                stop
            );
            assert_eq!(
                vm.actions.select.availability == ShowActionAvailability::Available,
                select
            );
        }
    }

    #[test]
    fn clear_action_requires_a_stored_cue_and_stopped_playback() {
        let available = |state, has_cue| {
            QueueNowPlayingPageVm::builder()
                .transport_state(state)
                .has_cue(has_cue)
                .build()
                .can_clear_show_cue()
        };
        // The app shows the failed state in place of the stored state.
        let shown_failed = |stored| {
            let mut vm = QueueNowPlayingPageVm::builder()
                .transport_state(stored)
                .has_cue(true)
                .build();
            vm.set_transport_state(TransportState::Failed);
            vm.can_clear_show_cue()
        };
        assert!(available(TransportState::Stopped, true));
        assert!(shown_failed(TransportState::Stopped));
        assert!(!shown_failed(TransportState::Playing));
        assert!(!shown_failed(TransportState::Paused));
        assert!(!available(TransportState::Empty, false));
        assert!(!available(TransportState::Stopped, false));
        assert!(!available(TransportState::Playing, true));
        assert!(!available(TransportState::Paused, true));
        assert!(!available(TransportState::Starting, true));
    }

    #[test]
    fn toggle_gives_no_intent_without_a_cue_and_a_show_intent_with_a_cue() {
        let intent = |state| {
            QueueNowPlayingPageVm::builder()
                .transport_state(state)
                .build()
                .toggle_intent()
        };
        assert_eq!(intent(TransportState::Empty), None);
        assert_eq!(intent(TransportState::Starting), None);
        assert_eq!(
            intent(TransportState::Stopped),
            Some((ShowToggleIntent::Start, "Start Show"))
        );
        assert_eq!(
            intent(TransportState::Failed),
            Some((ShowToggleIntent::Start, "Start Show"))
        );
        assert_eq!(
            intent(TransportState::Playing),
            Some((ShowToggleIntent::Pause, "Pause Show"))
        );
        assert_eq!(
            intent(TransportState::Paused),
            Some((ShowToggleIntent::Resume, "Resume Show"))
        );
    }

    #[test]
    fn cue_row_uses_entry_identity_and_names_a_missing_file() {
        let row = QueueRowDisplay::from_track(QueueTrackInput {
            id: 7,
            entry_id: Some(19),
            unavailable_reason: Some(QueueUnavailableReason::MissingFile),
            title: Some("Cue track".to_owned()),
            ..QueueTrackInput::default()
        });

        assert_eq!(row.id, "queue-entry-19");
        assert_eq!(row.unavailable_label, Some("File missing"));
        assert!(row.a11y_label.contains("Unavailable: File missing"));
    }

    #[test]
    fn single_track_playing_marks_now_playing_row() {
        let vm = QueueNowPlayingPageVm::builder()
            .tracks([track(7, "Opening", true)])
            .transport_state(TransportState::Playing)
            .build();

        assert_eq!(vm.rows.len(), 1);
        assert_eq!(vm.rows[0].id, "queue-row-7");
        assert_eq!(vm.rows[0].title, "Opening");
        assert_eq!(vm.rows[0].duration_label.as_deref(), Some("4:05"));
        assert!(vm.rows[0].now_playing);
        assert_eq!(
            vm.rows[0].a11y_label,
            "Now playing, Opening, by Artist, 4:05"
        );
        assert_eq!(vm.transport.play_pause_state, TransportState::Playing);
        assert_eq!(vm.transport.play_pause_label, "Pause");
        assert!(!vm.transport.disabled);
        assert!(vm.transport.skip_previous.disabled);
        assert!(vm.transport.skip_next.disabled);
    }

    #[test]
    fn multi_track_paused_keeps_queue_enabled() {
        let vm = QueueNowPlayingPageVm::builder()
            .tracks([track(1, "First", true), track(2, "Second", false)])
            .transport_state(TransportState::Paused)
            .skip_availability(false, true)
            .build();

        assert_eq!(vm.rows.len(), 2);
        assert_eq!(vm.transport.play_pause_state, TransportState::Paused);
        assert_eq!(vm.transport.play_pause_a11y_label, "Resume Show");
        assert!(!vm.transport.disabled);
        assert!(vm.transport.skip_previous.disabled);
        assert!(!vm.transport.skip_next.disabled);
        assert!(vm.rows[0].now_playing);
        assert!(!vm.rows[1].now_playing);
    }

    #[test]
    fn text_filter_defaults_to_full_queue() {
        let vm = QueueNowPlayingPageVm::builder()
            .tracks([track(1, "First", true), track(2, "Second", false)])
            .build();

        assert_eq!(vm.text_filter(), None);
        assert_eq!(vm.rows.len(), 2);
    }

    #[test]
    fn text_filter_projects_matching_rows_without_discarding_queue() {
        let mut vm = QueueNowPlayingPageVm::builder()
            .tracks([track(1, "First", true), track(2, "Second", false)])
            .build();

        vm.set_text_filter(Some("second".to_string()));

        assert_eq!(vm.text_filter(), Some("second"));
        assert_eq!(vm.rows.len(), 1);
        assert_eq!(vm.rows[0].title, "Second");

        vm.set_text_filter(None);

        assert_eq!(vm.text_filter(), None);
        assert_eq!(vm.rows.len(), 2);
        assert_eq!(vm.rows[0].title, "First");
        assert_eq!(vm.rows[1].title, "Second");
    }

    #[test]
    fn text_filter_matches_artist_and_is_case_insensitive() {
        let mut vm = QueueNowPlayingPageVm::builder()
            .tracks([
                QueueTrackInput {
                    entry_id: None,
                    artist: Some("Alice".to_string()),
                    ..track(1, "First", true)
                },
                QueueTrackInput {
                    entry_id: None,
                    artist: Some("Bob".to_string()),
                    ..track(2, "Second", false)
                },
            ])
            .build();

        vm.set_text_filter(Some("ALICE".to_string()));

        assert_eq!(vm.text_filter(), Some("ALICE"));
        assert_eq!(vm.rows.len(), 1);
        assert_eq!(vm.rows[0].artist.as_deref(), Some("Alice"));
    }

    #[test]
    fn whitespace_text_filter_clears_visible_projection() {
        let mut vm = QueueNowPlayingPageVm::builder()
            .tracks([track(1, "First", true), track(2, "Second", false)])
            .build();

        vm.set_text_filter(Some("first".to_string()));
        vm.set_text_filter(Some("   ".to_string()));

        assert_eq!(vm.text_filter(), None);
        assert_eq!(vm.rows.len(), 2);
    }
    /// A cue of five entries. `current` is the playing entry.
    fn cue_vm(state: TransportState, current: usize) -> QueueNowPlayingPageVm {
        QueueNowPlayingPageVm::builder()
            .tracks((0..5).map(|index| QueueTrackInput {
                entry_id: Some(100 + i64::try_from(index).unwrap()),
                now_playing: index == current && state != TransportState::Stopped,
                position_ms: Some(65_000),
                ..track(1, "Cue track", false)
            }))
            .transport_state(state)
            .has_cue(true)
            .build()
    }

    // ADR 0106 task 001, criterion 6.
    #[test]
    fn play_next_shows_after_the_next_row_and_on_no_other_row() {
        for state in [TransportState::Playing, TransportState::Paused] {
            let vm = cue_vm(state, 1);
            let shown = vm
                .rows
                .iter()
                .map(QueueRowDisplay::play_next)
                .collect::<Vec<_>>();
            assert_eq!(shown, [false, false, false, true, true]);
        }
        let stopped = cue_vm(TransportState::Stopped, 0);
        assert!(stopped.rows.iter().all(|row| !row.play_next()));
    }

    #[test]
    fn rows_give_played_playing_and_move_availability() {
        let vm = cue_vm(TransportState::Playing, 2);
        let labels = vm
            .rows
            .iter()
            .map(|row| row.state_label.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                Some("Played"),
                Some("Played"),
                Some("Playing, 3:00 left"),
                None,
                None
            ]
        );
        let movable = vm
            .rows
            .iter()
            .map(QueueRowDisplay::can_move)
            .collect::<Vec<_>>();
        assert_eq!(movable, [false, false, false, true, true]);
        assert_eq!(vm.cue_move_hint, Some(CUE_MOVE_HINT));
        assert_eq!(vm.key_move_index(103, true), None);
        assert_eq!(vm.key_move_index(103, false), Some(4));
        assert_eq!(vm.key_move_index(104, true), Some(3));
        assert_eq!(vm.key_move_index(104, false), None);
        assert_eq!(vm.key_move_index(102, false), None);
    }

    #[test]
    fn a_stopped_cue_moves_each_row_and_shows_no_hint() {
        let vm = cue_vm(TransportState::Stopped, 0);
        assert!(vm.rows.iter().all(QueueRowDisplay::can_move));
        assert!(vm.rows.iter().all(|row| row.state_label.is_none()));
        assert_eq!(vm.cue_move_hint, None);
        assert_eq!(vm.key_move_index(100, true), None);
        assert_eq!(vm.key_move_index(100, false), Some(1));
    }
}
