# ADR 0101 Task 005: The System Accessibility Settings

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks. If [task 002](adr-0101-task-002-header-glow.md) is not complete, this packet adds the field `reduce_transparency`.

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

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `ThemeProfile` (`src/theme_profile.rs:14`) has System, Dark, Light, HighContrastDark and HighContrastLight. Dark is the default.
- `effective_profile` (`src/ui/theme_profiles.rs:34`) maps System to Light or Dark from the window appearance. An explicit profile stays. Examined.
- `Environment` (`src/ui/tokens.rs:1270`) has `profile`, `appearance`, `scale` and `reduce_motion`. `allows_motion` reads `reduce_motion`. Examined.
- `install_theme_for_window` sets `reduce_motion: false` (`src/ui/theme_bridge.rs:85`). Examined.
- `src/app.rs:469` observes the window appearance and installs the theme again when the profile is System. Examined.
- `maybe_turning` (`src/ui/icons.rs:383`) turns an icon. The comment at `src/ui/icons.rs:343` says that the GPUI animation obeys the platform Reduce Motion. That does not set `reduce_motion` of HitIt.
- `Cargo.lock` holds `ashpd` 0.13.13 and `zbus` 5.19.0, through GPUI. `Cargo.toml` does not name them. Examined.
- This is an inference from the xdg-desktop-portal specification: the Settings portal gives the namespace `org.freedesktop.appearance` with the keys `contrast` (1 is higher contrast) and `reduced-motion` (1 is reduced motion). No portal key gives Reduce Transparency. Examine the specification and the `ashpd` 0.13 API before you use them.

Main session choice, 2026-11-15: with the profile System, Increase Contrast selects the high-contrast profile of the window appearance. An explicit profile that the person selected stays. Reduce Motion and Reduce Transparency apply with each profile. The operator can change this at the walk (open point 2 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points)).

Main session choice, 2026-11-15: on Linux, Reduce Transparency stays `false` when no portal key gives it. The report says so.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Environment | extend | `src/ui/tokens.rs` | `Environment` holds `reduce_motion`. Add `increase_contrast` and `reduce_transparency` there. |
| Profile selection | extend | `src/ui/theme_profiles.rs` | `effective_profile` maps System. Give it the contrast setting. |
| Theme install | extend | `src/ui/theme_bridge.rs` | It builds the `Environment`. Give it the read settings. |
| Settings source | new | `src/ui/system_settings.rs` | Searched `src/ui/`, `src/app.rs` and `src/runtime/` in HitIt. No code reads a system accessibility setting. Searched musicindex-lib 953e464 and Studio 44d1895 through the orchestrator on 2026-11-15 for ashpd and org.freedesktop.appearance: no match. Keep the reader in one file with no HitIt type, so that it can move to musicindex-lib when Studio needs it. |
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
