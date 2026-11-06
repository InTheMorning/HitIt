# ADR 0089 Task 008: The Rows Of The Results Page

Status: Ready - 2026-11-05.

Depends on: [packet 007](adr-0089-task-007-saved-search-edits-and-new-counts.md).

## Goal

The results page has two rows below its title (ADR 0089 Decision 4). "All | In Library" is part of the query: a click writes or removes `in:library`, and it shows no counts. The type tabs are All, Releases, Tracks, Artists and Labels. They select a group of one run and show their counts. They do not change the query.

The sort starts at Newest, and the view starts at Rows (Decision 5). A saved search has the same rows.

## Files To Inspect

`src/view_models/search_results/{mod,tabs,results,rows,paged_tab,saved_search,saved_search_groups}.rs`, `src/view_models/search_query.rs`, `src/ui/shells/search_results_inspector.rs`, `src/ui/shells/search_result_rows.rs`, `src/app/search_dispatch.rs`, `src/app/search_live.rs`, `src/app/search_selection.rs`, `src/app/saved_search_dispatch.rs`, `src/app/toolbar.rs`, `src/view_models/list_page.rs`, `src/ui/composites/list_page_head.rs`, `src/db/saved_searches.rs` (`start_tab`), `tests/architecture_tests.rs`.

## Design

1. `search_query.rs` gains a pure function `with_library_scope(text, ListScope) -> String`. It adds `in:library` at the end, or removes that term. It keeps each other term and the spacing rule. A conflict with `in:index` follows ADR 0087: the function removes neither term without a click on the scope, and the view model shows the present conflict message. The view model reads the state of the control from `QueryScope`.
2. A click on the scope writes the toolbar field text and starts a new run, by the path of a typed change. On a saved search page, it edits the query field of packet 007.
3. `SearchResultsTab` becomes All, Releases, Tracks, Artists and Labels, in this sequence. A tab shows its count from the rows of one run. `Has more` gives "20+" from a view model text. The label comment "A label shows no count" goes.
4. All is the default tab. It lists the groups in the tab sequence, with a heading for each group.
5. Artists and Labels split the present artist rows by the role of the feed (the link facts of ADR 0084 packet 029, musicindex R-WORD-3 and R-WORD-4). A name row with no role stays in Artists. Record the rule in the run record.
6. A stored `start_tab` keeps its values. A new tab value needs the CHECK of the table. Do not add a migration for it: a saved search opens at All, and the column stays until a later cleanup. Report this in the run record. If `start_tab` then has no reader, say so in the report and do not delete it here without a migration.
7. Sort: `ListPageKey::Results` and `SavedSearch` have Newest and A to Z, in the rows of one run. The sort does not change the query (Decision 4, the three conditions). Rows with no date go last. View: Rows and Tiles for the Releases tab. Other tabs show rows only, and the view icons are absent on them.
8. `SearchResultsHeaderMode` has one variant: delete it. The head uses `list_page_head`, with the save action in the title row.
9. The selection model of ADR 0084 stays on the Releases and Tracks tabs and on the groups of All, or the All tab has no Select. Choose the second if the first needs a new selection model, and record it.

## Do Not Touch

The live run timing of ADR 0087. Index request paths.

## Acceptance Criteria

Mechanical:

- `with_library_scope` writes and removes `in:library`, and a unit test covers an empty query, one term, a quoted term, a term in the middle and `in:index`.
- A type tab change does not change the query. A unit test holds this at the view model.
- The tabs have the sequence of Decision 4, and each exposes its count. The scope control exposes no count.
- `ListPageState::default_for` gives Newest and Rows for the results and a saved search.
- Artists and Labels split by role in a test with the stored fixture.
- The gate is Green.

Visual:

- V1: The rows show in the sequence of Decision 4 on the results page and on a saved search.
- V2: A click on In Library puts `in:library` in the field and runs. A second click removes it.
- V3: A type tab keeps the query text and shows the count.
- V4: The sort changes the rows and keeps the query.

## Test Commands

`cargo test search_results`, `cargo test search_query`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if the text rewrite conflicts with the parser rules of ADR 0087.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-008-results-rows.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decisions 4 and 5)
- docs/adr/0087-search-fields-and-live-results.md
- the files that the task lists

Goal:
- The scope row writes the query. The type tabs show counts and do not change the query.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The live run timing and the Index request paths.

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
