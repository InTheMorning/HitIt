# ADR 0089 Task 013: The Narrow Tab Bar

Status: Ready - 2026-11-05.

Depends on: [packet 011](adr-0089-task-011-live-page.md) and [packet 012](adr-0089-task-012-inbox.md).

## Goal

In a narrow window, a tab bar replaces the sidebar (musicindex ADR 0027 R-SHELL-1 item 7). It has Search, Releases, Show, Inbox and More. More opens a menu with each other destination (ADR 0089 Decision 1).

## Files To Inspect

`src/view_models/sidebar.rs`, `src/app.rs` (`render`), `src/ui/layouts.rs`, `src/ui/composites/` (the context menu and the segmented control), `src/ui/primitives/context_menu.rs`, `src/app/toolbar.rs`, `src/app/keyboard.rs`.

## Design

1. A pure function `shell_form(width_px, breakpoint_px)` gives `Sidebar` or `TabBar`. A named token gives the breakpoint. Reuse the value of `MUSIC_SINGLE_COLUMN_BREAKPOINT` if the token remains, with a shell name.
2. `TabBarVm` comes from `SidebarVm`. It gives the four named items, More, the count of the Inbox, and the items of the More menu: Artists, My playlists, Public playlists, Live and Settings, in the sidebar sequence. When the current destination is in the menu, More is the selected tab, and the menu marks the row.
3. A composite `src/ui/composites/tab_bar.rs` draws the bar at the bottom of the window, with named tokens. Each item has an icon, a word and an accessibility label. The feed state and "Check All Feeds" go to the More menu in this form, or to the Settings page if a menu cannot hold them. Record the choice.
4. The search field in the toolbar stays, with the narrow placeholder rule of R-TOOLBAR-1 item 3 only on Touch. The Desktop profile keeps the normal text.
5. Each keyboard section shortcut keeps its target.

## Do Not Touch

Page content. The now-playing bar of R-SHELL-1 item 5, which packet 016 owns.

## Acceptance Criteria

Mechanical:

- `shell_form` gives each form at each side of the breakpoint.
- `TabBarVm` gives Search, Releases, Show, Inbox and More. The More menu gives each other destination of Decision 1. A test compares the union with `SidebarVm`.
- The selected item follows the destination, and More is selected for a menu destination.
- The gate is Green.

Visual (narrow width):

- V1: The tab bar shows, and the sidebar is absent.
- V2: More opens the menu, and each item reaches its page.
- V3: A resize across the breakpoint keeps the page, the scroll and the selection.

## Test Commands

`cargo test tab_bar`, `cargo test sidebar`, then the full gate.

## Escalation

Stop if the menu primitive cannot open upward from a bottom bar.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-013-narrow-tab-bar.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 1)
- the files that the task lists

Goal:
- A tab bar with More in a narrow window.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- Page content and the now-playing bar.

Acceptance criteria:
- The mechanical criteria of the task.

Test commands:
- cargo fmt -- --check; cargo clippy --all-targets -- -D warnings; cargo test; cargo build --bin v4vmm

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
