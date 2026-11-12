//! Profile-specific semantic color resolution.
//!
//! [`crate::theme_profile::ThemeProfile`] stays GPUI-free at the crate root.
//! This module is the UI-layer resolver that maps a persisted profile onto
//! concrete token colors.

#![warn(clippy::pedantic)]

use gpui::Rgba;

use crate::theme_profile::ThemeProfile;
use crate::ui::tokens::{Appearance, SemanticColor};

/// Resolve a semantic color through a named visual profile.
#[must_use]
pub fn resolve_profile_color(profile: ThemeProfile, token: SemanticColor) -> Rgba {
    resolve_concrete_profile_color(profile, token)
}

/// Resolve a semantic color through a named visual profile, using
/// `system_appearance` when the profile follows the platform.
#[must_use]
pub fn resolve_profile_color_for_appearance(
    profile: ThemeProfile,
    system_appearance: Appearance,
    token: SemanticColor,
) -> Rgba {
    resolve_concrete_profile_color(effective_profile(profile, system_appearance), token)
}

/// Resolve `System` to the concrete light/dark profile currently in effect.
#[must_use]
pub const fn effective_profile(
    profile: ThemeProfile,
    system_appearance: Appearance,
) -> ThemeProfile {
    match profile {
        ThemeProfile::System => match system_appearance {
            Appearance::Light => ThemeProfile::Light,
            Appearance::Dark => ThemeProfile::Dark,
        },
        ThemeProfile::Dark
        | ThemeProfile::Light
        | ThemeProfile::HighContrastDark
        | ThemeProfile::HighContrastLight => profile,
    }
}

/// Resolve a profile to its base light/dark appearance.
#[must_use]
pub const fn appearance_for_profile(
    profile: ThemeProfile,
    system_appearance: Appearance,
) -> Appearance {
    appearance_for_concrete_profile(effective_profile(profile, system_appearance))
}

/// Resolve a concrete profile to its base light/dark appearance.
#[must_use]
pub const fn appearance_for_concrete_profile(profile: ThemeProfile) -> Appearance {
    match profile {
        ThemeProfile::System | ThemeProfile::Dark | ThemeProfile::HighContrastDark => {
            Appearance::Dark
        }
        ThemeProfile::Light | ThemeProfile::HighContrastLight => Appearance::Light,
    }
}

fn resolve_concrete_profile_color(profile: ThemeProfile, token: SemanticColor) -> Rgba {
    match profile {
        ThemeProfile::System | ThemeProfile::Dark => token.resolve(Appearance::Dark),
        ThemeProfile::Light => token.resolve(Appearance::Light),
        ThemeProfile::HighContrastDark => high_contrast_dark(token),
        ThemeProfile::HighContrastLight => high_contrast_light(token),
    }
}

#[expect(
    clippy::match_same_arms,
    reason = "different roles share accessible colors"
)]
fn high_contrast_dark(token: SemanticColor) -> Rgba {
    match token {
        SemanticColor::SystemBackground => hex(0x00_0000),
        SemanticColor::SidebarBackground => hex(0x05_0505),
        SemanticColor::SecondarySystemBackground => hex(0x08_0808),
        SemanticColor::TertiarySystemBackground => hex(0x16_1616),

        SemanticColor::Label => hex(0xff_ffff),
        SemanticColor::SecondaryLabel => hex(0xf2_f2f2),
        SemanticColor::TertiaryLabel => hex(0xd8_d8d8),
        SemanticColor::QuaternaryLabel => hex(0xb8_b8b8),

        SemanticColor::SystemFill => hex(0x2a_2a2a),
        SemanticColor::SecondaryFill => hex(0x1f_1f1f),
        SemanticColor::TertiaryFill => hex(0x16_1616),

        SemanticColor::SelectedContent => hex(0x00_3352),

        SemanticColor::Separator => hex(0x7a_7a7a),
        SemanticColor::OpaqueSeparator => hex(0xd0_d0d0),
        SemanticColor::CheckboxBorder => hex(0xd0_d0d0),

        SemanticColor::Accent => hex(0x66_d9ff),
        SemanticColor::AccentHover => hex(0x9a_e8ff),
        SemanticColor::AccentPressed => hex(0x38_caff),
        SemanticColor::OnAccent => hex(0x00_0000),
        SemanticColor::Focus => hex(0xff_f06a),
        SemanticColor::AccentTint => tint(0x66_d9ff, 0.18),
        // No sidebar glow under Increase Contrast (musicindex ADR 0056
        // R-SIDE-1 item 4).
        SemanticColor::SidebarGlowTop => tint(0x66_d9ff, 0.0),
        SemanticColor::AccentOutline => tint(0x66_d9ff, 0.34),
        SemanticColor::DangerTint => tint(0xff_6b6b, 0.16),
        SemanticColor::ButtonFill => tint(0x76_7680, 0.24),
        SemanticColor::ButtonFillHover => tint(0xff_ffff, 0.12),

        SemanticColor::Success => hex(0x00_e676),
        SemanticColor::Warning => hex(0xff_d60a),
        SemanticColor::Danger => hex(0xff_6b6b),
        SemanticColor::Info => hex(0x66_d9ff),

        SemanticColor::SuccessLabel => hex(0x00_e676),
        SemanticColor::WarningLabel => hex(0xff_d60a),
        SemanticColor::DangerLabel => hex(0xff_6b6b),
        SemanticColor::InfoLabel => hex(0x66_d9ff),

        SemanticColor::OnSuccess => hex(0x00_0000),
        SemanticColor::OnWarning => hex(0x00_0000),
        SemanticColor::OnDanger => hex(0x00_0000),
        SemanticColor::OnInfo => hex(0x00_0000),

        SemanticColor::DiffMatch => hex(0x00_e676),
        SemanticColor::DiffDifferent => hex(0xff_d60a),
        SemanticColor::DiffMissing => hex(0xff_8a65),

        SemanticColor::Id3FrameV22 => hex(0xd0_a8ff),
        SemanticColor::Id3FrameV23Only => hex(0xff_e066),
        SemanticColor::Id3FrameV24Only => hex(0x6f_e8e8),
        SemanticColor::Id3FrameUnknown => hex(0xff_a07f),

        // ADR 0083 Decision 1: brighter, more saturated entity colors so
        // each dot clears the contrast matrix against every high-contrast
        // dark background.
        SemanticColor::EntityFeed => hex(0xe0_a868),
        SemanticColor::EntityTrack => hex(0x00_c2d4),
        SemanticColor::EntityPlaylist => hex(0x6e_f0a0),
        SemanticColor::EntityArtist => hex(0xff_9fe0),
        // The other kinds of the shared entity palette (musicindex ADR 0032).
        SemanticColor::EntityLive => hex(0xff_7a70),
        SemanticColor::EntityHost => hex(0xff_c4e4),
        SemanticColor::EntityAuthor => hex(0xf6_c8f4),
        SemanticColor::EntityLabel => hex(0xb3_94ff),
        SemanticColor::EntityPublisher => hex(0xa6_8cff),
        SemanticColor::EntityProducer => hex(0xc4_8aff),
        SemanticColor::EntityNetwork => hex(0xd0_8cff),
        SemanticColor::EntityHosting => hex(0x9d_93ff),
        SemanticColor::EntitySponsor => hex(0xdc_7cff),
    }
}

#[expect(
    clippy::match_same_arms,
    reason = "different roles share accessible colors"
)]
fn high_contrast_light(token: SemanticColor) -> Rgba {
    match token {
        SemanticColor::SystemBackground => hex(0xff_ffff),
        SemanticColor::SidebarBackground => hex(0xfb_fbfb),
        SemanticColor::SecondarySystemBackground => hex(0xf7_f7f7),
        SemanticColor::TertiarySystemBackground => hex(0xee_eeee),

        SemanticColor::Label => hex(0x00_0000),
        SemanticColor::SecondaryLabel => hex(0x1c_1c1e),
        SemanticColor::TertiaryLabel => hex(0x3a_3a3c),
        SemanticColor::QuaternaryLabel => hex(0x5f_5f63),

        SemanticColor::SystemFill => hex(0xe0_e0e0),
        SemanticColor::SecondaryFill => hex(0xea_eaea),
        SemanticColor::TertiaryFill => hex(0xf2_f2f2),

        SemanticColor::SelectedContent => hex(0xbf_d3ff),

        SemanticColor::Separator => hex(0x76_7676),
        SemanticColor::OpaqueSeparator => hex(0x3a_3a3c),
        SemanticColor::CheckboxBorder => hex(0x3a_3a3c),

        SemanticColor::Accent => hex(0x00_40dd),
        SemanticColor::AccentHover => hex(0x00_53ff),
        SemanticColor::AccentPressed => hex(0x00_32a8),
        SemanticColor::OnAccent => hex(0xff_ffff),
        SemanticColor::Focus => hex(0x00_40dd),
        // No sidebar glow under Increase Contrast (musicindex ADR 0056
        // R-SIDE-1 item 4).
        SemanticColor::SidebarGlowTop => tint(0x00_40dd, 0.0),
        SemanticColor::AccentTint => tint(0x00_40dd, 0.12),
        SemanticColor::AccentOutline => tint(0x00_40dd, 0.40),
        SemanticColor::DangerTint => tint(0xb0_0020, 0.10),
        SemanticColor::ButtonFill => tint(0x78_7880, 0.14),
        SemanticColor::ButtonFillHover => tint(0x78_7880, 0.18),

        SemanticColor::Success => hex(0x00_6d2c),
        SemanticColor::Warning => hex(0x7a_4b00),
        SemanticColor::Danger => hex(0xb0_0020),
        SemanticColor::Info => hex(0x00_40dd),

        SemanticColor::SuccessLabel => hex(0x00_6d2c),
        SemanticColor::WarningLabel => hex(0x7a_4b00),
        SemanticColor::DangerLabel => hex(0xb0_0020),
        SemanticColor::InfoLabel => hex(0x00_40dd),

        SemanticColor::OnSuccess => hex(0xff_ffff),
        SemanticColor::OnWarning => hex(0xff_ffff),
        SemanticColor::OnDanger => hex(0xff_ffff),
        SemanticColor::OnInfo => hex(0xff_ffff),

        SemanticColor::DiffMatch => hex(0x00_6d2c),
        SemanticColor::DiffDifferent => hex(0x7a_4b00),
        SemanticColor::DiffMissing => hex(0xb0_0020),

        SemanticColor::Id3FrameV22 => hex(0x3e_1f7a),
        SemanticColor::Id3FrameV23Only => hex(0x5a_3a00),
        SemanticColor::Id3FrameV24Only => hex(0x00_4d54),
        SemanticColor::Id3FrameUnknown => hex(0x7a_2916),

        // ADR 0083 Decision 1: darker, more saturated entity colors so each
        // dot clears the contrast matrix against every high-contrast light
        // background.
        SemanticColor::EntityFeed => hex(0x6b_4717),
        SemanticColor::EntityTrack => hex(0x00_6b76),
        SemanticColor::EntityPlaylist => hex(0x00_4d2c),
        SemanticColor::EntityArtist => hex(0x5c_2c4f),
        // The other kinds of the shared entity palette (musicindex ADR 0032).
        SemanticColor::EntityLive => hex(0x9a_0010),
        SemanticColor::EntityHost => hex(0x5e_2a4a),
        SemanticColor::EntityAuthor => hex(0x52_2a55),
        SemanticColor::EntityLabel => hex(0x4f_2c8f),
        SemanticColor::EntityPublisher => hex(0x4a_30a0),
        SemanticColor::EntityProducer => hex(0x63_2a8a),
        SemanticColor::EntityNetwork => hex(0x66_2e8f),
        SemanticColor::EntityHosting => hex(0x46_3ea6),
        SemanticColor::EntitySponsor => hex(0x6e_2a7e),
    }
}

#[inline]
const fn tint(rgb: u32, alpha: f32) -> Rgba {
    let mut color = hex(rgb);
    color.a = alpha;
    color
}

const fn hex(rgb: u32) -> Rgba {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// musicindex ADR 0056 R-SIDE-1 item 4: the light and dark profiles have
    /// the sidebar glow, and the high-contrast profiles have none.
    #[test]
    fn adr_0056_high_contrast_profiles_have_no_glow() {
        for profile in [
            ThemeProfile::HighContrastDark,
            ThemeProfile::HighContrastLight,
        ] {
            let glow = resolve_profile_color(profile, SemanticColor::SidebarGlowTop);
            assert!(glow.a.abs() < f32::EPSILON, "{profile:?}");
        }
        for profile in [ThemeProfile::Dark, ThemeProfile::Light] {
            let glow = resolve_profile_color(profile, SemanticColor::SidebarGlowTop);
            assert!(glow.a > 0.0, "{profile:?}");
        }
    }

    #[test]
    fn system_profile_follows_supplied_appearance() {
        assert_eq!(
            effective_profile(ThemeProfile::System, Appearance::Light),
            ThemeProfile::Light
        );
        assert_eq!(
            effective_profile(ThemeProfile::System, Appearance::Dark),
            ThemeProfile::Dark
        );
        assert_eq!(
            appearance_for_profile(ThemeProfile::System, Appearance::Light),
            Appearance::Light
        );
    }

    #[test]
    fn explicit_profiles_ignore_system_appearance() {
        assert_eq!(
            effective_profile(ThemeProfile::HighContrastDark, Appearance::Light),
            ThemeProfile::HighContrastDark
        );
        assert_eq!(
            appearance_for_profile(ThemeProfile::HighContrastDark, Appearance::Light),
            Appearance::Dark
        );
    }

    #[test]
    fn system_color_resolution_uses_supplied_appearance() {
        assert_eq!(
            resolve_profile_color_for_appearance(
                ThemeProfile::System,
                Appearance::Light,
                SemanticColor::SystemBackground,
            ),
            SemanticColor::SystemBackground.resolve(Appearance::Light)
        );
        assert_eq!(
            resolve_profile_color_for_appearance(
                ThemeProfile::System,
                Appearance::Dark,
                SemanticColor::SystemBackground,
            ),
            SemanticColor::SystemBackground.resolve(Appearance::Dark)
        );
    }
}
