//! The Live list of the Index on the Library side (ADR 0089 Decision 1,
//! packet 011).
//!
//! The list reads `GET /v1/live-items` with `view=now` through
//! `present_command` on the ADR 0040 runtime. Each read of the page starts
//! again, and the rows of the last answer stay on view until the new answer
//! comes. A result of an older generation changes nothing.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{div, prelude::*, AnyElement, Context, Window};

use super::app_impl::command_error_detail;
use super::LibraryApp;
use crate::application::queries::live_page::FetchLivePage;
use crate::application::CommandContext;
use crate::presentation::collection_art::collection_art_image;
use crate::presentation::present_command;
use crate::ui::composites::list_page_head::{list_page_head, ListPageHeadHandlers};
use crate::ui::composites::ThumbnailSize;
use crate::ui::shells::library::destination_lists::render_library_destination_rows;
use crate::ui::shells::library::index_list_status::IndexListActions;
use crate::ui::shells::library::live_page::{render_live_empty, render_live_list_item};
use crate::view_models::index_name_list::IndexNameListLoadIntent;
use crate::view_models::list_page::ListPageKey;
use crate::view_models::live_page::{LiveListItemVm, LivePageVm};

impl LibraryApp {
    /// Starts a read of page 1 of the Live list (ADR 0089 packet 011). It
    /// runs when the Live destination shows, and when a page that the Live
    /// list opened goes back to it. The loaded rows stay on view until the
    /// answer.
    pub(crate) fn reload_live_list(&mut self, cx: &mut Context<Self>) {
        let intent = self.live_list.begin_reload();
        self.send_live_request(Some(intent), cx);
    }

    /// "Load more" at the end of the Live list.
    pub(crate) fn load_more_live(&mut self, cx: &mut Context<Self>) {
        let intent = self.live_list.begin_load(true);
        self.send_live_request(intent, cx);
    }

    /// "Try again" on the error row of the Live list.
    pub(crate) fn retry_live(&mut self, cx: &mut Context<Self>) {
        let intent = self.live_list.begin_retry();
        self.send_live_request(intent, cx);
    }

    /// Sends the request of `intent`. A result of an older generation changes
    /// nothing.
    fn send_live_request(
        &mut self,
        intent: Option<IndexNameListLoadIntent>,
        cx: &mut Context<Self>,
    ) {
        let Some(IndexNameListLoadIntent {
            cursor, generation, ..
        }) = intent
        else {
            return;
        };
        let command = FetchLivePage::new(self.musicindex_endpoint.clone(), cursor);
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, batch, cx| {
                if this.live_list.finish_load(batch, generation) {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if this
                    .live_list
                    .fail_load(command_error_detail(error), generation)
                {
                    cx.notify();
                }
            },
        );
    }

    /// The title row of the Live list (ADR 0089 Decision 6). The list has no
    /// scope, no sort and no view control.
    pub(crate) fn render_live_page_head(&self, cx: &mut Context<Self>) -> AnyElement {
        let display = self.vm.list_page_head(ListPageKey::Live, None, None);
        list_page_head(
            display,
            ListPageHeadHandlers {
                scope: None,
                select: None,
                sort: None,
                view: None,
                letter: None,
                letter_keys: None,
                actions: Vec::new(),
            },
            cx,
        )
    }

    /// Renders the rows of the Live list, with its status rows, or the empty
    /// text when the Index has no live item (ADR 0089 packet 011).
    pub(crate) fn render_live_pane(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let page = LivePageVm::of(&self.live_list, self.marked_entry.as_ref());
        if let Some(empty) = page.empty {
            return render_live_empty(empty);
        }
        let items = Rc::new(page.items);
        let ids: Vec<String> = items
            .iter()
            .map(|item| item.element_id().to_owned())
            .collect();
        self.live_list_state.sync(&ids, window);
        let item_count = items.len();
        render_library_destination_rows(
            &self.live_list_state,
            item_count,
            cx.processor(move |this, index: usize, _window, cx| {
                this.render_live_item(&items, index, cx)
            }),
            cx,
        )
    }

    /// Renders the item `index` of the Live `items`. A row reads its feed
    /// art only when it renders.
    fn render_live_item(
        &mut self,
        items: &[LiveListItemVm],
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(item) = items.get(index) else {
            return div().into_any_element();
        };
        let art = match item {
            LiveListItemVm::Row(row) => row
                .row
                .art
                .as_ref()
                .map(|art| collection_art_image(self, art, ThumbnailSize::Sm, cx)),
            LiveListItemVm::Status(_) => None,
        };
        render_live_list_item(item.clone(), art, Self::live_actions(), cx)
    }

    /// The "Try again" and "Load more" actions of the Live status rows.
    fn live_actions() -> IndexListActions {
        IndexListActions {
            retry: Self::retry_live,
            load_more: Some(Self::load_more_live),
        }
    }
}
