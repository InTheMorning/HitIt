# ADR 0102 Task 001: The Arrow Keys In The Tiles Views

Status: Waits - 2026-11-15. This packet starts when the operator opens it. It comes after the walk of sections 15 to 17 of [pending human checks](../pending-human-checks.md) and before [ADR 0101 task 001](adr-0101-task-001-shared-tokens.md).

## Goal

In the Tiles view of Releases and of Artists, each arrow key moves the focus to the tile in its direction. Up and Down go to the tile in the same column of the line above or below. Left and Right go to the tile before or after in the list. The lines come from the layout at each key press.

## Authority

- [ADR 0102](../adr/0102-the-arrow-keys-move-in-a-grid-of-tiles.md) Decisions 1 to 4 own this packet. Read them before the first edit.
- musicindex ADR 0078 R-GRIDKEY-1 (`../musicindex/docs/adr/0078-the-arrow-keys-move-in-a-grid-of-tiles.md`, at musicindex `574c4c4`). Items 1 to 6 are in lines 36 to 47. The vectors are in lines 52 to 70.
- [ADR 0092](../adr/0092-the-keys-of-a-list-row.md) Decision 1 owns the Tab stop, Enter, Home and End. This packet does not change them.
- [ADR 0094](../adr/0094-one-fold-header.md) owns Right and Left on a singles group row in the Rows view. This packet does not change the Rows view.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `3ab3edb`, read by the main session on 2026-11-15.

- `FocusStep` (`src/view_models/list_selection.rs:26`) has Up, Down, PageUp, PageDown, Home and End. It has no Left and no Right.
- `ListSelection::move_focus` (`src/view_models/list_selection.rs:259`) moves in the sequence of `visible`. It stops at the first and the last row.
- `move_music_selection_focus` (`src/app/keyboard.rs:534`) sends each step to `LibraryApp::move_selection_focus` (`src/library/app_impl.rs:4867`). Releases calls `reveal_releases_tile` in the Tiles view. Artists calls `move_artists_focus` (`:4738`), and `reveal_artists_row` calls `reveal_artists_tile` in the Tiles view.
- The keys "right" and "left" (`src/app/keyboard.rs:231` and `:237`) run `OpenFocusedGroup` and `CloseFocusedGroup`. These call `disclose_focused_group` (`src/library/app_impl.rs`). It acts on Releases in each view, and it does nothing on Artists.
- `render_tile_grid` (`src/ui/shells/library/releases_tiles.rs:75`) puts the tiles in one `flex_wrap` div. The `ScrollHandle` of the grid records the position of each child of the scroll content: the leading rows, the grid and the trailing rows. It does not record the position of each tile. This is an inference from the element tree. Examine it before you use it.
- `render_artists_tile_grid` (`src/ui/shells/library/releases_tiles.rs:327`) uses `render_tile_grid`. Its `tiles` holds the heading and the status row of each group at full width.
- `ScrollReveal` (`src/ui/composites/music_column_scroll.rs:289`) shows the tile with the probe at the next layout.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Key model | extend | `src/view_models/list_selection.rs` | `FocusStep` and `ListSelection::move_focus` own each focus move. Add Left, Right and the line move there. Do not write a second key model. |
| Key path | extend | `src/app/keyboard.rs` | `move_music_selection_focus` sends each step. In a Tiles view, Left and Right go to the key model, not to `disclose_focused_group`. |
| List key handler | extend | `src/library/app_impl.rs` | `move_selection_focus` and `move_artists_focus` give the steps to the key model and call the tile reveal. |
| Tile grid | extend | `src/ui/shells/library/releases_tiles.rs` | `render_tile_grid` holds the grid of the two views. Record the position of each tile there. |
| Tile reveal | reuse | `src/ui/composites/music_column_scroll.rs` | `ScrollReveal` shows the focused tile (R-GRIDKEY-1 item 6). |

## Required Changes

1. **The key model.** Add Left and Right to `FocusStep`. Add a move that takes the lines of the grid: for each line, the indices of its tiles in the sequence of `visible`. A line with no tile is not in the input.

   Left and Right move in the sequence. Up and Down follow R-GRIDKEY-1 item 3. With no lines, Up and Down move in the sequence, as before. Page Up and Page Down do not change (ADR 0102 Decision 3).
2. **The lines from the layout.** At each layout, record the position of each tile of the grid. At each key press, make the lines from the last positions. Tiles with the same top edge are on one line. Do not calculate a column count from the window width (ADR 0102 Alternatives).
3. **The key path.** In the Tiles view of Releases and of Artists, the arrow keys use the move of step 1 with the lines of step 2. In the Tiles view of Releases, Left and Right do not open or close a singles group (ADR 0102 Decision 2). The Rows view does not change.
4. **The reveal.** After each move, the focused tile scrolls into view with the reveal of each view.

## Do Not Change

- No change of the Rows view of a list.
- No change of the Tab stop, Enter, Home, End, Page Up or Page Down.
- No change of a tile look.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test of the key model gives each of the 11 vectors of musicindex ADR 0078 with 10 tiles in 4 columns. The message of each assertion names ADR 0102 Decision 1 and the vector.
- M2: a test of the key model uses two lines of different length with a heading line between them. Up and Down give the move of R-GRIDKEY-1 item 3 over the heading line.
- M3: a GPUI test opens the Tiles view of Releases with 10 tiles at two window widths. One width gives 4 columns, and one gives a different count. At each width, Down from tile 0 goes to the first tile of the second line. The layout gives that line. This proves R-GRIDKEY-1 item 5.
- M4: a GPUI test proves two facts. In the Tiles view of Releases, Right on a singles group tile moves the focus and does not open the group. In the Rows view, Right opens the group.
- M5: a GPUI test of the Artists Tiles view proves a move over a heading. Down from the last line of one group goes to a tile of the next group.
- M6: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo test --test architecture_tests` and `cargo build --bin hitit` pass.

Visual, for the operator, in the Tiles view of Releases and of Artists:

- V1: each arrow key moves to the tile in its direction. Up on the first line and Down on the last line do not move.
- V2: at a narrow window and at a wide window, Down goes to the tile below.
- V3: each focused tile is in view.

## Report

Give the files changed and the tests added. Tell how the layout gives the position of each tile.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the position of each tile cannot come from the layout without a change in a shared primitive. Report the shape of the change that it needs.
- Stop if Enter on a singles group tile does not open the group. Then no key opens a group in the Tiles view.
