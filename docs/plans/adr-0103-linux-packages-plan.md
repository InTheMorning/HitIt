# ADR 0103 Linux Packages Plan

## Goal

Make the `.deb`, the AppImage and the release workflow of [ADR 0103](../adr/0103-the-linux-packages-and-the-release-workflow.md). Close the two missing-program gaps of ADR 0103 Decision 6.

## Phases

| Phase | Task | Result | State |
| --- | --- | --- | --- |
| 1 | [Task 001](../tasks/adr-0103-task-001-deb-appimage-and-release-workflow.md) | `Cargo.toml` deb metadata, the AppImage script, the glibc script, `release.yml` and the runbook. A local `.deb` and AppImage. | Done, `5cc5655` |
| 2 | [Task 002](../tasks/adr-0103-task-002-missing-program-notices.md) | The playback notice names `mpv`. The remote host state names `ssh`. | Done |
| 3 | Operator | The first tag, the first workflow run and the visual checks of ADR 0103. | Open |

Phase 1 and phase 2 change different files. They can run in either sequence.

## Order Of Checks

1. Each task runs its mechanical checks and the gate of `AGENTS.md`.
2. The main session reads each diff and makes the commit.
3. The agent walks the AppImage on Xvfb. It cannot remove `mpv` from the host, so it uses a `PATH` without `mpv`.
4. The operator does the visual checks of ADR 0103 on Ubuntu 22.04 and on the newest Ubuntu.

## Open

- The first run of `release.yml` on GitHub proves the glibc floor of 2.35. A local build on Arch Linux cannot prove it, and this sandbox has no container service.
