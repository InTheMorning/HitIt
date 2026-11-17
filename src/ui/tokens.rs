//! Design tokens — single source of truth for color, spacing, radius, and typography.
//!
//! Inspired by Apple Human Interface Guidelines: every value has a semantic name
//! (`Label`, `SecondaryLabel`, `SystemBackground`, `SystemFill`, `Separator`,
//! `Accent`, …) rather than a raw hex literal. Base color tokens resolve
//! against an [`Appearance`] (`Light` or `Dark`); app rendering resolves them
//! through the active [`crate::theme_profile::ThemeProfile`].
//!
//! Call sites should never use raw `rgb(0x…)` literals. Use:
//!
//! ```ignore
//! use crate::ui::tokens::{color, SemanticColor};
//! let label = color(cx, SemanticColor::Label);
//! ```
//!
//! Dark is the default appearance for the app. Light values follow HIG so the
//! system can be flipped at runtime once a settings UI exists.

#![warn(clippy::pedantic)]

use std::time::Duration;

use gpui::{
    linear_color_stop, linear_gradient, point, px, App, Background, BoxShadow, FontWeight, Hsla,
    Pixels, Rgba, WindowAppearance,
};

use gpui_component::ActiveTheme;

use crate::media::cover_color::CoverColor;
use crate::media::header_glow::GlowStyle;
use crate::media::logo_frame::{BakedShadow, LOGO_FRAME_ASPECT};
use crate::theme_profile::ThemeProfile;
use crate::ui::contrast::{delinearize, linearize};

/// Visual appearance scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Appearance {
    Light,
    #[default]
    Dark,
}

impl Appearance {
    /// Returns the appearance currently bound to the rendering environment.
    ///
    /// Prefers the typed [`Environment`] global (the bundled SwiftUI-style
    /// accessor); falls back to the gpui-component theme mode for any path
    /// that hasn't installed the env yet.
    #[must_use]
    pub fn current(cx: &App) -> Self {
        if let Some(env) = cx.try_global::<Environment>() {
            return env.appearance;
        }
        if cx.theme().mode.is_dark() {
            Self::Dark
        } else {
            Self::Light
        }
    }
}

impl From<WindowAppearance> for Appearance {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
        }
    }
}

// -----------------------------------------------------------------------------
// SemanticColor — Apple HIG-style palette.
// -----------------------------------------------------------------------------

/// Semantic color token. Always prefer this enum over hardcoded hex literals.
///
/// The naming follows Apple's HIG so designers can reason about role
/// (`Label`, `Fill`, `Background`) rather than appearance (`Gray`, `Blue`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum SemanticColor {
    // Backgrounds — used for the window canvas and broad surfaces.
    SystemBackground,
    /// The fill of a group or a card (musicindex ADR 0045 R-ELEVATION-2,
    /// `level` in the shared copy at `src/assets/tokens.json`).
    GroupBackground,
    /// The fill of an open entry (musicindex ADR 0045 R-ELEVATION-2,
    /// `level` in the shared copy at `src/assets/tokens.json`).
    OpenEntryBackground,
    /// ADR 0101 Decision 1: the fill of a popover, a menu and a sheet. It is
    /// `color.light.surface` in Light and the dark raised surface in Dark
    /// (musicindex ADR 0045 R-ELEVATION-2).
    FloatingBackground,
    /// The fill of a hovered row, tile or plain button. It differs from the
    /// page level and from the group level in each theme. It is not a level.
    HoverBackground,
    /// ADR 0101 Decision 1: the fourth background tier of `search.html`.
    /// Used by the sidebar surface of the app, never by a content canvas.
    SidebarBackground,

    // Labels — text colors in descending prominence.
    Label,
    SecondaryLabel,
    TertiaryLabel,
    QuaternaryLabel,

    // Fills — chip / pill / inline button backgrounds.
    SystemFill,
    SecondaryFill,
    TertiaryFill,

    // Selection.
    SelectedContent,

    /// The glow at the top edge of the sidebar (musicindex ADR 0056
    /// R-SIDE-1): the accent of each theme at 9 % opacity
    /// (`sidebar-glow-top` in the shared copy at `src/assets/tokens.json`).
    SidebarGlowTop,

    // Separators.
    Separator,
    OpaqueSeparator,

    /// The border of a clear check box (musicindex ADR 0063 R-CHECK-1
    /// item 3 and R-CHECK-5). It has its own value, so a change to the
    /// separator lines does not change the check box.
    CheckboxBorder,

    // Tint / Accent.
    Accent,
    AccentHover,
    AccentPressed,
    OnAccent,
    Focus,
    /// The fill of an Action button (`accent-tint` in the shared copy at
    /// `src/assets/tokens.json`).
    AccentTint,
    /// The top stop of the selected look (musicindex ADR 0051 R-SELECT-1
    /// item 1). The value is `selected-top` of the shared copy.
    SelectedTop,
    /// The bottom stop of the selected look (`selected-bottom`).
    SelectedBottom,
    /// The 1 px inner edge of the selected look (`selected-edge`).
    SelectedEdge,
    /// The 1 px inner highlight at the top of the selected look
    /// (`selected-highlight`).
    SelectedHighlight,
    /// The fill of a destructive button (musicindex ADR 0020 R-TOKEN-1).
    DangerTint,
    /// The fill of a Neutral button (musicindex ADR 0042 R-ACT-5).
    ButtonFill,
    /// The fill of a Neutral button with the pointer on it.
    ButtonFillHover,

    // Status — system colors (HIG fidelity, used as fills / large text).
    Success,
    Warning,
    Danger,
    Info,

    // Status labels — darker variants tuned for body text on canvas.
    SuccessLabel,
    WarningLabel,
    DangerLabel,
    InfoLabel,

    // Text colors that pair with each filled status surface above.
    // Choice depends on the saturation of the fill in each appearance —
    // bright greens/oranges want near-black, dark reds/blues want white.
    OnSuccess,
    OnWarning,
    OnDanger,
    OnInfo,

    // HitIt-specific provenance diff colors.
    DiffMatch,
    DiffDifferent,
    DiffMissing,

    // HitIt-specific ID3 frame version chips. Used as label and frame-tag
    // color on metadata rows so the user can spot frame-version provenance
    // at a glance.
    Id3FrameV22,
    Id3FrameV23Only,
    Id3FrameV24Only,
    Id3FrameUnknown,

    // ADR 0101 Decision 2: one color for each stated entity kind, drawn
    // only as a small dot, never as text-on-fill. `EntityKind::fill_token`
    // in `ui::composites::tag_badge` is the one caller that maps a kind to
    // its token. `Release` reuses `EntityFeed` and `Recording` reuses
    // `EntityTrack`, because the website groups them in the same parent
    // color. A kind with no stated entity, such as `Generic`, does not use
    // one of these tokens. The values match `entity` in the shared copy at
    // `src/assets/tokens.json` (musicindex ADR 0032 R-DOT-2).
    EntityFeed,
    EntityTrack,
    EntityPlaylist,
    EntityLive,
    EntityArtist,
    EntityHost,
    EntityAuthor,
    EntityCurator,
    EntityLabel,
    EntityPublisher,
    EntityProducer,
    EntityNetwork,
    EntityHosting,
    EntitySponsor,
}

impl SemanticColor {
    /// Resolve the token to a concrete color for the given appearance.
    #[must_use]
    pub fn resolve(self, appearance: Appearance) -> Rgba {
        match appearance {
            Appearance::Dark => Self::dark_palette(self),
            Appearance::Light => Self::light_palette(self),
        }
    }

    fn dark_palette(token: Self) -> Rgba {
        // ADR 0101 Decision 1: the dark palette follows `search.html`.
        // `SystemBackground`, `SidebarBackground`, `GroupBackground`
        // (surface), `OpenEntryBackground` (raised surface), `Label`
        // (text), `SecondaryLabel` (muted text), `Separator`, and `Accent`
        // are the website's own values. `TertiaryLabel`, `QuaternaryLabel`,
        // the three fills, `AccentHover`, `AccentPressed`, `Focus`, and
        // `Info` are derived from those values. The ADR 0083 task 001
        // packet result records the derivation of each. `Warning` and
        // `WarningLabel` move from gold to orange in this change (Decision
        // 2), and this frees gold for the value palette. `DiffDifferent`
        // and `Id3FrameV23Only` move off gold too.
        //
        // Several semantic tokens intentionally resolve to the same hex.
        // For example, `SecondaryFill` and `OpenEntryBackground` share
        // the raised-surface value. Keep the match exhaustive so the
        // symmetry with `light_palette` stays obvious.
        #[expect(clippy::match_same_arms, reason = "different semantics, shared value")]
        match token {
            Self::SystemBackground => hex(0x0b_0b0d),
            Self::SidebarBackground => hex(0x14_1417),
            // musicindex ADR 0045 R-ELEVATION-2: the group and open entry levels.
            Self::GroupBackground => hex(0x1c_1c1f),
            Self::OpenEntryBackground => hex(0x26_262a),
            Self::FloatingBackground => hex(0x26_262a),
            Self::HoverBackground => hex(0x26_262a),

            Self::Label => hex(0xf5_f5f7),
            Self::SecondaryLabel => hex(0xa1_a1a6),
            // Derived: continues the Label -> SecondaryLabel step toward
            // SystemBackground (ADR 0083 task 001 packet result).
            Self::TertiaryLabel => hex(0x8e_8e93),
            Self::QuaternaryLabel => hex(0x43_4345),

            // Derived: the surface ladder shifted one tier, so a fill reads
            // as a chip raised above its surrounding surface.
            Self::SystemFill => hex(0x30_3035),
            Self::SecondaryFill => hex(0x26_262a),
            Self::TertiaryFill => hex(0x1c_1c1f),

            Self::SelectedContent => hex(0x2a_3352),

            // `white at 10%` per Decision 1 — a translucent hairline that
            // adapts to whatever surface sits behind it.
            Self::Separator => hex_alpha(0xff_ffff, 0.10),
            Self::OpaqueSeparator => hex(0x6a_708a),
            Self::CheckboxBorder => hex(0x6a_708a),

            Self::Accent => hex(0x2d_7bff),
            // Derived: Accent blended 20% toward white (hover) and 20%
            // toward black (pressed).
            Self::AccentHover => hex(0x57_95ff),
            Self::AccentPressed => hex(0x24_62cc),
            Self::OnAccent => hex(0x0b_0d13),
            // musicindex ADR 0080 R-TOKEN-1: `color.dark.focus` of the shared copy.
            Self::Focus => hex(0x2d_7bff),
            Self::AccentTint => hex_alpha(0x2d_7bff, 0.18),
            Self::SidebarGlowTop => hex_alpha(0x2d_7bff, 0.09),
            Self::SelectedTop => hex_alpha(0x2d_7bff, 0.24),
            Self::SelectedBottom => hex_alpha(0x2d_7bff, 0.12),
            Self::SelectedEdge => hex_alpha(0x2d_7bff, 0.34),
            Self::SelectedHighlight => hex_alpha(0xff_ffff, 0.14),
            Self::DangerTint => hex_alpha(0xff_6961, 0.16),
            Self::ButtonFill => hex_alpha(0x76_7680, 0.24),
            Self::ButtonFillHover => hex_alpha(0xff_ffff, 0.12),

            // musicindex ADR 0025 R-STATUS-1 owns the status values.
            Self::Success => hex(0x30_d158),
            // ADR 0101 Decision 2: orange, not gold, so gold is free for the
            // value palette. Started from #ff9f0a and kept the existing
            // OnWarning contrast pair.
            Self::Warning => hex(0xff_9f0a),
            Self::Danger => hex(0xff_6961),
            // Derived: reuses Accent, matching the pre-existing pattern
            // where Info tracks Accent's hue.
            Self::Info => hex(0x2d_7bff),

            // In dark, the system colors already meet body-text contrast
            // against the canvas — labels can match the system value.
            Self::SuccessLabel => hex(0x30_d158),
            Self::WarningLabel => hex(0xff_9f0a),
            Self::DangerLabel => hex(0xff_6961),
            // The accent of R-STATUS-1 gives 4.36:1 on a raised surface. This
            // value has its hue and gives 4.5:1 for text.
            Self::InfoLabel => hex(0x57_95ff),

            // Dark-mode status fills are all bright/saturated — near-black
            // text reads cleanly on every one of them.
            Self::OnSuccess => hex(0x0b_0d13),
            Self::OnWarning => hex(0x0b_0d13),
            Self::OnDanger => hex(0x0b_0d13),
            Self::OnInfo => hex(0x0b_0d13),

            Self::DiffMatch => hex(0x6f_d4a3),
            // ADR 0101 Decision 2: moved away from gold the same way as
            // Warning.
            Self::DiffDifferent => hex(0xff_9b1e),
            Self::DiffMissing => hex(0xff_a07f),

            Self::Id3FrameV22 => hex(0xb0_6cf4),
            // ADR 0101 Decision 2: moved away from gold the same way as
            // Warning.
            Self::Id3FrameV23Only => hex(0xff_b454),
            Self::Id3FrameV24Only => hex(0x3a_c4c4),
            Self::Id3FrameUnknown => hex(0xff_8a65),

            // ADR 0101 Decision 1: entity colors of `search.html`.
            Self::EntityFeed => hex(0xc4_965b),
            Self::EntityTrack => hex(0x00_8b99),
            Self::EntityPlaylist => hex(0x8e_dfad),
            Self::EntityArtist => hex(0xfa_b5e5),
            Self::EntityLive => hex(0xff_453a),
            Self::EntityHost => hex(0xff_b4d9),
            Self::EntityAuthor => hex(0xf3_b7f0),
            Self::EntityCurator => hex(0xea_b9f9),
            Self::EntityLabel => hex(0x87_5ccf),
            Self::EntityPublisher => hex(0x7c_60d4),
            Self::EntityProducer => hex(0x91_58c8),
            Self::EntityNetwork => hex(0x9a_54c0),
            Self::EntityHosting => hex(0x70_65d8),
            Self::EntitySponsor => hex(0xa2_51b7),
        }
    }

    fn light_palette(token: Self) -> Rgba {
        // ADR 0101 Decision 1: the light palette follows `search.html`.
        // `TertiaryLabel`, `QuaternaryLabel`, the three fills, `AccentHover`,
        // `AccentPressed`, `Focus`, and `Info` are derived like those in
        // `dark_palette`. See the ADR 0083 task 001 packet result.
        // `Warning`, `DiffDifferent`, and `Id3FrameV23Only` read as
        // orange/brown in light, so Decision 2 leaves their light values
        // unchanged.
        #[expect(clippy::match_same_arms, reason = "different semantics, shared value")]
        match token {
            Self::SystemBackground => hex(0xff_ffff),
            Self::SidebarBackground => hex(0xf5_f5f7),
            // musicindex ADR 0045 R-ELEVATION-2: the group and open entry levels.
            Self::GroupBackground => hex(0xf2_f2f7),
            Self::OpenEntryBackground => hex(0xff_ffff),
            Self::FloatingBackground => hex(0xff_ffff),
            Self::HoverBackground => hex(0xf2_f2f7),

            Self::Label => hex(0x1d_1d1f),
            Self::SecondaryLabel => hex(0x5c_5c62),
            // Derived: continues the Label -> SecondaryLabel step toward
            // SystemBackground (ADR 0083 task 001 packet result).
            Self::TertiaryLabel => hex(0x6e_6e73),
            Self::QuaternaryLabel => hex(0xc0_c0c0),

            // Derived: the surface ladder shifted one tier toward black, so
            // a fill reads as a chip pressed below its surrounding surface.
            Self::SystemFill => hex(0xc6_c6cf),
            Self::SecondaryFill => hex(0xd9_d9df),
            Self::TertiaryFill => hex(0xec_ecef),

            Self::SelectedContent => hex(0xd1_e0ff),

            // `#3c3c43 at 16%` per Decision 1 — a translucent hairline that
            // adapts to whatever surface sits behind it.
            Self::Separator => hex_alpha(0x3c_3c43, 0.16),
            Self::OpaqueSeparator => hex(0x8e_8e93),
            Self::CheckboxBorder => hex(0x7c_7c80),

            Self::Accent => hex(0x0a_5bd6),
            // Derived: Accent blended 20% toward white (hover) and 20%
            // toward black (pressed).
            Self::AccentHover => hex(0x3b_7cde),
            Self::AccentPressed => hex(0x08_49ab),
            Self::OnAccent => hex(0xff_ffff),
            // musicindex ADR 0080 R-TOKEN-1: `color.light.focus` of the shared copy.
            Self::Focus => hex(0x00_5bd3),
            Self::AccentTint => hex_alpha(0x0a_5bd6, 0.12),
            Self::SidebarGlowTop => hex_alpha(0x0a_5bd6, 0.09),
            Self::SelectedTop => hex_alpha(0x0a_5bd6, 0.20),
            Self::SelectedBottom => hex_alpha(0x0a_5bd6, 0.10),
            Self::SelectedEdge => hex_alpha(0x0a_5bd6, 0.40),
            Self::SelectedHighlight => hex_alpha(0xff_ffff, 0.70),
            Self::DangerTint => hex_alpha(0xc4_1e3a, 0.10),
            Self::ButtonFill => hex_alpha(0x78_7880, 0.14),
            Self::ButtonFillHover => hex_alpha(0x78_7880, 0.18),

            Self::Success => hex(0x34_c759),
            Self::Warning => hex(0xff_9500),
            Self::Danger => hex(0xc4_1e3a),
            // Derived: reuses Accent, matching the pre-existing pattern
            // where Info tracks Accent's hue.
            Self::Info => hex(0x0a_5bd6),

            // Label variants — tuned to satisfy 4.5:1 against SystemBackground
            // (white). Same intent as Apple's `.systemRed`/`.systemGreen`
            // *label* shades that ship in `UIColor` but are slightly darker
            // than the fills.
            Self::SuccessLabel => hex(0x1a_7f37),
            Self::WarningLabel => hex(0xa0_5a00),
            Self::DangerLabel => hex(0xc4_1e3a),
            Self::InfoLabel => hex(0x0a_5bd6),

            // iOS systemGreen / systemOrange are too bright to take white
            // text — Apple's own HIG examples render black on them. Red and
            // blue fills are dark enough to take white.
            Self::OnSuccess => hex(0x00_0000),
            Self::OnWarning => hex(0x00_0000),
            Self::OnDanger => hex(0xff_ffff),
            Self::OnInfo => hex(0xff_ffff),

            Self::DiffMatch => hex(0x1f_7a3a),
            Self::DiffDifferent => hex(0x96_5a00),
            Self::DiffMissing => hex(0xb1_3c20),

            // Light-mode frame chips: darker, lower-saturation variants of
            // the dark-mode hues, tuned to read as labels on white.
            Self::Id3FrameV22 => hex(0x5b_3b9e),
            Self::Id3FrameV23Only => hex(0x8b_5a00),
            Self::Id3FrameV24Only => hex(0x00_6d77),
            Self::Id3FrameUnknown => hex(0xb1_3c20),

            // ADR 0101 Decision 1: entity colors of `search.html`.
            Self::EntityFeed => hex(0x8a_5e21),
            Self::EntityTrack => hex(0x00_8e9c),
            Self::EntityPlaylist => hex(0x00_6238),
            Self::EntityArtist => hex(0x72_3763),
            Self::EntityLive => hex(0xd7_0015),
            Self::EntityHost => hex(0x76_3559),
            Self::EntityAuthor => hex(0x6d_396b),
            Self::EntityCurator => hex(0x66_3c73),
            Self::EntityLabel => hex(0x8d_62d5),
            Self::EntityPublisher => hex(0x82_67db),
            Self::EntityProducer => hex(0x97_5ecf),
            Self::EntityNetwork => hex(0xa0_5bc7),
            Self::EntityHosting => hex(0x75_6bdf),
            Self::EntitySponsor => hex(0xa9_57be),
        }
    }
}

impl SemanticColor {
    /// The token that Increase Contrast uses in place of `self`. A separator
    /// becomes opaque, and secondary text uses the primary text color
    /// (musicindex ADR 0068 R-A11Y-1 item 2).
    #[must_use]
    pub const fn for_contrast(self, increase_contrast: bool) -> Self {
        if !increase_contrast {
            return self;
        }
        match self {
            Self::Separator => Self::OpaqueSeparator,
            Self::SecondaryLabel => Self::Label,
            other => other,
        }
    }
}

/// Lay `top` over `background` and give an opaque color. Reduce Transparency
/// uses this for each translucent fill (musicindex ADR 0068 R-A11Y-1 item 3).
#[must_use]
pub fn flatten_over(top: Rgba, background: Rgba) -> Rgba {
    let keep = top.a;
    let rest = 1.0 - keep;
    Rgba {
        r: top.r * keep + background.r * rest,
        g: top.g * keep + background.g * rest,
        b: top.b * keep + background.b * rest,
        a: 1.0,
    }
}

/// Resolve a semantic color against the current appearance.
#[must_use]
pub fn color(cx: &App, token: SemanticColor) -> Rgba {
    let env = Environment::current(cx);
    crate::ui::theme_profiles::resolve_accessible_color(
        env.profile,
        env.appearance,
        token,
        env.increase_contrast,
        env.reduce_transparency,
    )
}

/// Resolve a semantic color for a render context, honoring an explicit
/// light/dark appearance override when one is provided.
#[must_use]
pub fn resolve_color(cx: &App, token: SemanticColor, appearance: Option<Appearance>) -> Rgba {
    let Some(appearance) = appearance else {
        return color(cx, token);
    };
    let env = Environment::current(cx);
    let profile = match appearance {
        Appearance::Dark => ThemeProfile::Dark,
        Appearance::Light => ThemeProfile::Light,
    };
    crate::ui::theme_profiles::resolve_accessible_color(
        profile,
        appearance,
        token,
        env.increase_contrast,
        env.reduce_transparency,
    )
}

// -----------------------------------------------------------------------------
// TypeDot — the type dot of musicindex ADR 0032 R-DOT-1.
// -----------------------------------------------------------------------------

/// The shape of the type dot. The values match `shape.type-dot` in the shared
/// copy at `src/assets/tokens.json`, and a test checks them.
pub struct TypeDot;

impl TypeDot {
    /// Diameter of a filled dot, in pixels at 1.0×.
    pub const SIZE_PX: f32 = 8.0;

    /// Width of the ring of a hollow dot, in pixels at 1.0×.
    pub const HOLLOW_RING_PX: f32 = 1.5;

    /// Diameter of a filled dot, scaled by the active [`ScaleFactor`].
    #[must_use]
    pub fn size(cx: &App) -> Pixels {
        scale_chrome_px(Self::SIZE_PX, ScaleFactor::current(cx))
    }

    /// Width of the ring of a hollow dot, scaled by the active [`ScaleFactor`].
    #[must_use]
    pub fn ring_width(cx: &App) -> Pixels {
        scale_chrome_px(Self::HOLLOW_RING_PX, ScaleFactor::current(cx))
    }
}

// -----------------------------------------------------------------------------
// SidebarGlow — musicindex ADR 0056 R-SIDE-1.
// -----------------------------------------------------------------------------

/// The glow at the top of the sidebar. The length matches
/// `sidebar.glow-length-px` in the shared copy at `src/assets/tokens.json`,
/// and a test checks it.
pub struct SidebarGlow;

impl SidebarGlow {
    /// Distance from the top edge where the glow has no color, in pixels at
    /// 1.0×.
    pub const LENGTH_PX: f32 = 320.0;

    /// Length of the glow, scaled by the active [`ScaleFactor`].
    #[must_use]
    pub fn length(cx: &App) -> Pixels {
        scale_chrome_px(Self::LENGTH_PX, ScaleFactor::current(cx))
    }

    /// The vertical gradient from `top` at the top edge to the same color
    /// with no opacity at the bottom edge of the glow layer.
    #[must_use]
    pub fn background(top: Rgba) -> gpui::Background {
        let [from, to] = Self::stops(top);
        gpui::linear_gradient(180.0, from, to)
    }

    /// The two color stops of the glow gradient.
    #[must_use]
    pub fn stops(top: Rgba) -> [gpui::LinearColorStop; 2] {
        let clear = Rgba { a: 0.0, ..top };
        [
            gpui::linear_color_stop(top, 0.0),
            gpui::linear_color_stop(clear, 1.0),
        ]
    }
}

// -----------------------------------------------------------------------------
// Spacing — 4-pt grid.
// -----------------------------------------------------------------------------

/// 4-pt spacing scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Spacing {
    /// 2 px.
    XXS,
    /// 4 px.
    XS,
    /// 8 px.
    SM,
    /// 12 px.
    MD,
    /// 16 px.
    LG,
    /// 20 px. Between `LG` and `XL`.
    LGPlus,
    /// 24 px.
    XL,
    /// 32 px.
    XXL,
    /// 48 px.
    XXXL,
}

impl Spacing {
    /// Base (1.0×) value. Use [`Self::scaled`] when an `App` is in scope so
    /// the user's UI scale is honored.
    #[must_use]
    pub const fn px(self) -> Pixels {
        match self {
            Self::XXS => px(2.0),
            Self::XS => px(4.0),
            Self::SM => px(8.0),
            Self::MD => px(12.0),
            Self::LG => px(16.0),
            Self::LGPlus => px(20.0),
            Self::XL => px(24.0),
            Self::XXL => px(32.0),
            Self::XXXL => px(48.0),
        }
    }

    /// Same as [`Self::px`] but multiplied by the active [`ScaleFactor`].
    ///
    /// ADR 0039: `Spacing` is a CHROME token — this resolves through
    /// [`scale_chrome_px`], never the TYPE domain.
    #[must_use]
    pub fn scaled(self, cx: &App) -> Pixels {
        scale_chrome_px(f32::from(self.px()), ScaleFactor::current(cx))
    }
}

// -----------------------------------------------------------------------------
// Radius.
// -----------------------------------------------------------------------------

/// Corner radius scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Radius {
    /// 6 px — chips, badges (`shape.radius.chip`).
    SM,
    /// 10 px — fields and rows (`shape.radius.field`, `shape.radius.row`).
    MD,
    /// 12 px — popovers (`shape.radius.popover`).
    LG,
    /// 20 px — cards (`shape.radius.card`).
    XL,
    /// 24 px — sheets (`shape.radius.sheet`).
    Sheet,
    /// Pill shape (999 px) — each button (musicindex ADR 0020 R-ACT-2).
    Full,
}

impl Radius {
    #[must_use]
    pub const fn px(self) -> Pixels {
        match self {
            Self::SM => px(6.0),
            Self::MD => px(10.0),
            Self::LG => px(12.0),
            Self::XL => px(20.0),
            Self::Sheet => px(24.0),
            Self::Full => px(999.0),
        }
    }

    /// ADR 0039: `Radius` is a CHROME token — this resolves through
    /// [`scale_chrome_px`], never the TYPE domain.
    #[must_use]
    pub fn scaled(self, cx: &App) -> Pixels {
        // The pill radius is intentionally capped — scaling 999px makes no
        // visual difference and risks integer overflow on extreme factors.
        if matches!(self, Self::Full) {
            return self.px();
        }
        scale_chrome_px(f32::from(self.px()), ScaleFactor::current(cx))
    }
}

// -----------------------------------------------------------------------------
// ArtworkRadius. ADR 0101 Decision 4.
// -----------------------------------------------------------------------------

/// The corner radius of an image of art.
///
/// musicindex ADR 0021 R-ART-4 item 2: the radius is 6 % of the edge, rounded
/// to a full pixel, with a minimum of 4 px. The shared token `art-radius` in
/// `src/assets/tokens.json` gives the same values, and a test compares them.
/// The radius is a function of the edge. It is not a [`Radius`] role, so a
/// change of a role does not change the art.
pub struct ArtworkRadius;

impl ArtworkRadius {
    /// The part of the edge that the radius takes (`fraction-of-edge`).
    pub const FRACTION_OF_EDGE: f32 = 0.06;
    /// The smallest radius in pixels (`minimum-px`).
    pub const MINIMUM_PX: f32 = 4.0;

    /// The radius for an edge in pixels. The result rounds half up.
    #[must_use]
    pub fn px_for_edge(edge_px: f32) -> f32 {
        (edge_px * Self::FRACTION_OF_EDGE + 0.5)
            .floor()
            .max(Self::MINIMUM_PX)
    }

    /// The radius for an edge that is already at the current UI scale
    /// (ADR 0039). The scaled edge sets the radius, so the radius keeps its
    /// proportion at each scale step.
    #[must_use]
    pub fn for_edge(edge: Pixels) -> Pixels {
        px(Self::px_for_edge(f32::from(edge)))
    }
}

// -----------------------------------------------------------------------------
// Typography.
// -----------------------------------------------------------------------------

/// Type-scale token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSize {
    /// 11 px — caption / micro labels.
    Micro,
    /// 12 px — captions.
    Caption,
    /// 13 px — body.
    Body,
    /// 15 px — headline.
    Headline,
    /// 17 px — large title row.
    Title3,
    /// 22 px — section title (`type.desktop.title-1`).
    Title2,
    /// 26 px — page title of an album, track or playlist
    /// (`type.desktop.large-title`, musicindex ADR 0068 R-TYPE-2).
    Title,
    /// 42 px — page title of an artist or label
    /// (`type.page-title.artist-label.display`, musicindex ADR 0068 R-TYPE-2).
    Display,
}

impl FontSize {
    #[must_use]
    pub const fn px(self) -> Pixels {
        match self {
            Self::Micro => px(11.0),
            Self::Caption => px(12.0),
            Self::Body => px(13.0),
            Self::Headline => px(15.0),
            Self::Title3 => px(17.0),
            Self::Title2 => px(22.0),
            Self::Title => px(26.0),
            Self::Display => px(42.0),
        }
    }

    /// Returns the type coefficient for this role at the specified scale
    ///
    /// ADR 0039 task 003 ratified the endpoints on 2026-09-18.
    /// Each role uses [`Self::type_endpoints`] and [`Self::interpolate`].
    /// Medium returns exactly `1.0`.
    /// This replaces task 001's uniform coefficients.
    /// Keep the endpoints separate for each role.
    /// Record new operator ratification in the ADR before changing an endpoint.
    #[must_use]
    pub const fn type_multiplier(self, scale: ScaleFactor) -> f32 {
        let (x_small, x_large) = self.type_endpoints();
        Self::interpolate(scale, x_small, x_large)
    }

    /// Returns the ratified `(x-small factor, x-large factor)` for this role
    ///
    /// ADR 0039 task 003 records the operator's decision on 2026-09-18.
    /// `Title` retains the former uniform x-small factor of 0.85.
    /// The x-small factor increases by 0.01 per role from `Title` toward `Micro`.
    /// Ratification did not change the proposed x-large factors.
    /// ADR 0083 task 002 adds `Display`. It uses the same per-role step as
    /// the other six roles. Its x-small value is `Title`'s value minus
    /// 0.01. Its x-large value is `Title`'s value minus 0.04.
    const fn type_endpoints(self) -> (f32, f32) {
        match self {
            Self::Micro => (0.91, 1.36),
            Self::Caption => (0.90, 1.32),
            Self::Body => (0.89, 1.28),
            Self::Headline => (0.88, 1.24),
            Self::Title3 => (0.87, 1.20),
            Self::Title2 => (0.86, 1.16),
            Self::Title => (0.85, 1.12),
            Self::Display => (0.84, 1.08),
        }
    }

    /// Interpolates the type factor between the ratified endpoints
    ///
    /// ADR 0039 task 003 uses [`Self::step_coordinate`] for the position `c`.
    /// The type resolver does not call [`ScaleFactor::chrome_multiplier`].
    /// Below medium, the factor is `1 - ((1 - c) / 0.15) * (1 - x_small)`.
    /// Medium returns exactly `1.0`.
    /// Above medium, the factor is `1 + ((c - 1) / 0.25) * (x_large - 1)`.
    /// X-small and x-large resolve to their respective endpoints.
    /// Small uses 8/15 of the downward change.
    /// Large uses 12/25 of the upward change.
    const fn interpolate(scale: ScaleFactor, x_small: f32, x_large: f32) -> f32 {
        match scale {
            ScaleFactor::Medium => 1.0,
            ScaleFactor::XSmall | ScaleFactor::Small => {
                let c = Self::step_coordinate(scale);
                1.0 - ((1.0 - c) / 0.15) * (1.0 - x_small)
            }
            ScaleFactor::Large | ScaleFactor::XLarge => {
                let c = Self::step_coordinate(scale);
                1.0 + ((c - 1.0) / 0.25) * (x_large - 1.0)
            }
        }
    }

    /// Returns the step coordinate for type interpolation
    ///
    /// ADR 0039 keeps these literals separate from [`ScaleFactor::chrome_multiplier`].
    /// Task 003 changed the type endpoints without changing the chrome resolver.
    const fn step_coordinate(scale: ScaleFactor) -> f32 {
        match scale {
            ScaleFactor::XSmall => 0.85,
            ScaleFactor::Small => 0.92,
            ScaleFactor::Medium => 1.0,
            ScaleFactor::Large => 1.12,
            ScaleFactor::XLarge => 1.25,
        }
    }

    /// Pure ADR 0039 TYPE-domain resolver — usable in tests without an
    /// `App`. [`Self::scaled`] is a thin environment-backed wrapper around
    /// this same resolver.
    #[must_use]
    pub fn scaled_px(self, scale: ScaleFactor) -> Pixels {
        px(f32::from(self.px()) * self.type_multiplier(scale))
    }

    /// ADR 0039: `FontSize` is the TYPE domain — this resolves through
    /// [`Self::scaled_px`]/[`Self::type_multiplier`], never CHROME.
    #[must_use]
    pub fn scaled(self, cx: &App) -> Pixels {
        self.scaled_px(ScaleFactor::current(cx))
    }
}

/// Type weight token (mirrors HIG weights).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

impl From<Weight> for FontWeight {
    fn from(w: Weight) -> Self {
        match w {
            Weight::Regular => Self::NORMAL,
            Weight::Medium => Self::MEDIUM,
            Weight::Semibold => Self::SEMIBOLD,
            Weight::Bold => Self::BOLD,
        }
    }
}

// -----------------------------------------------------------------------------
// ArtworkShadow. ADR 0101 Decision 4: artwork has a shadow. A larger image
// gets a larger shadow.
// -----------------------------------------------------------------------------

/// Artwork shadow scale.
///
/// `Lg` matches `ImageSize::Lg`, 80 px artwork. `Xl` matches
/// `ImageSize::Xl`, 152 px artwork. The Music content tile
/// also uses `Xl`, through `Size::ContentTileArtwork`. `XXl` matches
/// `ImageSize::XXl`, 200 px page-header artwork.
///
/// No other element draws this shadow. A guard in
/// `tests/architecture_tests.rs` checks this rule for ADR 0101 Decision 4.
/// The logo frame of a collection (ADR 0084 Decision 12) bakes the `XXl`
/// values into its composed image, through [`ArtworkShadow::baked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtworkShadow {
    /// 80 px artwork — `ImageSize::Lg`.
    Lg,
    /// 152 px artwork — `ImageSize::Xl` and the Music content tile.
    Xl,
    /// 200 px artwork — `ImageSize::XXl` page-header covers.
    XXl,
}

impl ArtworkShadow {
    /// Blur radius at the `Medium` scale step, in pixels.
    #[must_use]
    pub const fn blur_radius_base(self) -> Pixels {
        match self {
            Self::Lg => px(8.0),
            Self::Xl => px(16.0),
            Self::XXl => px(24.0),
        }
    }

    /// Vertical offset at the `Medium` scale step, in pixels.
    #[must_use]
    pub const fn offset_y_base(self) -> Pixels {
        match self {
            Self::Lg => px(2.0),
            Self::Xl => px(4.0),
            Self::XXl => px(6.0),
        }
    }

    /// Shadow opacity, black at this alpha value.
    ///
    /// `XXl` artwork is larger than `Xl` artwork. A darker shadow reads
    /// correctly on larger artwork.
    const fn alpha(self) -> f32 {
        match self {
            Self::Lg => 0.22,
            Self::Xl => 0.28,
            Self::XXl => 0.34,
        }
    }

    /// Returns the box shadow for this role at the current UI scale.
    ///
    /// ADR 0039: shadow geometry is CHROME — this resolves through
    /// [`scale_chrome_px`], never the TYPE domain.
    #[must_use]
    pub fn shadow(self, cx: &App) -> Vec<BoxShadow> {
        let scale = ScaleFactor::current(cx);
        let blur = scale_chrome_px(f32::from(self.blur_radius_base()), scale);
        let offset_y = scale_chrome_px(f32::from(self.offset_y_base()), scale);
        vec![BoxShadow {
            color: Hsla::from(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: self.alpha(),
            }),
            offset: point(px(0.0), offset_y),
            blur_radius: blur,
            spread_radius: px(0.0),
            inset: false,
        }]
    }
}

impl ArtworkShadow {
    /// The shadow values in device pixels, for an image that bakes the
    /// shadow (ADR 0084 Decision 12). `density` is the device pixels for
    /// each logical pixel.
    ///
    /// ADR 0039: shadow geometry is CHROME.
    #[must_use]
    pub fn baked(self, cx: &App, density: f32) -> BakedShadow {
        let scale = ScaleFactor::current(cx);
        BakedShadow {
            blur_px: f32::from(scale_chrome_px(f32::from(self.blur_radius_base()), scale))
                * density,
            offset_y_px: f32::from(scale_chrome_px(f32::from(self.offset_y_base()), scale))
                * density,
            alpha: self.alpha(),
        }
    }
}

// -----------------------------------------------------------------------------
// FloatingShadow. musicindex ADR 0068 R-ELEVATION-1 item 2: a popover, a menu
// and a sheet have one shadow.
// -----------------------------------------------------------------------------

/// The shadow of a floating layer: a popover, a menu and a sheet.
///
/// The values equal `shadow.floating` in the shared copy at
/// `src/assets/tokens.json`, and a test checks them. `Surface` draws it for
/// the floating elevation.
pub struct FloatingShadow;

impl FloatingShadow {
    /// Vertical offset at the `Medium` scale step, in pixels.
    pub const OFFSET_Y: f32 = 16.0;
    /// Blur radius at the `Medium` scale step, in pixels.
    pub const BLUR: f32 = 48.0;
    /// Shadow opacity in Dark.
    pub const DARK_ALPHA: f32 = 0.5;
    /// Shadow opacity in Light.
    pub const LIGHT_ALPHA: f32 = 0.18;

    /// The shadow opacity of an appearance, black at this alpha value.
    #[must_use]
    pub const fn alpha(appearance: Appearance) -> f32 {
        match appearance {
            Appearance::Dark => Self::DARK_ALPHA,
            Appearance::Light => Self::LIGHT_ALPHA,
        }
    }

    /// Returns the box shadow at the current UI scale.
    ///
    /// ADR 0039: shadow geometry is CHROME, so it resolves through
    /// [`scale_chrome_px`]. `appearance` is the override of the caller. With
    /// `None`, the shadow follows the current appearance.
    #[must_use]
    pub fn shadow(cx: &App, appearance: Option<Appearance>) -> Vec<BoxShadow> {
        let env = Environment::current(cx);
        let appearance = appearance.unwrap_or(env.appearance);
        vec![BoxShadow {
            color: Hsla::from(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: Self::alpha(appearance),
            }),
            offset: point(px(0.0), scale_chrome_px(Self::OFFSET_Y, env.scale)),
            blur_radius: scale_chrome_px(Self::BLUR, env.scale),
            spread_radius: px(0.0),
            inset: false,
        }]
    }
}

// -----------------------------------------------------------------------------
// ArtworkShape. ADR 0084 Decision 12: the MusicIndex logo frames the art of
// a collection.
// -----------------------------------------------------------------------------

/// The shape of collection art.
///
/// `src/media/logo_frame.rs` holds the polygons of the shape. This token
/// names them, and holds no number of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtworkShape {
    /// The `MusicIndex` logo: three pillars in a box that is wider than it
    /// is tall.
    LogoFrame,
}

impl ArtworkShape {
    /// The device pixels for each logical pixel of a composed logo image.
    /// The image stays sharp on a display with a scale of two.
    pub const RASTER_DENSITY: f32 = 2.0;

    /// The opacity of the entity color in a logo with no cover. The
    /// `MusicIndex` site mixes 45 percent of the entity color.
    pub const ENTITY_FILL_OPACITY: f32 = 0.45;

    /// The width of the shape divided by its height.
    #[must_use]
    pub const fn aspect(self) -> f32 {
        match self {
            Self::LogoFrame => LOGO_FRAME_ASPECT,
        }
    }
}

// -----------------------------------------------------------------------------
// CoverBackdrop. ADR 0101 Decision 4: a page header draws a glow from its
// cover (musicindex ADR 0021 R-ART-5). Each value is copied from the
// musicindex.org page header, `search.html` at `785ae28`. The calculated
// opacity follows musicindex `84ad724` (limit the glow).
// -----------------------------------------------------------------------------

/// The backdrop of a page header with a cover: a blurred glow from the
/// cover, film grain and a veil.
///
/// The app prepares the glow image in `media::header_glow`, because GPUI
/// has no blur, no blend mode and no tiled image. This token holds each
/// value of the glow, the veil and the box. A second set of values gives
/// the opaque fill for Reduce Transparency.
pub struct CoverBackdrop;

impl CoverBackdrop {
    /// The height of the glow box at the `Medium` scale step, in pixels.
    /// The box starts at the top of the page. The header height does not
    /// change it.
    pub const BOX_HEIGHT_BASE: f32 = 560.0;
    /// The height of the glow box in a narrow window, in pixels.
    pub const BOX_HEIGHT_NARROW_BASE: f32 = 460.0;
    /// The window width at or below which the box is narrow, in pixels.
    pub const NARROW_WIDTH_BASE: f32 = 1100.0;
    /// How far the art extends above the box, in pixels.
    pub const EXTEND_TOP_BASE: f32 = 200.0;
    /// How far the art extends at each side of the box, in pixels.
    pub const EXTEND_SIDE_BASE: f32 = 80.0;
    /// How far the art extends below the box, in pixels.
    pub const EXTEND_BOTTOM_BASE: f32 = 40.0;
    /// The blur of the art, in pixels at the width of the website art.
    pub const BLUR: f32 = 90.0;
    /// The saturation of the art.
    pub const SATURATION: f32 = 1.7;
    /// The highest opacity of the art. The page limits it further, so the
    /// header text keeps 4.5:1 (R-ART-5 item 2).
    pub const MAX_OPACITY: f32 = 0.85;
    /// The opacity of the entity color of an item with no image.
    pub const ENTITY_TINT: f32 = 0.45;
    /// The opacity of the entity color below a black and white cover.
    pub const MUTED_TINT: f32 = 0.40;
    /// The opacity of a black and white cover, with the luminosity blend.
    pub const MUTED_ART: f32 = 0.35;
    /// The lowest colorfulness of a cover that keeps its colors (Hasler
    /// and Suesstrunk, on a sample of 24 by 24 pixels).
    pub const MIN_COLORFULNESS: f32 = 18.0;
    /// The opacity of the veil at the top of the box in Dark appearance.
    pub const VEIL_TOP_DARK: f32 = 0.25;
    /// The opacity of the veil at the top of the box in Light appearance.
    pub const VEIL_TOP_LIGHT: f32 = 0.55;
    /// The opacity of the veil at `VEIL_MID_AT` in Dark appearance.
    pub const VEIL_MID_DARK: f32 = 0.60;
    /// The opacity of the veil at `VEIL_MID_AT` in Light appearance.
    pub const VEIL_MID_LIGHT: f32 = 0.80;
    /// The part of the box height where the two gradients of the veil meet.
    pub const VEIL_MID_AT: f32 = 0.55;
    /// The opacity of the grain in Dark appearance.
    pub const GRAIN_DARK: f32 = 0.22;
    /// The opacity of the grain in Light appearance.
    pub const GRAIN_LIGHT: f32 = 0.28;
    /// The side of the grain tile, in pixels. The page shows the tile at
    /// this size, in a pattern. It equals `media::header_glow::GRAIN_TILE_SIDE`.
    pub const GRAIN_TILE_SIDE: f32 = 240.0;
    /// The highest luminance of the opaque fill in Dark appearance. A
    /// brighter cover is darkened to this luminance.
    const DARK_MAX_LUMINANCE: f32 = 0.02;
    /// The lowest luminance of the opaque fill in Light appearance. A
    /// darker cover is lightened to this luminance.
    const LIGHT_MIN_LUMINANCE: f32 = 0.87;

    /// The height of the glow box for a window of `window_width`, at the
    /// current UI scale.
    ///
    /// ADR 0039: backdrop geometry is CHROME.
    #[must_use]
    pub fn box_height(window_width: Pixels, cx: &App) -> Pixels {
        let scale = ScaleFactor::current(cx);
        if window_width <= scale_chrome_px(Self::NARROW_WIDTH_BASE, scale) {
            scale_chrome_px(Self::BOX_HEIGHT_NARROW_BASE, scale)
        } else {
            scale_chrome_px(Self::BOX_HEIGHT_BASE, scale)
        }
    }

    /// How far the glow box reaches above and to the left of the header, so
    /// that the box starts at the edge of the scroll area. A page pads its
    /// scroll area with `Spacing::LG`.
    #[must_use]
    pub fn page_inset_start(cx: &App) -> Pixels {
        Spacing::LG.scaled(cx)
    }

    /// How far the glow box reaches to the right of the header, so that the
    /// box ends at the edge of the scroll area. It is the padding of the
    /// scroll area and the space of the overlay scrollbar of the page column.
    #[must_use]
    pub fn page_inset_end(cx: &App) -> Pixels {
        Spacing::LG.scaled(cx)
            + crate::ui::layouts::OVERLAY_SCROLLBAR_WIDTH
            + Spacing::SM.scaled(cx)
    }

    /// The side of the grain tile as the page draws it.
    #[must_use]
    pub fn grain_tile_side() -> Pixels {
        px(Self::GRAIN_TILE_SIDE)
    }

    /// The columns and rows of grain tiles that cover `width` by `height`.
    #[must_use]
    pub fn grain_cells(width: Pixels, height: Pixels) -> (usize, usize) {
        let side = Self::GRAIN_TILE_SIDE;
        let count = |extent: Pixels| {
            let cells = (f32::from(extent) / side).ceil().max(1.0);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "the value is a small positive whole number"
            )]
            let cells = cells as usize;
            cells
        };
        (count(width), count(height))
    }

    /// How far the art extends above the box, at the current UI scale.
    #[must_use]
    pub fn extend_top(cx: &App) -> Pixels {
        scale_chrome_px(Self::EXTEND_TOP_BASE, ScaleFactor::current(cx))
    }

    /// How far the art extends at each side of the box.
    #[must_use]
    pub fn extend_side(cx: &App) -> Pixels {
        scale_chrome_px(Self::EXTEND_SIDE_BASE, ScaleFactor::current(cx))
    }

    /// How far the art extends below the box.
    #[must_use]
    pub fn extend_bottom(cx: &App) -> Pixels {
        scale_chrome_px(Self::EXTEND_BOTTOM_BASE, ScaleFactor::current(cx))
    }

    /// The values of the glow preparation in `env`. They carry the colors
    /// of the live theme (the profile, the appearance and Increase
    /// Contrast), because the flatten color and the contrast target read
    /// them. The profile and Increase Contrast are in the cache key.
    #[must_use]
    pub fn glow_style(env: Environment) -> GlowStyle {
        let bytes = |token: SemanticColor| {
            let color = crate::ui::theme_profiles::resolve_accessible_color(
                env.profile,
                env.appearance,
                token,
                env.increase_contrast,
                env.reduce_transparency,
            );
            [color.r, color.g, color.b].map(|channel| {
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "the value is rounded and clamped to the range of a byte"
                )]
                let byte = (channel * 255.0).round().clamp(0.0, 255.0) as u8;
                byte
            })
        };
        let (veil_top, grain_opacity) = match env.appearance {
            Appearance::Dark => (Self::VEIL_TOP_DARK, Self::GRAIN_DARK),
            Appearance::Light => (Self::VEIL_TOP_LIGHT, Self::GRAIN_LIGHT),
        };
        let profile = match env.profile {
            ThemeProfile::System => 0,
            ThemeProfile::Dark => 1,
            ThemeProfile::Light => 2,
            ThemeProfile::HighContrastDark => 3,
            ThemeProfile::HighContrastLight => 4,
        };
        GlowStyle {
            background: bytes(SemanticColor::SystemBackground),
            label: bytes(SemanticColor::Label),
            secondary_label: bytes(SemanticColor::SecondaryLabel),
            veil_top,
            grain_opacity,
            max_opacity: Self::MAX_OPACITY,
            blur: Self::BLUR,
            saturation: Self::SATURATION,
            entity_tint: Self::ENTITY_TINT,
            muted_tint: Self::MUTED_TINT,
            muted_art: Self::MUTED_ART,
            min_colorfulness: Self::MIN_COLORFULNESS,
            profile,
            increase_contrast: env.increase_contrast,
        }
    }

    /// The glow values for the environment that `cx` installs.
    #[must_use]
    pub fn glow_style_current(cx: &App) -> GlowStyle {
        Self::glow_style(Environment::current(cx))
    }

    /// The two gradients of the veil, from the top of the box to the
    /// bottom: the page background at the top opacity to the middle
    /// opacity over the first part of the box, then to full opacity over
    /// the rest.
    #[must_use]
    pub fn veil(cx: &App) -> (Background, Background) {
        let appearance = Environment::current(cx).appearance;
        let (top, mid) = match appearance {
            Appearance::Dark => (Self::VEIL_TOP_DARK, Self::VEIL_MID_DARK),
            Appearance::Light => (Self::VEIL_TOP_LIGHT, Self::VEIL_MID_LIGHT),
        };
        let background = Hsla::from(color(cx, SemanticColor::SystemBackground));
        let stop = |opacity: f32, at: f32| linear_color_stop(background.opacity(opacity), at);
        (
            linear_gradient(180.0, stop(top, 0.0), stop(mid, 1.0)),
            linear_gradient(180.0, stop(mid, 0.0), stop(1.0, 1.0)),
        )
    }

    /// The opaque backdrop color of `cover` in `appearance`. Reduce
    /// Transparency draws it in place of the glow. The luminance limit
    /// keeps the header text at its contrast on every cover.
    #[must_use]
    pub fn tone(cover: CoverColor, appearance: Appearance) -> Rgba {
        let channel = |value: u8| linearize(f32::from(value) / 255.0);
        let linear = [
            channel(cover.red),
            channel(cover.green),
            channel(cover.blue),
        ];
        let luminance = 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
        let toned = match appearance {
            Appearance::Dark if luminance > Self::DARK_MAX_LUMINANCE => {
                let factor = Self::DARK_MAX_LUMINANCE / luminance;
                linear.map(|value| value * factor)
            }
            Appearance::Light if luminance < Self::LIGHT_MIN_LUMINANCE => {
                let toward_white = (Self::LIGHT_MIN_LUMINANCE - luminance) / (1.0 - luminance);
                linear.map(|value| (1.0 - value).mul_add(toward_white, value))
            }
            Appearance::Dark | Appearance::Light => linear,
        };
        Rgba {
            r: delinearize(toned[0]),
            g: delinearize(toned[1]),
            b: delinearize(toned[2]),
            a: 1.0,
        }
    }
}

// -----------------------------------------------------------------------------
// Size — semantic widths/heights for menus, popovers, scrollable columns, etc.
// -----------------------------------------------------------------------------

/// Semantic size token for recurring container widths and heights.
///
/// These values are HIG-aligned (`NSPopover` menus typically run 160–280pt
/// wide; scrollable lists use 240/320/480pt depending on density). Always
/// prefer this enum over a hard-coded `px(220.)` so the entire UI re-scales
/// when a future `ScaleFactor` token is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Size {
    /// 44 px — minimum interactive hit region.
    MinHitTarget,
    /// 28 px — compact button / toolbar affordance.
    ButtonSm,
    /// 32 px — default button.
    ButtonMd,
    /// 40 px — prominent dialog / sheet button.
    ButtonLg,
    /// 160 px — compact menu / dropdown.
    MenuCompact,
    /// 220 px — regular menu / popover (default).
    MenuRegular,
    /// 280 px — wide menu.
    MenuWide,
    /// 240 px — short scrollable column.
    ColumnShort,
    /// 320 px — regular scrollable column.
    ColumnRegular,
    /// 480 px — tall scrollable column.
    ColumnTall,
    /// 36 px — HIG menu row height (regular).
    RowMd,
    /// 44 px — HIG menu/list row height (touch-friendly).
    RowLg,
    /// 176 px — Music content tile outer width.
    ContentTileWidth,
    /// 152 px — Music content tile artwork edge.
    ContentTileArtwork,
    /// 600 px — readable measure for empty-state explanations and reports.
    NoticeWidth,
    /// 18 px — the visible square of a row-selection checkbox (ADR 0084
    /// Decision 10). The primitive wraps it in the pointer hit target.
    CheckboxIndicator,
    /// 28 px — the hit square of a small control inside a list row, such
    /// as the row checkbox and the disclosure chevron. The Apple HIG gives
    /// 28 × 28 pt as the default macOS control size for a pointer. The
    /// 44 px `MinHitTarget` is the iOS touch minimum, and stays the
    /// minimum height of the whole row.
    PointerHitTarget,
    /// 40 px — the distance that Up or Down scrolls a Music detail page
    /// with no selectable row (ADR 0084 Decision 10, packet 018). Page Up
    /// and Page Down scroll one view height less this step.
    KeyScrollStep,
    /// 3 px — the leading bar of a marked row (R-NAV-2 item 2, ADR 0089
    /// Decision 9). A marked row also uses the outline of a selected row.
    RowMarkBar,
    /// 480 px — the height that a virtual Music list renders above and
    /// below its view (ADR 0084 task 037). A row in this area is measured
    /// and shows its image before it scrolls into view.
    ListOverdraw,
}

impl Size {
    #[must_use]
    pub const fn px(self) -> Pixels {
        match self {
            Self::ButtonSm | Self::PointerHitTarget => px(28.0),
            Self::ButtonMd => px(32.0),
            Self::ButtonLg | Self::KeyScrollStep => px(40.0),
            Self::MenuCompact => px(160.0),
            Self::MenuRegular => px(220.0),
            Self::MenuWide => px(280.0),
            Self::ColumnShort => px(240.0),
            Self::ColumnRegular => px(320.0),
            Self::ColumnTall | Self::ListOverdraw => px(480.0),
            Self::RowMd => px(36.0),
            Self::MinHitTarget | Self::RowLg => px(44.0),
            Self::ContentTileWidth => px(176.0),
            Self::ContentTileArtwork => px(152.0),
            Self::NoticeWidth => px(600.0),
            Self::CheckboxIndicator => px(18.0),
            Self::RowMarkBar => px(3.0),
        }
    }

    /// ADR 0039: `Size` is a CHROME token — this resolves through
    /// [`scale_chrome_px`], never the TYPE domain.
    #[must_use]
    pub fn scaled(self, cx: &App) -> Pixels {
        scale_chrome_px(f32::from(self.px()), ScaleFactor::current(cx))
    }
}

// -----------------------------------------------------------------------------
// SkeletonBlock — redacted placeholder dimensions.
// -----------------------------------------------------------------------------

/// Semantic dimensions for skeleton placeholder blocks.
///
/// These values mirror the populated controls they stand in for, keeping
/// loading surfaces stable without scattering raw block sizes through
/// composites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkeletonBlock {
    /// 220 x 22 px — inspector title line.
    InspectorTitle,
    /// 160 x 14 px — inspector subtitle line.
    InspectorSubtitle,
    /// 120 x 12 px — inspector caption line.
    InspectorCaption,
    /// 96 x 12 px — recent-feed tile subtitle line.
    FeedTileSubtitle,
    /// 12 x 16 px — compact track-number placeholder.
    TrackNumber,
    /// 32 x 16 px — compact track-duration placeholder.
    TrackDuration,
}

impl SkeletonBlock {
    #[must_use]
    pub const fn px(self) -> (Pixels, Pixels) {
        match self {
            Self::InspectorTitle => (px(220.0), px(22.0)),
            Self::InspectorSubtitle => (px(160.0), px(14.0)),
            Self::InspectorCaption => (px(120.0), px(12.0)),
            Self::FeedTileSubtitle => (px(96.0), px(12.0)),
            Self::TrackNumber => (px(12.0), px(16.0)),
            Self::TrackDuration => (px(32.0), px(16.0)),
        }
    }

    /// ADR 0039: `SkeletonBlock` is a CHROME token — this resolves through
    /// [`scale_chrome_px`], never the TYPE domain.
    #[must_use]
    pub fn scaled(self, cx: &App) -> (Pixels, Pixels) {
        let (width, height) = self.px();
        let scale = ScaleFactor::current(cx);
        (
            scale_chrome_px(f32::from(width), scale),
            scale_chrome_px(f32::from(height), scale),
        )
    }
}

// -----------------------------------------------------------------------------
// TrackRowColumn — the fixed trailing columns of a track row.
// -----------------------------------------------------------------------------

/// The fixed trailing columns of a track row (operator finding of
/// 2026-10-20, ADR 0083 task 006). The time and library columns of a list
/// row (ADR 0089 Decision 6) take their widths from these tokens too.
///
/// Each column starts at the same position on each row of a track list,
/// also when the slot of the column is empty or hidden.
///
/// - The duration column holds text. Its width follows the TYPE scale of
///   its font role (ADR 0039), because the text grows with that role.
/// - The download-state column and the menu column hold one icon control.
///   Their width is [`Size::PointerHitTarget`] on the CHROME scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackRowColumn {
    /// [`Size::MinHitTarget`] — the download-state slot: the download
    /// button, or the queued or the downloading icon. The column has the
    /// width of the hit area of a `Button`, so it cuts no hover fill.
    DownloadState,
    /// 44 px text at `FontSize::Micro` — the duration, for example
    /// "1:02:03". The text is right-aligned.
    Duration,
    /// [`Size::MinHitTarget`] — the "⋯" menu button, the same width as the
    /// hit area of a `Button`.
    Menu,
}

impl TrackRowColumn {
    /// The trailing columns in row order, after the title.
    pub const ROW_ORDER: [Self; 3] = [Self::DownloadState, Self::Duration, Self::Menu];

    /// The width of the duration text at the base (1.0×) type size.
    const DURATION_TEXT_PX: f32 = 44.0;

    /// The column width at `scale`. The duration column follows the TYPE
    /// scale of `FontSize::Micro`. Each icon column follows the CHROME
    /// scale.
    #[must_use]
    pub fn scaled_px(self, scale: ScaleFactor) -> Pixels {
        match self {
            Self::Duration => px(Self::DURATION_TEXT_PX * FontSize::Micro.type_multiplier(scale)),
            Self::DownloadState | Self::Menu => {
                scale_chrome_px(f32::from(Size::MinHitTarget.px()), scale)
            }
        }
    }

    /// Same as [`Self::scaled_px`] at the active [`ScaleFactor`].
    #[must_use]
    pub fn scaled(self, cx: &App) -> Pixels {
        self.scaled_px(ScaleFactor::current(cx))
    }
}

// -----------------------------------------------------------------------------
// ScaleFactor — Apple Dynamic Type-style runtime UI scale.
// -----------------------------------------------------------------------------

/// Global UI scale, mirroring the named steps of iOS Dynamic Type.
///
/// Multiplied into every dimension token's `.scaled(cx)` accessor so the
/// entire UI can shrink or grow without any per-screen code change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScaleFactor {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
    XLarge,
}

impl ScaleFactor {
    /// ADR 0039 CHROME domain: the five geometry coefficients.
    ///
    /// Bit-identical to the pre-ADR-0039 uniform `ScaleFactor::multiplier()`
    /// values at every step. `Spacing`, `Radius`, `Size`, `SkeletonBlock`,
    /// icons, images, thumbnails and the geometry bridges listed in ADR 0039
    /// task 001 all route through this resolver; `FontSize` never does — see
    /// [`FontSize::type_multiplier`] for the TYPE domain.
    #[must_use]
    pub const fn chrome_multiplier(self) -> f32 {
        match self {
            Self::XSmall => 0.85,
            Self::Small => 0.92,
            Self::Medium => 1.0,
            Self::Large => 1.12,
            Self::XLarge => 1.25,
        }
    }

    /// Returns the scale stored on `cx`, or [`ScaleFactor::Medium`] if none
    /// has been installed yet (e.g. early-startup paths or tests).
    ///
    /// Prefers the bundled [`Environment`] global; falls back to the legacy
    /// stand-alone `ScaleFactor` global, then to the default.
    #[must_use]
    pub fn current(cx: &App) -> Self {
        if let Some(env) = cx.try_global::<Environment>() {
            return env.scale;
        }
        cx.try_global::<ScaleFactor>().copied().unwrap_or_default()
    }
}

impl gpui::Global for ScaleFactor {}

// -----------------------------------------------------------------------------
// Motion — named times of the interface.
// -----------------------------------------------------------------------------

/// The named times of the interface (ADR 0101 Decision 6).
///
/// The code that waits for a time reads it from this token. It holds no
/// time literal of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Motion {
    /// 500 ms — the time that a dragged track stays over a spring-loaded
    /// sidebar row before the row opens its destination. The value is near
    /// the default spring-loading delay of the macOS Finder. The operator
    /// walk of ADR 0083 task 008 confirms it.
    SpringLoadDelay,
    /// 300 ms. The pause after the last typed character before the search
    /// runs (ADR 0087 Decision 4).
    SearchPause,
}

impl Motion {
    /// The time of the token.
    #[must_use]
    pub const fn duration(self) -> Duration {
        match self {
            Self::SpringLoadDelay => Duration::from_millis(500),
            Self::SearchPause => Duration::from_millis(300),
        }
    }
}

// -----------------------------------------------------------------------------
// Environment — SwiftUI-style bundle of every value the UI tree reads at render.
// -----------------------------------------------------------------------------

/// Bundled rendering context that primitives and composites consult.
///
/// Modeled after `SwiftUI`'s `Environment`: a single typed value carrying the
/// appearance scheme, Dynamic-Type-style scale, and motion preference that
/// every component observes. Today it lives as a `gpui::Global` (app-scoped); per-subtree
/// override would be a wrapper element that swaps the global for the duration
/// of its render.
///
/// # Examples
///
/// ```ignore
/// use crate::ui::tokens::Environment;
/// let env = Environment::current(cx);
/// let bg = crate::ui::theme_profiles::resolve_profile_color(
///     env.profile,
///     SemanticColor::SystemBackground,
/// );
/// let pad = Spacing::Md.scaled(cx);   // reads env.scale internally
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Environment {
    pub profile: ThemeProfile,
    pub appearance: Appearance,
    pub scale: ScaleFactor,
    /// Mirrors the system Reduce Motion setting. Keep all animation decisions
    /// routed through [`Self::allows_motion`].
    pub reduce_motion: bool,
    /// Mirrors the system Increase Contrast setting (ADR 0101 Decision 1 and
    /// Decision 11, musicindex ADR 0068 R-A11Y-1).
    pub increase_contrast: bool,
    /// Mirrors the system Reduce Transparency setting. A translucent fill
    /// becomes an opaque fill.
    pub reduce_transparency: bool,
}

impl Environment {
    /// Returns the environment installed on `cx`, or the default
    /// (Dark / Medium) if none has been installed yet (early startup, tests).
    #[must_use]
    pub fn current(cx: &App) -> Self {
        cx.try_global::<Environment>().copied().unwrap_or_default()
    }

    /// Returns whether user-visible animation may run.
    #[must_use]
    pub const fn allows_motion(self) -> bool {
        !self.reduce_motion
    }

    /// Installs the environment as a `gpui::Global` and mirrors the scale
    /// into the legacy `ScaleFactor` global so any code still reading that
    /// directly keeps working during the migration.
    pub fn install(self, cx: &mut App) {
        cx.set_global(self);
        cx.set_global(self.scale);
    }
}

impl gpui::Global for Environment {}

// -----------------------------------------------------------------------------
// Helpers.
// -----------------------------------------------------------------------------

/// ADR 0039 CHROME domain: multiply a base point value by the chrome
/// coefficient and return `Pixels`.
///
/// Used by `Spacing`, `Radius`, `Size` and `SkeletonBlock`. `FontSize` never
/// calls this — see [`FontSize::scaled_px`] for the TYPE domain's own pure
/// resolver.
#[inline]
fn scale_chrome_px(base: f32, scale: ScaleFactor) -> Pixels {
    px(base * scale.chrome_multiplier())
}

#[inline]
const fn hex(rgb: u32) -> Rgba {
    // Decode 0xRRGGBB into a fully opaque sRGB color. The integer-to-float
    // casts are bounded to [0, 255] so precision loss is irrelevant.
    #[expect(
        clippy::cast_precision_loss,
        reason = "byte values 0..=255 fit exactly in f32"
    )]
    Rgba {
        r: ((rgb >> 16) & 0xff) as f32 / 255.0,
        g: ((rgb >> 8) & 0xff) as f32 / 255.0,
        b: (rgb & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

/// Decode `0xRRGGBB` with an explicit alpha. ADR 0101 Decision 1 states
/// `Separator` as a translucent overlay. In dark it is `white at 10%`. In
/// light it is `#3c3c43 at 16%`. This reads correctly over any surface
/// behind it.
#[inline]
const fn hex_alpha(rgb: u32, alpha: f32) -> Rgba {
    #[expect(
        clippy::cast_precision_loss,
        reason = "byte values 0..=255 fit exactly in f32"
    )]
    Rgba {
        r: ((rgb >> 16) & 0xff) as f32 / 255.0,
        g: ((rgb >> 8) & 0xff) as f32 / 255.0,
        b: (rgb & 0xff) as f32 / 255.0,
        a: alpha,
    }
}

/// ADR 0063: all log bodies use the same compact, scaled monospace role.
pub const LOG_TEXT_SIZE: FontSize = FontSize::Caption;
pub const LOG_LINE_HEIGHT: f32 = 1.5;

#[must_use]
pub fn log_font_family(cx: &App) -> gpui::SharedString {
    cx.theme().mono_font_family.clone()
}

// -----------------------------------------------------------------------------
// Tests.
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0084 Decision 10: a row control has the macOS pointer size, and
    /// its visible box fits inside it.
    #[test]
    fn pointer_hit_target_is_the_macos_control_size() {
        assert_eq!(Size::PointerHitTarget.px(), px(28.0));
        assert!(Size::CheckboxIndicator.px() < Size::PointerHitTarget.px());
        assert!(Size::PointerHitTarget.px() < Size::MinHitTarget.px());
    }

    /// R18-09, ADR 0084 packet 018: the key scroll step of a detail page
    /// with no selectable row comes from the named token. The view height
    /// step is one view height less one step.
    #[test]
    fn adr_0084_detail_selection_r18_09_key_scroll_uses_the_token_step() {
        use crate::ui::composites::music_column_scroll::key_scroll_offset_y;
        use crate::view_models::list_selection::FocusStep;

        let step = f32::from(Size::KeyScrollStep.px());
        assert!((step - 40.0).abs() < f32::EPSILON);
        let (view, max) = (400.0, 1000.0);
        let at = |key, offset| key_scroll_offset_y(key, offset, view, max, step);
        let near = |left: f32, right: f32| (left - right).abs() < f32::EPSILON;

        assert!(near(at(FocusStep::Down, -100.0), -100.0 - step));
        assert!(near(at(FocusStep::Up, -100.0), -100.0 + step));
        assert!(near(
            at(FocusStep::PageDown, -100.0),
            -100.0 - (view - step)
        ));
        assert!(near(at(FocusStep::PageUp, -500.0), -500.0 + (view - step)));
        assert!(near(at(FocusStep::Home, -500.0), 0.0));
        assert!(near(at(FocusStep::End, 0.0), -max));
        // The offset stops at the top and at the bottom.
        assert!(near(at(FocusStep::Up, 0.0), 0.0));
        assert!(near(at(FocusStep::PageDown, -990.0), -max));
    }

    #[test]
    fn dark_label_contrast_is_high() {
        // Sanity check: label-on-systemBackground must clear WCAG AA normal
        // text (4.5:1). ADR 0101 Decision 1 moved both tokens off their old
        // hex values, so this checks the ratio directly rather than
        // hardcoding a near-white / near-black channel threshold.
        let label = SemanticColor::Label.resolve(Appearance::Dark);
        let bg = SemanticColor::SystemBackground.resolve(Appearance::Dark);
        assert!(crate::ui::contrast::ratio(label, bg) >= 4.5);
    }

    #[test]
    fn light_label_contrast_is_high() {
        // See `dark_label_contrast_is_high`: ADR 0101 Decision 1 made Light
        // Label `#1d1d1f`, not pure black, so this checks the ratio directly.
        let label = SemanticColor::Label.resolve(Appearance::Light);
        let bg = SemanticColor::SystemBackground.resolve(Appearance::Light);
        assert!(crate::ui::contrast::ratio(label, bg) >= 4.5);
    }

    #[test]
    fn spacing_is_monotonic() {
        let sizes = [
            Spacing::XXS,
            Spacing::XS,
            Spacing::SM,
            Spacing::MD,
            Spacing::LG,
            Spacing::LGPlus,
            Spacing::XL,
            Spacing::XXL,
            Spacing::XXXL,
        ];
        for w in sizes.windows(2) {
            assert!(w[0].px() < w[1].px());
        }
    }

    #[test]
    fn radius_is_monotonic_until_full() {
        assert!(Radius::SM.px() < Radius::MD.px());
        assert!(Radius::MD.px() < Radius::LG.px());
        assert!(Radius::LG.px() < Radius::XL.px());
        assert!(Radius::XL.px() < Radius::Sheet.px());
        assert!(Radius::Sheet.px() < Radius::Full.px());
    }

    #[test]
    fn skeleton_block_tokens_match_placeholder_footprints() {
        assert_eq!(SkeletonBlock::InspectorTitle.px(), (px(220.0), px(22.0)));
        assert_eq!(SkeletonBlock::InspectorSubtitle.px(), (px(160.0), px(14.0)));
        assert_eq!(SkeletonBlock::InspectorCaption.px(), (px(120.0), px(12.0)));
        assert_eq!(SkeletonBlock::FeedTileSubtitle.px(), (px(96.0), px(12.0)));
        assert_eq!(SkeletonBlock::TrackNumber.px(), (px(12.0), px(16.0)));
        assert_eq!(SkeletonBlock::TrackDuration.px(), (px(32.0), px(16.0)));
    }

    #[test]
    fn reduce_motion_disables_environment_motion() {
        let env = Environment {
            reduce_motion: true,
            ..Environment::default()
        };

        assert!(!env.allows_motion());
    }

    #[test]
    fn increase_contrast_swaps_separator_and_secondary_label_only() {
        assert_eq!(
            SemanticColor::Separator.for_contrast(true),
            SemanticColor::OpaqueSeparator
        );
        assert_eq!(
            SemanticColor::SecondaryLabel.for_contrast(true),
            SemanticColor::Label
        );
        assert_eq!(
            SemanticColor::Separator.for_contrast(false),
            SemanticColor::Separator
        );
        assert_eq!(
            SemanticColor::Accent.for_contrast(true),
            SemanticColor::Accent
        );
    }

    #[test]
    fn flatten_over_gives_an_opaque_blend() {
        let white_half = Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.5,
        };
        let black = Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let flat = flatten_over(white_half, black);
        assert!((flat.a - 1.0).abs() < f32::EPSILON);
        assert!((flat.r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn hex_decodes_correctly() {
        let c = hex(0xff_8040);
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.g - 0.501_960_8).abs() < 1e-4);
        assert!((c.b - 0.250_980_4).abs() < 1e-4);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    /// musicindex ADR 0025 R-STATUS-1: a status text color has its shared value.
    /// The dark info text is the one difference, because the accent has 4.36:1.
    /// Fix: copy the value from the rule, or supersede the rule in musicindex.
    #[test]
    fn status_text_colors_match_the_shared_rule() {
        for (appearance, token, rgb) in [
            (Appearance::Dark, SemanticColor::SuccessLabel, 0x30_d158),
            (Appearance::Dark, SemanticColor::WarningLabel, 0xff_9f0a),
            (Appearance::Dark, SemanticColor::DangerLabel, 0xff_6961),
            (Appearance::Light, SemanticColor::SuccessLabel, 0x1a_7f37),
            (Appearance::Light, SemanticColor::WarningLabel, 0xa0_5a00),
            (Appearance::Light, SemanticColor::DangerLabel, 0xc4_1e3a),
            (Appearance::Light, SemanticColor::InfoLabel, 0x0a_5bd6),
        ] {
            assert_eq!(
                token.resolve(appearance),
                hex(rgb),
                "{token:?} in {appearance:?}"
            );
        }
    }

    /// musicindex ADR 0032 R-DOT-2: the button tints and fills match the shared copy in
    /// `src/assets/tokens.json`. Fix: copy the value from that file.
    #[test]
    fn button_tints_match_the_shared_tokens() {
        let shared: serde_json::Value =
            serde_json::from_str(include_str!("../assets/tokens.json")).expect("valid JSON");
        for (appearance, key) in [(Appearance::Dark, "dark"), (Appearance::Light, "light")] {
            for (token, name) in [
                (SemanticColor::AccentTint, "accent-tint"),
                (SemanticColor::DangerTint, "danger-tint"),
                (SemanticColor::ButtonFill, "button-fill"),
                (SemanticColor::ButtonFillHover, "button-fill-hover"),
                (SemanticColor::SelectedTop, "selected-top"),
                (SemanticColor::SelectedBottom, "selected-bottom"),
                (SemanticColor::SelectedEdge, "selected-edge"),
                (SemanticColor::SelectedHighlight, "selected-highlight"),
            ] {
                let color = token.resolve(appearance);
                let channel = |value: f32| (value * 255.0).round();
                let alpha = format!("{:.2}", color.a);
                let alpha = alpha.trim_end_matches('0').trim_start_matches('0');
                let expected = format!(
                    "rgba({}, {}, {}, {alpha})",
                    channel(color.r),
                    channel(color.g),
                    channel(color.b),
                );
                assert_eq!(
                    shared["color"][key][name].as_str(),
                    Some(expected.as_str()),
                    "{name} in {key}"
                );
            }
        }
    }

    fn shared_tokens() -> serde_json::Value {
        serde_json::from_str(include_str!("../assets/tokens.json")).expect("valid JSON")
    }

    /// musicindex ADR 0021 R-ART-4 item 2: the art radius is 6 % of the edge,
    /// rounded half up, with a minimum of 4 px.
    #[test]
    fn r_art_4_art_radius_is_six_percent_of_the_edge_with_a_minimum() {
        for (edge, radius) in [
            (32.0, 4.0),
            (40.0, 4.0),
            (48.0, 4.0),
            (84.0, 5.0),
            (152.0, 9.0),
            (200.0, 12.0),
        ] {
            assert!(
                (ArtworkRadius::px_for_edge(edge) - radius).abs() < f32::EPSILON,
                "edge {edge} must give radius {radius}"
            );
        }
        // 75 px gives 4.5 px, which rounds half up to 5 px.
        assert!((ArtworkRadius::px_for_edge(75.0) - 5.0).abs() < f32::EPSILON);
    }

    /// musicindex ADR 0021 R-ART-4 item 2: `ArtworkRadius` matches the shared
    /// token `shape.art-radius` in `src/assets/tokens.json`. Fix: copy the
    /// value from that file into `ArtworkRadius`, or ask for a new shared copy.
    #[test]
    fn r_art_4_art_radius_matches_the_shared_token() {
        let shared = shared_tokens();
        let token = &shared["shape"]["art-radius"];
        let fraction = token["fraction-of-edge"].as_f64().expect("a fraction");
        let minimum = token["minimum-px"].as_f64().expect("a minimum");
        assert!(
            (fraction - f64::from(ArtworkRadius::FRACTION_OF_EDGE)).abs() < 1e-6,
            "R-ART-4 item 2: fraction-of-edge is {fraction}. Fix: copy it into ArtworkRadius."
        );
        assert!(
            (minimum - f64::from(ArtworkRadius::MINIMUM_PX)).abs() < 1e-6,
            "R-ART-4 item 2: minimum-px is {minimum}. Fix: copy it into ArtworkRadius."
        );
        assert_eq!(token["rounding"].as_str(), Some("nearest pixel"));
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a color channel is 0 to 1, so the result is 0 to 255"
    )]
    fn rgb_of(color: Rgba) -> String {
        let channel = |value: f32| (value * 255.0).round() as u32;
        format!(
            "#{:02x}{:02x}{:02x}",
            channel(color.r),
            channel(color.g),
            channel(color.b)
        )
    }

    fn px_value(value: Pixels) -> f64 {
        f64::from(f32::from(value))
    }

    /// musicindex ADR 0080 R-TOKEN-1: the focus color of each theme equals the shared copy
    /// in `src/assets/tokens.json`. Fix: copy `color.dark.focus` and `color.light.focus`.
    #[test]
    fn focus_color_matches_the_shared_tokens() {
        let shared = shared_tokens();
        for (appearance, key) in [(Appearance::Dark, "dark"), (Appearance::Light, "light")] {
            assert_eq!(
                Some(rgb_of(SemanticColor::Focus.resolve(appearance)).as_str()),
                shared["color"][key]["focus"].as_str(),
                "musicindex ADR 0080 R-TOKEN-1: Focus in {key} must equal color.{key}.focus. \
                 Fix: copy the value from src/assets/tokens.json."
            );
        }
    }

    /// musicindex ADR 0068 R-TYPE-1 and R-TYPE-2: each `FontSize` role is a desktop size of the
    /// shared copy. Title is the large title and Display is the artist and label page title.
    /// Fix: use a size of `type.desktop` or `type.page-title.artist-label.display`.
    #[test]
    fn font_sizes_match_the_shared_tokens() {
        let shared = shared_tokens();
        let mut allowed: Vec<f64> = shared["type"]["desktop"]
            .as_object()
            .expect("type.desktop")
            .values()
            .map(|entry| entry[0].as_f64().expect("size"))
            .collect();
        let display = shared["type"]["page-title"]["artist-label"]["display"][0]
            .as_f64()
            .expect("display size");
        allowed.push(display);
        for role in [
            FontSize::Micro,
            FontSize::Caption,
            FontSize::Body,
            FontSize::Headline,
            FontSize::Title3,
            FontSize::Title2,
            FontSize::Title,
            FontSize::Display,
        ] {
            let size = px_value(role.px());
            assert!(
                allowed.contains(&size),
                "musicindex ADR 0068 R-TYPE-1: {role:?} is {size} px, not a size of type.desktop. \
                 Fix: use a size from src/assets/tokens.json."
            );
        }
        assert!(
            (px_value(FontSize::Title.px())
                - shared["type"]["desktop"]["large-title"][0]
                    .as_f64()
                    .expect("large title"))
            .abs()
                < f64::EPSILON,
            "musicindex ADR 0068 R-TYPE-2: Title must equal type.desktop.large-title."
        );
        assert!(
            (px_value(FontSize::Display.px()) - display).abs() < f64::EPSILON,
            "musicindex ADR 0068 R-TYPE-2: Display must equal type.page-title.artist-label.display."
        );
    }

    /// musicindex ADR 0068 R-RADIUS-1: the radii equal `shape.radius` of the shared copy.
    /// Fix: copy the value from `src/assets/tokens.json`.
    #[test]
    fn radii_match_the_shared_tokens() {
        let shared = shared_tokens();
        for (role, name) in [
            (Radius::SM, "chip"),
            (Radius::MD, "field"),
            (Radius::LG, "popover"),
            (Radius::XL, "card"),
            (Radius::Sheet, "sheet"),
        ] {
            assert!(
                (px_value(role.px()) - shared["shape"]["radius"][name].as_f64().expect(name)).abs()
                    < f64::EPSILON,
                "musicindex ADR 0068 R-RADIUS-1: {role:?} must equal shape.radius.{name}."
            );
        }
    }

    /// musicindex ADR 0068 R-SPACE-1: the spacing roles equal the `space` list of the shared
    /// copy, in order. Fix: copy the list from `src/assets/tokens.json`.
    #[test]
    fn spacing_matches_the_shared_tokens() {
        let shared = shared_tokens();
        let expected: Vec<f64> = shared["space"]
            .as_array()
            .expect("space")
            .iter()
            .map(|value| value.as_f64().expect("number"))
            .collect();
        let actual: Vec<f64> = [
            Spacing::XXS,
            Spacing::XS,
            Spacing::SM,
            Spacing::MD,
            Spacing::LG,
            Spacing::LGPlus,
            Spacing::XL,
            Spacing::XXL,
            Spacing::XXXL,
        ]
        .into_iter()
        .map(|role| px_value(role.px()))
        .collect();
        assert_eq!(
            actual, expected,
            "musicindex ADR 0068 R-SPACE-1: Spacing must equal the space list, in order."
        );
    }

    /// musicindex ADR 0045 R-ELEVATION-2: the page, group and open entry levels equal `level`
    /// of the shared copy, in order. Fix: copy the values from `src/assets/tokens.json`.
    #[test]
    fn levels_match_the_shared_tokens() {
        let shared = shared_tokens();
        for (appearance, key) in [(Appearance::Dark, "dark"), (Appearance::Light, "light")] {
            let actual: Vec<String> = [
                SemanticColor::SystemBackground,
                SemanticColor::GroupBackground,
                SemanticColor::OpenEntryBackground,
            ]
            .into_iter()
            .map(|token| rgb_of(token.resolve(appearance)))
            .collect();
            let expected: Vec<String> = shared["level"][key]
                .as_array()
                .expect("level list")
                .iter()
                .map(|value| value.as_str().expect("string").to_owned())
                .collect();
            assert_eq!(
                actual, expected,
                "musicindex ADR 0045 R-ELEVATION-2: the {key} levels must equal level.{key}."
            );
        }
    }

    /// musicindex ADR 0068 R-ELEVATION-1 item 2: the floating shadow equals
    /// `shadow.floating` of the shared copy in each theme. Fix: copy the values from
    /// `src/assets/tokens.json`.
    #[gpui::test]
    fn floating_shadow_matches_the_shared_tokens(cx: &mut gpui::TestAppContext) {
        let shared = shared_tokens();
        let floating = &shared["shadow"]["floating"];
        let number = |key: &str| floating[key].as_f64().expect("number");
        assert!((f64::from(FloatingShadow::OFFSET_Y) - number("offset-y")).abs() < 1e-6);
        assert!((f64::from(FloatingShadow::BLUR) - number("blur")).abs() < 1e-6);
        for (appearance, key) in [
            (Appearance::Dark, "dark-alpha"),
            (Appearance::Light, "light-alpha"),
        ] {
            assert!(
                (f64::from(FloatingShadow::alpha(appearance)) - number(key)).abs() < 1e-6,
                "musicindex ADR 0068 R-ELEVATION-1: the {key} must equal shadow.floating.{key}."
            );
            cx.update(|cx| {
                let layers = FloatingShadow::shadow(cx, Some(appearance));
                assert_eq!(layers.len(), 1, "a floating layer has exactly one shadow");
                assert!(!layers[0].inset, "the floating shadow is not inset");
                assert!(
                    (f64::from(layers[0].color.a) - number(key)).abs() < 1e-6,
                    "the shadow color alpha must equal shadow.floating.{key}."
                );
            });
        }
    }

    /// ADR 0101 Decision 1, task 012 step 4: each hover fill differs from the
    /// fill below it in each theme. Each row of the table names the hover fill,
    /// the callers and the fills below. Fix: change the hover token of the
    /// caller, and add the new pair to this table.
    #[test]
    fn each_hover_fill_differs_from_the_fill_below_it_in_each_theme() {
        use SemanticColor::{
            FloatingBackground, GroupBackground, HoverBackground, OpenEntryBackground,
            SidebarBackground, SystemBackground,
        };
        let cases: [(SemanticColor, &str, &[SemanticColor]); 3] = [
            (
                GroupBackground,
                "list row, tile, plain button, drag handle, unavailable playlist row",
                &[SystemBackground, FloatingBackground, OpenEntryBackground],
            ),
            (
                HoverBackground,
                "available playlist row and sidebar row",
                &[SystemBackground, SidebarBackground],
            ),
            (OpenEntryBackground, "show card", &[GroupBackground]),
        ];
        for profile in [
            ThemeProfile::Dark,
            ThemeProfile::Light,
            ThemeProfile::HighContrastDark,
            ThemeProfile::HighContrastLight,
        ] {
            for (hover, callers, below) in cases {
                for fill in below {
                    assert_ne!(
                        crate::ui::theme_profiles::resolve_profile_color(profile, hover),
                        crate::ui::theme_profiles::resolve_profile_color(profile, *fill),
                        "ADR 0101 Decision 1: the hover fill {hover:?} of {callers} equals {fill:?} in {profile:?}. Give the hover another token."
                    );
                }
            }
        }
    }

    /// musicindex ADR 0073 R-FOCUS-2: `HitIt` draws the focus ring as an inset border, so its
    /// width equals `focus-ring.hitit.width` and its offset is 0. Fix: keep
    /// `CONTROL_FOCUS_RING_WIDTH` equal to that width and draw the ring inset.
    #[test]
    fn focus_ring_matches_the_shared_tokens() {
        let shared = shared_tokens();
        assert!(
            (px_value(crate::ui::layouts::CONTROL_FOCUS_RING_WIDTH)
                - shared["focus-ring"]["hitit"]["width"]
                    .as_f64()
                    .expect("width"))
            .abs()
                < f64::EPSILON,
            "musicindex ADR 0073 R-FOCUS-2: CONTROL_FOCUS_RING_WIDTH must equal focus-ring.hitit.width."
        );
        assert_eq!(
            shared["focus-ring"]["hitit"]["offset"].as_f64(),
            Some(0.0),
            "musicindex ADR 0073 R-FOCUS-2: HitIt draws the ring inset, so focus-ring.hitit.offset is 0."
        );
        assert!(
            (px_value(crate::ui::layouts::CONTROL_FOCUS_RING_FILLED_OFFSET)
                - shared["focus-ring"]["hitit"]["filled-offset"]
                    .as_f64()
                    .expect("filled-offset"))
            .abs()
                < f64::EPSILON,
            "musicindex ADR 0073 R-FOCUS-2 item 6: CONTROL_FOCUS_RING_FILLED_OFFSET must equal \
             focus-ring.hitit.filled-offset. Fix: copy the value from src/assets/tokens.json."
        );
    }

    /// musicindex ADR 0056 R-SIDE-1: the glow color of each theme and the glow
    /// length match the shared copy in `src/assets/tokens.json`. Fix: copy the
    /// value from that file.
    #[test]
    fn adr_0056_glow_tokens_match_the_shared_tokens() {
        let shared: serde_json::Value =
            serde_json::from_str(include_str!("../assets/tokens.json")).expect("valid JSON");
        for (appearance, key, expected) in [
            (Appearance::Dark, "dark", "rgba(45, 123, 255, .09)"),
            (Appearance::Light, "light", "rgba(10, 91, 214, .09)"),
        ] {
            assert_eq!(
                shared["color"][key]["sidebar-glow-top"].as_str(),
                Some(expected),
                "{key} value in the file"
            );
            let color = SemanticColor::SidebarGlowTop.resolve(appearance);
            let channel = |value: f32| (value * 255.0).round();
            let alpha = format!("{:.2}", color.a);
            let alpha = alpha.trim_end_matches('0').trim_start_matches('0');
            let actual = format!(
                "rgba({}, {}, {}, {alpha})",
                channel(color.r),
                channel(color.g),
                channel(color.b),
            );
            assert_eq!(actual, expected, "{key} token");
        }
        assert_eq!(
            shared["sidebar"]["glow-length-px"].as_f64(),
            Some(f64::from(SidebarGlow::LENGTH_PX))
        );
        assert!((SidebarGlow::LENGTH_PX - 320.0).abs() < f32::EPSILON);
    }

    /// musicindex ADR 0056 R-SIDE-1 item 1: the gradient runs from the glow
    /// color at the top to the same color with no opacity at the bottom.
    #[test]
    fn adr_0056_glow_gradient_fades_to_clear() {
        let top = SemanticColor::SidebarGlowTop.resolve(Appearance::Dark);
        let [stop_from, stop_to] = SidebarGlow::stops(top);
        let from = Rgba::from(stop_from.color);
        let to = Rgba::from(stop_to.color);
        assert!((from.a - top.a).abs() < 1e-4);
        assert!(to.a.abs() < f32::EPSILON);
        assert!(stop_from.percentage.abs() < f32::EPSILON);
        assert!((stop_to.percentage - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hex_alpha_keeps_the_requested_alpha() {
        let c = hex_alpha(0xff_ffff, 0.10);
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.g - 1.0).abs() < f32::EPSILON);
        assert!((c.b - 1.0).abs() < f32::EPSILON);
        assert!((c.a - 0.10).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------
    // ADR 0083 task 001: palette and color roles.
    // -------------------------------------------------------------------

    /// R83-01: the Dark and Light values of each ADR 0101 Decision 1 role,
    /// through `SemanticColor::resolve`.
    #[test]
    fn adr_0083_decision_1_roles_resolve_to_the_website_values() {
        let cases: [(SemanticColor, u32, u32); 7] = [
            (SemanticColor::SystemBackground, 0x0b_0b0d, 0xff_ffff),
            (SemanticColor::SidebarBackground, 0x14_1417, 0xf5_f5f7),
            (SemanticColor::GroupBackground, 0x1c_1c1f, 0xf2_f2f7),
            (SemanticColor::HoverBackground, 0x26_262a, 0xf2_f2f7),
            (SemanticColor::Label, 0xf5_f5f7, 0x1d_1d1f),
            (SemanticColor::SecondaryLabel, 0xa1_a1a6, 0x5c_5c62),
            (SemanticColor::Accent, 0x2d_7bff, 0x0a_5bd6),
        ];
        for (token, dark_rgb, light_rgb) in cases {
            assert_eq!(
                token.resolve(Appearance::Dark),
                hex(dark_rgb),
                "{token:?} dark value drifted from ADR 0101 Decision 1"
            );
            assert_eq!(
                token.resolve(Appearance::Light),
                hex(light_rgb),
                "{token:?} light value drifted from ADR 0101 Decision 1"
            );
        }

        // Separator is a translucent overlay, not a solid hex.
        assert_eq!(
            SemanticColor::Separator.resolve(Appearance::Dark),
            hex_alpha(0xff_ffff, 0.10),
            "dark Separator must be white at 10%"
        );
        assert_eq!(
            SemanticColor::Separator.resolve(Appearance::Light),
            hex_alpha(0x3c_3c43, 0.16),
            "light Separator must be #3c3c43 at 16%"
        );
    }

    /// R83-05: the dark `Warning` and `WarningLabel` moved off gold.
    /// ADR 0101 Decision 2 frees `#ffd666` for the value palette.
    #[test]
    fn adr_0083_dark_warning_is_not_gold() {
        let former_gold = hex(0xff_d666);
        assert_ne!(
            SemanticColor::Warning.resolve(Appearance::Dark),
            former_gold,
            "dark Warning must not resolve to the former gold #ffd666"
        );
        assert_ne!(
            SemanticColor::WarningLabel.resolve(Appearance::Dark),
            former_gold,
            "dark WarningLabel must not resolve to the former gold #ffd666"
        );
        assert_eq!(
            SemanticColor::Warning.resolve(Appearance::Dark),
            hex(0xff_9f0a),
            "dark Warning must start from #ff9f0a"
        );
    }

    /// ADR 0101 Decision 2: `DiffDifferent` and `Id3FrameV23Only` moved away
    /// from gold in dark the same way as `Warning`.
    #[test]
    fn adr_0083_dark_diff_and_frame_chip_move_away_from_gold() {
        let former_gold = hex(0xff_d666);
        for token in [SemanticColor::DiffDifferent, SemanticColor::Id3FrameV23Only] {
            let resolved = token.resolve(Appearance::Dark);
            assert_ne!(
                resolved, former_gold,
                "{token:?} dark value must not resolve to the former gold #ffd666"
            );
        }
    }

    // -------------------------------------------------------------------
    // ADR 0083 task 002: font and artwork tokens.
    // -------------------------------------------------------------------

    /// R83-14: larger artwork gets a larger shadow.
    /// ADR 0083 task 005: the header text keeps its contrast on the cover
    /// backdrop for every cover color. `Label` and `SecondaryLabel` meet
    /// normal text contrast. `TertiaryLabel`, the header data row label,
    /// meets large text contrast, as it does on the plain background.
    #[test]
    fn adr_0083_cover_backdrop_keeps_header_text_contrast() {
        use crate::ui::contrast::{ratio, ContrastLevel};

        let mut covers = Vec::new();
        for red in [0u8, 64, 128, 192, 255] {
            for green in [0u8, 64, 128, 192, 255] {
                for blue in [0u8, 64, 128, 192, 255] {
                    covers.push(CoverColor { red, green, blue });
                }
            }
        }
        for appearance in [Appearance::Dark, Appearance::Light] {
            for cover in &covers {
                let backdrop = CoverBackdrop::tone(*cover, appearance);
                for (token, level) in [
                    (SemanticColor::Label, ContrastLevel::NormalText),
                    (SemanticColor::SecondaryLabel, ContrastLevel::NormalText),
                    (SemanticColor::TertiaryLabel, ContrastLevel::LargeOrGraphic),
                ] {
                    let contrast = ratio(token.resolve(appearance), backdrop);
                    assert!(
                        contrast >= level.min_ratio(),
                        "{token:?} on the {appearance:?} backdrop of {cover:?} has contrast {contrast}"
                    );
                }
            }
        }
    }

    /// The glow style with the colors of the real theme, for the glow tests.
    fn real_glow_style(appearance: Appearance) -> GlowStyle {
        let profile = match appearance {
            Appearance::Dark => ThemeProfile::Dark,
            Appearance::Light => ThemeProfile::Light,
        };
        CoverBackdrop::glow_style(Environment {
            profile,
            appearance,
            ..Environment::default()
        })
    }

    /// Task 002 correction 3: the page shows the grain tile at the size of
    /// the tile that the media layer makes, and the pattern covers the area.
    #[test]
    fn adr_0101_header_glow_grain_pattern_covers_its_area() {
        assert!(
            (CoverBackdrop::GRAIN_TILE_SIDE
                - f32::from(
                    u16::try_from(crate::media::header_glow::GRAIN_TILE_SIDE).expect("small")
                ))
            .abs()
                < f32::EPSILON,
            "ADR 0101 Decision 4: the token and media::header_glow::GRAIN_TILE_SIDE must agree."
        );
        assert_eq!(CoverBackdrop::grain_cells(px(240.0), px(240.0)), (1, 1));
        assert_eq!(CoverBackdrop::grain_cells(px(241.0), px(480.0)), (2, 2));
        assert_eq!(CoverBackdrop::grain_cells(px(0.0), px(0.0)), (1, 1));
        let (columns, rows) = CoverBackdrop::grain_cells(px(1000.0), px(800.0));
        assert_eq!((columns, rows), (5, 4));
    }

    /// Task 002 correction 7: the glow style follows the live theme. The
    /// profile and Increase Contrast change the colors and the cache key.
    #[test]
    fn adr_0101_header_glow_style_follows_the_live_theme() {
        let base = Environment {
            profile: ThemeProfile::Dark,
            appearance: Appearance::Dark,
            ..Environment::default()
        };
        let plain = CoverBackdrop::glow_style(base);
        let contrast = CoverBackdrop::glow_style(Environment {
            increase_contrast: true,
            ..base
        });
        let high = CoverBackdrop::glow_style(Environment {
            profile: ThemeProfile::HighContrastDark,
            ..base
        });
        assert!(contrast.increase_contrast);
        assert_ne!(plain.fingerprint(), contrast.fingerprint());
        assert_ne!(plain.fingerprint(), high.fingerprint());
        assert_ne!(
            plain.fingerprint(),
            real_glow_style(Appearance::Light).fingerprint()
        );
        let colors = |style: &GlowStyle| [style.background, style.label, style.secondary_label];
        let resolved = |token: SemanticColor| {
            let color = crate::ui::theme_profiles::resolve_accessible_color(
                base.profile,
                base.appearance,
                token,
                true,
                false,
            );
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            [color.r, color.g, color.b].map(|channel| (channel * 255.0).round() as u8)
        };
        assert_eq!(
            colors(&contrast),
            [
                resolved(SemanticColor::SystemBackground),
                resolved(SemanticColor::Label),
                resolved(SemanticColor::SecondaryLabel),
            ],
            "Increase Contrast gives the contrast colors of the theme"
        );
    }

    /// M5 (ADR 0101 Decision 4, task 002, musicindex ADR 0021 R-ART-5 item
    /// 2): at the opacity that the glow calculates, `Label` and
    /// `SecondaryLabel` keep 4.5:1 at the top of the veil. The test uses a
    /// white glow and a black glow in each theme, with the colors of the
    /// real theme.
    #[test]
    fn adr_0101_header_glow_m5_text_keeps_contrast_in_each_theme() {
        use crate::media::header_glow::{opacity_for_color, text_contrast_at_top_of_veil};

        for (appearance, name) in [(Appearance::Dark, "Dark"), (Appearance::Light, "Light")] {
            let style = real_glow_style(appearance);
            for (glow, glow_name) in [([255.0_f32; 3], "white"), ([0.0_f32; 3], "black")] {
                let opacity = opacity_for_color(glow, &style);
                let contrast = text_contrast_at_top_of_veil(glow, opacity, &style);
                println!(
                    "M5 {name} {glow_name} glow: opacity {opacity:.3}, contrast {contrast:.2}"
                );
                assert!(
                    contrast >= 4.5,
                    "ADR 0101 Decision 4 R-ART-5 item 2: the {glow_name} glow in {name} has contrast {contrast} at opacity {opacity}. Fix the opacity calculation in media::header_glow."
                );
            }
        }
    }

    /// The opacities that the website calculates for its test covers
    /// (musicindex `84ad724`): Dark, white 0.49 (muted blend), yellow 0.27,
    /// black 0.85. Light, black 0.73 (muted blend), white 0.85. The test
    /// prints this module's values. A value must stay within 0.03 of the
    /// website value, or the difference needs a recorded cause.
    #[test]
    fn adr_0101_header_glow_opacity_agrees_with_the_website() {
        use crate::media::header_glow::{
            glow_opacity, muted_glow_layer, opacity_for_color, SMALL_SIDE,
        };
        use image::{Rgba, RgbaImage};

        let solid = |value: u8| {
            RgbaImage::from_pixel(SMALL_SIDE, SMALL_SIDE, Rgba([value, value, value, 255]))
        };
        let entity = |token: SemanticColor, appearance: Appearance| {
            let color = token.resolve(appearance);
            // A theme channel lies in 0..=1, so the rounded value fits in a byte.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            [color.r, color.g, color.b].map(|channel| (channel * 255.0).round() as u8)
        };
        let dark = real_glow_style(Appearance::Dark);
        let light = real_glow_style(Appearance::Light);
        let muted = |value: u8, style: &GlowStyle, appearance: Appearance| {
            let tint = entity(SemanticColor::EntityFeed, appearance);
            glow_opacity(&muted_glow_layer(&solid(value), tint, style), style)
        };
        let results = [
            (
                "Dark muted white",
                muted(255, &dark, Appearance::Dark),
                0.49,
            ),
            (
                "Dark yellow",
                opacity_for_color([255.0, 255.0, 0.0], &dark),
                0.27,
            ),
            ("Dark black", opacity_for_color([0.0; 3], &dark), 0.85),
            (
                "Light muted black",
                muted(0, &light, Appearance::Light),
                0.73,
            ),
            ("Light white", opacity_for_color([255.0; 3], &light), 0.85),
        ];
        for (name, value, website) in results {
            println!("opacity {name}: {value:.3} (website {website:.2})");
        }
        for (name, value, website) in results {
            assert!(
                (value - website).abs() <= 0.03,
                "{name}: opacity {value} differs from the website value {website} by more than 0.03"
            );
        }
    }

    /// R11-10: the logo frame is 735 units wide and 681 units tall.
    #[test]
    fn adr_0084_logo_frame_r11_10_aspect() {
        let aspect: f32 = 735.0 / 681.0;
        assert!((ArtworkShape::LogoFrame.aspect() - aspect).abs() < f32::EPSILON);
    }

    /// ADR 0083 task 005: the backdrop keeps the hue of the cover.
    #[test]
    fn adr_0083_cover_backdrop_keeps_the_cover_hue() {
        let red = CoverColor {
            red: 220,
            green: 40,
            blue: 40,
        };
        for appearance in [Appearance::Dark, Appearance::Light] {
            let toned = CoverBackdrop::tone(red, appearance);
            assert!(
                toned.r > toned.g && toned.r > toned.b,
                "a red cover must give a red backdrop in {appearance:?}: {toned:?}"
            );
        }
    }

    #[test]
    fn adr_0083_artwork_shadow_grows_with_image_size() {
        assert!(
            ArtworkShadow::Xl.blur_radius_base() > ArtworkShadow::Lg.blur_radius_base(),
            "Xl blur radius must exceed Lg"
        );
        assert!(
            ArtworkShadow::Xl.offset_y_base() > ArtworkShadow::Lg.offset_y_base(),
            "Xl vertical offset must exceed Lg"
        );
        assert!(
            ArtworkShadow::XXl.blur_radius_base() > ArtworkShadow::Xl.blur_radius_base(),
            "XXl blur radius must exceed Xl"
        );
        assert!(
            ArtworkShadow::XXl.offset_y_base() > ArtworkShadow::Xl.offset_y_base(),
            "XXl vertical offset must exceed Xl"
        );
    }

    /// ADR 0101 Decision 4: the artwork shadow is a translucent black. No
    /// call site picks a raw literal color.
    #[gpui::test]
    fn adr_0083_artwork_shadow_is_translucent_black(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            for role in [ArtworkShadow::Lg, ArtworkShadow::Xl, ArtworkShadow::XXl] {
                let shadow = role.shadow(cx);
                assert_eq!(
                    shadow.len(),
                    1,
                    "{role:?} must draw exactly one shadow layer"
                );
                let layer = &shadow[0];
                assert!(!layer.inset, "{role:?} must not be an inset shadow");
                assert!(
                    layer.color.a > 0.0 && layer.color.a < 1.0,
                    "{role:?} shadow must be translucent"
                );
            }
        });
    }

    // -------------------------------------------------------------------
    // ADR 0039 task 001 (amended scope): CHROME/TYPE domain split.
    // -------------------------------------------------------------------

    const ADR_0039_STEPS: [(ScaleFactor, f32); 5] = [
        (ScaleFactor::XSmall, 0.85),
        (ScaleFactor::Small, 0.92),
        (ScaleFactor::Medium, 1.0),
        (ScaleFactor::Large, 1.12),
        (ScaleFactor::XLarge, 1.25),
    ];

    /// M1: the CHROME coefficients are bit-identical to the five former
    /// uniform `ScaleFactor` values — not merely close by an epsilon.
    #[test]
    fn adr_0039_chrome_multiplier_matches_former_uniform_values_bit_exact() {
        for (scale, former_value) in ADR_0039_STEPS {
            assert_eq!(
                scale.chrome_multiplier().to_bits(),
                former_value.to_bits(),
                "{scale:?} CHROME coefficient drifted from the former uniform value"
            );
        }
    }

    /// M1: every resolved `Spacing`/`Radius`/`Size`/`SkeletonBlock` value at
    /// every step matches the old uniform arithmetic bit-for-bit, including
    /// the `Radius::Full` pill exception. Exercises the environment-backed
    /// `.scaled(cx)` path, not only the pure coefficient table.
    #[gpui::test]
    fn adr_0039_chrome_resolved_tokens_match_former_arithmetic_at_every_step(
        cx: &mut gpui::TestAppContext,
    ) {
        let spacing = [
            Spacing::XXS,
            Spacing::XS,
            Spacing::SM,
            Spacing::MD,
            Spacing::LG,
            Spacing::XL,
            Spacing::XXL,
        ];
        let radius = [Radius::SM, Radius::MD, Radius::LG, Radius::XL, Radius::Full];
        let size = [
            Size::MinHitTarget,
            Size::ButtonSm,
            Size::ButtonMd,
            Size::ButtonLg,
            Size::MenuCompact,
            Size::MenuRegular,
            Size::MenuWide,
            Size::ColumnShort,
            Size::ColumnRegular,
            Size::ColumnTall,
            Size::RowMd,
            Size::RowLg,
            Size::ContentTileWidth,
            Size::ContentTileArtwork,
            Size::NoticeWidth,
        ];
        let skeleton = [
            SkeletonBlock::InspectorTitle,
            SkeletonBlock::InspectorSubtitle,
            SkeletonBlock::InspectorCaption,
            SkeletonBlock::FeedTileSubtitle,
            SkeletonBlock::TrackNumber,
            SkeletonBlock::TrackDuration,
        ];

        cx.update(|cx| {
            for (scale, former_multiplier) in ADR_0039_STEPS {
                cx.set_global(scale);

                for token in spacing {
                    let expected = f32::from(token.px()) * former_multiplier;
                    assert_eq!(
                        f32::from(token.scaled(cx)).to_bits(),
                        expected.to_bits(),
                        "Spacing::{token:?} at {scale:?} drifted from the former arithmetic"
                    );
                }

                for token in radius {
                    let expected = if matches!(token, Radius::Full) {
                        f32::from(token.px())
                    } else {
                        f32::from(token.px()) * former_multiplier
                    };
                    assert_eq!(
                        f32::from(token.scaled(cx)).to_bits(),
                        expected.to_bits(),
                        "Radius::{token:?} at {scale:?} drifted from the former arithmetic"
                    );
                }

                for token in size {
                    let expected = f32::from(token.px()) * former_multiplier;
                    assert_eq!(
                        f32::from(token.scaled(cx)).to_bits(),
                        expected.to_bits(),
                        "Size::{token:?} at {scale:?} drifted from the former arithmetic"
                    );
                }

                for token in skeleton {
                    let (base_w, base_h) = token.px();
                    let expected = (
                        f32::from(base_w) * former_multiplier,
                        f32::from(base_h) * former_multiplier,
                    );
                    let (resolved_w, resolved_h) = token.scaled(cx);
                    assert_eq!(
                        f32::from(resolved_w).to_bits(),
                        expected.0.to_bits(),
                        "SkeletonBlock::{token:?} width at {scale:?} drifted from the former arithmetic"
                    );
                    assert_eq!(
                        f32::from(resolved_h).to_bits(),
                        expected.1.to_bits(),
                        "SkeletonBlock::{token:?} height at {scale:?} drifted from the former arithmetic"
                    );
                }
            }
        });
    }

    /// ADR 0039 task 003 compares all 35 type outcomes with the ratified table.
    /// ADR 0083 task 002 adds `Display`'s 5 outcomes, for 40 in total.
    /// The comparison tolerance is 0.001 px.
    /// Expected values come from the table, independently of the resolver's formula.
    /// ADR 0101 Decision 3 changes the base of `Title2` (22), `Title` (26)
    /// and `Display` (42). Their rows are the new base times the same
    /// ratified factors.
    /// This test replaces task 001's assertion that every outcome matches uniform scaling.
    /// `adr_0039_non_medium_type_outcomes_differ_from_uniform_except_title_downward`
    /// retains the comparison with uniform scaling for the original seven roles.
    /// `Display` has no former-uniform baseline. ADR 0083 task 002 added
    /// it after that migration. This test proves `Display` here.
    #[test]
    fn adr_0039_type_outcomes_match_ratified_values() {
        // (role, [XSmall, Small, Medium, Large, XLarge] expected px).
        let cases: [(FontSize, [f32; 5]); 8] = [
            (FontSize::Micro, [10.01, 10.472, 11.00, 12.9008, 14.96]),
            (FontSize::Caption, [10.80, 11.36, 12.00, 13.8432, 15.84]),
            (FontSize::Body, [11.57, 12.237_333, 13.00, 14.7472, 16.64]),
            (FontSize::Headline, [13.20, 14.04, 15.00, 16.728, 18.60]),
            (FontSize::Title3, [14.79, 15.821_333, 17.00, 18.632, 20.40]),
            (FontSize::Title2, [18.92, 20.357_334, 22.00, 23.6896, 25.52]),
            (FontSize::Title, [22.10, 23.92, 26.00, 27.4976, 29.12]),
            (FontSize::Display, [35.28, 38.416, 42.00, 43.6128, 45.36]),
        ];

        let mut checked = 0;
        for (role, expected_by_step) in cases {
            for (i, (scale, _)) in ADR_0039_STEPS.iter().enumerate() {
                let resolved = f32::from(role.scaled_px(*scale));
                let expected = expected_by_step[i];
                assert!(
                    (resolved - expected).abs() < 0.001,
                    "{role:?} at {scale:?}: resolved {resolved} != ratified {expected}"
                );
                checked += 1;
            }
        }
        assert_eq!(
            checked, 40,
            "expected all 40 ratified role/step outcomes to be checked"
        );
    }

    /// ADR 0039 task 003 M1 compares the 28 non-medium outcomes with uniform scaling.
    /// Exactly 26 outcomes differ.
    /// `Title` at `XSmall` and `Small` retains bit-identical output.
    ///
    /// The operator retained `Title`'s x-small factor of 0.85.
    /// Interpolation therefore also retains its small factor of 0.92.
    /// Do not change these endpoints to remove the two accepted exceptions.
    ///
    /// This test excludes medium.
    /// `adr_0039_medium_type_returns_each_role_base_exactly` checks its seven unchanged outcomes.
    #[test]
    fn adr_0039_non_medium_type_outcomes_differ_from_uniform_except_title_downward() {
        let roles = [
            FontSize::Micro,
            FontSize::Caption,
            FontSize::Body,
            FontSize::Headline,
            FontSize::Title3,
            FontSize::Title2,
            FontSize::Title,
        ];
        let non_medium_steps: Vec<(ScaleFactor, f32)> = ADR_0039_STEPS
            .into_iter()
            .filter(|(scale, _)| !matches!(scale, ScaleFactor::Medium))
            .collect();
        assert_eq!(non_medium_steps.len(), 4, "expected 4 non-medium steps");

        let mut differing = 0;
        let mut equal = 0;
        for role in roles {
            for (scale, former_multiplier) in non_medium_steps.iter().copied() {
                let former = f32::from(role.px()) * former_multiplier;
                let resolved = f32::from(role.scaled_px(scale));
                let is_title_downward = matches!(role, FontSize::Title)
                    && matches!(scale, ScaleFactor::XSmall | ScaleFactor::Small);

                if is_title_downward {
                    assert_eq!(
                        resolved.to_bits(),
                        former.to_bits(),
                        "documented exception: {role:?} at {scale:?} must stay bit-identical \
                         to the former uniform result"
                    );
                    equal += 1;
                } else {
                    assert_ne!(
                        resolved.to_bits(),
                        former.to_bits(),
                        "{role:?} at {scale:?} unexpectedly matches the former uniform result"
                    );
                    differing += 1;
                }
            }
        }

        assert_eq!(
            equal, 2,
            "expected exactly 2 documented Title-downward exceptions"
        );
        assert_eq!(
            differing, 26,
            "expected the other 26 non-medium cells to differ from uniform"
        );
    }

    /// ADR 0039 task 003 M1: above medium, smaller roles grow more
    /// proportionally than larger roles, at both `Large` and `XLarge`.
    /// ADR 0083 task 002 appends `Display`, the role above `Title`.
    #[test]
    fn adr_0039_smaller_roles_grow_more_above_medium() {
        let roles_smallest_to_largest = [
            FontSize::Micro,
            FontSize::Caption,
            FontSize::Body,
            FontSize::Headline,
            FontSize::Title3,
            FontSize::Title2,
            FontSize::Title,
            FontSize::Display,
        ];
        for scale in [ScaleFactor::Large, ScaleFactor::XLarge] {
            for pair in roles_smallest_to_largest.windows(2) {
                let smaller_growth = pair[0].type_multiplier(scale) - 1.0;
                let larger_growth = pair[1].type_multiplier(scale) - 1.0;
                assert!(
                    smaller_growth > larger_growth,
                    "{:?} should grow more than {:?} at {scale:?} \
                     ({smaller_growth} <= {larger_growth})",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    /// ADR 0039 task 003 M1: below medium, smaller roles shrink less
    /// proportionally than larger roles, at both `XSmall` and `Small`.
    /// ADR 0083 task 002 appends `Display`, the role above `Title`.
    #[test]
    fn adr_0039_smaller_roles_shrink_less_below_medium() {
        let roles_smallest_to_largest = [
            FontSize::Micro,
            FontSize::Caption,
            FontSize::Body,
            FontSize::Headline,
            FontSize::Title3,
            FontSize::Title2,
            FontSize::Title,
            FontSize::Display,
        ];
        for scale in [ScaleFactor::XSmall, ScaleFactor::Small] {
            for pair in roles_smallest_to_largest.windows(2) {
                let smaller_loss = 1.0 - pair[0].type_multiplier(scale);
                let larger_loss = 1.0 - pair[1].type_multiplier(scale);
                assert!(
                    smaller_loss < larger_loss,
                    "{:?} should shrink less than {:?} at {scale:?} \
                     ({smaller_loss} >= {larger_loss})",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    /// M2: medium returns each role's exact base size (11, 12, 13, 15, 17,
    /// 22, 26, 42), unchanged by the domain split. ADR 0101 Decision 3
    /// gives the bases of `Title2`, `Title` and `Display`.
    #[test]
    fn adr_0039_medium_type_returns_each_role_base_exactly() {
        let expected = [
            (FontSize::Micro, 11.0),
            (FontSize::Caption, 12.0),
            (FontSize::Body, 13.0),
            (FontSize::Headline, 15.0),
            (FontSize::Title3, 17.0),
            (FontSize::Title2, 22.0),
            (FontSize::Title, 26.0),
            (FontSize::Display, 42.0),
        ];
        for (role, base) in expected {
            assert_eq!(role.scaled_px(ScaleFactor::Medium), px(base));
        }
    }

    /// M2: role ordering `Micro < Caption < Body < Headline < Title3 <
    /// Title2 < Title < Display` holds at every step. ADR 0083 task 002
    /// appends `Display`.
    #[test]
    fn adr_0039_type_role_ordering_holds_at_every_step() {
        let roles = [
            FontSize::Micro,
            FontSize::Caption,
            FontSize::Body,
            FontSize::Headline,
            FontSize::Title3,
            FontSize::Title2,
            FontSize::Title,
            FontSize::Display,
        ];
        for (scale, _) in ADR_0039_STEPS {
            for pair in roles.windows(2) {
                assert!(
                    pair[0].scaled_px(scale) < pair[1].scaled_px(scale),
                    "{:?} < {:?} failed at {scale:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    /// M2: each role grows monotonically across the five steps. ADR 0083
    /// task 002 appends `Display`.
    #[test]
    fn adr_0039_type_grows_monotonically_per_role_across_steps() {
        let roles = [
            FontSize::Micro,
            FontSize::Caption,
            FontSize::Body,
            FontSize::Headline,
            FontSize::Title3,
            FontSize::Title2,
            FontSize::Title,
            FontSize::Display,
        ];
        for role in roles {
            let resolved: Vec<Pixels> = ADR_0039_STEPS
                .iter()
                .map(|&(scale, _)| role.scaled_px(scale))
                .collect();
            for pair in resolved.windows(2) {
                assert!(
                    pair[0] < pair[1],
                    "{role:?} did not grow monotonically across steps"
                );
            }
        }
    }
}
