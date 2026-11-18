# ADR 0097 Task 008: The List Page Title Fits At 390 px

Status: Implemented - operator visual checks open, 2026-11-18. The orchestrator asked for this packet on 2026-11-18, with the approval of the operator.

## Goal

At a window width of 390 px, the title of a list page does not break inside a word. The fit test finds a break of the title.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) and musicindex ADR 0065 R-FIT-3: a break inside a word is a fault at 390 px and at 940 px.
- [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md), Non-Goals: a fault that a phase finds gets its own task.
- [ADR 0063](../adr/0063-show-dashboard-layout.md), Column Text Does Not Truncate.

## Defect

The [second agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-second.md), defect 3: at 390 px the Releases title shows "Relea" and "ses" on two lines. Screenshot: `docs/reviews/screenshots/agent-walk-2026-11-18-second/tiles-390-title-breaks.png`.

## Cause

The main session read this in the code on 2026-11-18:

- `list_page_head` (`src/ui/composites/list_page_head.rs`) puts the title and the trailing controls (sort, views, Select, actions) in one flex row. The title has `flex_1()` and `min_w_0()`, and the trailing controls do not shrink. At 390 px the title gets the remaining width, which is less than the width of one word.
- The title is a plain text child of a `div`. It records no fit probe, so `adr_0097_the_list_pages_fit_at_the_two_widths` cannot see the break.

## Change

- The title row wraps. When the title and the trailing controls do not fit on one line, the controls go to a second line below the title. The minimum width of the title is the width of its longest word.
- The title records a fit probe, as a `Label` or with the probe of `src/ui/primitives/fit_probe.rs`, so the fit test sees a break inside a word.
- A long query title can still wrap between words (ADR 0063). Do not call `truncate()` on the title.
- Do not change the tile grid. The one column of tiles at 390 px is not in this packet.

## Guard Change

The search results case of `adr_0097_the_list_pages_fit_at_the_two_widths` used the fixture `LONG_TITLE` as the query. It is a word of 40 letters with no space. With the title probe, that query breaks inside the word at 390 px, and the test reports a fault. The case now uses the query "Search results for a long query".

A query of one long word can still break inside the word. No rule says if a query title can break anywhere (`Label::break_anywhere`). This is open for the orchestrator.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| List page head | extend | src/ui/composites/list_page_head.rs | The shared owner of the title row of each list page. |
| Fit probe | reuse | src/ui/primitives/fit_probe.rs | It finds a break inside a word. |
| Label | reuse | src/ui/primitives/label.rs | A text with a fit probe. |
| List page fit test | reuse | src/app/fit_check/list_pages.rs | `adr_0097_the_list_pages_fit_at_the_two_widths` draws each list page at both widths. |
| Fit fixtures | reuse | src/library/fit_fixtures.rs | The fixture rows of the list pages. |

## Mechanical Criteria

- Add the fit probe of the title first. Then `adr_0097_the_list_pages_fit_at_the_two_widths` reports the break of the Releases title at 390 px, before the change of the title row. The implementer records that failure in the report.
- After the change, `adr_0097_the_list_pages_fit_at_the_two_widths` finds no fault.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: in Light and Dark at 390 px, examine the titles of Releases, Artists, My playlists, Public playlists and the Inbox. Each title is on one line with no break inside a word.
- V2: at 390 px the sort, the view buttons and Select are on a second line below the title when they do not fit beside it. Each control shows fully.
- V3: at 940 px and at 1440 px the title and the controls are on one line, as before.
- Incorrect: a title breaks inside a word, a control is cut, or the head is higher at 940 px than before.

## Observation Not In This Packet

At 390 px, the Tiles view of Releases shows one column of tiles in the left half of the window. Two tiles of 176 px, the gap, the padding and the scrollbar space are wider than the page. The second walk recorded it with defect 3. No ADR states the column count at 390 px, so this packet does not change it.
