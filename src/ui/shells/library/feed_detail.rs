//! Library feed (album) detail surface.
//!
//! `LibraryApp` is the host of the Library release page (ADR 0084 Decision
//! 4, packet 021). This module gives the facts, the resolved images and one
//! dispatch function to the shared release page shell. Mutations and
//! selected-entity state stay on `LibraryApp`.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, FocusHandle, Image, ScrollHandle, Window};

use crate::db;
use crate::library::{playlist_options, LibraryApp, LibraryAppEvent};
use crate::media::cover_color::CoverColor;
use crate::ui::composites::music_column_scroll::ListBarPin;
use crate::ui::composites::{list_bar_playlists, render_list_bar_copies, ListBarHandler};
use crate::ui::shells::library::detail::DetailRowSlot;
use crate::ui::shells::release_page::{
    render_release_page, ReleasePageDispatch, ReleasePageImages,
};
use crate::view_models::detail_selection::ListBarVm;
use crate::view_models::library::{AlbumNode, LibraryViewModel, MbTrackStatus};
use crate::view_models::release_page::{release_page_vm, ReleasePageFacts};
use crate::view_models::selection_action_bar::ListBarCommand;

/// Renders the Library release page of `album`. The list bar sits above
/// the track rows (ADR 0090 Decision 1).
#[expect(
    clippy::too_many_arguments,
    reason = "the page renderer takes each already-resolved value of the Library host"
)]
pub(crate) fn render_library_feed_detail(
    album: &AlbumNode,
    mb_status: &BTreeMap<i64, MbTrackStatus>,
    library_vm: &LibraryViewModel,
    album_thumbs: &BTreeMap<String, Option<Arc<Image>>>,
    cover_color: Option<CoverColor>,
    list_bar: Option<ListBarVm>,
    list_bar_pin: &ListBarPin,
    playlists: &[db::Playlist],
    row_slots: Vec<DetailRowSlot>,
    scroll: &ScrollHandle,
    detail_focus: &FocusHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let (selections, row_probes): (Vec<_>, Vec<_>) = row_slots
        .into_iter()
        .map(|slot| (slot.selection, slot.probe))
        .unzip();
    let facts = ReleasePageFacts::from_library_album(album, mb_status, library_vm)
        .with_row_selections(selections);
    let vm = release_page_vm(&facts);
    let (list_bar, pinned_list_bar) =
        render_library_list_bar_copies(list_bar, playlists, list_bar_pin, scroll, cx);
    let thumbnail = |url: Option<&str>| {
        url.and_then(|url| album_thumbs.get(url))
            .and_then(Clone::clone)
    };
    let images = ReleasePageImages {
        hero_image: thumbnail(album.image_href.as_deref()),
        cover_color,
        row_thumbnails: vm
            .rows
            .iter()
            .map(|row| thumbnail(row.thumbnail_url.as_deref()))
            .collect(),
        row_probes,
        contributor_thumbnails: Some(album_thumbs),
        scroll_handle: Some(scroll.clone()),
        row_menu_focus: Some(detail_focus.clone()),
        list_bar,
        pinned_list_bar,
    };
    render_release_page(&vm, images, &library_release_page_dispatch(cx), cx)
}

/// The dispatch function of a Library release page and of a singles group
/// page. It holds only the entity handle.
pub(crate) fn library_release_page_dispatch(cx: &Context<LibraryApp>) -> ReleasePageDispatch {
    let entity = cx.entity();
    Rc::new(move |command, window, cx| {
        entity.update(cx, |this, cx| {
            this.run_release_page_command(command, window, cx);
        });
    })
}

/// The in-flow list bar and its pinned copy of a Library page (ADR 0090).
/// Each command goes to the top app as a `LibraryAppEvent::ListBar`. The
/// singles group page and the window group page use this function too.
pub(crate) fn render_library_list_bar_copies(
    list_bar: Option<ListBarVm>,
    playlists: &[db::Playlist],
    list_bar_pin: &ListBarPin,
    scroll: &ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> (Option<AnyElement>, Option<AnyElement>) {
    let handler = library_list_bar_handler(cx);
    let bar_playlists = list_bar_playlists(playlist_options(playlists));
    render_list_bar_copies(
        list_bar,
        &bar_playlists,
        &handler,
        list_bar_pin,
        scroll,
        &cx.entity(),
        cx,
    )
}

/// The handler of a Library list bar. It emits each command, and the top
/// app runs it through its selection path.
fn library_list_bar_handler(cx: &mut Context<LibraryApp>) -> ListBarHandler {
    let entity = cx.entity();
    Rc::new(
        move |command: ListBarCommand, _window: &mut Window, cx: &mut gpui::App| {
            entity.update(cx, |this, cx| match command {
                // ADR 0090 Decision 2: the host of the page runs the sort,
                // the view and the reverse commands in place.
                ListBarCommand::Sort(sort, reversed) => {
                    this.set_group_list_order(sort, reversed, cx);
                }
                ListBarCommand::View(view) => this.set_group_list_view(view, cx),
                ListBarCommand::Reverse(reversed) => this.set_group_list_reversed(reversed, cx),
                command => cx.emit(LibraryAppEvent::ListBar(command)),
            });
        },
    )
}
