# ADR 0101 Task 012: The Levels On The Surfaces

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks. Task 001 adds the tokens `GroupBackground` and `OpenEntryBackground`.

## Goal

Each surface uses the level of its use. A group or a card has the group level. An open entry in a group has the open entry level. A popover, a menu and a sheet have the floating fill and one shadow. In Light, a group is `#f2f2f7` on the white page.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 1, "Light levels" and "Light level values", owns this packet. Read Decisions 1 and 4 before the first edit:
  - The light page is white. A group or a card is `#f2f2f7`. An open entry in a group is `#ffffff`.
  - In Light, a popover, a menu and a sheet have `#ffffff` with the shadow of R-ELEVATION-1.
  - The light raised surface `#ececef` is not a level.
  - The dark levels stay: background `#0b0b0d`, surface `#1c1c1f` and raised surface `#26262a`.
- musicindex ADR 0068 R-ELEVATION-1 item 2 (ADR 0101 Decision 4): a popover, a menu and a sheet have one shadow. The shared token `shadow.floating` gives offset 16, blur 48 and alpha 0.5 dark and 0.18 light.
- musicindex ADR 0045 R-ELEVATION-2. `src/assets/tokens.json` gives `level` and its note: "color.light.surface is the fill of a floating layer. color.light.surface-raised is not a level".
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `Surface::render` (`src/ui/primitives/surface.rs:80`) gives Sunken and Raised the color `SecondarySystemBackground` and Floating the color `TertiarySystemBackground`. Examined.
- `bg_surface` and `bg_surface_hi` (`src/ui/style.rs:65` and `:68`) give `SecondarySystemBackground` and `TertiarySystemBackground`. Examined.
- `src/assets/tokens.json` lines 412 to 438 give `level` and `shadow.floating`. Examined.
- The context menu (`src/ui/primitives/context_menu.rs:358`) and the popover (`src/ui/primitives/popover.rs:215` and `:238`) use Floating.
- The field list (`src/ui/composites/search_field_list.rs:140`) uses Raised.
- `src/ui/composites/show_card.rs:86` and `:91` use Secondary and Tertiary.
- Reports and panels use Secondary: `src/ui/composites/startup_report.rs:47` and `:146`, `src/ui/composites/maintenance_page.rs:306`, `src/ui/composites/show_detail_panel.rs:266` and `:289`, `src/ui/composites/show_log_pane.rs:165` and `src/ui/composites/log_frame.rs:199`.
- The open and hover entries of a playlist use the Tertiary alias (`src/ui/shells/playlist.rs:769`, `:773` and `:961`).
- Hover fills use Secondary: `src/ui/composites/list_row.rs:306` and `src/ui/composites/selectable_tile.rs:112`. In Light, Secondary is `#ffffff`, the color of the page. This is an inference: a hover in Light does not show.
- `src/ui/theme_bridge.rs` maps Secondary and Tertiary to many `gpui-component` theme fields (lines 141 to 239).

Main session choice, 2026-11-15: `SecondarySystemBackground` and `TertiarySystemBackground` keep their values. The packet moves each surface to the level of its use. A token that no surface uses after the move is deleted in the same change.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Surface primitive | extend | `src/ui/primitives/surface.rs` | It gives the fill of each elevation. Map each elevation to its level there. |
| Style helpers | extend | `src/ui/style.rs` | `bg_surface` and `bg_surface_hi` give the fills of many callers. |
| Shadow tokens | extend | `src/ui/tokens.rs` | The token module owns each shadow. Add the floating shadow, with a test against `shadow.floating`. |
| Theme bridge | extend | `src/ui/theme_bridge.rs` | It gives the `gpui-component` fields. Give each field the level of its use. |
| Popover | extend | `src/ui/primitives/popover.rs` | It draws a floating layer. |
| Context menu | extend | `src/ui/primitives/context_menu.rs` | It draws a floating layer. |

## Required Changes

1. **The elevations.** Sunken and Raised use `GroupBackground`. Floating uses the floating fill: `#ffffff` in Light and the dark raised surface in Dark, with the floating shadow.
2. **The floating shadow.** Add the shadow of `shadow.floating` to `src/ui/tokens.rs`. A popover, a menu and a sheet use it. A test compares it with the copy.
3. **The callers.** Each caller of "Recorded Facts" uses the level of its use: a group, a card, a report and a panel use the group level. An open entry in a group uses the open entry level. Report each caller with its use and its level.
4. **The hover.** A hover fill differs from the fill below it in each theme. Report each hover fill that you changed.
5. **The theme bridge.** Each `gpui-component` field of `src/ui/theme_bridge.rs` gets the level of its use. Report each field.
6. **The contrast.** Each text pair on a new level has 4.5:1. Task 001 gives the pairs of the two tokens. Add the pairs of each other text color that this packet puts on a level.
7. **The field list height.** Operator choice, 2026-11-15: the field list (`src/ui/composites/search_field_list.rs:114`) is at most 320 px high. When the window is lower, the list stops at the bottom of the window and scrolls inside. This closes the max height item of ADR 0087 packet 010.
8. **The guard.** A guard proves that no file outside `src/ui/tokens.rs`, `src/ui/style.rs` and `src/ui/theme_bridge.rs` names `SecondarySystemBackground` or `TertiarySystemBackground`. Its message names ADR 0101 Decision 1 and the fix.

## Do Not Change

- No change of a level value or the dark levels.
- No change of the high-contrast colors, other than the mapping of the two new tokens.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test proves that each elevation of `Surface` gives its level in each theme.
- M2: a test proves that the floating shadow equals `shadow.floating` in each theme.
- M3: a test proves that each hover fill of step 4 differs from the fill below it in each theme.
- M4: the contrast tests give each pair of step 6 at 4.5:1 or more.
- M5: the guard of step 8 passes.
- M5a: a test proves the field list height. It is the lower value of 320 px and the space below the field in the window.
- M6: the fit tests of ADR 0097 pass.
- M7: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M:

- V1: in Light, the page is white, and a group, a card and a report are light gray.
- V2: in Light, an open playlist entry is white in its gray group.
- V3: a popover, a menu and a sheet have one soft shadow in each theme.
- V4: a hover on a list row shows in Light and in Dark.
- V5: the Dark theme looks as before.
- V6: in a window of 500 px height, the field list stops at the bottom of the window and scrolls.

## Test Commands

- `cargo test --lib ui::primitives::surface`
- `cargo test --lib ui::tokens`
- `cargo test --lib ui::contrast`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M5 by name. Give each caller of step 3 and each field of step 5 with its level. Give each token that you deleted.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a surface has no clear use: a group, an open entry or a floating layer.
- Stop if a contrast pair of step 6 is below 4.5:1.
- Stop if a `gpui-component` field needs a level for two uses.
