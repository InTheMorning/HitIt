# ADR 0103 Task 002: The Notices For A Missing mpv And A Missing ssh

## Goal

The app tells the user when `mpv` or `ssh` is missing (ADR 0103 Decision 6, musicindex ADR 0084 R-PACKAGE-4 item 2).

## Authority

- [ADR 0103](../adr/0103-the-linux-packages-and-the-release-workflow.md) Decision 6.

## Recorded Facts

- `MpvDriver::new` does not start `mpv`. The first play starts it (`src/playback_driver/mpv.rs:83-84`). The error text is "spawn mpv at PATH".
- A failed play records a `Dependency::Playback` failure (`src/app/playback_bar.rs:254`). The report text is "App cannot use built-in playback." (`src/view_models/startup/capabilities.rs:469`). It does not name `mpv`.
- The converter probe reports a missing program with `ProbeOutcome::Missing` (`src/view_models/startup/converter.rs:37`).
- Remote control runs `ssh` (`src/broadcast/transport.rs:79`). The code has no separate state for a missing `ssh`. This is an inference.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Capability report | extend | `src/view_models/startup/capabilities.rs` | It owns the text of each dependency failure. Add the missing-program case there. |
| Playback driver error | extend | `src/playback_driver/mpv.rs` | Give a typed error for `ErrorKind::NotFound` at the spawn. |
| Remote transport | extend | `src/broadcast/transport.rs` | Give a typed state for a missing `ssh`. |
| Program probe | reuse | `src/audio_format/probe.rs` | The converter probe already finds a missing program. Use its method if it fits. |

## Required Changes

1. When the spawn of `mpv` gives `NotFound`, the playback report says that `mpv` is not installed, and names the path that the app tried.
2. When the spawn of `ssh` gives `NotFound`, the Show state says that `ssh` is not installed, not that the host cannot be reached.

## Acceptance Criteria

Mechanical:

1. A view model test: a `NotFound` spawn error of `mpv` gives a report text that contains `mpv` and "not installed".
2. A test: a `NotFound` spawn error of `ssh` gives the missing-program state, not `NotReachable`.
3. The gate of `AGENTS.md` passes.

Visual:

1. On Xvfb with a `PATH` without `mpv`, a play shows the notice that names `mpv`.

## Stop And Ask

- The change needs a new capability dependency or a new notice surface.

The delegate does not commit. Write prose in STE (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).
