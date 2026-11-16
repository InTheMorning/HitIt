//! Progress bar primitive for a task with a known length (ADR 0101 Decision
//! 12, musicindex ADR 0026 R-LOADING-1 item 3).
//!
//! The bar has a track, a fill and a count text, for example "3 of 9 done". The
//! count text carries the meaning, so the bar needs no color to be read. The
//! bar does not move by itself, so Reduce Motion does not change it. The text
//! is a [`Label`], so it records the fit probe (ADR 0097 Decision 4).

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, px, relative, App, IntoElement, RenderOnce, SharedString, Window};

use super::label::{Label, LabelVariant};
use crate::ui::tokens::{resolve_color, Appearance, Radius, SemanticColor, Spacing};

/// The height of the track.
pub(crate) const TRACK_HEIGHT: f32 = 6.0;

/// The minimum width of the track. A narrow window shrinks the track, not
/// the count text.
pub(crate) const TRACK_MIN_WIDTH: f32 = 48.0;

/// A bar and its count text.
#[derive(IntoElement)]
#[must_use]
pub struct ProgressBar {
    fraction: f32,
    count_text: SharedString,
    appearance: Option<Appearance>,
}

impl ProgressBar {
    /// Builds a bar that is `fraction` full (0.0 to 1.0) with the text
    /// `count_text`.
    pub fn new(fraction: f32, count_text: impl Into<SharedString>) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            count_text: count_text.into(),
            appearance: None,
        }
    }

    /// Overrides the resolved appearance.
    pub const fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let track = resolve_color(cx, SemanticColor::SystemFill, self.appearance);
        let fill = resolve_color(cx, SemanticColor::InfoLabel, self.appearance);
        let radius = Radius::SM.scaled(cx);
        // `SystemFill` is one step above `TertiaryFill`. `TertiaryFill` is the
        // color of a download item on Dark, so a track in it does not show.
        let track_box = div()
            .flex_1()
            .min_w(px(TRACK_MIN_WIDTH))
            .h(px(TRACK_HEIGHT))
            .rounded(radius)
            .bg(track)
            .overflow_hidden();
        let fill_box = div().h_full().w(relative(self.fraction)).bg(fill);
        #[cfg(test)]
        let (track_box, fill_box) = (
            track_box.debug_selector(|| "progress-bar-track".to_owned()),
            fill_box.debug_selector(|| "progress-bar-fill".to_owned()),
        );
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::SM.scaled(cx))
            .min_w_0()
            .child(track_box.child(fill_box))
            .child(
                div()
                    .flex_shrink_0()
                    .child(Label::new(self.count_text).variant(LabelVariant::Caption)),
            )
    }
}

#[cfg(test)]
mod tests {
    use gpui::{div, px, size, Context, ParentElement, Render, Styled, Window};

    use super::ProgressBar;
    use crate::ui::primitives::fit_probe;

    #[test]
    fn fraction_is_kept_between_zero_and_one() {
        assert!((ProgressBar::new(1.5, "x").fraction - 1.0).abs() < f32::EPSILON);
        assert!(ProgressBar::new(-1.0, "x").fraction.abs() < f32::EPSILON);
    }

    struct Case;

    impl Render for Case {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
            div()
                .size_full()
                .font_family(crate::ui::fonts::FIGTREE_FAMILY)
                .child(ProgressBar::new(0.33, "12345 of 99999"))
        }
    }

    /// ADR 0097 Decision 4: the count text of the bar records a fit probe and
    /// fits at the narrowest window width.
    #[test]
    fn adr_0097_the_progress_bar_fits_at_the_narrowest_width() {
        let mut cx = fit_probe::figtree_test_context();
        cx.update(gpui_component::init);
        let (_, window) = cx.add_window_view(|_, _| Case);
        window.simulate_resize(size(px(390.), px(300.)));
        window.run_until_parked();
        fit_probe::clear();
        window.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let (count, faults) = (fit_probe::count(), fit_probe::faults());
        cx.quit();
        assert!(
            count > 0,
            "ADR 0097 Decision 4: the bar recorded no fit probe."
        );
        assert!(faults.is_empty(), "{faults:#?}");
    }
}
