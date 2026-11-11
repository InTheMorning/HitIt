# ADR 0089 Task 010: Copy To My Playlists

Status: Implemented - 2026-11-06. V1 to V3 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 009](adr-0089-task-009-public-playlists.md).

## Goal

"Copy to My Playlists…" is in the item menu of a public playlist (ADR 0089 Decision 1). It makes a local playlist with the tracks of the public playlist. It downloads nothing.

## Files To Inspect

`src/playlist_service.rs`, `src/library_service.rs`, `src/feed_service.rs` (`ensure_feed_in_db`), `src/db.rs` (`playlists`, `playlist_tracks`, `tracks`), `src/application/commands/`, `src/view_models/public_playlists.rs`, `src/ui/composites/confirmation_dialog.rs`, `src/ui/composites/track_row_menu.rs`, `src/view_models/playlist_detail.rs`.

## Design

1. The command takes the feed GUID and a name. A dialog asks for the name and gives the title of the public playlist as its default. If the name exists, the dialog shows a message and stays open, because `playlists.name` is UNIQUE.
2. The command runs on the ADR 0040 runtime. It reads the remote items. For each remote item, it reads the remote feed through the path that `ensure_feed_in_db` uses. It makes a `tracks` row with `is_in_library` false when none exists. It uses the same RSS provenance as a download (ADR 0075). A remote item with no `remote_item_guid` is skipped and counted, because it names a feed and not a track.
3. The command appends the tracks in `position` sequence. A track that the Index cannot resolve is skipped and counted.
4. The result names each count: "Copied 12 of 15 tracks to Road Trip. 3 could not be read." It uses a report of the operator rules of `AGENTS.md`. The new playlist appears in My playlists in place.
5. The local playlist has no link to its source after the copy. A later change of the public playlist does not change it.
6. The playlist page must show a row with no file: the row keeps the download control of ADR 0083 Decision 6. Verify this with a unit test at the view model.

## Do Not Touch

The schema. If a new column seems needed, stop.

## Acceptance Criteria

Mechanical:

- The command makes the playlist in `position` sequence, and it writes no file. A database test with an in-memory connection and a stub Index reader holds this.
- A second copy with the same name gives the typed conflict, and it writes nothing.
- The result reports copied and skipped counts. An item with no item GUID counts as skipped.
- The menu view model of a public playlist has the item with its accessibility label.
- The gate is Green.

Visual:

- V1: The menu item opens the dialog with the title as the default name.
- V2: After the copy, My playlists shows the new playlist in place, and its rows show the download control.
- V3: A name conflict keeps the dialog open with its message.

## Test Commands

`cargo test copy_to_my_playlists`, `cargo test playlist_service`, then the full gate.

## Escalation

Stop if a `tracks` row cannot exist without a local feed row and the feed read fails often enough to make the copy empty.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-010-copy-to-my-playlists.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decision 1)
- the files that the task lists

Goal:
- A local copy of a public playlist, with no download and no schema change.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The schema.

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
