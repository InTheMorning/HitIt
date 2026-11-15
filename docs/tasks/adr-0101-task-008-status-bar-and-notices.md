# ADR 0101 Task 008: The Status Bar And The Notices

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks.

## Goal

The window has one status bar at the bottom, the same on each section. It replaces the live status band. The result of an action shows where the action was. An event that is not the result of a visible action goes to the status bar. HitIt shows no toast.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 7 owns the status bar. Read Decision 7 fully before the first edit:
  - Left: the latest reported event, in the words of the report owner (ADRs 0059 and 0063), with its recorded time.
  - Right: small items with a simple icon and a short word. Each item opens its detail. The items are feed state, files to update, reports with a count, and show state.
  - The show item appears only while a show is active. With no active show, Music and Settings show no broadcast state ([ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Decision 18).
  - The reports item opens the normal-shell notice of [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md). Its subjects, repair actions and retained actions stay reachable there and in Settings.
- musicindex ADR 0026 R-NOTICE-1 (`../musicindex/docs/adr/0026-cross-app-controls-and-feedback.md` lines 81 to 109, at musicindex `785ae28`):
  1. The result of an action shows where the action was. A copy button shows "Copied". The bar of an editor shows "Saved, not written".
  2. An event that is not the result of a visible action goes to one status area of the app, the status bar of HitIt.
  3. No app shows a toast: a message that slides in and goes away by itself.
- musicindex ADR 0028 R-PAGE-2 item 4 (`../musicindex/docs/adr/0028-cross-app-item-pages.md` lines 57 to 73): after a copy, the "…" button of the header shows "Copied".
- [ADR 0059](../adr/0059-broadcast-control-surface.md) and ADR 0063 own the words of a report.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `build_live_status_strip` (`src/app/show.rs:293`) builds the band only on Music and Settings. Examined.
- `TopApp` render (`src/app.rs:2181` to `:2222`) puts the capability notice below the toolbar, then the live status band, then `download_status_view`, then the section content. Examined.
- `LiveStatusStrip::render` (`src/ui/composites/live_status_strip.rs:65`) owns the band. `LiveStatusDisplay::from_projection` (`src/view_models/live_status.rs:175`) makes it active with playback, a broadcast or a recording. Examined. It also gives "No active show" and the action "Open Show".
- `capability_report` and `capability_notice` (`src/ui/composites/startup_report.rs:27` and `:135`) own the Settings report and the compact normal-shell notice. Its max height is 160 px (`src/ui/layouts.rs:37`).
- Other notice code: `src/ui/shells/publisher.rs:175`, `src/ui/shells/name_level_page.rs:118` and `src/ui/shells/search_results_inspector.rs:668`.
- `src/view_models/show/event_report.rs:92` gives the time of an event report.
- This is an inference: no toast exists in HitIt.
- Tests: `live_status_no_show_is_absent` (`src/view_models/live_status.rs:245`), `adr_0066_normal_notice_preserves_workspace_height` (`src/ui/composites/startup_report.rs:346`) and `adr_0066_optional_notices_have_one_normal_report_owner` (`src/app/bootstrap.rs:642`). Guard: `adr_0060_live_status_strip_contract_and_mount_are_guarded` (`tests/architecture_tests.rs:14306`).

Main session choice, 2026-11-15: the download status item stays where ADR 0098 puts it. The operator decides at the walk if it moves into the status bar.

Main session choice, 2026-11-15: in the narrow shell, the status bar is above the tab bar.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Live status projection | extend | `src/view_models/live_status.rs` | It gives the show state. The status bar view model uses it for the show item. |
| Status bar view model | new | `src/view_models/status_bar.rs` | Searched `src/view_models/` in HitIt. `live_status.rs` gives only the show. No view model gives the latest event and the right items. |
| Status bar composite | extend | `src/ui/composites/live_status_strip.rs` | It is the one band. Change it into the status bar, and rename the file and the type. |
| Normal-shell notice | reuse | `src/ui/composites/startup_report.rs` | `capability_notice` is the notice that the reports item opens. |
| Shell layout | extend | `src/app.rs` | It mounts the band. Mount the status bar at the bottom there. |
| Mount guard | extend | `tests/architecture_tests.rs` | The ADR 0060 guard covers the band. Change it to the status bar. |

## Required Changes

1. **The view model.** `status_bar.rs` gives the latest event with its time, and the right items: feed state, files to update, reports with a count, and show state. The show item exists only while a show is active. Each item gives its icon, its word, its accessibility label and the detail that it opens. Find the source of each item, and report it.
2. **The composite.** The status bar draws the view model in one row at the bottom of the window. The event text truncates. The right items keep their place.
3. **The shell.** The status bar shows on each section. The live status band and its mount go away. The capability notice no longer shows below the toolbar. The reports item opens it.
4. **The copy result.** After a copy, the "…" button of a page header shows "Copied". Find each copy action, and give it this result. A copy in a row menu shows "Copied" in the row.
5. **The other notices.** Examine the notices of `publisher.rs:175`, `name_level_page.rs:118` and `search_results_inspector.rs:668`. A notice that is the result of a visible action stays in its place. A notice of an event moves to the status bar. Report each one.
6. **The guards.** Change the ADR 0060 guard so that one composite owns the status bar and no screen renders a second band. Its message names ADR 0101 Decision 7 and the fix. Delete each guard and test of the band that no longer applies.

## Do Not Change

- No change of the words of a report. ADRs 0059 and 0063 own them.
- No change of the Settings report.
- No change of the download status item, other than its place in the code.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test proves that the status bar view model gives no show item when no show is active, and gives one when a show is active.
- M2: a test proves that the latest event gives its words and its recorded time.
- M3: a test proves that the reports item gives the count and opens the normal-shell notice.
- M4: the changed guard proves one composite owner and no second status band.
- M5: a test proves that a copy action gives "Copied" on its button.
- M6: a guard proves that no code builds a notice that closes itself on a timer. Its message names R-NOTICE-1 item 3.
- M7: the fit tests of ADR 0097 pass, with the status bar at 390 px and at 940 px.
- M8: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M, at normal and narrow widths:

- V1: the status bar shows at the bottom of Music, Show and Settings. It reads clearly.
- V2: with no active show, the bar shows no show item. Start a show. The show item appears.
- V3: the reports item opens the notice with its repair actions.
- V4: copy the feed URL on a release page. The "…" button shows "Copied".

## Test Commands

- `cargo test --lib view_models::status_bar`
- `cargo test --lib view_models::live_status`
- `cargo test --lib ui::composites::startup_report`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M6 by name. Give the source of each item of step 1. Give each copy action of step 4 and each notice of step 5 with its result.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if an item of the right side has no source in the code.
- Stop if the Show page needs the band for a control that the status bar does not give.
- Stop if a repair action of the notice is not reachable after the change.
