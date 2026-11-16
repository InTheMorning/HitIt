//! Placeholder primitive for rows that are not yet loaded.
//!
//! `Skeleton` is the design-system answer to "we asked the runtime for this
//! row but the page has not arrived yet". It is a token-driven, muted block
//! that occupies the row's footprint so the surrounding list does not jump
//! when the real content materializes.
//!
//! Sheen (ADR 0101 Decision 12, musicindex ADR 0026 R-LOADING-1): a soft band
//! moves across each block in [`SHEEN_PERIOD`]. The band is two gradients
//! that fade in and out. With Reduce Motion the block is static
//! ([`sheen_runs`], musicindex ADR 0068 R-A11Y-1). The animation asks for
//! frames only while a skeleton block is in the element tree, so a page with
//! no skeleton draws no extra frame.

#![warn(clippy::pedantic)]

use std::time::Duration;

use gpui::{
    div, linear_color_stop, linear_gradient, prelude::*, px, relative, Animation, AnimationExt,
    AnyElement, App, IntoElement, Pixels, RenderOnce, Window,
};

use crate::ui::tokens::{resolve_color, Appearance, Environment, Radius, SemanticColor, Spacing};

/// The time that the sheen needs to cross a block, from the left edge to the
/// right edge.
const SHEEN_PERIOD: Duration = Duration::from_millis(1600);

/// The width of the sheen band, as a fraction of the width of the block.
const SHEEN_BAND_WIDTH: f32 = 0.4;

/// The opacity of the sheen at the center of the band. The color is the
/// `Label` token, so the band is light on Dark and dark on Light.
const SHEEN_ALPHA: f32 = 0.08;

/// Returns whether the skeleton shows the sheen. Reduce Motion gives a
/// static block (ADR 0101 Decision 11, musicindex ADR 0068 R-A11Y-1).
const fn sheen_runs(environment: Environment) -> bool {
    environment.allows_motion()
}

/// The left edge of the sheen band at the animation time `delta` (0 to 1), as
/// a fraction of the width of the block. The band starts fully outside the
/// left edge and ends fully outside the right edge.
fn sheen_left(delta: f32) -> f32 {
    -SHEEN_BAND_WIDTH + delta * (1.0 + SHEEN_BAND_WIDTH)
}

/// The sheen band: two gradients, one that fades in and one that fades out.
/// `tint` is the center color of the band.
fn sheen_band(tint: gpui::Rgba) -> AnyElement {
    let clear = gpui::Rgba { a: 0.0, ..tint };
    let half = |from: gpui::Rgba, to: gpui::Rgba| {
        div().h_full().w(relative(0.5)).bg(linear_gradient(
            90.0,
            linear_color_stop(from, 0.0),
            linear_color_stop(to, 1.0),
        ))
    };
    div()
        .absolute()
        .top_0()
        .h_full()
        .w(relative(SHEEN_BAND_WIDTH))
        .flex()
        .flex_row()
        .child(half(clear, tint))
        .child(half(tint, clear))
        .with_animation(
            "skeleton-sheen",
            Animation::new(SHEEN_PERIOD).repeat(),
            |band, delta| band.left(relative(sheen_left(delta))),
        )
        .into_any_element()
}

/// Default height of a single text-row skeleton, matching the body text
/// line-height used throughout the library list views.
const DEFAULT_ROW_HEIGHT: f32 = 16.0;

/// Default width of a single text-row skeleton. Rows in lists usually
/// stretch via flex; this is just a sensible fallback when the parent does
/// not constrain the width.
const DEFAULT_ROW_WIDTH: f32 = 160.0;

/// Token-driven placeholder block used while paged rows are loading.
#[derive(IntoElement)]
#[must_use]
pub struct Skeleton {
    width: Pixels,
    height: Pixels,
    radius: Radius,
    appearance: Option<Appearance>,
    full_width: bool,
}

impl Skeleton {
    /// Builds a skeleton block with explicit `width` × `height` in pixels.
    pub fn block(width: Pixels, height: Pixels) -> Self {
        Self {
            width,
            height,
            radius: Radius::SM,
            appearance: None,
            full_width: false,
        }
    }

    /// Builds a single-text-row skeleton sized to match body text height.
    pub fn row() -> Self {
        Self::block(px(DEFAULT_ROW_WIDTH), px(DEFAULT_ROW_HEIGHT))
    }

    /// Lets the skeleton stretch to fill its parent's available width.
    /// Useful when rendered inside a flex row that owns the horizontal axis.
    pub const fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Override the corner radius. Defaults to [`Radius::SM`] which matches
    /// chip / badge corners.
    pub const fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// Override the resolved appearance. Mirrors the helper on
    /// [`super::LoadingMessage`] for consistency across loading affordances.
    pub const fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = resolve_color(cx, SemanticColor::TertiaryFill, self.appearance);
        let sheen = sheen_runs(Environment::current(cx)).then(|| {
            let label = resolve_color(cx, SemanticColor::Label, self.appearance);
            sheen_band(gpui::Rgba {
                a: SHEEN_ALPHA,
                ..label
            })
        });
        let mut block = div()
            .h(self.height)
            .bg(fill)
            .rounded(self.radius.scaled(cx))
            // A small vertical gutter so consecutive skeleton rows do not
            // touch and read as a single bar.
            .my(Spacing::XXS.scaled(cx));

        if self.full_width {
            block = block.flex_1();
        } else {
            block = block.w(self.width);
        }

        match sheen {
            // The block clips the band at its edges.
            Some(band) => block.relative().overflow_hidden().child(band),
            None => block,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R-LOADING-1 item 4: the skeleton asks for the sheen, and so for
    /// frames, only when Reduce Motion is off.
    #[test]
    fn sheen_runs_only_when_motion_is_allowed() {
        let moving = Environment::default();
        let reduced = Environment {
            reduce_motion: true,
            ..Environment::default()
        };
        assert!(sheen_runs(moving));
        assert!(!sheen_runs(reduced));
    }

    #[test]
    fn sheen_band_crosses_the_block_from_edge_to_edge() {
        assert!((sheen_left(0.0) + SHEEN_BAND_WIDTH).abs() < f32::EPSILON);
        assert!((sheen_left(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn block_uses_explicit_dimensions() {
        let skeleton = Skeleton::block(px(120.0), px(20.0));
        assert_eq!(skeleton.width, px(120.0));
        assert_eq!(skeleton.height, px(20.0));
        assert!(!skeleton.full_width);
    }

    #[test]
    fn row_uses_text_height_default() {
        let skeleton = Skeleton::row();
        assert_eq!(skeleton.height, px(DEFAULT_ROW_HEIGHT));
        assert_eq!(skeleton.width, px(DEFAULT_ROW_WIDTH));
    }

    #[test]
    fn full_width_flag_flips() {
        let skeleton = Skeleton::row().full_width();
        assert!(skeleton.full_width);
    }

    #[test]
    fn radius_override_is_recorded() {
        let skeleton = Skeleton::row().radius(Radius::MD);
        assert_eq!(skeleton.radius, Radius::MD);
    }

    #[test]
    fn appearance_override_is_recorded() {
        let skeleton = Skeleton::row().appearance(Appearance::Dark);
        assert_eq!(skeleton.appearance, Some(Appearance::Dark));
    }

    #[test]
    fn defaults_have_no_appearance_override() {
        let skeleton = Skeleton::row();
        assert!(skeleton.appearance.is_none());
        assert_eq!(skeleton.radius, Radius::SM);
    }
}
