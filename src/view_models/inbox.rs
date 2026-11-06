//! The Inbox page view model (ADR 0089 Decision 3).
//!
//! The Inbox holds what needs a decision before a show. It has three
//! sections, in this sequence: Tracks, Updates and Setup. A section with no
//! item is absent. The Tracks rows are projected by `library.rs`, because
//! they keep their selection and row actions. This module owns the section
//! order, the counts, the Updates and Setup items, and the order of the
//! problem tracks. It carries no renderer type.

#![warn(clippy::pedantic)]

use crate::application::queries::broadcast::BroadcastReadinessTrack;
use crate::view_models::startup::capabilities::CapabilityActionDisplay;
use crate::view_models::tag_update::files_text;

/// One section of the Inbox (ADR 0089 Decision 3). The order of the
/// variants is the order of the sections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InboxSection {
    Tracks,
    Updates,
    Setup,
}

impl InboxSection {
    /// The visible section title.
    #[must_use]
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Tracks => "Tracks",
            Self::Updates => "Updates",
            Self::Setup => "Setup",
        }
    }
}

/// A section that holds at least one item, with its item count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InboxSectionDisplay {
    pub(crate) section: InboxSection,
    pub(crate) title: &'static str,
    pub(crate) count: usize,
}

/// The kind of an Updates item. The kind names the existing action that the
/// item opens (ADR 0089 Decision 3, Updates).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InboxUpdateKind {
    /// Feeds with changes. The item opens "Apply updates".
    FeedUpdates,
    /// Files whose tags differ from the stored metadata. The item opens the
    /// "Update n file(s)" popup (ADR 0076 Decision 8).
    TagUpdates,
}

/// One item of the Updates section. Its count is the number of feeds or
/// files that it names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InboxUpdateItem {
    pub(crate) kind: InboxUpdateKind,
    /// The visible sentence of the item.
    pub(crate) text: String,
    /// The number of feeds or files of the item.
    pub(crate) count: usize,
    /// The label of the item's action button.
    pub(crate) button_label: String,
    /// The accessibility label of the action button.
    pub(crate) a11y_label: String,
    /// `true` while the source of the item runs its own action, so the
    /// button renders unavailable (ADR 0089 Decision 3).
    pub(crate) disabled: bool,
}

/// One item of the Setup section: a capability issue that a show needs. Its
/// action is the existing repair action of that issue (ADR 0066).
#[derive(Clone, Debug)]
pub(crate) struct InboxSetupItem {
    /// The visible sentence of the item.
    pub(crate) text: String,
    /// The repair action, with its label and typed availability.
    pub(crate) action: CapabilityActionDisplay,
}

/// The Inbox page view model (ADR 0089 Decision 3).
#[derive(Clone, Debug, Default)]
pub(crate) struct InboxPageVm {
    track_count: usize,
    updates: Vec<InboxUpdateItem>,
    setup: Vec<InboxSetupItem>,
}

impl InboxPageVm {
    /// Gives the page from its three parts.
    #[must_use]
    pub(crate) fn new(
        track_count: usize,
        updates: Vec<InboxUpdateItem>,
        setup: Vec<InboxSetupItem>,
    ) -> Self {
        Self {
            track_count,
            updates,
            setup,
        }
    }

    /// The sections that hold an item, in sequence. A section with no item
    /// is absent.
    #[must_use]
    pub(crate) fn sections(&self) -> Vec<InboxSectionDisplay> {
        [
            (InboxSection::Tracks, self.track_count),
            (InboxSection::Updates, self.update_count()),
            (InboxSection::Setup, self.setup.len()),
        ]
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(section, count)| InboxSectionDisplay {
            section,
            title: section.title(),
            count,
        })
        .collect()
    }

    /// The number of items that need a decision. Each track is one item, each
    /// setup issue is one item, and each update item counts its feeds or files.
    /// The sidebar count is this number (ADR 0089 Decision 3).
    #[must_use]
    pub(crate) fn item_count(&self) -> usize {
        self.track_count + self.update_count() + self.setup.len()
    }

    /// The Updates items.
    #[must_use]
    pub(crate) fn updates(&self) -> &[InboxUpdateItem] {
        &self.updates
    }

    /// The Setup items.
    #[must_use]
    pub(crate) fn setup(&self) -> &[InboxSetupItem] {
        &self.setup
    }

    fn update_count(&self) -> usize {
        self.updates.iter().map(|item| item.count).sum()
    }
}

/// The Updates items of the feed updates and the tag updates. A source with
/// no change gives no item. `feed_busy` is `true` while a feed check or an
/// update runs. `tag_writing` is `true` while a tag confirm writes files.
#[must_use]
pub(crate) fn update_items(
    stale_feed_count: usize,
    feed_busy: bool,
    tag_file_count: usize,
    tag_writing: bool,
) -> Vec<InboxUpdateItem> {
    let mut items = Vec::new();
    if stale_feed_count > 0 {
        let feeds = if stale_feed_count == 1 {
            "1 feed has".to_owned()
        } else {
            format!("{stale_feed_count} feeds have")
        };
        items.push(InboxUpdateItem {
            kind: InboxUpdateKind::FeedUpdates,
            text: format!("{feeds} updates from MusicIndex."),
            count: stale_feed_count,
            button_label: "Apply updates".to_owned(),
            a11y_label: format!("Apply updates to {feeds} changes"),
            disabled: feed_busy,
        });
    }
    if tag_file_count > 0 {
        let files = files_text(tag_file_count);
        items.push(InboxUpdateItem {
            kind: InboxUpdateKind::TagUpdates,
            text: format!("{files} have tags that differ from the stored metadata."),
            count: tag_file_count,
            button_label: format!("Update {files}"),
            a11y_label: format!("Show the {files} whose tags differ from the stored metadata"),
            disabled: tag_writing,
        });
    }
    items
}

/// Orders the problem tracks of the Inbox (ADR 0089 Decision 3, Answer 3).
/// The tracks of the show playlist come first, in the playlist sequence
/// `source_track_ids`. The other tracks follow in their present sequence.
/// With no session, `source_track_ids` is empty and the present sequence holds.
#[must_use]
pub(crate) fn order_problem_tracks<'a>(
    tracks: Vec<&'a BroadcastReadinessTrack>,
    source_track_ids: &[i64],
) -> Vec<&'a BroadcastReadinessTrack> {
    let mut first = Vec::new();
    for id in source_track_ids {
        if let Some(track) = tracks.iter().find(|track| track.track_id == *id) {
            if !first
                .iter()
                .any(|known: &&BroadcastReadinessTrack| known.track_id == *id)
            {
                first.push(*track);
            }
        }
    }
    let rest = tracks
        .into_iter()
        .filter(|track| !first.iter().any(|known| known.track_id == track.track_id));
    first.iter().copied().chain(rest).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::queries::broadcast::BroadcastReadinessState;

    fn track(track_id: i64, title: &str) -> BroadcastReadinessTrack {
        BroadcastReadinessTrack {
            track_id,
            title: title.to_owned(),
            artist: None,
            album: None,
            path: None,
            state: BroadcastReadinessState::NoRouteTag,
            reason: String::new(),
        }
    }

    fn ids(tracks: &[&BroadcastReadinessTrack]) -> Vec<i64> {
        tracks.iter().map(|track| track.track_id).collect()
    }

    /// ADR 0089 Decision 3, Answer 3: the tracks of the session playlist come
    /// first, in the playlist sequence.
    #[test]
    fn adr_0089_inbox_tracks_of_the_source_playlist_come_first_in_its_sequence() {
        let problems = [track(1, "a"), track(2, "b"), track(3, "c"), track(4, "d")];
        let ordered = order_problem_tracks(problems.iter().collect(), &[4, 2]);
        assert_eq!(ids(&ordered), [4, 2, 1, 3]);
    }

    /// ADR 0089 Decision 3, Answer 3: with no session, the present sequence
    /// holds, and no track has the priority.
    #[test]
    fn adr_0089_inbox_with_no_session_keeps_the_present_sequence() {
        let problems = [track(1, "a"), track(2, "b"), track(3, "c")];
        let ordered = order_problem_tracks(problems.iter().collect(), &[]);
        assert_eq!(ids(&ordered), [1, 2, 3]);
    }

    #[test]
    fn adr_0089_inbox_ignores_source_ids_that_are_not_problem_tracks() {
        let problems = [track(1, "a"), track(2, "b")];
        let ordered = order_problem_tracks(problems.iter().collect(), &[9, 2]);
        assert_eq!(ids(&ordered), [2, 1]);
    }

    /// ADR 0089 Decision 3: a section with no item is absent, and an empty
    /// Inbox has no section.
    #[test]
    fn adr_0089_inbox_sections_are_absent_when_empty() {
        let empty = InboxPageVm::default();
        assert!(empty.sections().is_empty());
        assert_eq!(empty.item_count(), 0);

        let tracks_only = InboxPageVm::new(2, Vec::new(), Vec::new());
        let sections = tracks_only.sections();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].section, InboxSection::Tracks);
        assert_eq!(sections[0].count, 2);
    }

    /// ADR 0089 Decision 3: the sections keep their sequence, Tracks, Updates
    /// and Setup, and the count is the sum of the items.
    #[test]
    fn adr_0089_inbox_sections_keep_sequence_and_sum_the_items() {
        let updates = update_items(2, false, 3, false);
        let page = InboxPageVm::new(1, updates, Vec::new());
        let sections: Vec<_> = page.sections().iter().map(|s| s.section).collect();
        assert_eq!(sections, [InboxSection::Tracks, InboxSection::Updates]);
        assert_eq!(page.item_count(), 1 + 2 + 3);
    }

    #[test]
    fn adr_0089_inbox_update_items_name_the_existing_actions() {
        let items = update_items(1, false, 1, false);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].kind, InboxUpdateKind::FeedUpdates);
        assert_eq!(items[0].button_label, "Apply updates");
        assert_eq!(items[0].count, 1);
        assert_eq!(items[1].kind, InboxUpdateKind::TagUpdates);
        assert_eq!(items[1].button_label, "Update 1 file");
        assert!(update_items(0, false, 0, false).is_empty());
        assert!(update_items(1, true, 0, false)[0].disabled);
    }
}
