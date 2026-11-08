//! The shared parts of a content tile (ADR 0089 packet 005 Part C, ADR 0090
//! task 004, musicindex ADR 0053 R-LIST-9).
//!
//! A tile of the Releases list, of a group page, of a public playlist and of
//! a release section has the same artwork and the same text lines. The
//! caller gives the cover and the text. These two functions draw them with
//! named tokens.

#![warn(clippy::pedantic)]

use std::sync::Arc;

use gpui::{div, prelude::*, AnyElement, App, FontWeight, Image};

use crate::ui::composites::thumbnail::logo_placeholder;
use crate::ui::composites::EntityKind;
use crate::ui::primitives::{Image as ImagePrimitive, ImageSize, Label};
use crate::ui::tokens::{FontSize, Radius, SemanticColor};

/// The artwork of a content tile: the cover, or the logo shape when no image
/// loaded. The Releases list tiles share it (ADR 0089 packet 005 Part
/// C).
pub(crate) fn render_content_list_tile_artwork(
    thumbnail: Option<Arc<Image>>,
    artwork_size: gpui::Pixels,
    cx: &App,
) -> AnyElement {
    match thumbnail {
        // ADR 0083 Decision 4: `Size::ContentTileArtwork` shares the `Xl`
        // footprint (152 px), so `ImageSize::Xl` renders the same dimension
        // and gives the content tile its artwork shadow.
        Some(image) => ImagePrimitive::new(image)
            .size(ImageSize::Xl)
            .radius(Radius::MD)
            .into_any_element(),
        // musicindex ADR 0021 R-ART-2: a release tile with no cover, or with a
        // cover that did not load, shows the logo shape. Only artwork has a
        // shadow (R-ART-4 item 3).
        None => logo_placeholder(EntityKind::Release, artwork_size, cx),
    }
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
