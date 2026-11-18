# ADR 0097 Task 005: The Sheets, Dialogs And Menus Fit At The Two Widths

Status: Ready - 2026-11-14. It waits for [task 002](adr-0097-task-002-list-pages-fit.md) and [task 003](adr-0097-task-003-item-pages-fit.md), because each overlay opens on their pages. This is phase 5 of the [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md).

## Goal

A GPUI layout test opens each sheet, dialog, menu and popover at 390 px and at 940 px. The fit probes find no fault of musicindex ADR 0065 R-FIT-3. The surfaces are Inspect sources, each confirmation dialog, the playlist popover, the context menus and the search field list.

A control in an overlay does not overlap a control of the page below it. The fault finder compares only the controls of one layer.

The test also finds a sheet that is wider than the window less the window margin (musicindex ADR 0077 R-SHEET-2). The test keeps a list of each sheet, dialog and popover. A "sheet" in this packet is a sheet, a dialog, a menu or a popover.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) Decisions 2, 4, 5 and 6. Decision 4 item 2 gives the overlap fault.
- musicindex ADR 0065 R-FIT-3. musicindex ADR 0077 R-SHEET-2 replaces R-FIT-2 item 4 (musicindex `d3db352`, orchestrator text of 2026-11-15):
  1. At each width of R-FIT-2, the check opens each page and each sheet of the app.
  2. Each app keeps a list of each sheet in its layout test. Each new sheet goes into that list in the same commit.
  3. The check examines an open sheet as a panel of R-FIT-3. It finds each fault of that rule in the sheet. It also finds a sheet that is wider than the window less the window margin, or that has content in that margin. Only a bottom sheet in a narrow window can be as wide as the window. Then the window margin is its inner padding, and its content must not go into that padding.
  4. HitIt makes this check as GPUI layout tests.
  5. The failure message names this record, the sheet, the width and the fix.
- musicindex ADR 0068 R-SPACE-1 item 2: the window margin is 16 px in a narrow window and 20 px in a wide window. For HitIt, narrow is below 940 px. This is an inference of the orchestrator. Thus a sheet is at most 358 px wide at 390 px, and at most 900 px wide at 940 px.
- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 11 adopts musicindex ADR 0077.

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

The [third agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-third.md) defect 2 is in the scope of this task: at 390 px, in Settings, General, in Light, the Use Defaults dialog is cut at the left edge of the window. The title shows as "se the default settings?". Screenshot: `../reviews/screenshots/agent-walk-2026-11-18-third/use-defaults-dialog-390-cut.png`. The fit check of the dialogs must find this fault before the fix, and must pass after it.
- The public playlist copy dialog has its own builder (`src/library/public_playlists.rs:285`).
- `AddToPlaylist` has a list mode and a create mode (`src/ui/composites/playlist_popover.rs:185`). Its trigger id ends in `-btn`.
- `ContextMenu` (`src/ui/primitives/context_menu.rs:543`) and `PointerContextMenu` (`:274`) draw the menus. `LibraryApp::open_focused_row_menu` opens a row menu from the keyboard (`src/library/app_impl.rs:4728`).
- `SearchFieldList` draws the search field list (`src/ui/composites/search_field_list.rs:332`). A test can make it directly.
- `TopApp` calls `render_window_layers` for the sheets and the dialogs (`src/ui/shells/window_layers.rs:13`).

## The Sheet List

The main session made this list on 2026-11-15 from a ChatGPT search at HitIt `80e8607`. The main session examined the widths. The "Expected fault" column is an inference from a code read, at 390 px, where the window allows 358 px.

| Sheet | Built at | Opens at | Width | Expected fault |
|---|---|---|---|---|
| Inspect Sources | `src/ui/composites/inspect_sources_sheet.rs:44` | `src/library/app_impl.rs:4101` | 560 px fixed (`src/ui/layouts.rs:32`) | Width: 560 px |
| Each confirmation dialog: Update File Tags, Delete Playlist (row and page), Remove from All Playlists, Remove from Library (track and album), Remove the selection, Delete Saved Search (row and page), Use Defaults, Clear the Show Cue | `src/ui/composites/confirmation_dialog.rs:137`. Clear the Show Cue: `src/app/show.rs:74` | `src/library/app_impl.rs:1441`, `:2109`, `:2284`, `:3510`, `:5541`, `src/ui/shells/library/playlist_detail.rs:225`, `src/app/saved_search_dispatch.rs:142` and `:161`, `src/app/settings.rs:60`, `src/app/show.rs:71` | Dialog 448 px, the default of `gpui-component`, with no max width. Body 320 px (`confirmation_dialog.rs:221`) | Width: 448 px |
| Copy Public Playlist | `src/ui/shells/library/public_playlists.rs:49` | `src/library/public_playlists.rs:304` | Dialog 448 px. Body 320 px | Width: 448 px |
| Search field list | `src/ui/composites/search_field_list.rs:125` | `src/app/search_field_list.rs:454` | 480 px (`Size::ColumnTall`). Window margin 8 px (`:158`) | Width: 480 px. The snap margin is 8 px, not 16 px |
| MusicBrainz release picker | `src/ui/composites/musicbrainz_panel.rs:128` | `src/ui/composites/musicbrainz_panel.rs:151` | 320 px to 520 px (`src/ui/layouts.rs:61`) | Width: possible with a long name |
| Add to playlist popover: release page, track page, singles group, selection bar, list bar | `src/ui/shells/release_page.rs:204`, `src/ui/shells/track_page.rs:148`, `src/ui/shells/library/singles_group.rs:281`, `src/ui/composites/selection_action_bar.rs:188` and `:346` | `src/ui/composites/playlist_popover.rs:227` | 220 px. Choices max 320 px high with no scroll (`playlist_popover.rs:347`) | Edge cut of a long list (R-SHEET-1 item 5) |
| Page selector: Settings groups, Diagnostics, Recovery, database task | `src/app/settings.rs:145` and `:155`, `src/ui/composites/startup_report.rs:250`, `src/ui/composites/maintenance_forms.rs:383` | `src/ui/composites/maintenance_page.rs:95` | 220 px, max 220 px high with scroll | None. The snap margin is 8 px, not 16 px |
| Menus: tab bar More, list sort, saved search row, release page, track page, Show event picker, Show More, Remove…, track row, release row, right-click row, selected text Copy | `src/ui/composites/tab_bar.rs:106`, `src/ui/composites/list_page_head.rs:202`, `src/ui/shells/library/search_page_list.rs:159`, `src/ui/shells/release_page.rs:265`, `src/ui/shells/track_page.rs:224`, `src/ui/composites/show_detail_panel.rs:619` and `:645`, `src/ui/composites/remove_menu.rs:41`, `src/ui/composites/track_row_menu.rs:341` and `:273`, `src/ui/composites/release_row_menu.rs:83`, `src/ui/composites/selectable_text.rs:219` | `src/ui/primitives/context_menu.rs:573`. Right-click: `src/ui/composites/track_row_menu.rs:115` | 220 px (`Size::MenuRegular`), max 220 px high with scroll. Window margin 8 px (`context_menu.rs:382`) | None. The snap margin is 8 px, not 16 px |
| Row menu levels: Add to Playlist and New Playlist | `src/ui/composites/track_row_menu.rs:177` | In the row menu | The sizes of the Add to playlist popover (`track_row_menu.rs:225` and `:251`) | Edge cut of a long list |

Inference: the frame actions menu cannot open now (`src/view_models/workspace/chrome.rs:116`). It is not in the list.

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
3. Put the phase 5 test in `src/app/fit_check/overlays.rs`. Put the list of "The Sheet List" in that file as one constant list. Each entry has the sheet name and a function that opens it. A comment above the list says: "musicindex ADR 0077 R-SHEET-2 item 2: each new sheet goes into this list in the same commit".

   Add the width fault: the sheet is wider than the window less the window margin on each side. No HitIt sheet is a bottom sheet. Inspect Sources opens at the right edge (`window.open_sheet`, `gpui-component` 0.6.1 `window_ext.rs:115`). Thus each HitIt sheet keeps the window margin on each side. The margin is 16 px below 940 px and 20 px at 940 px and above. The fault message has this form:

   ```text
   musicindex ADR 0077 R-SHEET-1: at 390 px the sheet "Inspect Sources" is 560 px wide, and the window allows 358 px.
   Give the sheet the width min(design width, window width - 2 × margin), and let each child shrink.
   ```
4. Open each surface of "The Sheet List" at each width of `WIDTHS`, with long names. These items give the content:
   - Inspect sources on a Library track.
   - One confirmation dialog of each of the seven kinds, and the public playlist copy dialog.
   - The playlist popover in its list mode with a minimum of 3 long playlist names, and in its create mode.
   - The row menu of a release row, a track row, a playlist row and a playlist entry row. Add the "Add to playlist" submenu.
   - The search field list with its fields, with matching fields and with an error.
5. Give each fault message the surface.
6. In the same change, set the phase 5 row of the phase plan to Done with the date.

## Do Not Change

- No change to a layout or a token. [ADR 0101 task 013](adr-0101-task-013-sheets-fit-the-window.md) corrects the width faults of R-SHEET-1. Each other fault that the test finds gets its own task.
- No change to the overlap fault inside one layer.

## Acceptance Criteria

Mechanical:

- A unit test puts a control in an overlay layer above a page control. It requires no overlap.
- A unit test puts two controls on top of each other in one overlay layer. It requires an overlap.
- The test `adr_0097_the_overlays_fit_at_the_two_widths` opens each surface of Required Changes item 4 at 390 px and at 940 px.
- Each draw records one or more probes in the overlay layer. This proves that the overlay was drawn.
- A unit test gives a sheet of 400 px at a window of 390 px. It requires the width fault, and the message names musicindex ADR 0077, the sheet, the width and the fix.
- The list in `src/app/fit_check/overlays.rs` has each entry of "The Sheet List".
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

- Stop if the test finds a fault that is not an expected width fault of "The Sheet List". Do not change the layout. Report each message.
- The expected width faults are not a stop. Mark each one as an expected fault in the test, with the name of ADR 0101 task 013. Task 013 removes each mark.
- Stop if an overlay does not draw in a `VisualTestContext`. Report the overlay and the cause.
- Stop if the layer needs a change to the production draw order.
