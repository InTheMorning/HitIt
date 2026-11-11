# ADR 0094 Task 001: Fold Header Composite

## Status

Ready - 2026-11-10. Implementation has not started. The operator decides the priority.

This task implements Decisions 1 to 7 of [ADR 0094](../adr/0094-one-fold-header.md). The keys of the singles group chevron are not in this task. They wait for an operator answer (ADR 0094 Decision 8).

## Goal

Add one `FoldHeader` composite with the look of a list row. Move each fold header of v4vmm to it. Remove `SectionHeader::disclosure` and `DisclosureGroup`.

## Files To Inspect

- `src/ui/primitives/section_header.rs`, `src/ui/composites/disclosure_group.rs`
- `src/ui/composites/list_row.rs` (the row look, and the note about the GPUI label sink)
- `src/ui/primitives/button.rs` (the activation route for Enter and Space)
- `src/ui/composites/track_metadata_grid.rs`, `src/view_models/track_metadata_grid.rs` (`group_heading_label`, `group_heading_display`)
- `src/ui/shells/release_page.rs`, `src/ui/shells/track_page.rs` (`DisclosureTextPanel`)
- `src/ui/shells/library/search_page_list.rs` (`how_to_search_disclosure_row`)
- `tests/architecture_tests.rs`

## Scope

1. Add `src/ui/composites/fold_header.rs` with `FoldHeader` and a display type. The display has the element id, the name, an optional count, the open condition and the a11y label.
2. The header has the `ListRow` look: the name in Body text and the Label color, then the count, then the chevron at the end. The chevron points right when closed and down when open. Nothing else changes when the fold opens.
3. The header is one control with the target size of musicindex ADR 0024 R-TARGET-1. It takes one Tab stop. Enter and Space toggle the fold through the shared Button activation route. A pointer click on the header toggles the fold.
4. The view models give the display. `group_heading_label` no longer puts the count in the name. It gives "3 unused" as the count when the number is more than 0.
5. Move the metadata groups, the Description panels of the release page and the track page, and "How to Search" to `FoldHeader`. The "Show more" button of a Description panel stays below the text.
6. Remove `SectionHeader::disclosure`, `DisclosureGroup` and the `show` and `hide` words when no caller is left. Keep `DisclosureIndicator` for the tree rows of ADR 0094 Decision 8.
7. Add the guard `adr_0094_one_fold_header`. It forbids `SectionHeader::disclosure`, `DisclosureGroup` and a second fold header composite in `src/`. Its failure message names ADR 0094 and the fix:

    ```text
    ADR 0094 Decision 1: each fold header uses FoldHeader in src/ui/composites/fold_header.rs.
    Replace this header with FoldHeader and give its name, count and open condition from the view model.
    ```

## Do Not Touch

- The singles group row and its chevron, and the value cells of the track metadata. They are tree rows (ADR 0094 Decision 8).
- The section headings that do not fold.
- The tokens and the `ListRow` look.

Do not run the app as an agent. The Xvfb screenshot script of the operator is permitted.

## Mechanical Criteria

Each criterion is checked by a unit test or a guard.

- The view-model tests give the name, the count, the open condition and the a11y label of each fold header. They cover a metadata group with 0 and with 3 unused fields, each Description panel and "How to Search".
- A composite test asserts that the open and the closed header have the same text and the same height.
- A key test asserts that Tab reaches the header, and that Enter and Space each toggle the fold one time. A held key toggles it one time.
- The guard `adr_0094_one_fold_header` passes, and it fails on a copy of `SectionHeader::disclosure`.

## Visual Criteria

These criteria need a person or the agent Xvfb screenshot check. They are open until a person examines them.

- Open a track page and a release page in the light and the dark theme, at M and at XL. Each fold header looks like a list row. The name is not in capitals, and the chevron is at the end.
- When a fold opens, the header text, height and position do not change. Only the chevron turns.
- "How to Search" on the search page shows the same header.
- The keyboard focus ring shows on the header after Tab.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin v4vmm
git diff --check
```
