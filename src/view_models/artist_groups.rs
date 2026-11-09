//! The groups of an artist list (musicindex ADR 0057 R-ARTIST-1).
//!
//! Each artist list shows its rows in three groups, in this sequence: the
//! stated artists, the inferred artists, and the artist names with no feed.
//! A list never mixes two groups in one sequence. The sort and the letter of
//! the list apply in each group, and a group with no row is absent.
//!
//! This module gives the group of a row, the pages that the scope "All"
//! loads (one page for each group), and the sections of the Artists list.
//! It has no renderer type.

#![warn(clippy::pedantic)]

use crate::api::ArtistNameRow;
use crate::view_models::artist_identity::NameFeedMatches;
use crate::view_models::artists_everything::{
    classify_index_artist, group_rows, sort_by_newest, IndexArtistClass, LibraryArtistRow,
};
use crate::view_models::index_name_list::{
    IndexListActionDisplay, IndexListStatusRowVm, IndexNameList, IndexNameListLoadIntent,
    IndexNameListPage,
};
use crate::view_models::letter_bar::name_at_or_after;
use crate::view_models::library::DestinationListRowVm;
use crate::view_models::list_page::ListScope;

/// The count of rows that a group of the scope "All" shows at first, and
/// the count that each "Show more" adds (musicindex ADR 0057 R-SITE-15 item
/// 2). The stated artists have no limit.
pub(crate) const SECTION_SIZE: usize = 20;

/// The prefix that the status rows of the Artists list share.
const STATUS_ID_PREFIX: &str = "music-index-artists-";

/// A group of an artist list, in the sequence of musicindex ADR 0057
/// R-ARTIST-1 item 1. The derived order is that sequence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ArtistGroup {
    /// A feed that both sides of a link confirm with the role `artist`.
    Stated,
    /// A feed with confirmed links and no stated role.
    Inferred,
    /// A name that only release credits give.
    Names,
}

impl ArtistGroup {
    /// The three groups, in the sequence of the list.
    pub(crate) const ALL: [Self; 3] = [Self::Stated, Self::Inferred, Self::Names];

    /// The heading words of the group, as the site shows them.
    #[must_use]
    pub(crate) const fn heading(self) -> &'static str {
        match self {
            Self::Stated => "Stated artists",
            Self::Inferred => "Inferred artists",
            Self::Names => "Artist names with no feed",
        }
    }

    /// The line under the heading. The names group says that no artist
    /// feed confirms its names (R-ARTIST-1 item 5).
    #[must_use]
    pub(crate) const fn intro(self) -> &'static str {
        match self {
            Self::Stated => "These feeds state rel=\"artist\".",
            Self::Inferred => {
                "No feed states rel=\"artist\". The releases link back and credit this name."
            }
            Self::Names => {
                "These names come only from release credits. No artist feed confirms them."
            }
        }
    }

    /// The word of the group in an element id.
    #[must_use]
    pub(crate) const fn slug(self) -> &'static str {
        match self {
            Self::Stated => "stated",
            Self::Inferred => "inferred",
            Self::Names => "names",
        }
    }

    /// Whether the scope "All" shows a limited count of rows and a "Show
    /// more" button for the group.
    #[must_use]
    pub(crate) const fn is_capped(self) -> bool {
        !matches!(self, Self::Stated)
    }

    /// The group of a class of a node row. A class that the lists do not
    /// show gives `None`.
    #[must_use]
    pub(crate) const fn of_class(class: IndexArtistClass) -> Option<Self> {
        match class {
            IndexArtistClass::Artist => Some(Self::Stated),
            IndexArtistClass::UnverifiedArtist => Some(Self::Inferred),
            IndexArtistClass::Name => Some(Self::Names),
            IndexArtistClass::OtherRole
            | IndexArtistClass::NotNamedFirst
            | IndexArtistClass::UnknownKind => None,
        }
    }

    /// The group of a row of `GET /v1/artist-names`.
    #[must_use]
    pub(crate) fn of_index_row(row: &ArtistNameRow) -> Option<Self> {
        Self::of_class(classify_index_artist(row))
    }

    fn element_id(self) -> String {
        format!("music-artists-group-{}", self.slug())
    }
}

/// The heading of one group. It names the group and gives its count when the
/// list knows the whole group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtistGroupHeadingVm {
    /// Stable element id.
    pub(crate) element_id: String,
    /// The group.
    pub(crate) group: ArtistGroup,
    /// The heading words, with the count when it is known, for example
    /// "Inferred artists · 1,773".
    pub(crate) title: String,
    /// The line under the heading.
    pub(crate) intro: &'static str,
}

impl ArtistGroupHeadingVm {
    fn new(group: ArtistGroup, count: Option<usize>) -> Self {
        let title = match count {
            Some(count) => format!("{} \u{b7} {}", group.heading(), with_thousands(count)),
            None => group.heading().to_owned(),
        };
        Self {
            element_id: group.element_id(),
            group,
            title,
            intro: group.intro(),
        }
    }
}

/// `count` with a comma between each three digits.
fn with_thousands(count: usize) -> String {
    let digits = count.to_string();
    let mut text = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

/// One group of the Artists list: its heading, its rows, and its own status
/// row (the loading row, the error row, "Load more" or "Show more").
#[derive(Clone, Debug)]
pub(crate) struct ArtistSection {
    /// The group.
    pub(crate) group: ArtistGroup,
    /// The heading. `None` for a section that has no row and only gives a
    /// status row.
    pub(crate) heading: Option<ArtistGroupHeadingVm>,
    /// The rows of the group, in the sort of the list.
    pub(crate) rows: Vec<DestinationListRowVm>,
    /// The status row of the group.
    pub(crate) status: Option<IndexListStatusRowVm>,
}

/// The pages that the scope "All" of the Artists list loads: one page for
/// each group. Each page holds only the rows of its group (the request of
/// the group keeps them), and each page has its own cursor.
#[derive(Clone, Debug)]
pub(crate) struct ArtistGroupPages {
    stated: IndexNameListPage<ArtistNameRow>,
    inferred: IndexNameListPage<ArtistNameRow>,
    names: IndexNameListPage<ArtistNameRow>,
    inferred_shown: usize,
    names_shown: usize,
}

impl Default for ArtistGroupPages {
    fn default() -> Self {
        Self {
            stated: IndexNameListPage::default(),
            inferred: IndexNameListPage::default(),
            names: IndexNameListPage::default(),
            inferred_shown: SECTION_SIZE,
            names_shown: SECTION_SIZE,
        }
    }
}

/// The request that one group of the Artists list sends.
pub(crate) type ArtistGroupIntent = (ArtistGroup, Option<IndexNameListLoadIntent>);

impl ArtistGroupPages {
    /// The page of `group`.
    #[must_use]
    pub(crate) const fn page(&self, group: ArtistGroup) -> &IndexNameListPage<ArtistNameRow> {
        match group {
            ArtistGroup::Stated => &self.stated,
            ArtistGroup::Inferred => &self.inferred,
            ArtistGroup::Names => &self.names,
        }
    }

    /// The page of `group`, to change.
    pub(crate) fn page_mut(&mut self, group: ArtistGroup) -> &mut IndexNameListPage<ArtistNameRow> {
        match group {
            ArtistGroup::Stated => &mut self.stated,
            ArtistGroup::Inferred => &mut self.inferred,
            ArtistGroup::Names => &mut self.names,
        }
    }

    /// The count of rows that the group shows now. The stated artists show
    /// each row.
    #[must_use]
    pub(crate) const fn shown(&self, group: ArtistGroup) -> usize {
        match group {
            ArtistGroup::Stated => usize::MAX,
            ArtistGroup::Inferred => self.inferred_shown,
            ArtistGroup::Names => self.names_shown,
        }
    }

    /// Shows the next rows of a limited group ("Show more").
    pub(crate) fn show_more(&mut self, group: ArtistGroup) {
        match group {
            ArtistGroup::Stated => {}
            ArtistGroup::Inferred => self.inferred_shown += SECTION_SIZE,
            ArtistGroup::Names => self.names_shown += SECTION_SIZE,
        }
    }

    /// Whether a limited group has fewer loaded rows than it shows and the
    /// node has more pages. The next page then loads at once.
    #[must_use]
    pub(crate) fn needs_rows(&self, group: ArtistGroup) -> bool {
        let page = self.page(group);
        group.is_capped() && page.has_more() && page.rows().len() < self.shown(group)
    }

    /// Whether no request was sent. The three pages start together.
    #[must_use]
    pub(crate) const fn is_not_started(&self) -> bool {
        self.stated.is_not_started()
    }

    /// The `from` text of the lists, or `None` for the start of the lists.
    #[must_use]
    pub(crate) fn from(&self) -> Option<&str> {
        self.stated.from()
    }

    /// Starts the three pages again at `from`. The limited groups show
    /// their first rows again. The result is the request of page 1 of each
    /// group.
    #[must_use]
    pub(crate) fn begin_from(&mut self, from: Option<&str>) -> Vec<ArtistGroupIntent> {
        self.inferred_shown = SECTION_SIZE;
        self.names_shown = SECTION_SIZE;
        ArtistGroup::ALL
            .into_iter()
            .map(|group| {
                let intent = self.page_mut(group).begin_from(from.map(str::to_owned));
                (group, intent)
            })
            .collect()
    }

    /// Begins the load of page 1 of each group, at the `from` text now.
    #[must_use]
    pub(crate) fn begin_first_pages(&mut self) -> Vec<ArtistGroupIntent> {
        let from = self.from().map(str::to_owned);
        self.begin_from(from.as_deref())
    }

    /// Begins the retry of each page whose last request failed.
    #[must_use]
    pub(crate) fn begin_retries(&mut self) -> Vec<ArtistGroupIntent> {
        ArtistGroup::ALL
            .into_iter()
            .map(|group| (group, self.page_mut(group).begin_retry()))
            .collect()
    }
}

#[cfg(test)]
impl ArtistGroupPages {
    /// The pages after the answer of page 1 of each group: each group holds
    /// the rows of `rows` that belong to it. `has_more` tells whether the node
    /// has more pages.
    pub(crate) fn loaded(rows: &[ArtistNameRow], has_more: bool) -> Self {
        use crate::view_models::index_name_list::IndexNameListBatch;

        let mut pages = Self::default();
        for (group, intent) in pages.begin_first_pages() {
            let intent = intent.expect("page 1 starts");
            let own = rows
                .iter()
                .filter(|row| ArtistGroup::of_index_row(row) == Some(group))
                .cloned()
                .collect();
            pages.page_mut(group).finish_load(
                IndexNameListBatch {
                    rows: own,
                    cursor: has_more.then(|| "next".to_owned()),
                    has_more,
                },
                intent.generation,
            );
        }
        pages
    }
}

/// The input of [`artist_sections`].
pub(crate) struct ArtistSectionsInput<'a> {
    /// The scope of the list.
    pub(crate) scope: ListScope,
    /// Whether the sort is Newest.
    pub(crate) newest: bool,
    /// Whether the local rows come in the reversed order.
    pub(crate) local_reversed: bool,
    /// Whether the node lists its rows in the reversed order.
    pub(crate) index_reversed: bool,
    /// The Library rows of every group.
    pub(crate) library: Vec<LibraryArtistRow>,
    /// The pages of the scope "All".
    pub(crate) pages: &'a ArtistGroupPages,
    /// The match of each name to the Library artist feeds.
    pub(crate) matches: &'a NameFeedMatches,
}

/// The sections of the Artists list, in the sequence of musicindex ADR 0057
/// R-ARTIST-1. The sort and the letter apply in each group, and each group
/// has its own rows, count and status row. A group with no row has no
/// heading. A loading row or an error row of page 1 comes once, at the start
/// of the list.
#[must_use]
pub(crate) fn artist_sections(input: &ArtistSectionsInput<'_>) -> Vec<ArtistSection> {
    let mut sections = Vec::new();
    let mut leading: Option<ArtistSection> = None;
    let mut first_page_status_seen = false;
    for group in ArtistGroup::ALL {
        let (mut rows, complete) = match input.scope {
            ListScope::InLibrary => (local_rows(input, group), true),
            ListScope::All => (
                index_rows(input, group),
                input.pages.page(group).is_complete(),
            ),
        };
        let total = rows.len();
        let mut status = None;
        if input.scope == ListScope::All {
            let shown = input.pages.shown(group);
            rows.truncate(shown);
            status = group_status(group, input.pages.page(group), total > rows.len());
        }
        if status
            .as_ref()
            .is_some_and(IndexListStatusRowVm::at_list_start)
        {
            if first_page_status_seen {
                status = None;
            } else {
                first_page_status_seen = true;
                if rows.is_empty() {
                    leading = Some(ArtistSection {
                        group,
                        heading: None,
                        rows: Vec::new(),
                        status: status.take(),
                    });
                    continue;
                }
            }
        }
        if rows.is_empty() && status.is_none() {
            continue;
        }
        let heading =
            (!rows.is_empty()).then(|| ArtistGroupHeadingVm::new(group, complete.then_some(total)));
        sections.push(ArtistSection {
            group,
            heading,
            rows,
            status,
        });
    }
    sections.splice(0..0, leading);
    sections
}

/// The local rows of `group`, in the sort of the list (musicindex ADR 0053
/// R-LIST-9 and R-LIST-12).
fn local_rows(input: &ArtistSectionsInput<'_>, group: ArtistGroup) -> Vec<DestinationListRowVm> {
    let mut rows: Vec<LibraryArtistRow> = input
        .library
        .iter()
        .filter(|entry| entry.group == group)
        .cloned()
        .collect();
    if input.newest {
        sort_by_newest(&mut rows, input.local_reversed);
    } else {
        rows.sort_by(|left, right| left.order.local_cmp(&right.order));
        if input.local_reversed {
            rows.reverse();
        }
    }
    rows.into_iter().map(|entry| entry.row).collect()
}

/// The rows of `group` in the scope "All": its Library rows merged with the
/// rows of its own page, with the letter of the list.
fn index_rows(input: &ArtistSectionsInput<'_>, group: ArtistGroup) -> Vec<DestinationListRowVm> {
    let page = input.pages.page(group);
    let rows = group_rows(
        group,
        &input.library,
        page,
        input.matches,
        input.newest,
        input.index_reversed,
    );
    if input.newest {
        // The node refuses `from` with the sort Newest, so the letter does
        // not apply there.
        return rows;
    }
    rows.into_iter()
        .filter(|row| name_at_or_after(&row.label, page.from(), input.index_reversed))
        .collect()
}

/// The status row of the group `group` in the scope "All". A limited group
/// gives "Show more" where the page would give "Load more". It gives
/// "Show more" too when the group holds more loaded rows than it shows.
fn group_status(
    group: ArtistGroup,
    page: &IndexNameListPage<ArtistNameRow>,
    rows_hidden: bool,
) -> Option<IndexListStatusRowVm> {
    let status = page
        .status_row(IndexNameList::Artists)
        .map(|row| scoped(row, group));
    if !group.is_capped() {
        return status;
    }
    match status {
        Some(IndexListStatusRowVm::LoadMore(action)) => Some(show_more(group, action.available)),
        None if rows_hidden => Some(show_more(group, true)),
        other => other,
    }
}

fn show_more(group: ArtistGroup, available: bool) -> IndexListStatusRowVm {
    IndexListStatusRowVm::LoadMore(IndexListActionDisplay {
        id: format!("{STATUS_ID_PREFIX}{}-show-more", group.slug()),
        label: "Show more",
        a11y_label: format!("Show more: {}", group.heading()),
        available,
    })
}

/// Gives the status row of the page of `group` an element id of its own, so
/// that the three groups never share an id in one list.
fn scoped(row: IndexListStatusRowVm, group: ArtistGroup) -> IndexListStatusRowVm {
    let scope = |id: String| {
        id.replacen(
            STATUS_ID_PREFIX,
            &format!("{STATUS_ID_PREFIX}{}-", group.slug()),
            1,
        )
    };
    match row {
        IndexListStatusRowVm::Loading { element_id, label } => IndexListStatusRowVm::Loading {
            element_id: scope(element_id),
            label,
        },
        IndexListStatusRowVm::Failed {
            element_id,
            first_page,
            message,
            detail,
            mut retry,
        } => {
            retry.id = scope(retry.id);
            IndexListStatusRowVm::Failed {
                element_id: scope(element_id),
                first_page,
                message,
                detail,
                retry,
            }
        }
        IndexListStatusRowVm::LoadMore(mut action) => {
            action.id = scope(action.id);
            IndexListStatusRowVm::LoadMore(action)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Musicindex ADR 0057 R-SITE-15 item 3: a count has a comma between
    /// each three digits.
    #[test]
    fn adr_0057_r_site_15_counts_have_thousands_separators() {
        assert_eq!(with_thousands(7), "7");
        assert_eq!(with_thousands(280), "280");
        assert_eq!(with_thousands(1_773), "1,773");
        assert_eq!(with_thousands(1_234_567), "1,234,567");
    }

    /// Musicindex ADR 0057 R-ARTIST-1 item 1: the order of the groups is the
    /// order of the stated artists, the inferred artists and the names.
    #[test]
    fn adr_0057_r_artist_1_group_order_is_stated_inferred_names() {
        assert_eq!(
            ArtistGroup::ALL,
            [
                ArtistGroup::Stated,
                ArtistGroup::Inferred,
                ArtistGroup::Names
            ]
        );
        assert!(ArtistGroup::Stated < ArtistGroup::Inferred);
        assert!(ArtistGroup::Inferred < ArtistGroup::Names);
        assert!(ArtistGroup::Names
            .intro()
            .contains("No artist feed confirms"));
    }
}
