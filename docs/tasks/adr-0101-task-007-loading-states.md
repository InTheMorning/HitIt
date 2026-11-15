# ADR 0101 Task 007: The Loading States

Status: Waits - 2026-11-15. This packet starts after [task 005](adr-0101-task-005-system-accessibility-settings.md), because the sheen and the spinner stop with Reduce Motion.

## Goal

A page or a list that loads shows a skeleton of its layout with a sheen. A short action shows a spinner in its button or row, with a busy label. A task with a known length shows a progress bar and a count. With Reduce Motion, the sheen and the spinner stop, and the busy label stays.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decisions 6, 11 and 12 own this packet.
- musicindex ADR 0026 R-LOADING-1 (`../musicindex/docs/adr/0026-cross-app-controls-and-feedback.md` lines 81 to 109, at musicindex `785ae28`):
  1. A page or a list that loads shows a skeleton of its layout at once. The content replaces the skeleton in the same places.
  2. A short action with no known length shows a spinner in its button or row, with a busy label, for example "Adding…".
  3. A task with a known length, for example a download, shows a progress bar and a count, for example "3 of 9".
  4. With Reduce Motion, the skeleton sheen and the spinner stop (R-A11Y-1). The busy label stays.
- musicindex ADR 0068 R-A11Y-1 item 1: a turning icon then shows a static state with its word.
- ADR 0101 Decision 6: the download state `LoaderCircle` turns. A partly downloaded release shows a count, for example "3 of 9".
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `Skeleton::render` (`src/ui/primitives/skeleton.rs:78`) draws static blocks. The module comment says "We intentionally avoid animated shimmer" (`:8` to `:11`). Examined.
- `SkeletonTrackRow` (`src/ui/composites/skeleton_track_row.rs:50`) and `SkeletonInspector` (`src/ui/composites/skeleton_inspector.rs:57`) give the layout of a row and of the inspector.
- The skeleton rows show in `src/ui/shells/library/content_list.rs:127`, `src/ui/shells/playlist.rs:646` and `src/app.rs:2082`. `pending_skeleton_count` (`src/view_models/pagination.rs:22`) gives their count.
- `maybe_turning` (`src/ui/icons.rs:383`) turns an icon in 800 ms with `with_animation`. It does not read `Environment::allows_motion`. Examined.
- `IconName::InProgress` maps to `LoaderCircle` (`src/ui/icons.rs:166`).
- Busy labels exist in the startup view models, for example "Checking…" (`src/view_models/startup.rs:267`). Examined.
- This is an inference: no primitive draws a progress bar.

Main session choice, 2026-11-15: the sheen is a band of two-stop gradients that moves across the skeleton in 1.6 s, with `Environment::allows_motion`. The website value of the sheen is not known to this session. The packet reports its values.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Skeleton | extend | `src/ui/primitives/skeleton.rs` | It draws each placeholder block. Add the sheen there, and replace the module comment. |
| Turning icon | extend | `src/ui/icons.rs` | `maybe_turning` is the one spinner. Make it read `allows_motion`. |
| Button | extend | `src/ui/primitives/button.rs` | A button with a busy state shows the spinner and the busy label. Find if it has a busy state, and extend it. |
| Progress bar | new | `src/ui/primitives/progress_bar.rs` | Searched `src/ui/primitives/` and `src/ui/composites/` in HitIt. The download status item gives text only. musicindex-lib keeps no GPUI renderer (orchestrator answer of 2026-11-15). |
| Download status | extend | `src/ui/composites/download_status_item.rs` | It shows the download count. Add the bar there. |

## Required Changes

1. **The sheen.** `Skeleton` draws a sheen when `allows_motion` is true, and a static block when it is false. Each skeleton composite gets the sheen through the primitive.
2. **The spinner.** `maybe_turning` turns only when `allows_motion` is true. When it is false, the icon is static and its word stays.
3. **The busy button.** A button of an action with no known length shows the spinner and a busy label that ends with "…" while the action runs. Find each action that is disabled while it runs, for example "Add to Playlist", and give it a busy label. Report each one.
4. **The progress bar.** A new primitive draws a bar and a count text, for example "3 of 9". The count text carries the meaning. The download status item and a partly downloaded release use it.
5. **The fit probe.** The new primitive records a fit probe (ADR 0097 Decision 4).

## Do Not Change

- No change of `pending_skeleton_count` or of the count of skeleton rows.
- No change of the download state icons of ADR 0101 Decision 6, other than the stop with Reduce Motion.
- No toast. Task 008 owns the notices.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test proves that `Skeleton` asks for the sheen when `allows_motion` is true and does not when it is false.
- M2: a test proves that `maybe_turning` gives no animation when `allows_motion` is false.
- M3: a test of each view model of step 3 proves the busy label while the action runs.
- M4: a test of the progress bar view model gives the count text "3 of 9" and the fraction.
- M5: the fit tests of ADR 0097 pass, with the probe of the new primitive.
- M6: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M:

- V1: open a long playlist. The skeleton rows show a sheen. The rows replace them in the same places.
- V2: add a track to a playlist. The button shows a spinner and "Adding…" while the add runs.
- V3: download a release of nine tracks. The status shows a bar and "3 of 9".
- V4: with reduced animation on in the desktop settings, the sheen and the spinner stop. The words stay.

## Test Commands

- `cargo test --lib ui::primitives::skeleton`
- `cargo test --lib ui::primitives::progress_bar`
- `cargo test --lib ui::icons`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M4 by name. Give each action of step 3 with its busy label. Give the sheen values.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the sheen makes the UI thread draw each frame while no skeleton shows.
- Stop if a busy action has no view model state that tells that it runs.
