# ADR 0097 Task 005: The Sheets, Dialogs And Menus Fit At The Two Widths

Status: Ready - 2026-11-14. It waits for [task 002](adr-0097-task-002-list-pages-fit.md) and [task 003](adr-0097-task-003-item-pages-fit.md), because each overlay opens on their pages. This is phase 5 of the [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md).

## Goal

A GPUI layout test opens each sheet, dialog, menu and popover at 390 px and at 940 px. The fit probes find no fault of musicindex ADR 0065 R-FIT-3. The surfaces are Inspect sources, each confirmation dialog, the playlist popover, the context menus and the search field list.

A control in an overlay does not overlap a control of the page below it. The fault finder compares only the controls of one layer.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) Decisions 2, 4, 5 and 6. Decision 4 item 2 gives the overlap fault.
- musicindex ADR 0065 R-FIT-2 item 4 and R-FIT-3.

## Recorded Facts

These facts come from a ChatGPT code search of 2026-11-14. The implementer examines each fact before it uses the fact.

- The overlap fault compares each pair of control probes and has no layer (`src/ui/primitives/fit_probe.rs:117` to `:121`, examined). Thus a menu item above a page button gives an overlap.
- Inspect sources: `open_inspect_sources_sheet(window, cx)` on a Library track (`src/library/app_impl.rs:4006`). `render_library_inspect_sources_sheet_body` draws it.
- `open_confirmation_dialog` calls `window.open_dialog`, and its builder calls `confirmation_dialog` (`src/ui/composites/confirmation_dialog.rs:118`). Nine sites open it, in seven kinds:
  - Update file tags (`src/library/app_impl.rs:1435`).
  - Delete a playlist, from its row menu (`src/library/app_impl.rs:2109`) and from its page (`src/ui/shells/library/playlist_detail.rs:223`).
  - Remove a track from all playlists (`src/library/app_impl.rs:2232`).
  - Remove a Library track or feed (`src/library/app_impl.rs:3460`).
  - Remove a selection (`src/library/app_impl.rs:5479`).
  - Delete a saved search, from a row (`src/app/saved_search_dispatch.rs:135`) and from its page (`:148`).
  - Use Defaults (`src/app/settings.rs:59`).
- The public playlist copy dialog has its own builder (`src/library/public_playlists.rs:285`).
- `AddToPlaylist` has a list mode and a create mode (`src/ui/composites/playlist_popover.rs:185`). Its trigger id ends in `-btn`.
- `ContextMenu` (`src/ui/primitives/context_menu.rs:543`) and `PointerContextMenu` (`:274`) draw the menus. `LibraryApp::open_focused_row_menu` opens a row menu from the keyboard (`src/library/app_impl.rs:4728`).
- `SearchFieldList` draws the search field list (`src/ui/composites/search_field_list.rs:332`). A test can make it directly.
- `TopApp` calls `render_window_layers` for the sheets and the dialogs (`src/ui/shells/window_layers.rs:13`).

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Shell fit harness | reuse | src/app/fit_check.rs | Task 002 moves it to `src/app/fit_check/mod.rs`. |
| Fit fixture adapter | reuse | src/library/app_impl.rs | Task 002 adds `src/library/fit_fixtures.rs`. Use it for the pages below the overlays. |
| Fit probe and fault finder | extend | src/ui/primitives/fit_probe.rs | Add a layer to each probe. Compare the controls of one layer only. |
| Window layers | extend | src/ui/shells/window_layers.rs | Mark the sheet and dialog layer for the probes in a test binary. |
| Popover | extend | src/ui/primitives/popover.rs | Mark the popover layer for the probes in a test binary. |
| Confirmation dialog | reuse | src/ui/composites/confirmation_dialog.rs | `open_confirmation_dialog` opens each confirmation. |

## Required Changes

1. In `fit_probe`, record the layer of each probe. The page is the first layer. Each open sheet, dialog, popover and menu is a layer above it. The overlap fault compares two controls only when they are in the same layer. Add a unit test for each result of Acceptance Criteria.
2. Mark each overlay layer in a test binary only. A production binary records nothing.
3. Put the phase 5 test in `src/app/fit_check/overlays.rs`.
4. Open each surface at each width of `WIDTHS`, with long names:
   - Inspect sources on a Library track.
   - One confirmation dialog of each of the seven kinds, and the public playlist copy dialog.
   - The playlist popover in its list mode with a minimum of 3 long playlist names, and in its create mode.
   - The row menu of a release row, a track row, a playlist row and a playlist entry row. Add the "Add to playlist" submenu.
   - The search field list with its fields, with matching fields and with an error.
5. Give each fault message the surface.
6. In the same change, set the phase 5 row of the phase plan to Done with the date.

## Do Not Change

- No change to a layout or a token. A fault that the test finds gets its own task.
- No change to the overlap fault inside one layer.

## Acceptance Criteria

Mechanical:

- A unit test puts a control in an overlay layer above a page control. It requires no overlap.
- A unit test puts two controls on top of each other in one overlay layer. It requires an overlap.
- The test `adr_0097_the_overlays_fit_at_the_two_widths` opens each surface of Required Changes item 4 at 390 px and at 940 px.
- Each draw records one or more probes in the overlay layer. This proves that the overlay was drawn.
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

Give the files changed and the tests added. Give the full text of each fault message. Name each overlay text that does not go through `Label` or `Button`.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the test finds a fault. Do not change the layout. Report each message.
- Stop if an overlay does not draw in a `VisualTestContext`. Report the overlay and the cause.
- Stop if the layer needs a change to the production draw order.
