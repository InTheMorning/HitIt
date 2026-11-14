# ADR 0097 Task 007: The List Rows Truncate A Long Name

Status: Implemented - 2026-11-14. The orchestrating session made the change with [task 002](adr-0097-task-002-list-pages-fit.md). The visual check is open in section 18 of [pending human checks](../pending-human-checks.md).

## Goal

A long name with no space stays on one line in the rows of the Inbox, the saved searches and the search results. The fit test of task 002 then finds no fault at 390 px.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) and its [phase plan](../plans/adr-0097-fit-check-phase-plan.md), Non-Goals: a fault that a phase finds gets its own task.
- musicindex ADR 0065 R-FIT-3: a break inside a word is a fault.
- [ADR 0063](../adr/0063-show-dashboard-layout.md), Column Text Does Not Truncate: `truncate()` is not permitted on text in a flex column. It is permitted on a row item with `flex_1()`.

## Faults Found By Task 002

The test of task 002 found these faults at 390 px with names of 40 letters and no space. No fault occurred at 940 px.

| Page | Label | Location before the fix |
|---|---|---|
| Inbox | Title and second line | `src/ui/shells/library/content_list.rs`, `render_content_list_row_text` |
| Search saved searches | Name of each saved search | `src/ui/shells/library/search_page_list.rs`, the saved search row |
| Search results | Title of a result row | `src/ui/shells/search_result_rows.rs`, `result_row_text` |

The labels did not use `.truncated()`. Their containers clip with `overflow_hidden()`, but that does not stop a wrap, so the text broke inside the word.

## Decision

The orchestrating session made this decision in the limits of ADR 0063. The rows of Releases and Artists (`render_destination_list_row`) use a block container with `flex_1().min_w_0()` and a `Label` with `.truncated()`. Those rows gave no fault in the same test. The three rows use the same pattern after the fix:

- Each label of the three rows calls `.truncated()`. The saved search subtitle also truncates.
- `result_row_text` is a block row item, not a flex column. Its second line is a flex row and does not change.

No primitive, token or layout width changes.

## Guard Change

The guard `adr_0084_search_list_r10_08_feed_rows_open_the_feed_page` stopped each `.truncated()` in `src/ui/shells/search_result_rows.rs`. ADR 0084 task 010 wrote it from its plan rule "Follow ADR 0063 and the Releases list row". The task record gives no incident in search. At that time, the Releases row did not truncate.

After this task, the guard applies the rule of ADR 0063: a function of that file that truncates text must not make a flex column. The check of the header in `search_results_inspector.rs` does not change. The orchestrating session proved that the guard fails when `result_row_text` is a flex column again.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Label truncation | reuse | src/ui/primitives/label.rs | `.truncated()` gives one line with an ellipsis. |
| Destination list row text | reuse | src/ui/shells/library/destination_lists.rs | The pattern of a block row item with truncated labels. |

## Mechanical Criteria

- `adr_0097_the_list_pages_fit_at_the_two_widths` finds no fault. Done.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- At 390 px, a long name in the Inbox, in Search and in the search results shows its first words and an ellipsis. It does not show only "...". The test cannot see the ADR 0063 defect, so a person checks this.

## Risk

The ADR 0063 defect gives an ellipsis with no words. The defect occurred in a flex column. The new rows are not flex columns. If the visual check finds only "...", restore the change in that row and report it.
