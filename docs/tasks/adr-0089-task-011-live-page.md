# ADR 0089 Task 011: The Live Page

Status: Implemented - 2026-11-06. V1 to V3 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 008](adr-0089-task-008-results-rows.md).

## Goal

The destination Live shows what is live now on the Index (ADR 0089 Decision 1). The source is `GET /v1/live-items` (The Contract).

## Files To Inspect

`src/api.rs`, `tests/fixtures/musicindex-openapi-0.18.1.json` (the path `/v1/live-items` and `LiveItemListResponse`), `src/application/queries/name_lists.rs`, `src/view_models/sidebar.rs`, `src/view_models/list_page.rs`, `src/view_models/workspace/nav.rs`, `src/ui/composites/list_row.rs`, `src/library/music_scopes.rs`.

## Design

1. `Api::fetch_live_items` uses `view=now`, `limit` and `cursor`. Open point: no upcoming items. A decode test reads the stored contract. Treat each field as untrusted input, and keep the rule of `link_scheme_added` that the contract states.
2. `src/view_models/live_page.rs` gives one row for each item: the title, the feed title, the image, the type word and the scheduled time as a text. It gives the loading row, the error row with Retry, Load more, and the empty state of R-EMPTY-1 with the words "Nothing is live now."
3. The sidebar row "Live" follows "Public playlists". `ListPageKey::Live` has no scope, no sort and Rows. ADR 0089 Decision 5 does not list this page, so it has no controls.
4. The main action of a row is a link out to `content_link`, if the response gives one. It uses the opener of the present link icons. A click on the row opens the release page of `feed_guid`. No playback (ADR 0068).
5. A page open and a Back to the page start a new read, with the old rows on view until the response comes. No timer in this packet. Add the absence of a live refresh to the responsiveness list if the walk finds it slow.
6. This page does not use the broadcast status of Show. A row is not an operational surface (ADR 0089 Decision 7).

## Do Not Touch

`src/view_models/live_status.rs`, the Show page and `create_live_item`.

## Acceptance Criteria

Mechanical:

- The request sends `view=now`. A unit test reads the built URL.
- `LivePageVm` gives rows, the empty state, the error state and Load more from `Pagination`.
- `SidebarVm` lists Live after Public playlists.
- Decode tests pass with the stored contract, with a null `content_link`.
- The gate is Green.

Visual:

- V1: Live shows the items on the Index, or the empty state.
- V2: A row click opens the release, and Back returns with the row marked.
- V3: The link out opens in the browser.

## Test Commands

`cargo test live_page`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if a row has no feed GUID in the response.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-011-live-page.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 1, The Contract)
- the files that the task lists

Goal:
- A Live list page from GET /v1/live-items with view=now.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The live status of Show and create_live_item.

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
