# ADR 0089 Task 014: The Row Rules

Status: Implemented - 2026-11-06. Play moves to packet 016. V1 to V4 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 005](adr-0089-task-005-releases-sort.md).

## Goal

Each list row follows ADR 0089 Decision 6, apart from the main action Play. Packet 016 owns Play, because ADR 0068 owns Music playback.

## Files To Inspect

`src/ui/shells/library/destination_lists.rs`, `src/ui/composites/list_row.rs`, `src/ui/composites/tag_badge.rs`, `src/ui/composites/track_row.rs`, `src/ui/composites/release_download_state.rs`, `src/ui/composites/release_row_menu.rs`, `src/ui/shells/library/content_list_row_actions.rs`, `src/ui/shells/search_result_rows.rs`, `src/view_models/library.rs` (`DestinationListRowVm`), `src/view_models/releases_everything.rs`, `src/ui/tokens.rs` (`TrackRowColumn`, `TypeDot`), `../musicindex/docs/adr/0038-a-dot-on-art-sits-on-its-corner.md`, `../musicindex/docs/adr/0032-the-type-dot-is-one-shared-marker.md`.

## Design

1. Each list row shows its type dot and type word (musicindex ADR 0038 R-DOT-4). `DestinationListRowVm` exposes the type, and the row draws the shared `TagBadge`. A numbered track row on a release page shows no dot, because its section heading gives the type.
2. Named tokens give one column for the library icon and one for the time, so they align in a list. Extend `TrackRowColumn` or add one token set. A row with no time keeps the empty column.
3. The time of a release row is the total duration, if the row knows it. An Index row with no duration keeps the empty column. Do not add a request.
4. The library icon is muted. A warning or a danger keeps its status color (`ReleaseDownloadState`). The word stays in the accessibility label.
5. Adding to a playlist is in the item menu only. Remove any other path on a row. The selection bar keeps "Add to playlist", because it is not a row.
6. The "Select" button is in the title row of the list page (packet 004). Verify this on each page and delete a remaining copy.
7. Apply the rules at the shared row owner. Search result rows use the same columns.

## Do Not Touch

The main action Play. The sort. The content of an item page.

## Acceptance Criteria

Mechanical:

- The view model of a list row exposes its type word, and a numbered track row on a release page exposes none.
- The composite owns the two columns from named tokens. A guard cites ADR 0089 Decision 6 and checks that `destination_lists.rs` has no raw width for them.
- The row view model exposes no add-to-playlist action outside its menu.
- The gate is Green.

Visual:

- V1: In Releases, Artists and the results, each row shows the dot and the word.
- V2: The icons and times align in one column in a list with mixed rows.
- V3: The library icon is muted. A failed release shows the danger color.
- V4: Add to playlist is only in the item menu.

## Test Commands

`cargo test list_row`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if musicindex R-DOT-4 conflicts with a marker on a row. A change to a shared rule needs a record in `../musicindex`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-014-row-rules.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 6)
- the files that the task lists

Goal:
- Rows show the type dot and word, two aligned columns, a muted library icon, and add to playlist only in the menu.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The main action Play, the sort and the item pages.

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
