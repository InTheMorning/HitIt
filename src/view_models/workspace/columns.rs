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

/// The operator's Music sidebar collapse choice (ADR 0084 Decision 1).
///
/// The preference lives for the session only, never in `config.toml`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum MusicSidebarPreference {
    /// Collapse under the breakpoint, else expand.
    #[default]
    Auto,
    /// Always collapsed, whatever the width.
    Collapsed,
    /// Always expanded, whatever the width.
    Expanded,
}

/// The Music sidebar's resolved show state (ADR 0084 Decision 1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MusicSidebarState {
    /// The six destinations, their labels, their counts, and the feed area.
    Expanded,
    /// Each destination as an icon only, with a tooltip.
    Collapsed,
}

impl MusicSidebarState {
    /// Toggles a resolved state to the opposite, explicit preference.
    #[must_use]
    pub(crate) const fn toggled(self) -> MusicSidebarPreference {
        match self {
            Self::Expanded => MusicSidebarPreference::Collapsed,
            Self::Collapsed => MusicSidebarPreference::Expanded,
        }
    }
}

/// Gives the Music sidebar's resolved show state (ADR 0084 Decision 1).
///
/// `width_px` is the current workspace width. `breakpoint_px` is the named
/// token `MUSIC_SIDEBAR_COLLAPSE_BREAKPOINT`. `Auto` collapses under the
/// breakpoint; `Collapsed` and `Expanded` hold at every width, so the
/// operator can change the sidebar regardless of the window size.
#[must_use]
pub(crate) fn resolve_music_sidebar_state(
    preference: MusicSidebarPreference,
    width_px: f32,
    breakpoint_px: f32,
) -> MusicSidebarState {
    match preference {
        MusicSidebarPreference::Collapsed => MusicSidebarState::Collapsed,
        MusicSidebarPreference::Expanded => MusicSidebarState::Expanded,
        MusicSidebarPreference::Auto => {
            if width_px < breakpoint_px {
                MusicSidebarState::Collapsed
            } else {
                MusicSidebarState::Expanded
            }
        }
    }
}

/// Gives the preference after a press of the collapse button (ADR 0084
/// Decision 1).
///
/// The press moves the sidebar to the opposite of its resolved state. When
/// that state is the one that the width rule gives, the preference returns
/// to `Auto`, so the sidebar follows the width again. Otherwise the press
/// sets an explicit preference that holds at each width.
#[must_use]
pub(crate) fn toggle_music_sidebar_preference(
    preference: MusicSidebarPreference,
    width_px: f32,
    breakpoint_px: f32,
) -> MusicSidebarPreference {
    let next = resolve_music_sidebar_state(preference, width_px, breakpoint_px).toggled();
    let width_rule =
        resolve_music_sidebar_state(MusicSidebarPreference::Auto, width_px, breakpoint_px);
    if resolve_music_sidebar_state(next, width_px, breakpoint_px) == width_rule {
        MusicSidebarPreference::Auto
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_music_sidebar_state, MusicSidebarPreference, MusicSidebarState};

    /// D5-05: `Auto` collapses under the breakpoint and expands at or above
    /// it. `Collapsed` and `Expanded` hold at both sides of the breakpoint.
    #[test]
    fn adr_0084_destinations_d5_05_sidebar_state_resolves_from_preference_and_width() {
        assert_eq!(
            resolve_music_sidebar_state(MusicSidebarPreference::Auto, 900.0, 1100.0),
            MusicSidebarState::Collapsed
        );
        assert_eq!(
            resolve_music_sidebar_state(MusicSidebarPreference::Auto, 1100.0, 1100.0),
            MusicSidebarState::Expanded
        );
        assert_eq!(
            resolve_music_sidebar_state(MusicSidebarPreference::Auto, 1400.0, 1100.0),
            MusicSidebarState::Expanded
        );
        assert_eq!(
            resolve_music_sidebar_state(MusicSidebarPreference::Collapsed, 1400.0, 1100.0),
            MusicSidebarState::Collapsed
        );
        assert_eq!(
            resolve_music_sidebar_state(MusicSidebarPreference::Expanded, 900.0, 1100.0),
            MusicSidebarState::Expanded
        );
    }

    /// D5-07: the collapse button's next preference is the opposite of the
    /// resolved state, so one click always moves the sidebar.
    #[test]
    fn adr_0084_destinations_d5_07_toggled_state_gives_the_opposite_preference() {
        assert_eq!(
            MusicSidebarState::Expanded.toggled(),
            MusicSidebarPreference::Collapsed
        );
        assert_eq!(
            MusicSidebarState::Collapsed.toggled(),
            MusicSidebarPreference::Expanded
        );
    }

    /// A collapse and an expand at a wide width return the preference to
    /// `Auto`, so the sidebar collapses again under the breakpoint. The
    /// operator found on 2026-10-16 that the sidebar no longer followed the
    /// width after steps 10 to 12 of the task 005 visual check.
    #[test]
    fn adr_0084_destinations_toggle_back_to_the_width_rule_restores_auto() {
        use super::toggle_music_sidebar_preference;
        let collapsed =
            toggle_music_sidebar_preference(MusicSidebarPreference::Auto, 1400.0, 1100.0);
        assert_eq!(collapsed, MusicSidebarPreference::Collapsed);
        let restored = toggle_music_sidebar_preference(collapsed, 1400.0, 1100.0);
        assert_eq!(restored, MusicSidebarPreference::Auto);
        assert_eq!(
            resolve_music_sidebar_state(restored, 900.0, 1100.0),
            MusicSidebarState::Collapsed,
            "after the restore, a narrow width collapses the sidebar again"
        );
        let expanded_narrow =
            toggle_music_sidebar_preference(MusicSidebarPreference::Auto, 900.0, 1100.0);
        assert_eq!(
            expanded_narrow,
            MusicSidebarPreference::Expanded,
            "an expand under the breakpoint holds"
        );
    }
}
