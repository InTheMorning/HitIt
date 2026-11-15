# ADR 0101 Task 004: The Payment Split Section

Status: Blocked - 2026-11-15. The operator pushed musicindex-lib `5e35cf5` on 2026-11-15. It holds the crate `musicindex-value` under musicindex ADRs 0075 and 0076. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks.

## Goal

A track page and a release page show the payment split of the item in one section. The section has one row for each recipient and one summary bar in the share color. The text gives each value, and the bar only supports the text.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 2, "Value", owns this packet. Read Decisions 2 and 12 before the first edit.
- musicindex ADR 0069 R-VALUE-1 and R-VALUE-2 (`../musicindex/docs/adr/0069-the-interface-rules-and-the-shell-of-the-three-apps.md` lines 59 to 77, at musicindex `785ae28`):
  - R-VALUE-1 item 1 is replaced. Use R-VALUE-5 of musicindex ADR 0075.
  - R-VALUE-1 item 2: a fee shows apart from the other recipients, with the word "fee".
  - R-VALUE-1 item 3: the total is not always 100. The raw share is also available, for example in a tooltip.
  - R-VALUE-2 item 1: one row for each recipient: the name, the percentage, and the address in a monospace font.
  - R-VALUE-2 items 2 and 3: one summary bar. Each segment has the width of its percentage. The bar only supports the text.
  - R-VALUE-2 item 4: the bar uses `share`, `#2d7bff` dark and `#1a56c0` light. Its track uses `share-track`, `rgba(45, 123, 255, .2)` dark and `rgba(26, 86, 192, .16)` light.
- musicindex ADR 0075 (musicindex `bf58554`) supersedes ADR 0074. R-VALUE-5 (orchestrator text of 2026-11-15):
  1. A percentage shows with one decimal place.
  2. Each recipient with a valid share n gets n / S × 100. S is the total of the valid shares of all recipients, fee recipients included. The fee flag does not change the percentage.
  3. A share that is not a number, below 0 or not finite gives no percentage. It counts as 0 in S.
  4. When S is 0, no recipient gets a percentage.
  5. A block with exactly one recipient gives it 100 %, whatever its share.
  6. Examples: 100, 100 and a fee of 10 give 47.6 %, 47.6 % and 4.8 %. A fee of 10 alone gives 100.0 %.
  7. When a block has a fee recipient, a line names the fee. The line says that the fee comes out of the payment, not on top of it.
- musicindex ADR 0076 (musicindex `a08dc01`) R-ADDRESS-1 to R-ADDRESS-3 (orchestrator text of 2026-11-15):
  - A row shows a node key or an npub in its short form. `musicindex_value::short_address(text: &str) -> String` gives it.
  - A custom value, an LNURL address and an address with "@" stay in full.
  - The hover text or the tooltip and the Copy action give the full address.
  - R-ADDRESS-3 item 4: an app has no copy of the function.
- musicindex ADR 0075 R-VALUE-4 puts the split code in the musicindex-lib folder `value/`, the crate `musicindex-value`.
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

The crate `musicindex-value` gives the split arithmetic (orchestrator answer of 2026-11-15). Its API is `percentages(shares: &[Option<f64>]) -> Vec<Option<f64>>`, with no recipient type. It gives the raw percentage of each share. HitIt rounds it to one decimal place (R-VALUE-5 item 1). The fee flag stays in HitIt, for the word "fee" and the line of R-VALUE-5 item 7. Do not copy the arithmetic into HitIt.

Main session choice, 2026-11-15: the section heading is "Value Split". The heading of a section of musicindex.org is not known to this session. The packet reports the heading, and the operator decides at the walk.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Stored route read | reuse | `src/db/payment_routes.rs` | `stored_route` gives the route of a track with the feed fallback. |
| Read models | extend | `src/views.rs` | The database conversions give an empty list. Fill `payment_routes` from the stored columns. |
| Track view model slot | extend | `src/view_models/track_detail.rs` | `value_routes` exists. Replace the text item with the split view model of step 2. |
| Split arithmetic | reuse | `Cargo.toml` | Pin the crate `musicindex-value` at musicindex-lib `5e35cf5c39a8ff68c64267156d0a7165c43f1826` (ADR 0075 R-VALUE-4). It gives `percentages` and `short_address`. |
| Split view model | new | `src/view_models/payment_split.rs` | Searched `src/view_models/` and `src/ui/composites/` in HitIt. No code gives the rows or the bar of a split. The view model calls `musicindex_value::percentages` for each percentage. |
| Split section | new | `src/ui/composites/payment_split.rs` | Searched `src/ui/composites/` in HitIt. No composite draws recipient rows with a bar. |
| Color tokens | extend | `src/ui/tokens.rs` | `SemanticColor` gives each color. Add `Share` and `ShareTrack`. |
| Track page surface | extend | `src/ui/composites/track_detail_surface.rs` | The track page draws its sections there. |
| Release page panels | extend | `src/ui/shells/entity.rs` | The release page draws its panels there. |

## Required Changes

0. **The pin.** Add `musicindex-value = { git = "https://github.com/InTheMorning/musicindex-lib", rev = "5e35cf5c39a8ff68c64267156d0a7165c43f1826" }` to `Cargo.toml`. Move `musicindex-search` to the same rev (musicindex ADR 0049 R-QUERY-4 item 1). Stop if the search tests do not pass at that rev.
1. **The stored routes.** The database conversions of `src/views.rs` fill `payment_routes` of a feed and a track from their stored columns. A track with no route of its own gets the route of its feed, as `stored_route` does.
2. **The split view model.** In `src/view_models/payment_split.rs`, write a pure function from a list of `PaymentRoute` to a split view model. It parses each split text to `Option<f64>`, with a parse function of the crate if it has one. It gets each percentage from `percentages`. It gives one segment of the bar for each row, in the order of the rows. A recipient with no percentage from `percentages` gives no percentage text and no segment.
   - The view model holds no color and no renderer type.
   - Each recipient gets one row with the name, the percentage text and the raw share text. The percentage text has one decimal place.
   - The row also has the display address, the full address and the fee flag. The display address of a node key or an npub comes from `short_address`. A custom value, an LNURL address and an address with "@" stay in full.
3. **The color tokens.** Add `Share` and `ShareTrack` to `SemanticColor`, with the values of `share` and `share-track` in each theme. A high-contrast profile uses `Share` for the bar and an opaque track. A test compares the values with `src/assets/tokens.json`.
4. **The section.** The composite draws the heading, the bar and the rows. The address uses the monospace font of ADR 0101 Decision 3. The tooltip of the address and its Copy action give the full address. The raw share is the tooltip of the percentage.
   - A fee row is last, with the word "fee". A block with a fee shows the line of R-VALUE-5 item 7.
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
- M2: a test gives 47.6 %, 47.6 % and 4.8 % for shares 100, 100 and a fee of 10. The fee row has the word "fee", and the view model gives the fee line of R-VALUE-5 item 7.
- M2a: a test gives 100.0 % for one recipient, also for a fee of 10 alone.
- M3: a test proves that a recipient with no percentage from `percentages` gives no percentage text and no segment.
- M4: a test proves that the segments have the order and the widths of the rows.
- M5: a test proves that `Share` and `ShareTrack` equal `share` and `share-track` of `src/assets/tokens.json` in each theme.
- M6: a test of the read model proves that a Library track with no route of its own gets the route of its feed.
- M7: a test proves that a track page and a release page with routes give the section, and that a page with no route gives none.
- M8: the fit tests of ADR 0097 pass.
- M8a: a test proves that a node key row and an npub row show the text of `musicindex_value::short_address`, and that the Copy payload is the full address.
- M8b: a test proves that an LNURL address, an address with "@" and a custom value stay in full.
- M8c: a guard proves that no HitIt file defines a short address function (R-ADDRESS-3 item 4). Its message names musicindex ADR 0076 and the fix: call `musicindex_value::short_address`.
- M9: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M, at 390 px and at 940 px:

- V1: a track page shows "Value Split" with one row for each recipient and a bar above the rows.
- V2: the text gives each percentage. The bar agrees with the text.
- V3: the address is monospace and does not push the row past the page.
- V3a: the short form of a node key has the same text at 390 px and at 940 px. The tooltip shows the full address.
- V4: a fee row shows "fee" and is last. The fee line says that the fee comes out of the payment.

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

## Later Items

These places show a node key or an npub in full. This packet does not change them. Orchestrator answer, 2026-11-15:

- `src/view_models/entity_detail.rs:1087`: the "Nostr" row of an entity detail gives the full npub. R-ADDRESS-1 applies, because it is a row. A later packet gives it the short form, and the tooltip and Copy give the full npub.
- `src/ui/shells/library/track_detail_metadata_values.rs:329`: the inspection tree of a value route gives the `address` child field in the Index context (`src/view_models/track_metadata_grid.rs:1336`). Decided: no change. Inspection shows raw data (ADR 0101 Decision 8). R-ADDRESS-1 item 6 needs a location where the full address is always available, and inspection is that location.

## Stop And Ask

- Stop if the HitIt code needs arithmetic on a share other than the rounding to one decimal place.
- Stop if an edge case of R-VALUE-5 items 2 to 5 is not given by `percentages`.
- Stop if a stored route has a recipient with no name and no address.
- Stop if the fill of step 1 changes a result of a tag write test.
- Stop if the contrast of `Share` on the background is below 3:1 in a theme.
