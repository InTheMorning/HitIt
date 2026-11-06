//! Search-results tab and origin models.

#![warn(clippy::pedantic)]

/// Stable numeric identity for one search-result row.
pub(crate) type SearchResultItemId = u64;

/// One tab of the search results (ADR 0089 Decision 4). `All` shows each
/// group in the order of [`Self::GROUPS`], with a heading for each group.
/// The other tabs select one group of the run. A tab does not change the
/// query. The tabs come in the order of [`Self::ALL`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SearchResultsTab {
    /// Each group of the run, with a heading for each group that has rows.
    #[default]
    All,
    /// Release result rows. ADR 0084 Decision 5: the curator sees
    /// "Release" in place of "Feed".
    Releases,
    /// Track result rows.
    Tracks,
    /// Artist result rows: the feed rows of artists and the name rows. A
    /// feed row with a role other than artist is a label row (see
    /// [`Self::Labels`]).
    Artists,
    /// Feed rows whose agreed roles hold a role other than artist (ADR 0089
    /// Decision 4, musicindex ADR 0033 R-WORD-3).
    Labels,
}

impl SearchResultsTab {
    /// Each tab, in the visible order.
    pub(crate) const ALL: [Self; 5] = [
        Self::All,
        Self::Releases,
        Self::Tracks,
        Self::Artists,
        Self::Labels,
    ];

    /// Each group that one tab selects, in the order that the tab All shows
    /// them.
    pub(crate) const GROUPS: [Self; 4] =
        [Self::Releases, Self::Tracks, Self::Artists, Self::Labels];

    /// Returns the visible label for this tab.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Releases => "Releases",
            Self::Tracks => "Tracks",
            Self::Artists => "Artists",
            Self::Labels => "Labels",
        }
    }

    /// The noun of the empty state of this tab, for example "releases".
    #[must_use]
    pub(crate) const fn plural_noun(self) -> &'static str {
        match self {
            Self::All => "results",
            Self::Releases => "releases",
            Self::Tracks => "tracks",
            Self::Artists => "artists",
            Self::Labels => "labels",
        }
    }
}

/// The source that found a search row. The row of a Library item and the
/// row of an Index item of one key merge into one row (packet 010).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchResultOrigin {
    /// Row came from the local library.
    Library,
    /// Row came from the remote index.
    Index,
}

/// The count of one tab, from the rows of one run (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SearchTabCount {
    /// The tab that the count belongs to.
    pub(crate) tab: SearchResultsTab,
    /// The rows that the tab holds now.
    pub(crate) count: usize,
    /// The rows are not all the rows of the query, so the count is a floor.
    pub(crate) has_more: bool,
}

impl SearchTabCount {
    /// The count text, for example "12", or "20+" when `has_more` is true.
    #[must_use]
    pub(crate) fn text(&self) -> String {
        if self.has_more {
            format!("{}+", self.count)
        } else {
            self.count.to_string()
        }
    }

    /// The label of the tab with its count, for example "Releases 20+".
    #[must_use]
    pub(crate) fn segment_label(&self) -> String {
        format!("{} {}", self.tab.label(), self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_0089_task_008_count_text_marks_a_partial_count() {
        let full = SearchTabCount {
            tab: SearchResultsTab::Tracks,
            count: 20,
            has_more: false,
        };
        let partial = SearchTabCount {
            has_more: true,
            ..full
        };
        assert_eq!(full.text(), "20");
        assert_eq!(partial.text(), "20+");
        assert_eq!(partial.segment_label(), "Tracks 20+");
    }
}
