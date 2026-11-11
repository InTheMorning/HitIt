# ADR 0089 Task 009: Public Playlists

Status: Implemented - 2026-11-06. V1 to V3 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 008](adr-0089-task-008-results-rows.md).

## Goal

The destination Public playlists lists the public playlists of the Index, which are musicL feeds (ADR 0089 Decision 1). Its sort starts at Newest, and its view starts at Tiles. It has no scope (Decision 5). A public playlist opens over the list.

## Files To Inspect

`src/api.rs` (`fetch_recent_feeds`, `fetch_feeds_by_name`, `RemoteItem`), `tests/fixtures/musicindex-openapi-0.18.1.json`, `src/application/queries/name_lists.rs`, `src/library/music_scopes.rs`, `src/view_models/list_page.rs`, `src/view_models/sidebar.rs`, `src/view_models/workspace/nav.rs`, `src/view_models/collection_art.rs`, `src/view_models/release_page.rs`, `src/ui/shells/release_page.rs`, `src/views.rs:417` and `:538`, and ../musicindex/docs/adr/0040-a-playlist-with-no-image-shows-a-mosaic.md.

## Design

1. A sidebar row "Public playlists" follows "My playlists". The entry `FrameNavigationEntry::PublicPlaylist(String)` holds the feed GUID. `ListPageKey::PublicPlaylists` has the defaults of Decision 5.
2. `src/view_models/public_playlists.rs` holds the list: its rows or tiles, its loading row, its error row and Load more, with the shape of the Index name lists. Newest uses `GET /v1/feeds/recent` with `medium=musicL`. A to Z uses `GET /v1/feeds/by-name` with `medium=musicL` and the letter bar of packet 005. An empty list gives the empty state of R-EMPTY-1.
3. The item page reads `GET /v1/feeds/{guid}` with `include=remote_items`. It lists each remote item in `position` sequence, with the feed title and the image, and with the rel word that the response gives. The header has the title and art. A remote item with a `remote_item_guid` opens the track page by its `TrackKey`. A remote item with only a feed GUID opens the release page. The page uses a loading state and an error state with Retry.
4. Treat each response as untrusted input. Add decode tests from the stored contract.
5. The item menu has only "Copy to My Playlists…", and packet 010 adds it. Open point: no other actions.
6. Art: the feed image, else the mosaic of musicindex ADR 0040 from the remote item images.
7. The fetch runs through the ADR 0040 runtime, never from a screen.

## Do Not Touch

`playlists` and `playlist_tracks` storage. My playlists.

## Acceptance Criteria

Mechanical:

- `ListPageState::default_for(PublicPlaylists)` gives Newest, Tiles and no scope.
- The list query sends `medium=musicL` with each sort. A unit test reads the built URL.
- The item view model lists the remote items in `position` sequence and tolerates a null `remote_item_guid`.
- The view model gives an empty state for an empty response, and an error state for a failed one.
- `SidebarVm` lists the row after My playlists.
- The gate is Green.

Visual:

- V1: Public playlists opens with tiles, newest first. The view icons change to rows.
- V2: A to Z shows the letter bar.
- V3: A playlist opens over the list, its rows open their pages, and Back returns with the tile marked.

## Test Commands

`cargo test public_playlists`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if the Index returns no musicL feeds through these endpoints, and record the response.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-009-public-playlists.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decisions 1 and 5, The Contract)
- the files that the task lists

Goal:
- A Public playlists list page and its item page from the Index.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- Local playlist storage and My playlists.

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
