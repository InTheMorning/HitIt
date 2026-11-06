# ADR 0089 Task 007: Saved Search Edits And New Counts

Status: Implemented - 2026-11-06. Parts A to C. V1 to V4 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 006](adr-0089-task-006-search-page.md).

## Goal

A saved search has the item menu Edit Query, Rename…, Duplicate and Delete Saved Search…. "Edit Query" or a click on its query puts the query in the toolbar search field, and the results change while the curator types. "Save Query" or Enter keeps the change, and "Cancel" or Escape restores the stored query (ADR 0089 Decision 4, R-TOOLBAR-1 item 2). The Search page shows the count of the new results of each saved search. A saved search keeps its text (ADR 0087 Decision 6).

## Files To Inspect

`src/db/saved_searches.rs`, `src/db.rs` (`CURRENT_VERSION`, the migration array, `schema_contract`), `src/db/upgrades.rs`, `src/library/saved_searches.rs`, `src/app/saved_search_dispatch.rs`, `src/app/search_live.rs`, `src/view_models/live_run.rs`, `src/view_models/search_results/saved_search.rs`, `src/view_models/search_page.rs`, `docs/adr/0016-*`.

## Design

1. Migration 22 in `src/db/saved_search_seen.rs`, through the ADR 0016 registry. It adds the nullable column `saved_searches.name` and the table `saved_search_seen(saved_search_id, result_key)`, with a foreign key and `ON DELETE CASCADE`. Follow the test pattern of migration 21: upgrade, failed migration, and contract.
2. A row shows the name when it has one, and else the query (`saved_search_label`). The query stays visible on the page.
3. `update_saved_search_query`, `rename_saved_search` and `duplicate_saved_search` are the new writers. The unique index on the query stays. A duplicate takes the text "name copy", and the query field opens so the curator changes it. A write that hits the unique index gives a typed result, and the view model shows a message. It never panics.
4. The query edit uses the toolbar search field, so the field list and its autocomplete work as in ADR 0087 Decision 3. The toolbar text is the live query of the saved search page, with the live run of ADR 0087 Decision 4. Escape restores the stored text and the stored results. The focus and Escape order follow ADR 0087.
5. The count of new results: a result key is the identity of a row: a release key, a track key or an artist key. When the curator opens a saved search, the run of that page stores each key of its results as seen, and the count becomes zero.
6. On the Search page, one runtime actor (ADR 0040) runs each saved search in sequence, with the first page only. It compares the keys with the seen table and gives a count. It runs when the page opens, at most one time for each 10 minutes. It uses a named constant. A failure gives no count, not a zero. A query that needs the Index waits behind the rate limits of the present request owner.
7. A first run of a saved search with no seen row stores its keys and gives zero. Thus an upgrade shows no false count.
8. The count is a word for the accessibility label ("3 new results"), and the row shows its text. It does not rely on color alone.
9. The item menu uses the shared row menu of `src/ui/composites`. Each item carries typed availability and an accessibility label.

## Do Not Touch

The results page rows (packet 008).

## Acceptance Criteria

Mechanical:

- Migration 22 is in the registry, with its upgrade and failure tests. `AGENTS.md` states schema version 22.
- The writers rename, update and duplicate, and a unique conflict gives the typed result.
- The count model gives results minus seen, gives zero after an open, and gives no count after a failed run. Unit tests hold each.
- The menu view model has the four items in the sequence of Decision 4.
- The gate is Green.

Visual:

- V1: Each menu item acts in place, and the list updates with no page change.
- V2: A click on a query makes a field. The results change while typing, Enter keeps the change, and Escape restores it.
- V3: A saved search with new results shows the count. After an open, the count goes.
- V4: The Search page does not freeze while the counts run.

## Test Commands

`cargo test saved_search`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if the seen table cannot hold the key of a result in one type.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-007-saved-search-edits-and-new-counts.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 4)
- docs/adr/0016-* and src/db/saved_searches.rs for the migration pattern
- the files that the task lists

Goal:
- Saved search edits, and the count of new results with migration 22.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The results page rows.

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
