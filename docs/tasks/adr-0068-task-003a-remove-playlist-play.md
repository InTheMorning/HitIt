# ADR 0068 Task 003a: Remove The Playlist Play That Starts Show

## Status

Ready - 2026-11-14. Implementation has not started. This is part 1 of 5 of [task 003](adr-0068-task-003-show-plays-the-cue.md).

## Goal

No page of Music starts program playback. The event `PlayPlaylistAt` and each path that it uses go. The CLI stays a session simulator.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Two Playback Purposes", Invariant 3 and "Follow-Up Work": no legacy GUI route stays.
- [Task 003](adr-0068-task-003-show-plays-the-cue.md), scope items 6 and 10.

## Recorded Facts - 2026-11-14

Each fact was read at commit `3b387f7`.

- `LibraryAppEvent::PlayPlaylistAt` is defined in `src/library.rs:87`. Its only emitter is `src/ui/shells/library/playlist_detail.rs:408`. Its handler is `src/app.rs:424`.
- `TopApp::play_playlist_at` is in `src/app/playback_bar.rs:19`. It makes `PlaybackOperation::Playlist` at line 27 and the command at line 37. The retry is at lines 103 to 107.
- The command `PlayPlaylistAt` is in `src/application/commands/playback.rs:112`. Its tests start at lines 528, 555 and 584.
- `PlaybackOperation::Playlist` is in `src/application/capability_recovery.rs:34`, with a check at line 187 and a test at line 978. Its label is in `src/view_models/startup/capabilities.rs:748`.
- `PlaybackOwner::play_playlist_at` is in `src/playback_owner.rs:131`. The tests at lines 547 and 586 call it.
- `playback::play_playlist_at` in `src/playback.rs:70` has a second caller, the CLI in `src/cli.rs:406`. It stays.
- `RssCheckTrigger::PlaybackStart` has one production emitter, `src/app.rs:436`. Stored rows of `rss_check_runs` can hold its token.
- The test `adr_0076_playlist_check_playback_start_runs_without_delaying_playback` (`src/application/commands/playlist.rs:776`) uses the command and reads the handler text of `src/app.rs`.
- The guards name the path in `tests/architecture_tests.rs:169` and `:4404`. The guard `adr_0066_optional_dependencies_are_scoped` (line 16322) counts four owner availability calls in `src/app/playback_bar.rs`. The guard `adr_0066_repair_routes_preserve_action_subject` (line 17019) counts seven retry checks.

## Files To Inspect

- `src/library.rs`, `src/ui/shells/library/playlist_detail.rs`, `src/view_models/library.rs`
- `src/app.rs`, `src/app/playback_bar.rs`
- `src/application/commands/playback.rs`, `src/application/commands/playlist.rs`, `src/application/capability_recovery.rs`
- `src/view_models/startup/capabilities.rs`, `src/playback_owner.rs`
- `tests/architecture_tests.rs`

## Files Likely To Change

The files of "Files To Inspect".

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Playback command set | extend | `src/application/commands/playback.rs` | Loses the command `PlayPlaylistAt`. |
| Retained action of a repair | extend | `src/application/capability_recovery.rs` | Loses `PlaybackOperation::Playlist`. |
| Playlist page rows | extend | `src/ui/shells/library/playlist_detail.rs` | Loses the row control that starts Show. |
| Session simulator | reuse | `src/playback.rs` | The CLI keeps `playback::play_playlist_at`. |

## Scope

1. Remove `LibraryAppEvent::PlayPlaylistAt`, its emitter and its handler. Remove the row control that sent it, with its view model field when no other code reads the field.
2. Remove `TopApp::play_playlist_at`, the command `PlayPlaylistAt`, `PlaybackOperation::Playlist` with its retry, its check and its label, and `PlaybackOwner::play_playlist_at`.
3. Remove the RSS check start in the removed handler. Keep the variant `RssCheckTrigger::PlaybackStart` and its token, because stored rows hold it. Give the variant a comment that says this.
4. Remove each test that tests only a removed path. An owner test that uses `play_playlist_at` only to load a track uses `load_track` or the session simulator in its place.
5. Remove the test `adr_0076_playlist_check_playback_start_runs_without_delaying_playback`. The trigger `CueLoad` of task 002 has its own test.
6. Change each guard that names a removed path. A changed count gives the new count. Add one guard that cites ADR 0068: no file in `src/` except `src/playback.rs` and `src/cli.rs` contains `play_playlist_at` or `PlayPlaylistAt`. Its failure message names ADR 0068 Invariant 3 and says "Start program playback only with a typed Show command".

## Do Not Touch

- `src/playback.rs` and `src/cli.rs`.
- The Show page, the Show view model, the keys and the owner methods that stay.
- The cue commands of task 002.
- `src/assets/tokens.json`.

## Constraints

- Do not add an audition control. Packet 004 and ADR 0089 packet 016 add it.
- Do not leave a dead function, a dead field or an unused import.
- Write each comment in Simplified Technical English. Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree.

## Mechanical Criteria

- No production code emits or handles `PlayPlaylistAt`. The new guard proves it.
- The CLI test of `playlist play` passes without a change.
- A stored `rss_check_runs` row with the token of `PlaybackStart` decodes. A test proves it, or an existing test already does.
- The test commands pass.

## Visual Criteria

None. The parent task holds the visual criteria.

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

- Stop if a caller of a removed path exists that this file does not list.
- Stop if the removal of the row control needs a change to a shared primitive or composite.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/adr-0068-task-003a-remove-playlist-play.md`
- `AGENTS.md`, the sections "Working Rules" and "Conventions"
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`
- The files of "Files To Inspect"

Goal:
- Remove the event `PlayPlaylistAt` and each desktop path that it uses. Keep the CLI session simulator.

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
