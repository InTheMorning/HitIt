# ADR 0068 Task 003e: The Clear Item Of The Show Page

## Status

Ready - 2026-11-15. Implementation has not started. This is part 5 of 5 of [task 003](adr-0068-task-003-show-plays-the-cue.md).

## Goal

The Show page has a Clear item that removes the stored cue. The item is available only while playback is stopped.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "The Show Cue Exists Before Playback", last paragraph, and the row "Show transport" of its revision table.
- Operator decision of 2026-11-14, item L4 of [task 002](adr-0068-task-002-stored-show-cue.md).
- musicindex ADR 0064: the place and the confirmation of a destructive button.
- [Task 003](adr-0068-task-003-show-plays-the-cue.md), scope item 11.

## Recorded Facts - 2026-11-14

Each fact was read at commit `3b387f7`.

- `ClearShowCue` is in `src/application/commands/playlist.rs:87`, with `execute` at line 101. No production code calls it.
- `clear_show_cue` (`src/db/show_cue.rs:289`) checks the session state and removes the entries and the header in one transaction.
- A successful load emits `ShowCueLoaded` (`src/library/app_impl.rs:2181`). `src/app.rs:441` handles it and refreshes Show.
- The playlist menu shows "Load into Show Cue" or "Replace Show Cue" from `show_cue_exists` (`src/db/show_cue.rs:214`).
- The Show refresh starts in `src/app/show.rs:321`. The Show view model is in `src/view_models/show.rs`.

## Files To Inspect

- `src/app/show.rs`, `src/view_models/show.rs`, `src/ui/shells/show.rs`
- `src/application/commands/playlist.rs`, `src/db/show_cue.rs`
- `src/library/app_impl.rs`, `src/app.rs`
- An existing destructive confirmation, for example the playlist delete confirmation that the guard `adr_0084_music_open_defects_r30_06_playlist_delete_opens_the_confirmation` names
- `tests/architecture_tests.rs`

## Files Likely To Change

- `src/app/show.rs`, `src/view_models/show.rs`, `src/ui/shells/show.rs`, `src/library/app_impl.rs`, `src/app.rs`, `tests/architecture_tests.rs`

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Cue clear command | reuse | `src/application/commands/playlist.rs` | `ClearShowCue` exists from task 002. |
| Show view model | extend | `src/view_models/show.rs` | Gives the Clear action with its availability and labels. |
| Show page adapter | extend | `src/app/show.rs` | Sends the command and refreshes the mounted views. |
| Destructive confirmation | reuse | `src/ui/composites/confirmation_dialog.rs` | The playlist delete confirmation in `src/ui/shells/playlist_delete_confirmation.rs` uses it. |

## Scope

1. The Show view model gives a Clear action with a label, an accessibility label and a typed availability. The action is available only when a cue exists and playback is stopped.
2. The Show page shows the Clear item near the cue, in the destructive role. It follows musicindex ADR 0064: a destructive button is not rightmost. The item opens a confirmation that gives the first focus to Cancel, as the parent task records in its "Operator Choices".
3. The handler runs `ClearShowCue` through the command bus. It checks the stopped state again. A Clear while playback plays or is paused changes nothing and reports the reason.
4. After a Clear, the mounted Show page shows no cue, and the mounted playlist menu shows "Load into Show Cue". No page change is necessary to see the result.
5. With no cue, the Show keys are not available, as tasks 003c and 003d decide. Add a test for the state after a Clear.

## Do Not Touch

- The storage function `clear_show_cue` and the command `ClearShowCue`, unless a test finds a defect.
- The transport and the owner.
- `src/assets/tokens.json`.

## Constraints

- Use the existing destructive role and the existing confirmation. Add no token, no primitive and no composite.
- A view model carries no renderer type. A renderer does not decide if a control is available.
- Write each comment in Simplified Technical English. Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Do not run the app. Leave the change in the working tree.

## Mechanical Criteria

- View model tests: the Clear action is available only with a cue and a stopped session. It is not available while playing, while paused and with no cue.
- A handler test: `ClearShowCue` runs, and the cue is empty.
- A handler test: a Clear while playback plays or is paused changes nothing.
- A test at the adapter layer: after a Clear, the playlist menu label is "Load into Show Cue", and the Show view model state is empty.
- The test commands pass.

## Visual Criteria

None in this file. V5 of the parent task covers this part.

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

- Stop if no existing confirmation fits the Clear item.
- Stop if the Clear item needs a change to the Show page layout that ADR 0063 owns.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/adr-0068-task-003e-show-clear-item.md`
- `AGENTS.md`, the sections "The Durable Set", "Working Rules" and "Conventions"
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`
- The files of "Files To Inspect"

Goal:
- The Show page has a Clear item that runs `ClearShowCue` while playback is stopped.

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
