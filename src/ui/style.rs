//! Visual roles used while screens finish migrating.
//!
//! This module replaces the old `ui::theme` compatibility shim. Every color
//! role in `color::*` resolves through the profile installed by `theme_bridge`
//! by way of [`crate::ui::tokens::SemanticColor`] — there are no raw rgb hex
//! literals in this file, and the architecture test
//! `ui_style_resolves_colors_through_token_layer` enforces that. Runtime
//! theme changes therefore repaint correctly without dark-only helper calls.
//!
//! The spacing, radius, and typography constants are fixed geometry kept for
//! legacy screen parity until those call sites move fully onto token enums
//! (`tokens::Spacing`, `tokens::Radius`, `tokens::FontSize`).

#![warn(clippy::pedantic)]

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use gpui::{px, FontWeight, Pixels, Rgba, Styled};

use crate::theme_profile::ThemeProfile;
use crate::ui::theme_profiles::resolve_accessible_color;
use crate::ui::tokens::{Appearance, SemanticColor};

static CURRENT_PROFILE: AtomicU8 = AtomicU8::new(profile_to_raw(ThemeProfile::Dark));

static INCREASE_CONTRAST: AtomicBool = AtomicBool::new(false);
static REDUCE_TRANSPARENCY: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_profile(
    profile: ThemeProfile,
    increase_contrast: bool,
    reduce_transparency: bool,
) {
    CURRENT_PROFILE.store(profile_to_raw(profile), Ordering::Relaxed);
    INCREASE_CONTRAST.store(increase_contrast, Ordering::Relaxed);
    REDUCE_TRANSPARENCY.store(reduce_transparency, Ordering::Relaxed);
}

const fn profile_to_raw(profile: ThemeProfile) -> u8 {
    match profile {
        ThemeProfile::System => 0,
        ThemeProfile::Dark => 1,
        ThemeProfile::Light => 2,
        ThemeProfile::HighContrastDark => 3,
        ThemeProfile::HighContrastLight => 4,
    }
}

fn current_profile() -> ThemeProfile {
    match CURRENT_PROFILE.load(Ordering::Relaxed) {
        0 => ThemeProfile::System,
        2 => ThemeProfile::Light,
        3 => ThemeProfile::HighContrastDark,
        4 => ThemeProfile::HighContrastLight,
        _ => ThemeProfile::Dark,
    }
}

fn role(token: SemanticColor) -> Rgba {
    // The installed profile is concrete, so the appearance has no effect.
    resolve_accessible_color(
        current_profile(),
        Appearance::Dark,
        token,
        INCREASE_CONTRAST.load(Ordering::Relaxed),
        REDUCE_TRANSPARENCY.load(Ordering::Relaxed),
    )
}

pub mod color {
    #![expect(
        clippy::must_use_candidate,
        reason = "compatibility color roles are consumed directly by GPUI builder chains"
    )]

    use super::{role, Rgba, SemanticColor};

    pub fn bg_canvas() -> Rgba {
        role(SemanticColor::SystemBackground)
    }
    /// ADR 0101 Decision 1: the fill of a group or a card.
    pub fn bg_group() -> Rgba {
        role(SemanticColor::GroupBackground)
    }
    /// ADR 0101 Decision 1: the fill of an open entry in a group.
    pub fn bg_open_entry() -> Rgba {
        role(SemanticColor::OpenEntryBackground)
    }
    /// ADR 0101 Decision 1: the fill of a layer above the content.
    pub fn bg_floating() -> Rgba {
        role(SemanticColor::FloatingBackground)
    }
    /// The fill of a hovered sidebar or playlist row. It differs from the
    /// page in each theme. In Light it is the group fill.
    pub fn bg_hover() -> Rgba {
        role(SemanticColor::HoverBackground)
    }
    /// ADR 0101 Decision 1: the sidebar surface, distinct from the canvas
    /// and the content surfaces.
    pub fn bg_sidebar() -> Rgba {
        role(SemanticColor::SidebarBackground)
    }
    /// musicindex ADR 0056 R-SIDE-1: the color at the top edge of the sidebar glow.
    pub fn sidebar_glow_top() -> Rgba {
        role(SemanticColor::SidebarGlowTop)
    }
    pub fn bg_selected() -> Rgba {
        role(SemanticColor::SelectedContent)
    }

    pub fn border_subtle() -> Rgba {
        role(SemanticColor::Separator)
    }
    pub fn border_strong() -> Rgba {
        role(SemanticColor::OpaqueSeparator)
    }

    pub fn text_primary() -> Rgba {
        role(SemanticColor::Label)
    }
    pub fn text_secondary() -> Rgba {
        role(SemanticColor::SecondaryLabel)
    }
    pub fn text_muted() -> Rgba {
        role(SemanticColor::TertiaryLabel)
    }
    pub fn text_on_accent() -> Rgba {
        role(SemanticColor::OnAccent)
    }

    pub fn accent() -> Rgba {
        role(SemanticColor::Accent)
    }
    /// musicindex ADR 0082 R-ROUTES-7: the share color of the value routes.
    pub fn share() -> Rgba {
        role(SemanticColor::Share)
    }
    /// musicindex ADR 0082 R-ROUTES-7: the share track color of the value routes.
    pub fn share_track() -> Rgba {
        role(SemanticColor::ShareTrack)
    }
    pub fn accent_hover() -> Rgba {
        role(SemanticColor::AccentHover)
    }
    pub fn accent_pressed() -> Rgba {
        role(SemanticColor::AccentPressed)
    }

    pub fn focus_ring() -> Rgba {
        role(SemanticColor::Focus)
    }

    pub fn id3_frame_v22() -> Rgba {
        role(SemanticColor::Id3FrameV22)
    }
    pub fn id3_frame_v23_only() -> Rgba {
        role(SemanticColor::Id3FrameV23Only)
    }
    pub fn id3_frame_v24_only() -> Rgba {
        role(SemanticColor::Id3FrameV24Only)
    }
    pub fn id3_frame_unknown() -> Rgba {
        role(SemanticColor::Id3FrameUnknown)
    }
}

pub mod spacing {
    use super::{px, Pixels};
    pub const NONE: Pixels = px(0.0);
    pub const XXS: Pixels = px(2.0);
    pub const XS: Pixels = px(4.0);
    pub const SM: Pixels = px(8.0);
    pub const MD: Pixels = px(12.0);
    pub const LG: Pixels = px(16.0);
    pub const XL: Pixels = px(24.0);
    pub const XXL: Pixels = px(32.0);
}

pub mod typography {
    use super::{px, FontWeight, Pixels, Styled};

    pub const LINE_TIGHT: Pixels = px(14.0);
    pub const LINE_COMPACT: Pixels = px(15.0);
    pub const LINE_BODY: Pixels = px(16.0);
    pub const LINE_DETAIL: Pixels = px(17.0);
    pub const LINE_TITLE: Pixels = px(20.0);
    pub const LINE_HEADER: Pixels = px(23.0);
    pub const SIZE_TITLE: Pixels = px(20.0);
    pub const SIZE_HEADLINE: Pixels = px(15.0);
    pub const SIZE_BODY: Pixels = px(13.0);
    pub const SIZE_CAPTION: Pixels = px(12.0);
    pub const SIZE_MICRO: Pixels = px(11.0);
    pub const SIZE_ARTWORK_FALLBACK: Pixels = px(28.0);

    pub fn type_title<T: Styled>(el: T) -> T {
        el.text_size(SIZE_TITLE).font_weight(FontWeight::SEMIBOLD)
    }

    pub fn type_headline<T: Styled>(el: T) -> T {
        el.text_size(SIZE_HEADLINE)
            .font_weight(FontWeight::SEMIBOLD)
    }

    pub fn type_body<T: Styled>(el: T) -> T {
        el.text_size(SIZE_BODY).font_weight(FontWeight::NORMAL)
    }

    pub fn type_caption<T: Styled>(el: T) -> T {
        el.text_size(SIZE_CAPTION).font_weight(FontWeight::NORMAL)
    }

    pub fn type_caption_strong<T: Styled>(el: T) -> T {
        el.text_size(SIZE_CAPTION).font_weight(FontWeight::MEDIUM)
    }

    pub fn type_micro<T: Styled>(el: T) -> T {
        el.text_size(SIZE_MICRO).font_weight(FontWeight::MEDIUM)
    }
}
