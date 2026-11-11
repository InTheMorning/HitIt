# ADR 0089 Task 001: One Page Stack

Status: Implemented - 2026-11-05. The visual checks V1 and V2 are open in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: none. The [phase plan](../plans/adr-0089-shell-phase-plan.md) holds the rules for each packet.

## Goal

Music shows the sidebar and one page. A list is a page at full width, and an item opens over its list (musicindex ADR 0027 R-SHELL-1 items 1 to 3, ADR 0089). No pane shows "Select an item in the list."

## Authority

- ADR 0089 Invariants: no view model exposes the text "Select an item". Each destination opens on content or an empty state.
- ADR 0089 amends ADR 0046: Music has no separate list frame and detail frame.

## Files To Inspect

`src/view_models/workspace/columns.rs`, `src/view_models/workspace/mod.rs`, `src/view_models/workspace/nav.rs`, `src/ui/shells/workspace.rs`, `src/ui/composites/frame_shell.rs`, `src/ui/layouts.rs`, `src/app.rs` (`visible_music_layout`, `render_workspace_content`), `src/app/keyboard.rs`, `src/library/app_impl.rs` (`render_list_column`, `render_library_detail_pane`), `src/view_models/library.rs:813`, `tests/architecture_tests.rs`.

## Design

1. `MusicColumns` loses `Three`, and `for_width` loses its width input. It becomes one fact: the current entry is a list root or a page. Delete the function if `opens_detail_page` is enough.
2. `WorkspaceShell` shows the sidebar and one of the content-list frame and the detail frame, at the remaining width. Delete `MUSIC_LIST_COLUMN_WIDTH` and the breakpoint if nothing else reads them.
3. Keep both frames in this packet. Packet 003 removes them when the sidebar leaves the frames. The frame that shows has Back and Forward, each with its typed availability.
4. The default location of Discover opens "New releases" as a page. Back from it shows the Discover views list. This is an interim result until packet 006.
5. Delete `empty_detail_label` and each path that renders it. A page entry whose data is not loaded shows its loading state. If an entry has no page and no load, the content shows its list root.
6. Keep the search results as a list root. A result opens over them, and Back returns to them.
7. The keyboard handlers act on the page that shows. A key handler of a frame that does not show has no effect.
8. Delete the guards of the three-column rule, for example `adr_0084_columns_c4_06_music_columns_use_named_tokens_with_no_split_pane` and the tests `adr_0084_columns_*` of `MusicColumns`. Change `adr_0060_music_surface_is_dominant_content_without_operational_panes` only for the column names that this packet deletes.
9. Add the situational guard `adr_0089_no_view_model_exposes_select_an_item`. It scans `src/view_models` and `src/views.rs` for the text "Select an item". Its failure message names ADR 0089 Invariants and the fix.

## Do Not Touch

The toolbar and the breadcrumb (packet 002). The sidebar content (packet 003). Show and Settings content.

## Acceptance Criteria

Mechanical:

- The workspace view model gives one visible page for a navigation state: the list frame for a list root, and the detail frame for a page. A unit test covers `Destination`, `Search`, `Release`, `Track` and `PlaylistDetail`, at each width.
- The guard `adr_0089_no_view_model_exposes_select_an_item` passes, and it fails when the text returns.
- Search for `empty_detail_label` in `src/` gives no result.
- The gate is Green.

Visual (operator, Light and Dark, normal and narrow width):

- V1: Releases fills the area beside the sidebar. A click on a row opens the release at the same width, and Back returns to the list.
- V2: No pane is empty after a click on each destination.

## Test Commands

`cargo test --test architecture_tests`, `cargo test workspace`, then the full gate of the phase plan.

## Escalation

Stop and report when a page needs both frames at one time, or when a guard of a rule that ADR 0089 keeps cannot pass.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-001-one-page-stack.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md
- the files that the task lists

Goal:
- Music shows the sidebar and one page, with no placeholder pane.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The toolbar, the breadcrumb, the sidebar content, Show and Settings.

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
