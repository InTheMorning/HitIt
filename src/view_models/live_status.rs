//! Show state for the status bar.
//!
//! ADR 0101 Decision 7 shows the show state as one item of the status bar,
//! and only while a show is active. The projection is GPUI-free and derives
//! from the same `ShowPageVm` snapshot that renders the Show screen.

#![warn(clippy::pedantic)]

use crate::view_models::show::{ShowNowPlayingDisplay, ShowPageVm};

/// Listener-facing now-playing facts of the show item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LiveStatusNowPlayingDisplay {
    pub(crate) line: String,
    pub(crate) state_label: &'static str,
    pub(crate) a11y_label: String,
}

impl LiveStatusNowPlayingDisplay {
    fn from_show(now_playing: &ShowNowPlayingDisplay, state_label: &'static str) -> Self {
        let line = now_playing.artist.as_ref().map_or_else(
            || now_playing.title.clone(),
            |artist| format!("{artist} - {}", now_playing.title),
        );
        Self {
            line,
            state_label,
            a11y_label: now_playing.a11y_label.clone(),
        }
    }
}

/// The accessibility label of the command that opens the Show section.
pub(crate) const OPEN_SHOW_A11Y_LABEL: &str = "Open Show";

/// Display-ready show state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LiveStatusDisplay {
    pub(crate) active: bool,
    pub(crate) summary_label: String,
    pub(crate) now_playing: Option<LiveStatusNowPlayingDisplay>,
}

impl LiveStatusDisplay {
    /// Projects the show state from the same Show page snapshot that Show
    /// uses.
    #[must_use]
    pub(crate) fn from_show_page(show: &ShowPageVm) -> Self {
        let active = show.is_active();
        let now_playing = show.now_playing.as_ref().map(|now_playing| {
            LiveStatusNowPlayingDisplay::from_show(now_playing, show.state_label)
        });
        let summary_label = now_playing.as_ref().map_or_else(
            || {
                if active {
                    "Show active".to_string()
                } else {
                    "No active show".to_string()
                }
            },
            |now_playing| now_playing.line.clone(),
        );
        Self {
            active,
            summary_label,
            now_playing,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::view_models::queue_now_playing::{
        QueueNowPlayingPageVm, QueueTrackInput, TransportState,
    };

    use super::*;

    fn track(id: i64, title: &str, now_playing: bool) -> QueueTrackInput {
        QueueTrackInput {
            id,
            entry_id: None,
            selected: false,
            selectable: false,
            unavailable_reason: None,
            title: Some(title.to_string()),
            artist: Some("Artist".to_string()),
            duration_seconds: Some(125),
            now_playing,
        }
    }

    #[test]
    fn live_status_no_show_is_absent() {
        let show = ShowPageVm::idle();
        let display = LiveStatusDisplay::from_show_page(&show);

        assert!(!display.active);
        assert_eq!(display.summary_label, "No active show");
        assert!(display.now_playing.is_none());
    }

    #[test]
    fn live_status_playing_show_projects_now_playing_line() {
        let show = ShowPageVm::from_queue(
            QueueNowPlayingPageVm::builder()
                .tracks([track(7, "Opening", true), track(8, "Next", false)])
                .transport_state(TransportState::Playing)
                .build(),
        );
        let display = LiveStatusDisplay::from_show_page(&show);

        assert!(display.active);
        assert_eq!(display.summary_label, "Artist - Opening");
        assert_eq!(
            display.now_playing,
            Some(LiveStatusNowPlayingDisplay {
                line: "Artist - Opening".to_string(),
                state_label: "Playing",
                a11y_label: "Now playing, Opening, by Artist, 2:05".to_string(),
            })
        );
    }
}
