# ADR 0106 Task 001: Move Cue Entries During A Show

## Status

Implemented - 2026-11-19. The mechanical criteria pass. The visual criteria wait for a person, in section 46 of the pending human checks.

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

Changed files:

- `src/db/show_cue.rs`: `move_show_cue_entry`, `play_show_cue_entry_next` and four store tests.
- `src/db.rs`: exports of the two functions.
- `src/application/commands/playback.rs`: command `MoveShowCueEntry` and enum `CueEntryPlacement`.
- `src/view_models/queue_now_playing.rs`: row state, move availability, "Play next" flag, hint text, `key_move_index` and three tests.
- `src/app/queue_now_playing.rs`: gives the play position of the current entry to the view model.
- `src/ui/composites/track_drag.rs`: `CueRowDragPayload` and `attach_cue_row_drag`. The drop target `cue_row_drop_target` is in `src/ui/shells/queue_now_playing.rs`, because the guard of ADR 0101 task 008 allows three `.on_drop(` handlers in `track_drag.rs`.
- `src/ui/shells/queue_now_playing.rs`, `src/ui/shells/show.rs`, `src/ui/composites/show_detail_panel.rs`: cue row drag and drop, the state label, the "Play next" button, the row menu and the hint text.
- `src/app.rs`, `src/app/playback_bar.rs`, `src/app/show.rs`, `src/app/keyboard.rs`: move handler, the marked row, and Alt+Up and Alt+Down on the Show page.
- `src/playback_owner.rs`: one owner test with a counting driver.
- Constructors of `QueueTrackInput` in tests: `src/view_models/live_status.rs`, `src/view_models/show.rs`, `src/view_models/status_bar.rs`.

Test for each mechanical criterion:

1. `a_move_after_the_current_entry_succeeds_while_playing` in `src/db/show_cue.rs`.
2. `active_playback_refuses_a_move_that_touches_the_current_entry_or_before` in `src/db/show_cue.rs`. It covers "playing" and "paused".
3. `stopped_session_moves_each_entry` in `src/db/show_cue.rs`.
4. `a_cue_move_during_play_does_not_call_the_driver_and_eof_plays_the_moved_entry` in `src/playback_owner.rs`.
5. `replace_and_clear_require_stopped_playback` passes with no change.
6. `play_next_shows_after_the_next_row_and_on_no_other_row` in `src/view_models/queue_now_playing.rs`.
7. The commit checks pass: `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check` and `cargo test --test architecture_tests`.

Deviations and open points:

- The task names `src/view_models/show.rs`. The cue rows are in `src/view_models/queue_now_playing.rs`, so the change is there.
- Each cue row has no focus handle. Alt+Up and Alt+Down move the row that the host clicked last. In a stopped Show, they move the selected row. No test covers the keys in the window. Visual check V1 to V3 must include the keys.
- The row menu is a button with the icon "More" and one item "Play next". It is not a right-click menu.
- While the Show plays and has no current entry, the store refuses each move.
- "Played" and "Playing, M:SS left" show the time left at the last refresh of the page. No test covers the refresh interval.

Main session review: the move limit now compares the index of the current entry in the cue, not its stored position. The result is the same while the positions are 0, 1, 2 and so on.
