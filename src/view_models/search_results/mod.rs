//! Search-results view models of the Music list column.
//!
//! ADR 0084 Decision 2: the toolbar search fills the list column, and a
//! result opens in the detail column. Packet 010 gives the view three tabs,
//! one row for each item with the state word of each Library item, and the
//! selection model of packets 018 and 019 on the Releases tab and the
//! Tracks tab. Each tab keeps its rows in one ADR 0041 `PagedListVm`
//! window.
#![warn(clippy::pedantic)]

use crate::view_models::artist_groups::ArtistGroup;
use crate::view_models::detail_selection::{
    detail_header_action, search_header_action, search_release_rows, search_track_rows,
    DetailPageRows, DetailRowClick, DetailRowKey, DetailSelection,
};
use crate::view_models::list_page::{ListPageKey, ListPageState, ListPages, ListScope, ListSort};
use crate::view_models::list_selection::{FocusStep, SelectionRow};
use crate::view_models::live_run::RunGeneration;
use musicindex_search::field_list::Library;
use musicindex_search::note::{scope_note, search_note, SearchNote};

use crate::view_models::search_query::{library_scope_of, SearchQuery};
use crate::view_models::selection_action_bar::SelectionActionBarVm;
use crate::view_models::window_groups::WindowGroupTileVm;
use crate::view_models::workspace::{FrameHeaderActionDisplay, FrameNavigationEntry};

mod empty_state;
mod failure;
mod index_detail;
mod index_result_note;
mod new_results;
mod paged_tab;
mod results;
mod rows;
mod saved_search;
mod saved_search_groups;
mod saved_search_query;
mod search_run;
mod sort;
mod tabs;
#[cfg(test)]
mod tests;

pub(crate) use empty_state::EmptyStateDisplay;
pub(crate) use failure::{SearchFailureAction, SearchFailureAvailability};
pub(crate) use index_detail::{IndexDetailDisplay, IndexDetailKind, IndexSearchResultRows};
pub(crate) use index_result_note::{IndexResultDiagnostic, IndexResultStopReason};
#[cfg(test)]
pub(crate) use musicindex_search::plan::IndexSkip;
pub(crate) use new_results::{new_result_count, new_results_text, NewResultCount, NewResultsText};
pub(crate) use paged_tab::SearchResultsPagedTab;
pub(crate) use results::{
    artist_feed_result_display, index_track_row_id, ArtistResultDisplay, ArtistSearchFacts,
    FeedResultDisplay, SearchResultTarget, TrackResultDisplay, INDEX_FEED_PAGE_RESULT_PREFIX,
};
pub(crate) use rows::{
    ReleaseKindPlacement, SearchLibraryLookup, SearchLibraryRelease, SearchLibraryTrack,
    SearchReleaseWindowFacts, SearchResultRowVm, SearchRowBadge,
};
pub(crate) use saved_search::{
    delete_saved_search_confirmation, is_query_saved, location_after_saved_search_delete,
    new_saved_search_action, save_search_action, saved_search_label, saved_search_menu,
    saved_search_rename_display, saved_search_restore, saved_search_result_is_current, SavedSearch,
    SavedSearchDeleteConfirmation, SavedSearchHeaderAction, SavedSearchId, SavedSearchMenuEntry,
    SavedSearchMenuItem, SavedSearchPage, SavedSearchRenameDisplay, SavedSearchRestore,
    SearchPageActionDisplay, SearchPageActionStyle, SAVED_SEARCH_MISSING_TEXT,
    SAVED_SEARCH_NAME_TAKEN_TEXT,
};
pub(crate) use saved_search_groups::{saved_search_window_page, SearchResultItemVm};
pub(crate) use saved_search_query::{SavedSearchQueryEdit, SavedSearchQueryKeep};
pub(crate) use search_run::{
    library_part_runs, live_run_step, IndexEntity, IndexOrder, IndexSearchPlan, IndexSearchRequest,
    LibrarySearchRows, LiveRunStep, SearchRun, SearchRunTrigger, INDEX_SKIP_ROWS_TEXT,
};
pub(crate) use tabs::{SearchResultItemId, SearchResultOrigin, SearchResultsTab, SearchTabCount};

use self::empty_state::empty_state_for;
use self::paged_tab::SearchRowMergeKey;
use self::results::local_library_result_rows;
use self::saved_search_groups::{
    saved_search_window_items, window_group_selection_row, SavedSearchWindowItem,
};

/// The count of loading rows while the Index part of a run loads (ADR 0087
/// packet 007 decision D2). The rows show after the rows of the active tab.
pub(crate) const INDEX_PENDING_ROW_COUNT: usize = 3;

/// The notes of the tabs that get no Index request while the plan sends one
/// (musicindex ADR 0052 R-QUERY-8 item 4).
#[derive(Clone, Debug, Default)]
struct TabScopeNotes {
    /// The note of the Releases tab.
    releases: Option<SearchNote>,
    /// The note of the Artists tab and of the Labels tab.
    artists: Option<SearchNote>,
}

/// GPUI-free page contract for the search results in the list column.
#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent fact of the page; ADR 0086 packet 005 names the two partial flags"
)]
pub(crate) struct SearchResultsInspectorPageVm {
    query: String,
    tab: SearchResultsTab,
    releases: SearchResultsPagedTab<FeedResultDisplay>,
    artists: SearchResultsPagedTab<ArtistResultDisplay>,
    tracks: SearchResultsPagedTab<TrackResultDisplay>,
    index_loading: bool,
    index_error: Option<EmptyStateDisplay>,
    /// The note of a text that sends no Index request (ADR 0087 Decision 5,
    /// musicindex ADR 0052 R-QUERY-8, ADR 0091). It is not an Index error.
    index_note: Option<SearchNote>,
    /// The note of the Releases tab and of the Artists tab when the plan
    /// sends a request and the tab gets none (R-QUERY-8 item 4).
    scope_notes: TabScopeNotes,
    empty_state: Option<EmptyStateDisplay>,
    /// The page entry of this view model: `FrameNavigationEntry::Search`
    /// for the toolbar search, or `FrameNavigationEntry::SavedSearch` for
    /// the saved-search page of the detail column (packet 012).
    entry: FrameNavigationEntry,
    /// The one selection of the page (packet 010 "The selection"), bound
    /// to `entry`.
    selection: DetailSelection,
    /// The status text of the view, for example a save failure (packet
    /// 012). `None` when the view has no status to tell.
    status_text: Option<String>,
    /// The Index feed rows are not all the feed rows of the query (ADR 0086
    /// packet 005 decision D3).
    feed_rows_partial: bool,
    /// The Library track search reached its limit, so the Library rows are
    /// not all the Library rows of the query (ADR 0086 packet 005 decision
    /// D3).
    library_rows_partial: bool,
    /// The generation of the newest run of the page (ADR 0087 Decision 4,
    /// packet 003). A result of an older run does not change the page.
    run_generation: RunGeneration,
    /// `true` when the page shows the Index rows or the Index error of its
    /// run (packet 007 decision D6).
    index_result_shown: bool,
    /// The Index result of this query that the page did not show, and its
    /// note (packet 007 decision D6).
    index_result_stop: Option<(IndexResultDiagnostic, EmptyStateDisplay)>,
    /// The list state of the page: its sort and its view, under the key of
    /// the results page or of a saved search (ADR 0089 Decision 5). A new
    /// run keeps it, see [`Self::with_list_state_of`].
    lists: ListPages,
    /// Whether the person selected the sort of the page. The default sort of
    /// a search page is Newest, and that default keeps relevance in the Index
    /// request (Stophammer ADR 0084). A new run keeps it.
    index_sort_selected: bool,
}

impl SearchResultsInspectorPageVm {
    /// Creates an empty search-results page for a query.
    #[must_use]
    pub(crate) fn new(query: impl Into<String>) -> Self {
        let query = query.into();
        let entry = FrameNavigationEntry::Search(query.clone());
        let mut selection = DetailSelection::new();
        selection.reset(Some(entry.clone()));
        let mut page = Self {
            query,
            tab: SearchResultsTab::default(),
            releases: SearchResultsPagedTab::empty(),
            artists: SearchResultsPagedTab::empty(),
            tracks: SearchResultsPagedTab::empty(),
            index_loading: false,
            index_error: None,
            index_note: None,
            scope_notes: TabScopeNotes::default(),
            empty_state: None,
            entry,
            selection,
            status_text: None,
            feed_rows_partial: false,
            library_rows_partial: false,
            run_generation: RunGeneration::default(),
            index_result_shown: false,
            index_result_stop: None,
            lists: ListPages::default(),
            index_sort_selected: false,
        };
        page.refresh_empty_state();
        page
    }

    /// Creates a search-results page from the Library rows of one run
    /// (ADR 0087 packet 002). The Releases tab and the Artists tab read
    /// `rows.releases`. The Tracks tab reads `rows.tracks`. `artist_facts`
    /// gives the match of each artist name (ADR 0084 Decision 7, packet
    /// 017).
    #[must_use]
    pub(crate) fn from_library_search(
        text: impl Into<String>,
        rows: &LibrarySearchRows,
        artist_facts: &ArtistSearchFacts,
    ) -> Self {
        let mut page = Self::new(text);
        let local_rows = local_library_result_rows(&rows.releases, &rows.tracks, artist_facts);
        page.artists = SearchResultsPagedTab::ready_library(local_rows.artists);
        page.releases = SearchResultsPagedTab::ready_library(local_rows.feeds);
        page.tracks = SearchResultsPagedTab::ready_library(local_rows.tracks);
        page.refresh_empty_state();
        page
    }

    /// Returns this page with the state of its Library rows: `partial` is
    /// true when the Library track search reached its limit (ADR 0086
    /// packet 005 decision D3).
    #[must_use]
    pub(crate) fn with_library_rows_partial(mut self, partial: bool) -> Self {
        self.library_rows_partial = partial;
        self
    }

    /// Returns this page with the list state of `previous`, the page that
    /// it replaces, so the sort and the view of the curator stay across a
    /// run (ADR 0089 Decision 5). `None` keeps the default state.
    #[must_use]
    pub(crate) fn with_list_state_of(mut self, previous: Option<&Self>) -> Self {
        if let Some(previous) = previous {
            self.lists = previous.lists.clone();
            self.index_sort_selected = previous.index_sort_selected;
        }
        self
    }

    /// The key of the list state of this page: a saved search, or the
    /// results page of the toolbar search.
    #[must_use]
    pub(crate) const fn list_key(&self) -> ListPageKey {
        match self.entry {
            FrameNavigationEntry::SavedSearch { .. } => ListPageKey::SavedSearch,
            _ => ListPageKey::Results,
        }
    }

    /// The list state of this page: its sort and its view (ADR 0089
    /// Decision 5). The sort is the effective sort of the page
    /// ([`Self::effective_sort`]).
    #[must_use]
    pub(crate) fn list_state(&self) -> ListPageState {
        ListPageState {
            sort: Some(self.effective_sort()),
            ..*self.lists.get(self.list_key())
        }
    }

    /// The sort of this page: the sort that the person selected, else
    /// Relevance when the query has a word, else Newest (musicindex ADR 0053
    /// R-LIST-9). A word is free text that is not a field term, so the query
    /// `in:library haleen` has one and the query `kind:single` has none.
    ///
    /// A saved search page starts with Newest, also with a word, so it shows
    /// the window groups of ADR 0086. The operator chose this on 2026-11-08,
    /// and the orchestrator session relayed the answer.
    fn effective_sort(&self) -> ListSort {
        if self.index_sort_selected {
            if let Some(sort) = self.lists.get(self.list_key()).sort {
                return sort;
            }
        }
        if matches!(self.entry, FrameNavigationEntry::SavedSearch { .. }) {
            return ListSort::Newest;
        }
        if SearchQuery::parse(&self.query).index_q().is_some() {
            ListSort::Relevance
        } else {
            ListSort::Newest
        }
    }

    /// Sets the sort of this page (ADR 0089 Decision 4). The query does not
    /// change, and the rows follow the sort at the next read.
    pub(crate) fn set_list_sort(&mut self, sort: ListSort) {
        self.lists.set_sort(self.list_key(), sort);
        self.index_sort_selected = true;
    }

    /// The sort of this page (musicindex ADR 0053 R-LIST-9).
    fn sort(&self) -> ListSort {
        self.effective_sort()
    }

    /// Sets the reversed state of this page (musicindex ADR 0043 R-LIST-7).
    /// The query does not change, and the rows follow at the next read.
    pub(crate) fn set_list_reversed(&mut self, reversed: bool) {
        self.lists.set_reversed(self.list_key(), reversed);
    }

    /// The order that the Index part of this page asks for (Stophammer ADR
    /// 0084). A sort that the person selected gives its sort. A page with no
    /// selected sort keeps relevance, which is the default of the node, also
    /// when the page shows Newest by default. The reversed state counts only
    /// where the page reverses (R-LIST-7).
    #[must_use]
    pub(crate) fn index_order(&self) -> IndexOrder {
        if !self.index_sort_selected {
            return IndexOrder::Relevance;
        }
        let reverse = self.reversed();
        match self.effective_sort() {
            // No search page offers Feed Order, Playlist Order or Curator
            // Order.
            ListSort::Relevance
            | ListSort::FeedOrder
            | ListSort::PlaylistOrder
            | ListSort::CuratorOrder => IndexOrder::Relevance,
            ListSort::Newest => IndexOrder::Date { reverse },
            ListSort::AToZ => IndexOrder::Name { reverse },
        }
    }

    /// Whether the rows come in the reversed order now. Only the results of
    /// the scope In Library reverse, because each of their rows is local.
    fn reversed(&self) -> bool {
        self.list_state()
            .reversed_in(self.list_key(), self.library_scope())
    }

    /// Returns this page with the run generation `generation` (packet 003).
    #[must_use]
    pub(crate) fn with_run_generation(mut self, generation: RunGeneration) -> Self {
        self.run_generation = generation;
        self
    }

    /// The generation of the newest run of the page.
    #[must_use]
    pub(crate) const fn run_generation(&self) -> RunGeneration {
        self.run_generation
    }

    /// `true` when this page can replace `current`: `current` is `None`, or
    /// this generation is equal to or greater than the generation of
    /// `current` (ADR 0087 invariant "A result of an older run never
    /// replaces the result of a newer run").
    #[must_use]
    pub(crate) fn is_newer_than(&self, current: Option<&Self>) -> bool {
        current.is_none_or(|current| self.run_generation >= current.run_generation)
    }

    /// Records that the app did not apply the Index result of the run
    /// `generation` for `query`, because of `reason` (packet 007 decision
    /// D6). The page records the note only when `query` is the page query
    /// and the page shows no Index result. Gives `true` when the page
    /// changed.
    pub(crate) fn record_index_result_stop(
        &mut self,
        query: &str,
        generation: RunGeneration,
        reason: IndexResultStopReason,
    ) -> bool {
        if self.query != query || self.index_result_shown {
            return false;
        }
        let diagnostic = IndexResultDiagnostic {
            query: self.query.clone(),
            run_generation: generation,
            page_generation: self.run_generation,
            reason,
        };
        let display = diagnostic.display();
        self.index_result_stop = Some((diagnostic, display));
        self.refresh_empty_state();
        true
    }

    /// The Index result that the page did not show, or `None` (packet 007
    /// decision D6). Production code reads the note through
    /// [`Self::index_notice`] and [`Self::empty_state`].
    #[cfg(test)]
    #[must_use]
    pub(crate) fn index_result_diagnostic(&self) -> Option<&IndexResultDiagnostic> {
        self.index_result_stop
            .as_ref()
            .map(|(diagnostic, _)| diagnostic)
    }

    /// The count of loading rows after the rows of the active tab (packet
    /// 007 decision D2). It is [`INDEX_PENDING_ROW_COUNT`] while the Index
    /// part loads, else 0. The rows of the active tab do not change it.
    #[must_use]
    pub(crate) const fn index_pending_row_count(&self) -> usize {
        if self.index_loading {
            INDEX_PENDING_ROW_COUNT
        } else {
            0
        }
    }

    /// Starts the Index part of the run `generation`. An older generation
    /// gives `false` and changes nothing. Else the page takes the
    /// generation, marks the Index part as loading and gives `true`.
    pub(crate) fn begin_index_run(&mut self, generation: RunGeneration) -> bool {
        if generation < self.run_generation {
            return false;
        }
        self.run_generation = generation;
        self.mark_index_loading();
        true
    }

    /// Applies the Index rows of the run `generation` for `query`. A
    /// generation other than the page generation gives `false` and changes
    /// nothing. Else the rule of `replace_index_results_for_query` applies.
    pub(crate) fn replace_index_results_for_run(
        &mut self,
        generation: RunGeneration,
        query: &str,
        rows: IndexSearchResultRows,
    ) -> bool {
        if generation != self.run_generation {
            return false;
        }
        self.replace_index_results_for_query(query, rows)
    }

    /// Records the Index error of the run `generation` for `query`. A
    /// generation other than the page generation, or a different query,
    /// gives `false` and changes nothing.
    pub(crate) fn set_index_error_for_run(
        &mut self,
        generation: RunGeneration,
        query: &str,
        error: &crate::application::CommandError,
        endpoint: &str,
        observed_at: std::time::SystemTime,
    ) -> bool {
        if generation != self.run_generation || self.query != query {
            return false;
        }
        self.set_index_error(error, endpoint, observed_at);
        true
    }

    /// Tells if the page holds each release row of the query (ADR 0086
    /// packet 005 "The open state"). The Index request is complete with no
    /// error, the feed response gave no later page, and the Library track
    /// search did not reach its limit. Then no later page can change a
    /// window group, so each group is closed.
    #[must_use]
    pub(crate) fn release_rows_complete(&self) -> bool {
        !self.index_loading
            && self.index_error.is_none()
            && !self.feed_rows_partial
            && !self.library_rows_partial
    }

    /// Returns this page as the page of the saved search `search` (packet
    /// 012). The page opens on the tab All (ADR 0089 Decision 4). The
    /// selection belongs to the saved-search entry.
    #[must_use]
    pub(crate) fn into_saved_search(mut self, search: &SavedSearch) -> Self {
        self.entry = search.entry();
        self.selection.reset(Some(self.entry.clone()));
        self.tab = SearchResultsTab::All;
        self.refresh_empty_state();
        self
    }

    /// Gives the page the saved-search `entry` and keeps `tab`, the tab that
    /// the curator chose. A live run of the query field uses it, so the tab
    /// stays when the results change (ADR 0089 Decision 4).
    #[must_use]
    pub(crate) fn into_saved_search_with_tab(
        mut self,
        entry: FrameNavigationEntry,
        tab: SearchResultsTab,
    ) -> Self {
        self.entry = entry.clone();
        self.selection.reset(Some(entry));
        self.tab = tab;
        self.refresh_empty_state();
        self
    }

    /// "Save search", or "Saved" when `saved` holds the query (packet 012).
    #[must_use]
    pub(crate) fn save_action(&self, saved: &[SavedSearch]) -> SearchPageActionDisplay {
        save_search_action(&self.query, saved)
    }

    /// Records the failure of "Save search" (packet 012). The status text
    /// names the query and the database error.
    pub(crate) fn set_save_failure(&mut self, error: &str) {
        self.status_text = Some(format!(
            "The app could not save the search \"{}\". Database error: {error}",
            self.query
        ));
    }

    /// Records the failure of "Delete…" of a saved search (packet 012). The
    /// status text names the query and the database error.
    pub(crate) fn set_delete_failure(&mut self, error: &str) {
        self.status_text = Some(format!(
            "The app could not delete the saved search \"{}\". Database error: {error}",
            self.query
        ));
    }

    /// Removes the status text, for example after a successful save.
    pub(crate) fn clear_status(&mut self) {
        self.status_text = None;
    }

    /// The status text of the view, or `None`.
    #[must_use]
    pub(crate) fn status_text(&self) -> Option<&str> {
        self.status_text.as_deref()
    }

    /// Marks the async remote Index search as pending.
    pub(crate) fn mark_index_loading(&mut self) {
        self.index_loading = true;
        self.index_result_shown = false;
        self.index_error = None;
        self.index_note = None;
        self.refresh_empty_state();
    }

    /// Records the note of a text that sends no Index request (ADR 0087
    /// Decision 5, packet 002 decision D17). The crate gives the title and
    /// the text (musicindex ADR 0052 R-QUERY-8, ADR 0091). A skip is not an
    /// Index error, so the page shows no loading rows and no error.
    pub(crate) fn set_index_skipped(&mut self, query: &SearchQuery, plan: &IndexSearchPlan) {
        self.index_loading = false;
        self.index_error = None;
        self.index_note = search_note(query, plan, Library::Present);
        self.refresh_empty_state();
    }

    /// Records the notes of the tabs that get no request while the plan
    /// sends one (R-QUERY-8 item 4). The Releases tab reads the feed note,
    /// and the Artists and Labels tabs read the publisher note.
    pub(crate) fn set_scope_notes(&mut self, query: &SearchQuery, plan: &IndexSearchPlan) {
        self.scope_notes = TabScopeNotes {
            releases: scope_note(query, plan, IndexEntity::Feed),
            artists: scope_note(query, plan, IndexEntity::Publisher),
        };
        self.refresh_empty_state();
    }

    /// The note of the Index part for the active tab, or `None`.
    fn tab_index_note(&self) -> Option<&SearchNote> {
        self.index_note.as_ref().or(match self.tab {
            SearchResultsTab::Releases => self.scope_notes.releases.as_ref(),
            SearchResultsTab::Artists | SearchResultsTab::Labels => {
                self.scope_notes.artists.as_ref()
            }
            SearchResultsTab::All | SearchResultsTab::Tracks => None,
        })
    }

    /// Returns whether the async remote Index search is pending. The
    /// renderer reads [`Self::index_pending_row_count`] (packet 007
    /// decision D2).
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn is_index_loading(&self) -> bool {
        self.index_loading
    }

    /// Merges the remote Index rows after the Library rows (packet 010
    /// "One row for each item"). A selected row that stays visible stays
    /// selected, and the focus stays.
    pub(crate) fn replace_index_results(&mut self, rows: IndexSearchResultRows) {
        self.index_loading = false;
        self.index_error = None;
        self.index_result_shown = true;
        self.index_result_stop = None;
        self.artists.replace_index_rows(rows.artists);
        self.releases.replace_index_rows(rows.feeds);
        self.tracks.replace_index_rows(rows.tracks);
        self.feed_rows_partial = rows.feed_rows_partial;
        let members = self.visible_members();
        self.selection.retain_visible(&members);
        self.refresh_empty_state();
    }

    /// Applies the Index rows of a request for `query` (packet 010 "The
    /// stale results"). A result for another query changes neither the
    /// rows nor the selection. Gives `true` when the rows changed.
    pub(crate) fn replace_index_results_for_query(
        &mut self,
        query: &str,
        rows: IndexSearchResultRows,
    ) -> bool {
        if self.query != query {
            return false;
        }
        self.replace_index_results(rows);
        true
    }

    /// Marks the async remote Index search as failed.
    pub(crate) fn set_index_error(
        &mut self,
        error: &crate::application::CommandError,
        endpoint: &str,
        observed_at: std::time::SystemTime,
    ) {
        self.index_loading = false;
        self.index_result_shown = true;
        self.index_result_stop = None;
        // Walk of 2026-10-21: an HTTP `400` reply is a query error, not an
        // availability error.
        let (title, failure) =
            if matches!(error, crate::application::CommandError::QueryRejected(_)) {
                (
                    failure::QUERY_REJECTED_TITLE,
                    failure::SearchFailureDisplay::query_rejected(
                        &self.query,
                        error,
                        endpoint,
                        observed_at,
                    ),
                )
            } else {
                (
                    "Music Index search unavailable",
                    failure::SearchFailureDisplay::new(error, endpoint, observed_at),
                )
            };
        let mut empty = EmptyStateDisplay::new(title, failure.summary.clone());
        empty.failure = Some(failure);
        self.index_error = Some(empty);
        self.refresh_empty_state();
    }

    /// Applies disclosure state or returns the exact report for clipboard delivery.
    pub(crate) fn activate_failure_action(
        &mut self,
        action: SearchFailureAction,
    ) -> Option<String> {
        let report = self
            .index_error
            .as_mut()?
            .failure
            .as_mut()?
            .activate(action);
        self.refresh_empty_state();
        report
    }

    /// Returns the query represented by this page.
    #[must_use]
    pub(crate) fn query(&self) -> &str {
        &self.query
    }

    /// The scope "All | In Library" of the query text (ADR 0089 Decision 4).
    #[must_use]
    pub(crate) fn library_scope(&self) -> ListScope {
        library_scope_of(&self.query)
    }

    /// Returns the selected tab.
    #[must_use]
    pub(crate) const fn tab(&self) -> SearchResultsTab {
        self.tab
    }

    /// Selects a tab. A change of tab clears the selection and sets
    /// `Normal` (packet 010 decision D8).
    pub(crate) fn set_tab(&mut self, tab: SearchResultsTab) {
        if tab != self.tab {
            self.selection.reset(Some(self.entry.clone()));
        }
        self.tab = tab;
        self.refresh_empty_state();
    }

    /// The label of the row `id` of the tab `tab`. The tab All searches
    /// each group, in the order of [`SearchResultsTab::GROUPS`].
    #[must_use]
    pub(crate) fn row_label(&self, tab: SearchResultsTab, id: &str) -> Option<String> {
        group_tabs_of(tab).into_iter().find_map(|tab| match tab {
            SearchResultsTab::Releases => self
                .releases
                .cached_row_matching(|row| row.id == id)
                .map(|row| row.label),
            SearchResultsTab::Tracks => self
                .tracks
                .cached_row_matching(|row| row.id == id)
                .map(|row| row.label),
            SearchResultsTab::Artists | SearchResultsTab::Labels => self
                .artist_rows_of(tab)
                .into_iter()
                .find(|row| row.id == id)
                .map(|row| row.label),
            SearchResultsTab::All => None,
        })
    }

    /// Returns whether a tab has no rows.
    #[must_use]
    pub(crate) fn is_empty(&self, tab: SearchResultsTab) -> bool {
        self.count_of(tab) == 0
    }

    /// The rows of one group of the run that the tab `tab` shows. The
    /// rows of the Artists tab and the Labels tab are one list, divided by
    /// the role of each feed (musicindex ADR 0033 R-WORD-3).
    fn artist_rows_of(&self, tab: SearchResultsTab) -> Vec<ArtistResultDisplay> {
        let mut rows: Vec<ArtistResultDisplay> = self
            .artists
            .rows()
            .into_iter()
            .filter(|row| rows::artist_tab(row) == tab)
            .collect();
        // An artist row has no date, so Newest keeps the order of the run.
        sort::sort_rows(&mut rows, self.sort(), |_| None, |row| row.label.as_str());
        if self.reversed() {
            rows.reverse();
        }
        rows
    }

    /// The release rows in the order of the sort. A row with no date uses
    /// the release date of its Library release, when the Library holds it.
    /// The grouped Releases of a saved search with Newest keep the window
    /// order of ADR 0086, because the groups read that order.
    fn sorted_releases(&self, lookup: &dyn SearchLibraryLookup) -> Vec<FeedResultDisplay> {
        let mut rows = self.releases.rows();
        if self.releases_grouped() {
            return rows;
        }
        sort::sort_rows(
            &mut rows,
            self.sort(),
            |row| {
                row.newest_item_at.or_else(|| {
                    row.key
                        .as_ref()
                        .and_then(|key| lookup.release_window_facts(key))
                        .and_then(|facts| facts.release_date)
                })
            },
            |row| row.label.as_str(),
        );
        if self.reversed() {
            rows.reverse();
        }
        rows
    }

    /// The track rows in the order of the sort. A track row has no date, so
    /// Newest keeps the order of the run.
    fn sorted_tracks(&self) -> Vec<TrackResultDisplay> {
        let mut rows = self.tracks.rows();
        sort::sort_rows(&mut rows, self.sort(), |_| None, |row| row.label.as_str());
        if self.reversed() {
            rows.reverse();
        }
        rows
    }

    /// The count of the rows of the tab `tab`. The tab All gives the sum of
    /// the groups.
    fn count_of(&self, tab: SearchResultsTab) -> usize {
        match tab {
            SearchResultsTab::All => SearchResultsTab::GROUPS
                .into_iter()
                .map(|group| self.count_of(group))
                .sum(),
            SearchResultsTab::Releases => self.releases.window().total(),
            SearchResultsTab::Tracks => self.tracks.window().total(),
            SearchResultsTab::Artists | SearchResultsTab::Labels => self.artist_rows_of(tab).len(),
        }
    }

    /// Tells if the rows of the tab `tab` are not all the rows of the
    /// query. The feed rows hold the artist and label rows (ADR 0086 packet
    /// 005 decision D3), and the Library track search holds the release and
    /// track rows.
    fn has_more_of(&self, tab: SearchResultsTab) -> bool {
        match tab {
            SearchResultsTab::All => SearchResultsTab::GROUPS
                .into_iter()
                .any(|group| self.has_more_of(group)),
            SearchResultsTab::Releases => self.feed_rows_partial || self.library_rows_partial,
            SearchResultsTab::Tracks => self.library_rows_partial,
            SearchResultsTab::Artists | SearchResultsTab::Labels => self.feed_rows_partial,
        }
    }

    /// The count of the tab `tab` from the rows of this run (ADR 0089
    /// Decision 4). A tab with rows and a partial result shows "20+".
    #[must_use]
    pub(crate) fn tab_count(&self, tab: SearchResultsTab) -> SearchTabCount {
        let count = self.count_of(tab);
        SearchTabCount {
            tab,
            count,
            has_more: count > 0 && self.has_more_of(tab),
        }
    }

    /// The counts of the tabs, in the order of [`SearchResultsTab::ALL`].
    #[must_use]
    pub(crate) fn tab_counts(&self) -> [SearchTabCount; 5] {
        SearchResultsTab::ALL.map(|tab| self.tab_count(tab))
    }

    /// Returns the empty state for the active tab, if any.
    #[must_use]
    pub(crate) fn empty_state(&self) -> Option<&EmptyStateDisplay> {
        self.empty_state.as_ref()
    }

    /// The note of the Index part under the rows of the active tab (walk of
    /// 2026-10-21, packet 028 D5). It shows when the tab has rows, for
    /// example Library rows, and the Index part failed: `MusicIndex` did
    /// not accept the query, or `MusicIndex` was not available. A tab with
    /// no rows shows the same note as its empty state.
    #[must_use]
    ///
    /// A skipped Index part shows the note of ADR 0087 Decision 5 when the
    /// Index cannot apply a term of the query.
    ///
    /// The note of an Index result that the page did not show comes first
    /// (packet 007 decision D6). It also shows after the loading rows of a
    /// tab with no rows. When the page shows it as the empty state, this
    /// function does not give it again.
    ///
    /// The note of the crate comes with one sentence of the app about its
    /// Library rows, which show above the note (R-QUERY-8 item 2).
    pub(crate) fn index_notice(&self) -> Option<EmptyStateDisplay> {
        if let Some((_, note)) = &self.index_result_stop {
            if self.empty_state.is_none() {
                return Some(note.clone());
            }
        }
        if self.is_empty(self.tab) {
            return None;
        }
        self.index_error.clone().or_else(|| {
            self.tab_index_note().map(|note| {
                EmptyStateDisplay::new(
                    note.title.clone(),
                    format!("{} {INDEX_SKIP_ROWS_TEXT}", note.text),
                )
            })
        })
    }

    /// Returns the visible row thumbnail hrefs of the active tab.
    #[must_use]
    pub(crate) fn thumbnail_hrefs(&self, lookup: &dyn SearchLibraryLookup) -> Vec<String> {
        group_tabs_of(self.tab)
            .into_iter()
            .flat_map(|tab| self.group_thumbnail_hrefs(tab, lookup))
            .collect()
    }

    /// The visible thumbnail hrefs of one group, in the order of the sort.
    fn group_thumbnail_hrefs(
        &self,
        tab: SearchResultsTab,
        lookup: &dyn SearchLibraryLookup,
    ) -> Vec<String> {
        match tab {
            SearchResultsTab::Releases => visible_thumbnail_hrefs(
                &self.sorted_releases(lookup),
                self.releases.window().page_size(),
                |row| row.thumbnail_href.as_deref(),
            ),
            SearchResultsTab::Artists | SearchResultsTab::Labels => self
                .artist_rows_of(tab)
                .iter()
                .filter_map(|row| row.thumbnail_href.clone())
                .take(self.artists.window().page_size())
                .collect(),
            SearchResultsTab::Tracks => visible_thumbnail_hrefs(
                &self.sorted_tracks(),
                self.tracks.window().page_size(),
                |row| row.thumbnail_href.as_deref(),
            ),
            SearchResultsTab::All => Vec::new(),
        }
    }

    /// The visible rows of the active tab, with the state word, the badge
    /// and the selection facts of each row. `lookup` gives the Library
    /// facts at this render. Production code reads the rows through
    /// [`Self::visible_items`].
    #[cfg(test)]
    #[must_use]
    pub(crate) fn visible_rows(&self, lookup: &dyn SearchLibraryLookup) -> Vec<SearchResultRowVm> {
        group_tabs_of(self.tab)
            .into_iter()
            .flat_map(|tab| self.group_rows(tab, lookup))
            .collect()
    }

    /// The rows of one group, in the visible order. The tab All has no
    /// group of its own, so it gives no row.
    fn group_rows(
        &self,
        tab: SearchResultsTab,
        lookup: &dyn SearchLibraryLookup,
    ) -> Vec<SearchResultRowVm> {
        match tab {
            SearchResultsTab::Releases => self
                .sorted_releases(lookup)
                .iter()
                .map(|row| {
                    rows::release_row_vm(
                        row,
                        lookup,
                        &self.selection,
                        self.release_kind_placement(),
                    )
                })
                .collect(),
            SearchResultsTab::Artists | SearchResultsTab::Labels => self
                .artist_rows_of(tab)
                .iter()
                .map(|row| rows::artist_row_vm(row, lookup))
                .collect(),
            SearchResultsTab::Tracks => self
                .sorted_tracks()
                .iter()
                .map(|row| rows::track_row_vm(row, lookup, &self.selection))
                .collect(),
            SearchResultsTab::All => Vec::new(),
        }
    }

    /// The visible items of the active tab (ADR 0086 Decision 6). The
    /// Releases tab of a saved search page gives one group item for two or
    /// more singles of one artist in one window, at the place of its
    /// anchor. Each other tab and the toolbar search give one row item for
    /// each row. `lookup` gives the Library facts at this render.
    #[must_use]
    pub(crate) fn visible_items(
        &self,
        lookup: &dyn SearchLibraryLookup,
    ) -> Vec<SearchResultItemVm> {
        if self.tab != SearchResultsTab::All {
            return self.group_items(self.tab, lookup);
        }
        // ADR 0089 Decision 4: the tab All lists each group that has rows,
        // in the order of `SearchResultsTab::GROUPS`, under its heading.
        SearchResultsTab::GROUPS
            .into_iter()
            .flat_map(|group| {
                let items = self.group_items(group, lookup);
                if items.is_empty() {
                    Vec::new()
                } else {
                    std::iter::once(SearchResultItemVm::Heading(group))
                        .chain(items)
                        .collect()
                }
            })
            .collect()
    }

    /// The items of one group. The Releases group of a saved search keeps
    /// its window groups (ADR 0086 Decision 6).
    fn group_items(
        &self,
        tab: SearchResultsTab,
        lookup: &dyn SearchLibraryLookup,
    ) -> Vec<SearchResultItemVm> {
        if tab == SearchResultsTab::Artists {
            return self.artist_group_items(lookup);
        }
        if tab != SearchResultsTab::Releases || !self.releases_grouped() {
            return self
                .group_rows(tab, lookup)
                .into_iter()
                .map(SearchResultItemVm::Row)
                .collect();
        }
        let rows = self.sorted_releases(lookup);
        let placement = self.release_kind_placement();
        saved_search_window_items(&rows, lookup, self.release_rows_complete())
            .into_iter()
            .map(|item| match item {
                SavedSearchWindowItem::Plain(position) => SearchResultItemVm::Row(
                    rows::release_row_vm(&rows[position], lookup, &self.selection, placement),
                ),
                SavedSearchWindowItem::Group { mut tile, .. } => {
                    tile.selection = self
                        .selection
                        .tile_selection(&window_group_selection_row(&tile), &tile.title);
                    SearchResultItemVm::WindowGroup(tile)
                }
            })
            .collect()
    }

    /// The items of the Artists group (musicindex ADR 0057 R-ARTIST-1): the
    /// stated artists, the inferred artists and the artist names with no
    /// feed, in this sequence, each under its heading. The sort of the list
    /// orders the rows in each group, and a group with no row has no
    /// heading.
    fn artist_group_items(&self, lookup: &dyn SearchLibraryLookup) -> Vec<SearchResultItemVm> {
        let rows = self.artist_rows_of(SearchResultsTab::Artists);
        ArtistGroup::ALL
            .into_iter()
            .flat_map(|group| {
                let items: Vec<SearchResultItemVm> = rows
                    .iter()
                    .filter(|row| row.group() == group)
                    .map(|row| SearchResultItemVm::Row(rows::artist_row_vm(row, lookup)))
                    .collect();
                if items.is_empty() {
                    Vec::new()
                } else {
                    std::iter::once(SearchResultItemVm::ArtistGroupHeading(group))
                        .chain(items)
                        .collect()
                }
            })
            .collect()
    }

    /// The group tile whose anchor row is `row_id` (ADR 0086 packet 005).
    /// A click and Enter on that row open the window group page of the
    /// tile. `None` for each other row.
    #[must_use]
    pub(crate) fn window_group_for_row(
        &self,
        row_id: &str,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<WindowGroupTileVm> {
        if !self.groups_release_rows() {
            return None;
        }
        saved_search_window_items(
            &self.sorted_releases(lookup),
            lookup,
            self.release_rows_complete(),
        )
        .into_iter()
        .find_map(|item| match item {
            SavedSearchWindowItem::Group { tile, .. } if tile.open_row_id == row_id => Some(*tile),
            SavedSearchWindowItem::Group { .. } | SavedSearchWindowItem::Plain(_) => None,
        })
    }

    /// Tells if the Releases rows of this page are grouped: a saved search
    /// page groups them (ADR 0086 packet 005 decision D1).
    fn releases_grouped(&self) -> bool {
        matches!(self.entry, FrameNavigationEntry::SavedSearch { .. })
            && self.sort() == ListSort::Newest
    }

    /// Tells if the active tab groups its rows: the Releases tab, or the
    /// tab All with its Releases group, of a saved search page (ADR 0086
    /// packet 005 decision D1).
    fn groups_release_rows(&self) -> bool {
        matches!(self.tab, SearchResultsTab::Releases | SearchResultsTab::All)
            && self.releases_grouped()
    }

    /// The selectable rows of the Releases tab of a saved search page with
    /// one group or more (ADR 0086 packet 004 decision D11). `rows` holds
    /// each shown release and each member of each group, in item order.
    /// `tiles` holds one selection row for each item with a key. `None`
    /// when no group shows.
    fn grouped_release_rows(&self, lookup: &dyn SearchLibraryLookup) -> Option<DetailPageRows> {
        if !self.groups_release_rows() {
            return None;
        }
        let rows = self.sorted_releases(lookup);
        let items = saved_search_window_items(&rows, lookup, true);
        if !items
            .iter()
            .any(|item| matches!(item, SavedSearchWindowItem::Group { .. }))
        {
            return None;
        }
        let mut ordered = Vec::new();
        let mut tiles = Vec::new();
        for item in items {
            match item {
                SavedSearchWindowItem::Plain(position) => {
                    let row = &rows[position];
                    if let Some(key) = row.merge_key() {
                        tiles.push(SelectionRow::of_one(DetailRowKey::Release(key)));
                    }
                    ordered.push(row.clone());
                }
                SavedSearchWindowItem::Group { tile, members } => {
                    tiles.push(window_group_selection_row(&tile));
                    ordered.extend(members.into_iter().map(|position| rows[position].clone()));
                }
            }
        }
        Some(DetailPageRows::GroupedReleases {
            rows: search_release_rows(&ordered, lookup),
            tiles,
        })
    }

    /// The place of the kind of a release row (walk of 2026-10-21). The
    /// toolbar search in the list column puts the kind in the second line.
    /// The saved-search page in the detail column keeps the kind badge.
    #[must_use]
    pub(crate) const fn release_kind_placement(&self) -> ReleaseKindPlacement {
        match self.entry {
            FrameNavigationEntry::SavedSearch { .. } => ReleaseKindPlacement::Badge,
            _ => ReleaseKindPlacement::Subtitle,
        }
    }

    /// The selectable rows of the active tab (packet 010 "The
    /// selection"). The Artists tab has no selection model, so it gives
    /// `None`.
    #[must_use]
    pub(crate) fn selection_rows(
        &self,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<DetailPageRows> {
        // ADR 0089 packet 008 decision 9: the tab All has no selection
        // model, so it has no Select button and no checkbox.
        if self.tab == SearchResultsTab::All {
            return None;
        }
        if let Some(grouped) = self.grouped_release_rows(lookup) {
            return Some(grouped);
        }
        match self.tab {
            SearchResultsTab::Releases => Some(DetailPageRows::Releases(search_release_rows(
                &self.sorted_releases(lookup),
                lookup,
            ))),
            SearchResultsTab::Tracks => Some(DetailPageRows::Tracks(search_track_rows(
                &self.sorted_tracks(),
                lookup,
            ))),
            SearchResultsTab::All | SearchResultsTab::Artists | SearchResultsTab::Labels => None,
        }
    }

    /// The selection of the search list.
    #[must_use]
    pub(crate) const fn selection(&self) -> &DetailSelection {
        &self.selection
    }

    /// The "Select" or "Done" button of the column header. It shows only
    /// on the Releases tab and the Tracks tab, and only when the tab has a
    /// row with a key. The toolbar search gives the id of the list column.
    /// The saved-search page gives the id of the detail column (packet
    /// 012).
    #[must_use]
    pub(crate) fn header_action(
        &self,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<FrameHeaderActionDisplay> {
        let rows = self.selection_rows(lookup);
        match self.entry {
            FrameNavigationEntry::SavedSearch { .. } => {
                detail_header_action(rows.as_ref(), self.selection.mode())
            }
            _ => search_header_action(rows.as_ref(), self.selection.mode()),
        }
    }

    /// The "Select" or "Done" button of the title row of a search page (ADR
    /// 0089 Decision 6). It has the rule of [`Self::header_action`].
    #[must_use]
    pub(crate) fn select_action(
        &self,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<SearchPageActionDisplay> {
        let action = self.header_action(lookup)?;
        Some(SearchPageActionDisplay {
            id: "music-search-list-select",
            label: action.label,
            a11y_label: action.a11y_label,
            available: true,
            style: SearchPageActionStyle::Plain,
        })
    }

    /// The action bar of packet 018, or `None` outside selection mode.
    #[must_use]
    pub(crate) fn action_bar(
        &self,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<SelectionActionBarVm> {
        self.selection_rows(lookup)?.action_bar(&self.selection)
    }

    /// A click on the body of the row with the selection key `key`. Gives
    /// `true` when the caller must open the page of the row: a plain click
    /// in `Normal` mode.
    pub(crate) fn click_row(
        &mut self,
        key: &DetailRowKey,
        click: DetailRowClick,
        lookup: &dyn SearchLibraryLookup,
    ) -> bool {
        let visible = self
            .selection_rows(lookup)
            .map(|rows| rows.selection_rows())
            .unwrap_or_default();
        self.selection.click(key, click, &visible)
    }

    /// A click on the checkbox of the row `key`. The checkbox of a group
    /// row changes the selection of each member of the group (ADR 0086
    /// packet 004 decision D12).
    pub(crate) fn toggle_row(&mut self, key: &DetailRowKey, lookup: &dyn SearchLibraryLookup) {
        let group = self.grouped_release_rows(lookup).and_then(|rows| {
            rows.selection_rows()
                .into_iter()
                .find(|row| &row.id == key && row.members.len() > 1)
        });
        match group {
            Some(row) => self.selection.toggle_selection_row(&row),
            None => self.selection.toggle_row(key),
        }
    }

    /// The "Select" and "Done" button.
    pub(crate) fn toggle_mode(&mut self) {
        self.selection.toggle_mode();
    }

    /// "Clear" of the bar: clears the rows and keeps the mode.
    pub(crate) fn clear_rows(&mut self) {
        self.selection.clear_rows();
    }

    /// Keeps the focus mark with the keyboard focus (R-FOCUS-1).
    pub(crate) fn sync_keyboard_focus(
        &mut self,
        shows: bool,
        has_marked_row: bool,
        lookup: &dyn SearchLibraryLookup,
    ) {
        match self.selectable_rows(lookup) {
            Some(rows) => {
                self.selection
                    .enter_with_keyboard(shows, has_marked_row, &rows.selection_rows());
            }
            None => self.selection.set_focus_mark_shows(shows),
        }
    }

    /// Space: changes the selection of the focused row.
    pub(crate) fn toggle_focused(&mut self, lookup: &dyn SearchLibraryLookup) {
        if let Some(rows) = self.selectable_rows(lookup) {
            self.selection.toggle_focused(&rows.selection_rows());
        }
    }

    /// Ctrl+A: selects each row of the active tab.
    pub(crate) fn select_all(&mut self, lookup: &dyn SearchLibraryLookup) {
        if let Some(rows) = self.selectable_rows(lookup) {
            self.selection.select_all(&rows.selection_rows());
        }
    }

    /// Up, Down, Page Up, Page Down, Home and End: moves the row focus.
    pub(crate) fn move_focus(
        &mut self,
        step: FocusStep,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<DetailRowKey> {
        let rows = self.focus_rows(lookup)?;
        self.selection.move_focus(step, &rows)
    }

    /// The rows that the keyboard focus moves through. The tab All moves
    /// through each group in display order, across the group headings
    /// (ADR 0089 Decision 9: All has no Select and no selection model).
    fn focus_rows(
        &self,
        lookup: &dyn SearchLibraryLookup,
    ) -> Option<Vec<SelectionRow<DetailRowKey, DetailRowKey>>> {
        if self.tab == SearchResultsTab::All {
            let rows: Vec<_> = self
                .visible_items(lookup)
                .iter()
                .filter_map(SearchResultItemVm::key)
                .map(SelectionRow::of_one)
                .collect();
            return (!rows.is_empty()).then_some(rows);
        }
        self.selectable_rows(lookup)
            .map(|rows| rows.selection_rows())
    }

    /// The index of the item with the keyboard focus among all the items
    /// of the active tab, or `None` when no item has the focus (packet 028
    /// D2, ADR 0086 packet 005). The items are the children of the scroll
    /// container in this order, thus the app layer scrolls to this index
    /// after a focus move.
    #[must_use]
    pub(crate) fn focused_row_index(&self, lookup: &dyn SearchLibraryLookup) -> Option<usize> {
        let focused = self.selection.focused()?;
        if self.tab == SearchResultsTab::All || self.groups_release_rows() {
            return self
                .visible_items(lookup)
                .iter()
                .position(|item| item.key().as_ref() == Some(focused));
        }
        match self.tab {
            SearchResultsTab::Releases => self.sorted_releases(lookup).iter().position(|row| {
                row.merge_key()
                    .is_some_and(|key| DetailRowKey::Release(key) == *focused)
            }),
            SearchResultsTab::Tracks => self.sorted_tracks().iter().position(|row| {
                row.merge_key()
                    .is_some_and(|key| DetailRowKey::Track(key) == *focused)
            }),
            SearchResultsTab::All | SearchResultsTab::Artists | SearchResultsTab::Labels => None,
        }
    }

    /// Escape: ends selection mode. Gives `false` in `Normal`.
    pub(crate) fn end_selecting(&mut self) -> bool {
        self.selection.end_selecting()
    }

    fn selectable_rows(&self, lookup: &dyn SearchLibraryLookup) -> Option<DetailPageRows> {
        self.selection_rows(lookup)
            .filter(DetailPageRows::has_selectable_rows)
    }

    /// The selection key of each visible row of the active tab. The keys
    /// need no Library fact.
    fn visible_members(&self) -> Vec<DetailRowKey> {
        let release_keys = || {
            self.releases
                .rows()
                .into_iter()
                .filter_map(|row| row.merge_key().map(DetailRowKey::Release))
        };
        let track_keys = || {
            self.tracks
                .rows()
                .into_iter()
                .filter_map(|row| row.merge_key().map(DetailRowKey::Track))
        };
        match self.tab {
            SearchResultsTab::Releases => release_keys().collect(),
            SearchResultsTab::Tracks => track_keys().collect(),
            SearchResultsTab::All => release_keys().chain(track_keys()).collect(),
            SearchResultsTab::Artists | SearchResultsTab::Labels => Vec::new(),
        }
    }

    fn refresh_empty_state(&mut self) {
        if !self.is_empty(self.tab) {
            self.empty_state = None;
            return;
        }
        if self.index_loading {
            self.empty_state = None;
            return;
        }
        if let Some(error) = &self.index_error {
            self.empty_state = Some(error.clone());
            return;
        }
        // Packet 007 decision D6: a page with no rows tells why it shows
        // no Index rows.
        if let Some((_, note)) = &self.index_result_stop {
            self.empty_state = Some(note.clone());
            return;
        }
        self.empty_state = Some(match self.tab_index_note() {
            Some(note) => EmptyStateDisplay::new(note.title.clone(), note.text.clone()),
            None => empty_state_for(self.tab, &self.query),
        });
    }
}

/// The tab rows have no caller outside the tests of this module:
/// production code reads the rows through [`SearchResultsInspectorPageVm::visible_rows`].
#[cfg(test)]
impl SearchResultsInspectorPageVm {
    /// Creates a search-results page whose three tabs read the same Library
    /// tracks.
    #[must_use]
    pub(crate) fn from_local_library_tracks(
        query: impl Into<String>,
        tracks: &[crate::db::TrackRow],
        artist_facts: &ArtistSearchFacts,
    ) -> Self {
        let rows = LibrarySearchRows {
            releases: tracks.to_vec(),
            tracks: tracks.to_vec(),
            matched_count: tracks.len(),
        };
        Self::from_library_search(query, &rows, artist_facts)
    }

    /// Returns the release rows.
    #[must_use]
    pub(crate) const fn releases(&self) -> &SearchResultsPagedTab<FeedResultDisplay> {
        &self.releases
    }

    /// Returns the artist rows.
    #[must_use]
    pub(crate) const fn artists(&self) -> &SearchResultsPagedTab<ArtistResultDisplay> {
        &self.artists
    }

    /// Returns the track rows.
    #[must_use]
    pub(crate) const fn tracks(&self) -> &SearchResultsPagedTab<TrackResultDisplay> {
        &self.tracks
    }
}

/// The group tabs that a tab shows: the four groups for the tab All, else
/// the tab itself.
fn group_tabs_of(tab: SearchResultsTab) -> Vec<SearchResultsTab> {
    match tab {
        SearchResultsTab::All => SearchResultsTab::GROUPS.to_vec(),
        group => vec![group],
    }
}

/// The thumbnail hrefs of the first page of `rows`, in their order.
fn visible_thumbnail_hrefs<Row>(
    rows: &[Row],
    page_size: usize,
    thumbnail_href: impl Fn(&Row) -> Option<&str>,
) -> Vec<String> {
    rows.iter()
        .take(page_size)
        .filter_map(|row| thumbnail_href(row).map(str::to_string))
        .collect()
}
