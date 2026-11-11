# ADR 0089 Task 003: One Sidebar

Status: Implemented - 2026-11-05. V1 to V3 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 002](adr-0089-task-002-toolbar-history-and-marked-row.md).

## Goal

One sidebar shows on each mount (ADR 0089 Decision 1). The toolbar has no section tabs. The app starts on Releases. The destination Tracks goes (Decision 2).

## Files To Inspect

`src/app.rs` (`AppTab`, `select_tab`, `WorkspaceScreenMount`, `render`, `visible_workspace_layout`), `src/app/tab_bar.rs`, `src/app/keyboard.rs`, `src/app/menu.rs`, `src/view_models/app_toolbar.rs`, `src/view_models/workspace/{mod,nav,columns,frame}.rs`, `src/ui/shells/workspace.rs`, `src/ui/shells/library/sidebar.rs`, `src/library.rs`, `src/library/app_impl.rs` (`render_sidebar_column`, `show_destination_list`), `src/view_models/library.rs` (`destination_rows`, `destination_row`), `src/ui/layouts.rs`, `tests/architecture_tests.rs`.

## Design

1. A new view model `SidebarVm` in `src/view_models/sidebar.rs` owns the rows. It lists the upper destinations, a separator, then the lower destinations. A typed `SidebarDestination` has a variant for each row. A later packet adds its own row. After this packet the upper rows are Discover, Releases, Artists, My playlists and Needs attention, and the lower rows are Show and Settings. Discover goes in packet 006, and Needs attention goes in packet 012. The packet that adds the last row (packet 012) asserts the full sequence of Decision 1.
2. The word "Playlists" becomes "My playlists" in the sidebar.
3. `TopApp::render` puts the sidebar beside the toolbar and the content, on each mount, so the toolbar is the toolbar of the content area (R-TOOLBAR-1). The application mark moves to the top of the sidebar. The sidebar is one list with no group labels. The feed state and "Check All Feeds" stay at its bottom (Decision 1).
4. A sidebar row selects its mount. Music rows call the existing open path. Show and Settings call `select_tab`. The selected row follows the mount and the destination.
5. `WorkspaceFrameKind::SourceList` goes, because the sidebar has left the frames. Move the content frame and the detail frame into one Music content frame if this is a small change. If it is not, record the result, and keep `Detail` for the next packet that needs the change. `WorkspaceLayout::from_config` must load an old `config.toml` that names a removed kind. A serde test holds this.
6. The toolbar tabs go. `AppToolbarTabKey`, `AppToolbarTabDisplay` and `render_tab_bar` lose the tabs. The keyboard actions and menu items that select a section stay. Rename `render_tab_bar` to `render_toolbar` and `src/app/tab_bar.rs` to `src/app/toolbar.rs`. `AppTab` stays as the mount state.
7. The icon-only sidebar, its collapse button and `MusicSidebarPreference` go. R-SHELL-1 item 7 gives a tab bar in a narrow window, and packet 013 builds it. Until that packet, the sidebar keeps its width at each window width.
8. The start: `default_navigation_entry` gives `Destination(Releases)`. `active_destination` starts at Releases. A saved location is not restored. The app starts on Releases (Decision 1).
9. Tracks goes: `MusicDestination::Tracks`, its list, its rows, its Tracks selection and the content-list view-mode control. Check each `content_list_*` path: the path stays only if another destination still reaches it. `Needs attention` keeps the list until packet 012. The "Tracks" tab of search stays.
10. Delete or change the guards of the replaced rules: `app_toolbar_exposes_tabs_and_global_search_without_now_playing_chip` (keep its now-playing and search parts), `adr_0048_library_settings_tabs_drive_content_list_nav`, `adr_0067_mount_and_section_changes_establish_a_persistent_focus_path` (keep the focus rule on the new sidebar rows), `adr_0084_destinations_*` where they state the six destinations. Add the situational guard `adr_0089_toolbar_has_no_section_tabs`.

## Do Not Touch

The list page controls (packet 004). The Discover content (packets 005 and 006). Show content, apart from its mount position.

## Acceptance Criteria

Mechanical:

- `SidebarVm` exposes the rows of this packet in sequence, with the separator before Show, and Releases as the default.
- `WorkspaceLayout::default_layout` gives `Destination(Releases)`, and an old config with `source_list` loads.
- `AppToolbarVm` exposes no tab.
- `MusicDestination::Tracks` and `MusicSidebarPreference` do not exist.
- ADR 0060 words stay: `Music`, `Library`, and `Show` as a screen mount.
- The gate is Green.

Visual:

- V1: The app starts on Releases. The sidebar has the rows of this packet, and a line before Show.
- V2: The sidebar stays on Show and Settings. A click on each row reaches its page.
- V3: A keyboard section shortcut still selects its section. The focus goes to the list page on Music, and to the matching sidebar row on Show and Settings.

## Test Commands

`cargo test --test architecture_tests`, `cargo test sidebar`, then the full gate.

## Escalation

Stop if the sidebar view cannot render on Show without `LibraryApp` state that Show mutates. Stop if a Show rule of ADR 0060 conflicts with the mount position.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-003-one-sidebar.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md
- docs/adr/0060-workflow-surface-structure.md
- the files that the task lists

Goal:
- One sidebar on each mount, no section tabs, the start page Releases, and Tracks goes.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The list page controls, Discover content and Show content.

Acceptance criteria:
- The mechanical criteria of the task.

Test commands:
- cargo fmt -- --check; cargo clippy --all-targets -- -D warnings; cargo test; cargo build --bin HitIt

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
