# ADR 0108 Task 002: The Popup Has Check Boxes

## Status

Implemented - 2026-11-21. The visual criteria are open in section 48 of the pending human checks.

## Goal

The "Update n files" popup has a check box for each file and each change group. The host clears a change, and the confirm records it as a skip. A folded section "Skipped (n)" lists the skips. The host can select a skip and write it. When only skips remain, a different control opens the popup to the skips.

## Authority

[ADR 0108](../adr/0108-the-tag-update-selects-each-change.md) Decisions 1, 2 and 4. [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) applies the check box rules R-CHECK-1 to R-CHECK-8 of musicindex ADR 0063. ADR 0084 Decision 10 gives the Mixed condition of the check box. [Task 001](adr-0108-task-001-the-scan-keeps-skips.md) gives the terms "change group" and "skip", `with_groups` and `skips_for_groups`.

## Change

1. Selection model. Add a pure `TagUpdateSelection` in `src/view_models/tag_update.rs`. It is made from one `TagUpdateScan`.
   - It holds the selected groups of each file and the selected skips. At the start, each group that is not skipped is selected (Decision 2.1). Each skip is cleared (Decision 4.2).
   - It holds `skipped_folded: bool`. It is `true` at the start. It is `false` when the popup opens from the control of item 6.
   - `toggle_group(track_id, group)`, `toggle_file(track_id)` and `toggle_skip(track_id, group)` change the selection. `toggle_file` selects each group of the file when one or more groups are cleared. In the other condition, it clears each group.
   - `file_state(track_id) -> CheckboxState`: Checked, NotChecked or Mixed (Decision 1.2). It covers only the groups that are not skipped.
   - `confirm_enabled()` is `true` when the file has one or more groups that are not skipped, or when one or more skips are selected (Decision 2.2).
   - `confirm_plan() -> (Vec<TagUpdateFile>, Vec<TagUpdateSkip>)`. The files are `with_groups` of the selected groups and the selected skips. The skips are `skips_for_groups` of the cleared groups that are not skipped.
   - A file that is in use or unreadable has no groups in the selection and is not in the plan.
2. Popup view model. `popup(snapshot, selection)` gives a display with:
   - One item for each file that has one or more groups that are not skipped. The item has a check box with `file_state`. A file that is in use or unreadable has no check box (Decision 1.4).
   - One line for each group, with a check box. A group of two changes shows the text of each change in its line.
   - A section "Skipped (n)" when `skip_count()` is more than 0. It has one line for each skipped group, with the file title, the frame, the stored value and the file value, and a check box.
   - `confirm_enabled`.
   - The accessible label of each check box names the file and the frame. Example: "Write TIT2 of the golden arms".
3. Confirmation dialog composite (`src/ui/composites/confirmation_dialog.rs`). Extend it. Do not make a copy.
   - `ConfirmationDialogItem` gets an optional check box. Each line gets an optional check box.
   - `ConfirmationDialogDisplay` gets `confirm_enabled: bool` and an optional folded section. The folded section uses `DisclosureGroup` (`src/ui/composites/disclosure_group.rs`).
   - `ConfirmationDialogHandlers` gets one optional handler for a click on a check box. It gets the id of the box. It gets one optional handler for the fold toggle.
   - Add an open function that takes a function `Fn(&App) -> ConfirmationDialogDisplay`. The dialog builder of gpui-component runs at each render of `Root` (`gpui-component-0.6.1/src/root.rs`, line 259). Thus the dialog shows the current selection. After each change of the selection, call `window.refresh()`.
   - The existing callers do not change their behavior. Their display has no check boxes, `confirm_enabled` is `true`, and no section.
   - Use `Checkbox::new(id, state)`, `.a11y_label` and `.on_click` (`src/ui/primitives/checkbox.rs`). The click area of the box and its text is one target (R-CHECK rules of ADR 0101).
4. Shell (`src/ui/shells/tag_update_confirmation.rs`): map the popup display and the handlers to the composite. The shell has no domain logic.
5. Library app (`src/library/app_impl.rs`):
   - `open_tag_update_popup` makes the `TagUpdateSelection` from the scan of the snapshot, and keeps it in `LibraryApp` while the popup is open.
   - A click on a check box changes the selection in `LibraryApp`.
   - The confirm sends `confirm_plan()` to `TagUpdateHandle::confirm`. Cancel sends nothing and clears the selection (Decision 2.3).
6. The control for skips only (Decision 4.4). When `count()` is 0 and `skip_count()` is more than 0, `button()` gives a display with the label "Skipped tag changes (n)". The status bar shows it in the slot of "Update n files" (`src/view_models/status_bar.rs`). A click opens the popup with the section "Skipped (n)" unfolded. The popup title is "Skipped tag changes". The confirm is not available until the host selects a skip.
7. Change these documents in the same change:
   - The status line and the report of this task file.
   - `docs/pending-human-checks.md`: a new section 48 with the visual criteria below.

Main session choices, 2026-11-21:

- The control for skips only is the existing status bar slot with a different label. No new area on the screen.
- The popup title keeps the count of task 001. It does not change while the host clears changes.
- If a step needs a decision that this task file and ADR 0108 do not give, the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Check box | reuse | src/ui/primitives/checkbox.rs | It has Checked, Mixed and NotChecked. |
| Disclosure group | reuse | src/ui/composites/disclosure_group.rs | It gives the folded section with `.collapsed` and `.on_toggle`. |
| Confirmation dialog | extend | src/ui/composites/confirmation_dialog.rs | It gets check boxes, `confirm_enabled`, a folded section and a display function. |
| Tag update view model | extend | src/view_models/tag_update.rs | It gets `TagUpdateSelection`, the popup check boxes and the label for skips only. |
| Tag update shell | extend | src/ui/shells/tag_update_confirmation.rs | It maps the new display and handlers. |
| Status bar view model | extend | src/view_models/status_bar.rs | It shows the label for skips only. |
| Library app popup | extend | src/library/app_impl.rs | It keeps the selection and sends the plan. |

## Mechanical Criteria

1. A selection test: at the start, each group that is not skipped is selected, and each skip is cleared. `file_state` is Checked.
2. A selection test: clear one group of a file with two groups. `file_state` is Mixed. Clear the other group. `file_state` is NotChecked. `toggle_file` then selects the two groups.
3. A selection test: `confirm_plan` writes only the selected groups and gives one skip for each change of each cleared group.
4. A selection test: with all groups cleared, `confirm_enabled` is `true`, the plan has no file, and it has the skips.
5. A selection test: with only skips and none selected, `confirm_enabled` is `false`. Select one skip. The plan writes its group and records no skip.
6. A selection test: two changes of one stored key are one group and one line.
7. A view model test: a file that is in use or unreadable has no check box, and it is not in the plan.
8. A view model test: the section "Skipped (n)" has the count of skipped groups, and each line has the frame, the stored value and the file value.
9. A view model test: with `count()` 0 and `skip_count()` 2, `button()` gives "Skipped tag changes (2)". With `skip_count()` 0, it gives no button.
10. A composite test: an existing display with no check boxes maps as before. Change the existing presenter test `adr_0076_tag_update_presenter_maps_files_to_dialog_items` only for the new fields.
11. The commit checks of AGENTS.md pass.

## Visual Criteria

These go in section 48 of [pending human checks](../pending-human-checks.md).

- V1: the popup shows a check box at each file and at each change. All are checked at the start. Light and Dark.
- V2: clear one change of a file with two changes. The file box shows Mixed. The confirm writes the other change.
- V3: after the confirm of V2, the count falls. The popup shows "Skipped (1)", folded.
- V4: with only skips, the status bar shows "Skipped tag changes (n)". A click opens the popup with the section unfolded. The confirm is not available until a skip is selected.
- V5: select a skip and confirm. The file gets the stored value, and the skip goes.
- V6: Tab moves to each check box, and Space changes it.

## Report

Changed files:

- `src/view_models/tag_update.rs`: `TagUpdateSelection`, `TagUpdateCheck`, `TagUpdateCheckState`, the popup check boxes, the section "Skipped (n)", `skip_count` and the control for skips only in `button`.
- `src/ui/composites/confirmation_dialog.rs`: check boxes, `confirm_enabled`, the folded section, the two new handlers and `open_confirmation_dialog_from`.
- `src/ui/composites/mod.rs`, `src/app/show.rs` and four shells: the new fields of the display and the item, with the old values.
- `src/ui/shells/tag_update_confirmation.rs`: maps the display and the handlers.
- `src/library.rs` and `src/library/app_impl.rs`: the selection, the check handlers, and the confirm with the plan.
- `src/view_models/status_bar.rs`: one test. The files item gets the label from `button`.
- `src/application/queries/tag_update.rs`: `skips_for_groups` is no longer `#[cfg(test)]`.
- `docs/pending-human-checks.md`: section 48.

Mechanical criteria and their tests:

1. `adr_0108_selection_starts_with_each_open_group_selected_and_each_skip_cleared`
2. `adr_0108_selection_file_box_is_mixed_when_one_group_is_cleared`
3. `adr_0108_selection_plan_writes_selected_groups_and_skips_cleared_ones`
4. `adr_0108_selection_with_all_groups_cleared_confirms_only_the_skips`
5. `adr_0108_selection_with_only_skips_needs_a_selected_skip`
6. `adr_0108_selection_two_changes_of_one_group_are_one_line`
7. `adr_0108_popup_gives_no_check_box_to_a_file_in_use_or_unreadable`
8. `adr_0108_popup_skipped_section_counts_groups_and_shows_values`
9. `adr_0108_button_for_skips_only_shows_the_skip_count` and `adr_0108_files_item_shows_skipped_changes_when_no_file_remains`
10. `adr_0076_tag_update_presenter_maps_files_to_dialog_items` (changed only for the new fields) and `adr_0108_confirmation_section_shows_items_only_when_unfolded`
11. The commit checks of AGENTS.md: see the report of the main session.

Open points:

- `popup` takes the selection only. The selection keeps the scan, so a subsequent scan does not change an open popup.
- The Music section button shows the same label as the status bar control.
- A skipped change of a file in use is in the count of the section. It has no check box.
- The two changes of one group show in one line, separated by a line break.
- A file in use or unreadable is not in the plan. Thus the write report does not show it. The popup message names these files.
- The walk found two observations that are possibly older than this task. They are not task 002 defects. The WOAR line showed a difference with two values that look equal. After the write, TYER showed "file: no value". `mid3v2` saves ID3v2.4, which has no TYER frame. The cause is not known.
- The popup has a large space between the header "Skipped (1)" and the file title.

The main session review made these changes:

- A line row is in the row of its file. GPUI gives a click to each row at the pointer position. Thus one click on a line toggled the line and the file. The row stops the click after the correction. `adr_0108_click_on_a_line_does_not_toggle_its_file` tests it.
- The check box primitive has no Tab stop. After the correction, each check row of the dialog is a Tab stop (`tab_index(0)`), and Space on a focused row is a click. The row shows the focus ring only after a key press (`focus_ring_shows`).
- The popup stopped the app when it opened. `open_confirmation_dialog_from` used the display function for `destructive`, and that function read `LibraryApp` in its own update. After the correction, the caller gives `destructive`, and the open does not use the display function. `adr_0108_open_from_the_update_of_the_display_owner` tests it.
- The agent walk on Xvfb, 2026-11-21: V1 to V6 gave the correct result. The visual criteria stay open for the operator in section 48.
