//! The saved searches of the Library (ADR 0084 Decision 3, packet 012).
//!
//! `LibraryApp` holds the saved searches for the Search page. It reads
//! them at start, and again after each save and each delete. Only
//! `src/db/saved_searches.rs` writes the table.

#![warn(clippy::pedantic)]

use gpui::{Context, Entity, Window};
use gpui_component::input::{InputEvent, InputState};

use crate::view_models::search_page::HowToSearchDisclosure;

use super::LibraryApp;
use crate::db::saved_searches::{
    self, SavedSearchCopy, SavedSearchInsert, SavedSearchWrite, StoredStartTab,
};
use crate::view_models::search_results::{
    SavedSearch, SavedSearchId, SearchResultsTab, SAVED_SEARCH_COPY_CONFLICT_TEXT,
    SAVED_SEARCH_MISSING_TEXT,
};
use crate::view_models::workspace::{FrameNavigationEntry, NavigationPathStart};

impl LibraryApp {
    /// The saved searches, in `id` order.
    pub(crate) fn saved_searches(&self) -> &[SavedSearch] {
        &self.saved_searches
    }

    /// Reads the saved searches again. A read failure keeps the present
    /// list and shows the error in the Library status.
    pub(crate) fn reload_saved_searches(&mut self) {
        let read = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::saved_searches(&conn)
        };
        match read {
            Ok(stored) => {
                self.saved_searches = stored.into_iter().map(SavedSearch::from_stored).collect();
            }
            Err(error) => self.vm.set_error_status(format!(
                "The app could not read the saved searches: {error:#}"
            )),
        }
    }

    /// Saves `query` with the tab `start_tab`. `created_at` is the time of
    /// the click, in Unix seconds. A query that is already saved changes
    /// nothing. The error text names the database error.
    pub(crate) fn save_search(
        &mut self,
        query: &str,
        start_tab: SearchResultsTab,
        created_at: i64,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let inserted = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::insert_saved_search(
                &conn,
                query,
                StoredStartTab::from(start_tab),
                created_at,
            )
        };
        let result = match inserted {
            Ok(SavedSearchInsert::Inserted(_) | SavedSearchInsert::AlreadySaved) => Ok(()),
            Err(error) => Err(format!("{error:#}")),
        };
        self.reload_saved_searches();
        cx.notify();
        result
    }

    /// Deletes the saved search `id`. The Library and the downloads do not
    /// change. The error text names the database error.
    pub(crate) fn delete_saved_search(
        &mut self,
        id: SavedSearchId,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let deleted = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::delete_saved_search(&conn, id.0)
        };
        self.reload_saved_searches();
        cx.notify();
        deleted.map(|_| ()).map_err(|error| format!("{error:#}"))
    }

    /// Opens a saved-search row directly under Search (ADR 0084
    /// Decisions 3 and 9), as `open_discover_view` does.
    pub(crate) fn open_saved_search_row(&mut self, search: &SavedSearch, cx: &mut Context<Self>) {
        self.open_page(search.entry(), NavigationPathStart::ListRow, cx);
    }

    /// Shows the rename input in the row of `search`, with its current text.
    pub(crate) fn begin_saved_search_rename(
        &mut self,
        search: &SavedSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current = search.display_label().to_owned();
        self.saved_search_renaming = Some(search.id);
        self.saved_search_message = None;
        self.saved_search_rename_input.update(cx, |input, cx| {
            input.set_value(current, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    /// Saves the name in the rename input (ADR 0089 Decision 4). A blank
    /// name clears the name. A saved search that no longer exists gives the
    /// missing text. A database error keeps the input open.
    pub(crate) fn submit_saved_search_rename(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.saved_search_renaming else {
            return;
        };
        let name = self.saved_search_rename_input.read(cx).value().to_string();
        let written = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::rename_saved_search(&conn, id.0, &name)
        };
        match written {
            Ok(SavedSearchWrite::Written) => {
                self.saved_search_renaming = None;
                self.saved_search_message = None;
                self.reload_saved_searches();
            }
            Ok(SavedSearchWrite::Missing) => {
                self.saved_search_renaming = None;
                self.saved_search_message = Some(SAVED_SEARCH_MISSING_TEXT.to_owned());
                self.reload_saved_searches();
            }
            Err(error) => {
                self.saved_search_message = Some(format!(
                    "The app could not rename the saved search: {error:#}"
                ));
            }
        }
        cx.notify();
    }

    /// Ends the rename of a saved search row without a change.
    pub(crate) fn cancel_saved_search_rename(&mut self, cx: &mut Context<Self>) {
        self.saved_search_renaming = None;
        self.saved_search_message = None;
        cx.notify();
    }

    /// The Enter key of the rename input saves the name.
    pub(crate) fn on_saved_search_rename_event(
        &mut self,
        _entity: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::PressEnter { .. } = event {
            self.submit_saved_search_rename(cx);
        }
    }

    /// Copies the saved search `id` as a new row, in place (ADR 0089
    /// Decision 4). The new row takes the name of the original with " copy".
    /// A copy whose text is saved already gives the conflict text.
    pub(crate) fn duplicate_saved_search_row(&mut self, id: SavedSearchId, cx: &mut Context<Self>) {
        let created_at = chrono::Utc::now().timestamp();
        let copied = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::duplicate_saved_search(&conn, id.0, created_at)
        };
        self.saved_search_message = match copied {
            Ok(SavedSearchCopy::Copied(_)) => None,
            Ok(SavedSearchCopy::AlreadySaved) => Some(SAVED_SEARCH_COPY_CONFLICT_TEXT.to_owned()),
            Ok(SavedSearchCopy::Missing) => Some(SAVED_SEARCH_MISSING_TEXT.to_owned()),
            Err(error) => Some(format!(
                "The app could not duplicate the saved search: {error:#}"
            )),
        };
        self.reload_saved_searches();
        cx.notify();
    }

    /// Shows or hides the steps of "How to Search" (ADR 0089 Decision 4).
    pub(crate) fn toggle_how_to_search(&mut self, cx: &mut Context<Self>) {
        self.how_to_search = match self.how_to_search {
            HowToSearchDisclosure::Hidden => HowToSearchDisclosure::Shown,
            HowToSearchDisclosure::Shown => HowToSearchDisclosure::Hidden,
        };
        cx.notify();
    }

    /// Sets the saved search of the current location, for the open row of
    /// the Search page.
    pub(crate) fn set_open_saved_search(&mut self, id: Option<SavedSearchId>) {
        self.open_saved_search = id;
    }

    /// Sets the page that Back left open over the current list root, for
    /// the marked row of the Music list (R-NAV-2 item 2).
    pub(crate) fn set_marked_entry(&mut self, entry: Option<FrameNavigationEntry>) {
        self.marked_entry = entry;
    }
}
