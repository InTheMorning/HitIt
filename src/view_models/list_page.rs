//! The list pages of the Music destinations (ADR 0089 Decisions 5 and 6,
//! musicindex ADR 0027 R-LIST-1).
//!
//! Each list page keeps its own sort, view and scope. This module gives
//! the page keys, the defaults of Decision 5, the scope model, and the
//! title row of a list page (`ListPageHeadVm`). It has no renderer type.
//! `src/ui/composites/list_page_head.rs` draws the title row.
//!
//! ADR 0089 packet 005, Part A, adds the sort menu and the letter bar
//! (`LetterBarVm` in `src/view_models/letter_bar.rs`). The sort menu lists
//! the sorts of the page. The view icons still appear on no page.

#![warn(clippy::pedantic)]

use crate::view_models::letter_bar::{LetterBarVm, LetterKey};
use crate::view_models::release_download_state::ReleaseDownloadState;
use crate::view_models::workspace::FrameHeaderActionDisplay;

/// A list page of the Music destinations. Packet 004 has three keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListPageKey {
    /// The Releases list.
    Releases,
    /// The Artists list.
    Artists,
    /// The playlists of the curator.
    MyPlaylists,
}

impl ListPageKey {
    /// The noun of the items, for example "releases" in "12 releases".
    #[must_use]
    pub(crate) const fn noun(self) -> &'static str {
        match self {
            Self::Releases => "releases",
            Self::Artists => "artists",
            Self::MyPlaylists => "playlists",
        }
    }

    /// The title of the page, shown in the title row.
    #[must_use]
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Releases => "Releases",
            Self::Artists => "Artists",
            Self::MyPlaylists => "My playlists",
        }
    }

    /// The views that the page offers. Packet 004 gives each page only the
    /// rows view, because no page has a tiles renderer yet (packet 005).
    #[must_use]
    pub(crate) const fn views(self) -> &'static [ListView] {
        match self {
            Self::Releases | Self::Artists | Self::MyPlaylists => &[ListView::Rows],
        }
    }

    /// The sorts that the page offers, in menu order. The first sort is the
    /// default (ADR 0089 Decision 5). A page with no sort gives none.
    #[must_use]
    pub(crate) const fn sorts(self) -> &'static [ListSort] {
        match self {
            Self::Releases => &[ListSort::Newest, ListSort::AToZ],
            Self::Artists => &[ListSort::AToZ],
            Self::MyPlaylists => &[],
        }
    }
}

/// The sort of a list page (ADR 0089 Decision 5, R-LIST-1 item 1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListSort {
    /// The newest item first.
    Newest,
    /// The items by name, A to Z.
    AToZ,
}

impl ListSort {
    /// The visible label of the sort.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Newest => "Newest",
            Self::AToZ => "A to Z",
        }
    }

    /// The stable element id of the sort menu entry.
    #[must_use]
    pub(crate) const fn option_id(self) -> &'static str {
        match self {
            Self::Newest => "music-list-sort-newest",
            Self::AToZ => "music-list-sort-a-to-z",
        }
    }
}

/// The view of a list page (ADR 0089 Decision 5, R-LIST-1 item 3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListView {
    /// One row for each item. Packet 005 adds the tiles view.
    Rows,
}

/// The scope of a list page (ADR 0089 Decision 5). The control reads
/// "All | In Library", and `All` is the default.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ListScope {
    /// The Library items and the `MusicIndex` items.
    #[default]
    All,
    /// The Library items only.
    InLibrary,
}

impl ListScope {
    /// Whether the scope holds an item. `library_state` is the download
    /// state of the item in the Library, or `None` when the item is not in
    /// the Library, for example a release the curator removed (ADR 0089
    /// Decision 5). `All` holds each item. `In Library` holds the items
    /// that are downloaded, partly downloaded, downloading, queued or
    /// failed.
    #[must_use]
    pub(crate) const fn holds(self, library_state: Option<ReleaseDownloadState>) -> bool {
        match self {
            Self::All => true,
            Self::InLibrary => matches!(
                library_state,
                Some(
                    ReleaseDownloadState::Downloaded
                        | ReleaseDownloadState::Partial { .. }
                        | ReleaseDownloadState::Downloading
                        | ReleaseDownloadState::Queued
                        | ReleaseDownloadState::Failed
                )
            ),
        }
    }
}

/// The state of one list page: its sort, its view and its scope. `None`
/// means the page has no such control (ADR 0089 Decision 5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ListPageState {
    /// The sort, or `None` when the curator sets the sequence.
    pub(crate) sort: Option<ListSort>,
    /// The view.
    pub(crate) view: ListView,
    /// The scope, or `None` when the page has no scope.
    pub(crate) scope: Option<ListScope>,
    /// The letter that the letter bar selected, or `None`. The letter bar
    /// shows only with the sort A to Z.
    pub(crate) letter: Option<LetterKey>,
}

impl ListPageState {
    /// The state of `key` at the start (ADR 0089 Decision 5 table).
    #[must_use]
    pub(crate) const fn default_for(key: ListPageKey) -> Self {
        match key {
            ListPageKey::Releases => Self {
                sort: Some(ListSort::Newest),
                view: ListView::Rows,
                scope: Some(ListScope::All),
                letter: None,
            },
            ListPageKey::Artists => Self {
                sort: Some(ListSort::AToZ),
                view: ListView::Rows,
                scope: Some(ListScope::All),
                letter: None,
            },
            ListPageKey::MyPlaylists => Self {
                sort: None,
                view: ListView::Rows,
                scope: None,
                letter: None,
            },
        }
    }
}

/// The state of each list page. Each page has its own state, so a change
/// of one page does not change another.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListPages {
    releases: ListPageState,
    artists: ListPageState,
    my_playlists: ListPageState,
}

impl Default for ListPages {
    fn default() -> Self {
        Self {
            releases: ListPageState::default_for(ListPageKey::Releases),
            artists: ListPageState::default_for(ListPageKey::Artists),
            my_playlists: ListPageState::default_for(ListPageKey::MyPlaylists),
        }
    }
}

impl ListPages {
    /// The state of the page `key`.
    #[must_use]
    pub(crate) const fn get(&self, key: ListPageKey) -> &ListPageState {
        match key {
            ListPageKey::Releases => &self.releases,
            ListPageKey::Artists => &self.artists,
            ListPageKey::MyPlaylists => &self.my_playlists,
        }
    }

    /// The state of the page `key`, for a change.
    pub(crate) const fn get_mut(&mut self, key: ListPageKey) -> &mut ListPageState {
        match key {
            ListPageKey::Releases => &mut self.releases,
            ListPageKey::Artists => &mut self.artists,
            ListPageKey::MyPlaylists => &mut self.my_playlists,
        }
    }

    /// Sets the scope of `key`. A page with no scope keeps `None`.
    pub(crate) fn set_scope(&mut self, key: ListPageKey, scope: ListScope) {
        let state = self.get_mut(key);
        if state.scope.is_some() {
            state.scope = Some(scope);
        }
    }

    /// Sets the sort of `key` (ADR 0089 Decision 5). A sort that the page
    /// does not offer changes nothing. A new sort clears the letter, because
    /// the letter belongs to the sort A to Z.
    pub(crate) fn set_sort(&mut self, key: ListPageKey, sort: ListSort) {
        if !key.sorts().contains(&sort) {
            return;
        }
        let state = self.get_mut(key);
        state.sort = Some(sort);
        state.letter = None;
    }

    /// Sets the letter that the letter bar of `key` selected.
    pub(crate) fn set_letter(&mut self, key: ListPageKey, letter: Option<LetterKey>) {
        self.get_mut(key).letter = letter;
    }
}

/// One option of the scope control (R-LIST-1 item 2).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListScopeOptionVm {
    /// Stable element id.
    pub(crate) id: &'static str,
    /// The scope that the option selects.
    pub(crate) scope: ListScope,
    /// The visible label, "All" or "In Library".
    pub(crate) label: &'static str,
    /// The accessibility label.
    pub(crate) a11y_label: String,
    /// The count beside the label. Only "In Library" shows one (ADR 0089
    /// Decision 5, Answer 5). `All` shows none, because the Index gives no
    /// total.
    pub(crate) count_text: Option<String>,
}

/// The scope control of a list page: "All | In Library".
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListScopeControlVm {
    /// Stable element id of the control.
    pub(crate) control_id: &'static str,
    /// The selected scope.
    pub(crate) selected: ListScope,
    /// The options, "All" first.
    pub(crate) options: [ListScopeOptionVm; 2],
}

/// One entry of the sort menu.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListSortOptionVm {
    /// Stable element id.
    pub(crate) id: &'static str,
    /// The sort of the entry.
    pub(crate) sort: ListSort,
    /// The visible label.
    pub(crate) label: &'static str,
    /// The accessibility label. It names the current sort too, so the
    /// check mark is not the only sign of it.
    pub(crate) a11y_label: String,
    /// Whether the entry is the current sort. The menu shows a check mark
    /// beside it.
    pub(crate) selected: bool,
}

/// The sort button of a list page (R-LIST-1 item 1), for example
/// "Newest".
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListSortButtonVm {
    /// Stable element id.
    pub(crate) id: &'static str,
    /// The visible label, the current sort.
    pub(crate) label: &'static str,
    /// The accessibility label.
    pub(crate) a11y_label: String,
    /// The menu entries.
    pub(crate) options: Vec<ListSortOptionVm>,
}

/// The title row of a list page (ADR 0089 Decision 6, R-LIST-1 item 4):
/// the title at the left, then the sort, the view icons and Select at the
/// right. The scope control sits below the title.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListPageHeadVm {
    /// The page title.
    pub(crate) title: &'static str,
    /// The scope control, or `None` when the page has no scope.
    pub(crate) scope: Option<ListScopeControlVm>,
    /// The sort button, or `None` when the page has no sort.
    pub(crate) sort: Option<ListSortButtonVm>,
    /// The letter bar, or `None` unless the sort is A to Z (R-LIST-1 item
    /// 2). It sits below the scope control.
    pub(crate) letters: Option<LetterBarVm>,
    /// The current view. `None` when the page offers one view only, so no
    /// view icon shows.
    pub(crate) view: Option<ListView>,
    /// The Select action of the page, or `None` when no item can be
    /// selected.
    pub(crate) select: Option<FrameHeaderActionDisplay>,
}

impl ListPageHeadVm {
    /// The title row of `key` in `state`. `library_count` is the number of
    /// Library releases or artists. "In Library" shows the count with each
    /// selection, and "All" never shows one (ADR 0089 Decision 5).
    #[must_use]
    pub(crate) fn new(
        key: ListPageKey,
        state: ListPageState,
        library_count: Option<usize>,
        select: Option<FrameHeaderActionDisplay>,
    ) -> Self {
        let view = (key.views().len() > 1).then_some(state.view);
        Self {
            title: key.title(),
            scope: state
                .scope
                .map(|selected| scope_control(key, selected, library_count)),
            sort: state.sort.map(|sort| sort_button(key, sort)),
            letters: (state.sort == Some(ListSort::AToZ))
                .then(|| LetterBarVm::new(key, state.letter)),
            view,
            select,
        }
    }
}

fn scope_control(
    key: ListPageKey,
    selected: ListScope,
    library_count: Option<usize>,
) -> ListScopeControlVm {
    let noun = key.noun();
    let (control_id, all_id, library_id) = match key {
        ListPageKey::Releases => (
            "music-scope-releases",
            "music-scope-releases-all",
            "music-scope-releases-in-library",
        ),
        ListPageKey::Artists => (
            "music-scope-artists",
            "music-scope-artists-all",
            "music-scope-artists-in-library",
        ),
        ListPageKey::MyPlaylists => (
            "music-scope-playlists",
            "music-scope-playlists-all",
            "music-scope-playlists-in-library",
        ),
    };
    ListScopeControlVm {
        control_id,
        selected,
        options: [
            ListScopeOptionVm {
                id: all_id,
                scope: ListScope::All,
                label: "All",
                a11y_label: format!("Show all {noun}, Library and MusicIndex"),
                count_text: None,
            },
            ListScopeOptionVm {
                id: library_id,
                scope: ListScope::InLibrary,
                label: "In Library",
                a11y_label: format!("Show {noun} in Library only"),
                count_text: library_count.map(|count| format!("{count} {noun}")),
            },
        ],
    }
}

/// Whether the list page must send its first `MusicIndex` name request
/// (ADR 0089 Decision 5). Only the scope "All" reads the `MusicIndex`
/// name list. Each page sends that request at most once, so
/// `page_not_started` must be true for the request to be sent.
#[must_use]
pub(crate) const fn needs_first_page(scope: ListScope, page_not_started: bool) -> bool {
    matches!(scope, ListScope::All) && page_not_started
}

fn sort_button(key: ListPageKey, sort: ListSort) -> ListSortButtonVm {
    let id = match key {
        ListPageKey::Releases => "music-list-sort-releases",
        ListPageKey::Artists => "music-list-sort-artists",
        ListPageKey::MyPlaylists => "music-list-sort-playlists",
    };
    ListSortButtonVm {
        id,
        label: sort.label(),
        a11y_label: format!("Sort {} by {}", key.noun(), sort.label()),
        options: key
            .sorts()
            .iter()
            .map(|&option| {
                let selected = option == sort;
                let current = if selected { ", current sort" } else { "" };
                ListSortOptionVm {
                    id: option.option_id(),
                    sort: option,
                    label: option.label(),
                    a11y_label: format!("Sort {} by {}{current}", key.noun(), option.label()),
                    selected,
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0089 Decision 5 table, for the rows that packet 004 has.
    #[test]
    fn adr_0089_list_page_default_for_gives_the_decision_5_table() {
        let releases = ListPageState::default_for(ListPageKey::Releases);
        assert_eq!(releases.sort, Some(ListSort::Newest));
        assert_eq!(releases.view, ListView::Rows);
        assert_eq!(releases.scope, Some(ListScope::All));

        let artists = ListPageState::default_for(ListPageKey::Artists);
        assert_eq!(artists.sort, Some(ListSort::AToZ));
        assert_eq!(artists.view, ListView::Rows);
        assert_eq!(artists.scope, Some(ListScope::All));

        let playlists = ListPageState::default_for(ListPageKey::MyPlaylists);
        assert_eq!(playlists.sort, None);
        assert_eq!(playlists.view, ListView::Rows);
        assert_eq!(playlists.scope, None);
    }

    /// A change of one page does not change another page (ADR 0089
    /// Decision 5).
    #[test]
    fn adr_0089_list_page_scope_change_keeps_other_pages() {
        let mut pages = ListPages::default();
        pages.set_scope(ListPageKey::Releases, ListScope::InLibrary);
        assert_eq!(
            pages.get(ListPageKey::Releases).scope,
            Some(ListScope::InLibrary)
        );
        assert_eq!(pages.get(ListPageKey::Artists).scope, Some(ListScope::All));
        pages.set_scope(ListPageKey::MyPlaylists, ListScope::InLibrary);
        assert_eq!(pages.get(ListPageKey::MyPlaylists).scope, None);
        assert_eq!(
            pages.get(ListPageKey::Releases).scope,
            Some(ListScope::InLibrary)
        );
    }

    /// ADR 0089 Decision 5: In Library holds the downloaded, partly
    /// downloaded, downloading, queued and failed releases. A removed
    /// release, with no Library state, is in All only.
    #[test]
    fn adr_0089_list_page_scope_holds_library_states_and_removed_releases() {
        for state in [
            ReleaseDownloadState::Downloaded,
            ReleaseDownloadState::Partial {
                downloaded: 1,
                total: 3,
            },
            ReleaseDownloadState::Downloading,
            ReleaseDownloadState::Queued,
            ReleaseDownloadState::Failed,
        ] {
            assert!(ListScope::InLibrary.holds(Some(state)), "{state:?}");
            assert!(ListScope::All.holds(Some(state)), "{state:?}");
        }
        assert!(!ListScope::InLibrary.holds(None), "a removed release");
        assert!(ListScope::All.holds(None), "a removed release");
        assert!(!ListScope::InLibrary.holds(Some(ReleaseDownloadState::NotDownloaded)));
    }

    /// ADR 0089 Decision 5: "In Library" shows a count with each selection,
    /// and "All" shows none, even when a count is given.
    #[test]
    fn adr_0089_list_page_head_counts_only_in_library() {
        let state = ListPageState {
            scope: Some(ListScope::InLibrary),
            ..ListPageState::default_for(ListPageKey::Releases)
        };
        let head = ListPageHeadVm::new(ListPageKey::Releases, state, Some(12), None);
        let scope = head.scope.expect("Releases has a scope");
        assert_eq!(scope.selected, ListScope::InLibrary);
        assert_eq!(scope.options[0].label, "All");
        assert_eq!(scope.options[0].count_text, None);
        assert_eq!(scope.options[1].label, "In Library");
        assert_eq!(scope.options[1].count_text.as_deref(), Some("12 releases"));

        let all = ListPageState::default_for(ListPageKey::Releases);
        let head = ListPageHeadVm::new(ListPageKey::Releases, all, Some(12), None);
        let scope = head.scope.expect("Releases has a scope");
        assert_eq!(scope.selected, ListScope::All);
        assert_eq!(scope.options[0].count_text, None, "All shows no count");
        assert_eq!(scope.options[1].count_text.as_deref(), Some("12 releases"));
    }

    /// ADR 0089 Decision 5: the first `MusicIndex` request belongs to the
    /// scope "All" only, and only once.
    #[test]
    fn adr_0089_list_page_needs_first_page_for_all_and_not_in_library() {
        assert!(needs_first_page(ListScope::All, true));
        assert!(!needs_first_page(ListScope::All, false), "sent once only");
        assert!(!needs_first_page(ListScope::InLibrary, true));
        assert!(!needs_first_page(ListScope::InLibrary, false));
    }

    /// The title row: the sort button shows the current sort, no view
    /// icon shows on a page with one view, and My playlists has no sort
    /// button and no scope.
    #[test]
    fn adr_0089_list_page_head_shows_current_sort_and_no_view_icon() {
        let releases = ListPageState::default_for(ListPageKey::Releases);
        let head = ListPageHeadVm::new(ListPageKey::Releases, releases, None, None);
        assert_eq!(head.title, "Releases");
        assert_eq!(head.view, None);
        let sort = head.sort.expect("Releases has a sort");
        assert_eq!(sort.label, "Newest");

        let playlists = ListPageState::default_for(ListPageKey::MyPlaylists);
        let head = ListPageHeadVm::new(ListPageKey::MyPlaylists, playlists, None, None);
        assert_eq!(head.title, "My playlists");
        assert!(head.sort.is_none());
        assert!(head.scope.is_none());
        assert!(head.letters.is_none());
    }

    /// R-LIST-1 item 1 (ADR 0089 Decision 5): Releases offers Newest and A
    /// to Z, Artists offers A to Z only, and the check mark marks the
    /// current sort only.
    #[test]
    fn adr_0089_list_page_sort_menu_lists_the_page_sorts_with_a_check_mark() {
        let releases = ListPageState::default_for(ListPageKey::Releases);
        let sort = ListPageHeadVm::new(ListPageKey::Releases, releases, None, None)
            .sort
            .expect("Releases has a sort");
        let labels: Vec<&str> = sort.options.iter().map(|option| option.label).collect();
        assert_eq!(labels, ["Newest", "A to Z"]);
        let checked: Vec<&str> = sort
            .options
            .iter()
            .filter(|option| option.selected)
            .map(|option| option.label)
            .collect();
        assert_eq!(checked, ["Newest"], "one check mark, on the current sort");
        assert!(sort.options[0].a11y_label.ends_with("current sort"));
        assert!(!sort.options[1].a11y_label.contains("current"));

        let artists = ListPageState::default_for(ListPageKey::Artists);
        let sort = ListPageHeadVm::new(ListPageKey::Artists, artists, None, None)
            .sort
            .expect("Artists has a sort");
        assert_eq!(sort.options.len(), 1, "Artists has one sort");
        assert_eq!(sort.options[0].label, "A to Z");
        assert!(sort.options[0].selected);
    }

    /// R-LIST-1 item 2 (ADR 0089 Decision 6): the letter bar shows only
    /// with the sort A to Z, and it has the letters A to Z and "#".
    #[test]
    fn adr_0089_list_page_letter_bar_shows_only_with_a_to_z() {
        let newest = ListPageState::default_for(ListPageKey::Releases);
        let head = ListPageHeadVm::new(ListPageKey::Releases, newest, None, None);
        assert!(head.letters.is_none(), "Newest shows no letter bar");

        let mut a_to_z = newest;
        a_to_z.sort = Some(ListSort::AToZ);
        let letters = ListPageHeadVm::new(ListPageKey::Releases, a_to_z, None, None)
            .letters
            .expect("A to Z shows the letter bar");
        assert_eq!(letters.letters.len(), 27, "A to Z and #");
        assert_eq!(letters.letters[0].label, "A");
        assert_eq!(letters.letters[25].label, "Z");
        assert_eq!(letters.letters[26].label, "#");

        let artists = ListPageState::default_for(ListPageKey::Artists);
        assert!(
            ListPageHeadVm::new(ListPageKey::Artists, artists, None, None)
                .letters
                .is_some()
        );
    }

    /// ADR 0089 Decision 5: a sort change sets the new sort on its page
    /// only, clears that page's letter, and a sort the page does not offer
    /// changes nothing.
    #[test]
    fn adr_0089_list_page_set_sort_changes_its_page_only() {
        let mut pages = ListPages::default();
        pages.set_letter(ListPageKey::Releases, Some(LetterKey::Letter('B')));
        pages.set_sort(ListPageKey::Releases, ListSort::AToZ);
        let releases = pages.get(ListPageKey::Releases);
        assert_eq!(releases.sort, Some(ListSort::AToZ));
        assert_eq!(releases.letter, None, "a new sort clears the letter");
        assert_eq!(pages.get(ListPageKey::Artists).sort, Some(ListSort::AToZ));

        pages.set_sort(ListPageKey::Artists, ListSort::Newest);
        assert_eq!(
            pages.get(ListPageKey::Artists).sort,
            Some(ListSort::AToZ),
            "Artists does not offer Newest"
        );
        pages.set_sort(ListPageKey::MyPlaylists, ListSort::AToZ);
        assert_eq!(pages.get(ListPageKey::MyPlaylists).sort, None);
    }
}
