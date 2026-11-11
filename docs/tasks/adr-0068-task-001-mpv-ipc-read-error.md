# ADR 0068 Task 001: The mpv IPC Read Error

## Status

Ready - 2026-11-11. Implementation has not started.

## Goal

Find the cause of the error `read mpv IPC message: Resource temporarily unavailable (os error 11)`, fix it, and add a test that fails before the fix. The operator saw this error on 2026-09-11 in the producer fixture check of [ADR 0066 task 004](adr-0066-task-004-optional-tool-isolation.md#playback-workflow-correction--2026-09-11).

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Follow-Up Work": the error needs a diagnosis and a regression proof before the audible playback check.
- [ADR 0021](../adr/0021-mpv-playback-driver.md): the driver boundary and the IPC rules.
- [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md): a playback failure is a fact apart from the publication state.

## Recorded Facts - 2026-11-11

These facts come from a code read. Nothing in the repository confirms the cause.

- The text comes from `read_message` in `src/playback_driver/mpv.rs:283` (`.context("read mpv IPC message")`).
- `os error 11` is `EAGAIN`, `WouldBlock`. A socket read with a read timeout gives it when the time ends.
- The socket has a read and a write timeout of `IPC_TIMEOUT`, 2 s (`src/playback_driver/mpv.rs:18` and :92-95).
- `command()` reads messages until it finds its `request_id`. It does not retry after a timeout.
- `poll()` sends two commands on each tick. The actor in `src/runtime/playback_polling.rs` calls it. `PlaybackOwner::poll` changes `status.error` to a "playback driver error".
- `MpvDriver` holds one `Mutex<MpvState>` for the full round trip of each command.

## Files To Inspect

- `src/playback_driver/mpv.rs`, `src/playback_driver/mod.rs`
- `src/playback_owner.rs` (`poll`), `src/runtime/playback_polling.rs`
- `src/app/playback_bar.rs` (`apply_playback_tick`, `PlaybackTickOutcome::Error`)

## Scope

1. Reproduce the error in a test. Use a local Unix socket server that acts as mpv. Make it slow to answer, or make it send events between the replies. Do not start a real mpv in a test.
2. Find the cause. Examples to examine: a reply that comes after 2 s, a read that a partial line interrupts, and an event stream that delays the reply.
3. Fix the cause in the driver. A timeout that is not a failure of mpv must not become a playback error. A real failure of mpv stays an error with its text.
4. Record the cause and the fix in the Report.

## Do Not Touch

- The owner, the session and the producer, apart from the error mapping that the cause needs.
- The socket path. Packet 004 changes it.

## Mechanical Criteria

- A test with the local socket server fails before the fix and passes after it.
- A test proves that a closed socket or an exited child still gives a playback error.
- The commit checks of AGENTS.md pass.

## Visual Criteria

None. The [final gate](../plans/adr-0068-show-cue-and-audition-phase-plan.md#final-gate) listens to playback with mpv.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```

## Report

Give the cause, the evidence for it, the fix and the tests.

## Stop And Ask

Stop if no test reproduces the error. Report each cause that you examined and the result.
