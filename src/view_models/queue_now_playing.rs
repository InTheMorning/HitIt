//! Queue and now-playing frame display contracts.
//!
//! ADR 0046 Phase 4 moves detailed playback controls out of the global toolbar
//! and into the Queue workspace frame. ADR 0060 moves that queue into Show and
//! removes output controls that have no command path. This module keeps those
//! display contracts GPUI-free so the shell can bind them to primitives without
//! owning playback state.

#![warn(clippy::pedantic)]

use crate::view_models::format::fmt_total_runtime_clock;
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
}

impl ShowActionsDisplay {
    fn for_state(state: TransportState) -> Self {
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
}

impl QueueRowDisplay {
    /// Projects a queue-track input into display-ready row data.
    #[must_use]
    pub(crate) fn from_track(input: QueueTrackInput) -> Self {
        let title = input
            .title
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Untitled track".to_string());
        let duration_label = input.duration_seconds.and_then(fmt_total_runtime_clock);
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
        }
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
        self.actions = ShowActionsDisplay::for_state(state);
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

    pub(crate) fn can_skip_previous(&self) -> bool {
        self.can_skip_previous
    }

    pub(crate) fn can_skip_next(&self) -> bool {
        self.can_skip_next
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
}

impl Default for QueueNowPlayingPageVmBuilder {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            transport_state: TransportState::Empty,
            can_skip_previous: false,
            can_skip_next: false,
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

    /// Projects the builder input into a display-ready page.
    #[must_use]
    pub(crate) fn build(self) -> QueueNowPlayingPageVm {
        let rows = self
            .tracks
            .into_iter()
            .map(QueueRowDisplay::from_track)
            .collect::<Vec<_>>();
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
            actions: ShowActionsDisplay::for_state(self.transport_state),
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
}
