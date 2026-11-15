# ADR 0068 Task 003: Show Plays The Cue

## Status

Implemented - 2026-11-15. Mechanical checks are Green. The visual criteria V1 to V5 are open in section 20 of [pending human checks](../pending-human-checks.md). This task was divided into five parts, and a coding model got one part at a time. This file keeps the scope, the criteria and the visual gate of the full task.

## Parts - 2026-11-14

Each part compiles and passes the commit checks alone. The parts run in this sequence, because they change the same files.

| Part | Scope items | State |
|---|---|---|
| [003a](adr-0068-task-003a-remove-playlist-play.md) | 6, and 10 for the removed paths | Implemented 2026-11-14 |
| [003b](adr-0068-task-003b-session-cue-entry-and-silent-restart.md) | 5 for the session, 9, and the open point from task 002 | Implemented 2026-11-14 |
| [003c](adr-0068-task-003c-show-start-pause-stop-select.md) | 1 and 2, 5 for the cuelist, 7, 8 for `cmd-alt-p`, 10 | Implemented 2026-11-14 |
| [003d](adr-0068-task-003d-next-previous-and-end-of-track.md) | 3 and 4, 8 for the skip keys, 10 | Implemented 2026-11-15 |
| [003e](adr-0068-task-003e-show-clear-item.md) | 11 | Implemented 2026-11-15 |

Between part 003a and part 003c, the desktop app cannot start playback. The operator does each push, so no release has this state.

## Operator Choices

The orchestrating session made these choices on 2026-11-14 inside ADR 0068. Each choice is open for the operator to change.

- Removed cue entry: a Clear or a Replace sets `playback_sessions.cue_entry_id` to NULL in its transaction. A reader that finds an entry id that is not in `show_cue_entries` treats it as no entry, and it selects no other entry. This needs no schema change. Part 003b implements the rule and its tests.
- Restart: startup sets a `playing` or `paused` session to `stopped` and keeps its track, its position and its observation time, as ADR 0068 "Persistence, Restart And CLI Compatibility" says. Part 003b implements it.
- `RssCheckTrigger::PlaybackStart` stays as a variant with no emitter, because stored rows of `rss_check_runs` hold its token.
- Select while playing or paused is not available. ADR 0068 gives a rule only for Select while stopped.
- Next or Previous while paused: the owner stops the driver, writes the target entry to the paused session and loads the entry at Resume. `PlaybackDriver::load` starts playback, so a load in pause makes a short sound. The driver trait does not change. Part 003d implements it.
- The Clear item opens a confirmation, as the playlist delete does. Part 003e implements it.
- "Stop And Ask" of this file: the only desktop caller of `play_playlist_at` is the chain that this task removes. The search of 2026-11-14 at commit `3b387f7` found no different caller.

## Goal

The Show owner plays the stored cue. Only typed Show commands start and control program playback. The playlist page has no Play that starts Show.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Show Owns Its Transport", "Publication And Authoritative State", "Commands, Keys And Layer Ownership", "Persistence, Restart And CLI Compatibility", and Answer 4 of its [revision of 2026-11-11](../adr/0068-show-cue-and-audition-isolation.md#revision-for-the-adr-0089-shell---2026-11-11).
- [ADR 0014](../adr/0014-playback-session-authoritative-state.md): the session stays the authoritative now-playing state.
- [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md): the session drain and the recovery of the player.

## Recorded Facts - 2026-11-11

- `PlaybackOwner::play_playlist_at`, `skip_next`, `skip_previous`, `pause`, `stop` and `poll` are in `src/playback_owner.rs`. They write the session through `src/playback.rs`.
- `TopApp::play_playlist_at` (`src/app/playback_bar.rs:19`) runs the command `PlayPlaylistAt` (`src/application/commands/playback.rs:112`). `PlaybackOperation` in `src/application/capability_recovery.rs` retries it.
- The only emitter of `LibraryAppEvent::PlayPlaylistAt` is `src/ui/shells/library/playlist_detail.rs:393`.
- The Show transport calls `queue_transport_action` (`src/app/queue_now_playing.rs:18`). `toggle_playback_paused` reads `show_page.queue.transport.play_pause_state`.
- The keys `TogglePlayback`, `SkipPlaybackNext` and `SkipPlaybackPrevious` have `AppKeyScope::Global` (`src/app/keyboard.rs:110-124`).
- The guards `workspace_frame_phase_4_guards_playback_commands_stay_out_of_toolbar`, `screens_do_not_call_migrated_playback_paths` and `workspace_frame_phase_4_guards_queue_now_playing_vm_contract` name the current paths.

## Files To Inspect

- `src/playback_owner.rs`, `src/playback.rs`, `src/runtime/playback_polling.rs`
- `src/application/commands/playback.rs`, `src/application/capability_recovery.rs`, `src/app/playback_bar.rs`
- `src/app/queue_now_playing.rs`, `src/view_models/queue_now_playing.rs`, `src/app/show.rs`
- `src/app/keyboard.rs`, `src/app.rs` (the `PlayPlaylistAt` handler)
- `src/ui/shells/library/playlist_detail.rs`
- `tests/architecture_tests.rs`

## Open Point From Task 002 - 2026-11-14

Migration 25 adds `playback_sessions.cue_entry_id` with no foreign key. `show_cue_entries.id` uses `AUTOINCREMENT`, so a Replace does not use an entry id again. The test `replace_never_gives_an_old_entry_id_again` proves it.

When this task writes the column, a Clear or a Replace can remove the entry of a stopped session. The session then points to an entry that is not in the table. Before the first write, give the rule for this condition and a test for it. If the rule needs a schema change, stop and ask.

## Scope

1. Typed Show commands: Start, Pause, Resume, Stop, Select, Next and Previous. Each acts on a cue entry id, not on a playlist row.
2. Start plays the selected entry. Stop ends audio and the local metadata, resets the position and keeps the cue. Select while stopped changes only the selection.
3. Next and Previous follow ADR 0068: while stopped they select, while playing they load, while paused they prepare with no sound. They are not available at the ends. An entry that cannot play is reported and is not skipped.
4. The end of a track goes to the next entry one time. At the last entry, playback stops. An observation carries the owner generation and the cue entry id, so a late end of a track cannot move a later track.
5. The session writes `cue_entry_id`. The Show cuelist always comes from the cue. The projection from the session playlist goes.
6. `LibraryAppEvent::PlayPlaylistAt`, its emitter, `TopApp::play_playlist_at`, the command `PlayPlaylistAt` and `PlaybackOperation::Playlist` go. The RSS check start on `PlayPlaylistAt` goes with them. The CLI `playlist play` stays a session simulator.
7. The Show view model gives the states empty, loaded and stopped, starting, playing, paused and failed. For each action, it gives the availability and the failure text. A command that the owner accepted is not a proof that audio plays.
8. Keys, Answer 4: the three keys are Show commands. They work on each page while a cue is loaded. With no cue, they are not available. They never start an audition.
9. Restart: the app keeps the cue and the selection, sets the session to stopped, and starts no audio.
10. Change each guard that names a removed path. Each new guard cites ADR 0068 and names the fix in its failure message.
11. The Show page gets a Clear item that runs `ClearShowCue` from task 002. Operator decision of 2026-11-14. The item is available only while playback is stopped. After a Clear, the cue menu and the Show page show no cue.

## Do Not Touch

- The second player and the audition. Packet 004 adds them.
- The now-playing panel. ADR 0089 packet 016 adds it.
- The CLI writer lease. Packet 005 adds it.

## Mechanical Criteria

- M3 of ADR 0068: Start, Pause, Stop, Select, Next and Previous obey the cue. A skip while paused loads no audible track. The last end of track stops. An entry that cannot play is reported. A second or late end of track does not move playback two times.
- M7 of ADR 0068, Show part: the Show commands have their own intents and accessible labels. With no cue, the keys are not available.
- M8 of ADR 0068, Show part: after a restart, the cue stays, and audio is stopped.
- No production code emits or handles `PlayPlaylistAt`.
- The Show view model gives the Clear item as available only while playback is stopped. A test of the Clear handler proves that `ClearShowCue` runs and that a Clear while playback plays or is paused changes nothing.
- Only the Show owner calls the drop-file producer.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: load a cue, then start it from Show. The cuelist marks the current entry, and the transport changes state.
- V2: pause, skip and stop. The cue and its order stay.
- V3: on a different page, `cmd-alt-p` pauses and resumes the show.
- V4: quit and open the app. The cue shows, and no audio plays.
- V5: stop playback, then select Clear on the Show page. The cuelist is empty, and the playlist menu shows "Load into Show Cue".

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo test --test architecture_tests
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```

## Report

Give the files changed, the removed paths, the changed guards and the tests.

## Corrections From The Agent Visual Check - 2026-11-15

The orchestrating session ran the app on an Xvfb display with mpv and a null audio output. The run found three defects, and each has a test now.

- The mpv driver took each `end-file` event as the end of a track. mpv also sends the event when a command stops or replaces a file. A Start after a Stop and a Next while a track played then moved one more entry. The driver now ignores the reasons `stop`, `quit` and `redirect`. Test: `end_file_event_of_a_stopped_or_replaced_file_is_not_eof`.
- mpv keeps its pause property across files. A Resume after a Next in pause and a Start after a Stop from pause loaded the track and stayed paused. The mpv `load` now clears the pause property first. Test: `load_clears_the_pause_of_the_file_before`.
- After a driver failure the page shows the failed state while the stored session is `playing`. The Clear button showed, and the store refused the Clear with no visible result. The Clear action is now available only when the stored session is stopped. Test: `clear_action_requires_a_stored_cue_and_stopped_playback`.

Operator choice: the two driver corrections are in `src/playback_driver/mpv.rs`. They change no trait and no command of the owner.

## Stop And Ask

Stop if a different caller of `play_playlist_at` exists outside the CLI and the tests.
