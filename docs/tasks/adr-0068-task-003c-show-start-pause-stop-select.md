# ADR 0068 Task 003c: Show Starts, Pauses, Stops And Selects A Cue Entry

## Status

Ready - 2026-11-14. Implementation has not started. This is part 3 of 5 of [task 003](adr-0068-task-003-show-plays-the-cue.md).

## Goal

Typed Show commands start, pause, resume and stop the selected cue entry, and select an entry. The Show cuelist always comes from the cue. The key `cmd-alt-p` is a Show command.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Show Owns Its Transport", "Publication And Authoritative State", "Commands, Keys And Layer Ownership", and Answer 4 of its revision of 2026-11-11.
- [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md): the retained action of a repair and the session drain.
- [Task 003](adr-0068-task-003-show-plays-the-cue.md), scope items 1, 2, 5, 7, 8 and 10.

## Recorded Facts - 2026-11-14

Each fact was read at commit `3b387f7`. Tasks 003a and 003b change some of these lines.

- `PlaybackOwner` (`src/playback_owner.rs:29`) has `eof_armed` and `loaded_track_id`. It has no generation and no cue entry. `load_track_path` (line 111), `pause` (186), `stop` (219) and `poll` (241) write the session through `src/playback.rs`.
- The polling snapshot (`src/runtime/playback_polling.rs:24`) holds an `Instant` and an outcome. Polling starts after a playback command in `src/app/playback_bar.rs:212`.
- `queue_now_playing_input` (`src/app/queue_now_playing.rs:35`) reads the cue only when no session is active (line 45). With an active session it calls `queue_tracks_for_session` (line 127). The cue rows do not give `selected_entry_id` (line 102). Their skip flags are false (line 114).
- The transport states are `Stopped`, `Playing` and `Paused` (`src/view_models/queue_now_playing.rs:21`). The transport is not available while stopped (line 183).
- `toggle_playback_paused` (`src/app/playback_bar.rs:64`) sends Resume when paused and Pause in each other state. It cannot start a stopped cue.
- `TogglePlayback` is defined in `src/app/keyboard.rs:20`, bound at line 110 and handled at line 368. The handler does not read the cue.
- The guards `workspace_frame_phase_4_guards_queue_frame_shell_wiring` (`tests/architecture_tests.rs:4275`), `workspace_frame_phase_4_guards_queue_now_playing_vm_contract` (4223), `adr_0060_live_status_and_show_share_cached_projection` (14440), `adr_0066_optional_dependencies_are_scoped` (16322) and `adr_0066_repair_routes_preserve_action_subject` (17019) name the current paths.

## Files To Inspect

- `src/playback_owner.rs`, `src/playback.rs`, `src/db/show_cue.rs`, `src/runtime/playback_polling.rs`
- `src/application/commands/playback.rs`, `src/application/capability_recovery.rs`, `src/view_models/startup/capabilities.rs`
- `src/app/playback_bar.rs`, `src/app/queue_now_playing.rs`, `src/app/show.rs`, `src/app/keyboard.rs`, `src/app.rs`
- `src/view_models/queue_now_playing.rs`, `src/view_models/show.rs`
- `src/ui/shells/show.rs`, `src/ui/shells/queue_now_playing.rs`
- `tests/architecture_tests.rs`

## Files Likely To Change

The files of "Files To Inspect".

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Show playback owner | extend | `src/playback_owner.rs` | Plays a cue entry and counts a generation. |
| Show cue storage | extend | `src/db/show_cue.rs` | Writes the selected entry. |
| Playback command set | extend | `src/application/commands/playback.rs` | Adds the typed Show commands. |
| Retained action of a repair | extend | `src/application/capability_recovery.rs` | Keeps the Show command and its entry for a retry. |
| Polling actor | extend | `src/runtime/playback_polling.rs` | The snapshot carries the generation and the entry. |
| Queue view model | extend | `src/view_models/queue_now_playing.rs` | Adds the states and the action availability. |
| Queue projection | extend | `src/app/queue_now_playing.rs` | Reads the cue in each state. |
| Key binding table | extend | `src/app/keyboard.rs` | `cmd-alt-p` becomes a Show command. |
| Cuelist row | reuse | `src/ui/shells/queue_now_playing.rs` | The row gets a select action from the view model. |

## Scope

1. The owner gets a generation counter and the active cue entry id. Each load, each stop and each driver change adds one to the generation.
2. Typed commands `StartShow`, `PauseShow`, `ResumeShow`, `StopShow` and `SelectShowCueEntry`. Each command names a cue entry id or acts on the selected entry. No command names a playlist row.
3. Start loads the selected entry and writes the track and `cue_entry_id` with `db::set_playback_session_track_with_cue` of task 003b. Start of an entry that cannot play reports the entry and its reason, and changes nothing.
4. Pause and Resume act on the active entry. Stop ends audio, clears the local metadata, resets the position and keeps the cue and its selection.
5. Select while stopped writes only `show_cue.selected_entry_id`. Select while playing or paused is not available in this task.
6. The cuelist comes from the cue in each state. Remove `queue_tracks_for_session` and the projection from the session playlist. The active row is the entry of the session. The selected row is `selected_entry_id`.
7. The view model gives the states empty, loaded and stopped, starting, playing, paused and failed. It gives the availability, the label and the accessibility label of Start, Pause, Resume, Stop and Select, and the failure text. The state is starting from the command until the first observation of the driver proves playback.
8. Next and Previous stay not available in this task. Task 003d adds them. Until then, the end of a track stops playback and keeps the cue.
9. The polling snapshot carries the generation and the cue entry id. The reducer drops an observation whose generation is not the current generation.
10. `TogglePlayback` becomes a Show command with a Show label: Start when stopped, Pause when playing, Resume when paused. With no cue, the action is not available, and the handler does nothing. It works on each page.
11. Change each guard that names a removed path. Each new guard cites ADR 0068 and names the fix in its failure message.

## Do Not Touch

- The second player, the audition and the now-playing panel.
- The CLI and `playback::play_playlist_at`.
- The Clear item. Task 003e adds it.
- The schema and `MIGRATIONS`.
- `src/assets/tokens.json`.

## Constraints

- A view model carries no renderer type. A renderer does not decide if a control is available.
- Do not call `cx.spawn` from a screen. Background work uses the ADR 0040 runtime.
- A new control uses an existing primitive or composite and named tokens. Add no token.
- Only the owner calls the drop-file producer.
- Write each comment in Simplified Technical English. Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Do not run the app. Leave the change in the working tree.

## Mechanical Criteria

- Owner tests with a recording driver: Start loads the selected entry and writes `cue_entry_id`. Pause and Resume keep the entry. Stop resets the position and keeps the cue and the selection.
- An owner test: Start of an entry that cannot play gives a typed error with the entry, and the session does not change.
- A store test: Select while stopped changes only `selected_entry_id`.
- An owner test: the end of a track stops playback, and a second end of track changes nothing.
- A reducer test: an observation with an old generation changes nothing.
- View model tests: the six states, and the availability of each action in each state. An accepted Start gives starting, not playing.
- A key test: with no cue, `TogglePlayback` is not available. With a cue, it gives the Show intent with its accessibility label.
- No code calls `queue_tracks_for_session`. The changed guard proves it.
- The test commands pass.

## Visual Criteria

None in this file. V1, V2, V3 and V4 of the parent task cover this part.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo test --test architecture_tests
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```

## Stop And Ask

- Stop if a state or an action needs a new token or a new primitive.
- Stop if a state or an action needs a change to a shared interface rule of musicindex.
- Stop if the owner cannot give a generation without a change to the `PlaybackDriver` trait.
- Stop if a reader of the session playlist projection exists outside `src/app/queue_now_playing.rs`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/adr-0068-task-003c-show-start-pause-stop-select.md`
- `docs/adr/0068-show-cue-and-audition-isolation.md`, the sections "Show Owns Its Transport" and "Commands, Keys And Layer Ownership"
- `AGENTS.md`, the sections "The Durable Set", "Working Rules" and "Conventions"
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`
- The files of "Files To Inspect"

Goal:
- Typed Show commands start, pause, resume, stop and select a cue entry. The cuelist comes from the cue in each state.

Constraints:
- The section "Constraints" of the task file. Do not commit.

Do not touch:
- The section "Do Not Touch" of the task file.

Acceptance criteria:
- The section "Mechanical Criteria" of the task file.

Test commands:
- The section "Test Commands" of the task file.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
