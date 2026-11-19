# ADR 0106 Task 002: A Load Opens The Show

## Status

Ready - 2026-11-19. It waits for no other task.

## Goal

"Load into Show cue" opens the Show page. The empty Show page lists My playlists, each with "Load into Show cue".

## Authority

[ADR 0106](../adr/0106-the-cue-changes-during-a-show.md) Decision 4.

## Change

1. After a load succeeds (`src/library.rs:790`), the app selects Show (`src/app.rs:439`). A failed load stays in Music with its error.
2. When the cue is empty, the Show view model gives the text "Select a playlist for your show.". It also gives the rows of My playlists in the Curator Order, each with "Load into Show cue". This replaces "No active show" and "Show playback is idle." (`src/view_models/show.rs:1864`).
3. The row action "Open playlists" of ADR 0105 opens this list. When ADR 0105 task 001 is not built, this item waits.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Show cue load | reuse | src/library.rs | The load command does not change. |
| Destination select | extend | src/app.rs | The success of a load selects Show. |
| Show view model | extend | src/view_models/show.rs | The empty state gives the text and the playlist rows. |
| Playlist list rows | reuse | src/ui/composites/list_row.rs | The rows of My playlists. |

## Mechanical Criteria

1. A test: a successful load selects Show. A failed load keeps Music and gives the error.
2. A view model test: an empty cue gives the text and one row for each playlist in the Curator Order, each with "Load into Show cue".
3. No source file has the text "Show playback is idle.".
4. The commit checks of AGENTS.md pass.

## Visual Criteria

These are in section 46 of [pending human checks](../pending-human-checks.md).

- V4: "Load into Show cue" on a playlist opens Show with that cue.
- V5: with an empty cue, Show lists the playlists. "Load into Show cue" on a row loads it.

## Report

Empty until the work is done.
