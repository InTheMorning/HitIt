//! The Live list of the Index (ADR 0089 Decision 1, packet 011).
//!
//! The view model gives each row, its type word, its schedule text and its
//! link out. This shell owns only the layout and the click wiring. A click
//! on a row opens the release page of its feed. The link out opens the
//! `http` or `https` link that the view model gave (ADR 0089 packet 011).

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Context, FontWeight, SharedString, Styled,
};

use crate::library::LibraryApp;
use crate::ui::composites::{
    CollectionArt, CollectionArtImage, EntityKind, ListRow, ListRowA11yLabel, TagBadge,
    TagBadgeDisplay, ThumbnailSize,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button as UiButton, ButtonSize, Label};
use crate::ui::shells::library::index_list_status::{
    render_index_list_status_row, IndexListActions,
};
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::collection_art::CollectionKind;
use crate::view_models::live_page::{LiveListItemVm, LiveRowVm};

/// The empty state of the Live list, with its title (musicindex ADR 0026
/// R-EMPTY-1). The page has no other text.
pub(crate) fn render_live_empty(title: &str) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .p(spacing::LG)
        .child(
            Label::new(title.to_owned())
                .size(FontSize::Caption)
                .color(SemanticColor::TertiaryLabel),
        )
        .into_any_element()
}

/// Renders one item of the Live list in the Rows view (ADR 0089 packet 011):
/// a live row with its feed art, or the status row of the page.
pub(crate) fn render_live_list_item(
    item: LiveListItemVm,
    art: Option<CollectionArtImage>,
    actions: IndexListActions,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    match item {
        LiveListItemVm::Row(row) => render_live_row(row, art, cx),
        LiveListItemVm::Status(status) => render_index_list_status_row(status, actions, cx),
    }
}

/// One live row. The row opens the release page of its feed. Its type dot
/// and word come from the shared `TagBadge` (musicindex ADR 0033 R-DOT).
fn render_live_row(
    row: LiveRowVm,
    art: Option<CollectionArtImage>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let target = row.row.open.clone();
    let element_id = row.row.element_id.clone();
    let art = CollectionArt::new(
        art.unwrap_or(CollectionArtImage::EntityColor),
        CollectionKind::Release,
        ThumbnailSize::Sm,
    )
    .into_any_element();
    let mut list_row = ListRow::new(SharedString::from(element_id.clone()))
        .a11y_label(ListRowA11yLabel {
            label: SharedString::from(row.row.a11y_label),
        })
        .marked(row.row.marked)
        .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
            this.open_destination_row(&target, cx);
        }))
        .child(art)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    Label::new(row.row.label)
                        .size(FontSize::Micro)
                        .weight(FontWeight::MEDIUM)
                        .truncated(),
                )
                .when(!row.row.secondary_text.is_empty(), |el| {
                    el.child(
                        Label::new(row.row.secondary_text)
                            .size(FontSize::Micro)
                            .color(SemanticColor::TertiaryLabel)
                            .truncated(),
                    )
                })
                .when(!row.when.is_empty(), |el| {
                    el.child(
                        Label::new(row.when)
                            .size(FontSize::Micro)
                            .color(SemanticColor::TertiaryLabel)
                            .truncated(),
                    )
                }),
        )
        .child(TagBadge::new(TagBadgeDisplay {
            kind: EntityKind::Live,
            label: Some(SharedString::from(row.type_word)),
        }))
        // ADR 0089 Decision 6: the live row keeps the time and library columns
        // of a Releases row, so its columns align with the rows of the list.
        .time_column(None)
        .library_column(None);
    if let Some(link) = row.link {
        let url = link.url;
        list_row = list_row.trailing(
            UiButton::styled(
                SharedString::from(format!("{element_id}-link")),
                ControlStyle::Ghost,
            )
            .size(ButtonSize::Sm)
            .label(link.label)
            .a11y_label(link.a11y_label)
            .on_click(move |_: &ClickEvent, _window, cx: &mut App| {
                cx.stop_propagation();
                let _ = open::that(&url);
            })
            .into_any_element(),
        );
    }
    list_row.into_any_element()
}
