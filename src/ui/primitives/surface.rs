//! Container primitive — the canonical "panel / card / popover body" surface.
//!
//! `Surface` is a token-driven wrapper around `div()` that picks an
//! appropriate background, border and corner radius based on a semantic
//! [`SurfaceElevation`]. Use it for popover content, dialog bodies, sidebars,
//! cards, and any other raised container.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, px, AnyElement, App, Div, IntoElement, Pixels, RenderOnce, Window};

use crate::ui::tokens::{
    resolve_color, Appearance, FloatingShadow, Radius, SemanticColor, Spacing,
};

/// The width of the border of a [`Surface`]. A caller that fits a surface in
/// a given height counts two borders.
pub const SURFACE_BORDER_WIDTH: Pixels = px(1.0);

/// HIG-aligned elevation tiers. Each tier maps to a different background +
/// border combination so nested surfaces remain visually distinguishable
/// without relying on shadow alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceElevation {
    /// Sits flat on the canvas — the group level with a hairline
    /// separator. Used for inline cards.
    Sunken,
    /// Raised panel — the group level with a stronger border and rounded
    /// corners. Used for a group or a card.
    Raised,
    /// A layer above the content — the floating fill and the floating
    /// shadow (ADR 0101 Decision 1 and Decision 4). Used for a popover, a
    /// menu and a sheet.
    Floating,
}

#[derive(IntoElement)]
#[must_use]
pub struct Surface {
    elevation: SurfaceElevation,
    padding: Spacing,
    radius: Radius,
    appearance: Option<Appearance>,
    children: Vec<AnyElement>,
}

impl Surface {
    pub fn new(elevation: SurfaceElevation) -> Self {
        let radius = match elevation {
            SurfaceElevation::Sunken => Radius::MD,
            SurfaceElevation::Raised | SurfaceElevation::Floating => Radius::LG,
        };
        Self {
            elevation,
            padding: Spacing::MD,
            radius,
            appearance: None,
            children: Vec::new(),
        }
    }

    pub fn padding(mut self, padding: Spacing) -> Self {
        self.padding = padding;
        self
    }

    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }
}

impl ParentElement for Surface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Surface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let bg = self.elevation.fill();
        let border = match self.elevation {
            SurfaceElevation::Sunken => SemanticColor::Separator,
            SurfaceElevation::Raised | SurfaceElevation::Floating => SemanticColor::OpaqueSeparator,
        };

        let mut el: Div = div()
            .bg(resolve_color(cx, bg, self.appearance))
            .border(SURFACE_BORDER_WIDTH)
            .border_color(resolve_color(cx, border, self.appearance))
            .rounded(self.radius.scaled(cx))
            .p(self.padding.scaled(cx));

        if self.elevation == SurfaceElevation::Floating {
            el = el.shadow(FloatingShadow::shadow(cx, self.appearance));
        }

        el.extend(self.children);
        el
    }
}

impl SurfaceElevation {
    /// The fill token of this elevation (ADR 0101 Decision 1).
    #[must_use]
    pub const fn fill(self) -> SemanticColor {
        match self {
            Self::Sunken | Self::Raised => SemanticColor::GroupBackground,
            Self::Floating => SemanticColor::FloatingBackground,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::tokens::Appearance;

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a color channel is between 0 and 255 after rounding"
    )]
    fn rgb(token: SemanticColor, appearance: Appearance) -> u32 {
        let color = token.resolve(appearance);
        let channel = |value: f32| (value * 255.0).round() as u32;
        (channel(color.r) << 16) | (channel(color.g) << 8) | channel(color.b)
    }

    /// ADR 0101 Decision 1: each elevation gives its level in each theme.
    /// Fix: map the elevation in `SurfaceElevation::fill`.
    #[test]
    fn each_elevation_gives_its_level_in_each_theme() {
        let cases = [
            (SurfaceElevation::Sunken, 0x1c_1c1f, 0xf2_f2f7),
            (SurfaceElevation::Raised, 0x1c_1c1f, 0xf2_f2f7),
            (SurfaceElevation::Floating, 0x26_262a, 0xff_ffff),
        ];
        for (elevation, dark, light) in cases {
            assert_eq!(
                rgb(elevation.fill(), Appearance::Dark),
                dark,
                "{elevation:?}"
            );
            assert_eq!(
                rgb(elevation.fill(), Appearance::Light),
                light,
                "{elevation:?}"
            );
        }
    }
}
