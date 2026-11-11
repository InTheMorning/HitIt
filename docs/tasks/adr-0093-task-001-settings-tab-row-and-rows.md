# ADR 0093 Task 001: Settings Tab Row And Rows

## Status

Ready - 2026-11-10. Implementation has not started. The operator decides the priority.

This task implements Decisions 1 to 8 of [ADR 0093](../adr/0093-settings-has-a-tab-row-and-one-save-bar.md). It does not need the draft of [ADR 0069 task 002](adr-0069-task-002-shared-guarded-editor.md). Task 002 adds the Save bar, the marks and the leave question.

## Goal

Replace the group menu of Settings with a segmented control below the page bar. Show each setting of General and Library as one row. Put the name and the hint at the start and the control at the end.

## Files To Inspect

- `src/view_models/settings.rs`, `src/app/settings.rs`, `src/ui/composites/settings.rs`
- `src/ui/composites/segmented_control.rs`, `src/ui/composites/list_page_head.rs` (the scope control uses the same control)
- `src/ui/composites/maintenance_page.rs` (`PageMenu`, which the Diagnostics page menu keeps)
- `tests/architecture_tests.rs`: `adr_0069_settings_group_ownership`
- `src/ui/composites/settings.rs` tests: `adr_0074_short_windows_reserve_space_for_task_content`, `adr_0069_settings_pages_retain_independent_scroll_offsets`

## Scope

1. `SettingsVm` gives the group control as segments: the key, the label, the accessible label and the selected group. The Settings screen renders it with `SegmentedControl`. The group `PageMenu` and `SettingsVm::GROUP_MENU` are removed.
2. The Diagnostics page menu stays. It is below the tab row on the Diagnostics group.
3. `SettingsVm` gives the sections and rows of ADR 0093 Decision 7. Each row has its name, its hint of one line and its control type. The renderer chooses no row and no text.
4. A shared row composite puts the name and the hint at the start and the control at the end. It puts the control below the name when the row is too narrow, and always for the endpoint text field.
5. The "Use Defaults" button moves to the row "Default values" of General. In this task it keeps its current behavior. ADR 0069 task 002 changes it to a draft edit.
6. Save stays where it is in this task. ADR 0069 task 002 moves it to the page bar.

## Do Not Touch

- The save route, `TopApp::save_settings`, the configuration editor and the correction owners. ADR 0069 task 002 owns them.
- The Diagnostics tool pages and their composition (ADR 0074).
- The tokens, the segmented control look and the type dot.

Do not run the app as an agent. The Xvfb screenshot script of the operator is permitted.

## Mechanical Criteria

Each criterion is checked by a unit test or a guard.

- The view model gives one group control with General, Library and Diagnostics in that sequence, and at most 5 segments.
- A group change through the control gives `SettingsEffect::Navigate` and keeps the scroll position of each group.
- The view model gives the sections and rows of ADR 0093 Decision 7 for General and Library, and no row for Diagnostics.
- The guard `adr_0069_settings_group_ownership` requires the segmented control and forbids the group `PageMenu`. Its failure message names ADR 0093 Decision 1.
- `adr_0074_short_windows_reserve_space_for_task_content` passes with the tab row at short height and XL.

## Visual Criteria

These criteria need a person or the agent Xvfb screenshot check. They are open until a person examines them.

- The tab row is directly below the page bar, in the light and the dark theme.
- At the narrow width, the tab row fills the width, and each label shows in full.
- Each row of General and Library has its name and hint at the left and its control at the right. The endpoint field is below its name.
- At XL and at short height, the rows and the Diagnostics page menu stay readable and reachable with Tab.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```
