# ADR 0097 Task 006: Each Text And Each Control Records A Probe

Status: Ready - 2026-11-14. It waits for tasks [002](adr-0097-task-002-list-pages-fit.md) to [005](adr-0097-task-005-sheets-dialogs-and-menus-fit.md), because their tests draw the text that this task probes. This is phase 6 of the [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md).

## Goal

Each text of the shared primitives, the composites and the shells goes through `Label`, or records a fit probe. Each interactive control records a control probe. Then the tests of phases 1 to 5 examine all the text and all the controls that they draw.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) Decision 4 item 1: the shared text and control primitives record fit probes.
- musicindex ADR 0065 R-FIT-3.
- The phase plan, Risks: a text that does not go through a probed primitive has no check. Only `Button` records a control probe.

## Recorded Facts

These facts come from a ChatGPT code search of 2026-11-14. The implementer examines each fact before it uses the fact. The search found 31 files in `src/ui/primitives` and `src/ui/composites` that draw a text with no probe.

- Primitives: `ContextMenu` empty text (`src/ui/primitives/context_menu.rs:716`), `LoadingMessage` (`src/ui/primitives/loading.rs:48`), `MultilineText` (`src/ui/primitives/multiline_text.rs:124`), `SectionHeader` (`src/ui/primitives/section_header.rs:122`) and the badges (`src/ui/primitives/status_badge.rs:26`, `:42`).
- Composites: `action_row`, `detail_grid`, `detail_header`, `download_status_item`, `file_header`, `frame_shell`, `list_page_head`, `live_status_strip`, `log_frame`, `maintenance_forms`, `maintenance_page`, `musicbrainz_panel`, `playlist_popover`, `release_detail_surface`, `search_field_list`, `selectable_text`, `settings`, `show_card`, `show_detail_panel`, `startup_report`, `tag_badge`, `track_detail_surface`, `track_drag`, `track_header`, `track_metadata_grid` and `track_row`. Each is in `src/ui/composites/`.
- Shells: for example `src/ui/shells/queue_now_playing.rs:200`, `src/ui/shells/playlist.rs:1098`, `src/ui/shells/entity.rs:286`, `src/ui/shells/publisher.rs:307`, `src/ui/shells/search_results_inspector.rs:349`, `src/ui/shells/show.rs:395` and `src/ui/shells/library/track_detail_metadata_values.rs:108`.
- Controls with no control probe: `Checkbox` (`src/ui/primitives/checkbox.rs:108`), the component `Input` of Settings and of the playlist popover (`src/ui/composites/settings.rs:228`, `src/ui/composites/playlist_popover.rs:444`), the rows and the examples of the search field list (`src/ui/composites/search_field_list.rs:242`, `:275`) and the header of `DisclosureGroup` (`src/ui/composites/disclosure_group.rs:249`).
- Tabs, segmented controls and the menu items use `Button`, so they have its probe.
- `selectable_text` draws a `StyledText` (`src/ui/composites/selectable_text.rs:190`). Its text layout can go to `fit_probe::probe`.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Label | reuse | src/ui/primitives/label.rs | A plain text moves to `Label`, which records the probe. |
| Fit probe | reuse | src/ui/primitives/fit_probe.rs | `probe`, `control_probe` and `control_text_probe` record a text or a control that cannot use `Label` or `Button`. |
| Checkbox | extend | src/ui/primitives/checkbox.rs | Add a control probe. |
| Disclosure group | extend | src/ui/composites/disclosure_group.rs | Add a control probe to its header. |
| Search field list | extend | src/ui/composites/search_field_list.rs | Add a control probe to each row and each example. |

## Required Changes

Do the parts in sequence. Run the fit tests after each part.

1. Part A, the primitives: move each text of the primitives in Recorded Facts to `Label`. If a text cannot move, add `fit_probe::probe` with its text layout. Add a control probe to `Checkbox`.
2. Part B, the composites: do the same for each composite in Recorded Facts. Add a control probe to these controls: the header of `DisclosureGroup`, the rows and examples of the search field list, and each text input.
3. Part C, the shells: search `src/ui/shells` for a text that does not go through `Label`, `Button` or a probed composite. Do the same for each one.
4. Keep the look of each text. When a text moves to `Label`, keep its size, weight, color and wrap. If `Label` cannot give the same look, add a probe and do not move the text.
5. Give each new probe a name that identifies its element in a failure message.
6. In the same change, set the phase 6 row of the phase plan to Done with the date. Remove the two Risks that phase 6 closes: the text with no probe and the controls with no control probe.

## Do Not Change

- No change to a layout or a token. A fault that the tests find gets its own task.
- No change to the look of a text.
- No probe in a production binary.

## Acceptance Criteria

Mechanical:

- Each file of Recorded Facts draws its text through `Label` or records a probe. The report lists each file and the method.
- A unit test draws a `Checkbox` across the edge of a box with `overflow_hidden` and requires an edge cut.
- A unit test puts a `Checkbox` on a `Button` and requires an overlap.
- The fit tests of phases 1 to 5 record more probes than before this task. The report gives the count before and after for each test.
- The fit tests find no fault, or the report lists each fault (Stop And Ask).
- The commit checks of AGENTS.md pass.

Visual: none for the fit. The look of each text must not change. The implementer compares a screenshot of each changed surface before and after, with the procedure of `docs/runbooks/`. The report gives the surfaces that it compared.

## Test Commands

- `cargo test --lib ui::primitives`
- `cargo test --lib app::fit_check`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Report after each part. Give the files changed, the method for each file and the probe counts. Give the full text of each fault message that a fit test found.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a fit test finds a fault. Do not change the layout. Report each message and the part that made the test find it.
- Stop if a text cannot move to `Label` and cannot give its text layout to a probe.
- Stop if the change of one part touches more than 40 files. Report the list, so that the main session can divide the part.
