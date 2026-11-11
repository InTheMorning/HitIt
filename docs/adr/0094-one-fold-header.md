# ADR 0094: One Fold Header

## Status

Accepted - 2026-11-10. The operator accepted musicindex ADR 0060 R-FOLD-1 and R-FOLD-2 on 2026-11-10 (musicindex `1959db9`, updated in `1e0bb12`). The orchestrator session relayed the rules and the operator decisions. This record applies them to v4vmm. Decisions 3 and 5 are choices of the planning session. This record changes no code.

Applies: musicindex ADR 0060 R-FOLD-1 and R-FOLD-2. Uses: musicindex ADR 0024 R-TARGET-1 for the target size.

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
2. **The closed header looks like a list row (R-FOLD-1).** It has the look of `ListRow`. The name is in Body text and the Label color, not in capitals. The count follows the name. The chevron is at the end and points right. The header has no "show" or "hide" word.
3. **The count is a field of its own.** The view model gives the name and the count separately. A metadata group gives its unused count, for example "3 unused", and gives no count when the number is 0. The Description panels and "How to Search" give no count.
4. **The open header (R-FOLD-2).** When the fold opens, the header keeps the same text, size and place. Only the chevron turns, and it points down.
5. **One control.** The full header is one control with the target size of musicindex ADR 0024 R-TARGET-1. It is one Tab stop. Enter and Space open and close the fold. They use the activation route of the shared Button primitive, with one activation for each key press (ADR 0069 amendment of 2026-09-13). A pointer click on any part of the header does the same.
6. **The open condition.** The view model gives the open condition and an a11y label that names it, for example "Description, closed". GPUI 0.2.x does not send the label to assistive technology (R-FOLD-1 item 4). When GPUI gets a label sink, the composite sends it with no change to a view model.
7. **The users.** The metadata groups, the two Description panels and "How to Search" use `FoldHeader`. The "Show more" button of a Description panel stays below the text, outside the header.
8. **Not fold headers.** The operator decided on 2026-11-10 that the singles group row and the expandable value cells are not fold headers. They are tree rows, and R-FOLD-1 does not apply (R-FOLD-1 item 5). The keys of the singles group chevron wait for an operator answer, which the orchestrator session has. A change of this record adds them.

## Invariants

1. Each fold header of v4vmm is a `FoldHeader`.
2. A fold header is one control and one Tab stop.
3. Only the chevron changes when a fold opens or closes.

## How This Decision Is Checked

Mechanical:

- The view-model tests give the name, the count, the open condition and the a11y label of each fold header of Decision 7.
- A composite test asserts that the open and the closed header have the same text and the same height.
- A key test asserts that Tab reaches the header, and that Enter and Space each toggle the fold one time.
- The guard `adr_0094_one_fold_header` forbids `SectionHeader::disclosure`, `DisclosureGroup` and a second fold header composite.

Visual, for the operator: open a track page and a release page in the light and the dark theme, at M and at XL. Open and close the Description panel and a metadata group. Open and close "How to Search". Each header looks like a list row, and only the chevron changes.

## Delivery

[ADR 0094 task 001](../tasks/adr-0094-task-001-fold-header-composite.md) delivers Decisions 1 to 7. It is Ready. The operator decides the priority.

## Alternatives Considered

musicindex ADR 0060 gives the shared alternatives.

### Keep `SectionHeader::disclosure` and change its look

The section heading and the fold header then share one primitive with two roles. A later change of the heading changes the fold too.
