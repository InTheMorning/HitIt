//! The Tiles view of the Releases list (ADR 0089 packet 005 Part C, ADR
//! 0084 Decisions 2 and 6, ADR 0086 Decision 3).
//!
//! Each item of the Releases list has one tile: a release, a single, a
//! singles group and a window group. The tiles reuse the content tile
//! composite and the grid of the Discover view. The status rows keep their
//! row at the start and at the end of the grid. The Rows view stays in
//! `destination_lists.rs`.

#![warn(clippy::pedantic)]

use std::sync::Arc;

use gpui::{
    div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, Image, ScrollHandle,
    SharedString, Styled,
};

use crate::library::LibraryApp;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::release_download_state::render_release_download_state;
use crate::ui::composites::selectable_tile::SelectableTile;
use crate::ui::composites::{CollectionArt, CollectionArtImage, ThumbnailSize};
use crate::ui::primitives::{Checkbox, Label};
use crate::ui::shells::library::content_list::render_content_list_tile_artwork;
use crate::ui::shells::library::content_list_scroll::attach_list_scroll_with;
use crate::ui::shells::library::destination_lists::checkbox_state_for;
use crate::ui::shells::library::index_list_status::{
    render_index_list_status_row, IndexListActions,
};
use crate::ui::shells::library::window_groups::render_releases_window_group_tile;
use crate::ui::tokens::{FontSize, SemanticColor, Spacing};
use crate::view_models::collection_art::CollectionKind;
use crate::view_models::library::{
    DestinationListRowTarget, DestinationListRowVm, ReleasesListItemVm,
};
use crate::view_models::singles_group::SinglesGroupRowVm;

/// Renders the Tiles view of the Releases list (ADR 0089 packet 005 Part
/// C). `leading` and `trailing` are the status rows, which keep their row
/// at the start and at the end of the list. `tiles` are the tiles of the
/// other items, in one grid. The grid holds its scroll in `scroll_handle`,
/// so the offset survives a change of view. The grid is not virtual, so
/// each tile of the list is in the element tree.
pub(crate) fn render_releases_tile_grid(
    scroll_handle: &ScrollHandle,
    leading: Vec<AnyElement>,
    tiles: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    has_more: bool,
    is_loading: bool,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    render_tile_grid(
        "music-releases-tiles",
        scroll_handle,
        leading,
        tiles,
        trailing,
        has_more,
        is_loading,
        |this, cx| this.start_recent_music_load(true, cx),
        cx,
    )
}

/// Renders a grid of tiles with the status rows at its start and its end
/// (ADR 0089 packets 005 and 009). `grid_id` names the grid element, and
/// `load_more` runs near the end of the scroll.
#[expect(
    clippy::too_many_arguments,
    reason = "the grid takes its id, its three rows, its two flags, its scroll action and the context"
)]
pub(crate) fn render_tile_grid(
    grid_id: &'static str,
    scroll_handle: &ScrollHandle,
    leading: Vec<AnyElement>,
    tiles: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    has_more: bool,
    is_loading: bool,
    load_more: fn(&mut LibraryApp, &mut Context<LibraryApp>),
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let grid = div()
        .id(grid_id)
        .flex_shrink_0()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(Spacing::MD.scaled(cx))
        .children(tiles);
    let content = page_scroll_content(cx)
        .gap(Spacing::SM.scaled(cx))
        .children(leading)
        .child(grid)
        .children(trailing);
    let container = div()
        .id("music-releases-tiles-container")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .px(Spacing::MD.scaled(cx))
        .pb(Spacing::MD.scaled(cx))
        .child(content);
    attach_list_scroll_with(
        container,
        has_more,
        is_loading,
        scroll_handle,
        load_more,
        cx,
    )
}

/// Renders one item of the Releases list in the Tiles view (ADR 0089 packet
/// 005 Part C). A release, a single and a singles group each have one tile,
/// and a window group has its tile. Each tile has the selection and the
/// click of its row. A status row keeps its row, the same as in the Rows
/// view.
pub(crate) fn render_releases_list_tile(
    item: ReleasesListItemVm,
    thumbnail: Option<Arc<Image>>,
    art: Option<CollectionArtImage>,
    actions: IndexListActions,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    match item {
        ReleasesListItemVm::LeadingStatus(row) | ReleasesListItemVm::TrailingStatus(row) => {
            render_index_list_status_row(row, actions, cx)
        }
        ReleasesListItemVm::Release(row) | ReleasesListItemVm::Single(row) => {
            render_releases_row_tile(row, thumbnail, cx)
        }
        ReleasesListItemVm::Group(group) => render_singles_group_tile(group, art, cx),
        ReleasesListItemVm::WindowGroup(tile) => render_releases_window_group_tile(
            &tile,
            art.unwrap_or(CollectionArtImage::EntityColor),
            cx,
        ),
    }
}

/// Renders the tile of one release row or single row (ADR 0084 Decision
/// 10). The checkbox sits at the cover corner, in selection mode only. A
/// click opens the release page, or selects with a modifier.
fn render_releases_row_tile(
    row: DestinationListRowVm,
    thumbnail: Option<Arc<Image>>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let artwork_size = crate::ui::tokens::Size::ContentTileArtwork.scaled(cx);
    let artwork = render_content_list_tile_artwork(thumbnail, artwork_size, cx);
    let checkbox = render_release_tile_checkbox(&row, cx);
    let target = row.open.clone();
    let download_state_id = format!("{}-download-state", row.element_id);
    let mut tile = SelectableTile::new(SharedString::from(row.element_id.clone()), artwork)
        .checkbox(checkbox)
        .selected(row.selection.is_marked())
        .focused(row.selection.focused)
        .marked(row.marked)
        .child(tile_text_line(
            row.label,
            FontSize::Caption,
            FontWeight::MEDIUM,
            SemanticColor::Label,
            artwork_size,
        ));
    if !row.secondary_text.is_empty() {
        tile = tile.child(tile_text_line(
            row.secondary_text,
            FontSize::Micro,
            FontWeight::NORMAL,
            SemanticColor::TertiaryLabel,
            artwork_size,
        ));
    }
    if let Some(state) = &row.download_state {
        tile = tile.child(render_release_download_state(state, download_state_id, cx));
    }
    tile.on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
        this.click_release_row(&target, event.modifiers(), cx);
    }))
    .into_any_element()
}

/// The checkbox of a release tile, or `None` outside selection mode (ADR
/// 0084 Decision 10 Amendment, 2026-10-19). It toggles the release.
fn render_release_tile_checkbox(
    row: &DestinationListRowVm,
    cx: &mut Context<LibraryApp>,
) -> Option<AnyElement> {
    if !row.selection.checkbox_visible {
        return None;
    }
    let target = row.open.clone();
    Some(
        Checkbox::new(
            SharedString::from(format!("{}-checkbox", row.element_id)),
            checkbox_state_for(row.selection.check_state),
        )
        .a11y_label(row.selection.checkbox_a11y_label.clone())
        .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            if let DestinationListRowTarget::Release(key) = &target {
                this.toggle_release_row_selection(key.clone(), cx);
            }
        }))
        .into_any_element(),
    )
}

/// Renders the tile of one singles group (ADR 0084 Decision 6). Its
/// checkbox selects each single, and a click follows the group row rule.
fn render_singles_group_tile(
    group: SinglesGroupRowVm,
    art: Option<CollectionArtImage>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let artwork_size = crate::ui::tokens::Size::ContentTileArtwork.scaled(cx);
    let artwork = CollectionArt::new(
        art.unwrap_or(CollectionArtImage::EntityColor),
        CollectionKind::SinglesGroup,
        ThumbnailSize::Xl,
    )
    .into_any_element();
    let checkbox = group.selection.checkbox_visible.then(|| {
        let toggle_key = group.open.clone();
        let toggle_keys = group.single_keys.clone();
        Checkbox::new(
            SharedString::from(format!("{}-checkbox", group.element_id)),
            checkbox_state_for(group.selection.check_state),
        )
        .a11y_label(group.selection.checkbox_a11y_label.clone())
        .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
            cx.stop_propagation();
            this.toggle_release_group_selection(&toggle_key, &toggle_keys, cx);
        }))
        .into_any_element()
    });
    let open_key = group.open.clone();
    let single_keys = group.single_keys.clone();
    let download_state_id = format!("{}-download-state", group.element_id);
    let mut tile = SelectableTile::new(SharedString::from(group.element_id.clone()), artwork)
        .checkbox(checkbox)
        .selected(group.selection.is_marked())
        .focused(group.selection.focused)
        .marked(group.marked)
        .child(tile_text_line(
            group.label,
            FontSize::Caption,
            FontWeight::MEDIUM,
            SemanticColor::Label,
            artwork_size,
        ));
    if !group.secondary_text.is_empty() {
        tile = tile.child(tile_text_line(
            group.secondary_text,
            FontSize::Micro,
            FontWeight::NORMAL,
            SemanticColor::TertiaryLabel,
            artwork_size,
        ));
    }
    tile = tile.child(render_release_download_state(
        &group.download_state,
        download_state_id,
        cx,
    ));
    tile.on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
        this.click_singles_group_row(&open_key, &single_keys, event.modifiers(), cx);
    }))
    .into_any_element()
}

/// One line of text in a tile, at the width of the cover. The title has
/// the primary color, and the secondary line has the tertiary color.
pub(crate) fn tile_text_line(
    text: String,
    size: FontSize,
    weight: FontWeight,
    color: SemanticColor,
    width: gpui::Pixels,
) -> AnyElement {
    div()
        .w(width)
        .min_w_0()
        .child(
            Label::new(text)
                .size(size)
                .weight(weight)
                .color(color)
                .truncated(),
        )
        .into_any_element()
}
