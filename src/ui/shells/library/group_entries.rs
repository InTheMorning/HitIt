//! The rows and the tiles of a group page (ADR 0090 Decision 2, musicindex
//! ADR 0053 R-LIST-9 row "A group of singles").
//!
//! The singles group page and the window group page draw their entries with
//! this module. The Rows view uses the track rows of the release page. The
//! Tiles view uses the content tile of the Releases tiles, with the
//! selection mark and the open action of the row.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::sync::Arc;

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Context, FocusHandle, FontWeight, Image,
    SharedString, Styled,
};

use crate::library::LibraryApp;
use crate::ui::composites::release_download_state::render_release_download_state;
use crate::ui::composites::selectable_tile::SelectableTile;
use crate::ui::primitives::Checkbox;
use crate::ui::shells::library::content_list::render_content_list_tile_artwork;
use crate::ui::shells::library::destination_lists::checkbox_state_for;
use crate::ui::shells::library::detail::DetailRowSlot;
use crate::ui::shells::library::releases_tiles::tile_text_line;
use crate::ui::shells::library::singles_group::{
    index_single_dispatch, index_single_vm, render_index_single_row,
    render_matched_by_name_heading, render_single_row,
};
use crate::ui::shells::release_page::{row_click_modifiers, ReleasePageDispatch};
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor, Size, Spacing};
use crate::view_models::library::MbTrackStatus;
use crate::view_models::list_page::ListView;
use crate::view_models::release_download_state::ReleaseDownloadStateVm;
use crate::view_models::release_page::{ReleasePageCommand, ReleaseTrackRowVm};
use crate::view_models::singles_group::{MatchedByNameHeadingVm, SinglesGroupPageEntryVm};
use crate::view_models::PlaylistOptionDisplayVm;

/// The facts that [`render_group_entries`] needs to draw the rows or the
/// tiles of a group page.
pub(super) struct GroupEntries<'a> {
    /// The rows in the shown order, the "Matched by name" part last.
    pub(super) rows: &'a [SinglesGroupPageEntryVm],
    /// The heading of the trailing "Matched by name" part.
    pub(super) matched_by_name: Option<&'a MatchedByNameHeadingVm>,
    /// The view of the page: rows or tiles (ADR 0090 Decision 2).
    pub(super) view: ListView,
    pub(super) mb_status: &'a BTreeMap<i64, MbTrackStatus>,
    pub(super) album_thumbs: &'a BTreeMap<String, Option<Arc<Image>>>,
    pub(super) playlists: &'a [PlaylistOptionDisplayVm],
    pub(super) detail_focus: &'a FocusHandle,
    pub(super) dispatch: &'a ReleasePageDispatch,
}

/// Draws the entries of a group page in the view of the page: one row for
/// each entry, or one tile for each entry in a grid. The "Matched by name"
/// heading stays above its part in both views. The singles group page and
/// the window group page share this function (musicindex ADR 0053 R-LIST-9
/// row "A group of singles").
pub(super) fn render_group_entries(
    entries: &GroupEntries<'_>,
    row_slots: Vec<DetailRowSlot>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let rows = entries.rows;
    let heading_at = entries.matched_by_name.map_or(rows.len(), |heading| {
        rows.len().saturating_sub(heading.count)
    });
    let tiles_view = entries.view == ListView::Tiles;
    let mut elements: Vec<AnyElement> = Vec::with_capacity(rows.len() + 2);
    let mut tiles: Vec<AnyElement> = Vec::new();
    for (index, (row, slot)) in rows.iter().zip(row_slots).enumerate() {
        if index == heading_at {
            if let Some(heading) = entries.matched_by_name {
                if !tiles.is_empty() {
                    elements.push(render_tile_part(std::mem::take(&mut tiles), cx));
                }
                elements.push(render_matched_by_name_heading(heading));
            }
        }
        match (row, tiles_view) {
            (SinglesGroupPageEntryVm::Library(row), false) => elements.push(render_single_row(
                row,
                index,
                entries.mb_status,
                entries.album_thumbs,
                entries.playlists,
                slot,
                entries.detail_focus,
                entries.dispatch,
                cx,
            )),
            (SinglesGroupPageEntryVm::Index(row), false) => elements.push(render_index_single_row(
                row,
                entries.album_thumbs,
                slot,
                entries.detail_focus,
                cx,
            )),
            (SinglesGroupPageEntryVm::Library(row), true) => {
                let vm = ReleaseTrackRowVm::library_single(
                    row,
                    entries.mb_status.get(&row.track.id),
                    slot.selection,
                );
                tiles.push(render_single_tile(
                    &vm,
                    index,
                    entries.album_thumbs,
                    slot.probe,
                    entries.dispatch,
                    cx,
                ));
            }
            (SinglesGroupPageEntryVm::Index(row), true) => {
                let vm = index_single_vm(row, slot.selection);
                let dispatch = index_single_dispatch(row.key.clone(), row.title.clone(), cx);
                tiles.push(render_single_tile(
                    &vm,
                    index,
                    entries.album_thumbs,
                    slot.probe,
                    &dispatch,
                    cx,
                ));
            }
        }
    }
    if !tiles.is_empty() {
        elements.push(render_tile_part(tiles, cx));
    }
    div()
        .flex()
        .flex_col()
        .gap(spacing::XXS)
        .children(elements)
        .into_any_element()
}

/// One part of the Tiles view: its tiles in a wrapping grid.
fn render_tile_part(tiles: Vec<AnyElement>, cx: &App) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(Spacing::MD.scaled(cx))
        .children(tiles)
        .into_any_element()
}

/// One tile of a single on a group page. It is the content tile of the
/// Releases tiles, with the selection mark and the open action of the row:
/// a click sends `ClickRow`, and the checkbox sends `ToggleRow` (ADR 0084
/// Decision 10). The tile shows the download state of the row, as a
/// Releases tile does. It shows no row menu and no download button.
fn render_single_tile(
    vm: &ReleaseTrackRowVm,
    index: usize,
    album_thumbs: &BTreeMap<String, Option<Arc<Image>>>,
    probe: Option<AnyElement>,
    dispatch: &ReleasePageDispatch,
    cx: &App,
) -> AnyElement {
    let thumbnail = vm
        .thumbnail_url
        .as_deref()
        .and_then(|url| album_thumbs.get(url))
        .and_then(Clone::clone);
    let artwork_size = Size::ContentTileArtwork.scaled(cx);
    let artwork = render_content_list_tile_artwork(thumbnail, artwork_size, cx);
    let checkbox = vm.selection.checkbox_visible.then(|| {
        let toggle = dispatch.clone();
        Checkbox::new(
            SharedString::from(format!("{}-checkbox", vm.row_id)),
            checkbox_state_for(vm.selection.check_state),
        )
        .a11y_label(vm.selection.checkbox_a11y_label.clone())
        .on_click(move |_: &ClickEvent, window, cx| {
            cx.stop_propagation();
            toggle(ReleasePageCommand::ToggleRow(index), window, cx);
        })
        .into_any_element()
    });
    let click = dispatch.clone();
    let mut tile = SelectableTile::new(SharedString::from(vm.row_id.clone()), artwork)
        .checkbox(checkbox)
        .selected(vm.selection.is_marked())
        .focused(vm.selection.focused)
        .scroll_probe(probe)
        .child(tile_text_line(
            vm.row.title.clone(),
            FontSize::Caption,
            FontWeight::MEDIUM,
            SemanticColor::Label,
            artwork_size,
        ));
    if let Some(date) = &vm.row.date_text {
        tile = tile.child(tile_text_line(
            date.clone(),
            FontSize::Micro,
            FontWeight::NORMAL,
            SemanticColor::TertiaryLabel,
            artwork_size,
        ));
    }
    if let Some(state) = ReleaseDownloadStateVm::of_slot(vm.download.state) {
        let id = format!("{}-download-state", vm.row_id);
        tile = tile.child(render_release_download_state(&state, id, cx));
    }
    tile.on_click(move |event: &ClickEvent, window, cx| {
        click(
            ReleasePageCommand::ClickRow(index, row_click_modifiers(event.modifiers())),
            window,
            cx,
        );
    })
    .into_any_element()
}
