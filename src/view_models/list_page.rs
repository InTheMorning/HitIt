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

use std::collections::BTreeMap;

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
    /// The page of one singles group. All such pages share one state
    /// (musicindex ADR 0053 R-LIST-9 row "A group of singles"). It is
    /// never a destination and never in the sidebar.
    SinglesGroup,
    /// The page of one window group of New releases. It has the sorts and
    /// the views of a singles group page. All such pages share one state.
    WindowGroup,
    /// The tracks of a release page, outside an editor (musicindex ADR 0053
    /// R-LIST-9 row "The tracks of a release, outside an editor"). All
    /// release pages share one state. It has one sort, Feed Order, and it
    /// reverses locally.
    ReleaseTracks,
    /// The entries of a public playlist page, outside an editor (musicindex
    /// ADR 0053 R-LIST-9 row "The items of a playlist, outside an editor").
    /// All such pages share one state. It has one sort, Playlist Order, and
    /// it reverses locally.
    PublicPlaylistItems,
    /// One release section of a feed page or of a name level page (musicindex
    /// ADR 0053 R-LIST-9 row "The releases of an artist, a label or a name").
    /// Each section heading keeps its own state, in [`ListPages`]. It has
    /// the sorts Newest and A to Z, rows and tiles, and it reverses locally.
    /// The key holds the defaults and the rules of every section.
    ReleaseSection,
}

impl ListPageKey {
    /// The noun of the items, for example "releases" in "12 releases".
    #[must_use]
    pub(crate) const fn noun(self) -> &'static str {
        match self {
            Self::Releases | Self::ReleaseSection => "releases",
            Self::Artists => "artists",
            Self::MyPlaylists | Self::PublicPlaylists => "playlists",
            Self::Live => "live items",
            Self::Results | Self::SavedSearch => "results",
            Self::SinglesGroup | Self::WindowGroup => "singles",
            Self::ReleaseTracks => "tracks",
            Self::PublicPlaylistItems => "items",
        }
    }

    /// The title of the page, shown in the title row.
    #[must_use]
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Releases | Self::ReleaseSection => "Releases",
            Self::Artists => "Artists",
            Self::MyPlaylists => "My playlists",
            Self::PublicPlaylists => "Public playlists",
            Self::Live => "Live",
            Self::Results => "Results",
            Self::SavedSearch => "Saved search",
            Self::SinglesGroup | Self::WindowGroup => "Singles",
            Self::ReleaseTracks => "Tracks",
            Self::PublicPlaylistItems => "Items",
        }
    }

    /// The views that the page offers, in icon order (ADR 0089 Decision
    /// 5). Releases, Artists and My playlists offer rows and tiles
    /// (musicindex ADR 0053 R-LIST-9 and R-LIST-10). The search keys offer
    /// rows only, so they show no view icon. The search
    /// keys have no tiles because the Releases tiles are tied to the
    /// Library list (ADR 0089 packet 008 Part C, deviation).
    #[must_use]
    pub(crate) const fn views(self) -> &'static [ListView] {
        match self {
            Self::Releases
            | Self::Artists
            | Self::PublicPlaylists
            | Self::SinglesGroup
            | Self::WindowGroup
            | Self::MyPlaylists
            | Self::ReleaseSection => &[ListView::Rows, ListView::Tiles],
            Self::Live
            | Self::Results
            | Self::SavedSearch
            | Self::ReleaseTracks
            | Self::PublicPlaylistItems => &[ListView::Rows],
        }
    }

    /// Whether the page reverses its rows locally in `scope` (musicindex ADR
    /// 0043 R-LIST-7 items 9 and 10). The rows of a local list reverse here:
    /// Releases and Artists with the scope In Library, My playlists, and
    /// Results and Saved search with the scope In Library (musicindex ADR
    /// 0053 R-LIST-9). The Index lists reverse through the node instead
    /// (`index_reverses`), so they are not reversed here.
    #[must_use]
    pub(crate) const fn reverses_in(self, scope: ListScope) -> bool {
        match self {
            Self::Releases | Self::Artists | Self::Results | Self::SavedSearch => {
                matches!(scope, ListScope::InLibrary)
            }
            Self::MyPlaylists
            | Self::SinglesGroup
            | Self::WindowGroup
            | Self::ReleaseTracks
            | Self::PublicPlaylistItems
            | Self::ReleaseSection => true,
            Self::PublicPlaylists | Self::Live => false,
        }
    }

    /// Whether the Index list of the page reverses through the node, with
    /// `reverse=true` (musicindex ADR 0053 R-LIST-8 item 4, ADR 0043 R-LIST-7
    /// item 10). Releases, Artists and Public playlists do, when the node
    /// lists `sort`.
    #[must_use]
    pub(crate) const fn index_reverses(self, node: NodeSorts) -> bool {
        matches!(self, Self::Releases | Self::Artists | Self::PublicPlaylists) && node.sort
    }

    /// Whether the page shows the reverse button in `scope`: a local reverse
    /// (`reverses_in`), or the Index reverse in the scope All
    /// (`index_reverses`).
    #[must_use]
    pub(crate) const fn can_reverse(self, scope: ListScope, node: NodeSorts) -> bool {
        self.reverses_in(scope) || (matches!(scope, ListScope::All) && self.index_reverses(node))
    }

    /// The sorts that the page offers, in menu order. The first sort is the
    /// default (ADR 0089 Decision 5), except for a search key. The default
    /// of a search key depends on the query: Relevance when the query has a
    /// word, else Newest (musicindex ADR 0053 R-LIST-9). A page with no sort
    /// gives none.
    #[must_use]
    pub(crate) const fn sorts(self) -> &'static [ListSort] {
        match self {
            Self::Releases
            | Self::PublicPlaylists
            | Self::SinglesGroup
            | Self::WindowGroup
            | Self::ReleaseSection => &[ListSort::Newest, ListSort::AToZ],
            Self::Results | Self::SavedSearch => {
                &[ListSort::Relevance, ListSort::AToZ, ListSort::Newest]
            }
            Self::Artists => &[ListSort::AToZ, ListSort::Newest],
            Self::ReleaseTracks => &[ListSort::FeedOrder],
            Self::PublicPlaylistItems => &[ListSort::PlaylistOrder],
            // ADR 0089 amendment of 2026-11-08, musicindex ADR 0053 R-LIST-9
            // row "Playlists that a person curates".
            Self::MyPlaylists => &[ListSort::CuratorOrder, ListSort::AToZ, ListSort::Newest],
            Self::Live => &[],
        }
    }

    /// The sorts that the page offers in `scope`, in menu order (musicindex
    /// ADR 0053 R-LIST-9 item 1: a sort shows in a scope only where that
    /// scope can give it). Artists gives Newest in the scope In Library from
    /// the local dates, and in the scope All only when the node lists
    /// `artist_newest` (Stophammer ADR 0089). Another page gives
    /// [`ListPageKey::sorts`].
    #[must_use]
    pub(crate) const fn sorts_in(self, scope: ListScope, node: NodeSorts) -> &'static [ListSort] {
        match (self, scope) {
            (Self::Artists, ListScope::All) if !node.artist_newest => &[ListSort::AToZ],
            _ => self.sorts(),
        }
    }

    /// Whether a list of `rows` rows shows the sort and reverse controls of
    /// the page. The tracks of a release and the entries of a playlist
    /// show them from [`SECTION_CONTROL_MIN_ROWS`] rows, and so does a release
    /// section of a feed page (musicindex ADR 0053 R-LIST-9). Another page shows them whatever its row count is.
    #[must_use]
    pub(crate) const fn offers_controls_for(self, rows: usize) -> bool {
        match self {
            Self::ReleaseTracks | Self::PublicPlaylistItems | Self::ReleaseSection => {
                rows >= SECTION_CONTROL_MIN_ROWS
            }
            _ => true,
        }
    }
}

/// `items` in the shown order: the reverse of the source order when
/// `reversed`, else the source order. The list of a release page and of a
/// playlist page reverses here, so each row lookup, each selection and each
/// command reads the same order as the rows (musicindex ADR 0053 R-LIST-9).
#[must_use]
pub(crate) fn in_shown_order<T: Clone>(items: &[T], reversed: bool) -> Vec<T> {
    if reversed {
        items.iter().rev().cloned().collect()
    } else {
        items.to_vec()
    }
}

/// The count of rows from which a list of a release page or of a playlist
/// page shows its sort and its reverse icon (musicindex ADR 0053 R-LIST-9).
/// The operator accepted the count of 8 items. A list with fewer rows shows
/// no control and keeps its default order, whatever the stored state is.
pub(crate) const SECTION_CONTROL_MIN_ROWS: usize = 8;

/// The sort of a list page (ADR 0089 Decision 5, R-LIST-1 item 1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListSort {
    /// The order of the search run. Only the search keys offer it. It never
    /// reverses (musicindex ADR 0053 R-LIST-9).
    Relevance,
    /// The newest item first.
    Newest,
    /// The items by name, A to Z.
    AToZ,
    /// The tracks of a release in the order of the feed. Only the key
    /// `ReleaseTracks` offers it (musicindex ADR 0053 R-LIST-9).
    FeedOrder,
    /// The entries of a playlist in the order of the playlist. Only the key
    /// `PublicPlaylistItems` offers it (musicindex ADR 0053 R-LIST-9).
    PlaylistOrder,
    /// The playlists in the sequence that the curator set, by the stored
    /// position. Only the key `MyPlaylists` offers it (ADR 0089 amendment of
    /// 2026-11-08, musicindex ADR 0053 R-LIST-9).
    CuratorOrder,
}

impl ListSort {
    /// The visible name of the sort. A reversed list names its reversed
    /// order (musicindex ADR 0043 R-LIST-7 item 5).
    #[must_use]
    pub(crate) const fn name(self, reversed: bool) -> &'static str {
        match (self, reversed) {
            (Self::Relevance, _) => "Relevance",
            (Self::Newest, false) => "Newest",
            (Self::Newest, true) => "Oldest",
            (Self::AToZ, false) => "A to Z",
            (Self::AToZ, true) => "Z to A",
            (Self::FeedOrder, false) => "Feed Order",
            (Self::FeedOrder, true) => "Feed Order, Reversed",
            (Self::PlaylistOrder, false) => "Playlist Order",
            (Self::PlaylistOrder, true) => "Playlist Order, Reversed",
            (Self::CuratorOrder, false) => "Curator Order",
            (Self::CuratorOrder, true) => "Curator Order, Reversed",
        }
    }

    /// The stable element id of the sort menu entry.
    #[must_use]
    pub(crate) const fn option_id(self) -> &'static str {
        match self {
            Self::Relevance => "music-list-sort-relevance",
            Self::Newest => "music-list-sort-newest",
            Self::AToZ => "music-list-sort-a-to-z",
            Self::FeedOrder => "music-list-sort-feed-order",
            Self::PlaylistOrder => "music-list-sort-playlist-order",
            Self::CuratorOrder => "music-list-sort-curator-order",
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

/// The sort capabilities of the Index node in `GET /v1/node/capabilities`
/// (musicindex ADR 0053 R-LIST-8 item 4 and R-LIST-9). A node that lists
/// `sort` takes `reverse` on the Index lists. A node that lists
/// `artist_newest` takes `sort=date` on `/v1/artist-names` (Stophammer ADR
/// 0089). Until the read answers, a page takes [`NodeSorts::NONE`], so no
/// reverse button and no Artists Newest show in the scope All.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct NodeSorts {
    /// The node lists `sort`.
    pub(crate) sort: bool,
    /// The node lists `artist_newest`.
    pub(crate) artist_newest: bool,
}

impl NodeSorts {
    /// The node lists no sort capability, or the read has not answered.
    pub(crate) const NONE: Self = Self {
        sort: false,
        artist_newest: false,
    };
    /// The node lists `sort` only.
    #[cfg(test)]
    pub(crate) const SORT: Self = Self {
        sort: true,
        artist_newest: false,
    };

    /// The fact of the capability names of the node.
    #[must_use]
    pub(crate) fn from_capabilities(capabilities: &[String]) -> Self {
        let lists = |name: &str| capabilities.iter().any(|listed| listed == name);
        Self {
            sort: lists("sort"),
            artist_newest: lists("artist_newest"),
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
    /// Whether the Index node lists `sort` (musicindex ADR 0053 R-LIST-8
    /// item 4). The app sets it from the node capabilities.
    pub(crate) index_sorts: NodeSorts,
}

impl ListPageState {
    /// Whether the rows of the page `key` reverse locally in `scope`. A list
    /// that cannot reverse locally in `scope` is never reversed, whatever the
    /// stored state is. The sort Relevance never reverses (musicindex ADR
    /// 0053 R-LIST-9). The stored state stays for a later scope or sort.
    #[must_use]
    pub(crate) fn reversed_in(self, key: ListPageKey, scope: ListScope) -> bool {
        self.reversed && !self.is_relevance() && key.reverses_in(scope)
    }

    /// Whether a list of `rows` rows of the page `key` comes in the reversed
    /// order. A list below the count of its controls is never reversed,
    /// whatever the stored state is (musicindex ADR 0053 R-LIST-9). The
    /// stored state stays for a longer list.
    #[must_use]
    pub(crate) fn reversed_for_rows(self, key: ListPageKey, rows: usize) -> bool {
        key.offers_controls_for(rows) && self.reversed_in(key, ListScope::All)
    }

    /// The sort of the page `key` in `scope` as the list shows it. A stored
    /// sort that the scope cannot give reads as the default, A to Z
    /// (musicindex ADR 0053 R-LIST-9 item 1). The stored state stays for a
    /// later scope or node answer.
    #[must_use]
    pub(crate) fn effective_sort(self, key: ListPageKey, scope: ListScope) -> Option<ListSort> {
        let sort = self.sort?;
        if key == ListPageKey::Artists && !key.sorts_in(scope, self.index_sorts).contains(&sort) {
            return Some(ListSort::AToZ);
        }
        Some(sort)
    }

    /// The sort that the node request of the Artists list sends (musicindex
    /// ADR 0053 R-LIST-9, Stophammer ADR 0089). It is `Date` when the stored
    /// sort is Newest and the node lists `artist_newest`, in each scope, so
    /// a loaded node list keeps the order of its pages.
    #[must_use]
    pub(crate) fn artist_node_sort(self, key: ListPageKey) -> crate::api::ArtistNameSort {
        if key == ListPageKey::Artists
            && self.sort == Some(ListSort::Newest)
            && self.index_sorts.artist_newest
        {
            crate::api::ArtistNameSort::Date
        } else {
            crate::api::ArtistNameSort::Name
        }
    }

    /// Whether the sort is Relevance, which never reverses (musicindex ADR
    /// 0053 R-LIST-9).
    #[must_use]
    fn is_relevance(self) -> bool {
        self.sort == Some(ListSort::Relevance)
    }

    /// Whether the page `key` shows the reverse button in `scope`. The sort
    /// Relevance shows none (musicindex ADR 0053 R-LIST-9).
    #[must_use]
    fn shows_reverse(self, key: ListPageKey, scope: ListScope) -> bool {
        !self.is_relevance() && key.can_reverse(scope, self.index_sorts)
    }

    /// Whether the order of the page `key` shows as reversed in `scope`: the
    /// sort name, the letter bar and the reverse button. It is true for a
    /// local reverse and for an Index reverse.
    #[must_use]
    pub(crate) fn order_reversed(self, key: ListPageKey, scope: ListScope) -> bool {
        self.reversed && self.shows_reverse(key, scope)
    }

    /// Whether the Index request of the page `key` sends `reverse=true`
    /// (musicindex ADR 0053 R-LIST-8 item 4). It follows the stored state in
    /// every scope, so a loaded Index list keeps the order of its page.
    #[must_use]
    pub(crate) const fn index_reversed(self, key: ListPageKey) -> bool {
        self.reversed && key.index_reverses(self.index_sorts)
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
                index_sorts: NodeSorts::NONE,
            },
            ListPageKey::Artists => Self {
                sort: Some(ListSort::AToZ),
                view: ListView::Rows,
                scope: Some(ListScope::All),
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
            },
            // ADR 0089 amendment of 2026-11-08: Curator Order, rows and no
            // scope.
            ListPageKey::MyPlaylists => Self {
                sort: Some(ListSort::CuratorOrder),
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
            },
            // ADR 0089 Decision 5: rows, no sort and no scope.
            ListPageKey::Live => Self {
                sort: None,
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
            },
            // ADR 0089 Decision 5: Newest, tiles, and no scope.
            ListPageKey::PublicPlaylists => Self {
                sort: Some(ListSort::Newest),
                view: ListView::Tiles,
                scope: None,
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
            },
            // The query holds the scope of a search page (ADR 0089 Decision
            // 4), so the page state has none. A group page has no scope
            // either (musicindex ADR 0053 R-LIST-9 row "A group of
            // singles": Newest, rows, reverse).
            ListPageKey::Results
            | ListPageKey::SavedSearch
            | ListPageKey::SinglesGroup
            | ListPageKey::WindowGroup
            | ListPageKey::ReleaseSection => Self {
                sort: Some(ListSort::Newest),
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
            },
            // Musicindex ADR 0053 R-LIST-9: one sort that keeps the order
            // of the source, rows only, no scope.
            ListPageKey::ReleaseTracks => Self {
                sort: Some(ListSort::FeedOrder),
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
            },
            ListPageKey::PublicPlaylistItems => Self {
                sort: Some(ListSort::PlaylistOrder),
                view: ListView::Rows,
                scope: None,
                letter: None,
                reversed: false,
                index_sorts: NodeSorts::NONE,
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
    singles_group: ListPageState,
    window_group: ListPageState,
    release_tracks: ListPageState,
    public_playlist_items: ListPageState,
    /// The defaults of a release section (`ListPageKey::ReleaseSection`).
    release_section: ListPageState,
    /// The state of each release section of a feed page or of a name level
    /// page, by the text of its heading (musicindex ADR 0043 R-LIST-7 item
    /// 6, ADR 0053 R-LIST-9). The section "Albums" of each feed page shares
    /// one state. A heading with no entry has the defaults.
    release_sections: BTreeMap<String, ListPageState>,
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
            singles_group: ListPageState::default_for(ListPageKey::SinglesGroup),
            window_group: ListPageState::default_for(ListPageKey::WindowGroup),
            release_tracks: ListPageState::default_for(ListPageKey::ReleaseTracks),
            public_playlist_items: ListPageState::default_for(ListPageKey::PublicPlaylistItems),
            release_section: ListPageState::default_for(ListPageKey::ReleaseSection),
            release_sections: BTreeMap::new(),
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
            ListPageKey::SinglesGroup => &self.singles_group,
            ListPageKey::WindowGroup => &self.window_group,
            ListPageKey::ReleaseTracks => &self.release_tracks,
            ListPageKey::PublicPlaylistItems => &self.public_playlist_items,
            ListPageKey::ReleaseSection => &self.release_section,
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
            ListPageKey::SinglesGroup => &mut self.singles_group,
            ListPageKey::WindowGroup => &mut self.window_group,
            ListPageKey::ReleaseTracks => &mut self.release_tracks,
            ListPageKey::PublicPlaylistItems => &mut self.public_playlist_items,
            ListPageKey::ReleaseSection => &mut self.release_section,
        }
    }

    /// Sets the fact of the node sorts on each page (musicindex ADR 0053
    /// R-LIST-8 item 4). Only the Index lists read it.
    pub(crate) fn set_index_sorts(&mut self, node: NodeSorts) {
        for state in [
            &mut self.releases,
            &mut self.artists,
            &mut self.my_playlists,
            &mut self.public_playlists,
            &mut self.live,
            &mut self.results,
            &mut self.saved_search,
            &mut self.singles_group,
            &mut self.window_group,
            &mut self.release_tracks,
            &mut self.public_playlist_items,
            &mut self.release_section,
        ] {
            state.index_sorts = node;
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

    /// Sets the reversed state of `key`. A page with no sort keeps false.
    /// Each page keeps its own state.
    pub(crate) fn set_reversed(&mut self, key: ListPageKey, reversed: bool) {
        if key.sorts().is_empty() {
            return;
        }
        self.get_mut(key).reversed = reversed;
    }

    /// The state of the release section with the heading `heading` (musicindex
    /// ADR 0053 R-LIST-9). A heading with no stored state has the defaults.
    #[must_use]
    pub(crate) fn section_state(&self, heading: &str) -> ListPageState {
        self.release_sections
            .get(heading)
            .copied()
            .unwrap_or(self.release_section)
    }

    /// Sets the sort of the release section `heading`. A sort that a release
    /// section does not offer changes nothing. Each heading keeps its own
    /// state.
    pub(crate) fn set_section_sort(&mut self, heading: &str, sort: ListSort) {
        if !ListPageKey::ReleaseSection.sorts().contains(&sort) {
            return;
        }
        self.section_state_mut(heading).sort = Some(sort);
    }

    /// Sets the view of the release section `heading`. A view that a release
    /// section does not offer changes nothing.
    pub(crate) fn set_section_view(&mut self, heading: &str, view: ListView) {
        if !ListPageKey::ReleaseSection.views().contains(&view) {
            return;
        }
        self.section_state_mut(heading).view = view;
    }

    /// Sets the reversed state of the release section `heading`.
    pub(crate) fn set_section_reversed(&mut self, heading: &str, reversed: bool) {
        self.section_state_mut(heading).reversed = reversed;
    }

    fn section_state_mut(&mut self, heading: &str) -> &mut ListPageState {
        let default = self.release_section;
        self.release_sections
            .entry(heading.to_owned())
            .or_insert(default)
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

/// The controls at the right end of a list bar above the rows (ADR 0090
/// Decision 2, musicindex ADR 0053 R-LIST-9). The sequence is the sort menu,
/// the view icons and the reverse icon (musicindex ADR 0043 R-LIST-6). A
/// part is `None` when the page does not offer it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListBarControlsVm {
    /// The sort menu, or `None` when the page has no sort.
    pub(crate) sort: Option<ListSortButtonVm>,
    /// The view icons, or `None` when the page offers one view only.
    pub(crate) view: Option<ListViewControlVm>,
    /// The reverse icon, or `None` when the list cannot reverse.
    pub(crate) reverse: Option<ListReverseVm>,
}

impl ListBarControlsVm {
    /// The controls of the page `key` in `state`. The sort names follow the
    /// reversed order (R-LIST-7 item 5).
    #[must_use]
    pub(crate) fn new(key: ListPageKey, state: ListPageState) -> Self {
        let scope = state.scope.unwrap_or_default();
        let reversed = state.order_reversed(key, scope);
        Self {
            sort: state.effective_sort(key, scope).map(|sort| {
                sort_button(key, sort, reversed, key.sorts_in(scope, state.index_sorts))
            }),
            view: (key.views().len() > 1).then(|| view_control(key, state.view)),
            reverse: state
                .shows_reverse(key, scope)
                .then(|| ListReverseVm::new(state.reversed)),
        }
    }
}

impl ListBarControlsVm {
    /// The controls of the page `key` in `state` for a list of `rows` rows.
    /// `None` when the list is below the count of its controls (musicindex
    /// ADR 0053 R-LIST-9).
    #[must_use]
    pub(crate) fn for_rows(key: ListPageKey, state: ListPageState, rows: usize) -> Option<Self> {
        key.offers_controls_for(rows).then(|| Self::new(key, state))
    }
}

/// The heading row above the entries of a list that has no list bar (musicindex
/// ADR 0053 R-LIST-11 item 3): the count at the left, the sort menu and the
/// reverse icon at the right.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SectionHeadingVm {
    /// The count of the entries, for example "12 items".
    pub(crate) count_text: String,
    /// The sort menu and the reverse icon. It has no view icons.
    pub(crate) controls: ListBarControlsVm,
}

impl SectionHeadingVm {
    /// The heading of the page `key` in `state` for `rows` entries. `None`
    /// when the list is below the count of its controls, so the page has no
    /// heading (musicindex ADR 0053 R-LIST-9). A heading has 8 entries or
    /// more, so its noun is always plural.
    #[must_use]
    pub(crate) fn new(key: ListPageKey, state: ListPageState, rows: usize) -> Option<Self> {
        let controls = ListBarControlsVm::for_rows(key, state, rows)?;
        Some(Self {
            count_text: format!("{rows} {}", key.noun()),
            controls,
        })
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
        let reversed = state.order_reversed(key, scope);
        Self {
            title: key.title().to_owned(),
            scope: state
                .scope
                .map(|selected| scope_control(key, selected, library_count)),
            sort: state.effective_sort(key, scope).map(|sort| {
                sort_button(key, sort, reversed, key.sorts_in(scope, state.index_sorts))
            }),
            // My playlists has no letter bar: it holds local rows only, and
            // it shows no name list of the Index.
            letters: (key != ListPageKey::MyPlaylists
                && state.effective_sort(key, scope) == Some(ListSort::AToZ))
            .then(|| ordered_letters(LetterBarVm::new(key, state.letter), reversed)),
            view: (key.views().len() > 1).then(|| view_control(key, state.view)),
            reverse: state
                .shows_reverse(key, scope)
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
        let reversed = state.order_reversed(key, scope);
        Self {
            title,
            scope: Some(search_scope_control(scope)),
            sort: state.effective_sort(key, scope).map(|sort| {
                sort_button(key, sort, reversed, key.sorts_in(scope, state.index_sorts))
            }),
            letters: None,
            // Rows only: the search keys offer no view icon (ADR 0089 packet
            // 008 Part C, deviation, see `ListPageKey::views`).
            view: None,
            reverse: state
                .shows_reverse(key, scope)
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
                a11y_label: "Show all results, Library and Music Index".to_owned(),
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
        // Public playlists, Live and the group pages have no scope, so no
        // scope control uses these ids.
        ListPageKey::Live
        | ListPageKey::PublicPlaylists
        | ListPageKey::SinglesGroup
        | ListPageKey::WindowGroup
        | ListPageKey::ReleaseTracks
        | ListPageKey::PublicPlaylistItems
        | ListPageKey::ReleaseSection => (
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
                a11y_label: format!("Show all {noun}, Library and Music Index"),
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

fn sort_button(
    key: ListPageKey,
    sort: ListSort,
    reversed: bool,
    offered: &[ListSort],
) -> ListSortButtonVm {
    let id = match key {
        ListPageKey::Releases => "music-list-sort-releases",
        ListPageKey::Artists => "music-list-sort-artists",
        ListPageKey::MyPlaylists => "music-list-sort-playlists",
        ListPageKey::PublicPlaylists => "music-list-sort-public-playlists",
        ListPageKey::Live => "music-list-sort-live",
        ListPageKey::Results => "music-list-sort-results",
        ListPageKey::SavedSearch => "music-list-sort-saved-search",
        ListPageKey::SinglesGroup => "music-list-sort-singles-group",
        ListPageKey::WindowGroup => "music-list-sort-window-group",
        ListPageKey::ReleaseTracks => "music-list-sort-release-tracks",
        ListPageKey::PublicPlaylistItems => "music-list-sort-playlist-items",
        ListPageKey::ReleaseSection => "music-list-sort-release-section",
    };
    ListSortButtonVm {
        id,
        label: sort.name(reversed),
        a11y_label: format!("Sort {} by {}", key.noun(), sort.name(reversed)),
        options: offered
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

    /// ADR 0089 amendment of 2026-11-08, musicindex ADR 0053 R-LIST-9 row
    /// "Playlists that a person curates": My playlists offers Curator Order,
    /// A to Z and Newest, rows and tiles, and the reverse icon for each sort.
    /// It starts with Curator Order and Rows. The sort menu names the reversed
    /// order of each sort.
    #[test]
    fn adr_0053_r9_curated_my_playlists_sorts_default_views_and_reverse() {
        let key = ListPageKey::MyPlaylists;
        assert_eq!(
            key.sorts(),
            [ListSort::CuratorOrder, ListSort::AToZ, ListSort::Newest]
        );
        assert_eq!(key.views(), [ListView::Rows, ListView::Tiles]);
        let state = ListPageState::default_for(key);
        assert_eq!(state.sort, Some(ListSort::CuratorOrder));
        assert_eq!(state.view, ListView::Rows);
        assert_eq!(state.scope, None);
        assert!(!state.reversed);
        assert_eq!(
            ListSort::CuratorOrder.option_id(),
            "music-list-sort-curator-order"
        );
        let head = ListPageHeadVm::new(key, state, None, None);
        assert_eq!(head.sort.as_ref().expect("sort").label, "Curator Order");
        assert_eq!(
            head.view.expect("view").options.len(),
            2,
            "rows and tiles icons"
        );
        assert!(head.letters.is_none(), "no letter bar on a local list");
        let expected = [
            (
                ListSort::CuratorOrder,
                "Curator Order",
                "Curator Order, Reversed",
            ),
            (ListSort::AToZ, "A to Z", "Z to A"),
            (ListSort::Newest, "Newest", "Oldest"),
        ];
        let mut pages = ListPages::default();
        for (sort, usual, reversed_name) in expected {
            pages.set_sort(key, sort);
            for (reversed, name) in [(false, usual), (true, reversed_name)] {
                pages.set_reversed(key, reversed);
                let head = ListPageHeadVm::new(key, *pages.get(key), None, None);
                assert_eq!(head.sort.as_ref().expect("sort").label, name);
                let reverse = head.reverse.expect("reverse icon for each sort");
                assert_eq!(reverse.pressed, reversed);
                assert!(head
                    .sort
                    .expect("sort")
                    .options
                    .iter()
                    .all(|option| { option.label == option.sort.name(reversed) }));
            }
        }
        // No other list offers Curator Order.
        for other in [
            ListPageKey::Releases,
            ListPageKey::Artists,
            ListPageKey::PublicPlaylists,
            ListPageKey::Live,
            ListPageKey::Results,
            ListPageKey::SavedSearch,
            ListPageKey::SinglesGroup,
            ListPageKey::WindowGroup,
            ListPageKey::ReleaseTracks,
            ListPageKey::PublicPlaylistItems,
            ListPageKey::ReleaseSection,
        ] {
            assert!(
                !other.sorts().contains(&ListSort::CuratorOrder),
                "{other:?}"
            );
        }
    }

    /// Musicindex ADR 0053 R-LIST-9 row "A group of singles": the two group
    /// keys offer Newest and A to Z, rows and tiles, and reverse. They start
    /// with Newest and Rows. Each kind keeps its own state.
    #[test]
    fn adr_0053_r9_group_keys_offer_sort_view_and_reverse() {
        let mut pages = ListPages::default();
        for key in [ListPageKey::SinglesGroup, ListPageKey::WindowGroup] {
            assert_eq!(key.sorts(), [ListSort::Newest, ListSort::AToZ]);
            assert_eq!(key.views(), [ListView::Rows, ListView::Tiles]);
            assert!(key.reverses_in(ListScope::All));
            assert!(key.reverses_in(ListScope::InLibrary));
            let state = ListPageState::default_for(key);
            assert_eq!(state.sort, Some(ListSort::Newest));
            assert_eq!(state.view, ListView::Rows);
            assert_eq!(state.scope, None);
            assert!(!state.reversed);
        }
        pages.set_sort(ListPageKey::SinglesGroup, ListSort::AToZ);
        pages.set_view(ListPageKey::SinglesGroup, ListView::Tiles);
        pages.set_reversed(ListPageKey::SinglesGroup, true);
        let window = pages.get(ListPageKey::WindowGroup);
        assert_eq!(window.sort, Some(ListSort::Newest));
        assert_eq!(window.view, ListView::Rows);
        assert!(!window.reversed);
        assert!(pages.get(ListPageKey::SinglesGroup).reversed);
    }

    /// The controls of a group list bar hold the sort menu, the view icons
    /// and the reverse icon. A reversed list names "Oldest" and "Z to A".
    #[test]
    fn adr_0053_r9_group_bar_controls_name_the_reversed_order() {
        let key = ListPageKey::SinglesGroup;
        let usual = ListBarControlsVm::new(key, ListPageState::default_for(key));
        let sort = usual.sort.expect("a group has a sort");
        assert_eq!(sort.label, "Newest");
        let labels: Vec<&str> = sort.options.iter().map(|option| option.label).collect();
        assert_eq!(labels, ["Newest", "A to Z"]);
        assert_eq!(usual.view.expect("two views").options.len(), 2);
        assert!(!usual.reverse.expect("a group reverses").pressed);

        let state = ListPageState {
            reversed: true,
            sort: Some(ListSort::AToZ),
            ..ListPageState::default_for(key)
        };
        let reversed = ListBarControlsVm::new(key, state);
        let sort = reversed.sort.expect("a group has a sort");
        assert_eq!(sort.label, "Z to A");
        let labels: Vec<&str> = sort.options.iter().map(|option| option.label).collect();
        assert_eq!(labels, ["Oldest", "Z to A"]);
        assert!(reversed.reverse.expect("a group reverses").pressed);
    }

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
        let playlists = ListPageHeadVm::new(
            ListPageKey::MyPlaylists,
            ListPageState {
                reversed: true,
                ..ListPageState::default_for(ListPageKey::MyPlaylists)
            },
            None,
            None,
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
        assert_eq!(playlists.sort, Some(ListSort::CuratorOrder));
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
    /// icon shows on a page with one view, and My playlists has its sort
    /// button and its view icons, and no scope.
    #[test]
    fn adr_0089_list_page_head_shows_current_sort_and_no_view_icon() {
        let playlists = ListPageState::default_for(ListPageKey::MyPlaylists);
        let head = ListPageHeadVm::new(ListPageKey::MyPlaylists, playlists, None, None);
        assert!(head.view.is_some(), "My playlists offers rows and tiles");
        assert_eq!(head.scope, None);
        let live = ListPageState::default_for(ListPageKey::Live);
        let head = ListPageHeadVm::new(ListPageKey::Live, live, None, None);
        assert_eq!(head.view, None, "Live keeps rows only");

        let releases = ListPageState::default_for(ListPageKey::Releases);
        let head = ListPageHeadVm::new(ListPageKey::Releases, releases, None, None);
        assert_eq!(head.title, "Releases");
        let sort = head.sort.expect("Releases has a sort");
        assert_eq!(sort.label, "Newest");

        let playlists = ListPageState::default_for(ListPageKey::MyPlaylists);
        let head = ListPageHeadVm::new(ListPageKey::MyPlaylists, playlists, None, None);
        assert_eq!(head.title, "My playlists");
        assert_eq!(head.sort.expect("sort").label, "Curator Order");
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
        pages.set_view(ListPageKey::MyPlaylists, ListView::Tiles);
        assert_eq!(
            pages.get(ListPageKey::MyPlaylists).view,
            ListView::Tiles,
            "My playlists offers rows and tiles"
        );
        pages.set_view(ListPageKey::Live, ListView::Tiles);
        assert_eq!(
            pages.get(ListPageKey::Live).view,
            ListView::Rows,
            "Live offers rows only"
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
    /// on any tab. The search keys offer Relevance, A to Z and Newest
    /// (musicindex ADR 0053 R-LIST-9).
    #[test]
    fn adr_0089_task_008_part_c_search_keys_show_no_view_icon() {
        for key in [ListPageKey::Results, ListPageKey::SavedSearch] {
            assert_eq!(key.views(), [ListView::Rows], "{key:?}");
            assert_eq!(
                key.sorts(),
                [ListSort::Relevance, ListSort::AToZ, ListSort::Newest],
                "{key:?}"
            );
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

        pages.set_sort(ListPageKey::Artists, ListSort::Relevance);
        assert_eq!(
            pages.get(ListPageKey::Artists).sort,
            Some(ListSort::AToZ),
            "Artists does not offer Relevance"
        );
        pages.set_sort(ListPageKey::MyPlaylists, ListSort::AToZ);
        assert_eq!(
            pages.get(ListPageKey::MyPlaylists).sort,
            Some(ListSort::AToZ)
        );
        pages.set_sort(ListPageKey::MyPlaylists, ListSort::Relevance);
        assert_eq!(
            pages.get(ListPageKey::MyPlaylists).sort,
            Some(ListSort::AToZ),
            "My playlists does not offer Relevance"
        );
        pages.set_sort(ListPageKey::Live, ListSort::AToZ);
        assert_eq!(pages.get(ListPageKey::Live).sort, None);
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

    /// musicindex ADR 0053 R-LIST-8 item 4 and ADR 0043 R-LIST-7 item 10: an
    /// Index list shows the reverse button only when the node lists `sort`.
    /// The button, the sort name and the letters then follow the reversed
    /// state, as for a local list.
    #[test]
    fn adr_0053_r8_index_lists_reverse_when_the_node_lists_sort() {
        let index_state = |key, node, reversed| ListPageState {
            reversed,
            index_sorts: node,
            scope: Some(ListScope::All),
            ..ListPageState::default_for(key)
        };
        for key in [
            ListPageKey::Releases,
            ListPageKey::Artists,
            ListPageKey::PublicPlaylists,
        ] {
            let unlisted =
                ListPageHeadVm::new(key, index_state(key, NodeSorts::NONE, true), None, None);
            assert!(unlisted.reverse.is_none(), "{key:?} without sort");
            assert!(
                unlisted
                    .sort
                    .as_ref()
                    .is_some_and(|sort| sort.label != "Oldest" && sort.label != "Z to A"),
                "{key:?} without sort keeps the usual name"
            );

            let listed =
                ListPageHeadVm::new(key, index_state(key, NodeSorts::SORT, false), None, None);
            let reverse = listed.reverse.expect("the node lists sort");
            assert!(!reverse.pressed, "{key:?} usual order");

            let listed =
                ListPageHeadVm::new(key, index_state(key, NodeSorts::SORT, true), None, None);
            assert!(listed.reverse.is_some_and(|reverse| reverse.pressed));
            let sort = listed.sort.expect("sort");
            assert!(
                sort.label == "Oldest" || sort.label == "Z to A",
                "{key:?} names its reversed order, got {}",
                sort.label
            );
        }
        let artists = ListPageHeadVm::new(
            ListPageKey::Artists,
            index_state(ListPageKey::Artists, NodeSorts::SORT, true),
            None,
            None,
        );
        assert_eq!(artists.letters.expect("letters").letters[0].label, "Z");
    }

    /// musicindex ADR 0053 R-LIST-8 item 4: the Index lists that the node
    /// cannot reverse, and the search and playlist pages, keep no reverse
    /// button, even when the node lists `sort`.
    #[test]
    fn adr_0053_r8_only_the_index_lists_reverse_through_the_node() {
        assert!(ListPageKey::Releases.index_reverses(NodeSorts::SORT));
        assert!(ListPageKey::Artists.index_reverses(NodeSorts::SORT));
        assert!(ListPageKey::PublicPlaylists.index_reverses(NodeSorts::SORT));
        assert!(!ListPageKey::Releases.index_reverses(NodeSorts::NONE));
        for key in [
            ListPageKey::Live,
            ListPageKey::SavedSearch,
            ListPageKey::Results,
            ListPageKey::MyPlaylists,
        ] {
            assert!(!key.index_reverses(NodeSorts::SORT), "{key:?}");
        }
        for key in [ListPageKey::Live, ListPageKey::SavedSearch] {
            let state = ListPageState {
                reversed: true,
                index_sorts: NodeSorts::SORT,
                ..ListPageState::default_for(key)
            };
            assert!(
                ListPageHeadVm::new(key, state, None, None)
                    .reverse
                    .is_none(),
                "{key:?}"
            );
        }
        let results = ListPageState {
            reversed: true,
            index_sorts: NodeSorts::SORT,
            ..ListPageState::default_for(ListPageKey::Results)
        };
        let head = ListPageHeadVm::for_search(
            ListPageKey::Results,
            results,
            String::new(),
            ListScope::All,
            Vec::new(),
        );
        assert!(
            head.reverse.is_none(),
            "a search page in All has no node reverse"
        );
    }

    /// musicindex ADR 0053 R-LIST-8 item 4: the request reverses only in
    /// the Index lists, and it follows the stored state in every scope. The
    /// local reverse of an In Library list does not reverse the Index rows.
    #[test]
    fn adr_0053_r8_index_request_reverses_by_state_and_local_rows_stay_usual() {
        let listed = ListPageState {
            reversed: true,
            index_sorts: NodeSorts::SORT,
            scope: Some(ListScope::All),
            ..ListPageState::default_for(ListPageKey::Releases)
        };
        assert!(listed.index_reversed(ListPageKey::Releases));
        assert!(listed.order_reversed(ListPageKey::Releases, ListScope::All));
        assert!(
            !listed.reversed_in(ListPageKey::Releases, ListScope::All),
            "the Index rows are not reversed a second time"
        );
        let in_library = ListPageState {
            scope: Some(ListScope::InLibrary),
            ..listed
        };
        assert!(
            in_library.index_reversed(ListPageKey::Releases),
            "the request follows the state"
        );
        assert!(in_library.reversed_in(ListPageKey::Releases, ListScope::InLibrary));

        let unlisted = ListPageState {
            index_sorts: NodeSorts::NONE,
            ..listed
        };
        assert!(!unlisted.index_reversed(ListPageKey::Releases));
        assert!(!unlisted.order_reversed(ListPageKey::Releases, ListScope::All));
        assert!(!ListPageState::default_for(ListPageKey::Live).index_reversed(ListPageKey::Live));
    }

    /// musicindex ADR 0053 R-LIST-8 item 4: each list keeps its own reversed
    /// state, and the node fact reaches each list page.
    #[test]
    fn adr_0053_r8_each_index_list_keeps_its_own_reversed_state() {
        let mut pages = ListPages::default();
        pages.set_index_sorts(NodeSorts::SORT);
        pages.set_reversed(ListPageKey::Releases, true);
        assert!(pages
            .get(ListPageKey::Releases)
            .index_reversed(ListPageKey::Releases));
        assert!(!pages
            .get(ListPageKey::Artists)
            .index_reversed(ListPageKey::Artists));
        assert!(!pages
            .get(ListPageKey::PublicPlaylists)
            .index_reversed(ListPageKey::PublicPlaylists));
        pages.set_reversed(ListPageKey::PublicPlaylists, true);
        pages.set_reversed(ListPageKey::Releases, false);
        assert!(pages
            .get(ListPageKey::PublicPlaylists)
            .index_reversed(ListPageKey::PublicPlaylists));
        assert!(!pages
            .get(ListPageKey::Releases)
            .index_reversed(ListPageKey::Releases));

        pages.set_index_sorts(NodeSorts::NONE);
        assert!(!pages
            .get(ListPageKey::PublicPlaylists)
            .order_reversed(ListPageKey::PublicPlaylists, ListScope::All));
    }

    /// Musicindex ADR 0053 R-LIST-9 row "Artists, Labels" item 1: Artists
    /// offers A to Z and Newest in the scope In Library, and in the scope All
    /// only when the node lists `artist_newest`. The default stays A to Z.
    #[test]
    fn adr_0053_r9_artists_newest_sorts_by_scope_and_node() {
        let key = ListPageKey::Artists;
        let both = [ListSort::AToZ, ListSort::Newest];
        assert_eq!(key.sorts_in(ListScope::InLibrary, NodeSorts::NONE), both);
        assert_eq!(key.sorts_in(ListScope::InLibrary, NodeSorts::SORT), both);
        assert_eq!(
            key.sorts_in(ListScope::All, NodeSorts::NONE),
            [ListSort::AToZ]
        );
        assert_eq!(
            key.sorts_in(ListScope::All, NodeSorts::SORT),
            [ListSort::AToZ]
        );
        let newest_node = NodeSorts {
            sort: true,
            artist_newest: true,
        };
        assert_eq!(key.sorts_in(ListScope::All, newest_node), both);
        assert_eq!(
            ListPageState::default_for(key).sort,
            Some(ListSort::AToZ),
            "the default stays A to Z"
        );
        // Another page offers the same sorts in each scope.
        assert_eq!(
            ListPageKey::Releases.sorts_in(ListScope::All, NodeSorts::NONE),
            ListPageKey::Releases.sorts()
        );
    }

    /// A stored Newest in the scope All reads as A to Z while the node does
    /// not list `artist_newest`: the menu, the letter bar and the node
    /// request. With the capability, Newest shows and the letter bar goes.
    #[test]
    fn adr_0053_r9_artists_newest_stored_sort_reads_as_a_to_z_in_all_without_the_capability() {
        let key = ListPageKey::Artists;
        let mut pages = ListPages::default();
        pages.set_sort(key, ListSort::Newest);
        let state = *pages.get(key);
        assert_eq!(state.sort, Some(ListSort::Newest), "the state is stored");

        let head = ListPageHeadVm::new(key, state, None, None);
        let sort = head.sort.expect("a sort menu");
        assert_eq!(sort.label, "A to Z");
        assert_eq!(sort.options.len(), 1);
        assert!(head.letters.is_some(), "A to Z has the letter bar");
        assert_eq!(
            state.effective_sort(key, ListScope::All),
            Some(ListSort::AToZ)
        );
        assert_eq!(
            state.artist_node_sort(key),
            crate::api::ArtistNameSort::Name
        );
        assert_eq!(
            state.effective_sort(key, ListScope::InLibrary),
            Some(ListSort::Newest)
        );

        pages.set_index_sorts(NodeSorts {
            sort: true,
            artist_newest: true,
        });
        pages.set_reversed(key, true);
        let state = *pages.get(key);
        let head = ListPageHeadVm::new(key, state, None, None);
        let sort = head.sort.expect("a sort menu");
        assert_eq!(sort.label, "Oldest", "a reversed Newest names Oldest");
        assert_eq!(sort.options.len(), 2);
        assert!(head.letters.is_none(), "Newest has no letter bar");
        assert!(head.reverse.is_some(), "the reverse icon keeps working");
        assert_eq!(
            state.artist_node_sort(key),
            crate::api::ArtistNameSort::Date
        );
        assert!(state.index_reversed(key));

        // The scope In Library gives Newest from the local dates, with no
        // node capability, and its node request stays by name.
        let mut local = ListPages::default();
        local.set_sort(key, ListSort::Newest);
        local.set_scope(key, ListScope::InLibrary);
        let head = ListPageHeadVm::new(key, *local.get(key), None, None);
        assert_eq!(head.sort.expect("a sort menu").label, "Newest");
        assert!(head.letters.is_none());
    }

    /// Stophammer ADR 0089: the node lists `artist_newest` apart from
    /// `sort`. `NodeSorts` records each capability, with and without it.
    #[test]
    fn adr_0053_r9_artists_newest_node_sorts_read_artist_newest() {
        let names = |values: &[&str]| -> Vec<String> {
            values.iter().map(|value| (*value).to_owned()).collect()
        };
        let both = NodeSorts::from_capabilities(&names(&["search", "sort", "artist_newest"]));
        assert!(both.sort && both.artist_newest);
        let sort_only = NodeSorts::from_capabilities(&names(&["search", "sort"]));
        assert_eq!(sort_only, NodeSorts::SORT);
        assert!(!sort_only.artist_newest);
        assert_eq!(NodeSorts::from_capabilities(&[]), NodeSorts::NONE);
    }

    /// musicindex ADR 0053 R-LIST-8 item 4: the node lists `sort` when its
    /// capability names contain `sort`. A node that does not list it, or an
    /// empty list, gives no reverse.
    #[test]
    fn adr_0053_r8_node_sorts_reads_the_capability_names() {
        let names = |values: &[&str]| -> Vec<String> {
            values.iter().map(|value| (*value).to_owned()).collect()
        };
        assert_eq!(
            NodeSorts::from_capabilities(&names(&["search", "sort"])),
            NodeSorts::SORT
        );
        assert_eq!(
            NodeSorts::from_capabilities(&names(&["search"])),
            NodeSorts::NONE
        );
        assert_eq!(NodeSorts::from_capabilities(&[]), NodeSorts::NONE);
        assert_eq!(NodeSorts::default(), NodeSorts::NONE);
    }

    /// Musicindex ADR 0053 R-LIST-9: the search keys offer Relevance, A to
    /// Z and Newest, in that menu order, and the menu has an entry for each.
    #[test]
    fn adr_0053_r9_search_keys_offer_relevance_a_to_z_newest() {
        for key in [ListPageKey::Results, ListPageKey::SavedSearch] {
            assert_eq!(
                key.sorts(),
                [ListSort::Relevance, ListSort::AToZ, ListSort::Newest],
                "{key:?}"
            );
            let mut state = ListPageState::default_for(key);
            state.sort = Some(ListSort::Relevance);
            let head = ListPageHeadVm::for_search(
                key,
                state,
                String::new(),
                ListScope::InLibrary,
                Vec::new(),
            );
            let button = head.sort.expect("a search page has a sort button");
            assert_eq!(button.label, "Relevance");
            let ids: Vec<&str> = button.options.iter().map(|option| option.id).collect();
            assert_eq!(
                ids,
                [
                    "music-list-sort-relevance",
                    "music-list-sort-a-to-z",
                    "music-list-sort-newest"
                ]
            );
            assert!(button.options[0].selected);
            assert!(button.options[0].a11y_label.ends_with(", current sort"));
        }
    }

    /// Musicindex ADR 0053 R-LIST-9: Relevance never reverses. No reverse
    /// button shows, and the stored reversed state is ignored.
    #[test]
    fn adr_0053_r9_relevance_gives_no_reverse() {
        for key in [ListPageKey::Results, ListPageKey::SavedSearch] {
            let mut state = ListPageState::default_for(key);
            state.sort = Some(ListSort::Relevance);
            state.reversed = true;
            assert!(!state.reversed_in(key, ListScope::InLibrary), "{key:?}");
            assert!(!state.order_reversed(key, ListScope::InLibrary), "{key:?}");
            let head = ListPageHeadVm::for_search(
                key,
                state,
                String::new(),
                ListScope::InLibrary,
                Vec::new(),
            );
            assert!(head.reverse.is_none(), "{key:?}");
            assert_eq!(head.sort.expect("sort").label, "Relevance");
        }
    }

    /// Musicindex ADR 0053 R-LIST-9: the search keys reverse in In Library
    /// on A to Z and Newest, and never in All.
    #[test]
    fn adr_0053_r9_search_keys_reverse_in_library_only() {
        for key in [ListPageKey::Results, ListPageKey::SavedSearch] {
            for sort in [ListSort::AToZ, ListSort::Newest] {
                let mut state = ListPageState::default_for(key);
                state.sort = Some(sort);
                state.reversed = true;
                assert!(state.reversed_in(key, ListScope::InLibrary), "{key:?}");
                assert!(!state.reversed_in(key, ListScope::All), "{key:?}");
                let library = ListPageHeadVm::for_search(
                    key,
                    state,
                    String::new(),
                    ListScope::InLibrary,
                    Vec::new(),
                );
                assert!(library.reverse.is_some(), "{key:?}");
                let all = ListPageHeadVm::for_search(
                    key,
                    state,
                    String::new(),
                    ListScope::All,
                    Vec::new(),
                );
                assert!(all.reverse.is_none(), "{key:?}");
            }
        }
    }

    /// Musicindex ADR 0053 R-LIST-9 rows "The tracks of a release, outside an
    /// editor" and "The items of a playlist, outside an editor": each key
    /// offers its one sort, starts with it, has no view icons, and reverses.
    /// The reversed names follow R-LIST-7 item 5.
    #[test]
    fn adr_0053_r9_order_keys_offer_one_sort_and_reverse() {
        let cases = [
            (
                ListPageKey::ReleaseTracks,
                ListSort::FeedOrder,
                "Feed Order",
                "Feed Order, Reversed",
                "music-list-sort-feed-order",
            ),
            (
                ListPageKey::PublicPlaylistItems,
                ListSort::PlaylistOrder,
                "Playlist Order",
                "Playlist Order, Reversed",
                "music-list-sort-playlist-order",
            ),
        ];
        for (key, sort, name, reversed_name, option_id) in cases {
            assert_eq!(key.sorts(), [sort]);
            assert_eq!(key.views(), [ListView::Rows]);
            assert!(key.reverses_in(ListScope::All));
            let state = ListPageState::default_for(key);
            assert_eq!(state.sort, Some(sort));
            assert_eq!(state.scope, None);
            assert_eq!(sort.name(false), name);
            assert_eq!(sort.name(true), reversed_name);
            assert_eq!(sort.option_id(), option_id);

            let usual = ListBarControlsVm::new(key, state);
            assert_eq!(usual.sort.expect("one sort").label, name);
            assert!(usual.view.is_none(), "no view icons");
            assert!(!usual.reverse.expect("reverses").pressed);

            let reversed = ListBarControlsVm::new(
                key,
                ListPageState {
                    reversed: true,
                    ..state
                },
            );
            let sort_button = reversed.sort.expect("one sort");
            assert_eq!(sort_button.label, reversed_name);
            assert_eq!(sort_button.options.len(), 1);
            assert!(reversed.reverse.expect("reverses").pressed);
        }
    }

    /// Each kind keeps one state: a change of the release tracks does not
    /// change the entries of a playlist.
    #[test]
    fn adr_0053_r9_order_keys_keep_their_own_state() {
        let mut pages = ListPages::default();
        pages.set_reversed(ListPageKey::ReleaseTracks, true);
        assert!(pages.get(ListPageKey::ReleaseTracks).reversed);
        assert!(!pages.get(ListPageKey::PublicPlaylistItems).reversed);
    }

    /// A list below `SECTION_CONTROL_MIN_ROWS` rows has no control and is
    /// never reversed. The stored state stays.
    #[test]
    fn adr_0053_r9_order_controls_start_at_the_count_of_eight() {
        assert_eq!(SECTION_CONTROL_MIN_ROWS, 8);
        for key in [ListPageKey::ReleaseTracks, ListPageKey::PublicPlaylistItems] {
            let state = ListPageState {
                reversed: true,
                ..ListPageState::default_for(key)
            };
            assert!(ListBarControlsVm::for_rows(key, state, 7).is_none());
            assert!(!state.reversed_for_rows(key, 7));
            assert!(ListBarControlsVm::for_rows(key, state, 8).is_some());
            assert!(state.reversed_for_rows(key, 8));
            assert!(state.reversed, "the stored state stays");
        }
        // A group page shows its controls whatever its count is.
        assert!(ListBarControlsVm::for_rows(
            ListPageKey::SinglesGroup,
            ListPageState::default_for(ListPageKey::SinglesGroup),
            1
        )
        .is_some());
    }

    /// The heading names the count, and "1 item" for one entry.
    #[test]
    fn adr_0053_r9_order_heading_names_the_count() {
        let key = ListPageKey::PublicPlaylistItems;
        let state = ListPageState::default_for(key);
        let heading = SectionHeadingVm::new(key, state, 12).expect("12 entries have a heading");
        assert_eq!(heading.count_text, "12 items");
        assert!(SectionHeadingVm::new(key, state, 7).is_none());
        assert_eq!(in_shown_order(&[1, 2, 3], true), [3, 2, 1]);
        assert_eq!(in_shown_order(&[1, 2, 3], false), [1, 2, 3]);
    }

    /// musicindex ADR 0053 R-LIST-10: Artists offers rows and tiles in
    /// both scopes. The default is rows, and the head has the view icons.
    #[test]
    fn adr_0053_r10_artists_offers_rows_and_tiles_in_both_scopes() {
        assert_eq!(
            ListPageKey::Artists.views(),
            [ListView::Rows, ListView::Tiles]
        );
        for scope in [ListScope::All, ListScope::InLibrary] {
            let state = ListPageState {
                scope: Some(scope),
                ..ListPageState::default_for(ListPageKey::Artists)
            };
            assert_eq!(state.view, ListView::Rows, "the default is rows");
            let views = ListPageHeadVm::new(ListPageKey::Artists, state, None, None)
                .view
                .expect("Artists shows view icons");
            let shown: Vec<ListView> = views.options.iter().map(|option| option.view).collect();
            assert_eq!(shown, [ListView::Rows, ListView::Tiles]);
        }
        let mut pages = ListPages::default();
        pages.set_view(ListPageKey::Artists, ListView::Tiles);
        assert_eq!(pages.get(ListPageKey::Artists).view, ListView::Tiles);
    }
}
