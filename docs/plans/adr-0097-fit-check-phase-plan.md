# ADR 0097 Fit Check Phase Plan

## Status

Active - 2026-11-13. This plan is advisory. It states no rule. [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) and [musicindex ADR 0065](../../../musicindex/docs/adr/0065-each-app-checks-its-fit-at-two-widths.md) own the rules.

## Goal

GPUI layout tests run at 390 px and at 940 px on each page, sheet and dialog of HitIt (musicindex ADR 0065 R-FIT-2 item 4). They find each fault of R-FIT-3: a break inside a word, an edge cut, an overflow and an overlap.

## Non-Goals

- A check at other widths. musicindex ADR 0065 does not give one.
- A change to a layout. A fault that a phase finds gets its own task, unless the fix is in a token or a primitive that the phase also changes.

## Sequence

| Phase | Content | State |
|---|---|---|
| 1A | The `gpui-pre` patch in the `InTheMorning/gpui-kit` fork: a public `TestAppContext` constructor that takes a `PlatformTextSystem`. The operator pushes the branch. `Cargo.toml` pins the rev | Done - 2026-11-13 |
| 1B | `WINDOW_MIN_WIDTH` of 390 px and `window_min_size` (ADR 0097 Decision 1). The fit probe in `Label` and `Button`, `Label::break_anywhere`, the fault finder and its unit tests (Decision 4) | Done - 2026-11-13 |
| 1C | The shell test mounts `TopApp` with an in-memory database at 390 px and at 940 px. It examines the start page, the tab bar or the sidebar, and the Settings navigation. It mounts the list bar at the content width of each window width | Done - 2026-11-13. The test found no fault |
| 2 | The list pages of the sidebar destinations with fixture rows. These are Releases, Artists, My playlists, Public playlists, Live, Search and the Inbox | Done - 2026-11-14. [Task 002](../tasks/adr-0097-task-002-list-pages-fit.md) found a break inside a word at 390 px on the Inbox, Search and the search results. [Task 007](../tasks/adr-0097-task-007-list-rows-truncate-long-names.md) corrects it |
| 3 | The item pages with fixture rows. These are the release, track, feed, name level, playlist, singles group and window group pages | Open - [task 003](../tasks/adr-0097-task-003-item-pages-fit.md) is Ready |
| 4 | The Settings groups and the maintenance pages, the Show page, the startup report and the recovery pages | Open - [task 004](../tasks/adr-0097-task-004-settings-show-and-recovery-fit.md) is Ready |
| 5 | The sheets, the dialogs, the menus and the popovers. These are Inspect sources, each confirmation dialog, the playlist popover, the context menus and the search field list | Open - [task 005](../tasks/adr-0097-task-005-sheets-dialogs-and-menus-fit.md) is Ready |
| 6 | The other text primitives and composites that draw a text with `div().child`. Each one records a probe, or moves to `Label` | Open - [task 006](../tasks/adr-0097-task-006-each-text-and-control-records-a-probe.md) is Ready |

Task 002 makes the fixture adapter and moves the harness, so tasks 003 to 005 wait for it. Task 004 makes the fault finder ignore the horizontal scroll area of `LogFrame`. Task 005 gives each probe a layer, so that an open overlay does not overlap the page below it.

ADR 0093 task 001 replaces the Settings group menu with a tab row. The change of that task also makes the phase 1C test examine the tab row.

## Risks

- A text that does not go through a probed primitive has no check. Phase 6 adds the check. Until then, a phase that adds a page also examines the text of that page.
- A horizontal scroll area shows only part of its content. If a page has one, the fault finder must ignore that area before that page gets a check.
- The phase 1C test mounts the list bar alone in a box of the content width. The page padding is not in the box, so the bar on a page has less width.
- Only `Button` records a control probe. A control of a different primitive has no overlap check until phase 6.
- Fixture rows with short names can hide a fault. Each fixture uses long names, for example a name of 40 characters with no space.
