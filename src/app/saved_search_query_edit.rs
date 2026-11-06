//! The query edit of the saved-search page (ADR 0089 Decision 4, ADR 0087
//! Decisions 3, 4 and 6).
//!
//! "Edit Query" of the item menu, and a click on the query, put the saved
//! query in the toolbar search field and give that field the focus. While
//! the edit is open, each change of the toolbar text runs the live query of
//! the page through the toolbar path (`end_search_pause`). The run uses the
//! saved-search slot, so the page keeps its place.
//!
//! "Save Query" and Enter keep the typed text through the Library writer.
//! "Cancel" and Escape restore the stored text and run it again. Leaving the
//! page ends the edit with no save. A new search ends it too, and the page
//! is then mounted again on its next open.
//!
//! The field reads no clock and calls no `cx.spawn` (ADR 0040).

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, SharedString, Window};

use crate::ui::composites::{ListRow, ListRowA11yLabel};
use crate::ui::primitives::Label;
use crate::ui::style::spacing;
use crate::ui::tokens::SemanticColor;
use crate::view_models::list_page::ListScope;
use crate::view_models::search_results::{
    SavedSearch, SavedSearchHeaderAction, SavedSearchPage, SavedSearchQueryEdit,
    SavedSearchQueryKeep, SearchRun,
};
use crate::view_models::workspace::{FrameNavigationEntry, NavigationPathStart};

use super::search_selection::SearchPageSlot;
use super::TopApp;

impl TopApp {
    /// Opens the query edit of the mounted saved-search page. The stored
    /// text goes into the toolbar field, which takes the focus. A page with
    /// no results opens no edit.
    pub(super) fn begin_saved_search_query_edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = self.saved_search_page.as_mut() else {
            return;
        };
        if !page.begin_query_edit() {
            return;
        }
        let query = page.query.clone();
        self.global_search_input.update(cx, |input, cx| {
            input.set_value(query, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    /// Tells if the mounted saved-search page has its query edit open.
    #[must_use]
    pub(super) fn saved_search_query_editing(&self) -> bool {
        self.saved_search_page
            .as_ref()
            .is_some_and(|page| page.edit.is_some())
    }

    /// Writes the scope of the "All | In Library" control into the toolbar
    /// field (ADR 0089 Decision 4). The edit opens when it is closed. The
    /// change then runs by the path of a typed change.
    pub(super) fn set_saved_search_library_scope(
        &mut self,
        scope: ListScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.saved_search_query_editing() {
            self.begin_saved_search_query_edit(window, cx);
        }
        if !self.saved_search_query_editing() {
            return;
        }
        self.set_toolbar_library_scope(scope, window, cx);
    }

    /// Opens the saved search `search` over the list, and then its query
    /// edit. "Edit Query" and "Duplicate" use it.
    pub(super) fn open_saved_search_query_edit(
        &mut self,
        search: &SavedSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_music_page(&search.entry(), NavigationPathStart::ListRow, cx);
        self.begin_saved_search_query_edit(window, cx);
    }

    /// Runs `text`, the typed text of the toolbar field, as the live query of
    /// the saved-search page. A text with no word, phrase or correct field
    /// runs nothing, so the page keeps its results (ADR 0087 Decision 4).
    pub(super) fn run_saved_search_query_text(&mut self, text: &str, cx: &mut Context<Self>) {
        let Some(edit) = self
            .saved_search_page
            .as_mut()
            .and_then(|page| page.edit.as_mut())
        else {
            return;
        };
        edit.set_typed(text);
        if !SearchRun::new(text).is_runnable() {
            cx.notify();
            return;
        }
        let run = self.next_search_run(SearchPageSlot::SavedSearch, text);
        let results = self
            .search_results_detail_for_run(&run, cx)
            .with_run_generation(run.generation());
        if let Some(page) = self.saved_search_page.as_mut() {
            page.set_results(results);
        }
        self.start_index_search_for_run(SearchPageSlot::SavedSearch, &run, cx);
        cx.notify();
    }

    /// "Save Query", or Enter in the toolbar field while the edit is open:
    /// keeps the typed text (ADR 0087 Decision 6).
    pub(super) fn keep_saved_search_query(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self
            .saved_search_page
            .as_ref()
            .and_then(|page| page.edit.as_ref())
            .map(SavedSearchQueryEdit::id)
        else {
            return;
        };
        let text = self.global_search_input.read(cx).value().to_string();
        self.toolbar_search_runs.gate.cancel_pause();
        if let Some(edit) = self
            .saved_search_page
            .as_mut()
            .and_then(|page| page.edit.as_mut())
        {
            edit.set_typed(&text);
        }
        let keep = self.library.update(cx, |library, cx| {
            library.save_saved_search_query(id, &text, cx)
        });
        self.apply_saved_search_query_keep(keep, cx);
    }

    /// Applies the outcome of a keep. A saved text ends the edit, changes the
    /// stored text, and runs the saved query again. A conflict, a blank text
    /// or an error keeps the edit open with its message in the title row. A
    /// missing row shows the deleted page.
    fn apply_saved_search_query_keep(
        &mut self,
        keep: SavedSearchQueryKeep,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = self.saved_search_page.as_mut() else {
            return;
        };
        let id = page.id;
        let Some(edit) = page.edit.as_mut() else {
            return;
        };
        let missing = keep == SavedSearchQueryKeep::Missing;
        if !edit.apply_keep(keep) && !missing {
            cx.notify();
            return;
        }
        let stored = edit.stored().to_owned();
        page.edit = None;
        page.query.clone_from(&stored);
        if missing {
            self.saved_search_page = Some(SavedSearchPage::deleted(id, stored));
            cx.notify();
            return;
        }
        // Each history entry of the saved search takes the saved text, so
        // Back and Forward resolve the saved search by its id.
        if let Some(content_list_id) = self.content_list_frame_id() {
            if let Some(nav) = self.workspace_layout.frame_nav_mut(content_list_id) {
                nav.rename_saved_search_entries(id, &stored);
            }
        }
        let search = self
            .library
            .read(cx)
            .saved_searches()
            .iter()
            .find(|search| search.id == id)
            .cloned();
        match search {
            Some(search) => self.run_saved_search(&search, cx),
            None => self.saved_search_page = Some(SavedSearchPage::deleted(id, stored)),
        }
        cx.notify();
    }

    /// "Cancel", or Escape in the toolbar field while the edit is open: ends
    /// the edit, restores the stored text in the toolbar field, and runs the
    /// stored query again, so the page shows its stored results.
    pub(super) fn cancel_saved_search_query(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = self.saved_search_page.as_mut() else {
            return;
        };
        if !page.cancel_query_edit() {
            return;
        }
        let id = page.id;
        let stored = page.query.clone();
        self.toolbar_search_runs.gate.cancel_pause();
        self.global_search_input.update(cx, |input, cx| {
            input.set_value(stored.clone(), window, cx);
        });
        let search = self
            .library
            .read(cx)
            .saved_searches()
            .iter()
            .find(|search| search.id == id)
            .cloned();
        match search {
            Some(search) => self.run_saved_search(&search, cx),
            None => self.saved_search_page = Some(SavedSearchPage::deleted(id, stored)),
        }
        cx.notify();
    }

    /// Ends an open query edit for a new search (ADR 0089 Decision 4). The
    /// new search sets the toolbar text itself, and the page is dropped, so
    /// its next open runs the stored query and shows no typed results.
    pub(super) fn end_saved_search_query_edit_for_new_search(&mut self, cx: &mut Context<Self>) {
        let ends = self
            .saved_search_page
            .as_ref()
            .is_some_and(SavedSearchPage::query_edit_ends_for_new_search);
        if !ends {
            return;
        }
        self.toolbar_search_runs.gate.cancel_pause();
        self.saved_search_page = None;
        cx.notify();
    }

    /// Ends an open query edit when the current location is another page.
    /// A Back, a destination change or a sidebar click ends the edit with no
    /// save. The toolbar field gets the stored text back, so it does not show
    /// the query that was not kept. A move to a toolbar search page keeps the
    /// text that the search gives.
    pub(super) fn discard_saved_search_query_edit_when_left(
        &mut self,
        current: Option<&FrameNavigationEntry>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = self.saved_search_page.as_ref() else {
            return;
        };
        if !page.query_edit_ends_when_left(current) {
            return;
        }
        let stored = page.query.clone();
        let restores_toolbar = !matches!(current, Some(FrameNavigationEntry::Search(_)));
        self.toolbar_search_runs.gate.cancel_pause();
        self.saved_search_page = None;
        if restores_toolbar {
            self.global_search_input.update(cx, |input, cx| {
                input.set_value(stored, window, cx);
            });
        }
    }

    /// Runs one action of the title row of the saved-search page (ADR 0089
    /// Decision 4).
    pub(super) fn run_saved_search_header_action(
        &mut self,
        action: SavedSearchHeaderAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            SavedSearchHeaderAction::Delete => self.request_saved_search_delete(window, cx),
            SavedSearchHeaderAction::SaveQuery => self.keep_saved_search_query(cx),
            SavedSearchHeaderAction::CancelQuery => self.cancel_saved_search_query(window, cx),
        }
    }

    /// The query line of the saved-search page while no edit is open: the
    /// stored text, as a row that opens the edit. An open edit shows its
    /// controls in the title row, so this line is absent.
    pub(super) fn render_saved_search_query_bar(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let page = self.saved_search_page.as_ref()?;
        if page.edit.is_some() {
            return None;
        }
        let query = page.query.clone();
        Some(
            ListRow::new(SharedString::from("music-saved-search-query"))
                .a11y_label(ListRowA11yLabel {
                    label: SharedString::from(format!(
                        "Edit the query \"{query}\" of this saved search"
                    )),
                })
                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                    this.begin_saved_search_query_edit(window, cx);
                }))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .pl(spacing::MD)
                        .child(
                            Label::new(query)
                                .color(SemanticColor::Label)
                                .weight(FontWeight::MEDIUM),
                        ),
                )
                .into_any_element(),
        )
    }
}
