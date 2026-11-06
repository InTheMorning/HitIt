//! The Search page (ADR 0089 Decision 4).
//!
//! The Search destination lists each saved search with its query. With no
//! saved search, the page gives the four steps of "How to Search" and the
//! example queries of ADR 0087 Decision 2. The page has no renderer type.
//! Each row holds the saved search that it opens.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;

use crate::view_models::search_query::QUERY_FIELDS;
use crate::view_models::search_results::{
    new_results_text, new_saved_search_action, saved_search_label, saved_search_menu,
    saved_search_rename_display, NewResultCount, NewResultsText, SavedSearch, SavedSearchId,
    SavedSearchMenuEntry, SavedSearchRenameDisplay, SearchPageActionDisplay,
};

/// The four steps of "How to Search" (ADR 0089 Decision 4).
pub(crate) const HOW_TO_SEARCH_STEPS: [&str; 4] = [
    "Click the search field in the toolbar.",
    "Type a word, a phrase or a field, for example artist:jsmovie.",
    "Wait for the results, or press Enter to run the search.",
    "Click Save search to keep the search in this list.",
];

/// A request of the Search page to the top app (ADR 0089 Decision 4). The
/// top app owns the toolbar field, so the page sends the request and the
/// top app changes the field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SearchPageAction {
    /// Empties the toolbar field and puts the focus in it.
    NewSavedSearch,
    /// Puts the query into the toolbar field and runs it.
    RunQuery(String),
    /// Opens the delete confirmation of a saved search (ADR 0089 Decision 4).
    DeleteSavedSearch(SavedSearch),
    /// Opens the saved search with its query field open (ADR 0089 Decision 4).
    EditSavedSearchQuery(SavedSearch),
    /// Copies the saved search, then opens the copy with its query field open.
    DuplicateSavedSearch(SavedSearchId),
}

/// One row of the Search page: one saved search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SearchPageRowVm {
    /// The saved search that the row opens.
    pub(crate) search: SavedSearch,
    /// Stable element id.
    pub(crate) element_id: String,
    /// The visible text: the query of the saved search.
    pub(crate) label: String,
    /// The subtitle: the Library and the Index (ADR 0084 Decision 3).
    pub(crate) subtitle: &'static str,
    /// The accessibility label of the row.
    pub(crate) a11y_label: String,
    /// Whether the row is the open page of the current location.
    pub(crate) selected: bool,
    /// The item menu of the row, in the sequence of ADR 0089 Decision 4.
    pub(crate) menu: [SavedSearchMenuEntry; 4],
    /// The inline rename controls of the row.
    pub(crate) rename: SavedSearchRenameDisplay,
    /// The count of new results (ADR 0089 Decision 4). `None` for a zero
    /// count and for a count that is unavailable.
    pub(crate) new_results: Option<NewResultsText>,
}

/// One example query of "How to Search".
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SearchExampleVm {
    /// Stable element id.
    pub(crate) element_id: String,
    /// The query that a click runs.
    pub(crate) query: &'static str,
    /// The accessibility label of the example.
    pub(crate) a11y_label: String,
}

/// The state of the "How to Search" disclosure over a list with saved
/// searches (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum HowToSearchDisclosure {
    /// The steps and examples are hidden.
    #[default]
    Hidden,
    /// The steps and examples show.
    Shown,
}

/// The view model of the Search page (ADR 0089 Decision 4).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SearchPageVm {
    rows: Vec<SearchPageRowVm>,
    examples: Vec<SearchExampleVm>,
    new_action: SearchPageActionDisplay,
    message: Option<String>,
}

impl SearchPageVm {
    /// The page for the saved searches `saved`, in `id` order. `open` is the
    /// saved search of the current location, when one is open.
    #[must_use]
    pub(crate) fn new(saved: &[SavedSearch], open: Option<SavedSearchId>) -> Self {
        let mut in_order: Vec<&SavedSearch> = saved.iter().collect();
        in_order.sort_by_key(|search| search.id);
        let rows = in_order
            .into_iter()
            .map(|search| SearchPageRowVm {
                search: search.clone(),
                element_id: format!("music-search-saved-search-{}", search.id.0),
                label: search.display_label().to_owned(),
                subtitle: SavedSearch::SUBTITLE,
                a11y_label: row_a11y_label(search, None),
                selected: open == Some(search.id),
                menu: saved_search_menu(search),
                rename: saved_search_rename_display(search),
                new_results: None,
            })
            .collect();
        // ADR 0087 Decision 2: the examples are the field table, one for each field.
        let examples = QUERY_FIELDS
            .iter()
            .map(|info| SearchExampleVm {
                element_id: format!("music-search-example-{}", info.example),
                query: info.example,
                a11y_label: format!("Run the example search {}", info.example),
            })
            .collect();
        Self {
            rows,
            examples,
            new_action: new_saved_search_action(),
            message: None,
        }
    }

    /// The same page with a message of the last request, for example a
    /// missing saved search or a copy that already exists.
    #[must_use]
    pub(crate) fn with_message(mut self, message: Option<String>) -> Self {
        self.message = message;
        self
    }

    /// The same page with the count of new results of each saved search
    /// (ADR 0089 Decision 4). The accessibility label of a row with new
    /// results names them in a sentence, so the count does not rely on
    /// color alone.
    #[must_use]
    pub(crate) fn with_new_counts(
        mut self,
        counts: &BTreeMap<SavedSearchId, NewResultCount>,
    ) -> Self {
        for row in &mut self.rows {
            let count = counts
                .get(&row.search.id)
                .copied()
                .unwrap_or(NewResultCount::Unavailable);
            row.new_results = new_results_text(count);
            row.a11y_label = row_a11y_label(&row.search, row.new_results.as_ref());
        }
        self
    }

    /// The message of the last request, if any.
    #[must_use]
    pub(crate) fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// "New Saved Search", a button with typed action state.
    #[must_use]
    pub(crate) const fn new_action(&self) -> &SearchPageActionDisplay {
        &self.new_action
    }

    /// The saved searches, one row each.
    #[must_use]
    pub(crate) fn rows(&self) -> &[SearchPageRowVm] {
        &self.rows
    }

    /// The four steps of "How to Search".
    #[must_use]
    pub(crate) const fn steps() -> &'static [&'static str; 4] {
        &HOW_TO_SEARCH_STEPS
    }

    /// The example queries of "How to Search".
    #[must_use]
    pub(crate) fn examples(&self) -> &[SearchExampleVm] {
        &self.examples
    }

    /// Tells if the steps show without a disclosure: no saved search is
    /// listed yet (ADR 0089 Decision 4).
    #[must_use]
    pub(crate) fn steps_show_first(&self) -> bool {
        self.rows.is_empty()
    }
}

/// The accessibility label of a saved search row. A row with new results
/// adds their sentence.
fn row_a11y_label(search: &SavedSearch, new_results: Option<&NewResultsText>) -> String {
    let base = format!(
        "{}, {}",
        saved_search_label(&search.query),
        SavedSearch::SUBTITLE
    );
    match new_results {
        Some(text) => format!("{base}, {}", text.sentence),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn saved(id: i64, query: &str) -> SavedSearch {
        SavedSearch {
            id: SavedSearchId(id),
            query: query.to_owned(),
            name: None,
        }
    }

    #[test]
    fn search_page_lists_each_saved_search_with_its_query_in_id_order() {
        let vm = SearchPageVm::new(&[saved(7, "ambient"), saved(3, "bluegrass")], None);
        let labels: Vec<&str> = vm.rows().iter().map(|row| row.label.as_str()).collect();
        assert_eq!(labels, ["bluegrass", "ambient"]);
        assert!(!vm.steps_show_first());
    }

    #[test]
    fn search_page_marks_the_open_saved_search_row() {
        let vm = SearchPageVm::new(
            &[saved(3, "bluegrass"), saved(4, "jazz")],
            Some(SavedSearchId(4)),
        );
        let selected: Vec<&str> = vm
            .rows()
            .iter()
            .filter(|row| row.selected)
            .map(|row| row.label.as_str())
            .collect();
        assert_eq!(selected, ["jazz"]);
    }

    #[test]
    fn search_page_with_no_saved_search_gives_the_steps_and_examples() {
        let vm = SearchPageVm::new(&[], None);
        assert!(vm.rows().is_empty());
        assert!(vm.steps_show_first());
        assert_eq!(SearchPageVm::steps().len(), 4);
        let queries: Vec<&str> = vm.examples().iter().map(|example| example.query).collect();
        let fields: Vec<&str> = QUERY_FIELDS.iter().map(|info| info.example).collect();
        assert_eq!(queries, fields);
    }

    #[test]
    fn search_page_row_shows_the_new_count_text_and_sentence() {
        let counts = BTreeMap::from([(SavedSearchId(3), NewResultCount::Count(3))]);
        let vm = SearchPageVm::new(&[saved(3, "bluegrass"), saved(4, "jazz")], None)
            .with_new_counts(&counts);
        let row = &vm.rows()[0];
        assert_eq!(row.new_results.as_ref().unwrap().count_text, "3 new");
        assert!(row.a11y_label.ends_with("3 new results"));
        assert!(vm.rows()[1].new_results.is_none());
        assert!(!vm.rows()[1].a11y_label.contains("new result"));
    }

    #[test]
    fn search_page_row_shows_no_count_for_zero_or_unavailable() {
        let counts = BTreeMap::from([
            (SavedSearchId(3), NewResultCount::Count(0)),
            (SavedSearchId(4), NewResultCount::Unavailable),
        ]);
        let vm = SearchPageVm::new(&[saved(3, "bluegrass"), saved(4, "jazz")], None)
            .with_new_counts(&counts);
        assert!(vm.rows().iter().all(|row| row.new_results.is_none()));
        assert!(vm
            .rows()
            .iter()
            .all(|row| !row.a11y_label.contains("new result")));
    }

    #[test]
    fn search_page_row_opens_its_saved_search_entry() {
        let vm = SearchPageVm::new(&[saved(9, "jazz")], None);
        assert_eq!(vm.rows()[0].search.id, SavedSearchId(9));
    }
}
