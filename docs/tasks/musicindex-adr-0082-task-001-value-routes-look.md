# musicindex ADR 0082 Task 001: The Value Routes Look

Status: Closed. The operator passed the visual criteria on 2026-11-17 (`cbdbf62` and `5407bd3`).

## Goal

The open value routes cell of the Library track detail compare grid uses the shared look of musicindex ADR 0082 (musicindex `424575f`, R-ROUTES-4 text at musicindex `6f37908`). The cell shows a summary bar, a names line and a panel with the groups "Recipients" and "Fees". Before this task, the cell showed a text tree.

The orchestrator sent this task on 2026-11-17. The rules that apply are R-ROUTES-1 to R-ROUTES-5, R-ROUTES-7 and R-ROUTES-8.

## Orchestrator Choices

Each choice is from 2026-11-17.

1. Operator choice (orchestrator, 2026-11-17): option A. Each cell is one block with no block label. R-ROUTES-6 does not apply to HitIt, because of R-ROUTES-6 item 3 and HitIt ADR 0099. HitIt does not read `value_blocks`.
2. Operator choice (orchestrator, 2026-11-17): the address of a row is on line 2, under the name, at all widths (musicindex `6f37908`, R-ROUTES-4 items 1, 5 and 6). A key uses the short form of musicindex ADR 0076. A Lightning address shows in full. An address that is wider than line 2 is cut with an ellipsis, and the hover text gives the full address. The row has no address column.
3. Operator choice (orchestrator, 2026-11-17): the open content of a row has labeled lines under the full address and Copy.

   "Type" gives "Lightning address" for `lnaddress`, "Lightning node" for `node` and the raw type for another type. "Shares" gives "N of TOTAL". TOTAL is the sum of the shares that the percentage uses.

   "Custom key" and "Custom value" show when the route has them. The open content does not show the percentage or "split". The labels are in one column. The address in the open content breaks only after "@" and after a dot.
4. The full copy of `tokens/tokens.json` from musicindex-assets `d979ef3` goes to `src/assets/tokens.json` in the same commit as this task.

## Main Session Choices

1. The panel shows in one row under the compare row, and the row spans all columns. The cell keeps its summary, for example "[3 items]". Reason: in the Inspect Sources sheet, the RSS value cell is approximately 160 px wide and the ID3 value cell is approximately 95 px wide. The row of R-ROUTES-4 needs approximately 300 px. The live walk showed the overflow (shot `s12-rss-open.png`).

   Each panel has the heading of its column, for example "RSS" or "MP3". The panels follow the sequence of the columns.
2. In a row that is narrower than the columns, the meter and the percentage column become smaller. The name keeps a minimum of 3 rem, and the percentage column keeps a minimum of 3.25 rem. These values are HitIt values, not shared tokens.
3. The high contrast themes have no shared share color. HitIt uses the accent color of the profile: `#66d9ff` for high contrast dark and `#0040dd` for high contrast light.
4. Defect correction: a tool such as mutagen ends a `TXXX` value with a NUL character, and the `id3` crate keeps it. The value routes JSON of such a file did not parse, and the compare grid showed a difference that was not real. The reader now removes a NUL character at the end of a `TXXX` value.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Share percentages, share parse and short address | reuse | Cargo.toml | The `musicindex-value` crate of musicindex-lib gives `percentages` and `short_address` (musicindex ADR 0075 R-VALUE-4, ADR 0076 R-ADDRESS-2). |
| Payment route and its parser | reuse | src/metadata.rs | `PaymentRoute` and `parse_value_routes` read the cell value. |
| Value routes view model | new | src/view_models/value_routes.rs | Searched src/view_models, src/metadata.rs and ~/build/musicindex-lib. No component makes the bar, the names line, the groups or the open lines. |
| Grid expansion view model | extend | src/view_models/track_metadata_grid.rs | `value_routes_panels` gives the open panels under the compare row. |
| Share colors and value routes sizes | extend | src/ui/tokens.rs | `SemanticColor::Share`, `ShareTrack` and `ValueRoutesShape` from the shared tokens. |
| High contrast colors | extend | src/ui/theme_profiles.rs | The share colors of the two high contrast profiles. |
| Disclosure indicator, button, tooltip and divider | reuse | src/ui/primitives | The row chevron, Copy, the hover text and the separators. |
| Grid row that spans all columns | reuse | src/ui/composites/track_metadata_grid.rs | The group heading cell already uses `col_span`. The panel row does the same. |
| Expandable cell display | reuse | src/view_models/track_metadata_grid.rs | `library_value_route_item_display` gives the key and the identifiers of each row. |
| ID3 frame reader | extend | src/audio_tags.rs | `extended_text_value` removes the NUL terminator. |

## Changes

1. `Cargo.toml`: the `musicindex-value` dependency at musicindex-lib `5e35cf5`.
2. `src/assets/tokens.json`: the full copy from musicindex-assets `d979ef3`.
3. `src/ui/tokens.rs`, `src/ui/style.rs` and `src/ui/theme_profiles.rs`: the share colors and the value routes sizes.
4. `src/view_models/value_routes.rs`: the bar segments, the names line, the groups, the rows, the open lines and the fee line.
5. `src/view_models/track_metadata_grid.rs`: `value_routes_panels`. The text tree helpers are deleted.
6. `src/ui/shells/library/track_detail_value_routes.rs`: the render of the panel.
7. `src/ui/shells/library/track_detail_metadata_cells.rs` and `track_detail_metadata_grid.rs`: the panel row under the compare row.
8. `src/ui/shells/library/track_detail_metadata_values.rs`: the open value routes cell keeps its summary. The text tree is deleted.
9. `src/audio_tags.rs`: the NUL terminator of a `TXXX` value is removed.
10. `tests/architecture_tests.rs`: the guard rows of the deleted helpers are removed. The guard of recipient names cites musicindex ADR 0082.

## Mechanical Criteria

Each criterion is proved by a test.

- M1. The view model gives the bar segments in the sequence of the website, with the named segments, the fee segments and one "other" segment (tests `r_routes_1_*`).
- M2. The segment widths fill the bar, and no segment is narrower than the minimum (test `r_routes_1_widths_fill_the_bar_and_keep_the_minimum`).
- M3. The names line gives the three largest named segments and "N more X %" (tests `r_routes_2_*`).
- M4. The groups are "RECIPIENTS" and "FEES", each with its empty text, and the fee line names each fee (tests `r_routes_3_*`).
- M5. A row gives the name, the meter fill, the percentage text and the short address (tests `r_routes_4_*`).
- M6. The open content gives the labeled lines "Type", "Shares", "Custom key" and "Custom value", and no line gives the percentage or "split" (test `r_routes_5_open_content_gives_labeled_lines`).
- M7. The open address has a break point only after "@" and after a dot (test `r_routes_5_open_address_breaks_after_at_and_dots`).
- M8. The grid view model gives one panel for each open value routes cell, in the sequence of the columns (test `r_routes_8_open_value_routes_panels_follow_the_open_cells`).
- M9. The share colors and the value routes sizes agree with the shared tokens (tests `share_color_matches_the_shared_tokens` and `value_routes_sizes_match_the_shared_tokens`).
- M10. A `TXXX` value with a NUL terminator gives a value routes JSON that parses (test `extended_text_value_drops_the_nul_terminator_of_mutagen`).

## Visual Criteria

A person judges each of these criteria. The operator passed V1 to V6 on 2026-11-17.

- V1. The summary bar has round ends, a gap between segments and diagonal stripes on a fee segment.
- V2. The names line is under the bar, in the center, and each part stays on one line.
- V3. The panel has rounded corners, the group headings, the separators and the fee line as the last line.
- V4. Each row has the name, the meter and the percentage on line 1, and the address on line 2. The meters of a panel are aligned.
- V5. An open row shows the full address, Copy and the labeled lines, with the labels in one column.
- V6. The look is correct in the dark, light, high contrast dark and high contrast light themes.

## Rules That HitIt Does Not Meet

- R-ROUTES-1 item 7: a segment cannot get keyboard focus. Only the pointer shows its hover text.
- R-ROUTES-1 item 8 and R-ROUTES-2 item 6: GPUI has no accessibility tree. The view model keeps the bar label as data only.
- R-ROUTES-3 item 3: GPUI has no letter spacing, so the heading has no .06 em spacing. HitIt has no 10 px type size, so the heading uses 11 px (`FontSize::Micro`).
- R-ROUTES-8 item 1: the panel shows in a row under the compare row, not in the cell (Main Session Choices item 1).
- A node key has no "@" and no dot. Thus the open content breaks it at any character when it is wider than the line.

## Test Commands

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test architecture_tests
cargo build --bin hitit
```

## Do Not Commit

A delegate does not commit. The main session reads the diff, runs the checks and makes the commit.
