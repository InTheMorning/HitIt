//! The Public playlists of the Index on the Library side (ADR 0089
//! Decision 1, Decision 5 and packet 009).
//!
//! The list of the public playlists reads `GET /v1/feeds/recent` in the
//! sort Newest, and `GET /v1/feeds/by-name` in the sort A to Z, both with
//! `medium=musicL`. The item page reads one playlist with its remote items.
//! Each request runs through `present_command` on the ADR 0040 runtime. A
//! result of an older generation changes nothing.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{div, prelude::*, AnyElement, Context, Window};

use super::app_impl::command_error_detail;
use super::LibraryApp;
use crate::application::queries::public_playlists::{
    FetchPublicPlaylist, FetchPublicPlaylistPage, PublicPlaylistLoad,
};
use crate::application::CommandContext;
use crate::presentation::collection_art::collection_art_image;
use crate::presentation::present_command;
use crate::ui::composites::letter_bar::LetterHandler;
use crate::ui::composites::list_page_head::{
    list_page_head, ListPageHeadHandlers, ListSortHandler, ListViewHandler,
};
use crate::ui::composites::ThumbnailSize;
use crate::ui::shells::library::destination_lists::{
    render_library_destination_rows, render_public_playlist_list_item,
};
use crate::ui::shells::library::index_list_status::{
    render_index_list_status_row, IndexListActions,
};
use crate::ui::shells::library::public_playlists::{
    render_public_playlist_failed, render_public_playlist_item, render_public_playlist_loading,
    render_public_playlist_page_body, render_public_playlist_tile,
};
use crate::ui::shells::library::releases_tiles::render_tile_grid;
use crate::view_models::index_name_list::{IndexListStatusRowVm, IndexNameListLoadIntent};
use crate::view_models::list_page::{ListPageKey, ListSort, ListView};
use crate::view_models::public_playlists::{
    public_playlist_list_items, PublicPlaylistListItemVm, PublicPlaylistLoadState,
};

impl LibraryApp {
    /// The sort of the Public playlists list (ADR 0089 Decision 5). The
    /// page starts at Newest.
    #[must_use]
    pub(crate) fn public_playlist_sort(&self) -> ListSort {
        self.vm
            .list_sort(ListPageKey::PublicPlaylists)
            .unwrap_or(ListSort::Newest)
    }

    /// Sends page 1 of the Public playlists list, once, when the
    /// destination opens (ADR 0089 Decision 5). The rows show after the
    /// answer, and the loading row shows before it.
    pub(crate) fn request_public_playlists_first_page(&mut self, cx: &mut Context<Self>) {
        if !self.public_playlist_list.is_not_started() {
            return;
        }
        let intent = self.public_playlist_list.begin_load(false);
        self.send_public_playlist_request(intent, cx);
    }

    /// Starts the Public playlists list again at `from`, the first row of a
    /// letter or `None` for the first row. A new sort starts the list at its
    /// first row (ADR 0089 Decision 5).
    pub(crate) fn restart_public_playlist_list(
        &mut self,
        from: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let intent = self.public_playlist_list.begin_from(from);
        self.send_public_playlist_request(intent, cx);
    }

    /// "Load more" at the end of the Public playlists list (ADR 0084 packet
    /// 016 decision D7).
    pub(crate) fn load_more_public_playlists(&mut self, cx: &mut Context<Self>) {
        let intent = self.public_playlist_list.begin_load(true);
        self.send_public_playlist_request(intent, cx);
    }

    /// "Try again" on the error row of the Public playlists list (ADR 0084
    /// packet 016 decision D8).
    pub(crate) fn retry_public_playlists(&mut self, cx: &mut Context<Self>) {
        let intent = self.public_playlist_list.begin_retry();
        self.send_public_playlist_request(intent, cx);
    }

    /// Sends the request of `intent` in the current sort. The sort of the
    /// request is the sort of the page now, and a result of an older
    /// generation changes nothing.
    fn send_public_playlist_request(
        &mut self,
        intent: Option<IndexNameListLoadIntent>,
        cx: &mut Context<Self>,
    ) {
        let Some(IndexNameListLoadIntent {
            cursor,
            generation,
            from,
        }) = intent
        else {
            return;
        };
        let sort = self.public_playlist_sort();
        // Only the sort A to Z reads a `from` text. The sort Newest reads
        // the recent feeds, which take none.
        let from = match sort {
            ListSort::AToZ => from,
            ListSort::Newest => None,
        };
        let command =
            FetchPublicPlaylistPage::new(self.musicindex_endpoint.clone(), sort, cursor, from);
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, batch, cx| {
                if this.public_playlist_list.finish_load(batch, generation) {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if this
                    .public_playlist_list
                    .fail_load(command_error_detail(error), generation)
                {
                    cx.notify();
                }
            },
        );
    }

    /// Opens the item page of the public playlist `feed_guid`: it reads the
    /// playlist with its remote items, unless the page holds it already
    /// (ADR 0089 packet 009). A failed read starts again.
    pub(crate) fn request_public_playlist(&mut self, feed_guid: &str, cx: &mut Context<Self>) {
        let Some(generation) = self.public_playlist_pages.begin(feed_guid) else {
            return;
        };
        cx.notify();
        let requested = feed_guid.to_owned();
        let failed = requested.clone();
        present_command(
            &self.command_runner,
            FetchPublicPlaylist::new(self.musicindex_endpoint.clone(), requested),
            CommandContext::next(),
            cx,
            move |this, load: PublicPlaylistLoad, cx| {
                if this
                    .public_playlist_pages
                    .finish(&load.feed_guid, &load.feed, generation)
                {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if this
                    .public_playlist_pages
                    .fail(&failed, command_error_detail(error), generation)
                {
                    cx.notify();
                }
            },
        );
    }

    /// "Try again" on the error state of the open playlist page (ADR 0089
    /// packet 009). It reads the playlist of the page again.
    pub(crate) fn retry_public_playlist(&mut self, cx: &mut Context<Self>) {
        if let Some(feed_guid) = self.public_playlist_pages.feed_guid().map(str::to_owned) {
            self.request_public_playlist(&feed_guid, cx);
        }
    }

    /// The loaded title of the public playlist `feed_guid`, or `None` before
    /// it loads (ADR 0089 Decision 9).
    #[must_use]
    pub(crate) fn public_playlist_title(&self, feed_guid: &str) -> Option<String> {
        self.public_playlist_pages
            .loaded_title(feed_guid)
            .map(str::to_owned)
    }

    /// Renders the title row of the Public playlists list (ADR 0089
    /// Decision 6). The list has no scope and no select action.
    pub(crate) fn render_public_playlists_page_head(&self, cx: &mut Context<Self>) -> AnyElement {
        let display = self
            .vm
            .list_page_head(ListPageKey::PublicPlaylists, None, None);
        let entity = cx.entity();
        let sort_entity = entity.clone();
        let on_sort: ListSortHandler = Rc::new(move |sort, _window, cx| {
            let sort = *sort;
            sort_entity.update(cx, |this, cx| {
                this.set_list_sort(ListPageKey::PublicPlaylists, sort, cx);
            });
        });
        let letter_entity = entity.clone();
        let on_letter: LetterHandler = Rc::new(move |letter, _window, cx| {
            let letter = *letter;
            letter_entity.update(cx, |this, cx| {
                this.set_list_letter(ListPageKey::PublicPlaylists, Some(letter), cx);
            });
        });
        let view_entity = entity;
        let on_view: ListViewHandler = Rc::new(move |view, _window, cx| {
            let view = *view;
            view_entity.update(cx, |this, cx| {
                this.set_list_view(ListPageKey::PublicPlaylists, view, cx);
            });
        });
        list_page_head(
            display,
            ListPageHeadHandlers {
                scope: None,
                select: None,
                sort: Some(on_sort),
                view: Some(on_view),
                letter: Some(on_letter),
                action: None,
            },
            cx,
        )
    }

    /// Renders the rows or the tiles of the Public playlists list, with its
    /// status rows (ADR 0089 packet 009).
    pub(crate) fn render_public_playlists_pane(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let items = Rc::new(public_playlist_list_items(
            &self.public_playlist_list,
            self.marked_entry.as_ref(),
        ));
        match self.vm.list_view(ListPageKey::PublicPlaylists) {
            ListView::Rows => {
                let ids: Vec<String> = items
                    .iter()
                    .map(|item| item.element_id().to_owned())
                    .collect();
                self.public_playlists_list_state.sync(&ids, window);
                let item_count = items.len();
                render_library_destination_rows(
                    &self.public_playlists_list_state,
                    item_count,
                    cx.processor(move |this, index: usize, _window, cx| {
                        this.render_public_playlists_row(&items, index, cx)
                    }),
                    cx,
                )
            }
            ListView::Tiles => self.render_public_playlists_tiles(&items, cx),
        }
    }

    /// Renders the item `index` of the Public playlists `items`. The row
    /// reads its art only when it renders.
    fn render_public_playlists_row(
        &mut self,
        items: &[PublicPlaylistListItemVm],
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(item) = items.get(index) else {
            return div().into_any_element();
        };
        let art = match item {
            PublicPlaylistListItemVm::Row(row) => row
                .art
                .as_ref()
                .map(|art| collection_art_image(self, art, ThumbnailSize::Sm, cx)),
            PublicPlaylistListItemVm::Status(_) => None,
        };
        render_public_playlist_list_item(item.clone(), art, Self::public_playlist_actions(), cx)
    }

    /// The "Try again" and "Load more" actions of the Public playlists
    /// status rows.
    fn public_playlist_actions() -> IndexListActions {
        IndexListActions {
            retry: Self::retry_public_playlists,
            load_more: Some(Self::load_more_public_playlists),
        }
    }

    /// Renders the Tiles view of the Public playlists list (ADR 0089 packet
    /// 009). The status rows keep their row at the start or at the end.
    fn render_public_playlists_tiles(
        &mut self,
        items: &[PublicPlaylistListItemVm],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut leading = Vec::new();
        let mut tiles = Vec::new();
        let mut trailing = Vec::new();
        let mut seen_row = false;
        for item in items {
            match item {
                PublicPlaylistListItemVm::Row(row) => {
                    seen_row = true;
                    let art = row
                        .art
                        .as_ref()
                        .map(|art| collection_art_image(self, art, ThumbnailSize::Xl, cx));
                    tiles.push(render_public_playlist_tile(row.clone(), art, cx));
                }
                PublicPlaylistListItemVm::Status(status) => {
                    let element = render_index_list_status_row(
                        status.clone(),
                        Self::public_playlist_actions(),
                        cx,
                    );
                    if seen_row {
                        trailing.push(element);
                    } else {
                        leading.push(element);
                    }
                }
            }
        }
        let has_more = !trailing.is_empty();
        let is_loading = items.iter().any(|item| {
            matches!(
                item,
                PublicPlaylistListItemVm::Status(IndexListStatusRowVm::Loading { .. })
            )
        });
        render_tile_grid(
            "music-public-playlists-tiles",
            &self.public_playlists_tiles_scroll,
            leading,
            tiles,
            trailing,
            has_more,
            is_loading,
            LibraryApp::load_more_public_playlists,
            cx,
        )
    }

    /// Renders the item page of the playlist `feed_guid` (ADR 0089 packet
    /// 009): the loading state, the error state with "Try again", or the
    /// page with its entries.
    pub(crate) fn render_public_playlist_page(
        &mut self,
        feed_guid: &str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.public_playlist_pages.state_of(feed_guid).cloned() {
            Some(PublicPlaylistLoadState::Loaded(page)) => {
                let header_art = collection_art_image(self, &page.art, ThumbnailSize::Xl, cx);
                let mut rows = Vec::new();
                for item in &page.items {
                    let art = collection_art_image(self, &item.art, ThumbnailSize::Sm, cx);
                    rows.push(render_public_playlist_item(item, art, cx));
                }
                render_public_playlist_page_body(&page, header_art, rows, cx)
            }
            Some(PublicPlaylistLoadState::Failed { detail }) => {
                render_public_playlist_failed(detail, cx)
            }
            Some(PublicPlaylistLoadState::Loading) | None => render_public_playlist_loading(),
        }
    }
}
