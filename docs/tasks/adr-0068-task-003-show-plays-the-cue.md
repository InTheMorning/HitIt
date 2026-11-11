# ADR 0068 Task 003: Show Plays The Cue

## Status

Ready after packet 002 - 2026-11-11. Implementation has not started.

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

## Do Not Touch

- The second player and the audition. Packet 004 adds them.
- The now-playing panel. ADR 0089 packet 016 adds it.
- The CLI writer lease. Packet 005 adds it.

## Mechanical Criteria

- M3 of ADR 0068: Start, Pause, Stop, Select, Next and Previous obey the cue. A skip while paused loads no audible track. The last end of track stops. An entry that cannot play is reported. A second or late end of track does not move playback two times.
- M7 of ADR 0068, Show part: the Show commands have their own intents and accessible labels. With no cue, the keys are not available.
- M8 of ADR 0068, Show part: after a restart, the cue stays, and audio is stopped.
- No production code emits or handles `PlayPlaylistAt`.
- Only the Show owner calls the drop-file producer.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: load a cue, then start it from Show. The cuelist marks the current entry, and the transport changes state.
- V2: pause, skip and stop. The cue and its order stay.
- V3: on a different page, `cmd-alt-p` pauses and resumes the show.
- V4: quit and open the app. The cue shows, and no audio plays.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo test --test architecture_tests
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin v4vmm
git diff --check
```

## Report

Give the files changed, the removed paths, the changed guards and the tests.

## Stop And Ask

Stop if a different caller of `play_playlist_at` exists outside the CLI and the tests.
