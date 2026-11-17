# ADR 0093 Task 001: Settings Tab Row And Rows

## Status

Implemented - 2026-11-16. The operator visual checks are open.

This task implements Decisions 1 to 8 of [ADR 0093](../adr/0093-settings-has-a-tab-row-and-one-save-bar.md). It does not need the draft of [ADR 0069 task 002](adr-0069-task-002-shared-guarded-editor.md). Task 002 adds the Save bar, the marks and the leave question.

## Goal

Replace the group menu of Settings with a segmented control below the page bar. Show each setting of General and Library as one row. Put the name and the hint at the start and the control at the end.

## Files To Inspect

- `src/view_models/settings.rs`, `src/app/settings.rs`, `src/ui/composites/settings.rs`
- `src/ui/composites/segmented_control.rs`, `src/ui/composites/list_page_head.rs` (the scope control uses the same control)
- `src/ui/composites/maintenance_page.rs` (`PageMenu`, which the Diagnostics page menu keeps)
- `tests/architecture_tests.rs`: `adr_0069_settings_group_ownership`
- `src/ui/composites/settings.rs` tests: `adr_0074_short_windows_reserve_space_for_task_content`, `adr_0069_settings_pages_retain_independent_scroll_offsets`

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Segmented control | reuse | src/ui/composites/segmented_control.rs | The scope control of `list_page_head.rs` uses it. The tab row uses it with no change to its look. |
| Settings view model | extend | src/view_models/settings.rs | It gives the group control, the sections and the rows. |
| Settings screen | extend | src/ui/composites/settings.rs | It draws the tab row and the rows from the view model. |
| Settings app glue | extend | src/app/settings.rs | It makes the navigation. The group `PageMenu` goes, and the Diagnostics `PageMenu` stays. |
| Diagnostics page menu | reuse | src/ui/composites/maintenance_page.rs | `PageMenu` stays for the Diagnostics tool pages. |
| Settings row | new | src/ui/composites/settings_row.rs | Searched src/ui/composites (action_row.rs, list_row.rs, track_row.rs) and ~/build/musicindex-lib (no GPUI code). No composite puts a name and a hint at the start and a control at the end, with a narrow fallback. |
| Segmented control | extend | src/ui/composites/segmented_control.rs | `fill_width` makes the control fill its parent on a narrow page (ADR 0093 Decision 3). |

## Scope

1. `SettingsVm` gives the group control as segments: the key, the label, the accessible label and the selected group. The Settings screen renders it with `SegmentedControl`. The group `PageMenu` and `SettingsVm::GROUP_MENU` are removed.
2. The Diagnostics page menu stays. It is below the tab row on the Diagnostics group.
3. `SettingsVm` gives the sections and rows of ADR 0093 Decision 7. Each row has its name, its hint of one line and its control type. The renderer chooses no row and no text.
4. A shared row composite puts the name and the hint at the start and the control at the end. It puts the control below the name when the row is too narrow, and always for the endpoint text field.
5. The "Use Defaults" button moves to the row "Default values" of General. In this task it keeps its current behavior. ADR 0069 task 002 changes it to a draft edit. It keeps the red tint and the confirmation until that change. After that change it keeps the red tint and the first position, and it has no confirmation (ADR 0093 status, musicindex ADR 0020 R-ACT-3 item 4).
6. Save stays where it is in this task. ADR 0069 task 002 moves it to the page bar.

## Do Not Touch

- The save route, `TopApp::save_settings`, the configuration editor and the correction owners. ADR 0069 task 002 owns them.
- The Diagnostics tool pages and their composition (ADR 0074).
- The tokens, the segmented control look and the type dot.

A delegate does not start the app. The primary agent does the walk on Xvfb.

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

## Stop And Ask

- Stop if a row of Decision 7 has no current control or value in the app.
- Stop if the narrow fallback of the row needs a width value that no token gives.
- Stop if a guard of ADR 0069 or ADR 0074 other than `adr_0069_settings_group_ownership` must change.

## Report

Give the files changed and the tests of the mechanical criteria by name. Give each decision that you made and this task does not give.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Implementation Report - 2026-11-16

Result: stopped. One mechanical criterion is not met. The task needs a decision (see Open Stop And Ask).

### Files changed

- `src/view_models/settings.rs`: group control, sections, row control types, `defaults_action`.
- `src/app/settings.rs`: tab row, section rows, `settings_content_row`.
- `src/ui/composites/settings.rs`: `settings_tab_row`, `settings_section_heading`, `settings_value_and_action`. The frame has the tab row below the page bar. `settings_field` is removed.
- `src/ui/composites/settings_row.rs` (new) and its entry in `src/ui/composites/mod.rs`.
- `tests/architecture_tests.rs`: `adr_0069_settings_group_ownership` only.

### Open Stop And Ask

`adr_0074_short_windows_reserve_space_for_task_content` fails at XL, width 438, height 382. The content area is 20.5 px high and the minimum is 110 px. With the Diagnostics menu in the page bar and only the tab row added, the content area is 67 px high. The tab row and the menu need about 100 px at XL. The packet forbids a change to this guard. The fit guard `adr_0097_the_shell_fits_at_the_two_widths` passes.

### Tests of the mechanical criteria

- One group control, 3 segments, at most 5: `adr_0093_group_control_has_stable_segments_and_accessible_labels`.
- Group change gives `Navigate`: the same test, and `adr_0069_navigation_and_reentry_emit_no_edit_effect`. The scroll handles are the unchanged `adr_0069_settings_pages_retain_independent_scroll_offsets`.
- Sections and rows of Decision 7: `adr_0093_sections_give_the_rows_of_decision_7`.
- Guard with the Decision 1 message: `adr_0069_settings_group_ownership`.
- Short window: `adr_0074_short_windows_reserve_space_for_task_content` FAILS.

### Decisions made

- Hints are the existing texts. They are one paragraph, but a long hint wraps in the narrow name block.
- New wording: the hint of "Default values" is "Resets the General and Library settings to the defaults and saves them." It comes from the confirmation text. The empty value of the converter row shows "Not set". The section headings are from the Decision 7 table.
- The row name is "Audio converter", as in Decision 7. The word "(optional)" is removed.
- The narrow fallback uses `flex_wrap` and the `ColumnShort` token as the minimum width of the name block. No new width value.
- The tab row keeps the natural width of the segmented control. It does not fill the width at narrow width. A change needs a change to the segmented control. The visual criterion stays open.
- `edit_actions` now gives Save only. `defaults_action` gives Use Defaults for its row. The test for the sequence of Use Defaults and Save is deleted. The two buttons are no longer in one row. `destructive_button_space` is removed from `settings_actions` for the same reason.
- The code in `render_settings` keeps the strings that `settings_form_inputs_fill_scaled_frame_width` requires, so that guard is unchanged.

### Checks

- `cargo fmt -- --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test --no-fail-fast --tests`: 3235 library tests pass and 1 fails (the short window test). The 416 tests of `architecture_tests` pass. `app::fit_check::` passes.
- `cargo build --bin hitit`: pass.
- `git diff --check`: pass.
- Not done: the app, Xvfb and all visual criteria.

## Stop Report - 2026-11-16

The primary agent read the delegate report and the test. The change stays in the working tree with no commit.

- `adr_0074_short_windows_reserve_space_for_task_content` fails at XL in a window of 438 px by 382 px. The task area is 20.5 px high, and the test needs 110 px. At M, the test passes.
- The tab row adds approximately 50 px at XL. With the Diagnostics page menu on the title line, the task area is 67 px. Thus each new line below the page bar makes the test fail.
- The packet does not include a change of this test. ADR 0074 owns its minimum, and ADR 0093 Decision 1 owns the tab row. The two decisions do not agree in this window.

Options for the operator:

- A (recommended): keep 110 px at M. At XL in a narrow window, the minimum is 60 px. The delegate measured 67 px with this layout. The Diagnostics page menu goes back to the title line. This changes ADR 0074 only for the narrow XL window.
- B: in a short window, the Settings title line holds the tab row, and the title does not show. The test requires the title today, so this also changes ADR 0074.
- C: keep 110 px. The tab row goes into a menu in a short window. This is against ADR 0093 Decision 3.

Operator choice, 2026-11-16, relayed by the orchestrator session: option A. Keep 110 px at M. At XL in a narrow, short window only, the minimum is 60 px, and the Diagnostics page menu moves back to the title line. The ADR 0074 test changes to match, and its failure message names this choice. The tab row fills the width at a narrow width, and the "Default values" hint uses the new text.

## Second Stop Report - 2026-11-16

The primary agent made option A. The change stays in the working tree with no commit.

- `SegmentedControl::fill_width` (`src/ui/composites/segmented_control.rs`) makes the control fill its parent, and the segments share the width. Below the stacked width of ADR 0074 (760 px, unscaled), the tab row fills the width. The test checks the width at 438 px.
- At a narrow width, the Diagnostics page menu is on the title line. At a wide width, the menu is at the end of the tab row line. On the title line, the menu makes that line as high as a button. Then the wide XL content area is 157 px, not 170.5 px.
- At XL in a narrow window, the action band (75 px) is higher than the content area (67 px). The 60 px minimum permits this, so the test compares the band with the content area at M only. The band stays above the content area.
- The "Default values" hint has the new text.

The content heights at a window height of 382 px, with this change:

| Page | Scale | Width 438 | Width 1407 |
|---|---|---|---|
| Settings, Diagnostics | M | 126.5 px (110) | 210.5 px (200) |
| Settings, Diagnostics | XL | 67 px (60) | 170.5 px (200) - fails |
| Recovery | M | 165.5 px (110) | 254 px (200) |
| Recovery | XL | 132 px (60) | 222 px (200) |

The first stop report did not show the wide XL window, because the test stops when one assertion fails. Option A covers only the narrow window. ADR 0093 Decision 1 puts the tab row below the page bar, so the title and the tab row cannot share one line.

Options for the operator:

- A2 (recommended): at XL in a short window, the minimum is 160 px at a wide width. The narrow minimum stays 60 px. M is not changed. The measured height is 170.5 px.
- B2: at a wide width, the title, the tab row and the page menu share one line. This is against ADR 0093 Decision 1.
- C2: keep 200 px at wide XL. Settings hides the title in a short window at XL. The test requires the title today, so this changes ADR 0074.

Orchestrator choice, 2026-11-16: A2. At XL in a short window, the wide minimum is 160 px. The narrow minimum stays 60 px, and M does not change. A2 applies the operator choice A to the wide window. Choice A gives a lower minimum only at XL in a short window, and keeps the tab row below the page bar. Thus the orchestrator session decides it, not the operator.

At a wide width, the page menu stays at the end of the tab row line. The test compares the action band with the content area at M only.

## Walk Result - 2026-11-16

The agent walked Settings on an Xvfb display with the debug binary, in Light at M.

- At 1200 px, the tab row is below the title with its natural width. On Diagnostics, the page menu is at the end of the tab row line.
- At 438 px, the tab row fills the width, and each label shows in full. The page menu is on the title line.
- The "Default values" hint shows the new text.

The operator visual checks stay open.
