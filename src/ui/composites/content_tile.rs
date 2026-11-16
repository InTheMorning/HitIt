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
use crate::ui::tokens::{FontSize, SemanticColor};

/// The artwork of a content tile: the cover, or the logo shape when no image
/// loaded. The Releases list tiles share it (ADR 0089 packet 005 Part
/// C).
pub(crate) fn render_content_list_tile_artwork(
    thumbnail: Option<Arc<Image>>,
    artwork_size: gpui::Pixels,
    cx: &App,
) -> AnyElement {
    match thumbnail {
        // ADR 0101 Decision 4: `Size::ContentTileArtwork` shares the `Xl`
        // footprint (152 px), so `ImageSize::Xl` renders the same dimension
        // and gives the content tile its artwork shadow. The primitive sets
        // the radius from the edge (R-ART-4 item 2).
        Some(image) => ImagePrimitive::new(image)
            .size(ImageSize::Xl)
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

#[cfg(test)]
mod tests {
    /// musicindex ADR 0021 R-ART-4 item 2: the tile image sets no `Radius`
    /// role and no radius of its own. The image primitive gives it the art
    /// radius of its edge, as it does for `Thumbnail`.
    #[test]
    fn r_art_4_content_tile_uses_the_art_radius_function() {
        let source = include_str!("content_tile.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            !production.contains("Radius::") && !production.contains(".radius("),
            "R-ART-4 item 2: the tile image must use the art radius of the image primitive. \
             Fix: do not set a radius or a Radius role here."
        );
        let primitive = include_str!("../primitives/image.rs");
        assert!(
            primitive.contains("ArtworkRadius::for_edge(dim)"),
            "R-ART-4 item 2: the image primitive must use ArtworkRadius. \
             Fix: call ArtworkRadius::for_edge with the edge."
        );
    }
}
