# ADR 0101 Task 013: Each Sheet Fits The Window

Status: Waits - 2026-11-15. This packet starts after [ADR 0097 task 005](adr-0097-task-005-sheets-dialogs-and-menus-fit.md), because that test finds each width fault. It comes before [ADR 0097 task 006](adr-0097-task-006-each-text-and-control-records-a-probe.md).

## Goal

Each sheet, dialog, menu and popover is not wider than the window less the window margin. Its content shrinks, wraps and scrolls only vertically. The test of ADR 0097 task 005 then finds no width fault, and this packet removes each expected fault mark.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 11 adopts musicindex ADR 0077.
- musicindex ADR 0077 R-SHEET-1 (musicindex `d3db352`, orchestrator text of 2026-11-15). A "sheet" is a sheet, a dialog or a popover:
  1. A sheet is not wider than the window. Its width is the smaller of two values: its design width, and the window width less the window margin on each side.
  2. Each child of a sheet can become narrower than its content. A field, a select or a button row has no fixed width that is larger than the sheet.
  3. Text wraps between words. A node key or an npub uses the short form of musicindex ADR 0076 R-ADDRESS-1. Other text with no break point, for example a URL, has the break attribute of musicindex ADR 0065 R-FIT-3 item 3.
  4. A row of fields or buttons that does not fit wraps to the next line (musicindex ADR 0080 R-FIT-1 item 2).
  5. A sheet scrolls only vertically. It has no horizontal scroll.
- musicindex ADR 0077 R-SHEET-1 item 1 also lets a bottom sheet in a narrow window be as wide as the window. Then the window margin is its inner padding. No HitIt sheet is a bottom sheet, because Inspect Sources opens at the right edge. Do not change a sheet into a bottom sheet in this packet.
- musicindex ADR 0068 R-SPACE-1 item 2: the window margin is 16 px in a narrow window and 20 px in a wide window. For HitIt, narrow is below 940 px. This is an inference of the orchestrator.
- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) owns the fit check. The `rust-dev` skill applies to each Rust change.

## Recorded Facts

"The Sheet List" of [ADR 0097 task 005](adr-0097-task-005-sheets-dialogs-and-menus-fit.md#the-sheet-list) gives each sheet with its file, its line and its width. The main session examined these facts at HitIt `530b844`:

- `DialogProps::default` of `gpui-component` 0.6.1 gives a width of 448 px and no max width (`dialog/dialog.rs:156`). `Dialog::max_w` sets a max width (`:394`).
- `Sheet` of `gpui-component` 0.6.1 uses its `size` with no limit (`sheet.rs:179`). HitIt gives 560 px (`src/ui/layouts.rs:32`).
- The search field list is `Size::ColumnTall`, 480 px, and snaps to the window with a margin of `Spacing::SM`, 8 px (`src/ui/composites/search_field_list.rs:158`).
- A menu snaps to the window with a margin of `Spacing::SM`, 8 px (`src/ui/primitives/context_menu.rs:382`).
- The playlist choices have a max height of 320 px and no scroll (`src/ui/composites/playlist_popover.rs:347`).
- The MusicBrainz picker is 320 px to 520 px wide (`src/ui/layouts.rs:61` and `:62`).

## The Sheets That Differ From R-SHEET-1

This list is an inference from a code read, at 390 px, where the window allows 358 px:

1. Inspect Sources: 560 px wide.
2. Each confirmation dialog: 448 px wide.
3. Copy Public Playlist: 448 px wide.
4. The search field list: 480 px wide. Its snap margin is 8 px.
5. The MusicBrainz release picker: it can be 520 px wide with a long name.
6. The Add to playlist popover and the Add to Playlist level of a row menu: a long list has no scroll (item 5).
7. Each menu and page selector: the snap margin is 8 px, not 16 px. The width fits.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Window margin | extend | `src/ui/tokens.rs` | `Spacing` gives each space. Add the window margin function. It gives 16 px below 940 px and 20 px at 940 px and above. |
| Sheet width | extend | `src/ui/layouts.rs` | It holds the sheet and menu widths. Add one function that gives min(design width, window width - 2 × margin). |
| Confirmation dialog | extend | `src/ui/composites/confirmation_dialog.rs` | It opens each confirmation. Give the dialog the width of the function. |
| Inspect Sources sheet | extend | `src/ui/composites/inspect_sources_sheet.rs` | It opens the sheet. Give it the width of the function. |
| Search field list | extend | `src/ui/composites/search_field_list.rs` | Give it the width of the function and the window margin. |
| Context menu | extend | `src/ui/primitives/context_menu.rs` | Give its snap the window margin. |
| Playlist popover | extend | `src/ui/composites/playlist_popover.rs` | Give its choices a vertical scroll. |

## Required Changes

1. **The window margin.** One function gives the window margin for a window width, with a comment that cites R-SPACE-1 item 2.
2. **The sheet width.** One function gives min(design width, window width - 2 × margin). Each sheet of "The Sheets That Differ From R-SHEET-1" uses it. The public playlist dialog uses it too.
3. **The children.** Each child of a changed sheet can shrink. A button row wraps when it does not fit (R-SHEET-1 items 2 and 4).
4. **The snap margin.** Each menu, popover and the search field list snap to the window with the window margin.
5. **The vertical scroll.** The playlist choices scroll vertically when the list is higher than 320 px.
6. **The marks.** Remove each expected fault mark of ADR 0097 task 005.
7. **The text.** Each text in a sheet wraps between words. Report each text with no break point. Do not change an address. Task 004 owns the short form.

## Do Not Change

- No change of a design width at 940 px.
- No change of the fit probe or the fault finder. ADR 0097 task 005 owns them.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test of the margin function gives 16 px at 390 px and at 939 px, and 20 px at 940 px.
- M2: a test of the width function gives 358 px for a design width of 560 px at 390 px, and 560 px at 940 px.
- M3: `adr_0097_the_overlays_fit_at_the_two_widths` passes with no expected fault mark.
- M4: a test proves that the playlist choices scroll when they are higher than 320 px.
- M5: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light:

- V1: at 390 px, Inspect Sources, a confirmation dialog and the search field list have a space of 16 px on each side.
- V2: at 940 px, each sheet has its design width as before.
- V3: a playlist popover with 20 playlists scrolls.

## Test Commands

- `cargo test --lib ui::tokens`
- `cargo test --lib ui::layouts`
- `cargo test --lib app::fit_check`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M4 by name. Give each sheet with its width at 390 px before and after. Give each text with no break point.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a `gpui-component` sheet or dialog cannot take a width that changes with the window.
- Stop if the test of ADR 0097 task 005 finds a fault that is not in "The Sheets That Differ From R-SHEET-1".
- Stop if a sheet needs a horizontal scroll for its content.
