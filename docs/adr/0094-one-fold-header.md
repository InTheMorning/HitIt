# ADR 0094: One Fold Header

## Status

Accepted - 2026-11-10. The operator accepted musicindex ADR 0060 R-FOLD-1 and R-FOLD-2 on 2026-11-10 (musicindex `1959db9`, updated in `1e0bb12`). The orchestrator session relayed the rules and the operator decisions. This record applies them to v4vmm. Decisions 3 and 5 are choices of the planning session. This record changes no code.

On 2026-11-10 the operator also accepted musicindex `5123573`. It changes R-FOLD-1 items 1, 3 and 5. It adds R-FOLD-3 and musicindex ADR 0061.

Applies: musicindex ADR 0060 R-FOLD-1, R-FOLD-2 and R-FOLD-3. Uses: musicindex ADR 0024 R-TARGET-1 for the target size, and musicindex ADR 0061 R-LISTKEY-1 for the keys of the singles group row.

## Context

On 2026-11-10 v4vmm has these fold controls:

| Control | File | Look and work |
| --- | --- | --- |
| `DisclosureGroup` with `SectionHeader::disclosure` | `src/ui/composites/disclosure_group.rs`, `src/ui/primitives/section_header.rs` | A chevron at the start, the name in bold Micro text in the SecondaryLabel color, then the word "show" or "hide". A click only: no Tab stop, no Enter and no Space. The a11y label is discarded |
| The metadata groups of the track metadata grid | `src/ui/composites/track_metadata_grid.rs` | `DisclosureGroup`. The count is in the name, for example "Credits (3 unused)" |
| The Description panel of a release page and a track page | `src/ui/shells/release_page.rs`, `src/ui/shells/track_page.rs` | `DisclosureTextPanel`, which uses `DisclosureGroup`. A separate "Show more" button |
| "How to Search" on the search page | `src/ui/shells/library/search_page_list.rs` | A `ListRow` with no chevron and no count |
| The singles group row | `src/ui/shells/library/destination_lists.rs` | A chevron at the start opens the group in place. Right and Left open and close the focused group row |
| The expandable value cells of the track metadata | `src/ui/shells/library/track_detail_metadata_values.rs` | A chevron at the start of a value cell |

GPUI 0.2.x has no sink for the label of a row (`src/ui/composites/list_row.rs`). Thus no v4vmm control can give its open condition to assistive technology on 2026-11-10.

## Decision

1. **One fold header composite.** A new composite `FoldHeader` replaces `SectionHeader::disclosure` and `DisclosureGroup`. Each fold header of v4vmm uses it. `SectionHeader` keeps its heading role only.
2. **The closed header looks like a list row (R-FOLD-1 item 1).** It has the look of `ListRow`. The header starts with the chevron, which points right (R-FOLD-3 item 1). Then it shows the name in Body text and the Label color, not in capitals, and then the count. The header has no "show" or "hide" word. v4vmm draws the chevron with its own icon (R-FOLD-3 item 6).
3. **The count is a field of its own.** The view model gives the name and the count separately. A metadata group gives its unused count, for example "3 unused", and gives no count when the number is 0. The Description panels and "How to Search" give no count.
4. **The open header (R-FOLD-2 items 1 and 2).** When the fold opens, the header keeps the same text, size and place. Only the chevron turns, and it points down (R-FOLD-3 item 1).
5. **One control (R-FOLD-1 item 3).** The full header is one control with the target size of musicindex ADR 0024 R-TARGET-1. It has its own Tab stop. Enter and Space open and close the fold. They use the activation route of the shared Button primitive, with one activation for each key press (ADR 0069 amendment of 2026-09-13). A pointer click on any part of the header does the same.
6. **The open condition.** The view model gives the open condition and an a11y label that names it, for example "Description, closed". GPUI 0.2.x does not send the label to assistive technology (R-FOLD-1 item 4). When GPUI gets a label sink, the composite sends it with no change to a view model.
7. **The users.** The metadata groups, the two Description panels and "How to Search" use `FoldHeader`. The "Show more" button of a Description panel stays below the text, outside the header.
8. **Not fold headers (R-FOLD-1 item 5).** R-FOLD-1 applies only to a fold header. The singles group row is a tree row. An expandable value cell of the track metadata shows more text, and it is none of the three row types. R-FOLD-1 does not apply to them. A tree row has no `›` at the end (R-FOLD-3 item 5).
9. **The keys of the singles group row.** The operator decided on 2026-11-10 that the singles group row keeps the keys of a tree row (R-FOLD-1 item 5, musicindex ADR 0061 R-LISTKEY-1):
    - Right opens the focused group row, and Left closes it (R-LISTKEY-1 item 3). These keys exist in `src/app/keyboard.rs`.
    - The chevron gets no Tab stop (R-LISTKEY-1 item 6). The list stays one Tab stop (R-LISTKEY-1 item 1, ADR 0092 Decision 1), and ADR 0090 Invariants need no exception.
    - Enter on the row keeps opening the group page (R-LISTKEY-1 item 4).
    - The a11y label of the row gives the open condition, for example "Open" or "Closed". The view model gives it. GPUI 0.2.x has no sink for it (R-LISTKEY-2).

## Invariants

1. Each fold header of v4vmm is a `FoldHeader`.
2. A fold header is one control with its own Tab stop.
3. A fold header starts with its chevron. Only the chevron changes when a fold opens or closes.

## How This Decision Is Checked

Mechanical:

- The view-model tests give the name, the count, the open condition and the a11y label of each fold header of Decision 7.
- A composite test asserts that the open and the closed header have the same text and the same height.
- A key test asserts that Tab reaches the header, and that Enter and Space each toggle the fold one time.
- A key test asserts that Right opens and Left closes the focused singles group row on Releases, and that Tab does not stop on its chevron.
- The guard `adr_0094_one_fold_header` forbids `SectionHeader::disclosure`, `DisclosureGroup` and a second fold header composite.

Visual, for the operator: open a track page and a release page in the light and the dark theme, at M and at XL. Open and close the Description panel and a metadata group. Open and close "How to Search". Each header starts with its chevron and looks like a list row. Only the chevron changes.

## Delivery

[ADR 0094 task 001](../tasks/adr-0094-task-001-fold-header-composite.md) delivers Decisions 1 to 7. [ADR 0094 task 002](../tasks/adr-0094-task-002-singles-group-row-keys.md) delivers Decision 9. Both are Ready. The operator decides the priority.

## Alternatives Considered

musicindex ADR 0060 gives the shared alternatives.

### Keep `SectionHeader::disclosure` and change its look

The section heading and the fold header then share one primitive with two roles. A later change of the heading changes the fold too.
