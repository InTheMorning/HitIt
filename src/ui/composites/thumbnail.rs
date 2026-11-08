//! Thumbnail composite — square artwork tile. With no image, it shows the
//! logo shape in 45 % of the entity color (musicindex ADR 0021 R-ART-2), with
//! no letters and no icon.
//!
//! Sizing is expressed as a semantic [`ThumbnailSize`] (`Sm` / `Md` / `Lg` / `Xl` / `XXl`) which
//! resolves through the global scale, so a "Large" thumbnail in a
//! detail header automatically grows when the user picks a larger UI scale.

#![warn(clippy::pedantic)]

use std::sync::Arc;

use gpui::{
    div, svg, AnyElement, App, Image, IntoElement, ParentElement, Pixels, RenderOnce, Rgba, Styled,
    Window,
};

use crate::ui::icons::LOGO_FRAME_ASSET_PATH;
use crate::ui::primitives::{Image as ImagePrimitive, ImageSize};
use crate::ui::tokens::{ArtworkShape, Radius, ScaleFactor};

use super::tag_badge::EntityKind;

/// Semantic thumbnail sizes. Base (1.0×) values follow Apple HIG list /
/// detail-row conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailSize {
    /// 32 px — list rows, search results.
    Sm,
    /// 48 px — sidebar and now-playing strips.
    Md,
    /// 80 px — detail-view header.
    Lg,
    /// 152 px — the content tile artwork, with the artwork shadow. It has
    /// the footprint of `Size::ContentTileArtwork` (ADR 0086 Decision 3).
    Xl,
    /// 200 px — the album page header cover, with the artwork shadow (ADR
    /// 0083 Decision 4).
    XXl,
}

impl ThumbnailSize {
    fn base(self) -> f32 {
        match self {
            Self::Sm => 32.0,
            Self::Md => 48.0,
            Self::Lg => 80.0,
            Self::Xl => 152.0,
            Self::XXl => 200.0,
        }
    }

    /// The image primitive size of this thumbnail size. The primitive draws
    /// the artwork shadow of the size.
    #[must_use]
    pub const fn image_size(self) -> ImageSize {
        match self {
            Self::Sm => ImageSize::Sm,
            Self::Md => ImageSize::Md,
            Self::Lg => ImageSize::Lg,
            Self::Xl => ImageSize::Xl,
            Self::XXl => ImageSize::XXl,
        }
    }

    /// Pick a [`ThumbnailSize`] from a legacy raw-pixel hint.
    ///
    /// Used while migrating call sites that still pass `(size: f32, large: bool)`
    /// pairs. The mapping mirrors the historical `render_thumb` helper:
    /// the explicit `large` flag wins, then `>= 64` is `Lg`, then
    /// `>= 40` is `Md`, else `Sm`.
    #[must_use]
    pub fn from_legacy_px(size: f32, large: bool) -> Self {
        if large || size >= 64.0 {
            Self::Lg
        } else if size >= 40.0 {
            Self::Md
        } else {
            Self::Sm
        }
    }

    /// ADR 0039: artwork sizing is geometry — this resolves through CHROME.
    pub(crate) fn scaled(self, cx: &App) -> Pixels {
        gpui::px(self.base() * ScaleFactor::current(cx).chrome_multiplier())
    }

    pub(crate) fn radius(self) -> Radius {
        match self {
            Self::Sm | Self::Md => Radius::SM,
            Self::Lg | Self::Xl => Radius::MD,
            Self::XXl => Radius::LG,
        }
    }
}

#[derive(IntoElement)]
#[must_use]
pub struct Thumbnail {
    image: Option<Arc<Image>>,
    kind: EntityKind,
    size: ThumbnailSize,
}

impl Thumbnail {
    pub fn new(kind: EntityKind, size: ThumbnailSize) -> Self {
        Self {
            image: None,
            kind,
            size,
        }
    }

    pub fn image(mut self, image: Option<Arc<Image>>) -> Self {
        self.image = image;
        self
    }
}

impl RenderOnce for Thumbnail {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        match self.image {
            Some(image) => ImagePrimitive::new(image)
                .size(self.size.image_size())
                .radius(self.size.radius())
                .into_any_element(),
            None => logo_placeholder(self.kind, self.size.scaled(cx), cx),
        }
    }
}

/// The fill of the logo shape of `kind` (musicindex ADR 0021 R-ART-2 item 2).
#[must_use]
pub(crate) fn logo_placeholder_fill(kind: EntityKind, cx: &App) -> Rgba {
    Rgba {
        a: ArtworkShape::ENTITY_FILL_OPACITY,
        ..kind.fill_color(cx)
    }
}

/// The art of an item with no image, or with an image that did not load: the
/// logo shape in a square slot of edge `dim` (musicindex ADR 0021 R-ART-2). It
/// has no letters and no icon. Each art size uses it.
pub(crate) fn logo_placeholder(kind: EntityKind, dim: Pixels, cx: &App) -> AnyElement {
    div()
        .w(dim)
        .h(dim)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path(LOGO_FRAME_ASSET_PATH)
                .w(dim)
                .h(dim / ArtworkShape::LogoFrame.aspect())
                .text_color(logo_placeholder_fill(kind, cx)),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R83-55: each thumbnail size has the base dimension of its image
    /// primitive size, so the 200 px header cover draws the `XXl` shadow.
    #[test]
    fn adr_0083_thumbnail_sizes_match_their_image_sizes() {
        assert_eq!(ThumbnailSize::XXl.image_size(), ImageSize::XXl);
        for (size, base) in [
            (ThumbnailSize::Sm, 32.0),
            (ThumbnailSize::Md, 48.0),
            (ThumbnailSize::Lg, 80.0),
            (ThumbnailSize::Xl, 152.0),
            (ThumbnailSize::XXl, 200.0),
        ] {
            assert!((size.base() - base).abs() < f32::EPSILON, "{size:?}");
            assert!(
                (size.image_size().base_px() - base).abs() < f32::EPSILON,
                "{size:?}"
            );
        }
    }

    /// R86-2-32: the 152 px tile size uses the `Xl` image size, so a group
    /// tile has the footprint and the shadow of a release tile.
    #[test]
    fn adr_0086_window_groups_xl_thumbnail_uses_the_xl_image_size() {
        assert_eq!(ThumbnailSize::Xl.image_size(), ImageSize::Xl);
    }

    /// musicindex ADR 0021 R-ART-2, walk of 2026-11-07: rows and tiles with no
    /// image showed "AL" or an empty gray square. The placeholder is the logo
    /// shape, with 45 % of the entity color.
    #[test]
    fn r_art_2_placeholder_is_the_logo_shape_in_45_percent_of_the_entity_color() {
        assert!((ArtworkShape::ENTITY_FILL_OPACITY - 0.45).abs() < f32::EPSILON);
        assert_eq!(LOGO_FRAME_ASSET_PATH, "artwork/logo-frame.svg");
        let source = include_str!("thumbnail.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or_default();
        assert!(
            !production.contains("monogram"),
            "R-ART-2 item 4: no letters"
        );
    }
}
