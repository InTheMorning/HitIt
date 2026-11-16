//! Cover backdrop composite: a page header over a glow from its cover
//! (ADR 0101 Decision 4, task 002; musicindex ADR 0021 R-ART-5).
//!
//! The glow box starts at the top of the scroll content of the page. It
//! has the full width of the page column, with no padding at its sides,
//! and the veil and the window edges stop it. The box holds the prepared
//! glow image, a pattern of grain tiles and two stacked gradients of the
//! veil. The header draws above the box at its own position. The glow
//! image comes from `media::header_glow`, with its blur, saturation and
//! opacity already in the pixels. The grain is one tile of its own size
//! and alpha. The page opacity does not change it.
//!
//! - With no prepared glow, the composite draws nothing. The veil on the
//!   page background shows the background, and the header does not move
//!   when the glow arrives.
//! - With Reduce Motion, an animated glow shows its first frame (musicindex
//!   ADR 0028 R-ART-6).
//! - With Reduce Transparency, the box has an opaque fill of the cover
//!   color. It has no grain and no blur (musicindex ADR 0068 R-A11Y-1
//!   item 3).

#![warn(clippy::pedantic)]

use std::sync::Arc;

use gpui::{
    div, img, relative, AnyElement, App, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, RenderImage, RenderOnce, SharedString, Styled, StyledImage, Window,
};

use crate::media::cover_color::CoverColor;
use crate::media::header_glow::GlowImage;
use crate::ui::tokens::{CoverBackdrop as CoverBackdropToken, Environment};

/// What the glow box draws below the veil.
#[derive(Clone)]
pub(crate) enum GlowLayer {
    /// The glow image at its calculated opacity, and the grain tile above
    /// it. `animated` is true when the image has more than one frame and
    /// the environment allows motion.
    Image {
        image: Arc<RenderImage>,
        grain: Arc<RenderImage>,
        opacity: f32,
        animated: bool,
    },
    /// An opaque fill of the cover color. It has no grain and no blur.
    Opaque(CoverColor),
}

/// Selects the layer of `glow` for `environment`. `None` draws no layer.
pub(crate) fn glow_layer(glow: Option<&GlowImage>, environment: Environment) -> Option<GlowLayer> {
    let glow = glow?;
    if environment.reduce_transparency {
        return glow.color().map(GlowLayer::Opaque);
    }
    let allows_motion = environment.allows_motion();
    Some(GlowLayer::Image {
        image: Arc::clone(glow.image(allows_motion)),
        grain: Arc::clone(glow.grain()),
        opacity: glow.opacity(),
        animated: allows_motion && glow.frame_count() > 1,
    })
}

#[derive(IntoElement)]
#[must_use]
pub struct CoverBackdrop {
    glow: Option<Arc<GlowImage>>,
    header: AnyElement,
}

impl CoverBackdrop {
    pub fn new(glow: Option<Arc<GlowImage>>, header: AnyElement) -> Self {
        Self { glow, header }
    }
}

impl RenderOnce for CoverBackdrop {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let environment = Environment::current(cx);
        let backdrop = glow_layer(self.glow.as_deref(), environment).map(|layer| {
            let (upper_veil, lower_veil) = CoverBackdropToken::veil(cx);
            let box_height = CoverBackdropToken::box_height(window.viewport_size().width, cx);
            let extend_top = CoverBackdropToken::extend_top(cx);
            let extend_side = CoverBackdropToken::extend_side(cx);
            let extend_bottom = CoverBackdropToken::extend_bottom(cx);
            let art = div()
                .absolute()
                .top(-extend_top)
                .bottom(-extend_bottom)
                .left(-extend_side)
                .right(-extend_side);
            let art = match layer {
                GlowLayer::Opaque(color) => {
                    art.bg(CoverBackdropToken::tone(color, environment.appearance))
                }
                GlowLayer::Image {
                    image,
                    grain,
                    opacity,
                    animated,
                } => {
                    let picture = img(ImageSource::Render(Arc::clone(&image)))
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                        .opacity(opacity);
                    let art = if animated {
                        // An animated image needs a stable id to tick its frames.
                        art.child(
                            picture.id(SharedString::from(format!("header-glow:{}", image.id.0))),
                        )
                    } else {
                        art.child(picture)
                    };
                    let (columns, rows) = CoverBackdropToken::grain_cells(
                        window.viewport_size().width + extend_side * 2.0,
                        box_height + extend_top + extend_bottom,
                    );
                    art.child(grain_pattern(&grain, columns, rows))
                }
            };
            div()
                .absolute()
                .top(-CoverBackdropToken::page_inset_start(cx))
                .left(-CoverBackdropToken::page_inset_start(cx))
                .right(-CoverBackdropToken::page_inset_end(cx))
                .h(box_height)
                .overflow_hidden()
                .child(art)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .h(relative(CoverBackdropToken::VEIL_MID_AT))
                        .bg(upper_veil),
                )
                .child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .h(relative(1.0 - CoverBackdropToken::VEIL_MID_AT))
                        .bg(lower_veil),
                )
        });
        div().relative().children(backdrop).child(self.header)
    }
}

/// A pattern of `columns` by `rows` grain tiles. Each tile has its own size
/// and alpha, and no opacity is set on it, so the page opacity of the glow
/// does not change the grain. The pattern starts at the top left of the art
/// and the art edge clips the last column and row.
fn grain_pattern(grain: &Arc<RenderImage>, columns: usize, rows: usize) -> AnyElement {
    let side = CoverBackdropToken::grain_tile_side();
    let mut pattern = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .overflow_hidden()
        .flex()
        .flex_col();
    for _ in 0..rows {
        let mut row = div().flex().flex_row().flex_shrink_0().h(side);
        for _ in 0..columns {
            row = row.child(
                img(ImageSource::Render(Arc::clone(grain)))
                    .w(side)
                    .h(side)
                    .flex_shrink_0()
                    .object_fit(ObjectFit::Fill),
            );
        }
        pattern = pattern.child(row);
    }
    pattern.into_any_element()
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;
    use crate::media::header_glow::{prepare, GlowFrame, GlowSource, PreparedGlow, SourceFrame};

    fn animated_glow() -> GlowImage {
        let style = CoverBackdropToken::glow_style(Environment::default());
        let frames = [[200, 30, 40], [30, 200, 40], [30, 40, 200]]
            .into_iter()
            .map(|color| SourceFrame {
                pixels: RgbaImage::from_pixel(8, 8, Rgba([color[0], color[1], color[2], 255])),
                delay_ms: 40,
            })
            .collect();
        let glow = prepare(&GlowSource::Image(frames), false, [135, 92, 207], &style);
        assert_eq!(glow.frames.len(), 3);
        GlowImage::new(&glow)
    }

    /// M6 (ADR 0101 Decision 4, task 002): with Reduce Motion, an animated
    /// glow gives its first frame, and with motion it gives each frame.
    #[test]
    fn adr_0101_header_glow_m6_reduce_motion_draws_the_first_frame() {
        let glow = animated_glow();
        let moving = Environment::default();
        let still = Environment {
            reduce_motion: true,
            ..Environment::default()
        };
        let Some(GlowLayer::Image {
            image, animated, ..
        }) = glow_layer(Some(&glow), moving)
        else {
            panic!("motion draws the glow image");
        };
        assert_eq!(image.frame_count(), 3);
        assert!(animated, "the image must tick its frames");
        let Some(GlowLayer::Image {
            image, animated, ..
        }) = glow_layer(Some(&glow), still)
        else {
            panic!("Reduce Motion draws the glow image");
        };
        assert_eq!(image.frame_count(), 1, "Reduce Motion shows one frame");
        assert!(!animated);
    }

    /// M6: with Reduce Transparency, the glow gives an opaque fill of the
    /// cover color and no image, so there is no grain and no blur.
    #[test]
    fn adr_0101_header_glow_m6_reduce_transparency_draws_an_opaque_fill() {
        let glow = animated_glow();
        let environment = Environment {
            reduce_transparency: true,
            ..Environment::default()
        };
        assert!(
            matches!(
                glow_layer(Some(&glow), environment),
                Some(GlowLayer::Opaque(_))
            ),
            "Reduce Transparency must draw the opaque fill"
        );
        let colorless = GlowImage::new(&PreparedGlow {
            frames: vec![GlowFrame {
                pixels: RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 255])),
                delay_ms: 40,
            }],
            opacity: 0.5,
            grain_opacity: 0.2,
            color: None,
        });
        assert!(
            glow_layer(Some(&colorless), environment).is_none(),
            "a glow with no color draws no fill"
        );
        assert!(glow_layer(None, environment).is_none());
        assert!(glow_layer(None, Environment::default()).is_none());
    }

    /// Shows a header in the backdrop, so that a test can read its place.
    struct HeaderPlace(Option<Arc<GlowImage>>);

    impl gpui::Render for HeaderPlace {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div().size_full().child(CoverBackdrop::new(
                self.0.clone(),
                div()
                    .debug_selector(|| "cover-backdrop-header".to_owned())
                    .h(gpui::px(100.0))
                    .into_any_element(),
            ))
        }
    }

    /// Task 002 correction 2: the backdrop adds no padding. The header
    /// keeps its place with a glow and without one.
    #[gpui::test]
    fn adr_0101_header_glow_backdrop_keeps_the_header_in_place(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let glow = Arc::new(animated_glow());
        let mut places = Vec::new();
        for glow in [None, Some(glow)] {
            let (_, cx) = cx.add_window_view(|_, _| HeaderPlace(glow.clone()));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            places.push(
                cx.debug_bounds("cover-backdrop-header")
                    .expect("the header renders"),
            );
        }
        assert_eq!(
            places[0], places[1],
            "ADR 0101 Decision 4 (task 002 correction 2): CoverBackdrop must add no padding, so the header keeps its place when the glow arrives."
        );
        assert_eq!(places[0].origin, gpui::point(gpui::px(0.0), gpui::px(0.0)));
    }
}
