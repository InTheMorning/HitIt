//! The selected look of musicindex ADR 0051 R-SELECT-1 (`HitIt` ADR 0101
//! Decision 2).
//!
//! A selected sidebar item, list row, segment and tab have one look: a
//! two-stop gradient from `selected-top` to `selected-bottom`, a 1 px inner
//! edge and a 1 px inner highlight at the top. The look has no other fill and
//! no outer shadow. The edge and the highlight are layers inside the border
//! box, so the border stays free for the focus ring and no element moves.
//!
//! [`selected_look`] is the one owner of this look. A caller must not draw a
//! selected state with `AccentTint`. A guard in `tests/architecture_tests.rs`
//! checks this.

#![warn(clippy::pedantic)]

use gpui::{
    div, linear_color_stop, linear_gradient, px, App, InteractiveElement, LinearColorStop,
    ParentElement, Pixels, Rgba, Styled,
};

use crate::ui::tokens::{resolve_color, Appearance, SemanticColor};

/// The largest inset of the highlight at each side. A pill has a large
/// radius, and the highlight must not reach the corner curve.
const MAX_HIGHLIGHT_INSET_PX: f32 = 12.0;

/// The four colors of the selected look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SelectedLookColors {
    pub(crate) top: Rgba,
    pub(crate) bottom: Rgba,
    pub(crate) edge: Rgba,
    pub(crate) highlight: Rgba,
}

impl SelectedLookColors {
    /// The colors of the current theme, or of an explicit appearance.
    pub(crate) fn resolve(cx: &App, appearance: Option<Appearance>) -> Self {
        Self {
            top: resolve_color(cx, SemanticColor::SelectedTop, appearance),
            bottom: resolve_color(cx, SemanticColor::SelectedBottom, appearance),
            edge: resolve_color(cx, SemanticColor::SelectedEdge, appearance),
            highlight: resolve_color(cx, SemanticColor::SelectedHighlight, appearance),
        }
    }

    /// The two stops of the gradient, from the top to the bottom. GPUI gives
    /// two stops, so the highlight is a layer and not a third stop.
    pub(crate) fn stops(self) -> [LinearColorStop; 2] {
        [
            linear_color_stop(self.top, 0.0),
            linear_color_stop(self.bottom, 1.0),
        ]
    }
}

/// The inset of the highlight at each side for a given corner radius.
pub(crate) fn highlight_inset(radius: Pixels) -> Pixels {
    radius.min(px(MAX_HIGHLIGHT_INSET_PX))
}

/// Gives the selected look to `el`. The element gets the gradient as its
/// only fill. The edge and the highlight are absolute layers over it.
/// `radius` is the corner radius of `el`.
pub(crate) fn selected_look<E>(el: E, radius: Pixels, appearance: Option<Appearance>, cx: &App) -> E
where
    E: Styled + ParentElement,
{
    let colors = SelectedLookColors::resolve(cx, appearance);
    let [from, to] = colors.stops();
    let inset = highlight_inset(radius);
    el.relative()
        .bg(linear_gradient(180.0, from, to))
        .child(
            div()
                .debug_selector(|| "selected-look-edge".to_owned())
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .rounded(radius)
                .border_1()
                .border_color(colors.edge),
        )
        .child(
            div()
                .debug_selector(|| "selected-look-highlight".to_owned())
                .absolute()
                .top(px(1.0))
                .left(inset)
                .right(inset)
                .h(px(1.0))
                .bg(colors.highlight),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Bounds, IntoElement, Render, TestAppContext, VisualTestContext, Window};

    struct LookView;

    impl Render for LookView {
        fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
            selected_look(
                div().debug_selector(|| "selected-look-host".to_owned()),
                px(8.0),
                None,
                cx,
            )
            .w(px(200.0))
            .h(px(40.0))
        }
    }

    fn bounds(cx: &mut VisualTestContext, name: &'static str) -> Bounds<Pixels> {
        cx.debug_bounds(name).expect("the layer records its bounds")
    }

    /// musicindex ADR 0051 R-SELECT-1 items 1 and 2: the look gives a gradient
    /// of the two tokens, and an edge and a highlight as 1 px layers. Fix: use
    /// `selected_look`, and give no other fill.
    #[gpui::test]
    fn selected_look_gives_the_gradient_the_edge_and_the_highlight(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let (_view, cx) = cx.add_window_view(|_, _| LookView);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let host = bounds(cx, "selected-look-host");
        let edge = bounds(cx, "selected-look-edge");
        let highlight = bounds(cx, "selected-look-highlight");
        assert_eq!(edge, host, "the edge layer must cover the element");
        assert_eq!(highlight.size.height, px(1.0));
        assert_eq!(highlight.origin.y, host.origin.y + px(1.0));
        assert_eq!(highlight.origin.x, host.origin.x + px(8.0));
        assert_eq!(highlight.size.width, host.size.width - px(16.0));
        cx.update(|_, cx| {
            let colors = SelectedLookColors::resolve(cx, None);
            let [from, to] = colors.stops();
            assert_eq!(from.color, colors.top.into());
            assert_eq!(to.color, colors.bottom.into());
            assert!(from.percentage.abs() < f32::EPSILON);
            assert!((to.percentage - 1.0).abs() < f32::EPSILON);
        });
    }

    #[test]
    fn highlight_inset_stops_at_the_corner_limit() {
        assert_eq!(highlight_inset(px(6.0)), px(6.0));
        assert_eq!(highlight_inset(px(999.0)), px(MAX_HIGHLIGHT_INSET_PX));
    }

    /// ADR 0101 Decision 11: a high-contrast profile gives two equal stops
    /// and an opaque edge.
    #[test]
    fn high_contrast_profiles_give_equal_stops_and_an_opaque_edge() {
        use crate::ui::theme_profiles::resolve_profile_color;
        for profile in [
            crate::theme_profile::ThemeProfile::HighContrastDark,
            crate::theme_profile::ThemeProfile::HighContrastLight,
        ] {
            let top = resolve_profile_color(profile, SemanticColor::SelectedTop);
            let bottom = resolve_profile_color(profile, SemanticColor::SelectedBottom);
            let edge = resolve_profile_color(profile, SemanticColor::SelectedEdge);
            assert_eq!(top, bottom, "{profile:?}");
            assert!((edge.a - 1.0).abs() < f32::EPSILON, "{profile:?}");
        }
    }
}
