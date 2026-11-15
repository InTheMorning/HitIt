# Orchestrated Run Of 2026-11-14: Show Plays The Cue

The run implements [ADR 0068 task 003](../tasks/adr-0068-task-003-show-plays-the-cue.md) in five parts. The operator gave the instruction "orchestrate" on 2026-11-14. This record is advisory. ADR 0068 owns each rule.

## Decisions Of The Planning Model

| Number | Decision | Cause |
|---|---|---|
| D1 | The run holds task 003 only. | `AGENTS.md` permits one phase for each session. |
| D2 | Task 003 is divided into parts 003a to 003e. | The task has 11 scope items, and a coding model must not decide architecture. |
| D3 | The removal of the playlist Play is the first part. | ADR 0068 permits no legacy GUI route, and the old owner path moves through playlist rows. |
| D4 | A Clear or a Replace sets the session cue entry to NULL. | Open point of task 002. No schema change is necessary. |
| D5 | Select while playing or paused is not available. | ADR 0068 gives a rule only for Select while stopped. |
| D6 | The Clear item opens a confirmation. | The playlist delete has one, and a Clear removes stored data. |

The section "Operator Choices" of the task file gives the full text of D4 to D6.

## Sources

A ChatGPT `ask` (`gpt-6.1-sol`, effort medium) gave the code locations at commit `3b387f7`. The planning model examined the callers of `play_playlist_at` and the references to `cue_entry_id` with a search of its own. The other locations come only from that report.

## Result

| Part | Worker | Result |
|---|---|---|
| 003a | | |
| 003b | | |
| 003c | | |
| 003d | | |
| 003e | | |

## Open Visual Checks

V1 to V5 of the task file. The walk is at the end of the run.
