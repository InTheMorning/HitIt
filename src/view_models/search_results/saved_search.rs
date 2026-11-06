//! Saved searches (ADR 0084 Decision 3, packet 012).
//!
//! A saved search is a view of the Search page.
//! Its page runs its query against the Library and the Index, and shows
//! the results in the detail column with the view model of packet 010.
//! This module gives the types, the labels, the typed actions, the delete
//! confirmation and the stale-result rule of the saved-search page.
//! ADR 0089 Decision 4 adds the display name of a saved search and the
//! query field of the page (see `saved_search_query`).

#![warn(clippy::pedantic)]

use crate::db::saved_searches::{StoredSavedSearch, StoredStartTab};
use crate::view_models::workspace::{FrameNavigationEntry, MusicDestination};

use super::saved_search_query::SavedSearchQueryEdit;
use super::{EmptyStateDisplay, SearchResultsInspectorPageVm, SearchResultsTab};

/// The identity of one saved search: its `saved_searches` row id.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct SavedSearchId(pub(crate) i64);

/// One saved search, as the Search page shows it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearch {
    pub(crate) id: SavedSearchId,
    /// The trimmed query text.
    pub(crate) query: String,
    /// The name that the operator gave, or `None` to show the query.
    pub(crate) name: Option<String>,
}

impl SavedSearch {
    /// The frame title of the saved-search page.
    pub(crate) const FRAME_TITLE: &'static str = "Saved search";

    /// The subtitle of a saved-search row of the Search page. Each saved search
    /// runs against the Library and the Index (ADR 0084 Decision 3).
    pub(crate) const SUBTITLE: &'static str = "Library and Index";

    /// A saved search from its stored row.
    #[must_use]
    pub(crate) fn from_stored(stored: StoredSavedSearch) -> Self {
        Self {
            id: SavedSearchId(stored.id),
            query: stored.query,
            name: stored.name,
        }
    }

    /// The text of the row: the name when there is one, else the query
    /// (ADR 0089 Decision 4). The query stays visible on the page.
    #[must_use]
    pub(crate) fn display_label(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.query)
    }

    /// The navigation entry of this saved search.
    #[must_use]
    pub(crate) fn entry(&self) -> FrameNavigationEntry {
        FrameNavigationEntry::SavedSearch {
            id: self.id,
            query: self.query.clone(),
        }
    }
}

/// The label of a saved search, in the Search page and in the toolbar title, for
/// example "Saved: bluegrass".
#[must_use]
pub(crate) fn saved_search_label(query: &str) -> String {
    format!("Saved: {query}")
}

/// The stored start tab of a new saved search. The column keeps its three
/// values (ADR 0089 Decision 4): a saved search opens on All, so the stored
/// value of All and of Labels is the value that the column held before.
impl From<SearchResultsTab> for StoredStartTab {
    fn from(tab: SearchResultsTab) -> Self {
        match tab {
            SearchResultsTab::All | SearchResultsTab::Releases => Self::Releases,
            SearchResultsTab::Artists | SearchResultsTab::Labels => Self::Artists,
            SearchResultsTab::Tracks => Self::Tracks,
        }
    }
}

/// Tells if `query` is saved. The compare ignores the case of ASCII
/// letters, as the `NOCASE` index of `saved_searches` does.
#[must_use]
pub(crate) fn is_query_saved(saved: &[SavedSearch], query: &str) -> bool {
    let query = query.trim();
    saved
        .iter()
        .any(|search| search.query.eq_ignore_ascii_case(query))
}

/// The control style of a search page action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SearchPageActionStyle {
    /// A plain button: "Save search".
    Plain,
    /// A destructive button: "Delete…".
    Destructive,
}

/// One typed action of the header of a search page (packet 012). The
/// view model decides its text and its availability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SearchPageActionDisplay {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) a11y_label: String,
    pub(crate) available: bool,
    pub(crate) style: SearchPageActionStyle,
}

/// "Save search" of the toolbar search, or "Saved" when the query is
/// already saved.
#[must_use]
pub(crate) fn save_search_action(query: &str, saved: &[SavedSearch]) -> SearchPageActionDisplay {
    if is_query_saved(saved, query) {
        SearchPageActionDisplay {
            id: "music-search-list-save-search",
            label: "Saved",
            a11y_label: "This search is saved in Search".to_owned(),
            available: false,
            style: SearchPageActionStyle::Plain,
        }
    } else {
        SearchPageActionDisplay {
            id: "music-search-list-save-search",
            label: "Save search",
            a11y_label: format!("Save the search \"{}\" in Search", query.trim()),
            available: true,
            style: SearchPageActionStyle::Plain,
        }
    }
}

/// "Delete…" of the saved-search page.
#[must_use]
pub(crate) fn delete_saved_search_action(query: &str) -> SearchPageActionDisplay {
    SearchPageActionDisplay {
        id: "music-saved-search-delete",
        label: "Delete\u{2026}",
        a11y_label: format!("Delete the saved search \"{query}\""),
        available: true,
        style: SearchPageActionStyle::Destructive,
    }
}

/// The text of the delete confirmation of a saved search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearchDeleteConfirmation {
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) cancel_button_id: &'static str,
    pub(crate) cancel_label: &'static str,
    pub(crate) cancel_a11y_label: &'static str,
    pub(crate) confirm_button_id: &'static str,
    pub(crate) confirm_label: &'static str,
    pub(crate) confirm_a11y_label: String,
}

/// The delete confirmation of a saved search with the query `query`.
#[must_use]
pub(crate) fn delete_saved_search_confirmation(query: &str) -> SavedSearchDeleteConfirmation {
    SavedSearchDeleteConfirmation {
        title: "Delete saved search?".to_owned(),
        message: format!(
            "Delete the saved search \"{query}\"? The Library and the downloads do not change."
        ),
        cancel_button_id: "music-saved-search-delete-cancel",
        cancel_label: "Cancel",
        cancel_a11y_label: "Keep the saved search",
        confirm_button_id: "music-saved-search-delete-confirm",
        confirm_label: "Delete",
        confirm_a11y_label: format!("Delete the saved search \"{query}\""),
    }
}

/// The location after the delete of a saved search: Search, as a
/// sidebar click opens it, with "New releases" (ADR 0084 Decision 9).
#[must_use]
pub(crate) const fn location_after_saved_search_delete() -> FrameNavigationEntry {
    FrameNavigationEntry::Destination(MusicDestination::Search)
}

/// The state of the mounted saved-search page.
#[derive(Debug)]
pub(crate) enum SavedSearchPageState {
    /// The saved search exists. The page holds its results.
    Open(Box<SearchResultsInspectorPageVm>),
    /// The saved search was deleted. The page shows an empty state and
    /// runs no query.
    Deleted,
}

/// The mounted saved-search page of the detail column.
///
/// `query` is the stored text. It changes only when a keep saves a new text.
/// While `edit` is open, the results show the live query of the field.
#[derive(Debug)]
pub(crate) struct SavedSearchPage {
    pub(crate) id: SavedSearchId,
    pub(crate) query: String,
    pub(crate) state: SavedSearchPageState,
    /// The open query field, or `None` when the page shows its stored text.
    pub(crate) edit: Option<SavedSearchQueryEdit>,
}

impl SavedSearchPage {
    /// The title of the empty state of a deleted saved search.
    pub(crate) const DELETED_TITLE: &'static str = "Saved search deleted";

    /// The text of the empty state of a deleted saved search.
    pub(crate) const DELETED_TEXT: &'static str = "This saved search was deleted.";

    /// A page with the results of `search`: the Library rows of `results`
    /// and the tab of the save.
    #[must_use]
    pub(crate) fn open(search: &SavedSearch, results: SearchResultsInspectorPageVm) -> Self {
        Self {
            id: search.id,
            query: search.query.clone(),
            state: SavedSearchPageState::Open(Box::new(results.into_saved_search(search))),
            edit: None,
        }
    }

    /// The page of a deleted saved search.
    #[must_use]
    pub(crate) fn deleted(id: SavedSearchId, query: impl Into<String>) -> Self {
        Self {
            id,
            query: query.into(),
            state: SavedSearchPageState::Deleted,
            edit: None,
        }
    }

    /// The query whose results the page shows: the live query of the open
    /// field, or the stored text when no field is open (ADR 0087 Decision 4).
    #[must_use]
    pub(crate) fn live_query(&self) -> &str {
        self.edit
            .as_ref()
            .map_or(self.query.as_str(), SavedSearchQueryEdit::live_query)
    }

    /// Tells if the header shows the query as its headline. The header shows
    /// the query, or the query field, never both (ADR 0089 Decision 4).
    #[must_use]
    pub(crate) fn header_shows_query(&self) -> bool {
        self.edit.is_none()
    }

    /// Opens the query field over the open page. Gives `false` for a deleted
    /// page, or when the field is open already.
    pub(crate) fn begin_query_edit(&mut self) -> bool {
        if self.edit.is_some() || self.results().is_none() {
            return false;
        }
        self.edit = Some(SavedSearchQueryEdit::open(self.id, &self.query));
        true
    }

    /// Replaces the results of the open page, for example with the live run
    /// of the field. A deleted page keeps its state.
    pub(crate) fn set_results(&mut self, results: SearchResultsInspectorPageVm) {
        let entry = FrameNavigationEntry::SavedSearch {
            id: self.id,
            query: self.query.clone(),
        };
        if let SavedSearchPageState::Open(current) = &mut self.state {
            let tab = current.tab();
            let results = results.with_list_state_of(Some(&**current));
            **current = results.into_saved_search_with_tab(entry, tab);
        }
    }

    /// The empty state of a deleted saved search.
    #[must_use]
    pub(crate) fn deleted_empty_state() -> EmptyStateDisplay {
        EmptyStateDisplay::new(Self::DELETED_TITLE, Self::DELETED_TEXT)
    }

    /// The empty state while the page of the current saved search is not
    /// mounted yet.
    #[must_use]
    pub(crate) fn loading_empty_state() -> EmptyStateDisplay {
        EmptyStateDisplay::new(
            SavedSearch::FRAME_TITLE,
            "The app is loading the results of the saved search.",
        )
    }

    /// The results of the page, or `None` for a deleted saved search.
    #[must_use]
    pub(crate) fn results(&self) -> Option<&SearchResultsInspectorPageVm> {
        match &self.state {
            SavedSearchPageState::Open(results) => Some(results),
            SavedSearchPageState::Deleted => None,
        }
    }

    /// The mutable results of the page.
    pub(crate) fn results_mut(&mut self) -> Option<&mut SearchResultsInspectorPageVm> {
        match &mut self.state {
            SavedSearchPageState::Open(results) => Some(results),
            SavedSearchPageState::Deleted => None,
        }
    }

    /// The header action of the page: "Delete…", or `None` for a deleted
    /// saved search.
    #[must_use]
    pub(crate) fn delete_action(&self) -> Option<SearchPageActionDisplay> {
        self.results()
            .map(|_| delete_saved_search_action(&self.query))
    }
}

/// What the restore of a saved-search entry does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchRestore {
    /// The mounted page is the page of this entry. It keeps its rows and
    /// its selection, and no query runs.
    Keep,
    /// A new page runs the query of this saved search: the Library rows at
    /// once, then the Index request.
    RunQuery(SavedSearch),
    /// The saved search was deleted. The page shows the empty state, and
    /// no query runs.
    Deleted,
}

/// Decides the restore of the entry `SavedSearch { id, query }`. A mounted
/// page with the same id and the same query keeps its rows. Any other case
/// runs the query again: the Library rows at once, then the Index request.
#[must_use]
pub(crate) fn saved_search_restore(
    mounted: Option<&SavedSearchPage>,
    id: SavedSearchId,
    query: &str,
    saved: &[SavedSearch],
) -> SavedSearchRestore {
    let Some(search) = saved
        .iter()
        .find(|search| search.id == id && search.query == query)
    else {
        return SavedSearchRestore::Deleted;
    };
    let same_page = mounted
        .is_some_and(|page| page.id == id && page.query == query && page.results().is_some());
    if same_page {
        SavedSearchRestore::Keep
    } else {
        SavedSearchRestore::RunQuery(search.clone())
    }
}

/// Tells if an Index result for the saved search `id` with the query
/// `query` can write the mounted page. The mounted page must hold that
/// same id and that same query. A move to saved search B replaces the
/// mounted page with the page of B, so a stale result for A then does not
/// match. A move away to "New releases" or to a toolbar search does not
/// touch the mounted page, so a result that arrives after that move still
/// writes the page of the saved search it belongs to.
#[must_use]
pub(crate) fn saved_search_result_is_current(
    mounted: Option<&SavedSearchPage>,
    id: SavedSearchId,
    query: &str,
) -> bool {
    mounted
        .is_some_and(|page| page.id == id && page.live_query() == query && page.results().is_some())
}

/// The text that shows when a saved search no longer exists (ADR 0089
/// Decision 4).
pub(crate) const SAVED_SEARCH_MISSING_TEXT: &str = "This saved search no longer exists.";

/// The text that shows when a copy has the text of a saved search that
/// exists already (ADR 0089 Decision 4).
pub(crate) const SAVED_SEARCH_COPY_CONFLICT_TEXT: &str =
    "A saved search with this text already exists. Rename it first.";

/// One item of the item menu of a saved search (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchMenuItem {
    /// "Edit Query": opens the saved search page.
    EditQuery,
    /// "Rename…": the name of the row becomes an input.
    Rename,
    /// "Duplicate": a new row with " copy" after the name.
    Duplicate,
    /// "Delete Saved Search…": the delete confirmation.
    Delete,
}

/// One typed item of the item menu of a saved search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearchMenuEntry {
    pub(crate) item: SavedSearchMenuItem,
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) a11y_label: String,
    pub(crate) available: bool,
    pub(crate) destructive: bool,
}

/// The item menu of a saved search, in the sequence of ADR 0089 Decision 4:
/// Edit Query, Rename, Duplicate and Delete Saved Search. Each item is
/// available, and only the delete item is destructive.
#[must_use]
pub(crate) fn saved_search_menu(search: &SavedSearch) -> [SavedSearchMenuEntry; 4] {
    let name = search.display_label();
    [
        SavedSearchMenuEntry {
            item: SavedSearchMenuItem::EditQuery,
            id: "music-saved-search-menu-edit-query",
            label: "Edit Query",
            a11y_label: format!("Edit the query of the saved search \"{name}\""),
            available: true,
            destructive: false,
        },
        SavedSearchMenuEntry {
            item: SavedSearchMenuItem::Rename,
            id: "music-saved-search-menu-rename",
            label: "Rename\u{2026}",
            a11y_label: format!("Rename the saved search \"{name}\""),
            available: true,
            destructive: false,
        },
        SavedSearchMenuEntry {
            item: SavedSearchMenuItem::Duplicate,
            id: "music-saved-search-menu-duplicate",
            label: "Duplicate",
            a11y_label: format!("Duplicate the saved search \"{name}\""),
            available: true,
            destructive: false,
        },
        SavedSearchMenuEntry {
            item: SavedSearchMenuItem::Delete,
            id: "music-saved-search-menu-delete",
            label: "Delete Saved Search\u{2026}",
            a11y_label: format!("Delete the saved search \"{name}\""),
            available: true,
            destructive: true,
        },
    ]
}

/// The inline rename of a saved search row (ADR 0089 Decision 4).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearchRenameDisplay {
    pub(crate) input_placeholder: &'static str,
    pub(crate) save_id: &'static str,
    pub(crate) save_label: &'static str,
    pub(crate) save_a11y_label: String,
    pub(crate) cancel_id: &'static str,
    pub(crate) cancel_label: &'static str,
    pub(crate) cancel_a11y_label: String,
}

/// The rename controls of a saved search row.
#[must_use]
pub(crate) fn saved_search_rename_display(search: &SavedSearch) -> SavedSearchRenameDisplay {
    let name = search.display_label();
    SavedSearchRenameDisplay {
        input_placeholder: "Saved search name",
        save_id: "music-saved-search-rename-save",
        save_label: "Save",
        save_a11y_label: format!("Save the name of the saved search \"{name}\""),
        cancel_id: "music-saved-search-rename-cancel",
        cancel_label: "Cancel",
        cancel_a11y_label: format!("Cancel the rename of the saved search \"{name}\""),
    }
}

/// "New Saved Search" of the Search page (ADR 0089 Decision 4). It is a
/// button, and it is always available.
#[must_use]
pub(crate) fn new_saved_search_action() -> SearchPageActionDisplay {
    SearchPageActionDisplay {
        id: "music-search-new-saved-search",
        label: "New Saved Search",
        a11y_label: "New Saved Search. Puts the focus in the search field.".to_owned(),
        available: true,
        style: SearchPageActionStyle::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        delete_saved_search_confirmation, location_after_saved_search_delete, save_search_action,
        saved_search_restore, saved_search_result_is_current, SavedSearch, SavedSearchId,
        SavedSearchPage, SavedSearchRestore,
    };
    use crate::view_models::search_results::tests::{index_release, index_track, RecordedLibrary};
    use crate::view_models::search_results::{
        ArtistSearchFacts, IndexSearchResultRows, SearchResultsInspectorPageVm, SearchResultsTab,
    };
    use crate::view_models::selection_action_bar::SelectionActionKind;
    use crate::view_models::workspace::{
        FrameNavigationEntry, FrameNavigationState, MusicDestination,
    };
    fn saved(id: i64, query: &str) -> SavedSearch {
        SavedSearch {
            id: SavedSearchId(id),
            query: query.to_owned(),
            name: None,
        }
    }

    fn open_page(search: &SavedSearch) -> SavedSearchPage {
        SavedSearchPage::open(
            search,
            SearchResultsInspectorPageVm::from_local_library_tracks(
                &search.query,
                &[],
                &ArtistSearchFacts::default(),
            ),
        )
    }

    /// R12-03 (ADR 0084 Decision 3): "Save search" is available for a new
    /// query. A saved query, also with another case, gives "Saved", not
    /// available, with its accessibility label.
    #[test]
    fn adr_0084_saved_search_r12_03_save_action_follows_the_saved_list() {
        let list = [saved(1, "bluegrass")];

        let new = save_search_action("ambient", &list);
        assert_eq!(new.label, "Save search");
        assert!(new.available);
        assert_eq!(new.a11y_label, "Save the search \"ambient\" in Search");

        let present = save_search_action("Bluegrass", &list);
        assert_eq!(present.label, "Saved");
        assert!(!present.available);
        assert_eq!(present.a11y_label, "This search is saved in Search");

        let vm = SearchResultsInspectorPageVm::new("bluegrass");
        assert_eq!(vm.save_action(&list).label, "Saved");
        assert_eq!(vm.save_action(&[]).label, "Save search");
    }

    /// R12-06 (ADR 0084 Decision 3, ADR 0089 Decision 4): the page opens on
    /// the tab All, also when the Index rows arrive first.
    #[test]
    fn adr_0084_saved_search_r12_06_page_opens_on_all() {
        let search = saved(4, "bluegrass");
        let mut page = open_page(&search);
        let results = page.results_mut().expect("an open page has results");
        assert_eq!(results.tab(), SearchResultsTab::All);
        results.mark_index_loading();

        results.replace_index_results(IndexSearchResultRows {
            feeds: vec![index_release(2_000_000_000, "g-index", "Index Release")],
            ..IndexSearchResultRows::default()
        });

        assert_eq!(
            results.tab(),
            SearchResultsTab::All,
            "the Releases rows do not change the tab All"
        );
        assert_eq!(
            results.selection().entry(),
            Some(&search.entry()),
            "the selection belongs to the saved-search entry"
        );
    }

    /// R12-07 (ADR 0084 Decision 3): after a move from saved search A to
    /// saved search B, an Index result for A does not change the page of
    /// B. The move runs the query of B, which replaces the mounted page of
    /// A with the page of B, so the stale result for A no longer matches.
    #[test]
    fn adr_0084_saved_search_r12_07_stale_result_changes_nothing() {
        let a = saved(1, "bluegrass");
        let b = saved(2, "ambient");

        let page_a = open_page(&a);
        assert!(saved_search_result_is_current(
            Some(&page_a),
            a.id,
            &a.query
        ));

        // A move from A to B replaces the mounted page.
        let page_b = open_page(&b);
        assert!(
            !saved_search_result_is_current(Some(&page_b), a.id, &a.query),
            "the mounted page is now B, so a late reply for A does not match"
        );
        assert!(saved_search_result_is_current(
            Some(&page_b),
            b.id,
            &b.query
        ));
    }

    /// R12-09 (ADR 0084 Decision 3): the entry of a deleted saved search
    /// gives the empty state, and the restore starts no query.
    #[test]
    fn adr_0084_saved_search_r12_09_deleted_entry_runs_no_query() {
        let a = saved(1, "bluegrass");
        let mounted = open_page(&a);

        assert_eq!(
            saved_search_restore(Some(&mounted), a.id, &a.query, &[]),
            SavedSearchRestore::Deleted
        );
        let deleted = SavedSearchPage::deleted(a.id, &a.query);
        assert!(deleted.results().is_none());
        assert!(deleted.delete_action().is_none());
        assert_eq!(
            SavedSearchPage::DELETED_TEXT,
            "This saved search was deleted."
        );

        // A mounted page of the same id and the same query keeps its
        // rows. With no mounted page, the restore runs the query.
        let list = [a.clone()];
        assert_eq!(
            saved_search_restore(Some(&mounted), a.id, &a.query, &list),
            SavedSearchRestore::Keep
        );
        assert_eq!(
            saved_search_restore(None, a.id, &a.query, &list),
            SavedSearchRestore::RunQuery(a)
        );
    }

    /// R12-10 (ADR 0084 Decision 3): the saved-search page gives the
    /// header action and the bar of packet 010 on its Releases tab and its
    /// Tracks tab. The header action has the id of the detail column.
    #[test]
    fn adr_0084_saved_search_r12_10_header_action_and_bar() {
        let search = saved(5, "q");
        let mut page = open_page(&search);
        let results = page.results_mut().unwrap();
        results.set_tab(SearchResultsTab::Releases);
        results.replace_index_results(IndexSearchResultRows {
            feeds: vec![index_release(2_000_000_000, "g-index", "Index Release")],
            tracks: vec![index_track(
                3_000_000_000,
                Some("g-index"),
                "t1",
                "Index Track",
            )],
            ..IndexSearchResultRows::default()
        });
        let lookup = RecordedLibrary::default();

        let action = results.header_action(&lookup).expect("Releases has Select");
        assert_eq!(action.label, "Select");
        assert_eq!(action.id, "music-detail-column-select-releases");
        results.toggle_mode();
        results.select_all(&lookup);
        let bar = results.action_bar(&lookup).expect("Selecting shows a bar");
        assert_eq!(
            bar.action(SelectionActionKind::Download)
                .map(|action| action.label.clone())
                .as_deref(),
            Some("Download 1 release")
        );

        results.set_tab(SearchResultsTab::Tracks);
        let action = results.header_action(&lookup).expect("Tracks has Select");
        assert_eq!(action.id, "music-detail-column-select-tracks");
        results.toggle_mode();
        results.select_all(&lookup);
        let bar = results.action_bar(&lookup).expect("Selecting shows a bar");
        assert_eq!(
            bar.action(SelectionActionKind::Download)
                .map(|action| action.label.clone())
                .as_deref(),
            Some("Download 1 track")
        );
        assert_eq!(
            results.selection().entry(),
            Some(&search.entry()),
            "a tab change keeps the selection on the saved-search entry"
        );
    }

    /// Review correction (2026-10-21): a late Index reply for the open
    /// saved search A must not be dropped after the current location path
    /// no longer holds the entry of A. The page then stayed "loading",
    /// because Back gave `Keep` with no rows to show.
    ///
    /// A reply for A still matches while the mounted page holds the id and
    /// the query of A. A move to the Releases destination does not touch
    /// the mounted saved-search page, so the reply writes it, and Back
    /// shows the rows, not a stuck loading state.
    #[test]
    fn adr_0084_saved_search_late_reply_after_leaving_the_path_is_not_stuck_loading() {
        let a = saved(1, "bluegrass");
        let mut page = open_page(&a);
        page.results_mut()
            .expect("an open page has results")
            .mark_index_loading();

        // Leave to the Releases destination, a path that holds no entry
        // for A, before the reply arrives. The production code calls no
        // saved-search function for this move, so the mounted page stays
        // the page of A.
        let mut nav = FrameNavigationState::new_with_page(
            FrameNavigationEntry::Destination(MusicDestination::Search),
            a.entry(),
        );
        nav.start_path(
            FrameNavigationEntry::Destination(MusicDestination::Releases),
            None,
        );
        assert!(
            !nav.path_entries().contains(&a.entry()),
            "the path no longer holds the entry of A"
        );

        // The late reply for A still matches, because the mounted page is
        // still the page of A.
        assert!(saved_search_result_is_current(Some(&page), a.id, &a.query));
        let wrote = page
            .results_mut()
            .expect("an open page has results")
            .replace_index_results_for_query(
                &a.query,
                IndexSearchResultRows {
                    feeds: vec![index_release(2_000_000_000, "g-index", "Index Release")],
                    ..IndexSearchResultRows::default()
                },
            );
        assert!(wrote, "the reply writes the page of A");

        // Back restores A: Keep, because the mounted page already holds
        // the rows of A.
        assert_eq!(
            saved_search_restore(Some(&page), a.id, &a.query, std::slice::from_ref(&a)),
            SavedSearchRestore::Keep
        );
        let results = page.results().expect("an open page has results");
        assert!(!results.is_index_loading(), "the page is not stuck loading");
    }

    /// R12-08 (ADR 0084 Decision 3): the confirmation names the query and
    /// gives "Delete". After the delete, the location is Search with
    /// "New releases".
    #[test]
    fn adr_0084_saved_search_r12_08_delete_confirmation_and_location() {
        let confirmation = delete_saved_search_confirmation("bluegrass");
        assert_eq!(
            confirmation.message,
            "Delete the saved search \"bluegrass\"? The Library and the downloads do not change."
        );
        assert_eq!(confirmation.confirm_label, "Delete");

        let search = saved(3, "bluegrass");
        let action = open_page(&search)
            .delete_action()
            .expect("an open page gives Delete");
        assert_eq!(action.label, "Delete\u{2026}");
        assert!(action.available);

        let entry = location_after_saved_search_delete();
        let FrameNavigationEntry::Destination(_) = &entry else {
            panic!("the delete opens a destination");
        };
        let mut nav = FrameNavigationState::new_with_page(
            FrameNavigationEntry::Destination(MusicDestination::Search),
            search.entry(),
        );
        nav.start_path(entry.clone(), None);
        assert_eq!(
            nav.path_entries(),
            vec![FrameNavigationEntry::Destination(MusicDestination::Search)]
        );
    }

    /// ADR 0089 Decision 4: a row shows its name when it has one, and else
    /// its query.
    #[test]
    fn adr_0089_saved_search_row_shows_the_name_or_the_query() {
        let mut search = saved(3, "bluegrass");
        assert_eq!(search.display_label(), "bluegrass");
        search.name = Some("Old time".to_owned());
        assert_eq!(search.display_label(), "Old time");
        assert_eq!(search.query, "bluegrass", "the query stays stored");
    }

    /// ADR 0089 Decision 4: the item menu has the four items in the sequence
    /// Edit Query, Rename, Duplicate and Delete Saved Search. Each item is
    /// available, and only the delete item is destructive.
    #[test]
    fn adr_0089_saved_search_menu_has_the_four_items_in_order() {
        let search = saved(3, "bluegrass");
        let menu = super::saved_search_menu(&search);
        let labels: Vec<&str> = menu.iter().map(|entry| entry.label).collect();
        assert_eq!(
            labels,
            [
                "Edit Query",
                "Rename\u{2026}",
                "Duplicate",
                "Delete Saved Search\u{2026}"
            ]
        );
        let items: Vec<super::SavedSearchMenuItem> = menu.iter().map(|entry| entry.item).collect();
        assert_eq!(
            items,
            [
                super::SavedSearchMenuItem::EditQuery,
                super::SavedSearchMenuItem::Rename,
                super::SavedSearchMenuItem::Duplicate,
                super::SavedSearchMenuItem::Delete,
            ]
        );
        assert!(menu.iter().all(|entry| entry.available));
        let destructive: Vec<bool> = menu.iter().map(|entry| entry.destructive).collect();
        assert_eq!(destructive, [false, false, false, true]);
        assert_eq!(menu[3].a11y_label, "Delete the saved search \"bluegrass\"");
    }

    /// ADR 0089 Decision 4: the rename controls and the new-search button
    /// carry their labels and accessibility labels from the view model.
    #[test]
    fn adr_0089_saved_search_rename_and_new_button_are_typed() {
        let mut search = saved(3, "bluegrass");
        search.name = Some("Old time".to_owned());
        let rename = super::saved_search_rename_display(&search);
        assert_eq!(rename.save_label, "Save");
        assert_eq!(
            rename.cancel_a11y_label,
            "Cancel the rename of the saved search \"Old time\""
        );
        let new = super::new_saved_search_action();
        assert_eq!(new.label, "New Saved Search");
        assert!(new.available);
        assert_eq!(new.id, "music-search-new-saved-search");
    }

    /// ADR 0089 Decision 4: the query field opens over the page with the
    /// stored text, once. The live query is the typed text, and the stored
    /// text stays. A result is current for the typed text only while the
    /// field is open (ADR 0087 Decision 4).
    #[test]
    fn adr_0089_saved_search_query_edit_live_query_and_current_results() {
        let a = saved(1, "bluegrass");
        let mut page = open_page(&a);
        assert!(page.begin_query_edit());
        assert!(!page.begin_query_edit(), "a second open changes nothing");
        assert_eq!(page.live_query(), "bluegrass");

        page.edit
            .as_mut()
            .expect("the field is open")
            .set_typed("bluegrass banjo");
        assert_eq!(page.live_query(), "bluegrass banjo");
        assert_eq!(page.query, "bluegrass", "the stored text stays");
        assert!(saved_search_result_is_current(
            Some(&page),
            a.id,
            "bluegrass banjo"
        ));
        assert!(
            !saved_search_result_is_current(Some(&page), a.id, "bluegrass"),
            "a result of the stored text is stale while the field shows another text"
        );

        page.edit = None;
        assert_eq!(page.live_query(), "bluegrass");
        assert!(saved_search_result_is_current(
            Some(&page),
            a.id,
            "bluegrass"
        ));
    }

    /// ADR 0089 Decision 4: a live run replaces the results of the page with
    /// the typed text. The page keeps the tab that the curator chose and the
    /// saved-search entry.
    #[test]
    fn adr_0089_saved_search_query_edit_live_results_keep_the_tab() {
        let a = saved(1, "bluegrass");
        let mut page = open_page(&a);
        page.results_mut()
            .expect("an open page has results")
            .set_tab(SearchResultsTab::Tracks);
        assert!(page.begin_query_edit());
        page.set_results(SearchResultsInspectorPageVm::from_local_library_tracks(
            "bluegrass banjo",
            &[],
            &ArtistSearchFacts::default(),
        ));
        let results = page.results().expect("an open page has results");
        assert_eq!(results.query(), "bluegrass banjo");
        assert_eq!(results.tab(), SearchResultsTab::Tracks);
        assert_eq!(results.selection().entry(), Some(&a.entry()));
    }

    /// ADR 0089 Decision 4: a deleted saved search opens no query field.
    #[test]
    fn adr_0089_saved_search_query_edit_deleted_page_opens_no_field() {
        let mut page = SavedSearchPage::deleted(SavedSearchId(1), "bluegrass");
        assert!(!page.begin_query_edit());
        assert!(page.edit.is_none());
        assert_eq!(page.live_query(), "bluegrass");
    }

    /// ADR 0089 Decision 4 (review of 2026-11-06): the header shows the query,
    /// or the query field, never both. The page gives that fact.
    #[test]
    fn adr_0089_saved_search_query_edit_header_shows_the_query_or_the_field() {
        let a = saved(1, "bluegrass");
        let mut page = open_page(&a);
        assert!(page.header_shows_query());
        assert!(page.begin_query_edit());
        assert!(
            !page.header_shows_query(),
            "the field replaces the headline"
        );
        page.edit = None;
        assert!(page.header_shows_query());
    }
}
