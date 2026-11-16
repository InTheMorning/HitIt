# ADR 0101 Task 001: The Shared Tokens

Status: Implemented - operator visual checks open, 2026-11-16. The orchestrator session said to start, with the approval of the operator of 2026-11-16. The visual checks of the three defect fixes of 2026-11-14 stay open in sections 15 to 17 of `docs/pending-human-checks.md`. This packet did not wait for them.

## Goal

The base tokens of HitIt agree with the shared token file. This packet changes the focus color, the type sizes, the radii, the spacing scale and the light levels in `src/ui/tokens.rs`. Tests compare these values and the focus ring with `src/assets/tokens.json`. The focus ring geometry does not change.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decisions 1, 3, 11 and 12 own this packet. Read Decisions 1, 3 and 12 fully before the first edit.
- musicindex owns the shared values. They are ADR 0069 R-TOKEN-1 (focus color) and ADR 0068 R-TYPE-1, R-TYPE-2, R-SPACE-1 and R-RADIUS-1. They are also ADR 0045 R-ELEVATION-2 (light levels) and ADR 0073 R-FOCUS-2 (the ring of Studio and HitIt, musicindex commit `785ae28`). `src/assets/tokens.json` is the HitIt copy of these values. Do not read a musicindex file. The copy gives each value that this packet needs.
- ADR 0039 owns the UI scale. Each changed base value scales as before.
- [AGENTS.md](../../AGENTS.md): the layers, the commit checks and the conventions.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from commit `6b75e07`, read on 2026-11-15.

- `src/assets/tokens.json` is the copy of musicindex-assets `b8399cc` (HitIt commit `77420fc`). It has `focus-ring.hitit` with width 2 and offset 0.
- The focus token is `Focus` in `src/ui/tokens.rs`: `#5795ff` dark (line 256) and `#3b7cde` light (line 365). Each is a copy of `AccentHover`. The copy gives `color.dark.focus` `#2d7bff` and `color.light.focus` `#005bd3`.
- `FontSize::px` in `src/ui/tokens.rs` (lines 638 to 647) gives Micro 11, Caption 12, Body 13, Headline 15, Title3 17, Title2 20, Title 24 and Display 30. The copy gives `type.desktop`: 26, 22, 17, 15, 13, 13, 12, 11, 10, 10 and 10. It gives `type.page-title.album-track-playlist.desktop` "large-title bold (26)" and `type.page-title.artist-label.display` 42 bold.
- `Radius::px` (lines 583 to 587) gives SM 4 (chips, badges), MD 6 (inputs), LG 10 (popovers, sheets), XL 14 (large overlays) and Full 999. The copy gives `shape.radius`: chip 6, row 10, field 10, popover 12, card 20 and sheet 24.
- `Spacing::px` (line 538) gives 2, 4, 8, 12, 16, 24 and 32. The copy gives `space` 2, 4, 8, 12, 16, 20, 24, 32 and 48.
- The light `SystemBackground` and `SecondarySystemBackground` are `#ffffff`, and `TertiarySystemBackground` is `#ececef` (lines 331 to 334). The copy gives `level.light` `#ffffff`, `#f2f2f7`, `#ffffff` and `level.dark` `#0b0b0d`, `#1c1c1f`, `#26262a`, for the page, a group or card, and an open entry. `color.light.surface-raised` `#ececef` is not a level.
- `CONTROL_FOCUS_RING_WIDTH` in `src/ui/layouts.rs:29` is 2 px. `src/ui/primitives/button.rs:807` draws the ring as an inset border, so its offset is 0.
- `DetailHeader::title_size` in `src/ui/composites/detail_header.rs` defaults to `FontSize::Title2`. `src/ui/shells/entity.rs:314`, `src/ui/shells/publisher.rs:146` and `src/ui/shells/name_level_page.rs:112` pass `FontSize::Display`.
- Two tests read the copy now: `button_tints_match_the_shared_tokens` and `adr_0056_glow_tokens_match_the_shared_tokens` in the test module of `src/ui/tokens.rs`.

Main session choice, 2026-11-15: each size role keeps its name. Each value moves to the nearest size of `type.desktop` that keeps the order of the roles. Title2 becomes 22 (title-1), Title becomes 26 (large-title) and Display becomes 42. Micro, Caption, Body, Headline and Title3 keep 11, 12, 13, 15 and 17, which are desktop sizes. The HitIt role `Headline` (15) is the shared `title-3`, not the shared `headline` (13). A rename of the roles is not in this packet.

Main session choice, 2026-11-15: each radius role keeps its name and gets the shared value of its use. SM is 6 (chip), MD is 10 (field and row), LG is 12 (popover) and XL is 20 (card). A new role `Sheet` gives 24.

Main session choice, 2026-11-15: two new spacing roles give 20 and 48. `LGPlus` is 20, between LG and XL. `XXXL` is 48.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Theme color tables | extend | `src/ui/tokens.rs` | `dark_palette` and `light_palette` give each color token. Change `Focus` and add the two level tokens there. |
| Size, radius and spacing roles | extend | `src/ui/tokens.rs` | `FontSize`, `Radius` and `Spacing` give each base value. Change the values and add the new roles there. |
| Shared token tests | extend | `src/ui/tokens.rs` | The test module reads `src/assets/tokens.json` with `include_str!`. Add the new comparisons in the same module as `button_tints_match_the_shared_tokens`. |
| Shared copy check | reuse | `tests/architecture_tests.rs` | `shared_token_copy_equals_musicindex_assets` compares the copy byte for byte. |
| Focus ring width | reuse | `src/ui/layouts.rs` | `CONTROL_FOCUS_RING_WIDTH` is the one ring width. Do not change it. |
| Contrast pairs | extend | `src/ui/contrast.rs` | The contrast tests give each pair of each role. Add the pairs of the new levels. |
| Page header title size | extend | `src/ui/composites/detail_header.rs` | `title_size` sets the size of a page title. The callers select the role of R-TYPE-2. |

## Required Changes

1. **The focus color.** Set `Focus` to `#2d7bff` dark and `#005bd3` light. Replace the comment "Derived: reuses AccentHover" with a comment that cites musicindex ADR 0069 R-TOKEN-1. Do not change the high-contrast profiles.
2. **The type sizes.** Set Title2 to 22, Title to 26 and Display to 42, as the main session choice gives. Update the doc comment of each changed role. Keep the per-role scale endpoints of ADR 0039 (`src/ui/layouts.rs:145` to `:156`) unless a test of ADR 0039 fails. If one fails, stop and ask.
3. **The page titles.** An album, track or playlist page title uses `FontSize::Title` (26, Large Title) in bold. An artist or label page title uses `FontSize::Display` (42) in bold. Examine each caller of `title_size` and the track page header. Change the size of each caller that does not agree. Update the doc comment of `title_size`. Report each caller with its page kind and its size.
4. **The radii.** Set SM 6, MD 10, LG 12 and XL 20, and add `Sheet` 24, as the main session choice gives. Update each doc comment. Find each sheet surface that sets `Radius::LG`, for example the confirmation dialog and the Inspect Sources sheet, and give it `Radius::Sheet`. Report each sheet that you changed.
5. **The spacing.** Add `LGPlus` (20) and `XXXL` (48) to `Spacing`, with `px` and `scaled`. Change no current caller.
6. **The light levels.** Add two color tokens: `GroupBackground` (dark `#1c1c1f`, light `#f2f2f7`) and `OpenEntryBackground` (dark `#26262a`, light `#ffffff`). Each doc comment cites musicindex ADR 0045 R-ELEVATION-2. Give the two tokens in each theme profile that lists the color tokens, the high-contrast profiles included. A high-contrast profile uses its own values for the same levels. Do not change `SecondarySystemBackground` or `TertiarySystemBackground`. Do not use the new tokens in a surface. The surface packets of ADR 0101 Decision 12 do that.
7. **The contrast pairs.** Add the contrast pairs of `Label`, `SecondaryLabel` and each status text color on `GroupBackground` and on `OpenEntryBackground`, in each theme. Each pair needs 4.5:1. If a pair fails, stop and ask.
8. **The token tests.** In the test module of `src/ui/tokens.rs`, add one test for each item. Each test reads `src/assets/tokens.json` and gives a failure message that names the rule and the fix:
   - The focus color of each theme equals `color.dark.focus` and `color.light.focus`.
   - The base value of each `FontSize` role is a size of `type.desktop` or `type.page-title.artist-label.display`. `FontSize::Title` equals the size of `type.desktop.large-title`. `FontSize::Display` equals `type.page-title.artist-label.display`.
   - The values of `Radius` SM, MD, LG, XL and Sheet equal `shape.radius` chip, field, popover, card and sheet.
   - The values of `Spacing` equal the `space` list, in order.
   - The light and dark `SystemBackground`, `GroupBackground` and `OpenEntryBackground` equal `level.light` and `level.dark`, in order.
   - `CONTROL_FOCUS_RING_WIDTH` equals `focus-ring.hitit.width`, and `focus-ring.hitit.offset` is 0, because HitIt draws the ring inset. The message cites musicindex ADR 0073 R-FOCUS-2. musicindex ADR 0073 asks for this test.

## Do Not Change

- No change of the focus ring geometry: its width, its inset border and the time it shows.
- No edit of `src/assets/tokens.json`. It is a read-only copy.
- No change of the accent, the status colors, the entity colors or the dark levels.
- No new use of `GroupBackground`, `OpenEntryBackground`, `Spacing::LGPlus` or `Spacing::XXXL` in a surface.
- No rename of a size, radius or spacing role.
- No change of the UI scale steps of ADR 0039.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: `shared_token_copy_equals_musicindex_assets` passes.
- M2: a test proves that the focus color of each theme equals the copy.
- M3: a test proves that each `FontSize` value is a shared desktop size, and that Title and Display equal the R-TYPE-2 page title sizes.
- M4: a test proves that the `Radius` values equal `shape.radius`.
- M5: a test proves that the `Spacing` values equal `space`.
- M6: a test proves that the three level tokens of each theme equal `level.light` and `level.dark`.
- M7: a test proves that `CONTROL_FOCUS_RING_WIDTH` equals `focus-ring.hitit.width` and that `focus-ring.hitit.offset` is 0. Its message cites musicindex ADR 0073 R-FOCUS-2.
- M8: the contrast tests give each pair of step 7 at 4.5:1 or more.
- M9: a test of `src/ui/composites/detail_header.rs` or of each page shell proves the title size of an album, a track, a playlist, an artist and a label page.
- M10: the fit tests of ADR 0097 pass at their widths.
- M11: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, listed apart, for the operator to walk in Dark and Light at M, at normal and narrow widths:

- V1: Tab through a HitIt page. The focus ring is 2 px with no gap, in the new focus color, and it shows fully on a list row, a tile and a dialog button at 390 px and at 940 px.
- V2: an album, a track and a playlist page title is 26 px bold. An artist and a label page name is 42 px bold. No title is clipped. This walk also carries ADR 0084 packet 026 V10, the clipped titles.
- V3: chips, fields, rows, popovers, cards and sheets show the new radii. A sheet has a larger radius than a popover.
- V4: the text of each list and page keeps its place, with no clipped text and no overlap.

The moved visual requirements of size and color in the [overhaul plan](../plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07) stay. Each one asks for each element in its place, with no clipped text, in Light and Dark at normal and narrow widths. ADR 0101 replaces none of them. This packet changes the sizes that they check. Thus the operator walks them on the build of this packet, in the walks of their owners. The max height of the field list (ADR 0087 packet 010) stays for a surface packet of ADR 0101 Decision 12.

## Test Commands

- `cargo test --lib ui::tokens`
- `cargo test --lib ui::contrast`
- `cargo test --lib ui::composites::detail_header`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M2 to M9 by name. Give each caller of step 3 with its page kind and its size, and each sheet of step 4. Give the contrast ratio of each pair of step 7.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a value of `src/assets/tokens.json` is not the value that "Recorded Facts" gives.
- Stop if a contrast pair of step 7 is below 4.5:1.
- Stop if a scale test of ADR 0039 fails after the new type sizes.
- Stop if a page kind of step 3 is not clear from its shell.
- Stop if a fit test of ADR 0097 fails, and report the view and the width.

## Implementation Report - 2026-11-16

A delegate made the first change. It stopped at two conditions of "Stop And Ask": a test of ADR 0039 and a fit test of ADR 0097 gave errors. The primary agent examined each test and completed the packet.

Files changed: `src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/contrast.rs`, `src/ui/composites/detail_header.rs`, `src/ui/composites/mod.rs`, `src/ui/composites/track_header.rs`, `src/ui/composites/settings.rs`, `src/ui/composites/track_detail_surface.rs`, `src/ui/icons.rs`, `src/ui/style.rs`, `src/ui/primitives/fit_probe.rs`, `src/ui/shells/entity.rs`, `src/ui/shells/name_level_page.rs`, `src/ui/shells/playlist.rs`, `src/ui/shells/publisher.rs` and `tests/architecture_tests.rs`.

Tests:

- M2: `focus_color_matches_the_shared_tokens`
- M3: `font_sizes_match_the_shared_tokens`
- M4: `radii_match_the_shared_tokens`
- M5: `spacing_matches_the_shared_tokens`
- M6: `levels_match_the_shared_tokens`
- M7: `focus_ring_matches_the_shared_tokens`
- M8: `dark_palette_meets_wcag`, `light_palette_meets_wcag` and the two tests of the high contrast profiles. Each one includes the 12 new pairs and needs 4.5:1. The delegate did not record the ratio of each pair.
- M9: `page_title_size_follows_the_page_kind` in `src/ui/composites/detail_header.rs`

Page titles of step 3. `PageTitleKind` in `src/ui/composites/detail_header.rs` gives the dimension of the title for each type of page:

| Caller | Page type | Title before | Title after |
| --- | --- | --- | --- |
| `render_contract_header`, `src/ui/shells/entity.rs` | album (release) | Display (30) | Title (26) |
| `TrackHeader`, `src/ui/composites/track_header.rs` | track page and track inspector | Title2 (20) | Title (26) |
| `src/ui/shells/playlist.rs` | playlist | Title2 (20) | Title (26) |
| `src/ui/shells/name_level_page.rs` | artist | Display (30) | Display (42) |
| `src/ui/shells/publisher.rs` | feed page with a role badge | Display (30) | Display (42), `PageTitleKind::Label` |

Each page title is bold.

Step 4: no sheet sets `Radius::LG`. The confirmation dialog uses the component theme value `radius_lg`. The component theme also gives that value to alerts, tabs, accordions and notifications, so this packet does not change it. The Inspect Sources sheet does not use it. Thus no sheet changed.

Main session choices, 2026-11-16:

- The ADR 0039 rule is the multiplier of each role at each UI scale step (`reservation_endpoints`, `src/ui/layouts.rs`). The multipliers do not change. The table of `adr_0039_type_outcomes_match_ratified_values` and the bases of `adr_0039_medium_type_returns_each_role_base_exactly` use the new bases of Title2, Title and Display with the same multipliers.
- `adr_0083_display_is_30_at_medium_and_grows_with_scale` asserted a rule of the archived ADR 0083. ADR 0101 Decision 3 replaces that rule, so the test is deleted. M3 and `adr_0039_type_grows_monotonically_per_role_across_steps` prove its two facts.
- The fit probe gave a cut for a row that its scroll area hid, because the row crossed the bottom of the window (My playlists, 390 px). The probe gives a vertical cut only for an element that its area shows. The test `adr_0097_the_window_edge_cuts_only_a_shown_element` proves this.
- With the Settings title at 22 px, the header of Settings did not stay on one line at XL in a page of 438 px. Then the ADR 0074 test `adr_0074_short_windows_reserve_space_for_task_content` gave an error. The space between the items of the header is `Spacing::XS`. ADR 0093 task 001 replaces this header. The operator examines this item in V5.
- `src/ui/style.rs` had a second radius scale with the earlier values. It is deleted, and its three callers use `Radius`.
- `render_rss_icon_link` in `src/ui/icons.rs` had no caller. It is deleted, with its entry in `interactive_surfaces_route_through_minimum_hit_target_token`.

Open points for a surface packet:

- The component theme values `radius` (6) and `radius_lg` (8) are not the shared radii.
- The high contrast profiles give their own values for the two new levels: dark `#0d0d0d` and `#161616`, light `#f7f7f7` and `#ffffff`.

Visual, added to the list above:

- V5: Settings at XL in a narrow window. The title and the two menus are on one line, with a small space between them.


## Walk Result - 2026-11-16

The agent walked V1 to V5 on an Xvfb display with the debug binary, in Dark. Light was not walked.

- V1: Fail.
  - The ring has the new focus color.
  - On "Download album" of an Index release page, a dark space of approximately 4 px is between the ring and the button.
  - The ring is a border of the hit target (`keyboard_button_focus`, `src/ui/primitives/button.rs`). The hit target is 44 px high or more, and the shown button is smaller.
  - Commit `72f37cf` changed only the color of this ring, so the space was there before this packet.
  - [ADR 0101 task 014](adr-0101-task-014-the-button-ring-has-no-gap.md) corrects it.
- V2: Pass.
  - "GLOBAL FEDPILLED INDUSTRIAL COMPLEX" is 26 px bold on one line.
  - The publisher page of "Fedpill Records" gives its name at 42 px bold on two lines, with no clipping.
  - The line height is approximately 1.6 (68 px between the two lines). The page title uses the line height of GPUI. The shared tokens give no line height.
- V3: Pass for the HitIt radii: the menu of Settings is a popover of 12 px. The path field of Settings, Diagnostics is a gpui-component input with a corner of approximately 4 px. This is the open point of the component theme radius above.
- V4: Pass. The release page, the publisher page, Settings and Releases at XL show no clipped text and no overlap.
- V5: Pass. Settings, Diagnostics at XL in a window of 440 px: the title and the two menus are on one line, with a small space between them.
