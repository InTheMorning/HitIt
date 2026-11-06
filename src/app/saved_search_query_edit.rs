//! The query field of the saved-search page (ADR 0089 Decision 4, ADR 0087
//! Decisions 4 and 6).
//!
//! A click on the query, or "Edit Query" of the item menu, opens the field.
//! Each change of the field text arms the pause of `Motion::SearchPause`. When
//! the pause ends, the live run of the page runs the typed text. The live run
//! uses the run owner of `SearchPageSlot::SavedSearch`, the same owner as the
//! stored run of the page. So the toolbar run does not conflict with it.
//!
//! "Save Query" and Enter keep the text through the Library writer. Escape and
//! "Cancel" run the stored query again. A Back or a destination change ends
//! the field: the page is dropped, so Back runs the stored query.
//!
//! The field reads no clock and calls no `cx.spawn` (ADR 0040). The timer is
//! `present_after`, as in `search_live.rs`.

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Context, Entity, FontWeight, SharedString, Window,
};
use gpui_component::input::{Escape, Input, InputEvent, InputState};
use gpui_component::Size;

use crate::presentation::delayed_presenter::present_after;
use crate::ui::composites::{ListRow, ListRowA11yLabel};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::primary_selection::PrimarySelectionExt as _;
use crate::ui::primitives::{Button as UiButton, Label};
use crate::ui::sizable_bridge::SizableScaled;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, Motion, SemanticColor};
use crate::view_models::list_page::ListScope;
use crate::view_models::live_run::PauseTicket;
use crate::view_models::search_query::with_library_scope;
use crate::view_models::search_results::{
    saved_search_query_field, SavedSearch, SavedSearchPage, SavedSearchQueryEdit,
    SavedSearchQueryFieldDisplay, SavedSearchQueryKeep, SearchRun,
};
use crate::view_models::workspace::{FrameNavigationEntry, NavigationPathStart};

use super::search_selection::SearchPageSlot;
use super::TopApp;

impl TopApp {
    /// Opens the query field of the mounted saved-search page, with its
    /// stored text, and puts the focus in it. A page with no results opens
    /// no field.
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
        self.saved_search_query_input.update(cx, |input, cx| {
            input.set_value(query, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    /// Writes the scope of the "All | In Library" control into the query
    /// field of the saved-search page (ADR 0089 Decision 4). The field opens
    /// when it is closed. The change starts the run by the path of a typed
    /// change. A page with no results opens no field, so nothing changes.
    pub(super) fn set_saved_search_library_scope(
        &mut self,
        scope: ListScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let is_open = self
            .saved_search_page
            .as_ref()
            .is_some_and(|page| page.edit.is_some());
        if !is_open {
            self.begin_saved_search_query_edit(window, cx);
        }
        let open = self
            .saved_search_page
            .as_ref()
            .is_some_and(|page| page.edit.is_some());
        if !open {
            return;
        }
        let text = self.saved_search_query_input.read(cx).value().to_string();
        let next = with_library_scope(&text, scope);
        if next == text {
            return;
        }
        self.saved_search_query_input
            .update(cx, |input, cx| input.set_value(next, window, cx));
        self.start_saved_search_query_pause(cx);
    }

    /// Opens the saved search `search` over the list, and then its query
    /// field. "Edit Query" and "Duplicate" use it.
    pub(super) fn open_saved_search_query_edit(
        &mut self,
        search: &SavedSearch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_music_page(&search.entry(), NavigationPathStart::ListRow, cx);
        self.begin_saved_search_query_edit(window, cx);
    }

    /// The change of the query field arms the pause. A change while no field
    /// is open arms nothing.
    pub(super) fn start_saved_search_query_pause(&mut self, cx: &mut Context<Self>) {
        if self
            .saved_search_page
            .as_ref()
            .is_none_or(|page| page.edit.is_none())
        {
            return;
        }
        let ticket = self
            .search_runs_mut(SearchPageSlot::SavedSearch)
            .gate
            .text_changed();
        present_after(Motion::SearchPause.duration(), cx, move |this, cx| {
            this.end_saved_search_query_pause(ticket, cx);
        });
    }

    /// Runs the typed text when the pause of `ticket` ends, and `ticket` is
    /// the ticket of the last change.
    fn end_saved_search_query_pause(&mut self, ticket: PauseTicket, cx: &mut Context<Self>) {
        if !self
            .search_runs_mut(SearchPageSlot::SavedSearch)
            .gate
            .pause_ended(ticket)
        {
            return;
        }
        let text = self.saved_search_query_input.read(cx).value().to_string();
        self.run_saved_search_query_text(&text, cx);
    }

    /// Records `text` as the typed text, and runs it on the saved-search
    /// page. A text with no word, phrase or correct field runs nothing, so
    /// the page keeps its results (ADR 0087 Decision 4).
    fn run_saved_search_query_text(&mut self, text: &str, cx: &mut Context<Self>) {
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

    /// "Save Query", or Enter in the field: keeps the typed text (ADR 0087
    /// Decision 6).
    pub(super) fn keep_saved_search_query(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self
            .saved_search_page
            .as_ref()
            .and_then(|page| page.edit.as_ref())
            .map(SavedSearchQueryEdit::id)
        else {
            return;
        };
        let text = self.saved_search_query_input.read(cx).value().to_string();
        self.search_runs_mut(SearchPageSlot::SavedSearch)
            .gate
            .cancel_pause();
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

    /// Applies the outcome of a keep. A saved text closes the field, changes
    /// the stored text, and runs the saved query again. A conflict, a blank
    /// text or an error keeps the field open with its message. A missing row
    /// shows the deleted page.
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

    /// Escape and "Cancel": ends the field, and runs the stored query again,
    /// so the page shows the stored text and its stored results.
    pub(super) fn cancel_saved_search_query(&mut self, cx: &mut Context<Self>) {
        let Some(page) = self.saved_search_page.as_mut() else {
            return;
        };
        if page.edit.take().is_none() {
            return;
        }
        let id = page.id;
        let stored = page.query.clone();
        self.search_runs_mut(SearchPageSlot::SavedSearch)
            .gate
            .cancel_pause();
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

    /// Ends the open field when the current location is another page. A Back
    /// or a destination change cancels the edit (ADR 0089 Decision 4). The
    /// page is dropped, so the next open runs the stored query.
    pub(super) fn discard_saved_search_query_edit_when_left(
        &mut self,
        current: Option<&FrameNavigationEntry>,
    ) {
        let Some(page) = self.saved_search_page.as_ref() else {
            return;
        };
        if page.edit.is_none() {
            return;
        }
        let stays = matches!(
            current,
            Some(FrameNavigationEntry::SavedSearch { id, .. }) if *id == page.id
        );
        if !stays {
            self.saved_search_page = None;
        }
    }

    /// The Enter key of the query field keeps the text.
    pub(super) fn on_saved_search_query_event(
        &mut self,
        _entity: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => self.start_saved_search_query_pause(cx),
            InputEvent::PressEnter { .. } => self.keep_saved_search_query(cx),
            InputEvent::Focus | InputEvent::Blur => {}
        }
    }

    /// The query line of the saved-search page. Closed, it is the stored text
    /// as a button that opens the field. Open, it is the field, "Save Query",
    /// "Cancel" and the message of the last keep. Escape cancels in both
    /// cases where the page has its field.
    pub(super) fn render_saved_search_query_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(page) = self.saved_search_page.as_ref() else {
            return div().into_any_element();
        };
        let Some(edit) = page.edit.as_ref() else {
            let query = page.query.clone();
            return ListRow::new(SharedString::from("music-saved-search-query"))
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
                .into_any_element();
        };
        let field = saved_search_query_field(edit);
        render_open_query_field(&field, &self.saved_search_query_input, cx)
    }
}

/// The open query field: the input, the two controls and the message.
/// Escape in the field cancels the edit.
fn render_open_query_field(
    field: &SavedSearchQueryFieldDisplay,
    input: &Entity<InputState>,
    cx: &mut Context<TopApp>,
) -> AnyElement {
    let entity = cx.entity();
    let save = UiButton::styled(SharedString::from(field.save.id), ControlStyle::Primary)
        .label(field.save.label)
        .a11y_label(field.save.a11y_label.clone())
        .disabled(!field.save.available)
        .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
            this.keep_saved_search_query(cx);
        }));
    let cancel = UiButton::styled(SharedString::from(field.cancel.id), ControlStyle::Primary)
        .label(field.cancel.label)
        .a11y_label(field.cancel.a11y_label.clone())
        .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
            this.cancel_saved_search_query(cx);
        }));
    let row = div()
        .id("music-saved-search-query-field")
        .flex()
        .flex_row()
        .items_center()
        .gap(spacing::SM)
        .px(spacing::MD)
        .capture_action(move |_: &Escape, _window, cx: &mut App| {
            entity.update(cx, TopApp::cancel_saved_search_query);
        })
        .child(
            div().flex_1().min_w_0().child(
                Input::new(input)
                    .cleanable(false)
                    .scaled(Size::Small, cx)
                    .with_primary_selection(input),
            ),
        )
        .child(save)
        .child(cancel);
    let mut column = div()
        .flex()
        .flex_col()
        .gap(spacing::XXS)
        .py(spacing::XXS)
        .child(row);
    if let Some(message) = field.message.as_deref() {
        column = column.child(
            div().px(spacing::MD).child(
                Label::new(message.to_owned())
                    .size(FontSize::Micro)
                    .color(SemanticColor::TertiaryLabel),
            ),
        );
    }
    column.into_any_element()
}
