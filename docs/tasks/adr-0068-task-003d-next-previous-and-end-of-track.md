# ADR 0068 Task 003d: Next, Previous And The End Of A Track

## Status

Implemented - 2026-11-15. Mechanical checks are Green. This file has no visual criterion. This is part 4 of 5 of [task 003](adr-0068-task-003-show-plays-the-cue.md).

## Goal

Next and Previous move through the cue entries. The end of a track goes to the next entry one time. A late end of track cannot move a later track.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Show Owns Its Transport", paragraphs 3 and 4, and Answer 4 of its revision of 2026-11-11.
- [Task 003](adr-0068-task-003-show-plays-the-cue.md), scope items 3, 4, 8 and 10.

## Recorded Facts - 2026-11-14

Each fact was read at commit `3b387f7`. Tasks 003a to 003c change some of these lines.

- `PlaybackOwner::skip_next` (`src/playback_owner.rs:197`) and `skip_previous` (line 204) call `playback::skip_next` and `playback::skip_previous`. Those functions select a playable row of the session playlist (`src/playback.rs:153`). They do not keep a pause.
- The end of a track is handled in `PlaybackOwner::poll` at line 278. The owner disarms `eof_armed`, moves, loads and arms again.
- `SkipPlaybackNext` and `SkipPlaybackPrevious` are defined in `src/app/keyboard.rs:21` and `:22`, bound at lines 116 and 122, and handled at lines 377 and 386. The handlers do not read the view model flags.
- The CLI commands `playback next` and `playback previous` use `playback::skip_next` and `playback::skip_previous`.

## Facts After Task 003c - commit `2843451`

- `PlaybackOwner` has `generation`, `active_cue_entry_id`, `start_show_entry` and `advance_generation`. Use `start_show_entry` as the pattern for a load of a cue entry.
- The typed error `ShowCueEntryCannotPlay` in `src/playback_owner.rs` holds the entry and the reason. Use it for a target entry that cannot play. Do not add a second error type.
- In `PlaybackOwner::poll`, the end of a track of a session with a cue entry stops playback. The branch for a session with no cue entry calls `playback::skip_next`. Scope item 2 removes that call.
- `QueueNowPlayingPageVm::toggle_intent` and `can_stop_show` in `src/view_models/queue_now_playing.rs` give the intent and the availability to the handlers in `src/app/playback_bar.rs`. Do the same for Next and Previous.
- `QueueNowPlayingPageVm::set_transport_state` sets the two skip flags to false. Keep the skip availability when a state override applies.
- `presentation_after_command` in `src/app/playback_bar.rs` gives the starting state after an accepted Start.
- The guard `adr_0066_optional_dependencies_are_scoped` counts four owner availability calls in `src/app/playback_bar.rs`.

## Paused Move Without A Driver Change - 2026-11-14

The first run stopped, because `PlaybackDriver::load` starts playback and the trait has no load in pause. The orchestrating session confirmed this in `src/playback_driver/mpv.rs` and made this choice. The trait does not change.

- The owner gets the field `prepared_cue_entry_id: Option<i64>`. It holds a cue entry that the session shows as paused and that the driver did not load.
- Next or Previous while paused calls `driver.stop()`, which makes no sound. It does not call `driver.load`. It adds one to the generation. It writes the track and the cue entry of the target to the session with position 0, and the session stays `paused`. It sets `active_cue_entry_id` and `loaded_track_id` to the target, and `prepared_cue_entry_id` to the target entry.
- While `prepared_cue_entry_id` is set and the session is `paused`, `poll` does not load, does not call `driver.poll` and gives `PollOutcome::Reconciled(None)`.
- Resume with a prepared entry calls `driver.load` of that entry at position 0 in place of `driver.pause(false)`. It adds one to the generation, arms the end of track and clears `prepared_cue_entry_id`. When the file of the entry is missing, Resume gives `ShowCueEntryCannotPlay` and the pause stays.
- Stop, Start and each load clear `prepared_cue_entry_id`.
- The owner test for the paused move uses a driver that records each call. It proves that the move sends only `stop`, and that Resume sends one `load`.

## Files To Inspect

- `src/playback_owner.rs`, `src/playback.rs`, `src/db/show_cue.rs`, `src/runtime/playback_polling.rs`
- `src/application/commands/playback.rs`, `src/application/capability_recovery.rs`
- `src/app/playback_bar.rs`, `src/app/keyboard.rs`, `src/app/queue_now_playing.rs`
- `src/view_models/queue_now_playing.rs`
- `tests/architecture_tests.rs`

## Files Likely To Change

The files of "Files To Inspect", but not `src/playback.rs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Show playback owner | extend | `src/playback_owner.rs` | Moves through cue entries and handles the end of a track. |
| Show cue storage | extend | `src/db/show_cue.rs` | Gives the entry before and after an entry. |
| Playback command set | extend | `src/application/commands/playback.rs` | Adds `NextShowCueEntry` and `PreviousShowCueEntry`. |
| Queue view model | extend | `src/view_models/queue_now_playing.rs` | Gives the availability of Next and Previous. |
| Key binding table | extend | `src/app/keyboard.rs` | The two skip keys become Show commands. |
| Session simulator | reuse | `src/playback.rs` | The CLI keeps its playlist skips. |

## Scope

1. A store function gives the entry directly before and directly after a cue entry, by position. It does not skip an entry that cannot play.
2. Typed commands `NextShowCueEntry` and `PreviousShowCueEntry`. They replace the owner calls to the playlist skips. The owner no longer calls `playback::skip_next` or `playback::skip_previous`.
3. While stopped, Next and Previous change only the selection.
4. While playing, they load the adjacent entry and write the session.
5. While paused, they select and prepare the adjacent entry and keep the pause. The driver gets no command that makes sound. The recording driver of the tests proves this.
6. At the first entry, Previous is not available. At the last entry, Next is not available.
7. When the target entry cannot play, the command reports the entry and its reason. The current state stays.
8. The end of a track loads the next entry one time. At the last entry, playback stops and the cue stays. When the next entry cannot play, playback stops, and the Show view model reports that entry.
9. The owner accepts an end of track only when its generation and its cue entry id are the current values. Each load adds one to the generation, as task 003c decides.
10. The keys `SkipPlaybackNext` and `SkipPlaybackPrevious` become Show commands with Show labels. They work on each page while a cue is loaded. They read the availability of the view model.
11. Change each guard that names a removed path. Each new guard cites ADR 0068 and names the fix in its failure message.

## Do Not Touch

- `src/playback.rs` and `src/cli.rs`. The CLI stays a session simulator.
- The second player, the audition and the now-playing panel.
- The Clear item. Task 003e adds it.
- `src/assets/tokens.json`.

## Constraints

- A view model carries no renderer type. A renderer does not decide if a control is available.
- Only the owner calls the drop-file producer.
- Write each comment in Simplified Technical English. Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Do not run the app. Leave the change in the working tree.

## Mechanical Criteria

- Owner tests with a recording driver, one for each state: stopped selects, playing loads, paused prepares with no sound.
- A cue with the same track two times: Next moves by entry, not by track.
- A source playlist that changed after the load does not change Next and Previous.
- Next at the last entry and Previous at the first entry are not available in the view model, and the commands change nothing.
- A target entry that cannot play gives a typed error with the entry, and the state stays.
- The end of a track moves one entry. A second end of track with the same generation changes nothing. An end of track with an old generation changes nothing.
- The end of the last entry stops playback and keeps the cue.
- A next entry that cannot play at the end of a track stops playback, and the view model names the entry.
- With no cue, the two skip keys are not available.
- The owner does not call `playback::skip_next` or `playback::skip_previous`. A guard that cites ADR 0068 proves it.
- The test commands pass.

## Visual Criteria

None in this file. V2 of the parent task covers this part.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo test --test architecture_tests
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```

## Result - 2026-11-15

A ChatGPT run made the change. The orchestrating session examined the diff and ran each check.

- The first run stopped at the driver trait. The section "Paused Move Without A Driver Change" records the choice that followed.
- `PlaybackOwner::move_show_cue` replaces `skip_next` and `skip_previous` of the owner. The commands are `NextShowCueEntry` and `PreviousShowCueEntry`.
- The end of a track of a session with no cue entry stops playback. Before, it moved through the session playlist.
- The new guard is `adr_0068_show_transport_moves_through_cue_entries`.
- The orchestrating session added four tests: the move while stopped, a target that cannot play, the end of a track before an entry that cannot play, and the skip availability at the cue boundaries and with no cue.

Deviations from this file:

- An entry that cannot play at the end of a track gives the typed error to the polling outcome. The status line of the app names the entry, and the view model gives the failed state until the subsequent observation.
- The owner holds its lock for one poll, so an end of track from an old generation cannot reach the move. The owner compares the cue entry of the session with its active entry before it moves. No test sends an old generation to the owner.
- The accessibility labels of the two transport buttons stay "Previous track" and "Next track".

## Stop And Ask

- Stop if the section "Paused Move Without A Driver Change" cannot be done without a change to the `PlaybackDriver` trait.
- Stop if a desktop caller of the playlist skips exists outside the owner.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/adr-0068-task-003d-next-previous-and-end-of-track.md`
- `docs/adr/0068-show-cue-and-audition-isolation.md`, the section "Show Owns Its Transport"
- `AGENTS.md`, the sections "The Durable Set", "Working Rules" and "Conventions"
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`
- The files of "Files To Inspect"

Goal:
- Next, Previous and the end of a track move through the cue entries, and a late end of track changes nothing.

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
