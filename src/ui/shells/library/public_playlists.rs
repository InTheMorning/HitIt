//! The item page of a public playlist (ADR 0089 Decision 1, packet 009).
//!
//! The view model gives the title, the art, each entry and each text. This
//! shell owns only the layout and the click wiring. It reads no data. The
//! status of the page (loading, failed) is drawn here from the view model
//! state, with its "Try again" action.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, SharedString, Styled};

use crate::library::LibraryApp;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::selectable_tile::SelectableTile;
use crate::ui::composites::{
    CollectionArt, CollectionArtImage, ListRow, ListRowA11yLabel, ThumbnailSize,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button as UiButton, Label};
use crate::ui::shells::library::releases_tiles::tile_text_line;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::collection_art::CollectionKind;
use crate::view_models::library::DestinationListRowVm;
use crate::view_models::public_playlists::{PublicPlaylistItemVm, PublicPlaylistPageVm};
use crate::view_models::workspace::NavigationPathStart;

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
/// `rows` are the entries, each from [`render_public_playlist_item`].
pub(crate) fn render_public_playlist_page_body(
    page: &PublicPlaylistPageVm,
    header_art: CollectionArtImage,
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
            div().flex_1().min_w_0().child(
                Label::new(page.title.clone())
                    .size(FontSize::Body)
                    .weight(FontWeight::SEMIBOLD)
                    .truncated(),
            ),
        );
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
