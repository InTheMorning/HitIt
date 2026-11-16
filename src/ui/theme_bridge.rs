//! Bridge between the app's design tokens and `gpui-component`'s theme.
//!
//! `gpui-component` ships its own light/dark palettes. Without intervention
//! every component (`Button`, `Popover`, `Input`, `Sidebar`, …) renders with
//! those defaults — the root cause of every contrast bug we have shipped on
//! our dark canvas.
//!
//! [`install_theme`] is called once during application startup right after
//! [`gpui_component::init`]. It:
//!
//! 1. Calls `Theme::change(ThemeMode, …)` so the global is initialised.
//! 2. Overwrites every relevant field of `ThemeColor` with the value our
//!    [`SemanticColor`] palette would resolve to.
//!
//! After this call every gpui-component widget renders with our colors
//! automatically — no per-component overrides required.

#![warn(clippy::pedantic)]

use gpui::{App, Hsla, Rgba, Window};
use gpui_component::{Theme, ThemeMode};

use crate::config::UiScale;
use crate::theme_profile::ThemeProfile;
use crate::ui::system_settings::AccessibilitySettings;
use crate::ui::theme_profiles::{
    appearance_for_profile, effective_profile, resolve_accessible_color,
};
use crate::ui::tokens::{Appearance, ScaleFactor, SemanticColor};

impl From<UiScale> for ScaleFactor {
    fn from(s: UiScale) -> Self {
        match s {
            UiScale::XSmall => Self::XSmall,
            UiScale::Small => Self::Small,
            UiScale::Medium => Self::Medium,
            UiScale::Large => Self::Large,
            UiScale::XLarge => Self::XLarge,
        }
    }
}

/// Install the app's palette and UI scale into the global `gpui-component`
/// theme + the application context.
///
/// Idempotent: safe to call from every window-construction path, and to
/// re-call when the user changes appearance or scale at runtime.
pub fn install_theme(profile: ThemeProfile, scale: ScaleFactor, cx: &mut App) {
    install_theme_for_appearance(profile, scale, cx.window_appearance().into(), None, cx);
}

/// Install the app's palette using a concrete window's platform appearance.
///
/// Use this when the caller has a `Window`; it is the most accurate source on
/// Linux and mirrors `gpui-component`'s own system-appearance strategy.
pub fn install_theme_for_window(
    profile: ThemeProfile,
    scale: ScaleFactor,
    window: &mut Window,
    cx: &mut App,
) {
    install_theme_for_appearance(profile, scale, window.appearance().into(), Some(window), cx);
}

/// Apply a new value of the system accessibility settings. The theme is
/// installed again for each open window, as for a change of the appearance
/// (ADR 0101 Decision 11). A value that did not change does nothing.
pub fn apply_accessibility_settings(settings: AccessibilitySettings, cx: &mut App) {
    if cx.try_global::<AccessibilitySettings>() == Some(&settings) {
        return;
    }
    cx.set_global(settings);
    let environment = crate::ui::tokens::Environment::current(cx);
    let windows = cx.windows();
    if windows.is_empty() {
        install_theme(environment.profile, environment.scale, cx);
    }
    for handle in windows {
        let _ = handle.update(cx, |_, window, cx| {
            install_theme_for_window(environment.profile, environment.scale, window, cx);
        });
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "flat field-by-field assignment list is clearer than splitting"
)]
fn install_theme_for_appearance(
    profile: ThemeProfile,
    scale: ScaleFactor,
    system_appearance: Appearance,
    window: Option<&mut Window>,
    cx: &mut App,
) {
    let accessibility = AccessibilitySettings::current(cx);
    let appearance = appearance_for_profile(profile, system_appearance);
    let concrete_profile =
        effective_profile(profile, system_appearance, accessibility.increase_contrast);
    crate::ui::style::install_profile(
        concrete_profile,
        accessibility.increase_contrast,
        accessibility.reduce_transparency,
    );

    // Install the bundled Environment so every component can read appearance
    // and scale through the single SwiftUI-style accessor. Mirrors the scale
    // into the legacy `ScaleFactor` global for any code still reading it
    // directly during the migration.
    crate::ui::tokens::Environment {
        profile,
        appearance,
        scale,
        reduce_motion: accessibility.reduce_motion,
        increase_contrast: accessibility.increase_contrast,
        reduce_transparency: accessibility.reduce_transparency,
    }
    .install(cx);

    let mode = match appearance {
        Appearance::Dark => ThemeMode::Dark,
        Appearance::Light => ThemeMode::Light,
    };

    // Initialise / reset gpui-component's defaults for this mode.
    Theme::change(mode, window, cx);

    let theme = Theme::global_mut(cx);

    // ADR 0101 Decision 3: Figtree is the interface font family. The
    // monospace family stays the gpui-component default. See
    // `crate::ui::tokens::log_font_family`.
    theme.font_family = crate::ui::fonts::FIGTREE_FAMILY.into();

    // Backgrounds.
    let bg = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::SystemBackground,
    ));
    let group = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::GroupBackground,
    ));
    let open_entry = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::OpenEntryBackground,
    ));
    let floating = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::FloatingBackground,
    ));
    let hover = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::HoverBackground,
    ));
    let sidebar = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::SidebarBackground,
    ));

    // Labels.
    let label = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Label,
    ));
    let label2 = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::SecondaryLabel,
    ));

    // Fills / accents / borders.
    let fill = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::SystemFill,
    ));
    let separator = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Separator,
    ));
    let opaque_sep = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::OpaqueSeparator,
    ));
    let accent = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Accent,
    ));
    let accent_h = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::AccentHover,
    ));
    let accent_p = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::AccentPressed,
    ));
    let on_accent = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::OnAccent,
    ));
    let focus = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Focus,
    ));
    let selected = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::SelectedContent,
    ));

    // Status.
    let success = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Success,
    ));
    let danger = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Danger,
    ));
    let warning = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Warning,
    ));
    let info = hsla(resolve(
        concrete_profile,
        accessibility,
        SemanticColor::Info,
    ));

    // Window / canvas.
    theme.background = bg;
    theme.foreground = label;

    // Popover surfaces — the bug that started this overhaul.
    theme.popover = floating;
    theme.popover_foreground = label;

    // Primary buttons (filled).
    theme.primary = accent;
    theme.primary_hover = accent_h;
    theme.primary_active = accent_p;
    theme.primary_foreground = on_accent;

    // Secondary buttons (tinted / ghost surface).
    theme.secondary = fill;
    theme.secondary_hover = hover;
    theme.secondary_active = opaque_sep;
    theme.secondary_foreground = label;

    // Muted text & surfaces.
    theme.muted = group;
    theme.muted_foreground = label2;

    // Inputs.
    theme.input = group;
    theme.border = separator;
    theme.ring = focus;
    theme.caret = label;
    theme.selection = selected;

    // Accent + link.
    theme.accent = accent;
    theme.accent_foreground = on_accent;
    theme.link = accent;
    theme.link_hover = accent_h;
    theme.link_active = accent_p;

    // Status colors.
    theme.success = success;
    theme.success_hover = success;
    theme.success_active = success;
    theme.success_foreground = on_accent;
    theme.danger = danger;
    theme.danger_hover = danger;
    theme.danger_active = danger;
    theme.danger_foreground = on_accent;
    theme.warning = warning;
    theme.warning_hover = warning;
    theme.warning_active = warning;
    theme.warning_foreground = on_accent;
    theme.info = info;
    theme.info_hover = info;
    theme.info_active = info;
    theme.info_foreground = on_accent;

    // Sidebar — the sidebar tier of ADR 0101 Decision 1.
    theme.sidebar = sidebar;
    theme.sidebar_foreground = label;
    theme.sidebar_border = separator;
    theme.sidebar_accent = hover;
    theme.sidebar_accent_foreground = label;
    theme.sidebar_primary = accent;
    theme.sidebar_primary_foreground = on_accent;

    // Lists.
    theme.colors.list = bg;
    theme.list_hover = hover;
    theme.list_active = selected;
    theme.list_active_border = accent;
    theme.list_even = group;
    theme.list_head = group;

    // Tables.
    theme.table = bg;
    theme.table_hover = hover;
    theme.table_active = selected;
    theme.table_active_border = accent;
    theme.table_even = group;
    theme.table_head = group;
    theme.table_head_foreground = label2;
    theme.table_row_border = separator;

    // Title bar / window chrome.
    theme.title_bar = bg;
    theme.title_bar_border = separator;
    theme.window_border = separator;

    // Tabs.
    theme.tab = bg;
    theme.tab_active = open_entry;
    theme.tab_active_foreground = label;
    theme.tab_bar = bg;
    theme.tab_bar_segmented = group;

    // Misc surfaces that would otherwise show light defaults.
    theme.accordion = group;
    theme.group_box = group;
    theme.group_box_foreground = label;
    theme.description_list_label = group;
    theme.description_list_label_foreground = label2;
    theme.drop_target = selected;
    theme.skeleton = hover;
    theme.scrollbar = group;
    theme.scrollbar_thumb = opaque_sep;
    theme.scrollbar_thumb_hover = label2;

    theme.overlay = scrim();
    theme.drag_border = accent;

    // Force every open window to re-render so the new scale + colors are
    // picked up immediately. Without this, the next user interaction (or a
    // background timer) would be needed before any visual change.
    theme.tokens = theme.colors.into();
    Theme::sync_base(cx);
    cx.refresh_windows();
}

/// Resolve `token` through the concrete profile with the system
/// accessibility settings. The profile is concrete, so the appearance has no
/// effect.
fn resolve(
    profile: ThemeProfile,
    accessibility: AccessibilitySettings,
    token: SemanticColor,
) -> Rgba {
    resolve_accessible_color(
        profile,
        Appearance::Dark,
        token,
        accessibility.increase_contrast,
        accessibility.reduce_transparency,
    )
}

/// Convert our `Rgba` token output to gpui's `Hsla` that the theme stores.
#[inline]
fn hsla(c: Rgba) -> Hsla {
    c.into()
}

/// Translucent black used as overlay scrim under modals.
fn scrim() -> Hsla {
    Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.5,
    }
    .into()
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[gpui::test]
    fn adr_0071_migrated_base_and_component_palettes_follow_the_app(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            {
                install_theme(ThemeProfile::Dark, ScaleFactor::Medium, cx);
                let theme = Theme::global(cx);
                assert_eq!(theme.tokens.background.color, theme.background);
                assert_eq!(theme.tokens.scrollbar_thumb.color, theme.scrollbar_thumb);
                assert_eq!(
                    gpui_base::Theme::global(cx).tokens.colors.selection,
                    theme.selection
                );
            }
        });
    }

    #[gpui::test]
    fn m1_each_setting_reaches_the_environment(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            install_theme(ThemeProfile::Dark, ScaleFactor::Medium, cx);
            let plain = crate::ui::tokens::Environment::current(cx);
            assert!(plain.allows_motion());
            assert!(!plain.increase_contrast);
            assert!(!plain.reduce_transparency);

            apply_accessibility_settings(
                AccessibilitySettings {
                    reduce_motion: true,
                    increase_contrast: true,
                    reduce_transparency: true,
                },
                cx,
            );
            let env = crate::ui::tokens::Environment::current(cx);
            assert!(env.reduce_motion);
            assert!(env.increase_contrast);
            assert!(env.reduce_transparency);
            assert_eq!(env.profile, ThemeProfile::Dark);
        });
    }

    /// The theme view model with Increase Contrast and an explicit Dark
    /// profile gives an opaque border and the primary text color for
    /// secondary text.
    #[gpui::test]
    fn m3_increase_contrast_changes_the_theme_of_an_explicit_profile(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_component::init(cx);
            apply_accessibility_settings(
                AccessibilitySettings {
                    increase_contrast: true,
                    ..AccessibilitySettings::default()
                },
                cx,
            );
            for profile in [ThemeProfile::Dark, ThemeProfile::Light] {
                install_theme(profile, ScaleFactor::Medium, cx);
                let theme = Theme::global(cx);
                assert!((theme.border.a - 1.0).abs() < f32::EPSILON, "{profile:?}");
                assert_eq!(theme.muted_foreground, theme.foreground, "{profile:?}");
            }
        });
    }

    /// With the profile System, Increase Contrast gives the high-contrast
    /// colors of the window appearance.
    #[gpui::test]
    fn increase_contrast_with_system_installs_a_high_contrast_profile(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(|cx| {
            gpui_component::init(cx);
            apply_accessibility_settings(
                AccessibilitySettings {
                    increase_contrast: true,
                    ..AccessibilitySettings::default()
                },
                cx,
            );
            install_theme_for_appearance(
                ThemeProfile::System,
                ScaleFactor::Medium,
                Appearance::Dark,
                None,
                cx,
            );
            let theme = Theme::global(cx);
            let expected = hsla(resolve_accessible_color(
                ThemeProfile::HighContrastDark,
                Appearance::Dark,
                SemanticColor::SystemBackground,
                false,
                false,
            ));
            assert_eq!(theme.background, expected);
        });
    }
}
