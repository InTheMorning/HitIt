# ADR 0101 Task 012: The Levels On The Surfaces

Status: Implemented - operator visual checks open, 2026-11-16.

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

## Implementation Report - 2026-11-16

Status: The change is in the working tree. There is no commit. The visual checks V1 to V6 are open.

### Files changed

`src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/style.rs`, `src/ui/theme_bridge.rs`, `src/ui/contrast.rs`, `src/ui/primitives/surface.rs`, `src/ui/primitives/popover.rs`, `src/ui/primitives/button.rs`, `src/ui/composites/search_field_list.rs`, `src/app/search_field_list.rs`, `src/app/toolbar.rs`, `src/ui/composites/` (`download_status_item`, `list_row`, `live_status_strip`, `log_frame`, `maintenance_page`, `selectable_tile`, `selection_action_bar`, `show_card`, `show_detail_panel`, `show_log_pane`, `startup_report`, `track_drag`), `src/ui/shells/playlist.rs`, `src/ui/shells/library/sidebar.rs`, `tests/architecture_tests.rs`.

### Tests

- M1: `ui::primitives::surface::tests::each_elevation_gives_its_level_in_each_theme`.
- M2: `ui::tokens::tests::floating_shadow_matches_the_shared_tokens`.
- M3: `ui::tokens::tests::each_hover_fill_differs_from_the_fill_below_it_in_each_theme`. It covers four profiles.
- M4: `ui::contrast::tests::dark_palette_meets_wcag`, `light_palette_meets_wcag`, `high_contrast_dark_profile_meets_wcag`, `high_contrast_light_profile_meets_wcag` and `pressable_button_token_pairs_meet_wcag`.
- M5: `adr_0101_surfaces_do_not_name_the_old_surface_tokens` and `adr_0101_level_guard_fails_for_a_shell_sample` in `tests/architecture_tests.rs`.
- M5a: `ui::composites::search_field_list::tests::field_list_height_is_the_lower_of_the_cap_and_the_space_below` and `field_list_height_follows_the_window`.
- M6: `app::fit_check::` passes (3 tests).

### Tokens

- New: `FloatingBackground` (Dark `#26262a`, Light `#ffffff`, high contrast as `OpenEntryBackground`).
- New: `HoverBackground` (Dark `#26262a`, Light `#f2f2f7`, high contrast keeps the values of `TertiarySystemBackground`).
- New: `FloatingShadow` in `src/ui/tokens.rs`. It has offset 16, blur 48 and alpha 0.5 Dark and 0.18 Light.
- Deleted: `SecondarySystemBackground` and `TertiarySystemBackground`, with their palette entries.
- Deleted: `bg_surface` and `bg_surface_hi` in `src/ui/style.rs`. New there: `bg_group`, `bg_open_entry`, `bg_floating` and `bg_hover`.

### Callers (step 3)

| Caller | Use | Level |
|---|---|---|
| `Surface` Sunken and Raised | group, card | group |
| `Surface` Floating (context menu, popover body) | floating layer | floating fill and floating shadow |
| Popover arrow | floating layer | floating fill |
| Field list (`search_field_list.rs`) | dropdown over content | Floating elevation, so floating fill and shadow |
| Field list row border | blends with the list fill | floating fill |
| `show_card.rs` | card | group |
| `startup_report.rs` (2 places), `maintenance_page.rs`, `show_detail_panel.rs` (2), `show_log_pane.rs`, `log_frame.rs` | report, panel | group |
| `toolbar.rs`, `download_status_item.rs`, `live_status_strip.rs`, `selection_action_bar.rs` (2) | bar | group |
| `playlist.rs` unavailable row | open entry | open entry |
| `track_drag.rs` drag preview | floating layer | floating fill |

### Hover fills (step 4)

| Caller | Before | After | Below it |
|---|---|---|---|
| `list_row.rs`, `selectable_tile.rs` | Secondary | group | page |
| `button.rs` plain button, alpha 0.6 | Secondary | group | page, floating, open entry |
| `show_card.rs` card | Tertiary | open entry | group |
| `playlist.rs` available row | Tertiary alias | hover | page |
| `playlist.rs` unavailable row (new) | Tertiary alias | group | open entry |
| `playlist.rs` drag handle | Tertiary alias | group | page, open entry |
| `sidebar.rs` row | Tertiary alias | hover | sidebar |

### Theme bridge fields (step 5)

- floating: `popover`.
- group: `muted`, `input`, `list_even`, `list_head`, `table_even`, `table_head`, `tab_bar_segmented`, `accordion`, `group_box`, `description_list_label`, `scrollbar`.
- open entry: `tab_active`.
- hover: `secondary_hover`, `sidebar_accent`, `list_hover`, `table_hover`, `skeleton`.
- sidebar (`SidebarBackground`): `sidebar`.
- Unchanged: `background`, `title_bar`, `tab`, `tab_bar`, `list` and `table` use the page level.

### Contrast (step 6)

Each pair of the two deleted tokens moved to a new token. Added pairs: floating fill with Label, SecondaryLabel, TertiaryLabel, Accent, WarningLabel and DangerLabel. Hover fill with SecondaryLabel, TertiaryLabel and Accent. Open entry with TertiaryLabel. All pass. TertiaryLabel on the Light hover fill is 4.54:1.

### Field list height (step 7)

`row_list_max_height` returns the lower value of 320 px and the space below the field. The list writes the bottom edge of the field to a cell. It asks for one new frame when that value changes. `SearchFieldList::new` has a new argument, `field_bottom`.

### Choices that the task file does not give

- Two new tokens, `FloatingBackground` and `HoverBackground`. The two deleted tokens had no name for these uses.
- Light `HoverBackground` is `#f2f2f7`. The value of `TertiarySystemBackground` was `#ececef`. TertiaryLabel on `#ececef` is 4.30:1, below 4.5:1. Thus the Light hover is the same as the group fill. In Light, a hover on a row in a group does not show.
- A bar surface (toolbar, status strip, selection bar, download item) has the group level. In Light, the toolbar changes from white to gray. The task file names no level for a bar.
- The field list is a floating layer, not a group. It uses Floating, so it has the shadow.
- The unavailable playlist row has the open entry fill and the group fill as its hover.
- The two tokens are deleted. Thus the guard of step 8 only stops a new use of these names.
- The sidebar bridge field uses `SidebarBackground`. Sidebar hover in Light (`#f2f2f7` on `#f5f5f7`) differs only slightly.
- The field list height does not count the height of the notices. A list with a notice can extend below the window edge by the height of the notice. The anchor then moves the list up.
- A new constant, `SURFACE_BORDER_WIDTH`, in `src/ui/primitives/surface.rs`, because the composite literal guard forbids a `px(...)` literal.
- The test wrapper of the field list in `src/app/search_field_list.rs` and `src/ui/composites/search_field_list.rs` is 36 px and 40 px high. Before, the wrapper filled the window, and the list started at the bottom edge of the window.
- The duplicate pair `Focus` on `SystemBackground` is unchanged.

### Checks

- `cargo fmt -- --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test`: 3233 passed in the library, 416 passed in `architecture_tests`, 0 defective.
- `cargo build --bin hitit`: pass.
- Visual checks V1 to V6: open. The primary agent does the walk.

## Primary Agent Check - 2026-11-16

The primary agent read the diff and ran the four checks of M7. All pass: 3233 library tests and 416 architecture tests.

The primary agent accepts the choices of the delegate, with these notes:

- The toolbar and the other bars have the group level. In Light, the toolbar is gray (`#f2f2f7`) on the white page. Section 33 of `docs/pending-human-checks.md` asks the operator to examine it.
- In Light, the hover fill is the group fill. A hover on a row on the page shows. A hover on a row in a group does not show, and a hover on a sidebar row is almost the same as the sidebar. A darker fill puts TertiaryLabel below 4.5:1. This stays open.
- The primary agent corrected the comment of `bg_hover` in `src/ui/style.rs`. The comment before the correction said that the fill differs from the group.

## Walk Result - 2026-11-16

The agent walked on an Xvfb display with the debug binary at M. The test data has no playlist.

- V1: Pass in Light. The page is `#ffffff`. The toolbar, the Show cards and the Cuelist panel are `#f2f2f7`.
- V2: Open. The test data has no playlist.
- V3: Pass for the sort menu in Light and Dark, and for the field list in Light. Each one has the floating fill and one soft shadow. The test did not open a sheet.
- V4: Pass on the Releases list. A hovered row is `#f2f2f7` in Light and `#1c1c1f` in Dark.
- V5: Pass. In Dark, the page is `#0b0b0d`, the toolbar and the cards are `#1c1c1f`, and the menu is `#26262a`. These are the values before the change.
- V6: Pass in Light. In a window of 300 px height, the field list stops at the window bottom and scrolls to the last row. In a window of 500 px height, the 320 px limit applies.

The operator confirmation stays open.
