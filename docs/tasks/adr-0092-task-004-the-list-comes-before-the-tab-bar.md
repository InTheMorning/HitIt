# ADR 0092 Task 004: The List Comes Before The Tab Bar

Status: Implemented - operator visual checks open, 2026-11-18. The orchestrator asked for this packet on 2026-11-18, with the approval of the operator.

## Goal

At a window width of 390 px, Tab goes from the last control of a list page header to the list. The tab bar at the bottom of the window comes after the page, its list and the status bar.

## Authority

- [ADR 0092](../adr/0092-the-keys-of-a-list-row.md): Tab reaches the list, and the list is one Tab stop.
- [ADR 0092 task 002](adr-0092-task-002-detail-list-follows-the-page-controls.md): the list comes after the controls of its page.
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md): in a narrow window, the tab bar of R-SHELL-1 item 7 has Search, Releases, Show, Inbox and More.
- [ADR 0101 task 008](adr-0101-task-008-status-bar-and-notices.md): the status bar items come after the list.

## Defect

The [second agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-second.md) found defect 2 in the Rows view of Releases at 390 px. Tab goes from "In Library" to "Search" in the tab bar. Screenshot: `docs/reviews/screenshots/agent-walk-2026-11-18-second/rows-390-tab-skips-list.png`.

## Cause

The main session read this in the code on 2026-11-18:

- `Button::new` (`src/ui/primitives/button.rs`) gives each button Tab index 0.
- `render_tab_bar` (`src/ui/composites/tab_bar.rs`) does not change the Tab index of a tab or of More.
- The list column focus handle has Tab index 1 (`LibraryApp::new` and `render_list_column` in `src/library/app_impl.rs`).
- GPUI sorts the Tab stops by index first and then by insertion order. Thus each tab of the tab bar comes before the list.

The list is a Tab stop at 390 px. It comes after More, not after "In Library". This is an inference from the code. The test of this packet proves it.

## Change

The orchestrating session made this choice in the limits of ADR 0092. The tab bar is at the bottom of the window, so it is last in the Tab sequence.

- Give each tab and the More trigger of `render_tab_bar` a Tab index after the list and the status bar items. An example is index 2.
- Do not change the Tab index of the sidebar. In a wide window the sidebar is at the left and stays first.
- If the More menu or another control needs a different index, stop and report.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Tab bar | extend | src/ui/composites/tab_bar.rs | The owner of the narrow tab bar. |
| Menu trigger Tab index | extend | src/ui/primitives/context_menu.rs | `ContextMenu::trigger_tab_index` gives the More trigger its index. |
| Button Tab index | reuse | src/ui/primitives/button.rs | `Button::tab_index` sets the index. |
| List Tab stop | reuse | src/view_models/list_selection.rs | `list_is_tab_stop` does not change. |
| Status bar order test | reuse | src/ui/composites/status_bar.rs | `adr_0101_task_008_tab_reaches_the_list_before_the_status_bar` gives the test pattern. |

## Mechanical Criteria

- A new GPUI test `adr_0092_task_004_the_list_comes_before_the_tab_bar` mounts a narrow shell at 390 px. The shell has a list page header control, a list with one row, a status bar item and the tab bar. From the header control, Tab focuses the list, then the status bar item, then the first tab. The test fails before the change.
- `adr_0101_task_008_tab_reaches_the_list_before_the_status_bar` passes without a change.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: in Light and Dark at 390 px, Releases, Rows view: Tab from "In Library". The first row of the list shows the focus ring.
- V2: the next Tab stops go to the status bar items, then to Search, Releases, Show, Inbox and More in the tab bar.
- V3: shift+Tab from Search goes back through the status bar to the list.
- V4: at 1440 px the sidebar is still the first Tab stop of the window.
- Incorrect: the list does not take the focus after "In Library", or a tab of the tab bar comes before the list.
