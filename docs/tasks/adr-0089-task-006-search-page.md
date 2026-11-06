# ADR 0089 Task 006: The Search Page

Status: Implemented - 2026-11-06. V1 to V4 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 005](adr-0089-task-005-releases-sort.md).

## Goal

The Search destination opens a page that lists each saved search (ADR 0089 Decision 4). Discover goes (Decision 2).

## Files To Inspect

`src/view_models/search_results/saved_search.rs`, `src/library/saved_searches.rs`, `src/app/saved_search_dispatch.rs`, `src/ui/shells/library/views_list.rs`, `src/ui/shells/saved_search_delete_confirmation.rs`, `src/view_models/sidebar.rs`, `src/view_models/workspace/nav.rs`, `src/app/keyboard.rs` (`handle_focus_search`), `src/app/search_dispatch.rs`, `src/ui/composites/list_row.rs`, `src/library/app_impl.rs`, `tests/architecture_tests.rs`.

## Design

1. `MusicDestination::Discover` becomes `MusicDestination::Search`, first in the sidebar. `Search` has the page view model `SearchPageVm` in `src/view_models/search_page.rs`.
2. A click on the destination opens the Search page and puts the focus in the toolbar field (Decision 4). The focus path reuses `handle_focus_search`. The toolbar command stays the same on each page, and the field does not move.
3. The page lists each saved search with its query, in a list page with the title "Search" and no sort, view or scope. It has the button "New Saved Search". The button puts the focus in the toolbar field, and the field holds an empty query. "Save" on the results page stays as the way to store it. If the results page already has this action, reuse it.
4. "How to Search" gives four steps and example queries, from the view model. The steps show when no search is saved. A click on an example runs the query. Take the examples from the ADR 0087 field table. This state is the empty state of R-EMPTY-1. A saved search list with rows has a disclosure "How to Search".
5. A saved search opens over the list, with the same results as the results page. The entry `SavedSearch` under the destination Search replaces the Discover parent. Back returns to the Search page and marks the row.
6. The item menu has Delete Saved Search…. Packet 007 adds its other items. The delete dialog stays.
7. Delete `src/ui/shells/library/views_list.rs` and the Discover guards. Guards of saved searches change their paths only. Delete `ADR 0060` Discover guards only if their rule concerns the old surface name. Keep `adr_0060_discover_surface_stays_deleted` if it holds a dead-surface rule that remains true. Check each.
8. The name "Discover" stays only in a test of a config or a comment that history requires. A search for `Discover` in `src/` gives no result in code.

## Do Not Touch

The tabs and scope of the results page (packet 008). Rename, Duplicate, Edit Query and counts (packet 007).

## Acceptance Criteria

Mechanical:

- `SearchPageVm` lists each saved search with its query. With no saved search it gives the four steps and the examples.
- A saved search opens under the destination Search, and Back gives that destination.
- `MusicDestination::Discover` does not exist. `SidebarVm` lists Search first.
- The gate is Green.

Visual:

- V1: A click on Search opens the page, and the cursor is in the toolbar field.
- V2: With no saved search, the steps and examples show. A click on an example runs it.
- V3: A saved search opens over the list, and Back returns with the row marked.
- V4: The delete dialog removes the row in place.

## Test Commands

`cargo test saved_search`, `cargo test search_page`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if the Search page and the results page need one navigation entry that ADR 0087 does not permit.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-006-search-page.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 4)
- docs/adr/0087-search-fields-and-live-results.md
- the files that the task lists

Goal:
- The Search page lists the saved searches, and Discover goes.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The results page tabs and scope. Rename, Duplicate, Edit Query and counts.

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
