# ADR 0089 Task 002: Toolbar History And The Marked Row

Status: Ready - 2026-11-05.

Depends on: [packet 001](adr-0089-task-001-one-page-stack.md).

## Goal

The location breadcrumb goes (ADR 0089 Decision 9). The toolbar has the `‹ ›` icons and the page title at the left (musicindex ADR 0027 R-TOOLBAR-1 item 1). Back returns to the list at the same scroll position and marks the row of the item that was open (R-NAV-2 item 2).

## Files To Inspect

`src/app/breadcrumb.rs`, `src/ui/composites/breadcrumb_trail.rs`, `src/view_models/workspace/breadcrumb.rs`, `src/view_models/workspace/chrome.rs`, `src/view_models/workspace/nav.rs`, `src/ui/composites/frame_shell.rs`, `src/ui/shells/workspace.rs`, `src/app.rs`, `src/app/tab_bar.rs`, `src/view_models/app_toolbar.rs`, `src/ui/composites/list_row.rs`, `src/ui/composites/music_column_scroll.rs`, `src/library/app_impl.rs`, `tests/architecture_tests.rs`.

## Design

1. Delete `src/app/breadcrumb.rs`, `src/ui/composites/breadcrumb_trail.rs`, `src/view_models/workspace/breadcrumb.rs` and each slot, labeler and handler of the breadcrumb. Keep the location logic of `nav.rs` that Back, Forward and the destination need. Keep the title resolution of `content_list_breadcrumb_labels` under a new name, because the toolbar title needs the same labels.
2. `AppToolbarVm` gets the history: Back and Forward with typed availability, the accessible names and tooltips "Back" and "Forward", and the page title. Move the frame title overrides of `render_workspace_content` (singles group, window group, feed page) to one function that gives the title of the current entry. Forward is disabled with no subsequent page (R-NAV-2 item 4).
3. The frame shell stops showing Back, Forward and the title in Music. Show and Settings show the same toolbar: Back and Forward are absent on a mount with no history. A surface that cannot act is absent.
4. R-NAV-2 item 3: a path that opens an item starts at the destination that holds it, so Back opens that list. A unit test holds this for a release, a track and a playlist. No app path opens a shared link at present.
5. The memory of an entry stores the list scroll beside `detail_scroll`. A list root records its scroll when a page opens over it, and Back restores it. Each list that the destination uses has a test at the model layer. Verify if `MusicColumnList` keeps its offset while it does not render. If it does, record this and add no second store.
6. `FrameNavigationState` gives the entry that was open over the current list root: `marked_entry()`. It is `None` after a new path. The list row view models expose `marked: bool` from this fact. `ListRow` draws the mark through a new `RowOutline` state from named tokens. It does not rely on color alone: it uses the same outline width as selection and a leading bar. The mark goes on the next row click or destination change.
7. Delete the breadcrumb guards: `adr_0047_task_013_frame_shell_renders_breadcrumb_chrome`, `adr_0047_breadcrumb_unification_guards_frame_shell_helpers_removed`, `breadcrumb_pop_syncs_library_detail`, and the breadcrumb parts of `source_fact_placeholder_and_breadcrumb_regressions_are_guarded` and of the tests at `tests/architecture_tests.rs:3417` and `:4103`. Keep each part of a test that holds another rule.
8. Add the situational guard `adr_0089_no_view_model_exposes_a_breadcrumb`.

## Do Not Touch

The section tabs of the toolbar (packet 003). The search field behavior.

## Acceptance Criteria

Mechanical:

- `AppToolbarVm` exposes Back, Forward and the title for a Music page, with the availability of the navigation state.
- `FrameNavigationState` gives R-NAV-2 items 2 to 4: the restored scroll, the marked entry, the holding list and a disabled Forward. Unit tests hold each.
- The row view model exposes `marked` for the entry that was open, and for no other row.
- Search for `breadcrumb` (any case) in `src/` gives no result, apart from the guard text.
- The gate is Green.

Visual:

- V1: The toolbar shows `‹ ›` and the title at the left, and the search field does not change its width on focus.
- V2: Open a release from far down Releases, then press Back. The list has the same scroll position, and the row is marked.
- V3: Forward is disabled at the last page.

## Test Commands

`cargo test workspace`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop when a test of a rule that stays depends on the breadcrumb and no other model holds the fact.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-002-toolbar-history-and-marked-row.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md
- ../musicindex/docs/adr/0027-cross-app-shell-lists-and-toolbar.md
- the files that the task lists

Goal:
- Delete the breadcrumb. Put Back, Forward and the title in the toolbar. Keep the list scroll and mark the row.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The section tabs and the search field behavior.

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
