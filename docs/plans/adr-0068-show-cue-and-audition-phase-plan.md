# ADR 0068 Show Cue And Audition Phase Plan

## Status

Active - 2026-11-11. This plan is advisory. It states no rule. [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md) owns each rule, with its [revision for the ADR 0089 shell](../adr/0068-show-cue-and-audition-isolation.md#revision-for-the-adr-0089-shell---2026-11-11). The operator accepted ADR 0068 on 2026-11-11.

## Goal

The curator loads a playlist into a stored Show cue and starts it from Show. A second player auditions a track or an album on a monitor output. An audition never changes the Show state or the broadcast metadata.

## Non-Goals

- No Settings page for the audio outputs. ADR 0069 phase 005 adds it after packet 004. Until then, the configuration file holds the audition output.
- No streaming of a track that is not downloaded, no mixing, no waveform and no audition playlist (ADR 0068).
- No more than one stored show, no append to a cue, and no move of one cue entry.
- No play history.

## Assumptions

Each fact below was read on 2026-11-11 at commit `ff0aca4`.

- `PlaybackOwner` (`src/playback_owner.rs`) is the only player. `TopApp` holds it as `Option<Arc<Mutex<PlaybackOwner<ConfiguredPlaybackDriver>>>>`. The polling actor in `src/runtime/playback_polling.rs` and `SessionDrain` in `src/application/session_lifecycle.rs` hold the same `Arc`.
- The owner writes the session `DEFAULT_SESSION_ID` ("default") through `src/playback.rs`. It also owns the drop-file producer.
- The mpv socket is `mpv-{pid}.sock` in `$XDG_RUNTIME_DIR/mpv` (`src/playback_driver/mpv.rs:159`). A second driver in the same process collides with it. mpv starts with `--idle`, `--no-video` and `--input-ipc-server` only. No audio device argument exists.
- `PlaybackConfig` (`src/config.rs:101`) has `driver` and `mpv_path`. No output field exists.
- `LibraryAppEvent::PlayPlaylistAt` has one emitter, `src/ui/shells/library/playlist_detail.rs:393`. Its handler in `src/app.rs:418` starts playback and the RSS check with `RssCheckTrigger::PlaybackStart`.
- The Show cuelist is not stored. `queue_now_playing_vm` in `src/app/queue_now_playing.rs` reads it from `playback_sessions.playlist_id` and the current playlist rows.
- `playback_sessions` has no entry table. Schema version 24 is current. The migration registry is `MIGRATIONS` in `src/db.rs`.
- Library removal does not read `playback_sessions`. `db::tracks_in_use_by_show` (`src/db.rs:2815`) gives the tracks in use for the tag update.
- The keys `cmd-alt-p`, `cmd-alt-right` and `cmd-alt-left` have `AppKeyScope::Global` (`src/app/keyboard.rs`).
- The CLI commands `playlist play` and `playback …` change the session with no player. `now-playing --json` only reads it.
- The mpv IPC error of 2026-09-11 is `read mpv IPC message: Resource temporarily unavailable (os error 11)`. No test reproduces it.

## Packet Register

| Packet | Scope | Depends on | State |
|---|---|---|---|
| [001](../tasks/adr-0068-task-001-mpv-ipc-read-error.md) | Find the cause of the mpv IPC read error, fix it and add a regression test | None | Ready |
| [002](../tasks/adr-0068-task-002-stored-show-cue.md) | The stored cue (migration 25), Load, Replace and Clear, the cue in Show while nothing plays, removal and tag-update checks, the RSS check trigger | None | Ready |
| [003](../tasks/adr-0068-task-003-show-plays-the-cue.md) | The Show owner plays the cue. Music Play to Show goes. The Show keys, the restart and the Show states | 002 | Ready after 002 |
| [004](../tasks/adr-0068-task-004-audition-player.md) | A second driver instance, the audition owner and the audition output | 001, 003 | Ready after 001 and 003 |
| [005](../tasks/adr-0068-task-005-cli-writer-lease.md) | The CLI cannot change the session while the desktop owns it | 003 | Ready after 003 |
| [ADR 0089 packet 016](../tasks/adr-0089-task-016-now-playing-panel.md) | The now-playing panel with a Show part and an Audition part, and the row action Play | 003, 004 | Waits for 003 and 004 |

## Sequence

Packets 001 and 002 can run at the same time, because they change different files. Packet 003 follows 002. Packet 004 follows 001 and 003, because it changes `src/playback_driver/mpv.rs` and the owner. Packet 005 follows 003. ADR 0089 packet 016 is last.

## Final Gate

The [operator visual and audio check](../adr/0068-show-cue-and-audition-isolation.md#operator-visual-and-audio-check) of ADR 0068 runs after ADR 0089 packet 016. Packet 016 supplies its fixture commands. The check needs a Linux desktop, mpv, two audio outputs and a capture of the program output. ADR 0066 task 004 keeps its paused playback checks until this gate passes.
