# ADR 0097 Task 004: Settings, Show And The Recovery Pages Fit At The Two Widths

Status: Ready - 2026-11-14. It waits for [task 002](adr-0097-task-002-list-pages-fit.md), which moves the harness. This is phase 4 of the [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md).

## Goal

A GPUI layout test draws the Settings groups, the maintenance pages and the Show page at 390 px and at 940 px. It also draws the startup report and the recovery pages. The fit probes find no fault of musicindex ADR 0065 R-FIT-3.

The fault finder ignores a text in a horizontal scroll area. The area shows only a part of its content.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) Decisions 2, 4, 5 and 6.
- musicindex ADR 0065 R-FIT-2 item 4 and R-FIT-3.
- The phase plan, Risks: the fault finder must ignore a horizontal scroll area before its page gets a check.

## Recorded Facts

These facts come from a ChatGPT code search of 2026-11-14. The implementer examines each fact before it uses the fact.

- The Settings groups are General, Library and Diagnostics. `settings_action(SettingsAction::SelectGroup(group), window, cx)` selects one (`src/app/settings.rs:42`, `src/view_models/settings.rs:12`).
- The Diagnostics pages are Database, Configuration, Background tools, App session and Cached files. `SettingsAction::SelectDiagnostic(DiagnosticPage::...)` selects one (`src/view_models/settings.rs:50`).
- `MaintenanceView::{Instructions, Report}` gives the two views of a maintenance page (`src/view_models/maintenance.rs:8`).
- `ConfigurationEditor::new` (`src/presentation/configuration_editor.rs:47`) and `DatabaseTools::new` (`src/presentation/database_tools.rs:47`) make the two editors. `DatabaseTask::{Check, Backup, Preserve, Restore, Repair}` gives the database tasks (`src/view_models/startup/database.rs:62`).
- The Show page opens with `SidebarDestination::Show`. `build_show_screen` makes it (`src/app/show.rs:54`).
- Bootstrap mounts `Root<StartupScreen>`, not `TopApp` (`src/app/bootstrap.rs:81`). `StartupScreen` draws `TopApp` when it has one. If it has no `TopApp`, it draws the startup report (`src/app/startup.rs:653` to `:679`).
- `StartupScreen::new(None, opened)` gives a blocked worker outcome (`src/app/startup.rs:56`). The recovery pages are Startup, Configuration and Database.
- A session that drains with no maintenance draws `session_drain` (`src/app/startup.rs:642`).
- `LogFrame` is the only horizontal scroll area in `src/ui`. It uses `.overflow_scroll()` on two axes (`src/ui/composites/log_frame.rs:217`, examined). The maintenance reports, the startup report and the Show log hold it.
- [ADR 0093 task 001](adr-0093-task-001-settings-tab-row-and-rows.md) replaces the Settings group menu with a tab row. If it lands first, this test draws the tab row.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Shell fit harness | reuse | src/app/fit_check.rs | Task 002 moves it to `src/app/fit_check/mod.rs`. Use its `init`, `top_app` and `draw_and_read`. |
| Fit probe and fault finder | extend | src/ui/primitives/fit_probe.rs | Add the horizontal scroll area, so that a probe in it gets no horizontal edge cut or overflow. |
| Log frame | extend | src/ui/composites/log_frame.rs | Mark its scroll area for the fault finder in a test binary. |
| Startup screen | reuse | src/app/startup.rs | `StartupScreen::new` gives the startup report and the recovery pages. |
| Settings actions | reuse | src/app/settings.rs | `settings_action` selects each group and each page. |

## Required Changes

1. In `fit_probe`, add a test-only mark for a horizontal scroll area. A probe in that area gets no edge cut on the horizontal axis. It gets no overflow on the horizontal axis. It gets the other faults. Add a unit test for each of these two results.
2. Make `LogFrame` add the mark in a test binary. A production binary records nothing, as with the other probes (ADR 0097 Decision 4 item 1).
3. Put the phase 4 test in `src/app/fit_check/settings_and_recovery.rs`.
4. Draw each surface at each width of `WIDTHS`:
   - Each Settings group, and each Diagnostics page in each `MaintenanceView`.
   - The configuration editor and each database task.
   - The confirmation text of a database restore.
   - The Show page with a long show name and a long log line.
   - The startup report, each recovery page, and the session drain page. Mount `StartupScreen` for these.
5. Give long text to each field that the test can set. Examples are a path of 120 characters and a long error.
6. Give each fault message the surface and its view.
7. In the same change, set the phase 4 row of the phase plan to Done with the date. Remove the phase plan Risk about the horizontal scroll area.

## Do Not Change

- No change to a layout or a token. A fault that the test finds gets its own task.
- No change to the production behavior of `LogFrame`.

## Acceptance Criteria

Mechanical:

- A unit test puts a long text in a marked horizontal scroll area and requires no horizontal edge cut and no overflow.
- A unit test puts two buttons on top of each other in a marked area and requires an overlap.
- The test `adr_0097_settings_show_and_recovery_fit_at_the_two_widths` draws each surface of Required Changes item 4 at 390 px and at 940 px.
- Each draw records one or more probes.
- The test finds no fault, or the report lists each fault (Stop And Ask).
- The tests of the earlier phases pass with no change.
- The commit checks of AGENTS.md pass.

Visual: none. The fit test is the check.

## Test Commands

- `cargo test --lib ui::primitives::fit_probe`
- `cargo test --lib app::fit_check`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests added. Give the full text of each fault message. Name each surface text that does not go through `Label` or `Button`.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the test finds a fault. Do not change the layout. Report each message.
- Stop if a recovery page needs an operating worker or a file on disk. Report the page and what it needs.
- Stop if a horizontal scroll area other than `LogFrame` is on a surface.
