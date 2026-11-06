//! The saved searches of the Library (ADR 0084 Decision 3, packet 012).
//!
//! `LibraryApp` holds the saved searches for the Search page. It reads
//! them at start, and again after each save and each delete. Only
//! `src/db/saved_searches.rs` writes the table.

#![warn(clippy::pedantic)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;

use gpui::{Context, Entity, Window};
use gpui_component::input::{InputEvent, InputState};

use crate::view_models::search_page::HowToSearchDisclosure;

use super::LibraryApp;
use crate::db::saved_search_seen;
use crate::db::saved_searches::{
    self, SavedSearchCopy, SavedSearchQueryWrite, SavedSearchWrite, StoredStartTab,
};
use crate::presentation::bridge_watch;
use crate::runtime::saved_search_counts::{
    self, count_run_due, SavedSearchCountJob, SavedSearchCountRequest, SavedSearchCountsSnapshot,
};
use crate::view_models::search_results::{
    is_query_saved, NewResultCount, SavedSearch, SavedSearchId, SavedSearchQueryKeep,
    SearchResultsTab, SAVED_SEARCH_MISSING_TEXT, SAVED_SEARCH_NAME_TAKEN_TEXT,
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
    /// the click, in Unix seconds. A query that a saved search holds already
    /// changes nothing, so one click cannot save a copy (ADR 0084 Decision
    /// 3). The table itself accepts a repeated query. The error text names
    /// the database error.
    pub(crate) fn save_search(
        &mut self,
        query: &str,
        start_tab: SearchResultsTab,
        created_at: i64,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if is_query_saved(&self.saved_searches, query) {
            return Ok(());
        }
        let inserted = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::insert_saved_search(
                &conn,
                query,
                StoredStartTab::from(start_tab),
                created_at,
            )
        };
        let result = inserted.map(|_| ()).map_err(|error| format!("{error:#}"));
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
        let current = search.name.clone();
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
            Ok(SavedSearchWrite::NameTaken) => {
                // The input stays open, so the curator can choose another name.
                self.saved_search_message = Some(SAVED_SEARCH_NAME_TAKEN_TEXT.to_owned());
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
    /// Decision 4). The new row keeps the query, and it takes the name of the
    /// original with " copy", or its next free number. Gives the id of the
    /// new row, so the caller can open it.
    pub(crate) fn duplicate_saved_search_row(
        &mut self,
        id: SavedSearchId,
        cx: &mut Context<Self>,
    ) -> Option<SavedSearchId> {
        let created_at = chrono::Utc::now().timestamp();
        let copied = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::duplicate_saved_search(&conn, id.0, created_at)
        };
        let copy_id = match copied {
            Ok(SavedSearchCopy::Copied(copy_id)) => {
                self.saved_search_message = None;
                Some(SavedSearchId(copy_id))
            }
            Ok(SavedSearchCopy::Missing) => {
                self.saved_search_message = Some(SAVED_SEARCH_MISSING_TEXT.to_owned());
                None
            }
            Err(error) => {
                self.saved_search_message = Some(format!(
                    "The app could not duplicate the saved search: {error:#}"
                ));
                None
            }
        };
        self.reload_saved_searches();
        cx.notify();
        copy_id
    }

    /// Saves `query` as the text of the saved search `id` (ADR 0087 Decision
    /// 6). The text is trimmed. Gives the typed result of the writer, so the
    /// caller shows a conflict or a blank text. The error text names the
    /// database error.
    pub(crate) fn save_saved_search_query(
        &mut self,
        id: SavedSearchId,
        query: &str,
        cx: &mut Context<Self>,
    ) -> SavedSearchQueryKeep {
        let written = {
            let conn = self.conn.lock().expect("lock db");
            saved_searches::update_saved_search_query(&conn, id.0, query)
        };
        let keep = match written {
            Ok(SavedSearchQueryWrite::Written) => {
                SavedSearchQueryKeep::Saved(query.trim().to_owned())
            }
            Ok(SavedSearchQueryWrite::Blank) => SavedSearchQueryKeep::Blank,
            Ok(SavedSearchQueryWrite::Missing) => SavedSearchQueryKeep::Missing,
            Err(error) => SavedSearchQueryKeep::Failed(format!("{error:#}")),
        };
        self.reload_saved_searches();
        cx.notify();
        keep
    }

    /// Starts the count of new results of every saved search (ADR 0089
    /// Decision 4, packet 007 Part C). The actor starts at the first call.
    /// A call within the count interval of the last request sends nothing.
    pub(crate) fn request_saved_search_counts(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        if self.saved_searches.is_empty()
            || !count_run_due(self.saved_search_counts_requested_at, now)
        {
            return;
        }
        let Some(host) = self.runtime_host.clone() else {
            return;
        };
        if self.saved_search_counts_handle.is_none() {
            let db_path = self
                .conn
                .lock()
                .ok()
                .and_then(|conn| conn.path().map(PathBuf::from));
            let Some(db_path) = db_path else {
                return;
            };
            let conn = match crate::db::startup::open_existing(&db_path) {
                Ok(conn) => conn,
                Err(error) => {
                    self.vm.set_error_status(format!(
                        "The app could not open the database for the saved search counts: {}",
                        error.reason
                    ));
                    return;
                }
            };
            let _enter = host.handle().enter();
            let handle = saved_search_counts::spawn(
                conn,
                self.musicindex_endpoint.clone(),
                self.command_runner.session(),
            );
            bridge_watch(
                handle.subscribe(),
                |this: &mut Self, snapshot: SavedSearchCountsSnapshot, cx| {
                    this.saved_search_counts = snapshot.counts;
                    cx.notify();
                },
                cx,
            );
            self.saved_search_counts_handle = Some(handle);
        }
        let request = SavedSearchCountRequest {
            jobs: self
                .saved_searches
                .iter()
                .map(|search| SavedSearchCountJob {
                    id: search.id,
                    query: search.query.clone(),
                })
                .collect(),
            library: self.vm.library_search_facts(),
            artists: self.artist_search_facts(),
        };
        let sent = self
            .saved_search_counts_handle
            .as_ref()
            .is_some_and(|handle| handle.request(request));
        if sent {
            self.saved_search_counts_requested_at = Some(now);
        }
    }

    /// Stores the result keys `keys` of the saved search `id` as seen (ADR
    /// 0089 Decision 4, packet 007 Part C). An open of the saved search and
    /// each Index result of it call this. The count of the search becomes
    /// zero, in place. The error text names the database error.
    pub(crate) fn mark_saved_search_seen(
        &mut self,
        id: SavedSearchId,
        keys: &BTreeSet<String>,
        cx: &mut Context<Self>,
    ) {
        let written = {
            let conn = self.conn.lock().expect("lock db");
            saved_search_seen::mark_seen(&conn, id.0, keys)
        };
        match written {
            Ok(()) => {
                self.saved_search_counts
                    .insert(id, NewResultCount::Count(0));
            }
            Err(error) => self.vm.set_error_status(format!(
                "The app could not mark the results of the saved search as seen: {error:#}"
            )),
        }
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
