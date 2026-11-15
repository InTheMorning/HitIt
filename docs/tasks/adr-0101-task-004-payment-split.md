# ADR 0101 Task 004: The Payment Split Section

Status: Blocked - 2026-11-15. This packet waits for the operator push of musicindex-lib `0fadde6`. Start when `git ls-remote https://github.com/InTheMorning/musicindex-lib` shows that commit, or when the operator says that the push is done. It also starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks.

## Goal

A track page and a release page show the payment split of the item in one section. The section has one row for each recipient and one summary bar in the share color. The text gives each value, and the bar only supports the text.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 2, "Value", owns this packet. Read Decisions 2 and 12 before the first edit.
- musicindex ADR 0069 R-VALUE-1 and R-VALUE-2 (`../musicindex/docs/adr/0069-the-interface-rules-and-the-shell-of-the-three-apps.md` lines 59 to 77, at musicindex `785ae28`):
  - R-VALUE-1 item 1 is replaced. Use R-VALUE-3 of musicindex ADR 0074.
  - R-VALUE-1 item 2: a fee shows apart from the other recipients, with the word "fee".
  - R-VALUE-1 item 3: the total is not always 100. The raw share is also available, for example in a tooltip.
  - R-VALUE-2 item 1: one row for each recipient: the name, the percentage, and the address in a monospace font.
  - R-VALUE-2 items 2 and 3: one summary bar. Each segment has the width of its percentage. The bar only supports the text.
  - R-VALUE-2 item 4: the bar uses `share`, `#2d7bff` dark and `#1a56c0` light. Its track uses `share-track`, `rgba(45, 123, 255, .2)` dark and `rgba(26, 86, 192, .16)` light.
- musicindex ADR 0074 (`../musicindex/docs/adr/0074-the-split-code-and-the-fee-of-a-split.md`, at musicindex `ff77cc9`):
  - R-VALUE-3 replaces R-VALUE-1 item 1. A fee keeps its share as its percentage, off the top. Each other recipient gets its part of the rest. Items 2 to 7 give the edge cases.
  - R-VALUE-4 puts the split code in the musicindex-lib folder `value/`, the crate `musicindex-value`.
- [ADR 0099](../adr/0099-the-rss-path-keeps-only-lightning-payment-routes.md) keeps only Lightning routes on the RSS path. Read its Decision before the first edit.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) Decision I: RSS is the source of truth.
- musicindex ADR 0023 R-DATE-1 item 4: a percentage has one decimal place.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `PaymentRoute` (`src/api.rs:479`) has `recipient_name`, `route_type`, `split`, `fee`, `address`, `custom_key` and `custom_value`. Examined.
- `stored_route` (`src/db/payment_routes.rs:249`) reads the route of a track, and the route of its feed when the track has none. Examined.
- The feed and track read models carry `payment_routes` (`src/views.rs:143` and `:196`). The conversions from the API keep the routes (`:436`, `:653`). The conversions from the database give an empty list (`:514` and `:716`). Examined at `:514`.
- `TrackDetailSlots::value_routes` (`src/view_models/track_detail.rs:771`) is a list of `ValueRouteItem` with `recipient` and `split` text. Examined.
- `value_route_split_label` (`src/view_models/track_metadata_grid.rs:458`) gives the raw split with "%" and no decimal rule. Examined.
- The inspection view shows the routes as rows of the metadata grid (`src/ui/shells/library/track_detail_metadata_values.rs:273`).
- `TrackDetailSurface` (`src/ui/composites/track_detail_surface.rs:28`) has no payment section. The release page draws its panels through `src/ui/shells/entity.rs:104`.
- `src/assets/tokens.json:26` and `:49` give `share` and `share-track`. No `SemanticColor` gives them.
- This is an inference: no code colors a recipient by `EntityKind`.

The crate `musicindex-value` at musicindex-lib `0fadde6c993fe5aec87a7627c6149c1cc9ccc9a0` gives the split arithmetic (orchestrator answer of 2026-11-15):

- `Recipient { share: Option<f64>, fee: bool }`.
- `percentages(&[Recipient]) -> Vec<Option<f64>>` gives the raw percentage of each recipient. HitIt rounds it to one decimal place (R-VALUE-3 item 1).
- `parse_share(&str) -> Option<f64>` parses the split text, as Studio does.

Operator choice, 2026-11-15 (musicindex-research `0958285`): the Studio rule is correct. Shares 100, 100 and a fee of 10 give 45 %, 45 % and 10 %. Do not copy the arithmetic into HitIt.

Main session choice, 2026-11-15: the section heading is "Value Split". The heading of a section of musicindex.org is not known to this session. The packet reports the heading, and the operator decides at the walk.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Stored route read | reuse | `src/db/payment_routes.rs` | `stored_route` gives the route of a track with the feed fallback. |
| Read models | extend | `src/views.rs` | The database conversions give an empty list. Fill `payment_routes` from the stored columns. |
| Track view model slot | extend | `src/view_models/track_detail.rs` | `value_routes` exists. Replace the text item with the split view model of step 2. |
| Split arithmetic | reuse | `Cargo.toml` | Pin the crate `musicindex-value` at musicindex-lib `0fadde6c993fe5aec87a7627c6149c1cc9ccc9a0` (R-VALUE-4). It gives `percentages` and `parse_share`. |
| Split view model | new | `src/view_models/payment_split.rs` | Searched `src/view_models/` and `src/ui/composites/` in HitIt. No code gives the rows or the bar of a split. The view model calls `musicindex_value::percentages` for each percentage. |
| Split section | new | `src/ui/composites/payment_split.rs` | Searched `src/ui/composites/` in HitIt. No composite draws recipient rows with a bar. |
| Color tokens | extend | `src/ui/tokens.rs` | `SemanticColor` gives each color. Add `Share` and `ShareTrack`. |
| Track page surface | extend | `src/ui/composites/track_detail_surface.rs` | The track page draws its sections there. |
| Release page panels | extend | `src/ui/shells/entity.rs` | The release page draws its panels there. |

## Required Changes

0. **The pin.** Add `musicindex-value = { git = "https://github.com/InTheMorning/musicindex-lib", rev = "0fadde6c993fe5aec87a7627c6149c1cc9ccc9a0" }` to `Cargo.toml`. Move `musicindex-search` to the same rev (musicindex ADR 0049 R-QUERY-4 item 1). Stop if the search tests do not pass at that rev.
1. **The stored routes.** The database conversions of `src/views.rs` fill `payment_routes` of a feed and a track from their stored columns. A track with no route of its own gets the route of its feed, as `stored_route` does.
2. **The split view model.** In `src/view_models/payment_split.rs`, write a pure function from a list of `PaymentRoute` to a split view model. It parses each split with `parse_share` and gets each percentage from `percentages`. It gives one segment of the bar for each row, in the order of the rows. A recipient with no percentage from `percentages` gives no percentage text and no segment. The view model holds no color and no renderer type.
   - Each recipient gets one row with the name, the percentage text with one decimal place, the raw share text, the address and the fee flag.
3. **The color tokens.** Add `Share` and `ShareTrack` to `SemanticColor`, with the values of `share` and `share-track` in each theme. A high-contrast profile uses `Share` for the bar and an opaque track. A test compares the values with `src/assets/tokens.json`.
4. **The section.** The composite draws the heading, the bar and the rows. The address uses the monospace font of ADR 0101 Decision 3. The raw share is the tooltip of the percentage. A fee row is last, with the word "fee".
5. **The pages.** The track page shows the section of its track. The release page shows the section of its feed. A page with no route shows no section. The Index pages use the routes of the API read model.
6. **The inspection view.** The inspection view keeps its rows. Its split text uses the formatter of step 2, so that one function formats a share.

## Do Not Change

- No change of the stored route, the migration or the route for a file write.
- No `value_time_splits`. The time splits of a track are a different packet.
- No entity color on a recipient (ADR 0101 Decision 2).
- No payment action. The section only shows the split.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test of the split view model gives 66.7 % and 33.3 % for splits 2 and 1, and gives the raw shares.
- M2: a test gives a fee row with its share as its percentage and the word "fee". Each other row gets its part of the rest.
- M3: a test proves that a recipient with no percentage from `percentages` gives no percentage text and no segment.
- M4: a test proves that the segments have the order and the widths of the rows.
- M5: a test proves that `Share` and `ShareTrack` equal `share` and `share-track` of `src/assets/tokens.json` in each theme.
- M6: a test of the read model proves that a Library track with no route of its own gets the route of its feed.
- M7: a test proves that a track page and a release page with routes give the section, and that a page with no route gives none.
- M8: the fit tests of ADR 0097 pass.
- M9: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M, at 390 px and at 940 px:

- V1: a track page shows "Value Split" with one row for each recipient and a bar above the rows.
- V2: the text gives each percentage. The bar agrees with the text.
- V3: the address is monospace and does not push the row past the page.
- V4: a fee row shows "fee" and is last.

## Test Commands

- `cargo test --lib view_models::payment_split`
- `cargo test --lib ui::tokens`
- `cargo test --lib views`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M7 by name. Give the heading that the section uses.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the HitIt code needs arithmetic on a share other than the rounding to one decimal place.
- Stop if an edge case of R-VALUE-3 items 2 to 7 is not given by `percentages`.
- Stop if a stored route has a recipient with no name and no address.
- Stop if the fill of step 1 changes a result of a tag write test.
- Stop if the contrast of `Share` on the background is below 3:1 in a theme.
