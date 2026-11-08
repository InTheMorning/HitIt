//! The heading controls and the tiles of a release section (ADR 0090 task
//! 004, musicindex ADR 0053 R-LIST-9 row "The releases of an artist, a label
//! or a name", R-LIST-10, R-LIST-11 item 3).
//!
//! From `SECTION_CONTROL_MIN_ROWS` rows, a section of a feed page or of a
//! name level page has its sort menu and its view icons at
//! the right end of its heading. The view model gives each control, its id
//! and its state. This composite lays them out with the renderers of the
//! list page head, and it draws the Tiles view with the tile parts of the
//! Releases tiles.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, App, ClickEvent, FontWeight, Image, IntoElement, ParentElement, SharedString,
    Styled,
};

use crate::ui::composites::content_tile::{render_content_list_tile_artwork, tile_text_line};
use crate::ui::composites::list_page_head::{
    sort_menu_scoped, view_icons_scoped, ListSortHandler, ListViewHandler,
};
use crate::ui::composites::release_download_state::render_release_download_state;
use crate::ui::composites::release_section_list::{
    checkbox_state_for, title_display_text, ReleaseSectionRows, SectionCommandCallback,
};
use crate::ui::composites::selectable_tile::SelectableTile;
use crate::ui::primitives::{Checkbox, SectionHeader};
use crate::ui::tokens::{FontSize, SemanticColor, Size, Spacing};
use crate::view_models::list_page::ListBarControlsVm;
use crate::view_models::publisher_page::PublisherPageAlbumVm;
use crate::view_models::release_section_order::{ReleaseSectionCommand, ReleaseSectionControlsVm};

/// The heading of a section with controls: the heading text at the left, and
/// the sort menu and the view icons at the right end (musicindex ADR 0054
/// R-LIST-13). A narrow window wraps the
/// controls below the text. The element id of each control holds the heading,
/// so two sections of one page share no state.
pub(crate) fn render_section_heading_with_controls(
    heading: &str,
    controls: ReleaseSectionControlsVm,
    on_command: &SectionCommandCallback,
    cx: &App,
) -> AnyElement {
    let ReleaseSectionControlsVm {
        id,
        controls: ListBarControlsVm { sort, view, .. },
    } = controls;
    let scope = format!("section-controls:{}", id.heading());
    let scope = scope.as_str();
    let mut right = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .ml_auto()
        .gap(Spacing::XS.scaled(cx));
    if let Some(sort) = sort {
        let (command, id) = (on_command.clone(), id.clone());
        let handler: ListSortHandler = Rc::new(move |sort, reversed, window, cx| {
            command(
                &id,
                ReleaseSectionCommand::Sort(*sort, reversed),
                window,
                cx,
            );
        });
        right = right.child(sort_menu_scoped(sort, Some(&handler), false, Some(scope)));
    }
    if let Some(view) = view {
        let (command, id) = (on_command.clone(), id.clone());
        let handler: ListViewHandler = Rc::new(move |view, window, cx| {
            command(&id, ReleaseSectionCommand::View(*view), window, cx);
        });
        right = right.child(view_icons_scoped(
            view,
            Some(&handler),
            false,
            Some(scope),
            cx,
        ));
    }
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .child(SectionHeader::new(heading.to_owned()))
        .child(right)
        .into_any_element()
}

/// The Tiles view of a section: one tile for each release, in a wrapping
/// grid. A tile has the selection checkbox in selection mode, the open action
/// of its row and the download state of its row.
pub(crate) fn render_release_tiles(
    rows_of_section: &[PublisherPageAlbumVm],
    section_heading: &str,
    album_thumbs: &BTreeMap<String, Option<Arc<Image>>>,
    rows: &ReleaseSectionRows,
    cx: &App,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(Spacing::MD.scaled(cx))
        .children(
            rows_of_section
                .iter()
                .map(|row| render_release_tile(row, section_heading, album_thumbs, rows, cx)),
        )
        .into_any_element()
}

/// One tile of a release. It is the content tile of the Releases tiles with
/// the selection facts and the open action of the row: a click sends the
/// click of the row, and the checkbox sends the toggle of the row (ADR 0084
/// Decision 10). It shows no row menu.
fn render_release_tile(
    row: &PublisherPageAlbumVm,
    section_heading: &str,
    album_thumbs: &BTreeMap<String, Option<Arc<Image>>>,
    rows: &ReleaseSectionRows,
    cx: &App,
) -> AnyElement {
    let (title, _) = title_display_text(&row.title);
    let row_id = row.element_id(section_heading);
    let thumbnail = row
        .image_url
        .as_deref()
        .and_then(|url| album_thumbs.get(url))
        .and_then(Clone::clone);
    let artwork_size = Size::ContentTileArtwork.scaled(cx);
    let artwork = render_content_list_tile_artwork(thumbnail, artwork_size, cx);
    let selection = &row.selection;
    let checkbox = match (selection.checkbox_visible, &row.open) {
        (true, Some(open)) => {
            let on_toggle_release = Rc::clone(&rows.on_toggle_release);
            let key = open.key.clone();
            Some(
                Checkbox::new(
                    SharedString::from(format!("{row_id}-checkbox")),
                    checkbox_state_for(selection.check_state),
                )
                .a11y_label(selection.checkbox_a11y_label.clone())
                .on_click(move |_event: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    on_toggle_release(&key, window, cx);
                })
                .into_any_element(),
            )
        }
        _ => None,
    };
    let mut card = SelectableTile::new(SharedString::from(row_id.clone()), artwork)
        .checkbox(checkbox)
        .selected(selection.is_marked())
        .focused(selection.focused)
        .scroll_probe(selection.focused.then(|| rows.reveal.probe()))
        .child(tile_text_line(
            title.clone(),
            FontSize::Caption,
            FontWeight::MEDIUM,
            SemanticColor::Label,
            artwork_size,
        ));
    if let Some(artist) = &row.artist {
        card = card.child(tile_text_line(
            artist.display_text(),
            FontSize::Micro,
            FontWeight::NORMAL,
            SemanticColor::TertiaryLabel,
            artwork_size,
        ));
    }
    card = card.child(render_release_download_state(
        &row.download_state,
        format!("{row_id}-download-state"),
        cx,
    ));
    if let Some(open) = &row.open {
        let on_click_release = Rc::clone(&rows.on_click_release);
        let key = open.key.clone();
        card = card.on_click(move |event: &ClickEvent, window, cx| {
            on_click_release(&key, title.clone(), event.modifiers(), window, cx);
        });
    }
    card.into_any_element()
}
