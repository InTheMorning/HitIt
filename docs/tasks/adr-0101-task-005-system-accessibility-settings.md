# ADR 0101 Task 005: The System Accessibility Settings

Status: Implemented - operator visual checks open, 2026-11-16.

## Goal

HitIt reads three settings from the system: Reduce Motion, Increase Contrast and Reduce Transparency. The theme view model applies them. A change of a setting while HitIt runs applies with no restart.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 1, "High contrast", and Decision 11 own this packet. Read Decisions 1, 11 and 12 before the first edit.
- musicindex ADR 0068 R-A11Y-1 (`../musicindex/docs/adr/0068-the-foundations-of-the-three-apps.md` lines 216 to 228, at musicindex `785ae28`). HitIt reads the same three settings from the operating system:
  1. Reduce Motion. A movement becomes a fade or no change. The live pulse, the skeleton sheen and the turning icon stop. A turning icon then shows a static state with its word.
  2. Increase Contrast. Separators become opaque. Each control gets a 1 px outline. Secondary text uses the primary text color. HitIt completes its two high-contrast profiles for the same purpose.
  3. Reduce Transparency. The grain, the glow blur and the translucent bars are replaced with opaque fills.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ba43138`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `ThemeProfile` (`src/theme_profile.rs:14`) has System, Dark, Light, HighContrastDark and HighContrastLight. Dark is the default.
- `effective_profile` (`src/ui/theme_profiles.rs:34`) maps System to Light or Dark from the window appearance. An explicit profile stays. Examined.
- `Environment` (`src/ui/tokens.rs:1270`) has `profile`, `appearance`, `scale` and `reduce_motion`. `allows_motion` reads `reduce_motion`. Examined.
- `install_theme_for_window` sets `reduce_motion: false` (`src/ui/theme_bridge.rs:85`). Examined.
- `src/app.rs:469` observes the window appearance and installs the theme again when the profile is System. Examined.
- `maybe_turning` (`src/ui/icons.rs:383`) turns an icon. The comment at `src/ui/icons.rs:343` says that the GPUI animation obeys the platform Reduce Motion. That does not set `reduce_motion` of HitIt.
- `Cargo.lock` holds `ashpd` 0.13.13 and `zbus` 5.19.0, through GPUI. `Cargo.toml` does not name them. Examined.
- This is an inference from the xdg-desktop-portal specification: the Settings portal gives the namespace `org.freedesktop.appearance` with the keys `contrast` (1 is higher contrast) and `reduced-motion` (1 is reduced motion). No portal key gives Reduce Transparency. Examine the specification and the `ashpd` 0.13 API before you use them.

Operator choice, 2026-11-15: with the profile System, Increase Contrast selects the high-contrast profile of the window appearance. An explicit profile that the person selected stays. Reduce Motion and Reduce Transparency apply with each profile. This is open point 2 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points).

Main session choice, 2026-11-15: on Linux, Reduce Transparency stays `false` when no portal key gives it. The report says so.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Environment | extend | `src/ui/tokens.rs` | `Environment` holds `reduce_motion`. Add `increase_contrast` and `reduce_transparency` there. |
| Profile selection | extend | `src/ui/theme_profiles.rs` | `effective_profile` maps System. Give it the contrast setting. |
| Theme install | extend | `src/ui/theme_bridge.rs` | It builds the `Environment`. Give it the read settings. |
| Settings source | new | `src/ui/system_settings.rs` | Searched `src/ui/`, `src/app.rs` and `src/runtime/` in HitIt. No code reads a system accessibility setting. Searched musicindex-lib 953e464 and Studio 3cbb5bd through the orchestrator on 2026-11-15 for ashpd and org.freedesktop.appearance: no match. Keep the reader in one file with no HitIt type, so that it can move to musicindex-lib when Studio needs it. |
| Appearance observer | extend | `src/app.rs` | It installs the theme again after a change. Do the same after a change of a setting. |

## Required Changes

1. **The source.** Write a trait for the source of the three settings, with a portal source and a fake source for tests. The portal source reads `org.freedesktop.appearance` through `ashpd`, and it follows the change signal. A failed read gives `false` for each setting and one log line. The read does not block the UI thread.
2. **The environment.** Add `increase_contrast` and `reduce_transparency` to `Environment`. Set the three values from the source.
3. **The profile.** With the profile System and Increase Contrast, `effective_profile` gives HighContrastDark or HighContrastLight from the appearance.
4. **Increase Contrast with a different profile.** Use an explicit Dark or Light profile with Increase Contrast. Then separators are opaque and each control has a 1 px outline. Secondary text uses the primary text color. Find the place in the theme that gives these values, and change them there.
5. **Reduce Motion.** `allows_motion` is false. The turning icon shows its static state with its word. The live pulse stops. Report each animation that you found, with its file and line.
6. **Reduce Transparency.** Each translucent bar gets an opaque fill. The glow of task 002 reads the field. Report each translucent fill that you found.
7. **The change while HitIt runs.** A change of a setting installs the theme again, as a change of the appearance does.

## Do Not Change

- No change of the high-contrast colors. Their contrast tests stay.
- No Settings control for the three settings. The system owns them.
- No new dependency other than a direct `ashpd` entry in `Cargo.toml` at the version in `Cargo.lock`.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test with the fake source proves that each setting reaches `Environment`.
- M2: a test proves that System with Increase Contrast gives the high-contrast profile of each appearance, and that an explicit Dark or Light stays.
- M3: a test proves that Increase Contrast with Dark or Light gives opaque separators, a 1 px control outline and the primary text color for secondary text.
- M4: a test proves that Reduce Motion stops the turning icon and keeps its word.
- M5: a test proves that Reduce Transparency gives opaque fills to each translucent bar of step 6.
- M6: a test proves that a failed read gives `false` for each setting.
- M7: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator. Each step needs a change of a system setting:

- V1: with the profile System, turn on high contrast in the desktop settings. HitIt changes to its high-contrast profile with no restart.
- V2: turn on reduced animation in the desktop settings. A download icon stops turning and keeps its word.
- V3: with the profile Dark and high contrast on, the separators and the control outlines are clear.

## Test Commands

- `cargo test --lib ui::system_settings`
- `cargo test --lib ui::theme_profiles`
- `cargo test --lib ui::tokens`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M6 by name. Give the portal keys that you used and the specification version that names them. Give each animation of step 5 and each translucent fill of step 6. Say if a key for Reduce Transparency exists.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if `ashpd` 0.13 cannot read the Settings portal without a second async runtime.
- Stop if the portal gives no key for contrast or reduced motion.

## Implementation Report - 2026-11-16

Status: the mechanical criteria M1 to M7 pass. The visual criteria V1 to V3 are open. They need the operator.

Files changed: `Cargo.toml`, `Cargo.lock`, `src/ui/system_settings.rs` (new), `src/ui/mod.rs`, `src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/theme_bridge.rs`, `src/ui/style.rs`, `src/ui/icons.rs`, `src/ui/primitives/button.rs`, `src/app/bootstrap.rs`, `tests/architecture_tests.rs`.

Portal keys: `contrast` (1 is higher contrast) and `reduced-motion` (1 is reduced motion) of the namespace `org.freedesktop.appearance`. Source: `ashpd` 0.13.13, `src/desktop/settings.rs` (`CONTRAST_KEY`, `REDUCED_MOTION_KEY`, `Settings::contrast`, `Settings::reduced_motion`, `receive_setting_changed_with_args`). No key for Reduce Transparency exists in that API. On Linux, Reduce Transparency stays `false`.

Dependency: a direct `ashpd` entry at `=0.13.13` with `default-features = false` and the features `async-io` and `settings`. These are the features that GPUI uses. No second async runtime is added. `Cargo.lock` gains only the line `ashpd` in the `hitit` package.

Tests:

- M1: `ui::system_settings::tests::m1_each_setting_reaches_the_environment` and `ui::theme_bridge::migration_tests::m1_each_setting_reaches_the_environment`.
- M2: `ui::theme_profiles::tests::increase_contrast_selects_the_high_contrast_profile_of_each_appearance`, `increase_contrast_keeps_an_explicit_profile` and `ui::theme_bridge::migration_tests::increase_contrast_with_system_installs_a_high_contrast_profile`.
- M3: `ui::theme_profiles::tests::increase_contrast_gives_opaque_separator_and_primary_secondary_text`, `ui::theme_bridge::migration_tests::m3_increase_contrast_changes_the_theme_of_an_explicit_profile` and `ui::primitives::button::tests::increase_contrast_gives_each_button_a_one_pixel_outline`.
- M4: `ui::icons::tests::reduce_motion_stops_a_turning_icon`. The word of the state is in the tooltip and the accessibility label. The turn flag does not change them.
- M5: `ui::theme_profiles::tests::reduce_transparency_gives_an_opaque_fill_for_each_translucent_token` and `ui::primitives::button::tests::reduce_transparency_gives_a_button_an_opaque_hover_fill`.
- M6: `ui::system_settings::tests::m6_a_failed_read_gives_false_for_each_setting`.

Animations found: the turning icon (`src/ui/icons.rs`, `maybe_turning`, stopped by `icon_turns`). The animated thumbnail already uses `allows_motion` (`src/library/app_impl.rs`). No live pulse or skeleton sheen exists in the code. The skeleton has no shimmer by design.

Translucent fills found, all as tokens in `src/ui/tokens.rs`: `Separator`, `AccentTint`, `SidebarGlowTop`, `AccentOutline`, `DangerTint`, `ButtonFill`, `ButtonFillHover`. Reduce Transparency lays each over `SystemBackground` (`resolve_accessible_color`). The hover fill of a plain button is also made opaque. Not changed: the modal scrim (`theme_bridge.rs`, `scrim`), the cover backdrop gradient (`src/ui/tokens.rs`, `CoverBackdrop`) and the disabled opacity of a control. They are not bars.

Notes: the guard `animation_paths_route_through_reduce_motion_environment` required the text `reduce_motion: false`. It now requires the read from the settings. The settings are read after the first window opens, so the first frame can show the old values.

## Primary Agent Check - 2026-11-16

- The primary agent read the diff and ran the gate: fmt, clippy, 3173 lib tests, 412 architecture tests and the debug build pass.
- `Cargo.lock` adds no package. GPUI already builds `ashpd` 0.13.13.
- The guard `animation_paths_route_through_reduce_motion_environment` now requires `reduce_motion: accessibility.reduce_motion`. The guard keeps its function: the environment sets Reduce Motion. The primary agent accepts this change.
- On the Xvfb display with no session bus, the app writes one log line ("The system accessibility settings are not available") and runs with no preference.
- V1 to V3 need a change of a desktop setting. The agent cannot do them on the Xvfb display.

## Operator Report - 2026-11-19

The operator saw this line at the start: "the portal gives no reduced-motion key: Portal request failed: org.freedesktop.portal.Error.NotFound". The portal of that desktop has the contrast key and no reduced-motion key. Thus the read gave an error, and HitIt did not use the contrast key.

Main session choice, 2026-11-19: HitIt reads each key alone. A missing key gives no preference for that key only. The read gives an error and the log line only when the portal gives no key. The tests `a_missing_key_keeps_the_other_setting` and `the_read_fails_when_both_keys_are_missing` examine this.
