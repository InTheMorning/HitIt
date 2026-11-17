//! The bounded list of app events (ADR 0101 Decision 7, task 008).
//!
//! The status bar shows the latest event of this list. The list holds two
//! kinds of event. The first kind is a show event report (ADRs 0059 and
//! 0063). The second kind is an event with no visible action, as musicindex
//! ADR 0026 R-NOTICE-1 item 2 names it: a feed check, a download or a tag
//! write that fails. Each event keeps the words of its owner and its
//! recorded time. This module makes no new words.

#![warn(clippy::pedantic)]

use std::collections::VecDeque;

use chrono::{DateTime, Utc};

use crate::view_models::show::EventSectionInput;

/// The most events that the list keeps. The oldest event goes first.
pub(crate) const APP_EVENT_LIMIT: usize = 50;

/// The part of the app that owns the words of an event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppEventOwner {
    /// A show event report (ADRs 0059 and 0063).
    ShowEventReport,
    /// A feed check of a playlist that fails (ADR 0076).
    FeedCheck,
    /// A download that fails (ADR 0084 Decision 10).
    Download,
    /// A tag write that fails (ADR 0076 Decision 8).
    TagWrite,
}

/// One event of the list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppEvent {
    pub(crate) owner: AppEventOwner,
    /// The words of the owner.
    pub(crate) words: String,
    /// The time when the owner recorded the event.
    pub(crate) at: DateTime<Utc>,
}

impl AppEvent {
    /// An event that the owner records now.
    #[must_use]
    pub(crate) fn now(owner: AppEventOwner, words: impl Into<String>) -> Self {
        Self {
            owner,
            words: words.into(),
            at: Utc::now(),
        }
    }
}

/// The events of one app session, newest first.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct AppEventList {
    events: VecDeque<AppEvent>,
    /// The time of the newest show report that the list recorded, with the
    /// words of each report at that time. An older or equal report does not
    /// enter the list again after the list drops it.
    newest_show_report: Option<(DateTime<Utc>, Vec<String>)>,
}

impl AppEventList {
    /// Puts `event` first. A list of [`APP_EVENT_LIMIT`] events drops its
    /// oldest event.
    pub(crate) fn record(&mut self, event: AppEvent) {
        self.events.push_front(event);
        self.events.truncate(APP_EVENT_LIMIT);
    }

    /// Records each report of `input` that the list does not hold yet, the
    /// oldest report first. Returns the number of new events.
    pub(crate) fn record_show_reports(&mut self, input: &EventSectionInput) -> usize {
        let mut recorded = 0;
        for (at, words) in input.report_events() {
            let event = AppEvent {
                owner: AppEventOwner::ShowEventReport,
                words,
                at,
            };
            let seen = self
                .newest_show_report
                .as_ref()
                .is_some_and(|(newest, seen)| {
                    at < *newest || (at == *newest && seen.contains(&event.words))
                });
            if seen || self.events.contains(&event) {
                continue;
            }
            match &mut self.newest_show_report {
                Some((newest, seen)) if *newest == at => seen.push(event.words.clone()),
                _ => self.newest_show_report = Some((at, vec![event.words.clone()])),
            }
            self.record(event);
            recorded += 1;
        }
        recorded
    }

    /// The newest event.
    #[must_use]
    pub(crate) fn latest(&self) -> Option<&AppEvent> {
        self.events.front()
    }

    /// The events, newest first.
    #[cfg(test)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &AppEvent> {
        self.events.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::show::{
        EventCommandFeedback, EventCommandState, EventRegistryInput, EventReportOperation,
        EventSectionInput, EventTargetListInput,
    };

    fn at(second: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + second, 0).expect("a valid time")
    }

    fn input() -> EventSectionInput {
        EventSectionInput {
            liveness_confirmed: true,
            registry: EventRegistryInput::default(),
            context: "Fixture publisher".to_owned(),
            request: 0,
            feedback: EventCommandFeedback::default(),
            selected_event: None,
            targets: EventTargetListInput::Unknown,
            attach_target_name: "default".to_owned(),
            remote_host: false,
        }
    }

    /// M2a: the list keeps at most 50 events, newest first.
    #[test]
    fn adr_0101_task_008_list_keeps_fifty_events_newest_first() {
        let mut list = AppEventList::default();
        for second in 0..60 {
            list.record(AppEvent {
                owner: AppEventOwner::Download,
                words: format!("event {second}"),
                at: at(second),
            });
        }
        assert_eq!(list.iter().count(), APP_EVENT_LIMIT);
        assert_eq!(
            list.latest().map(|event| event.words.as_str()),
            Some("event 59")
        );
        let words: Vec<_> = list.iter().map(|event| event.words.clone()).collect();
        assert_eq!(words.first().map(String::as_str), Some("event 59"));
        assert_eq!(words.last().map(String::as_str), Some("event 10"));
    }

    /// M2a: a show report and a failed feed check both go into the list.
    #[test]
    fn adr_0101_task_008_show_report_and_failed_feed_check_enter_the_list() {
        let mut list = AppEventList::default();
        let mut section = input();
        section.feedback.selection = EventCommandState::Succeeded;
        section.record_event_report(EventReportOperation::Selection, at(1));
        assert_eq!(list.record_show_reports(&section), 1);
        // The same report does not enter the list a second time.
        assert_eq!(list.record_show_reports(&section), 0);
        list.record(AppEvent {
            owner: AppEventOwner::FeedCheck,
            words: "The request to relay.test failed.".to_owned(),
            at: at(2),
        });
        let owners: Vec<_> = list.iter().map(|event| event.owner).collect();
        assert_eq!(
            owners,
            vec![AppEventOwner::FeedCheck, AppEventOwner::ShowEventReport]
        );
    }

    /// A show report that the list dropped does not enter the list again
    /// (review finding of 2026-11-16).
    #[test]
    fn adr_0101_task_008_a_dropped_show_report_does_not_enter_again() {
        let mut list = AppEventList::default();
        let mut section = input();
        section.feedback.selection = EventCommandState::Succeeded;
        section.record_event_report(EventReportOperation::Selection, at(1));
        assert_eq!(list.record_show_reports(&section), 1);
        for second in 0..60 {
            list.record(AppEvent {
                owner: AppEventOwner::Download,
                words: format!("event {second}"),
                at: at(100 + second),
            });
        }
        assert!(list
            .iter()
            .all(|event| event.owner != AppEventOwner::ShowEventReport));
        assert_eq!(list.record_show_reports(&section), 0);
        assert_eq!(
            list.latest().map(|event| event.words.as_str()),
            Some("event 59")
        );
    }

    /// M2: the latest event gives the words of its owner and its recorded
    /// time.
    #[test]
    fn adr_0101_task_008_latest_event_keeps_owner_words_and_time() {
        let mut list = AppEventList::default();
        let mut section = input();
        section.feedback.selection = EventCommandState::Succeeded;
        section.record_event_report(EventReportOperation::Selection, at(5));
        list.record_show_reports(&section);
        let latest = list.latest().expect("an event");
        assert_eq!(latest.at, at(5));
        assert_eq!(
            latest.words,
            "App saved event the selected event as your selected event."
        );
    }
}
