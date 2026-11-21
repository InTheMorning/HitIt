# ADR 0101 Task 007: The Loading States

Status: Implemented - operator visual checks open, 2026-11-16.

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

Each fact is from HitIt `ba43138`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `Skeleton::render` (`src/ui/primitives/skeleton.rs:78`) draws static blocks. The module comment says "We intentionally avoid animated shimmer" (`:8` to `:11`). Examined.
- `SkeletonTrackRow` (`src/ui/composites/skeleton_track_row.rs:50`) and `SkeletonInspector` (`src/ui/composites/skeleton_inspector.rs:57`) give the layout of a row and of the inspector.
- The skeleton rows show in `src/ui/shells/library/content_list.rs:127`, `src/ui/shells/playlist.rs:646` and `src/app.rs:2082`. `pending_skeleton_count` (`src/view_models/pagination.rs:22`) gives their count.
- `maybe_turning` (`src/ui/icons.rs:383`) turns an icon in 800 ms with `with_animation`. It does not read `Environment::allows_motion`. Examined.
- `IconName::InProgress` maps to `LoaderCircle` (`src/ui/icons.rs:166`).
- Busy labels exist in the startup view models, for example "Checking…" (`src/view_models/startup.rs:267`). Examined.
- This is an inference: no primitive draws a progress bar.

- 2026-11-16, examined by the main session at HitIt `5969e6c`: task 005 did step 2. `icon_turns` (`src/ui/icons.rs:381`) gives `turning && environment.allows_motion()`, and `maybe_turning` turns only when `icon_turns` gives `true`. The test at `src/ui/icons.rs:738` proves M2. Do not change this code.

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

## Implementation Report - 2026-11-16

Step 2 (the spinner) was done by task 005. This run did not change `icon_turns`. M2 is met by `reduce_motion_stops_a_turning_icon` in `src/ui/icons.rs`.

Files changed:

- `src/ui/primitives/skeleton.rs`: the sheen, the new module comment, tests.
- `src/ui/primitives/progress_bar.rs` (new), `src/ui/primitives/mod.rs`: the progress bar primitive and its `mod` and `use` lines.
- `src/view_models/progress_count.rs` (new), `src/view_models/mod.rs`: the view model of the count text and the fraction.
- `src/view_models/download_progress.rs`, `src/ui/composites/download_status_item.rs`: the bar in the download status item of a running batch.
- `src/ui/primitives/button.rs`: `Button::busy`.
- `src/view_models/entity_detail.rs`, `src/ui/shells/library/track_detail_metadata.rs`: the busy state of "Apply tags".
- `src/view_models/startup/session.rs`, `src/ui/composites/maintenance_forms.rs`: the busy label of "Retry drain".
- `src/ui/composites/startup_report.rs`: the busy state of the startup and capability buttons.
- `src/view_models/startup/capabilities.rs`: one added assertion in an existing test.

Tests:

- M1: `ui::primitives::skeleton::tests::sheen_runs_only_when_motion_is_allowed`, with `sheen_band_crosses_the_block_from_edge_to_edge`.
- M3: see the actions below.
- M4: `view_models::progress_count::tests::progress_count_gives_the_count_text_and_the_fraction`, `progress_count_keeps_the_fraction_between_zero_and_one`, and `view_models::download_progress::tests::adr_0101_running_batch_gives_the_progress_bar_its_count`.
- M5: `ui::primitives::progress_bar::tests::adr_0097_the_progress_bar_fits_at_the_narrowest_width` records the fit probe (through `Label`) at 390 px.
- Button: `ui::primitives::button::tests::a_busy_button_shows_the_spinner_and_keeps_its_label`.

Actions of step 3. `Button::busy(true)` disables the button and shows the turning spinner in place of the leading icon. The label comes from the view model.

| Action | Busy label | Test (M3) |
|---|---|---|
| Check again (startup recovery) | "Checking…" | `view_models::startup` test at the line `assert_eq!(button.label, "Checking…")` (existing) |
| Open app (startup recovery) | "Opening…" | No new test. The label was already in `StartupReportVm::action`. |
| Check again of a background tool | "Checking…" | `view_models::startup::capabilities` test, assertion added |
| Retry drain (session maintenance) | "Retrying drain…" (new) | `view_models::startup::session` test, assertion added |
| Apply tags (track metadata) | "Applying tags…" (was "Applying tags...") | `view_models::entity_detail::tests::track_metadata_action_state_projects_loading_and_staged_id3_display` (`apply_busy` field) |

Not changed:

- "Add to Playlist": no busy state exists. The popover adds in one step with no running state, so V2 of this packet cannot be met as written.
- The track and release primary actions "Downloading..." and "Removing..." (`EntityActionVm`): the production renderers use only their enabled state and kind for menu availability. No button shows them. Their labels keep three dots.
- The public playlist copy row: it already shows `COPYING_TEXT` ("Copying to My Playlists…") as row text. It is a row text, not a button.
- Other `StartupAvailability::Working` uses (Settings scale choices, correction actions) mark a sibling button that is disabled while another action runs. They carry no busy label of their own.
- A partly downloaded release: no release row shows a download count in the code today. The progress bar is on the download status item only.

Main agent choice, 2026-11-16: the count of the bar is the jobs that ended, with or without an error, of all jobs, for example "3 of 9 done". The title of the item gives the position of the current job, for example "Downloading 4 of 9". The word "done" shows the difference.

Sheen values:

- One band that crosses each block from the left edge, fully outside, to the right edge, fully outside.
- Band width: 40 % of the block width.
- Period: 1.6 s, repeated, linear easing.
- Color: the `Label` token with alpha 0.08 at the center. Light on Dark, dark on Light.
- Two child gradients of 50 % of the band each, at 90 degrees. The left one goes from alpha 0 to 0.08. The right one goes from 0.08 to alpha 0.

How the sheen asks for frames: `with_animation` on the band element calls the frame request of GPUI while the element is in the tree. Only a `Skeleton` block adds the band. A page with no skeleton adds no band and no frame request. With Reduce Motion (`sheen_runs` gives `false`), the block has no band, so it makes no frame request.

### Corrections of 2026-11-16 (walk of V3)

1. The track of the bar did not show. Cause: the track used `TertiaryFill`. On Dark that color is #1C1C1F, the same as the background of the download status item. The width and the height of the track were correct. The track now uses `SystemFill`. A new test, `ui::composites::download_status_item::tests::the_progress_bar_of_a_running_batch_has_a_track_and_a_fill`, proves that the track is at least `TRACK_MIN_WIDTH` wide and `TRACK_HEIGHT` high, and that the fill is the fraction of the track. The test cannot check the color. The operator must judge the color in V3.
2. The count text is "K of M done", for example "3 of 9 done". The fraction and the title did not change. The tests of M4 use this text.

## Primary Agent Check - 2026-11-16

- The primary agent read the diff and ran the gate: fmt, clippy, 3183 lib tests, 413 architecture tests and `cargo build --bin hitit` pass.
- Agent walk on the Xvfb display, debug binary, Dark, M:
  - V3, first walk: the item of a batch in progress showed "0 of 1", but no track. The corrections above are the result. The second walk could not start a batch that stays in progress, because wavlake.com gave HTTP 403 for each feed download. A finished batch shows no bar. V3 stays open.
  - V1: not walked. The scratch library has no playlist, and a public playlist loads all its rows in one step, so no skeleton row shows.
  - V2: cannot be met as written. "Add to Playlist" has no busy condition.
- Open point for the orchestrator: R-LOADING-1 item 1 asks for a skeleton of the layout when a page loads. The first load of a playlist page and of the Releases list shows a text, for example "Loading playlist". This packet does not change those pages.
