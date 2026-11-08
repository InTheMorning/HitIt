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
use std::sync::Arc;

use gpui::{div, prelude::*, AnyElement, App, Context, Entity, Window};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::WindowExt;

use super::app_impl::command_error_detail;
use super::LibraryApp;
use crate::application::commands::playlist::{
    CopyPublicPlaylistResult, CopyPublicPlaylistToMyPlaylists,
};
use crate::application::queries::public_playlists::{
    FetchPublicPlaylist, FetchPublicPlaylistPage, PublicPlaylistLoad,
};
use crate::application::CommandContext;
use crate::presentation::collection_art::collection_art_image;
use crate::presentation::present_command;
use crate::ui::composites::letter_bar::LetterHandler;
use crate::ui::composites::list_page_head::{
    list_page_head, ListPageHeadHandlers, ListReverseHandler, ListSortHandler, ListViewHandler,
};
use crate::ui::composites::list_row::{menu_cell, row_menu_parts, RowMenu};
use crate::ui::composites::playlist_popover::{CreateHandler, SelectHandler};
use crate::ui::composites::section_heading::section_heading;
use crate::ui::composites::track_row_menu::{MenuActionHandler, TrackRowMenuHandlers};
use crate::ui::composites::ThumbnailSize;
use crate::ui::shells::library::destination_lists::render_library_destination_rows;
use crate::ui::shells::library::index_list_status::{
    render_index_list_status_row, IndexListActions,
};
use crate::ui::shells::library::public_playlists::{
    public_playlist_copy_dialog, render_public_playlist_failed, render_public_playlist_item,
    render_public_playlist_list_item, render_public_playlist_loading,
    render_public_playlist_page_body, render_public_playlist_tile,
};
use crate::ui::shells::library::releases_tiles::render_tile_grid;
use crate::view_models::index_name_list::{IndexListStatusRowVm, IndexNameListLoadIntent};
use crate::view_models::library::DestinationListRowTarget;
use crate::view_models::list_page::{ListPageKey, ListSort, ListView};
use crate::view_models::public_playlists::{
    public_playlist_copy_menu, public_playlist_copy_report, public_playlist_list_items,
    PublicPlaylistCopyDialogVm, PublicPlaylistCopyName, PublicPlaylistCopyReport,
    PublicPlaylistListItemVm, PublicPlaylistLoadState, PublicPlaylistPageVm, PublicPlaylistRowVm,
    COPYING_TEXT, PUBLIC_PLAYLIST_NO_READABLE_TRACKS,
};
use crate::view_models::release_page::TrackRowMenuAction;

/// The dialog that copies a public playlist to My playlists (ADR 0089
/// packet 010). `message` names a rejected name, and the dialog stays open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublicPlaylistCopyDialog {
    pub(crate) feed_guid: String,
    pub(crate) title: String,
    pub(crate) message: Option<String>,
}

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
            ListSort::Newest
            | ListSort::Relevance
            | ListSort::FeedOrder
            | ListSort::PlaylistOrder
            | ListSort::CuratorOrder => None,
        };
        let reverse = self.vm.list_index_reversed(ListPageKey::PublicPlaylists);
        let command = FetchPublicPlaylistPage::new(
            self.musicindex_endpoint.clone(),
            sort,
            cursor,
            from,
            reverse,
        );
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

    /// Sets the public playlist of the open page, or `None` when no such page
    /// is open. A copy report ends when its page closes (ADR 0089 packet
    /// 010), so the report is kept only on the page that it names.
    pub(crate) fn set_open_public_playlist(&mut self, open: Option<&str>) {
        let kept = self
            .public_playlist_copy_report
            .as_ref()
            .is_some_and(|report| report.text_on_page(open).is_some());
        if !kept {
            self.public_playlist_copy_report = None;
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
        let display = self.with_letter_cursor(self.vm.list_page_head(
            ListPageKey::PublicPlaylists,
            None,
            None,
        ));
        let letter_keys = self.letter_bar_keys(ListPageKey::PublicPlaylists, cx);
        let entity = cx.entity();
        let sort_entity = entity.clone();
        let on_sort: ListSortHandler = Rc::new(move |sort, reversed, _window, cx| {
            let sort = *sort;
            sort_entity.update(cx, |this, cx| {
                this.set_list_order(ListPageKey::PublicPlaylists, sort, reversed, cx);
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
                letter_keys: Some(letter_keys),
                actions: Vec::new(),
            },
            cx,
        )
    }

    /// Opens the copy dialog for the playlist `feed_guid`, with `title` as
    /// the default name (ADR 0089 packet 010).
    pub(crate) fn open_public_playlist_copy_dialog(
        &mut self,
        feed_guid: &str,
        title: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let default_name = title.to_owned();
        self.public_playlist_copy_input.update(cx, |input, cx| {
            input.set_value(default_name, window, cx);
            input.focus(window, cx);
        });
        self.public_playlist_copy = Some(PublicPlaylistCopyDialog {
            feed_guid: feed_guid.to_owned(),
            title: title.to_owned(),
            message: None,
        });
        let weak = cx.weak_entity();
        let input = self.public_playlist_copy_input.clone();
        window.open_dialog(cx, move |dialog, _window, cx| {
            let (title, message) = weak
                .upgrade()
                .and_then(|app| {
                    app.read(cx)
                        .public_playlist_copy
                        .as_ref()
                        .map(|copy| (copy.title.clone(), copy.message.clone()))
                })
                .unwrap_or_default();
            public_playlist_copy_dialog(
                dialog,
                PublicPlaylistCopyDialogVm::new(&title, message),
                &input,
                weak.clone(),
                cx,
            )
        });
        cx.notify();
    }

    /// Checks the name in the copy dialog and starts the copy (ADR 0089
    /// packet 010). A blank name or a name that exists keeps the dialog
    /// open with its message. Gives `true` when the copy starts.
    pub(crate) fn submit_public_playlist_copy(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(copy) = self.public_playlist_copy.clone() else {
            return false;
        };
        let input = self.public_playlist_copy_input.read(cx).value().to_string();
        let check = {
            let existing: Vec<&str> = self
                .vm
                .playlists()
                .iter()
                .map(|playlist| playlist.name.as_str())
                .collect();
            PublicPlaylistCopyName::check(&input, &existing)
        };
        let name = match check {
            PublicPlaylistCopyName::Ready(name) => name,
            rejected => {
                if let Some(copy) = self.public_playlist_copy.as_mut() {
                    copy.message = rejected.message();
                }
                cx.notify();
                return false;
            }
        };
        self.public_playlist_copy = None;
        self.run_public_playlist_copy(&copy.feed_guid, name, cx);
        true
    }

    /// Ends the copy dialog without a copy (ADR 0089 packet 010).
    pub(crate) fn cancel_public_playlist_copy(&mut self, cx: &mut Context<Self>) {
        self.public_playlist_copy = None;
        cx.notify();
    }

    /// The Enter key of the copy dialog starts the copy (ADR 0089 packet
    /// 010). A dialog that stays open repaints with its message.
    pub(crate) fn on_public_playlist_copy_input_event(
        &mut self,
        _input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::PressEnter { .. } = event {
            if self.submit_public_playlist_copy(cx) {
                window.close_dialog(cx);
            } else {
                window.refresh();
            }
        }
    }

    /// Runs the copy of the public playlist `feed_guid` to a new local
    /// playlist `name` on the runtime (ADR 0089 packet 010). The row of the
    /// playlist shows the busy text until the command ends. The result
    /// updates My playlists in place.
    fn run_public_playlist_copy(&mut self, feed_guid: &str, name: String, cx: &mut Context<Self>) {
        self.copying_public_playlist = Some(feed_guid.to_owned());
        // A new copy ends the report of the last copy (ADR 0089 packet 010).
        self.public_playlist_copy_report = None;
        cx.notify();
        let command = CopyPublicPlaylistToMyPlaylists::new(
            Arc::clone(&self.conn),
            self.musicindex_endpoint.clone(),
            feed_guid,
            name,
        );
        let report_guid = feed_guid.to_owned();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                this.copying_public_playlist = None;
                let text = match result {
                    CopyPublicPlaylistResult::Copied(outcome) => {
                        this.reload_playlists(cx);
                        public_playlist_copy_report(&outcome.name, outcome.copied, outcome.total)
                    }
                    CopyPublicPlaylistResult::NameTaken(name) => {
                        PublicPlaylistCopyName::Taken(name)
                            .message()
                            .unwrap_or_default()
                    }
                    CopyPublicPlaylistResult::NoReadableTracks => {
                        PUBLIC_PLAYLIST_NO_READABLE_TRACKS.to_owned()
                    }
                };
                this.vm.set_status(text.clone());
                this.public_playlist_copy_report =
                    Some(PublicPlaylistCopyReport::new(&report_guid, text));
                cx.notify();
            },
            move |this, error, cx| {
                this.copying_public_playlist = None;
                this.vm.set_error_status(command_error_detail(error));
                cx.notify();
            },
        );
    }

    /// Runs one item of a public playlist menu (ADR 0089 packet 010).
    /// "Copy to My Playlists…" opens the copy dialog.
    pub(crate) fn run_public_playlist_menu_action(
        &mut self,
        feed_guid: &str,
        title: &str,
        action: TrackRowMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if action == TrackRowMenuAction::CopyToMyPlaylists {
            self.open_public_playlist_copy_dialog(feed_guid, title, window, cx);
        }
    }

    /// The "More actions" menu of a public playlist, as the row or page
    /// menu cell of the Public playlists list (ADR 0089 packet 010).
    fn public_playlist_menu_of(
        &self,
        menu: crate::view_models::release_page::TrackRowMenuVm,
        feed_guid: String,
        title: String,
        cx: &mut Context<Self>,
    ) -> RowMenu {
        let entity = cx.weak_entity();
        let action: MenuActionHandler = Rc::new(
            move |action: TrackRowMenuAction, window: &mut Window, cx: &mut App| {
                let _ = entity.update(cx, |this, cx| {
                    this.run_public_playlist_menu_action(&feed_guid, &title, action, window, cx);
                });
            },
        );
        // A public playlist menu has no playlist level, so these two
        // handlers do nothing.
        let select_playlist: SelectHandler = Rc::new(|_: &i64, _: &mut Window, _: &mut App| {});
        let create_playlist: CreateHandler = Rc::new(|_: &String, _: &mut Window, _: &mut App| {});
        RowMenu {
            menu,
            playlists: Rc::new(Vec::new()),
            handlers: TrackRowMenuHandlers {
                action,
                select_playlist,
                create_playlist,
            },
            return_focus: self.list_column_focus.clone(),
        }
    }

    /// The menu cell content of a public playlist row, or `None` when the
    /// row has no menu.
    fn public_playlist_row_menu(
        &self,
        row: &PublicPlaylistRowVm,
        cx: &mut Context<Self>,
    ) -> Option<RowMenu> {
        let menu = row.menu.clone()?;
        let DestinationListRowTarget::PublicPlaylist(feed_guid) = &row.row.open else {
            return None;
        };
        Some(self.public_playlist_menu_of(menu, feed_guid.clone(), row.row.label.clone(), cx))
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
            self.copying_public_playlist.as_deref(),
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
        let (art, row_menu) = match item {
            PublicPlaylistListItemVm::Row(row) => (
                row.row
                    .art
                    .as_ref()
                    .map(|art| collection_art_image(self, art, ThumbnailSize::Sm, cx)),
                self.public_playlist_row_menu(row, cx),
            ),
            PublicPlaylistListItemVm::Status(_) => (None, None),
        };
        render_public_playlist_list_item(
            item.clone(),
            art,
            row_menu,
            Self::public_playlist_actions(),
            cx,
        )
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
                        .row
                        .art
                        .as_ref()
                        .map(|art| collection_art_image(self, art, ThumbnailSize::Xl, cx));
                    tiles.push(render_public_playlist_tile(row.row.clone(), art, cx));
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

    /// The heading row above the entries of `page`, or `None` when the page
    /// has none (musicindex ADR 0053 R-LIST-11 item 3). The Reverse
    /// toggle changes the one state of the entries of a playlist, and
    /// the page builds again in place.
    fn render_public_playlist_heading(
        page: &PublicPlaylistPageVm,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let heading = page.heading.clone()?;
        let entity = cx.entity();
        let on_reverse: ListReverseHandler = Rc::new(move |reversed, _window, cx| {
            entity.update(cx, |this, cx| {
                this.set_list_reversed(ListPageKey::PublicPlaylistItems, reversed, cx);
            });
        });
        Some(section_heading(heading, None, Some(&on_reverse), cx))
    }

    /// Renders the item page of the playlist `feed_guid` (ADR 0089 packet
    /// 009): the loading state, the error state with "Try again", or the
    /// page with its entries.
    pub(crate) fn render_public_playlist_page(
        &mut self,
        feed_guid: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.public_playlist_pages.state_of(feed_guid).cloned() {
            Some(PublicPlaylistLoadState::Loaded(page)) => {
                let header_art = collection_art_image(self, &page.art, ThumbnailSize::Xl, cx);
                // Musicindex ADR 0053 R-LIST-9: the page shows its entries in
                // the order of the stored state, and the heading row has the
                // Reverse toggle from the count of the controls.
                let page = page.shown(self.vm.list_page_state(ListPageKey::PublicPlaylistItems));
                let heading = Self::render_public_playlist_heading(&page, cx);
                let mut rows = Vec::new();
                for item in &page.items {
                    let art = collection_art_image(self, &item.art, ThumbnailSize::Sm, cx);
                    rows.push(render_public_playlist_item(item, art, cx));
                }
                // The "More actions" menu of the page (ADR 0089 packet 010). A
                // page has no row to hover, so the cell always shows, as on a
                // release page.
                let copying = self.copying_public_playlist.is_some();
                let menu = public_playlist_copy_menu(feed_guid, &page.title, copying).map(|menu| {
                    self.public_playlist_menu_of(menu, feed_guid.to_owned(), page.title.clone(), cx)
                });
                let (menu_cell, _root) = row_menu_parts(menu, menu_cell(cx), true, window, cx);
                // The copy text shows while the copy runs. Then the report of
                // the copy shows in place, next to the title (ADR 0089 packet 010).
                let copying = self.copying_public_playlist.as_deref() == Some(feed_guid);
                let report = self
                    .public_playlist_copy_report
                    .as_ref()
                    .and_then(|report| report.text_on_page(Some(feed_guid)))
                    .map(str::to_owned);
                let status = if copying {
                    Some(COPYING_TEXT.to_owned())
                } else {
                    report
                };
                render_public_playlist_page_body(
                    &page,
                    header_art,
                    menu_cell.into_any_element(),
                    status.as_deref(),
                    heading,
                    rows,
                    cx,
                )
            }
            Some(PublicPlaylistLoadState::Failed { detail }) => {
                render_public_playlist_failed(detail, cx)
            }
            Some(PublicPlaylistLoadState::Loading) | None => render_public_playlist_loading(),
        }
    }
}
