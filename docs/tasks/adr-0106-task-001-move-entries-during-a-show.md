# ADR 0106 Task 001: Move Cue Entries During A Show

## Status

Ready - 2026-11-19. It waits for no other task.

## Goal

While the Show plays or is paused, the host moves a cue entry after the current entry. The host uses a drag, Alt+Up and Alt+Down, or "Play next". The current track does not stop.

## Authority

[ADR 0106](../adr/0106-the-cue-changes-during-a-show.md) Decisions 1 to 3.

## Change

1. `src/db/show_cue.rs`: add a move function. In one transaction it reads the session state and the current entry. While stopped, it moves each entry. While playing or paused, it refuses a move that touches the current entry or an entry before it. It renumbers the positions.

   `show_cue_can_change` keeps its rule for replace and clear.
2. Add a "Play next" command. It moves the entry to the position directly after the current entry.
3. The Show cue view model gives "Played" and "Playing" with the time left. It gives the move availability of each row, and "Play next" on each row that is not next. It gives the text "Drag a row that did not play, or select "Play next". The playing track does not stop."
4. The cue rows get the drag of `src/ui/composites/track_drag.rs` and the keys Alt+Up and Alt+Down of ADR 0092. The row menu gets "Play next".

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Show cue store | extend | src/db/show_cue.rs | It gets the move function. The lock of replace and clear stays. |
| Position move | reuse | src/db/playlist_order.rs | `move_playlist` gives the pattern of a renumbered move. The cue move follows it. |
| Row drag | extend | src/ui/composites/track_drag.rs | It gets a cue row payload. |
| Row move keys | reuse | src/app/keyboard.rs | Alt+Up and Alt+Down already move a row of My playlists. |
| Show view model | extend | src/view_models/show.rs | It gives the row states and the move availability. |

## Mechanical Criteria

1. A store test: while playing, a move of an entry after the current entry to a later position succeeds.
2. A store test: while playing or paused, the store refuses three moves. They are a move of the current entry, a move of an earlier entry, and a move to a position before or at the current entry. The cue does not change.
3. A store test: while stopped, each move succeeds.
4. An owner test with a fake driver: a move during play does not call the driver. The next EOF plays the moved entry.
5. The test `replace_and_clear_require_stopped_playback` passes with no change.
6. A view model test: "Play next" shows on each row after the next row and on no other row.
7. The commit checks of AGENTS.md pass.

## Visual Criteria

These are in section 46 of [pending human checks](../pending-human-checks.md).

- V1: while the Show plays, a drag of a later row moves it, and the track does not stop.
- V2: "Play next" on the fourth row puts it after the playing row. At the end of the playing track, that row plays.
- V3: a played row and the playing row do not move.

## Report

Empty until the work is done.
