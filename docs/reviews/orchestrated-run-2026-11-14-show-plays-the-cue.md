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
| 003a | ChatGPT `gpt-6-luna`, effort medium, two runs | The first run stopped at the playlist shell file, and the planning model added the file to the scope. The planning model read the diff, removed the unused Library playback availability and ran the gate: 3085 tests and 410 guards passed. Clippy, the format check and the build are Green. |
| 003b | ChatGPT `gpt-6-luna`, effort medium | The planning model read the diff, added one test and ran the gate: 3090 tests and 410 guards passed. Clippy, the format check and the build are Green. |
| 003c | ChatGPT `gpt-6-luna`, effort high. The run ended with a broker refusal and no report. | The planning model read the diff, made five corrections and ran the gate: 3098 tests and 410 guards passed. Clippy, the format check and the build are Green. The task file lists the corrections and the deviations. |
| 003d | ChatGPT `gpt-6-luna`, effort medium, then effort high. The first run stopped at the driver trait and changed no code. | The planning model read the diff, added four tests and ran the gate: 3104 tests and 411 guards passed. Clippy, the format check and the build are Green. |
| 003e | ChatGPT `gpt-6-luna`, effort medium | The planning model read the diff, removed a database read on the render thread, added one test and ran the gate: 3107 tests and 411 guards passed. Clippy, the format check and the build are Green. |

## Open Visual Checks

V1 to V5 of the task file are in section 20 of [pending human checks](../pending-human-checks.md). The walk is at the end of the run.
