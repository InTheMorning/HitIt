//! Music page visibility (ADR 0089 Decision 1 and Invariants).
//!
//! Music shows the sidebar and one page at the remaining width. A list root
//! (a destination or a search) fills the list frame. A page fills the detail
//! frame, and it opens over its list. This module holds that rule as a pure
//! function of the navigation entry. It carries no renderer type and no
//! width, so it does not import `gpui`.

#![warn(clippy::pedantic)]

use super::nav::FrameNavigationEntry;

/// Whether the content of a Music page is ready (ADR 0089 Decision 5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MusicPageContent {
    /// The page has its data, or it renders its own state (an Index page).
    Loaded,
    /// The page has no data yet, and the Library tree is loading. The page
    /// shows the shared loading state.
    Loading,
    /// The page has no data and nothing loads it. The list root shows.
    Absent,
}

/// Gives the page content state of a Music page entry (ADR 0089 Decision 5).
///
/// `own_content` is `true` for a page that renders its own state, such as an
/// Index release. `detail_shows` is `true` when the Library detail holds the
/// page. `library_loading` is `true` while the Library tree reloads.
#[must_use]
pub(crate) const fn music_page_content(
    own_content: bool,
    detail_shows: bool,
    library_loading: bool,
) -> MusicPageContent {
    if own_content || detail_shows {
        MusicPageContent::Loaded
    } else if library_loading {
        MusicPageContent::Loading
    } else {
        MusicPageContent::Absent
    }
}

/// The one Music frame that shows for a navigation entry (ADR 0089).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MusicColumns {
    /// The current entry is a list root, or a page with no content. The list
    /// frame shows.
    ListRoot,
    /// The current entry is a page with content, or it is loading. The
    /// detail frame shows, and the list frame does not.
    Page,
}

impl MusicColumns {
    /// Gives the visible Music frame for the current navigation entry and
    /// its page content state (ADR 0089 Decisions 1 and 5).
    #[must_use]
    pub(crate) const fn for_entry(entry: &FrameNavigationEntry, content: MusicPageContent) -> Self {
        match content {
            MusicPageContent::Absent => Self::ListRoot,
            MusicPageContent::Loaded | MusicPageContent::Loading => {
                if entry.opens_detail_page() {
                    Self::Page
                } else {
                    Self::ListRoot
                }
            }
        }
    }

    /// Returns whether the list frame shows.
    #[must_use]
    pub(crate) const fn shows_list(self) -> bool {
        matches!(self, Self::ListRoot)
    }

    /// Returns whether the detail frame shows.
    #[must_use]
    pub(crate) const fn shows_detail(self) -> bool {
        matches!(self, Self::Page)
    }
}

/// The keyboard target of the Music surface (ADR 0089 Decision 1 and
/// Invariants). The app gives it the window focus when it changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MusicFocusTarget {
    /// The Library list column of a list root.
    ListColumn,
    /// The Library detail column of a page.
    DetailColumn,
    /// The search results list of a search root.
    SearchResults,
    /// The saved-search page, which holds its own focus.
    SavedSearchPage,
}

/// Returns whether a new focus target takes the keyboard focus from the
/// toolbar search field (ADR 0087 Decision 4). A live run changes the list
/// root while the curator types, so a list root does not take the focus. A
/// page that opens does.
#[must_use]
pub(crate) const fn music_focus_moves(
    target: MusicFocusTarget,
    search_field_focused: bool,
) -> bool {
    !search_field_focused
        || matches!(
            target,
            MusicFocusTarget::DetailColumn | MusicFocusTarget::SavedSearchPage
        )
}

/// Gives the keyboard target for the visible Music frame (ADR 0089).
///
/// `search_results_open` is `true` when the location starts from a search
/// with results. A page keeps the focus on its own column, so a row click or
/// Back moves the focus to the frame that shows.
#[must_use]
pub(crate) const fn music_focus_target(
    columns: MusicColumns,
    entry: &FrameNavigationEntry,
    search_results_open: bool,
) -> MusicFocusTarget {
    match columns {
        MusicColumns::Page => {
            if matches!(entry, FrameNavigationEntry::SavedSearch { .. }) {
                MusicFocusTarget::SavedSearchPage
            } else {
                MusicFocusTarget::DetailColumn
            }
        }
        MusicColumns::ListRoot => {
            if search_results_open {
                MusicFocusTarget::SearchResults
            } else {
                MusicFocusTarget::ListColumn
            }
        }
    }
}
