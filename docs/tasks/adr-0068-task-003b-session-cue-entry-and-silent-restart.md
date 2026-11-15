# ADR 0068 Task 003b: The Session Cue Entry And The Silent Restart

## Status

Ready after task 003a - 2026-11-14. Implementation has not started. This is part 2 of 5 of [task 003](adr-0068-task-003-show-plays-the-cue.md).

## Goal

The session can hold the cue entry that plays. A Clear or a Replace cannot leave a session that points to a removed entry. After a restart, the session is stopped and no audio plays.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Publication And Authoritative State" and "Persistence, Restart And CLI Compatibility", Invariant 7.
- [ADR 0014](../adr/0014-playback-session-authoritative-state.md): the session is the authoritative now-playing state.
- [Task 003](adr-0068-task-003-show-plays-the-cue.md), scope items 5 and 9, and its section "Operator Choices".

## Recorded Facts - 2026-11-14

Each fact was read at commit `3b387f7`.

- Migration 25 (`src/db/show_cue.rs:22`) adds `playback_sessions.cue_entry_id`, nullable, with no foreign key. No code reads it or writes it.
- `PlaybackSessionRow` is in `src/db.rs:92`. Its decoder is at line 2134, the track upsert at line 2651 and the SELECT at line 2704.
- `load_show_cue` (`src/db/show_cue.rs:236`) and `clear_show_cue` (line 289) run in one transaction each. `show_cue_can_change` (line 224) permits a change only when the session is absent or stopped.
- Startup makes the owner in `src/app/bootstrap.rs:178` and asks for the Show refresh at line 270. It does not change the session. A stored `playing` or `paused` status stays and shows in Show.
- `playback::stop` (`src/playback.rs:149`) resets the position. `SessionDrain::finish` (`src/application/session_lifecycle.rs:247`) records `stopped` through `finish_session_playback`.
- `TopApp::drop` (`src/app.rs:2133`) clears the metadata file and does not stop the stored session.

## Files To Inspect

- `src/db.rs`, `src/db/show_cue.rs`, `src/playback.rs`
- `src/app/bootstrap.rs`, `src/startup.rs`, `src/application/session_lifecycle.rs`
- `tests/architecture_tests.rs`

## Files Likely To Change

- `src/db.rs`, `src/db/show_cue.rs`, `src/playback.rs`, `src/app/bootstrap.rs`

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Playback session store | extend | `src/db.rs` | Reads and writes `cue_entry_id`, and adds the restart write. |
| Show cue storage | extend | `src/db/show_cue.rs` | Clear and Replace set the session entry to NULL. |
| Session simulator | extend | `src/playback.rs` | Gives the session entry to its callers. |
| Startup sequence | extend | `src/app/bootstrap.rs` | Sets the session to stopped before the first Show refresh. |

## Scope

1. `PlaybackSessionRow` gets `cue_entry_id: Option<i64>`. The decoder and the SELECT read it.
2. Add one store function that sets the track and the cue entry of a session in one statement. The existing track upsert writes NULL to `cue_entry_id`, so a CLI write never keeps an old entry.
3. Rule for a removed entry, from the parent task: `load_show_cue` and `clear_show_cue` set `playback_sessions.cue_entry_id` to NULL in their transaction. A reader that finds an entry id that is not in `show_cue_entries` treats it as no entry. It never selects a different entry.
4. Add a store function that sets a session to `stopped` and keeps its track, its position and its observation time. Do not use `playback::stop` for this, because it resets the position.
5. Startup calls the function of item 4 before the first Show refresh. It does this only when the status of the default session is `playing` or `paused`. It starts no driver and loads no track.
6. Add no schema change. Schema version 25 stays.

## Do Not Touch

- `PlaybackOwner`, the commands, the view models and the Show page. Task 003c changes them.
- The schema and `MIGRATIONS`.
- `src/cli.rs` and the JSON contracts of the CLI.
- `src/assets/tokens.json`.

## Constraints

- Blocking database access stays off the render thread. Use the path that startup already uses for its database reads.
- Write each comment in Simplified Technical English. Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree.

## Mechanical Criteria

- A store test: a session with a cue entry, then a Replace. The session entry is NULL, and the track of the session stays.
- A store test: the same for a Clear.
- A store test: a session row that holds an entry id that is not in the table. The reader gives no entry.
- A store test: the restart write keeps the track, the position and the observation time, and the status is `stopped`.
- A startup test at the layer that owns the startup step: a stored `playing` session is `stopped` after the step, and the owner has no loaded track.
- A CLI track write after a cue write leaves `cue_entry_id` NULL.
- The test commands pass.

## Visual Criteria

None. V4 of the parent task covers the restart.

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

- Stop if the rule of scope item 3 needs a schema change.
- Stop if startup has no place for the restart write before the first Show refresh.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/adr-0068-task-003b-session-cue-entry-and-silent-restart.md`
- `AGENTS.md`, the sections "Working Rules" and "Conventions"
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`
- The files of "Files To Inspect"

Goal:
- The session reads and writes its cue entry, a Clear or a Replace sets it to NULL, and startup sets the session to stopped.

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
