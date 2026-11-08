//! The My playlists list, in the Rows view and in the Tiles view (ADR 0084
//! Decision 2, ADR 0089 amendment of 2026-11-08, musicindex ADR 0053 R-LIST-9
//! row "Playlists that a person curates").
//!
//! The view model gives the rows in the shown order, the tiles, and the drag
//! of each row. This shell owns only layout and interaction wiring. With the
//! sort Curator Order, not reversed, in the Rows view, a playlist row drags to
//! a new place, and a drop calls `LibraryApp::move_playlist_to`. The Rows view
//! also keeps the drop of tracks on a row (ADR 0083 task 008).

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;

use gpui::{
    div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, ScrollHandle, SharedString,
    Styled,
};

use crate::library::LibraryApp;
use crate::ui::composites::content_tile::tile_text_line;
use crate::ui::composites::list_row::RowMenu;
use crate::ui::composites::music_column_scroll::music_column_tracked_scrollbar;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::selectable_tile::SelectableTile;
use crate::ui::composites::track_drag::{
    self, playlist_drop_target, PlaylistDrop, PlaylistReorder,
};
use crate::ui::composites::{
    CollectionArt, CollectionArtImage, ListRow, ListRowA11yLabel, ThumbnailSize,
};
use crate::ui::primitives::Label;
use crate::ui::shells::library::destination_lists::{
    collection_art, render_new_playlist_row, RowArts,
};
use crate::ui::shells::library::destination_row_second_line::destination_row_second_line;
use crate::ui::shells::library::releases_tiles::render_tile_grid;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::collection_art::CollectionKind;
use crate::view_models::library::{PlaylistSidebarRowVm, PlaylistSidebarVm};
use crate::view_models::list_page::ListView;
use crate::view_models::playlist_curation::PlaylistTileVm;
use crate::view_models::workspace::NavigationPathStart;

/// Renders the My playlists list in `view`: the inline new-playlist input and
/// each playlist as a row or as a tile. The title row holds "New Playlist".
pub(crate) fn render_my_playlists(
    sidebar: PlaylistSidebarVm,
    view: ListView,
    row_menus: BTreeMap<i64, RowMenu>,
    new_playlist_input: &gpui::Entity<gpui_component::input::InputState>,
    arts: &RowArts,
    scroll: &ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let new_row = sidebar.creating_playlist.then(|| {
        render_new_playlist_row(
            sidebar.new_playlist_input_id,
            sidebar.new_playlist_add_button_id,
            sidebar.new_playlist_add_label,
            new_playlist_input,
            cx,
        )
    });
    match view {
        ListView::Rows => render_rows(sidebar, row_menus, new_row, arts, scroll, cx),
        ListView::Tiles => render_tiles(&sidebar, new_row, arts, scroll, cx),
    }
}

/// The Rows view. Each row opens its playlist, takes the drop of tracks, and,
/// when the view model gives it a drag, moves in the curator order.
fn render_rows(
    sidebar: PlaylistSidebarVm,
    mut row_menus: BTreeMap<i64, RowMenu>,
    new_row: Option<AnyElement>,
    arts: &RowArts,
    scroll: &ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let mut items: Vec<AnyElement> = new_row.into_iter().collect();
    // ADR 0083 task 008, D6: a drop uses the add path of the menu item.
    let on_add =
        track_drag::playlist_drop_handler(cx.listener(|this, drop: &PlaylistDrop, _, cx| {
            this.end_track_drag();
            this.add_tracks_to_playlist(drop.track_ids.clone(), drop.playlist_id, cx);
        }));
    // ADR 0089 amendment of 2026-11-08: a drop of a playlist row moves it.
    let on_reorder: track_drag::PlaylistReorderHandler =
        std::rc::Rc::new(cx.listener(|this, reorder: &PlaylistReorder, _, cx| {
            this.end_track_drag();
            this.move_playlist_to(reorder.playlist_id, reorder.to_index, cx);
        }));
    for row in sidebar.rows {
        let PlaylistSidebarRowVm {
            id: playlist_id,
            element_id,
            name,
            track_count_label,
            type_tag,
            selected,
            marked,
            drag,
            shown_index,
            ..
        } = row;
        let art = collection_art(arts.get(&element_id).cloned(), CollectionKind::Playlist);
        let reorders = drag.is_some();
        let row = ListRow::new(SharedString::from(element_id))
            .a11y_label(ListRowA11yLabel {
                label: format!("Playlist: {name}").into(),
            })
            .selected(selected)
            .marked(marked)
            .playlist_drag(drag)
            // ADR 0089 Decision 6: each list row has the item menu cell.
            .menu_cell(row_menus.remove(&playlist_id))
            .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
                this.open_page(
                    crate::view_models::workspace::FrameNavigationEntry::PlaylistDetail(
                        playlist_id,
                    ),
                    NavigationPathStart::CurrentPath,
                    cx,
                );
            }))
            .child(div().pl(spacing::MD).child(art))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        Label::new(name)
                            .color(if selected {
                                SemanticColor::Accent
                            } else {
                                SemanticColor::Label
                            })
                            .truncated(),
                    )
                    // ADR 0089 Decision 6: each list row shows its type dot
                    // and its type word.
                    .children(destination_row_second_line(
                        Some(type_tag),
                        track_count_label,
                    )),
            )
            .into_any_element();
        items.push(playlist_drop_target(
            row,
            playlist_id,
            &on_add,
            reorders.then_some((shown_index, &on_reorder)),
            cx,
        ));
    }
    let container = div()
        .id("music-playlists-destination-list")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .pt(spacing::MD)
        .pb(spacing::MD)
        .child(page_scroll_content(cx).gap(spacing::XXS).children(items));
    music_column_tracked_scrollbar(container, scroll, None)
}

/// The Tiles view (musicindex ADR 0053 R-LIST-9). Each tile has the art of
/// its row, the name and the track count line, and a click opens the
/// playlist as the row does. A tile has no drag and no menu.
fn render_tiles(
    sidebar: &PlaylistSidebarVm,
    new_row: Option<AnyElement>,
    arts: &RowArts,
    scroll: &ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let tiles = sidebar
        .tiles()
        .into_iter()
        .map(|tile| {
            let image = arts.get(&tile.element_id).cloned();
            render_playlist_tile(tile, image, cx)
        })
        .collect();
    render_tile_grid(
        "music-playlists-tiles",
        scroll,
        new_row.into_iter().collect(),
        tiles,
        Vec::new(),
        false,
        false,
        |_, _| {},
        cx,
    )
}

/// One playlist tile: the shared tile parts, the art of the row, the name and
/// the track count line.
fn render_playlist_tile(
    tile: PlaylistTileVm,
    image: Option<CollectionArtImage>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let artwork_size = crate::ui::tokens::Size::ContentTileArtwork.scaled(cx);
    let artwork = CollectionArt::new(
        image.unwrap_or(CollectionArtImage::EntityColor),
        CollectionKind::Playlist,
        ThumbnailSize::Xl,
    )
    .into_any_element();
    let open = tile.open.clone();
    SelectableTile::new(SharedString::from(tile.element_id), artwork)
        .selected(tile.selected)
        .marked(tile.marked)
        .child(tile_text_line(
            tile.label,
            FontSize::Caption,
            FontWeight::MEDIUM,
            SemanticColor::Label,
            artwork_size,
        ))
        .child(tile_text_line(
            tile.second_line,
            FontSize::Micro,
            FontWeight::NORMAL,
            SemanticColor::TertiaryLabel,
            artwork_size,
        ))
        .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
            this.open_page(open.clone(), NavigationPathStart::CurrentPath, cx);
        }))
        .into_any_element()
}
