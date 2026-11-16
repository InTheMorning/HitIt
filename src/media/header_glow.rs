//! The glow image of a page header (ADR 0101 Decision 4, musicindex ADR
//! 0021 R-ART-5).
//!
//! The app prepares the glow outside the renderer with the `image` crate.
//! GPUI 0.3.1 has no blur, no blend mode and no tiled image, so each step
//! of the website glow happens in the pixels: a small copy of the art, a
//! blur, a saturation and the muted blend of a black and white cover. The
//! prepared frames are opaque. They already include the page background
//! where a layer is translucent, so the renderer only draws the frames at
//! the opacity of [`PreparedGlow::opacity`].
//!
//! The film grain is not in the frames. A frame is small, and the renderer
//! stretches it over the page header, so a grain in it would grow with the
//! frame. The grain is one tile with its own size and its own alpha
//! ([`grain_tile`]). The renderer shows the tile in a pattern of tiles
//! above the glow, at the dimension of the tile, and the opacity of the
//! glow does not change it.
//!
//! One opacity value limits the glow of a page (ADR 0101 Decision 4,
//! R-ART-5 item 2). It is the highest value, with a maximum, at which the
//! text colors `Label` and `SecondaryLabel` keep 4.5:1 at the top of the
//! veil. The hue of the glow stays. The module reads no renderer type.

#![warn(clippy::pedantic)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use gpui::RenderImage;
use image::codecs::gif::GifDecoder;
use image::imageops::{self, FilterType};
use image::{AnimationDecoder, Delay, DynamicImage, Frame, Rgba, RgbaImage};

use crate::media::cover_color::{main_color, CoverColor};

/// The side of the reduced copy of a source image, in pixels. The blur
/// runs at this size.
pub const SMALL_SIDE: u32 = 96;
/// The side of the grain tile, in pixels. It is the grain tile of the
/// website, so the grain keeps its size on the screen.
pub const GRAIN_TILE_SIDE: u32 = 240;
/// The drawn width of the art on the website for which the blur is
/// stated, in pixels. It scales the blur to the reduced copy.
const BLUR_REFERENCE_WIDTH: f32 = 1060.0;
/// The side of the sample for the colorfulness and for the opacity.
const SAMPLE_SIDE: u32 = 24;
/// The side of one block of the sample for the opacity.
const BLOCK_SIDE: u32 = 6;
/// The most frames of an animated cover that the glow keeps. A cover with
/// more frames gives its first frame only, so the glow does not move out
/// of step with the cover.
pub const MAX_FRAMES: usize = 48;
/// The delay of a frame that gives none, in milliseconds.
const DEFAULT_DELAY_MS: u32 = 100;
/// The seed of the grain. One seed gives one grain, so the same input
/// gives the same output.
const GRAIN_SEED: u32 = 0x9e37_79b9;
/// The steps of the search for the opacity.
const OPACITY_STEPS: u32 = 24;
/// The contrast that the text keeps at the top of the veil.
const TEXT_CONTRAST: f32 = 4.5;

/// The values of one glow. The page header tokens give them
/// (`CoverBackdrop` in `src/ui/tokens.rs`), and the theme gives the colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowStyle {
    /// The page background, in sRGB. It is the color of the veil.
    pub background: [u8; 3],
    /// The text color `Label`, in sRGB.
    pub label: [u8; 3],
    /// The text color `SecondaryLabel`, in sRGB.
    pub secondary_label: [u8; 3],
    /// The opacity of the veil at the top of the header.
    pub veil_top: f32,
    /// The opacity of the grain.
    pub grain_opacity: f32,
    /// The highest opacity of the glow.
    pub max_opacity: f32,
    /// The blur of the website, in pixels at the drawn width of the art.
    pub blur: f32,
    /// The saturation of the art.
    pub saturation: f32,
    /// The opacity of the entity color when the item has no image.
    pub entity_tint: f32,
    /// The opacity of the entity color below a black and white cover.
    pub muted_tint: f32,
    /// The opacity of a black and white cover.
    pub muted_art: f32,
    /// The lowest colorfulness of a cover that keeps its colors.
    pub min_colorfulness: f32,
    /// The theme profile of the colors above, as a number. It is a part of
    /// the cache key, so a change of profile prepares a new glow.
    pub profile: u8,
    /// Increase Contrast changes the colors above. It is a part of the
    /// cache key.
    pub increase_contrast: bool,
}

impl GlowStyle {
    /// A number that is equal for equal styles. The cache key holds it.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.background.hash(&mut hasher);
        self.label.hash(&mut hasher);
        self.secondary_label.hash(&mut hasher);
        self.profile.hash(&mut hasher);
        self.increase_contrast.hash(&mut hasher);
        for value in [
            self.veil_top,
            self.grain_opacity,
            self.max_opacity,
            self.blur,
            self.saturation,
            self.entity_tint,
            self.muted_tint,
            self.muted_art,
            self.min_colorfulness,
        ] {
            value.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }
}

/// One frame of a source image, reduced to [`SMALL_SIDE`] and opaque.
#[derive(Clone, Debug)]
pub struct SourceFrame {
    pub pixels: RgbaImage,
    pub delay_ms: u32,
}

/// One candidate image. It has at least one frame. A still image has one.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub frames: Vec<SourceFrame>,
}

/// The art of one glow.
#[derive(Clone, Debug)]
pub enum GlowSource {
    /// One image. An animated image has more than one frame.
    Image(Vec<SourceFrame>),
    /// Two images in two equal columns.
    Pair(Box<[RgbaImage; 2]>),
    /// No image: the entity color.
    Entity,
}

/// The choice among the candidates of a page.
#[derive(Clone, Debug)]
pub struct Chosen {
    pub source: GlowSource,
    /// Each candidate is black and white. The glow uses the muted blend.
    pub muted: bool,
}

/// One prepared frame. It is opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlowFrame {
    pub pixels: RgbaImage,
    pub delay_ms: u32,
}

/// The glow of one page, ready for the renderer.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedGlow {
    pub frames: Vec<GlowFrame>,
    /// The opacity of the glow. It is at most `GlowStyle::max_opacity`.
    pub opacity: f32,
    /// The opacity of the grain of the theme of the glow.
    pub grain_opacity: f32,
    /// The main color of the art, for the opaque fill of Reduce
    /// Transparency.
    pub color: Option<CoverColor>,
}

// -----------------------------------------------------------------------------
// Reading an image
// -----------------------------------------------------------------------------

/// Reduces `image` to a [`SMALL_SIDE`] square that fills the square, and
/// flattens its transparency on the page background.
#[must_use]
pub fn reduce(image: &DynamicImage, background: [u8; 3]) -> RgbaImage {
    flatten(
        &image
            .resize_to_fill(SMALL_SIDE, SMALL_SIDE, FilterType::Triangle)
            .into_rgba8(),
        background,
    )
}

fn flatten(image: &RgbaImage, background: [u8; 3]) -> RgbaImage {
    let mut flat = image.clone();
    for pixel in flat.pixels_mut() {
        let alpha = f32::from(pixel[3]) / 255.0;
        for channel in 0..3 {
            pixel[channel] = to_byte(
                f32::from(pixel[channel]) * alpha + f32::from(background[channel]) * (1.0 - alpha),
            );
        }
        pixel[3] = 255;
    }
    flat
}

/// Decodes encoded image bytes into a candidate. With `all_frames`, an
/// animated GIF gives each frame. A GIF with more than [`MAX_FRAMES`]
/// frames gives its first frame only. Otherwise the candidate has the
/// first frame. `None` when the bytes do not decode.
#[must_use]
pub fn decode_candidate(bytes: &[u8], all_frames: bool, background: [u8; 3]) -> Option<Candidate> {
    if all_frames && image::guess_format(bytes).ok() == Some(image::ImageFormat::Gif) {
        if let Some(frames) = decode_gif_frames(bytes, background) {
            return Some(Candidate { frames });
        }
    }
    let image = image::load_from_memory(bytes).ok()?;
    Some(Candidate {
        frames: vec![SourceFrame {
            pixels: reduce(&image, background),
            delay_ms: DEFAULT_DELAY_MS,
        }],
    })
}

fn decode_gif_frames(bytes: &[u8], background: [u8; 3]) -> Option<Vec<SourceFrame>> {
    let decoder = GifDecoder::new(Cursor::new(bytes)).ok()?;
    let mut frames: Vec<SourceFrame> = decoder
        .into_frames()
        .take(MAX_FRAMES + 1)
        .map_while(Result::ok)
        .map(|frame| {
            let (numerator, denominator) = frame.delay().numer_denom_ms();
            let delay_ms = (numerator / denominator.max(1)).max(1);
            let image = DynamicImage::ImageRgba8(frame.into_buffer());
            SourceFrame {
                pixels: reduce(&image, background),
                delay_ms,
            }
        })
        .collect();
    if frames.len() > MAX_FRAMES {
        // The cover has more frames than the glow keeps. A glow of the
        // first frames would move out of step with the cover.
        frames.truncate(1);
    }
    (!frames.is_empty()).then_some(frames)
}

// -----------------------------------------------------------------------------
// The choice among the candidates
// -----------------------------------------------------------------------------

/// The colorfulness of an image after Hasler and Suesstrunk, on a sample
/// of 24 by 24 pixels. A gray image gives 0.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "a sample of 576 pixels is exact in f32"
)]
pub fn colorfulness(image: &RgbaImage) -> f32 {
    let sample = imageops::resize(image, SAMPLE_SIDE, SAMPLE_SIDE, FilterType::Triangle);
    let count = (SAMPLE_SIDE * SAMPLE_SIDE) as f32;
    let (mut rg_sum, mut yb_sum) = (0.0_f32, 0.0_f32);
    let mut pairs = Vec::with_capacity(sample.pixels().len());
    for pixel in sample.pixels() {
        let [red, green, blue] = [
            f32::from(pixel[0]),
            f32::from(pixel[1]),
            f32::from(pixel[2]),
        ];
        let rg = red - green;
        let yb = 0.5 * (red + green) - blue;
        rg_sum += rg;
        yb_sum += yb;
        pairs.push((rg, yb));
    }
    let (rg_mean, yb_mean) = (rg_sum / count, yb_sum / count);
    let (mut rg_var, mut yb_var) = (0.0_f32, 0.0_f32);
    for (rg, yb) in pairs {
        rg_var += (rg - rg_mean).powi(2);
        yb_var += (yb - yb_mean).powi(2);
    }
    let sigma = ((rg_var + yb_var) / count).sqrt();
    let mu = rg_mean.hypot(yb_mean);
    0.3_f32.mul_add(mu, sigma)
}

/// Chooses the art of a page from its candidates, in the order of the
/// page. Each candidate with a colorfulness of `min_colorfulness` or more
/// keeps its colors. One or two of them, the highest, give the glow. Two
/// stand in two equal columns, in the order of the candidates. With no
/// colorful candidate, the highest one gives the muted blend. `None` when
/// there is no candidate.
#[must_use]
pub fn choose(candidates: Vec<Candidate>, style: &GlowStyle) -> Option<Chosen> {
    let scores: Vec<f32> = candidates
        .iter()
        .map(|candidate| colorfulness(&candidate.frames[0].pixels))
        .collect();
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    // A stable sort keeps the page order for equal scores.
    order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
    let mut kept: Vec<usize> = order
        .iter()
        .copied()
        .filter(|index| scores[*index] >= style.min_colorfulness)
        .take(2)
        .collect();
    kept.sort_unstable();
    let mut candidates: Vec<Option<Candidate>> = candidates.into_iter().map(Some).collect();
    match kept.as_slice() {
        [] => {
            let best = *order.first()?;
            let candidate = candidates[best].take()?;
            Some(Chosen {
                source: GlowSource::Image(candidate.frames),
                muted: true,
            })
        }
        [one] => {
            let candidate = candidates[*one].take()?;
            Some(Chosen {
                source: GlowSource::Image(candidate.frames),
                muted: false,
            })
        }
        [left, right] => {
            let left = candidates[*left].take()?.frames.swap_remove(0).pixels;
            let right = candidates[*right].take()?.frames.swap_remove(0).pixels;
            Some(Chosen {
                source: GlowSource::Pair(Box::new([left, right])),
                muted: false,
            })
        }
        _ => None,
    }
}

// -----------------------------------------------------------------------------
// The preparation
// -----------------------------------------------------------------------------

/// Prepares the glow of `source`. A still image gives one frame, and an
/// animated image gives one frame for each frame, with the same delays.
/// `entity` is the entity color of the page. It is the color of a glow with
/// no image, and the tint below a muted glow.
#[must_use]
pub fn prepare(
    source: &GlowSource,
    muted: bool,
    entity: [u8; 3],
    style: &GlowStyle,
) -> PreparedGlow {
    let inputs: Vec<(RgbaImage, u32)> = match source {
        GlowSource::Image(frames) => frames
            .iter()
            .map(|frame| (frame.pixels.clone(), frame.delay_ms))
            .collect(),
        GlowSource::Pair(pair) => {
            let mut canvas = RgbaImage::new(SMALL_SIDE * 2, SMALL_SIDE);
            imageops::replace(&mut canvas, &pair[0], 0, 0);
            imageops::replace(&mut canvas, &pair[1], i64::from(SMALL_SIDE), 0);
            vec![(canvas, DEFAULT_DELAY_MS)]
        }
        GlowSource::Entity => vec![(
            RgbaImage::from_pixel(SMALL_SIDE, SMALL_SIDE, Rgba([0, 0, 0, 255])),
            DEFAULT_DELAY_MS,
        )],
    };
    // `SMALL_SIDE` is 96, so the conversion is exact.
    #[allow(clippy::cast_precision_loss)]
    let sigma = style.blur * SMALL_SIDE as f32 / BLUR_REFERENCE_WIDTH;
    let mut opacity = style.max_opacity;
    let mut frames = Vec::with_capacity(inputs.len());
    let mut color = None;
    for (index, (input, delay_ms)) in inputs.into_iter().enumerate() {
        let layered = if matches!(source, GlowSource::Entity) {
            entity_layer(&input, entity, style)
        } else {
            let art = saturate(&imageops::blur(&input, sigma), style.saturation);
            if muted {
                muted_glow_layer(&art, entity, style)
            } else {
                art
            }
        };
        if index == 0 {
            color = match source {
                GlowSource::Entity => Some(CoverColor {
                    red: entity[0],
                    green: entity[1],
                    blue: entity[2],
                }),
                _ => main_color(&input),
            };
        }
        opacity = opacity.min(glow_opacity(&layered, style));
        frames.push(GlowFrame {
            pixels: layered,
            delay_ms,
        });
    }
    PreparedGlow {
        frames,
        opacity,
        grain_opacity: style.grain_opacity,
        color,
    }
}

/// The glow of an item with no image: the entity color over the page
/// background.
fn entity_layer(shape: &RgbaImage, entity: [u8; 3], style: &GlowStyle) -> RgbaImage {
    let color = mix(
        style.background.map(f32::from),
        entity.map(f32::from),
        style.entity_tint,
    );
    RgbaImage::from_pixel(
        shape.width(),
        shape.height(),
        Rgba([to_byte(color[0]), to_byte(color[1]), to_byte(color[2]), 255]),
    )
}

/// The muted blend of a black and white cover: the entity color at
/// `muted_tint` over the page background, then the art at `muted_art`
/// with the luminosity blend.
pub fn muted_glow_layer(art: &RgbaImage, entity: [u8; 3], style: &GlowStyle) -> RgbaImage {
    let tinted = mix(
        style.background.map(f32::from),
        entity.map(f32::from),
        style.muted_tint,
    );
    let mut layered = art.clone();
    for pixel in layered.pixels_mut() {
        let source = [
            f32::from(pixel[0]),
            f32::from(pixel[1]),
            f32::from(pixel[2]),
        ];
        let blended = set_luminosity(tinted, luminosity(source));
        let result = mix(tinted, blended, style.muted_art);
        *pixel = Rgba([
            to_byte(result[0]),
            to_byte(result[1]),
            to_byte(result[2]),
            255,
        ]);
    }
    layered
}

/// The saturation matrix of the CSS filter `saturate`.
fn saturate(image: &RgbaImage, amount: f32) -> RgbaImage {
    let mut out = image.clone();
    for pixel in out.pixels_mut() {
        let [red, green, blue] = [
            f32::from(pixel[0]),
            f32::from(pixel[1]),
            f32::from(pixel[2]),
        ];
        pixel[0] = to_byte(
            (0.213 + 0.787 * amount) * red
                + (0.715 - 0.715 * amount) * green
                + (0.072 - 0.072 * amount) * blue,
        );
        pixel[1] = to_byte(
            (0.213 - 0.213 * amount) * red
                + (0.715 + 0.285 * amount) * green
                + (0.072 - 0.072 * amount) * blue,
        );
        pixel[2] = to_byte(
            (0.213 - 0.213 * amount) * red
                + (0.715 - 0.715 * amount) * green
                + (0.072 + 0.928 * amount) * blue,
        );
    }
    out
}

/// The film grain as one tile with its own alpha.
///
/// The website blends gray noise over the glow with the overlay blend.
/// GPUI has no blend mode, so the tile approximates it with source-over:
/// a noise value above the middle gives a white pixel, and a value below
/// gives a black pixel. The alpha is the distance from the middle, times
/// `grain_opacity`. On a middle tone, the tile gives the same change as the
/// overlay blend. Three hashed layers make the noise, so its values gather
/// near the middle as the fractal noise of the website does.
#[must_use]
pub fn grain_tile(grain_opacity: f32) -> RgbaImage {
    RgbaImage::from_fn(GRAIN_TILE_SIDE, GRAIN_TILE_SIDE, |x, y| {
        let noise = (hash_unit(x, y, 1) + hash_unit(x, y, 2) + hash_unit(x, y, 3)) / 3.0;
        let distance = (noise - 0.5).abs() * 2.0;
        let shade = if noise > 0.5 { 255 } else { 0 };
        Rgba([
            shade,
            shade,
            shade,
            to_byte(distance * grain_opacity * 255.0),
        ])
    })
}

/// A value in 0 to 1 for one pixel and one layer.
#[expect(
    clippy::cast_precision_loss,
    reason = "the hash keeps 24 bits, which f32 holds exactly"
)]
fn hash_unit(x: u32, y: u32, layer: u32) -> f32 {
    let mut value = x
        .wrapping_mul(0x85eb_ca6b)
        .wrapping_add(y.wrapping_mul(0xc2b2_ae35))
        .wrapping_add(layer.wrapping_mul(0x27d4_eb2f))
        .wrapping_add(GRAIN_SEED);
    value ^= value >> 15;
    value = value.wrapping_mul(0x2c1b_3c6d);
    value ^= value >> 12;
    value = value.wrapping_mul(0x297a_2d39);
    value ^= value >> 15;
    (value >> 8) as f32 / 16_777_215.0
}

// -----------------------------------------------------------------------------
// The opacity
// -----------------------------------------------------------------------------

/// The highest opacity, with `max_opacity` as the maximum, at which the
/// text keeps 4.5:1 at the top of the veil, for the worst block of the
/// glow. A block is 6 by 6 pixels of a sample of 24 by 24 pixels. A glow
/// that is black on a Dark page keeps the maximum, and a white glow gets
/// a low value.
#[must_use]
pub fn glow_opacity(layered: &RgbaImage, style: &GlowStyle) -> f32 {
    let sample = imageops::resize(layered, SAMPLE_SIDE, SAMPLE_SIDE, FilterType::Triangle);
    let blocks = SAMPLE_SIDE / BLOCK_SIDE;
    let mut lowest = style.max_opacity;
    for block_y in 0..blocks {
        for block_x in 0..blocks {
            let mut sum = [0.0_f32; 3];
            for y in 0..BLOCK_SIDE {
                for x in 0..BLOCK_SIDE {
                    let pixel =
                        sample.get_pixel(block_x * BLOCK_SIDE + x, block_y * BLOCK_SIDE + y);
                    for channel in 0..3 {
                        sum[channel] += f32::from(pixel[channel]);
                    }
                }
            }
            #[expect(clippy::cast_precision_loss, reason = "a block holds 36 pixels")]
            let area = (BLOCK_SIDE * BLOCK_SIDE) as f32;
            lowest = lowest.min(opacity_for_color(sum.map(|value| value / area), style));
        }
    }
    lowest
}

/// The highest opacity for one glow color, in sRGB values from 0 to 255.
#[must_use]
pub fn opacity_for_color(glow: [f32; 3], style: &GlowStyle) -> f32 {
    if text_keeps_contrast(glow, style.max_opacity, style) {
        return style.max_opacity;
    }
    if !text_keeps_contrast(glow, 0.0, style) {
        return 0.0;
    }
    let (mut low, mut high) = (0.0_f32, style.max_opacity);
    for _ in 0..OPACITY_STEPS {
        let middle = f32::midpoint(low, high);
        if text_keeps_contrast(glow, middle, style) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

/// The color at the top of the veil: the glow at `opacity` over the page
/// background, then the veil at `veil_top`.
#[must_use]
pub fn color_at_top_of_veil(glow: [f32; 3], opacity: f32, style: &GlowStyle) -> [f32; 3] {
    let background = style.background.map(f32::from);
    let lit = mix(background, glow, opacity);
    mix(lit, background, style.veil_top)
}

/// The lower contrast of `Label` and `SecondaryLabel` at the top of the
/// veil, for a glow color at `opacity`.
#[must_use]
pub fn text_contrast_at_top_of_veil(glow: [f32; 3], opacity: f32, style: &GlowStyle) -> f32 {
    let top = color_at_top_of_veil(glow, opacity, style);
    [style.label, style.secondary_label]
        .into_iter()
        .map(|text| contrast(text.map(f32::from), top))
        .fold(f32::INFINITY, f32::min)
}

fn text_keeps_contrast(glow: [f32; 3], opacity: f32, style: &GlowStyle) -> bool {
    text_contrast_at_top_of_veil(glow, opacity, style) >= TEXT_CONTRAST
}

fn contrast(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(rgb: [f32; 3]) -> f32 {
    let linear = |value: f32| {
        let channel = (value / 255.0).clamp(0.0, 1.0);
        if channel <= 0.039_28 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
}

// -----------------------------------------------------------------------------
// Color helpers
// -----------------------------------------------------------------------------

fn mix(from: [f32; 3], to: [f32; 3], amount: f32) -> [f32; 3] {
    [
        (to[0] - from[0]).mul_add(amount, from[0]),
        (to[1] - from[1]).mul_add(amount, from[1]),
        (to[2] - from[2]).mul_add(amount, from[2]),
    ]
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is rounded and clamped to the range of a byte"
)]
fn to_byte(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

/// The luminosity of the W3C compositing specification, from 0 to 255.
fn luminosity(rgb: [f32; 3]) -> f32 {
    0.3 * rgb[0] + 0.59 * rgb[1] + 0.11 * rgb[2]
}

/// `SetLum` of the W3C compositing specification: the color with the
/// luminosity `target`, with its hue and saturation.
fn set_luminosity(rgb: [f32; 3], target: f32) -> [f32; 3] {
    let shift = target - luminosity(rgb);
    clip_color(rgb.map(|value| value + shift))
}

fn clip_color(rgb: [f32; 3]) -> [f32; 3] {
    let lum = luminosity(rgb);
    let low = rgb.into_iter().fold(f32::INFINITY, f32::min);
    let high = rgb.into_iter().fold(f32::NEG_INFINITY, f32::max);
    let mut out = rgb;
    if low < 0.0 {
        out = out.map(|value| lum + (value - lum) * lum / (lum - low));
    }
    if high > 255.0 {
        out = out.map(|value| lum + (value - lum) * (255.0 - lum) / (high - lum));
    }
    out
}

// -----------------------------------------------------------------------------
// The request
// -----------------------------------------------------------------------------

/// The most candidate images that one glow reads.
pub const MAX_CANDIDATES: usize = 8;

/// What the cache needs to prepare one glow, and what it keeps the glow
/// under. The theme is in the key through the fingerprint of the style,
/// which holds the colors, the profile and Increase Contrast.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlowKey {
    /// The URLs of the candidates, in the order of the page. Empty for an
    /// item with no image.
    pub urls: Vec<String>,
    /// The URLs that the glow uses when the one own image of the item does
    /// not read.
    pub fallback_urls: Vec<String>,
    pub entity: [u8; 3],
    pub style: u64,
}

impl GlowKey {
    /// The key of `urls` with `entity` and `style`. At most
    /// [`MAX_CANDIDATES`] URLs stay.
    #[must_use]
    pub fn new(
        urls: &[String],
        fallback_urls: &[String],
        entity: [u8; 3],
        style: &GlowStyle,
    ) -> Self {
        Self {
            urls: urls.iter().take(MAX_CANDIDATES).cloned().collect(),
            fallback_urls: fallback_urls.iter().take(MAX_CANDIDATES).cloned().collect(),
            entity,
            style: style.fingerprint(),
        }
    }
}

/// A request to prepare one glow.
#[derive(Clone, Debug)]
pub struct GlowRequest {
    pub urls: Vec<String>,
    pub fallback_urls: Vec<String>,
    pub entity: [u8; 3],
    pub style: GlowStyle,
}

impl GlowRequest {
    /// The key that the cache keeps the glow under.
    #[must_use]
    pub fn key(&self) -> GlowKey {
        GlowKey::new(&self.urls, &self.fallback_urls, self.entity, &self.style)
    }
}

/// Prepares the glow of encoded candidate images. An image that does not
/// decode is dropped. With no image left, the glow has the entity color.
/// An animated GIF keeps its frames only when it is the one candidate.
#[must_use]
pub fn prepare_from_encoded(
    images: &[Vec<u8>],
    entity: [u8; 3],
    style: &GlowStyle,
) -> PreparedGlow {
    let all_frames = images.len() == 1;
    let candidates: Vec<Candidate> = images
        .iter()
        .filter_map(|bytes| decode_candidate(bytes, all_frames, style.background))
        .collect();
    match choose(candidates, style) {
        Some(chosen) => prepare(&chosen.source, chosen.muted, entity, style),
        None => prepare(&GlowSource::Entity, false, entity, style),
    }
}

// -----------------------------------------------------------------------------
// The renderer image
// -----------------------------------------------------------------------------

/// The glow of one page as GPUI draws it: the frames of the image, a still
/// copy with the first frame, the opacity and the main color.
pub struct GlowImage {
    frames: Arc<RenderImage>,
    still: Arc<RenderImage>,
    grain: Arc<RenderImage>,
    frame_count: usize,
    opacity: f32,
    color: Option<CoverColor>,
    complete: bool,
}

/// The grain tiles that exist, one for each grain opacity. A theme has one
/// grain opacity, so the app holds one tile for each theme.
static GRAIN_TILES: Mutex<Vec<(u32, Arc<RenderImage>)>> = Mutex::new(Vec::new());

/// The renderer image of the grain tile for `grain_opacity`. The first call
/// makes the tile, and each later call gives the same image.
fn shared_grain_tile(grain_opacity: f32) -> Arc<RenderImage> {
    let bits = grain_opacity.to_bits();
    let Ok(mut tiles) = GRAIN_TILES.lock() else {
        return Arc::new(render_tile(&grain_tile(grain_opacity)));
    };
    if let Some((_, tile)) = tiles.iter().find(|(key, _)| *key == bits) {
        return Arc::clone(tile);
    }
    let tile = Arc::new(render_tile(&grain_tile(grain_opacity)));
    tiles.push((bits, Arc::clone(&tile)));
    tile
}

fn render_tile(tile: &RgbaImage) -> RenderImage {
    // GPUI reads the bytes in the order blue, green, red, alpha. Its shader
    // multiplies the color by the alpha, so the bytes are not premultiplied.
    let mut pixels = tile.clone();
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    RenderImage::new(vec![Frame::new(pixels)])
}

impl std::fmt::Debug for GlowImage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GlowImage")
            .field("frame_count", &self.frame_count)
            .field("opacity", &self.opacity)
            .finish_non_exhaustive()
    }
}

impl GlowImage {
    /// Builds the renderer image of `glow`. GPUI reads the bytes in the
    /// order blue, green, red, alpha.
    #[must_use]
    pub fn new(glow: &PreparedGlow) -> Self {
        let frames = Arc::new(RenderImage::new(
            glow.frames.iter().map(render_frame).collect::<Vec<_>>(),
        ));
        let still = if glow.frames.len() > 1 {
            Arc::new(RenderImage::new(vec![render_frame(&glow.frames[0])]))
        } else {
            Arc::clone(&frames)
        };
        Self {
            frames,
            still,
            grain: shared_grain_tile(glow.grain_opacity),
            frame_count: glow.frames.len(),
            opacity: glow.opacity,
            color: glow.color,
            complete: true,
        }
    }

    /// Marks the glow as prepared from an incomplete read: a candidate
    /// image did not read. The cache does not keep such a glow.
    #[must_use]
    pub fn incomplete(mut self) -> Self {
        self.complete = false;
        self
    }

    /// True when each candidate image read.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// The grain tile. It has its own size and alpha, and the page shows it
    /// in a pattern above the glow.
    #[must_use]
    pub fn grain(&self) -> &Arc<RenderImage> {
        &self.grain
    }

    /// The renderer images that the glow owns, for the removal from the
    /// atlas of the window. The shared grain tile is not in the list.
    #[must_use]
    pub fn owned_images(&self) -> Vec<Arc<RenderImage>> {
        let mut images = vec![Arc::clone(&self.frames)];
        if !Arc::ptr_eq(&self.frames, &self.still) {
            images.push(Arc::clone(&self.still));
        }
        images
    }

    /// The image to draw. With Reduce Motion, an animated glow gives its
    /// first frame (musicindex ADR 0028 R-ART-6).
    #[must_use]
    pub fn image(&self, allows_motion: bool) -> &Arc<RenderImage> {
        if allows_motion {
            &self.frames
        } else {
            &self.still
        }
    }

    /// The number of frames of the animated image.
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// The opacity of the glow.
    #[must_use]
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// The main color of the art.
    #[must_use]
    pub fn color(&self) -> Option<CoverColor> {
        self.color
    }
}

fn render_frame(frame: &GlowFrame) -> Frame {
    let mut pixels = frame.pixels.clone();
    for pixel in pixels.pixels_mut() {
        pixel.0.swap(0, 2);
    }
    Frame::from_parts(
        pixels,
        0,
        0,
        Delay::from_numer_denom_ms(frame.delay_ms.max(1), 1),
    )
}

#[cfg(test)]
mod tests {
    use image::codecs::gif::GifEncoder;

    use super::*;

    const DARK_BACKGROUND: [u8; 3] = [0x0b, 0x0b, 0x0d];
    const LIGHT_BACKGROUND: [u8; 3] = [0xff, 0xff, 0xff];
    const ENTITY: [u8; 3] = [0x87, 0x5c, 0xcf];

    fn dark_style() -> GlowStyle {
        GlowStyle {
            background: DARK_BACKGROUND,
            label: [0xf5, 0xf5, 0xf7],
            secondary_label: [0xa1, 0xa1, 0xa6],
            veil_top: 0.25,
            grain_opacity: 0.22,
            max_opacity: 0.85,
            blur: 90.0,
            saturation: 1.7,
            entity_tint: 0.45,
            muted_tint: 0.40,
            muted_art: 0.35,
            min_colorfulness: 18.0,
            profile: 1,
            increase_contrast: false,
        }
    }

    fn light_style() -> GlowStyle {
        GlowStyle {
            background: LIGHT_BACKGROUND,
            label: [0x1d, 0x1d, 0x1f],
            secondary_label: [0x5c, 0x5c, 0x62],
            veil_top: 0.55,
            grain_opacity: 0.28,
            ..dark_style()
        }
    }

    fn solid(color: [u8; 3]) -> RgbaImage {
        RgbaImage::from_pixel(
            SMALL_SIDE,
            SMALL_SIDE,
            Rgba([color[0], color[1], color[2], 255]),
        )
    }

    /// A left half and a right half in two colors.
    fn halves(left: [u8; 3], right: [u8; 3]) -> RgbaImage {
        let mut image = solid(left);
        for x in SMALL_SIDE / 2..SMALL_SIDE {
            for y in 0..SMALL_SIDE {
                image.put_pixel(x, y, Rgba([right[0], right[1], right[2], 255]));
            }
        }
        image
    }

    fn frame(image: RgbaImage, delay_ms: u32) -> SourceFrame {
        SourceFrame {
            pixels: image,
            delay_ms,
        }
    }

    fn candidate(image: RgbaImage) -> Candidate {
        Candidate {
            frames: vec![frame(image, DEFAULT_DELAY_MS)],
        }
    }

    fn mean_color(image: &RgbaImage) -> [f32; 3] {
        let mut sum = [0.0_f32; 3];
        for pixel in image.pixels() {
            for channel in 0..3 {
                sum[channel] += f32::from(pixel[channel]);
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "a test image is small")]
        let count = image.pixels().len() as f32;
        sum.map(|value| value / count)
    }

    /// M1 (ADR 0101 Decision 4, task 002): a still cover gives one frame
    /// of the prepared size, and the same input gives the same output.
    #[test]
    fn adr_0101_header_glow_m1_still_cover_gives_one_stable_frame() {
        let source = GlowSource::Image(vec![frame(halves([200, 30, 40], [10, 10, 250]), 100)]);
        let style = dark_style();
        let first = prepare(&source, false, ENTITY, &style);
        let second = prepare(&source, false, ENTITY, &style);
        assert_eq!(first.frames.len(), 1, "a still cover gives one frame");
        assert_eq!(
            first.frames[0].pixels.dimensions(),
            (SMALL_SIDE, SMALL_SIDE),
            "the frame has the prepared size"
        );
        assert!(
            first.frames[0].pixels.pixels().all(|pixel| pixel[3] == 255),
            "a prepared frame is opaque"
        );
        assert_eq!(first, second, "one input must give one output");
    }

    /// M2: an animated cover of three frames gives three frames with the
    /// same delays. The test goes through the GIF decoder.
    #[test]
    fn adr_0101_header_glow_m2_animated_cover_keeps_its_frames_and_delays() {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut bytes);
            for (color, delay_ms) in [
                ([200, 30, 40], 40),
                ([30, 200, 40], 80),
                ([30, 40, 200], 120),
            ] {
                let buffer =
                    RgbaImage::from_pixel(40, 40, Rgba([color[0], color[1], color[2], 255]));
                encoder
                    .encode_frame(Frame::from_parts(
                        buffer,
                        0,
                        0,
                        Delay::from_numer_denom_ms(delay_ms, 1),
                    ))
                    .expect("encode frame");
            }
        }
        let style = dark_style();
        let candidate = decode_candidate(&bytes, true, style.background).expect("decode the GIF");
        assert_eq!(candidate.frames.len(), 3, "the decoder keeps each frame");
        let delays: Vec<u32> = candidate.frames.iter().map(|f| f.delay_ms).collect();
        let glow = prepare(&GlowSource::Image(candidate.frames), false, ENTITY, &style);
        assert_eq!(glow.frames.len(), 3);
        assert_eq!(
            glow.frames.iter().map(|f| f.delay_ms).collect::<Vec<_>>(),
            delays,
            "the delays stay"
        );
        assert_eq!(delays, vec![40, 80, 120]);
        let still =
            decode_candidate(&bytes, false, style.background).expect("decode the first frame");
        assert_eq!(
            still.frames.len(),
            1,
            "a decode for a still keeps one frame"
        );
    }

    /// M3: a black and white cover gets the blend of the entity color.
    #[test]
    fn adr_0101_header_glow_m3_black_and_white_cover_gets_the_entity_blend() {
        let style = dark_style();
        let gray = halves([40, 40, 40], [210, 210, 210]);
        let chosen = choose(vec![candidate(gray.clone())], &style).expect("one candidate");
        assert!(chosen.muted, "a gray cover is black and white");
        let muted = prepare(&chosen.source, true, [0xd0, 0x30, 0x30], &style);
        let plain = prepare(
            &GlowSource::Image(vec![frame(gray, 100)]),
            false,
            [0xd0, 0x30, 0x30],
            &style,
        );
        let muted_mean = mean_color(&muted.frames[0].pixels);
        let plain_mean = mean_color(&plain.frames[0].pixels);
        assert!(
            muted_mean[0] - muted_mean[2] > 10.0,
            "the muted glow must lean to the red entity color: {muted_mean:?}"
        );
        assert!(
            (plain_mean[0] - plain_mean[2]).abs() < 2.0,
            "a gray glow without the blend stays gray: {plain_mean:?}"
        );
    }

    /// The colorfulness of the answers of 2026-11-16: a gray image is
    /// below 18, and an image of two strong colors is above.
    #[test]
    fn adr_0101_header_glow_colorfulness_separates_gray_from_color() {
        assert!(colorfulness(&solid([120, 120, 120])) < 1.0);
        assert!(colorfulness(&halves([200, 30, 40], [10, 10, 250])) > 18.0);
        assert!(colorfulness(&halves([60, 60, 60], [200, 200, 200])) < 18.0);
    }

    /// The choice of the artist and label pages: the one or two most
    /// colorful candidates stay, two stand in the order of the page, and
    /// a gray candidate does not stay.
    #[test]
    fn adr_0101_header_glow_choice_keeps_the_one_or_two_most_colorful() {
        let style = dark_style();
        let red = candidate(halves([220, 30, 30], [220, 30, 30]));
        let blue = candidate(halves([30, 30, 220], [30, 30, 220]));
        let gray = candidate(solid([120, 120, 120]));
        let vivid = candidate(halves([230, 20, 20], [20, 20, 230]));

        let one = choose(vec![gray.clone(), red.clone()], &style).expect("choice");
        assert!(!one.muted);
        assert!(
            matches!(one.source, GlowSource::Image(_)),
            "one colorful candidate gives one image"
        );

        let two = choose(vec![red.clone(), gray.clone(), blue.clone()], &style).expect("choice");
        let GlowSource::Pair(pair) = two.source else {
            panic!("two colorful candidates give two columns");
        };
        assert!(
            pair[0].get_pixel(0, 0)[0] > 150,
            "red stays in the left column"
        );
        assert!(
            pair[1].get_pixel(0, 0)[2] > 150,
            "blue stays in the right column"
        );

        let three = choose(vec![red, vivid, blue], &style).expect("choice");
        assert!(
            matches!(three.source, GlowSource::Pair(_)),
            "two candidates at most stay"
        );

        let none = choose(vec![gray], &style).expect("choice");
        assert!(none.muted, "gray candidates only give the muted blend");
        assert!(choose(Vec::new(), &style).is_none());
    }

    /// M5 (ADR 0101 Decision 4, R-ART-5 item 2): at the opacity that the
    /// module calculates, `Label` and `SecondaryLabel` keep 4.5:1 at the
    /// top of the veil, for a white glow and a black glow in each theme.
    #[test]
    fn adr_0101_header_glow_m5_text_keeps_contrast_at_the_calculated_opacity() {
        for (name, style) in [("Dark", dark_style()), ("Light", light_style())] {
            for glow in [[255.0_f32; 3], [0.0_f32; 3]] {
                let opacity = opacity_for_color(glow, &style);
                let contrast = text_contrast_at_top_of_veil(glow, opacity, &style);
                assert!(
                    contrast >= TEXT_CONTRAST,
                    "ADR 0101 Decision 4 R-ART-5 item 2: {name} glow {glow:?} at opacity {opacity} gives contrast {contrast}. Lower the glow opacity in header_glow::opacity_for_color."
                );
            }
        }
        let white_in_dark = opacity_for_color([255.0; 3], &dark_style());
        let black_in_dark = opacity_for_color([0.0; 3], &dark_style());
        assert!(
            white_in_dark < 0.3,
            "a white glow is limited in Dark: {white_in_dark}"
        );
        assert!(
            (black_in_dark - 0.85).abs() < 1e-6,
            "a black glow keeps the maximum in Dark"
        );
    }

    /// The contrast of the text at the top of the veil, for the worst block
    /// of one prepared frame at `opacity`. It reads the pixels of the frame,
    /// and it does not call the opacity search of the module.
    fn worst_contrast(frame: &RgbaImage, opacity: f32, style: &GlowStyle) -> f32 {
        let sample = imageops::resize(frame, 24, 24, FilterType::Triangle);
        let mut worst = f32::INFINITY;
        for block_y in 0..4 {
            for block_x in 0..4 {
                let mut sum = [0.0_f32; 3];
                for y in 0..6 {
                    for x in 0..6 {
                        let pixel = sample.get_pixel(block_x * 6 + x, block_y * 6 + y);
                        for channel in 0..3 {
                            sum[channel] += f32::from(pixel[channel]);
                        }
                    }
                }
                let mean = sum.map(|value| value / 36.0);
                worst = worst.min(text_contrast_at_top_of_veil(mean, opacity, style));
            }
        }
        worst
    }

    /// The opacity of a real glow keeps the text at 4.5:1 for the worst
    /// block of each frame. The test measures the contrast of the prepared
    /// pixels. A glow with no image is limited too, and a bright glow gets
    /// a lower opacity than the maximum.
    #[test]
    fn adr_0101_header_glow_prepared_opacity_keeps_text_contrast() {
        for style in [dark_style(), light_style()] {
            for source in [
                GlowSource::Image(vec![frame(halves([255, 255, 255], [250, 250, 0]), 100)]),
                GlowSource::Image(vec![
                    frame(halves([250, 250, 250], [20, 20, 20]), 100),
                    frame(halves([10, 10, 10], [240, 240, 240]), 100),
                ]),
                GlowSource::Entity,
            ] {
                let glow = prepare(&source, false, ENTITY, &style);
                assert!(glow.opacity <= style.max_opacity);
                assert!(glow.opacity >= 0.0);
                for prepared in &glow.frames {
                    let contrast = worst_contrast(&prepared.pixels, glow.opacity, &style);
                    assert!(
                        contrast >= TEXT_CONTRAST - 0.01,
                        "ADR 0101 Decision 4 R-ART-5 item 2: a prepared frame has contrast {contrast} at opacity {}. Fix header_glow::glow_opacity.",
                        glow.opacity
                    );
                }
            }
        }
        let style = dark_style();
        let bright = prepare(
            &GlowSource::Image(vec![frame(solid([255, 255, 255]), 100)]),
            false,
            ENTITY,
            &style,
        );
        assert!(
            bright.opacity < style.max_opacity,
            "a bright glow in Dark needs a lower opacity than the maximum"
        );
    }

    /// The grain is one tile of its own size, with alpha that the grain
    /// opacity limits. A glow gives the tile of its theme, and one tile
    /// serves each glow of that theme.
    #[test]
    fn adr_0101_header_glow_grain_is_one_tile_with_its_own_alpha() {
        let tile = grain_tile(0.22);
        assert_eq!(tile.dimensions(), (GRAIN_TILE_SIDE, GRAIN_TILE_SIDE));
        let strongest = tile.pixels().map(|pixel| pixel[3]).max().unwrap_or(0);
        assert!(strongest > 0, "the tile has grain");
        assert!(
            f32::from(strongest) <= 0.22 * 255.0 + 1.0,
            "the grain opacity limits the alpha: {strongest}"
        );
        assert_eq!(tile, grain_tile(0.22), "one opacity gives one tile");

        let style = dark_style();
        let source = GlowSource::Image(vec![frame(solid([200, 30, 40]), 100)]);
        let first = GlowImage::new(&prepare(&source, false, ENTITY, &style));
        let second = GlowImage::new(&prepare(&source, false, ENTITY, &style));
        assert!(
            Arc::ptr_eq(first.grain(), second.grain()),
            "a theme has one grain tile"
        );
        let light = GlowImage::new(&prepare(&source, false, ENTITY, &light_style()));
        assert!(
            !Arc::ptr_eq(first.grain(), light.grain()),
            "the Light theme has its own grain tile"
        );
    }

    /// A GIF with more frames than the glow keeps gives its first frame
    /// only. A GIF with the most frames that the glow keeps gives each one.
    #[test]
    fn adr_0101_header_glow_gif_over_the_frame_limit_gives_the_first_frame() {
        let gif = |count: usize| {
            let mut bytes = Vec::new();
            {
                let mut encoder = GifEncoder::new(&mut bytes);
                for index in 0..count {
                    let shade = u8::try_from(index * 4 % 256).unwrap_or(0);
                    let buffer = RgbaImage::from_pixel(8, 8, Rgba([shade, 40, 90, 255]));
                    encoder
                        .encode_frame(Frame::from_parts(
                            buffer,
                            0,
                            0,
                            Delay::from_numer_denom_ms(50, 1),
                        ))
                        .expect("encode frame");
                }
            }
            bytes
        };
        let background = dark_style().background;
        let kept = decode_candidate(&gif(MAX_FRAMES), true, background).expect("decode");
        assert_eq!(kept.frames.len(), MAX_FRAMES);
        let over = decode_candidate(&gif(MAX_FRAMES + 1), true, background).expect("decode");
        assert_eq!(over.frames.len(), 1, "a longer cover gives its first frame");
    }

    /// The still copy of an animated glow has the first frame only.
    #[test]
    fn adr_0101_header_glow_image_gives_one_frame_without_motion() {
        let style = dark_style();
        let source = GlowSource::Image(vec![
            frame(solid([200, 30, 40]), 40),
            frame(solid([30, 200, 40]), 80),
            frame(solid([30, 40, 200]), 120),
        ]);
        let glow = GlowImage::new(&prepare(&source, false, ENTITY, &style));
        assert_eq!(glow.frame_count(), 3);
        assert_eq!(glow.image(true).frame_count(), 3, "motion shows each frame");
        assert_eq!(
            glow.image(false).frame_count(),
            1,
            "Reduce Motion shows one frame"
        );
    }
}
