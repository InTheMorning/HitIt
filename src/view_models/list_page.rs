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
//! the sorts of the page. ADR 0089 packet 005, Part C, adds the tiles view
//! of the Releases list and its view icons (R-LIST-1 item 3).

#![warn(clippy::pedantic)]

use crate::view_models::letter_bar::{LetterBarVm, LetterKey};
use crate::view_models::release_download_state::ReleaseDownloadState;
use crate::view_models::search_results::SearchPageActionDisplay;
use crate::view_models::workspace::FrameHeaderActionDisplay;

/// A list page. The Music list pages have three keys. The two search keys
/// hold the sort and the view of the results page and of a saved search
/// (ADR 0089 Decisions 4 and 5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListPageKey {
    /// The Releases list.
    Releases,
    /// The Artists list.
    Artists,
    /// The playlists of the curator.
    MyPlaylists,
    /// The public playlists of the Index, the musicL feeds (ADR 0089
    /// Decision 1, packet 009).
    PublicPlaylists,
    /// The live items of the Index that are live now (ADR 0089 Decision 1,
    /// packet 011). It has no scope and no sort.
    Live,
    /// The results page of the toolbar search.
    Results,
    /// The page of a saved search.
    SavedSearch,
}

impl ListPageKey {
    /// The noun of the items, for example "releases" in "12 releases".
    #[must_use]
    pub(crate) const fn noun(self) -> &'static str {
        match self {
            Self::Releases => "releases",
            Self::Artists => "artists",
            Self::MyPlaylists | Self::PublicPlaylists => "playlists",
            Self::Live => "live items",
            Self::Results | Self::SavedSearch => "results",
        }
    }

    /// The title of the page, shown in the title row.
    #[must_use]
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Releases => "Releases",
            Self::Artists => "Artists",
            Self::MyPlaylists => "My playlists",
            Self::PublicPlaylists => "Public playlists",
            Self::Live => "Live",
            Self::Results => "Results",
            Self::SavedSearch => "Saved search",
        }
    }

    /// The views that the page offers, in icon order (ADR 0089 Decision
    /// 5). Releases offers rows and tiles. Artists, My playlists and the
    /// search keys offer rows only, so they show no view icon. The search
    /// keys have no tiles because the Releases tiles are tied to the
    /// Library list (ADR 0089 packet 008 Part C, deviation).
    #[must_use]
    pub(crate) const fn views(self) -> &'static [ListView] {
        match self {
            Self::Releases | Self::PublicPlaylists => &[ListView::Rows, ListView::Tiles],
            Self::Artists | Self::MyPlaylists | Self::Live | Self::Results | Self::SavedSearch => {
                &[ListView::Rows]
            }
        }
    }

    /// Whether the page can reverse all its rows in `scope` (musicindex ADR
    /// 0043 R-LIST-7 items 9 and 10). A list reverses only when each of its
    /// rows is local: Releases and Artists with the scope In Library, My
    /// playlists, and Results with the scope In Library. A list from the
    /// Index cannot reverse until the Index gives a direction, so its
    /// reversed state has no effect.
    #[must_use]
    pub(crate) const fn reverses_in(self, scope: ListScope) -> bool {
        match self {
            Self::Releases | Self::Artists | Self::Results => {
                matches!(scope, ListScope::InLibrary)
            }
            Self::MyPlaylists => true,
            Self::PublicPlaylists | Self::Live | Self::SavedSearch => false,
        }
    }

    /// The sorts that the page offers, in menu order. The first sort is the
    /// default (ADR 0089 Decision 5). A page with no sort gives none.
    #[must_use]
    pub(crate) const fn sorts(self) -> &'static [ListSort] {
        match self {
            Self::Releases | Self::PublicPlaylists | Self::Results | Self::SavedSearch => {
                &[ListSort::Newest, ListSort::AToZ]
            }
            Self::Artists => &[ListSort::AToZ],
            Self::MyPlaylists | Self::Live => &[],
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
    /// The visible name of the sort. A reversed list names its reversed
    /// order (musicindex ADR 0043 R-LIST-7 item 5).
    #[must_use]
    pub(crate) const fn name(self, reversed: bool) -> &'static str {
        match (self, reversed) {
            (Self::Newest, false) => "Newest",
            (Self::Newest, true) => "Oldest",
            (Self::AToZ, false) => "A to Z",
            (Self::AToZ, true) => "Z to A",
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
    /// One row for each item.
    Rows,
    /// One tile for each item, in a grid.
    Tiles,
}

impl ListView {
    /// The visible label of the view, used by the accessibility label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Rows => "rows",
            Self::Tiles => "tiles",
        }
    }

    /// The stable element id of the view icon button.
    #[must_use]
    pub(crate) const fn option_id(self) -> &'static str {
        match self {
            Self::Rows => "music-list-view-rows",
            Self::Tiles => "music-list-view-tiles",
        }
    }
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
    /// Whether the order of the list is reversed (musicindex ADR 0043
    /// R-LIST-7 item 6). It starts false, and a new sort keeps it.
    pub(crate) reversed: bool,
}

impl ListPageState {
    /// Whether the order of the page `key` is reversed in `scope`. A list
    /// that cannot reverse in `scope` is never reversed, whatever the
    /// stored state is. The stored state stays for a later scope.
    #[must_use]
    pub(crate) const fn reversed_in(self, key: ListPageKey, scope: ListScope) -> bool {
        self.reversed && key.reverses_in(scope)
    }

    /// The state of `key` at the start (ADR 0089 Decision 5 table).
    #[must_use]
    pub(crate) const fn default_for(key: ListPageKey) -> Self {
        match key {
            ListPageKey::Releases => Self {
                sort: Some(ListSort::Newest),
                view: ListView::Rows,
                scope: Some(ListScope::All),
                letter: None,
                reversed: false,
            },
            ListPageKey::Artists => Self {
                sort: Some(ListSort::AToZ),
                view: ListView::Rows,
                scope: Some(ListScope::All),
                letter: None,
                reversed: false,
            },
            // ADR 0089 Decision 5: rows, no sort and no scope.
            ListPageKey::MyPlaylists | ListPageKey::Live => Self {
                sort: None,
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
            },
            // ADR 0089 Decision 5: Newest, tiles, and no scope.
            ListPageKey::PublicPlaylists => Self {
                sort: Some(ListSort::Newest),
                view: ListView::Tiles,
                scope: None,
                letter: None,
                reversed: false,
            },
            // The query holds the scope of a search page (ADR 0089 Decision
            // 4), so the page state has none.
            ListPageKey::Results | ListPageKey::SavedSearch => Self {
                sort: Some(ListSort::Newest),
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
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
    public_playlists: ListPageState,
    live: ListPageState,
    results: ListPageState,
    saved_search: ListPageState,
}

impl Default for ListPages {
    fn default() -> Self {
        Self {
            releases: ListPageState::default_for(ListPageKey::Releases),
            artists: ListPageState::default_for(ListPageKey::Artists),
            my_playlists: ListPageState::default_for(ListPageKey::MyPlaylists),
            public_playlists: ListPageState::default_for(ListPageKey::PublicPlaylists),
            live: ListPageState::default_for(ListPageKey::Live),
            results: ListPageState::default_for(ListPageKey::Results),
            saved_search: ListPageState::default_for(ListPageKey::SavedSearch),
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
            ListPageKey::PublicPlaylists => &self.public_playlists,
            ListPageKey::Live => &self.live,
            ListPageKey::Results => &self.results,
            ListPageKey::SavedSearch => &self.saved_search,
        }
    }

    /// The state of the page `key`, for a change.
    pub(crate) const fn get_mut(&mut self, key: ListPageKey) -> &mut ListPageState {
        match key {
            ListPageKey::Releases => &mut self.releases,
            ListPageKey::Artists => &mut self.artists,
            ListPageKey::MyPlaylists => &mut self.my_playlists,
            ListPageKey::PublicPlaylists => &mut self.public_playlists,
            ListPageKey::Live => &mut self.live,
            ListPageKey::Results => &mut self.results,
            ListPageKey::SavedSearch => &mut self.saved_search,
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

    /// Sets the reversed state of `key`. A page with no sort keeps false,
    /// except My playlists, which reverses the sequence of the curator. Each
    /// page keeps its own state.
    pub(crate) fn set_reversed(&mut self, key: ListPageKey, reversed: bool) {
        if key.sorts().is_empty() && key != ListPageKey::MyPlaylists {
            return;
        }
        self.get_mut(key).reversed = reversed;
    }

    /// Sets the letter that the letter bar of `key` selected.
    pub(crate) fn set_letter(&mut self, key: ListPageKey, letter: Option<LetterKey>) {
        self.get_mut(key).letter = letter;
    }

    /// Sets the view of `key` (ADR 0089 Decision 5). A view that the page
    /// does not offer changes nothing. Each page keeps its own view.
    pub(crate) fn set_view(&mut self, key: ListPageKey, view: ListView) {
        if !key.views().contains(&view) {
            return;
        }
        self.get_mut(key).view = view;
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

/// One view icon of a list page (R-LIST-1 item 3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListViewOptionVm {
    /// Stable element id.
    pub(crate) id: &'static str,
    /// The view that the icon selects.
    pub(crate) view: ListView,
    /// The accessibility label, for example "Show releases as tiles".
    pub(crate) a11y_label: String,
    /// Whether the icon is the current view. The current icon has a tint.
    pub(crate) selected: bool,
}

/// The view icons of a list page: one icon button for each view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListViewControlVm {
    /// The view icons, in the order of [`ListPageKey::views`].
    pub(crate) options: Vec<ListViewOptionVm>,
}

/// The icon of the reverse button. It is a typed name, so the view model
/// carries no renderer type. The renderer maps it to an icon.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListReverseIcon {
    /// The usual order.
    SortUsual,
    /// The reversed order.
    SortReversed,
}

/// The reverse button of a list page (musicindex ADR 0043 R-LIST-7).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListReverseVm {
    /// Stable element id.
    pub(crate) id: &'static str,
    /// The label, the tooltip and the accessibility label.
    pub(crate) label: &'static str,
    /// Whether the order is reversed. Assistive technology reads it.
    pub(crate) pressed: bool,
    /// The icon. It differs in the reversed order, so color is not the only
    /// sign of the state.
    pub(crate) icon: ListReverseIcon,
}

impl ListReverseVm {
    /// The reverse button for the state `reversed`.
    #[must_use]
    pub(crate) const fn new(reversed: bool) -> Self {
        Self {
            id: "music-list-reverse",
            label: "Reverse the Order",
            pressed: reversed,
            icon: if reversed {
                ListReverseIcon::SortReversed
            } else {
                ListReverseIcon::SortUsual
            },
        }
    }
}

/// The title row of a list page (ADR 0089 Decision 6, R-LIST-1 item 4):
/// the title at the left, then the sort, the view icons and Select at the
/// right. The scope control sits below the title.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListPageHeadVm {
    /// The page title. A search page gives the query text, or an empty text
    /// when its query field is open.
    pub(crate) title: String,
    /// The scope control, or `None` when the page has no scope.
    pub(crate) scope: Option<ListScopeControlVm>,
    /// The sort button, or `None` when the page has no sort.
    pub(crate) sort: Option<ListSortButtonVm>,
    /// The letter bar, or `None` unless the sort is A to Z (R-LIST-1 item
    /// 2). It sits below the scope control.
    pub(crate) letters: Option<LetterBarVm>,
    /// The view icons. `None` when the page offers one view only, so no
    /// view icon shows.
    pub(crate) view: Option<ListViewControlVm>,
    /// The reverse button, or `None` when the list cannot reverse its whole
    /// order (R-LIST-7 item 10). It sits after the view icons.
    pub(crate) reverse: Option<ListReverseVm>,
    /// The Select action of the page, or `None` when no item can be
    /// selected.
    pub(crate) select: Option<FrameHeaderActionDisplay>,
    /// The typed actions of a search page, for example "Save Query" and
    /// "Cancel", or "Delete…" (ADR 0089 Decision 4). They sit at the right
    /// of the title row, in this order. Empty for a Music list page.
    pub(crate) actions: Vec<SearchPageActionDisplay>,
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
        let scope = state.scope.unwrap_or_default();
        let reversed = state.reversed_in(key, scope);
        Self {
            title: key.title().to_owned(),
            scope: state
                .scope
                .map(|selected| scope_control(key, selected, library_count)),
            sort: state.sort.map(|sort| sort_button(key, sort, reversed)),
            letters: (state.sort == Some(ListSort::AToZ))
                .then(|| ordered_letters(LetterBarVm::new(key, state.letter), reversed)),
            view: (key.views().len() > 1).then(|| view_control(key, state.view)),
            reverse: key
                .reverses_in(scope)
                .then(|| ListReverseVm::new(state.reversed)),
            select,
            actions: Vec::new(),
        }
    }

    /// The title row of a page with a title and at most one typed action:
    /// the Search page, My playlists and the Inbox (ADR 0089 Decision 6).
    #[must_use]
    pub(crate) fn titled(title: &str, action: Option<SearchPageActionDisplay>) -> Self {
        Self {
            title: title.to_owned(),
            scope: None,
            sort: None,
            letters: None,
            view: None,
            reverse: None,
            select: None,
            actions: action.into_iter().collect(),
        }
    }

    /// This title row with the reverse button of the page `key` in `state`.
    /// A page that cannot reverse keeps no button.
    #[must_use]
    pub(crate) fn with_reverse_of(mut self, key: ListPageKey, state: ListPageState) -> Self {
        let scope = state.scope.unwrap_or_default();
        self.reverse = key
            .reverses_in(scope)
            .then(|| ListReverseVm::new(state.reversed));
        self
    }

    /// The title row of a search page (ADR 0089 Decision 4): the title, the
    /// scope "All | In Library" of the query, the sort of `state`, and the
    /// typed action of the page. The scope control shows no count, because
    /// the Index gives no total. A search page has no view icon and no
    /// letter bar.
    #[must_use]
    pub(crate) fn for_search(
        key: ListPageKey,
        state: ListPageState,
        title: String,
        scope: ListScope,
        actions: Vec<SearchPageActionDisplay>,
    ) -> Self {
        let reversed = state.reversed_in(key, scope);
        Self {
            title,
            scope: Some(search_scope_control(scope)),
            sort: state.sort.map(|sort| sort_button(key, sort, reversed)),
            letters: None,
            // Rows only: the search keys offer no view icon (ADR 0089 packet
            // 008 Part C, deviation, see `ListPageKey::views`).
            view: None,
            reverse: key
                .reverses_in(scope)
                .then(|| ListReverseVm::new(state.reversed)),
            select: None,
            actions,
        }
    }
}

/// The scope control of a search page (ADR 0089 Decision 4). It has no
/// count beside its labels.
fn search_scope_control(selected: ListScope) -> ListScopeControlVm {
    ListScopeControlVm {
        control_id: "music-scope-search",
        selected,
        options: [
            ListScopeOptionVm {
                id: "music-scope-search-all",
                scope: ListScope::All,
                label: "All",
                a11y_label: "Show all results, Library and MusicIndex".to_owned(),
                count_text: None,
            },
            ListScopeOptionVm {
                id: "music-scope-search-in-library",
                scope: ListScope::InLibrary,
                label: "In Library",
                a11y_label: "Show results in Library only".to_owned(),
                count_text: None,
            },
        ],
    }
}

fn view_control(key: ListPageKey, selected: ListView) -> ListViewControlVm {
    ListViewControlVm {
        options: key
            .views()
            .iter()
            .map(|&view| ListViewOptionVm {
                id: view.option_id(),
                view,
                a11y_label: format!("Show {} as {}", key.noun(), view.label()),
                selected: view == selected,
            })
            .collect(),
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
        // Public playlists and Live have no scope, so no scope control uses
        // these ids.
        ListPageKey::Live | ListPageKey::PublicPlaylists => (
            "music-scope-public-playlists",
            "music-scope-public-playlists-all",
            "music-scope-public-playlists-in-library",
        ),
        ListPageKey::Results | ListPageKey::SavedSearch => (
            "music-scope-search",
            "music-scope-search-all",
            "music-scope-search-in-library",
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

/// The letters of a reversed A to Z list run from Z to A (R-LIST-7 item 7).
/// The letter "#" stays last.
fn ordered_letters(mut bar: LetterBarVm, reversed: bool) -> LetterBarVm {
    if reversed {
        let named = bar.letters.len().saturating_sub(1);
        bar.letters[..named].reverse();
    }
    bar
}

fn sort_button(key: ListPageKey, sort: ListSort, reversed: bool) -> ListSortButtonVm {
    let id = match key {
        ListPageKey::Releases => "music-list-sort-releases",
        ListPageKey::Artists => "music-list-sort-artists",
        ListPageKey::MyPlaylists => "music-list-sort-playlists",
        ListPageKey::PublicPlaylists => "music-list-sort-public-playlists",
        ListPageKey::Live => "music-list-sort-live",
        ListPageKey::Results => "music-list-sort-results",
        ListPageKey::SavedSearch => "music-list-sort-saved-search",
    };
    ListSortButtonVm {
        id,
        label: sort.name(reversed),
        a11y_label: format!("Sort {} by {}", key.noun(), sort.name(reversed)),
        options: key
            .sorts()
            .iter()
            .map(|&option| {
                let selected = option == sort;
                let current = if selected { ", current sort" } else { "" };
                ListSortOptionVm {
                    id: option.option_id(),
                    sort: option,
                    label: option.name(reversed),
                    a11y_label: format!(
                        "Sort {} by {}{current}",
                        key.noun(),
                        option.name(reversed)
                    ),
                    selected,
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R-LIST-7 item 5: each sort has its reversed name, in the button and
    /// in each menu entry.
    #[test]
    fn adr_0043_task_002_reversed_names_show_in_button_and_menu() {
        assert_eq!(ListSort::Newest.name(false), "Newest");
        assert_eq!(ListSort::Newest.name(true), "Oldest");
        assert_eq!(ListSort::AToZ.name(false), "A to Z");
        assert_eq!(ListSort::AToZ.name(true), "Z to A");
        let state = ListPageState {
            reversed: true,
            scope: Some(ListScope::InLibrary),
            ..ListPageState::default_for(ListPageKey::Releases)
        };
        let sort = ListPageHeadVm::new(ListPageKey::Releases, state, None, None)
            .sort
            .expect("Releases has a sort");
        assert_eq!(sort.label, "Oldest");
        assert!(sort.a11y_label.contains("Oldest"));
        let labels: Vec<&str> = sort.options.iter().map(|option| option.label).collect();
        assert_eq!(labels, ["Oldest", "Z to A"]);
        assert!(sort.options[0].a11y_label.contains("Oldest"));
    }

    /// R-LIST-7 item 6: each list keeps its own reversed state. The state
    /// starts false, and a new sort keeps it. A page with no sort keeps false.
    #[test]
    fn adr_0043_task_002_each_list_keeps_its_own_reversed_state() {
        let mut pages = ListPages::default();
        assert!(!pages.get(ListPageKey::Releases).reversed);
        pages.set_reversed(ListPageKey::Releases, true);
        assert!(pages.get(ListPageKey::Releases).reversed);
        assert!(!pages.get(ListPageKey::Artists).reversed);
        assert!(!pages.get(ListPageKey::PublicPlaylists).reversed);
        pages.set_sort(ListPageKey::Releases, ListSort::AToZ);
        assert!(
            pages.get(ListPageKey::Releases).reversed,
            "a new sort keeps it"
        );
        pages.set_reversed(ListPageKey::Live, true);
        assert!(!pages.get(ListPageKey::Live).reversed);
        pages.set_reversed(ListPageKey::MyPlaylists, true);
        assert!(pages.get(ListPageKey::MyPlaylists).reversed);
    }

    /// R-LIST-7 item 7: the letters of a reversed A to Z list run from Z to
    /// A, and "#" stays last. The reverse button has its typed icon.
    #[test]
    fn adr_0043_task_002_reversed_letters_run_from_z_to_a() {
        let mut state = ListPageState {
            scope: Some(ListScope::InLibrary),
            ..ListPageState::default_for(ListPageKey::Artists)
        };
        let labels = |state| {
            ListPageHeadVm::new(ListPageKey::Artists, state, None, None)
                .letters
                .expect("A to Z shows the letter bar")
                .letters
                .into_iter()
                .map(|letter| letter.label)
                .collect::<Vec<String>>()
        };
        let usual = labels(state);
        assert_eq!((usual[0].as_str(), usual[25].as_str()), ("A", "Z"));
        state.reversed = true;
        let reversed = labels(state);
        assert_eq!(reversed.len(), 27);
        assert_eq!((reversed[0].as_str(), reversed[25].as_str()), ("Z", "A"));
        assert_eq!(reversed[26], "#");

        let usual = ListReverseVm::new(false);
        assert!(!usual.pressed);
        assert_eq!(usual.icon, ListReverseIcon::SortUsual);
        let reversed = ListReverseVm::new(true);
        assert!(reversed.pressed);
        assert_eq!(reversed.icon, ListReverseIcon::SortReversed);
        assert_eq!(reversed.label, "Reverse the Order");
    }

    /// R-LIST-7 items 9 and 10: a list with only local rows has the reverse
    /// button, after the view icons in the title row, and each list from
    /// the Index has none. A reversed state has no effect on a list from
    /// the Index: its names and its letters stay usual, and the stored
    /// state stays for the scope In Library.
    #[test]
    fn adr_0043_task_003_only_local_lists_have_the_reverse_button() {
        let local = |key, scope| ListPageState {
            reversed: true,
            scope,
            sort: Some(ListSort::AToZ),
            ..ListPageState::default_for(key)
        };
        for key in [ListPageKey::Releases, ListPageKey::Artists] {
            let head = ListPageHeadVm::new(key, local(key, Some(ListScope::InLibrary)), None, None);
            let reverse = head.reverse.expect("In Library reverses");
            assert!(reverse.pressed);
            assert_eq!(head.sort.expect("sort").label, "Z to A");
            let letters = head.letters.expect("letters");
            assert_eq!(letters.letters[0].label, "Z");

            let head = ListPageHeadVm::new(key, local(key, Some(ListScope::All)), None, None);
            assert!(head.reverse.is_none(), "{key:?} from the Index");
            assert_eq!(head.sort.expect("sort").label, "A to Z");
            assert_eq!(head.letters.expect("letters").letters[0].label, "A");
        }
        let playlists = ListPageHeadVm::titled("My playlists", None).with_reverse_of(
            ListPageKey::MyPlaylists,
            ListPageState {
                reversed: true,
                ..ListPageState::default_for(ListPageKey::MyPlaylists)
            },
        );
        assert!(playlists.reverse.is_some_and(|reverse| reverse.pressed));
        for key in [
            ListPageKey::PublicPlaylists,
            ListPageKey::Live,
            ListPageKey::SavedSearch,
        ] {
            let state = ListPageState {
                reversed: true,
                ..ListPageState::default_for(key)
            };
            assert!(
                ListPageHeadVm::new(key, state, None, None)
                    .reverse
                    .is_none(),
                "{key:?}"
            );
        }
        let state = ListPageState {
            reversed: true,
            ..ListPageState::default_for(ListPageKey::Results)
        };
        for (scope, shows) in [(ListScope::All, false), (ListScope::InLibrary, true)] {
            let head = ListPageHeadVm::for_search(
                ListPageKey::Results,
                state,
                String::new(),
                scope,
                Vec::new(),
            );
            assert_eq!(head.reverse.is_some(), shows, "{scope:?}");
            let label = head.sort.expect("sort").label;
            assert_eq!(label, if shows { "Oldest" } else { "Newest" });
        }
        let mut pages = ListPages::default();
        pages.set_reversed(ListPageKey::Releases, true);
        assert!(pages.get(ListPageKey::Releases).reversed, "the state stays");
    }

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

    /// ADR 0089 Decision 4: the title row of a search page has the scope
    /// of the query, no count, and the typed action. It has no sort, no
    /// view and no letter bar.
    #[test]
    fn adr_0089_task_008_search_head_has_scope_without_count_and_action() {
        use crate::view_models::search_results::SearchPageActionStyle;
        let action = SearchPageActionDisplay {
            id: "music-search-list-save-search",
            label: "Save search",
            a11y_label: "Save this search".to_owned(),
            available: true,
            style: SearchPageActionStyle::Plain,
        };
        let head = ListPageHeadVm::for_search(
            ListPageKey::Results,
            ListPageState::default_for(ListPageKey::Results),
            "in:library haleen".to_owned(),
            ListScope::InLibrary,
            vec![action.clone()],
        );
        assert_eq!(head.title, "in:library haleen");
        assert_eq!(head.actions, vec![action]);
        assert_eq!(head.sort.as_ref().map(|sort| sort.label), Some("Newest"));
        assert!(head.view.is_none() && head.letters.is_none());
        let scope = head.scope.expect("a search page has the scope control");
        assert_eq!(scope.selected, ListScope::InLibrary);
        assert_eq!(scope.control_id, "music-scope-search");
        assert!(scope
            .options
            .iter()
            .all(|option| option.count_text.is_none()));
        assert_eq!(scope.options[0].label, "All");
        assert_eq!(scope.options[1].label, "In Library");
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
        let artists = ListPageState::default_for(ListPageKey::Artists);
        let head = ListPageHeadVm::new(ListPageKey::Artists, artists, None, None);
        assert_eq!(head.view, None, "Artists keeps rows only");

        let releases = ListPageState::default_for(ListPageKey::Releases);
        let head = ListPageHeadVm::new(ListPageKey::Releases, releases, None, None);
        assert_eq!(head.title, "Releases");
        let sort = head.sort.expect("Releases has a sort");
        assert_eq!(sort.label, "Newest");

        let playlists = ListPageState::default_for(ListPageKey::MyPlaylists);
        let head = ListPageHeadVm::new(ListPageKey::MyPlaylists, playlists, None, None);
        assert_eq!(head.title, "My playlists");
        assert!(head.sort.is_none());
        assert!(head.scope.is_none());
        assert!(head.letters.is_none());
    }

    /// R-LIST-1 item 3 (ADR 0089 Decision 5): Releases shows two view
    /// icons, rows and tiles, and the icon of the current view is selected.
    /// The default view is rows. A view the page does not offer changes
    /// nothing, and each page keeps its own view.
    #[test]
    fn adr_0089_list_page_view_icons_mark_the_current_view_per_page() {
        let releases = ListPageState::default_for(ListPageKey::Releases);
        assert_eq!(releases.view, ListView::Rows, "the default is rows");
        let views = ListPageHeadVm::new(ListPageKey::Releases, releases, None, None)
            .view
            .expect("Releases shows view icons");
        let shown: Vec<ListView> = views.options.iter().map(|option| option.view).collect();
        assert_eq!(shown, [ListView::Rows, ListView::Tiles]);
        let selected: Vec<ListView> = views
            .options
            .iter()
            .filter(|option| option.selected)
            .map(|option| option.view)
            .collect();
        assert_eq!(selected, [ListView::Rows]);

        let mut pages = ListPages::default();
        pages.set_view(ListPageKey::Releases, ListView::Tiles);
        assert_eq!(pages.get(ListPageKey::Releases).view, ListView::Tiles);
        assert_eq!(pages.get(ListPageKey::Artists).view, ListView::Rows);
        pages.set_view(ListPageKey::Artists, ListView::Tiles);
        assert_eq!(
            pages.get(ListPageKey::Artists).view,
            ListView::Rows,
            "Artists offers rows only"
        );
        let tiles = ListPageHeadVm::new(
            ListPageKey::Releases,
            *pages.get(ListPageKey::Releases),
            None,
            None,
        )
        .view
        .expect("Releases shows view icons");
        assert!(tiles.options[1].selected, "tiles is the current view");
        assert!(!tiles.options[0].selected);
    }

    /// ADR 0089 packet 008 Part C (deviation): the search keys offer rows
    /// only, so no view icon shows on the results page or on a saved search,
    /// on any tab. The search keys offer the sorts Newest and A to Z.
    #[test]
    fn adr_0089_task_008_part_c_search_keys_show_no_view_icon() {
        for key in [ListPageKey::Results, ListPageKey::SavedSearch] {
            assert_eq!(key.views(), [ListView::Rows], "{key:?}");
            assert_eq!(key.sorts(), [ListSort::Newest, ListSort::AToZ], "{key:?}");
            let state = ListPageState::default_for(key);
            let head =
                ListPageHeadVm::for_search(key, state, String::new(), ListScope::All, Vec::new());
            assert!(head.view.is_none(), "{key:?}");
            assert_eq!(
                head.sort.as_ref().map(|sort| sort.label),
                Some("Newest"),
                "{key:?}"
            );
        }
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

    /// ADR 0089 Decision 5 and packet 009  Public playlists starts at Newest,
    /// tiles and no scope. It offers rows and tiles, and the two sorts. Its
    /// title row has no scope control and no select action.
    #[test]
    fn adr_0089_public_playlists_defaults_and_title_row() {
        let state = ListPageState::default_for(ListPageKey::PublicPlaylists);
        assert_eq!(state.sort, Some(ListSort::Newest));
        assert_eq!(state.view, ListView::Tiles);
        assert_eq!(state.scope, None);
        assert_eq!(
            ListPageKey::PublicPlaylists.views(),
            [ListView::Rows, ListView::Tiles]
        );
        assert_eq!(
            ListPageKey::PublicPlaylists.sorts(),
            [ListSort::Newest, ListSort::AToZ]
        );

        let head = ListPageHeadVm::new(ListPageKey::PublicPlaylists, state, None, None);
        assert_eq!(head.title, "Public playlists");
        assert!(head.scope.is_none());
        assert!(head.select.is_none());
        let views = head.view.expect("two views show their icons");
        let selected: Vec<ListView> = views
            .options
            .iter()
            .filter(|option| option.selected)
            .map(|option| option.view)
            .collect();
        assert_eq!(selected, [ListView::Tiles]);

        let mut pages = ListPages::default();
        pages.set_scope(ListPageKey::PublicPlaylists, ListScope::InLibrary);
        assert_eq!(pages.get(ListPageKey::PublicPlaylists).scope, None);
        pages.set_sort(ListPageKey::PublicPlaylists, ListSort::AToZ);
        assert_eq!(
            pages.get(ListPageKey::PublicPlaylists).sort,
            Some(ListSort::AToZ)
        );
        let a_to_z = ListPageHeadVm::new(
            ListPageKey::PublicPlaylists,
            *pages.get(ListPageKey::PublicPlaylists),
            None,
            None,
        );
        assert!(a_to_z.letters.is_some(), "A to Z shows the letter bar");
    }

    /// ADR 0089 packet 011: the Live page has rows only, no sort and no
    /// scope. Its title row has no scope, no select and no sort control.
    #[test]
    fn adr_0089_live_page_has_rows_and_no_controls() {
        let state = ListPageState::default_for(ListPageKey::Live);
        assert_eq!(state.sort, None);
        assert_eq!(state.view, ListView::Rows);
        assert_eq!(state.scope, None);
        assert_eq!(ListPageKey::Live.views(), [ListView::Rows]);
        assert!(ListPageKey::Live.sorts().is_empty());

        let head = ListPageHeadVm::new(ListPageKey::Live, state, None, None);
        assert_eq!(head.title, "Live");
        assert!(head.scope.is_none());
        assert!(head.sort.is_none());
        assert!(head.letters.is_none());
        assert!(head.view.is_none(), "one view shows no view icon");
    }
}
