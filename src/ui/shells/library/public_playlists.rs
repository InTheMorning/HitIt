//! The item page of a public playlist (ADR 0089 Decision 1, packet 009).
//!
//! The view model gives the title, the art, each entry and each text. This
//! shell owns only the layout and the click wiring. It reads no data. The
//! status of the page (loading, failed) is drawn here from the view model
//! state, with its "Try again" action.

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Context, Entity, FontWeight, SharedString,
    Styled, WeakEntity,
};
use gpui_component::dialog::Dialog;
use gpui_component::input::{Input, InputState};
use gpui_component::WindowExt;

use crate::library::LibraryApp;
use crate::ui::composites::list_row::RowMenu;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::selectable_tile::SelectableTile;
use crate::ui::composites::{
    CollectionArt, CollectionArtImage, ListRow, ListRowA11yLabel, ThumbnailSize,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::primary_selection::PrimarySelectionExt as _;
use crate::ui::primitives::{Button as UiButton, ButtonSize, Label};
use crate::ui::shells::library::destination_lists::{
    collection_art, render_destination_list_row, DestinationRowMenu, LeadingDisclosure, RowArt,
};
use crate::ui::shells::library::index_list_status::{
    render_index_list_status_row, IndexListActions,
};
use crate::ui::shells::library::releases_tiles::tile_text_line;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor, Size};
use crate::view_models::collection_art::CollectionKind;
use crate::view_models::library::DestinationListRowVm;
use crate::view_models::public_playlists::{
    PublicPlaylistCopyDialogVm, PublicPlaylistItemVm, PublicPlaylistListItemVm,
    PublicPlaylistPageVm, COPY_CANCEL_LABEL, COPY_CONFIRM_LABEL, COPY_DIALOG_TITLE,
};
use crate::view_models::workspace::NavigationPathStart;

/// Builds the copy dialog of a public playlist (ADR 0089 packet 010). The
/// name field, "Cancel" and "Copy" are here. "Copy" keeps the dialog open
/// when the name is rejected, and the message shows in it. The Library
/// decides the name, so this shell reads no playlist data.
pub(crate) fn public_playlist_copy_dialog(
    dialog: Dialog,
    display: PublicPlaylistCopyDialogVm,
    input: &Entity<InputState>,
    weak: WeakEntity<LibraryApp>,
    cx: &App,
) -> Dialog {
    let cancel_weak = weak.clone();
    let button_cancel_weak = weak.clone();
    let confirm_weak = weak;
    let body_width = Size::ColumnRegular.scaled(cx);
    dialog
        .title(COPY_DIALOG_TITLE)
        .overlay_closable(false)
        .close_button(false)
        .on_cancel(move |_, _window, cx| {
            let _ = cancel_weak.update(cx, LibraryApp::cancel_public_playlist_copy);
            true
        })
        .child(
            div()
                .w(body_width)
                .flex()
                .flex_col()
                .gap(spacing::MD)
                .child(
                    Label::new(display.body)
                        .size(FontSize::Body)
                        .color(SemanticColor::SecondaryLabel),
                )
                .child(Input::new(input).with_primary_selection(input))
                .when_some(display.message, |el, message| {
                    el.child(
                        Label::new(message)
                            .size(FontSize::Caption)
                            .color(SemanticColor::Danger),
                    )
                })
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(spacing::SM)
                        .child(
                            UiButton::styled(
                                SharedString::from("public-playlist-copy-cancel"),
                                ControlStyle::Secondary,
                            )
                            .size(ButtonSize::Md)
                            .label(COPY_CANCEL_LABEL)
                            .a11y_label("Cancel the copy to My Playlists")
                            .on_click(move |_, window, cx| {
                                let _ = button_cancel_weak
                                    .update(cx, LibraryApp::cancel_public_playlist_copy);
                                window.close_dialog(cx);
                            }),
                        )
                        .child(
                            UiButton::styled(
                                SharedString::from("public-playlist-copy-confirm"),
                                ControlStyle::Primary,
                            )
                            .size(ButtonSize::Md)
                            .label(COPY_CONFIRM_LABEL)
                            .a11y_label("Copy the public playlist to My Playlists")
                            .on_click(move |_, window, cx| {
                                let copied = confirm_weak
                                    .update(cx, LibraryApp::submit_public_playlist_copy)
                                    .unwrap_or(false);
                                window.refresh();
                                if copied {
                                    window.close_dialog(cx);
                                }
                            }),
                        ),
                ),
        )
}

/// Renders one playlist tile in the Tiles view (ADR 0089 packet 009). The
/// tile shows the art, the title and the secondary text. A click opens the
/// playlist page, and the marked tile is the page that Back left open.
pub(crate) fn render_public_playlist_tile(
    row: DestinationListRowVm,
    art: Option<CollectionArtImage>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let artwork_size = crate::ui::tokens::Size::ContentTileArtwork.scaled(cx);
    let artwork = CollectionArt::new(
        art.unwrap_or(CollectionArtImage::EntityColor),
        CollectionKind::Playlist,
        ThumbnailSize::Xl,
    )
    .into_any_element();
    let target = row.open.clone();
    let mut tile = SelectableTile::new(SharedString::from(row.element_id.clone()), artwork)
        .selected(false)
        .focused(false)
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
    tile.on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
        this.open_destination_row(&target, cx);
    }))
    .into_any_element()
}

/// The page of a public playlist, with its title and art at the top and
/// its entries below. `header_art` is the loaded art of the playlist.
/// `header_menu` is the "More actions" cell of the page. `status` is the
/// busy text while a copy runs (ADR 0089 packet 010). `rows` are the
/// entries, each from [`render_public_playlist_item`].
pub(crate) fn render_public_playlist_page_body(
    page: &PublicPlaylistPageVm,
    header_art: CollectionArtImage,
    header_menu: AnyElement,
    status: Option<&str>,
    rows: Vec<AnyElement>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(spacing::MD)
        .child(
            CollectionArt::new(header_art, CollectionKind::Playlist, ThumbnailSize::Xl)
                .into_any_element(),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(
                    Label::new(page.title.clone())
                        .size(FontSize::Body)
                        .weight(FontWeight::SEMIBOLD)
                        .truncated(),
                )
                .when_some(status, |el, status| {
                    el.child(
                        Label::new(status.to_owned())
                            .size(FontSize::Caption)
                            .color(SemanticColor::TertiaryLabel),
                    )
                }),
        )
        .child(header_menu);
    let content = page_scroll_content(cx)
        .gap(spacing::SM)
        .child(header)
        .children(rows);
    div()
        .id("public-playlist-page")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .px(spacing::MD)
        .pb(spacing::MD)
        .child(
            div()
                .id("public-playlist-page-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(content),
        )
        .into_any_element()
}

/// One entry of a public playlist page (ADR 0089 packet 009). A row with a
/// target opens its track page or its release page. A row with no target
/// shows its text only.
pub(crate) fn render_public_playlist_item(
    item: &PublicPlaylistItemVm,
    art: CollectionArtImage,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let label = format!("{}. {}", item.number, item.label);
    let row =
        ListRow::new(SharedString::from(item.element_id.clone())).a11y_label(ListRowA11yLabel {
            label: SharedString::from(item.a11y_label.clone()),
        });
    let row = match item.target.clone() {
        Some(target) => row.on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
            this.open_page(target.entry(), NavigationPathStart::CurrentPath, cx);
        })),
        None => row,
    };
    row.child(
        CollectionArt::new(art, CollectionKind::SinglesGroup, ThumbnailSize::Sm).into_any_element(),
    )
    .child(
        div()
            .flex_1()
            .min_w_0()
            .child(
                Label::new(label)
                    .size(FontSize::Micro)
                    .weight(FontWeight::MEDIUM)
                    .truncated(),
            )
            .when(!item.secondary_text.is_empty(), |el| {
                el.child(
                    Label::new(item.secondary_text.clone())
                        .size(FontSize::Micro)
                        .color(SemanticColor::TertiaryLabel)
                        .truncated(),
                )
            }),
    )
    .into_any_element()
}

/// The loading state of a public playlist page (ADR 0089 packet 009).
pub(crate) fn render_public_playlist_loading() -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .p(spacing::LG)
        .child(
            Label::new("Loading playlist")
                .size(FontSize::Caption)
                .color(SemanticColor::TertiaryLabel),
        )
        .into_any_element()
}

/// The error state of a public playlist page, with "Try again" (ADR 0089
/// packet 009). `detail` names the cause, and `retry` sends the read
/// again.
pub(crate) fn render_public_playlist_failed(
    detail: String,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(spacing::XS)
        .p(spacing::LG)
        .child(
            Label::new("MusicIndex did not answer the playlist request.")
                .size(FontSize::Caption)
                .color(SemanticColor::Danger),
        )
        .child(
            Label::new(detail)
                .size(FontSize::Micro)
                .color(SemanticColor::TertiaryLabel),
        )
        .child(
            UiButton::styled(
                SharedString::from("music-public-playlist-retry"),
                ControlStyle::Ghost,
            )
            .label("Try again")
            .a11y_label("Send the MusicIndex playlist request again")
            .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                this.retry_public_playlist(cx);
            })),
        )
        .into_any_element()
}

/// Renders one item of the Public playlists list in the Rows view (ADR 0089
/// packet 009): a playlist row with the logo frame `art`, or the status row
/// of the page. A row opens its playlist page. `row_menu` is the menu cell of
/// a row (ADR 0089 packet 010).
pub(crate) fn render_public_playlist_list_item(
    item: PublicPlaylistListItemVm,
    art: Option<CollectionArtImage>,
    row_menu: Option<RowMenu>,
    actions: IndexListActions,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    match item {
        PublicPlaylistListItemVm::Row(row) => {
            let art = RowArt::Collection(collection_art(art, CollectionKind::Playlist));
            render_destination_list_row(
                row.row,
                art,
                LeadingDisclosure::None,
                DestinationRowMenu::Playlist(row_menu),
                cx,
            )
        }
        PublicPlaylistListItemVm::Status(row) => render_index_list_status_row(row, actions, cx),
    }
}
