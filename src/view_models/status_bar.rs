//! The status bar display contract (ADR 0101 Decision 7, task 008).
//!
//! One bar at the bottom of the window. Its left side shows the latest event
//! of the app event list. Its right side shows small items: feed state, files
//! to update, reports with a count, and show state. The show item exists only
//! while a show is active. Each item opens its detail. The projection is
//! GPUI-free.

#![warn(clippy::pedantic)]

use crate::runtime::{PlaylistRssCheckSnapshot, RssFeedOutcome, TagUpdateSnapshot};
use icu_locale_core::Locale;

use crate::view_models::app_events::{AppEvent, AppEventList};
use crate::view_models::format::{fmt_date_in, fmt_time_in};
use crate::view_models::live_status::{LiveStatusDisplay, OPEN_SHOW_A11Y_LABEL};
use crate::view_models::tag_update::{self, TagUpdateAvailability};

/// The element id of the bar.
pub(crate) const STATUS_BAR_ID: &str = "status-bar";

/// The kind of a right side item. The order of the kinds is the order of the
/// items in the bar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatusItemKind {
    Feed,
    Files,
    Reports,
    Show,
}

/// The icon of an item. The renderer maps it to an icon of the app.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatusItemIcon {
    Feed,
    Files,
    Reports,
    Show,
}

/// The detail that an item opens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatusItemDetail {
    /// The page of a playlist whose feed check runs or has failed feeds.
    FeedCheck { playlist_id: i64 },
    /// The popup of the files to update (ADR 0076 Decision 8).
    TagUpdatePopup,
    /// The normal-shell notice of the reports (ADR 0066).
    ReportsNotice,
    /// The Show section.
    ShowSection,
}

/// One item of the right side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StatusItemDisplay {
    pub(crate) kind: StatusItemKind,
    pub(crate) id: &'static str,
    pub(crate) icon: StatusItemIcon,
    pub(crate) word: String,
    pub(crate) a11y_label: String,
    pub(crate) detail: StatusItemDetail,
    /// `false` while the detail cannot open.
    pub(crate) enabled: bool,
}

/// The latest event with its recorded time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StatusEventDisplay {
    /// The words of the owner.
    pub(crate) words: String,
    /// The recorded time, in UTC.
    pub(crate) time_label: String,
    /// The words with the full recorded time.
    pub(crate) a11y_label: String,
}

/// The source facts of the bar.
pub(crate) struct StatusBarInput<'a> {
    pub(crate) events: &'a AppEventList,
    /// The show state, from `LiveStatusDisplay::from_show_page`.
    pub(crate) live: &'a LiveStatusDisplay,
    pub(crate) tag_update: Option<&'a TagUpdateSnapshot>,
    pub(crate) feed_checks: Option<&'a PlaylistRssCheckSnapshot>,
    /// The count of `CapabilityReportVm::report_count`.
    pub(crate) report_count: usize,
}

/// Display-ready status bar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StatusBarDisplay {
    pub(crate) id: &'static str,
    pub(crate) event: Option<StatusEventDisplay>,
    pub(crate) items: Vec<StatusItemDisplay>,
}

impl StatusEventDisplay {
    /// The display of `event` in `locale`, at the time zone with the offset
    /// `offset_secs`. The list keeps the time in UTC. Only the display
    /// converts it (orchestrator choice of ADR 0101 task 008).
    #[must_use]
    pub(crate) fn from_event_in(event: &AppEvent, locale: &Locale, offset_secs: i32) -> Self {
        let ts = event.at.timestamp();
        let time = fmt_time_in(ts, locale, offset_secs).unwrap_or_default();
        let date = fmt_date_in(ts, locale, offset_secs).unwrap_or_default();
        Self {
            words: event.words.clone(),
            a11y_label: format!("{} Recorded at {time}, {date}.", event.words),
            time_label: time,
        }
    }
}

impl StatusBarDisplay {
    /// Projects the bar from its sources.
    #[must_use]
    pub(crate) fn project(input: &StatusBarInput<'_>) -> Self {
        let event = input.events.latest().map(|event| {
            let ts = event.at.timestamp();
            StatusEventDisplay::from_event_in(
                event,
                &crate::view_models::name_order::system_locale(),
                crate::view_models::format::local_offset_secs(ts),
            )
        });
        let items = [
            feed_item(input.feed_checks),
            files_item(input.tag_update),
            reports_item(input.report_count),
            show_item(input.live),
        ]
        .into_iter()
        .flatten()
        .collect();
        Self {
            id: STATUS_BAR_ID,
            event,
            items,
        }
    }

    /// The item of `kind`, if the bar shows it.
    #[cfg(test)]
    fn item(&self, kind: StatusItemKind) -> Option<&StatusItemDisplay> {
        self.items.iter().find(|item| item.kind == kind)
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    if count == 1 {
        format!("{count} {one}")
    } else {
        format!("{count} {many}")
    }
}

/// The feed state: a check that runs, or the failed feeds of the latest run.
/// Source: `PlaylistRssCheckSnapshot` of `src/runtime/playlist_rss_check.rs`.
fn feed_item(checks: Option<&PlaylistRssCheckSnapshot>) -> Option<StatusItemDisplay> {
    let checks = checks?;
    if let Some(playlist_id) = checks.running_playlists().next() {
        return Some(StatusItemDisplay {
            kind: StatusItemKind::Feed,
            id: "status-feed",
            icon: StatusItemIcon::Feed,
            word: "Checking feeds".to_owned(),
            a11y_label: "A feed check runs. Open the playlist".to_owned(),
            detail: StatusItemDetail::FeedCheck { playlist_id },
            enabled: true,
        });
    }
    let (playlist_id, run) = checks.runs().max_by_key(|(_, run)| run.started_at_us)?;
    let failed = run.count(RssFeedOutcome::Failed);
    if failed == 0 {
        return None;
    }
    let word = format!("{} failed", plural(failed, "feed", "feeds"));
    Some(StatusItemDisplay {
        kind: StatusItemKind::Feed,
        id: "status-feed",
        icon: StatusItemIcon::Feed,
        a11y_label: format!("{word} in the latest feed check. Open the playlist"),
        word,
        detail: StatusItemDetail::FeedCheck { playlist_id },
        enabled: true,
    })
}

/// The files to update. Source: `view_models::tag_update::button`.
fn files_item(snapshot: Option<&TagUpdateSnapshot>) -> Option<StatusItemDisplay> {
    let button = tag_update::button(snapshot)?;
    Some(StatusItemDisplay {
        kind: StatusItemKind::Files,
        id: "status-files",
        icon: StatusItemIcon::Files,
        word: button.label,
        a11y_label: button.a11y_label,
        detail: StatusItemDetail::TagUpdatePopup,
        enabled: button.availability == TagUpdateAvailability::Available,
    })
}

/// The reports with their count. Source: `CapabilityReportVm::report_count`.
fn reports_item(count: usize) -> Option<StatusItemDisplay> {
    if count == 0 {
        return None;
    }
    let word = plural(count, "report", "reports");
    Some(StatusItemDisplay {
        kind: StatusItemKind::Reports,
        id: "status-reports",
        icon: StatusItemIcon::Reports,
        a11y_label: format!("{word}. Open the notice with the repair actions"),
        word,
        detail: StatusItemDetail::ReportsNotice,
        enabled: true,
    })
}

/// The show state, only while a show is active. Source:
/// `LiveStatusDisplay::from_show_page`.
fn show_item(live: &LiveStatusDisplay) -> Option<StatusItemDisplay> {
    if !live.active {
        return None;
    }
    let word = live
        .now_playing
        .as_ref()
        .map_or("Show active", |now_playing| now_playing.state_label)
        .to_owned();
    Some(StatusItemDisplay {
        kind: StatusItemKind::Show,
        id: "status-show",
        icon: StatusItemIcon::Show,
        a11y_label: format!("Show: {}. {OPEN_SHOW_A11Y_LABEL}", live.summary_label),
        word,
        detail: StatusItemDetail::ShowSection,
        enabled: true,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::view_models::app_events::AppEventOwner;
    use crate::view_models::queue_now_playing::{
        QueueNowPlayingPageVm, QueueTrackInput, TransportState,
    };
    use crate::view_models::show::ShowPageVm;
    use crate::view_models::startup::capabilities::CapabilityReportVm;

    fn at(second: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + second, 0).expect("a valid time")
    }

    fn playing_show() -> ShowPageVm {
        ShowPageVm::from_queue(
            QueueNowPlayingPageVm::builder()
                .tracks([QueueTrackInput {
                    id: 7,
                    entry_id: None,
                    selected: false,
                    selectable: false,
                    unavailable_reason: None,
                    title: Some("Opening".to_string()),
                    artist: Some("Artist".to_string()),
                    duration_seconds: Some(125),
                    now_playing: true,
                }])
                .transport_state(TransportState::Playing)
                .build(),
        )
    }

    fn project(events: &AppEventList, show: &ShowPageVm, report_count: usize) -> StatusBarDisplay {
        StatusBarDisplay::project(&StatusBarInput {
            events,
            live: &LiveStatusDisplay::from_show_page(show),
            tag_update: None,
            feed_checks: None,
            report_count,
        })
    }

    /// M1: the bar gives no show item with no active show, and one with an
    /// active show.
    #[test]
    fn adr_0101_task_008_show_item_exists_only_while_a_show_is_active() {
        let events = AppEventList::default();
        let idle = project(&events, &ShowPageVm::idle(), 0);
        assert!(idle.item(StatusItemKind::Show).is_none());
        let active = project(&events, &playing_show(), 0);
        let item = active.item(StatusItemKind::Show).expect("a show item");
        assert_eq!(item.word, "Playing");
        assert_eq!(item.detail, StatusItemDetail::ShowSection);
    }

    /// M2: the left side gives the latest event with its words and its
    /// recorded time.
    #[test]
    fn adr_0101_task_008_latest_event_gives_words_and_recorded_time() {
        let mut events = AppEventList::default();
        let none = project(&events, &ShowPageVm::idle(), 0);
        assert_eq!(none.event, None);
        events.record(AppEvent {
            owner: AppEventOwner::FeedCheck,
            words: "The request to relay.test failed.".to_owned(),
            at: at(0),
        });
        events.record(AppEvent {
            owner: AppEventOwner::Download,
            words: "The app could not download Song: no space".to_owned(),
            at: at(61),
        });
        let bar = project(&events, &ShowPageVm::idle(), 0);
        let event = bar.event.expect("an event");
        assert_eq!(event.words, "The app could not download Song: no space");
        assert!(!event.time_label.is_empty());
        assert!(!event.time_label.contains("UTC"));
    }

    /// Orchestrator choice of ADR 0101 task 008: the time of the event is
    /// the local time in the time format of the language. The list keeps
    /// the time in UTC. 2027-01-15 08:01:01 UTC is 03:01:01 at UTC-5.
    #[test]
    fn adr_0101_task_008_event_time_is_local_in_the_language_format() {
        let event = AppEvent {
            owner: AppEventOwner::Download,
            words: "The app could not download Song: no space".to_owned(),
            at: at(61),
        };
        let us: Locale = "en-US".parse().expect("a locale");
        let de: Locale = "de-DE".parse().expect("a locale");
        let east_coast = StatusEventDisplay::from_event_in(&event, &us, -5 * 3600);
        assert_eq!(east_coast.time_label, "3:01:01\u{202f}AM");
        assert!(east_coast
            .a11y_label
            .ends_with("Recorded at 3:01:01\u{202f}AM, Jan 15, 2027."));
        let berlin = StatusEventDisplay::from_event_in(&event, &de, 3600);
        assert_eq!(berlin.time_label, "09:01:01");
        assert_eq!(event.at.timestamp(), 1_800_000_061);
    }

    /// M3: the reports item gives the count and opens the normal-shell
    /// notice. It is absent with no report.
    #[test]
    fn adr_0101_task_008_reports_item_gives_count_and_opens_the_notice() {
        let events = AppEventList::default();
        let none = project(&events, &ShowPageVm::idle(), 0);
        assert!(none.item(StatusItemKind::Reports).is_none());
        let reports = CapabilityReportVm::setup_failures_fixture();
        let bar = project(&events, &ShowPageVm::idle(), reports.report_count());
        let item = bar.item(StatusItemKind::Reports).expect("a reports item");
        assert_eq!(item.word, "3 reports");
        assert_eq!(item.detail, StatusItemDetail::ReportsNotice);
        assert!(item.a11y_label.starts_with("3 reports."));
    }
}
