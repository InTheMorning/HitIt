# ADR 0089 Task 015: Inspect Sources Is A Sheet

Status: Implemented - 2026-11-06. V1 to V4 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 003](adr-0089-task-003-one-sidebar.md).

## Goal

"Inspect Sources" opens as a sheet (ADR 0089 Decision 8, musicindex ADR 0027 R-SHELL-1 item 6). ADR 0083 Decision 8 keeps its content. The app has no general inspector.

## Files To Inspect

`src/ui/shells/library/track_detail.rs:65-80`, `src/ui/composites/track_detail_surface.rs`, `src/ui/composites/disclosure_group.rs`, `src/library.rs:142` and `:329`, `src/library/app_impl.rs:2853` and `:3710`, `src/library/release_page.rs:45-49`, `src/ui/shells/window_layers.rs`, `src/ui/composites/confirmation_dialog.rs`, and `docs/adr/0083-design-language.md` (Decision 8).

## Design

1. A new composite `src/ui/composites/inspect_sources_sheet.rs` shows the section content in a sheet through `Root::open_sheet`, with `render_window_layers` as the layer. It has a title, the tabs or groups that the present section has, and Close. The content stays scrollable.
2. The present section and its disclosure leave the track page. The item menu entry "Inspect Sources" opens the sheet. When the request comes from a release row, the track page opens and then the sheet opens. `pending_inspect_sources` stays for that path.
3. Escape and Close close the sheet, and the focus returns to the control that opened it. A mutation while the sheet is open (a provider fetch) updates the sheet in place.
4. Keep the source state in `InspectorFrame`. Rename only if the name now says something false.
5. Index track pages keep the rule that they have no file tags (the open finding of the ADR 0084 plan).

## Do Not Touch

The content rows of the section. The tag update flow.

## Acceptance Criteria

Mechanical:

- The track page view model no longer exposes the Inspect Sources section. A new view model fact gives the sheet state, and a unit test holds open, close and an update in place.
- The item menu exposes "Inspect Sources" with its accessibility label.
- A guard citing ADR 0089 Decision 8 holds that `track_detail.rs` builds no Inspect Sources group.
- The gate is Green.

Visual:

- V1: The menu item opens a sheet over the track page, and the page stays below it.
- V2: Escape closes it, and the focus returns to the menu button.
- V3: The content scrolls, and a long value wraps.
- V4: A row menu "Inspect Sources" from a release page opens the track and the sheet.

## Test Commands

`cargo test inspect`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if `Root::open_sheet` cannot host retained state updates. Then report and propose a window layer of the same form.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-015-inspect-sources-sheet.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 8)
- docs/adr/0083-design-language.md (Decision 8)
- the files that the task lists

Goal:
- Inspect Sources opens as a sheet, with the same content.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The content rows and the tag update flow.

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
