# ADR 0102: The Arrow Keys Move In A Grid Of Tiles

## Status

Accepted - 2026-11-15. The orchestrator session relayed musicindex ADR 0078 on 2026-11-15 and asked HitIt to use it. The operator asked for the keys in the walk of 2026-11-15.

- Bar: crosses repositories. musicindex ADR 0078 R-GRIDKEY-1 binds musicindex.org, Studio and HitIt.
- Changes: [ADR 0092](0092-the-keys-of-a-list-row.md). In a Tiles view, the arrow keys follow R-GRIDKEY-1, not the sequence of R-LISTKEY-1.

Uses: musicindex ADR 0078 R-GRIDKEY-1 (`../musicindex/docs/adr/0078-the-arrow-keys-move-in-a-grid-of-tiles.md`, musicindex commit `574c4c4`).

In this record, a grid of tiles is a list that shows its items as tiles on more than one line. A line is the tiles at the same height in the layout. A column is the position of a tile in its line. These terms are from musicindex ADR 0078.

## Context

- In the walk of 2026-11-15, Tiles view of Artists, the operator gave this text: "up/down/left/right arrows to choose tiles in that direction". [ADR 0092 task 001](../tasks/adr-0092-task-001-artists-list-focus.md), section "Walk Correction - 2026-11-15", sent the question to the orchestrator.
- HitIt has two grids of tiles: the Tiles view of Releases and the Tiles view of Artists. ADR 0092 gives them the keys of R-LISTKEY-1. Up and Down move one tile in the sequence.
- In Releases, Right and Left open and close a singles group, or move the focus from a single to its group (`disclose_focused_group`, `src/library/app_impl.rs`). This applies in each view. In Artists, Left and Right do nothing.
- `ListSelection::move_focus` (`src/view_models/list_selection.rs`) moves in the sequence of the list. It has no data about lines.
- The Artists Tiles view has a heading and a status row for each group. Each is a full line. Thus a line can have fewer tiles than the window width permits.

## Decision

### 1. A grid of tiles follows R-GRIDKEY-1

The Tiles views of Releases and Artists follow musicindex ADR 0078 R-GRIDKEY-1 items 1 to 6:

1. The grid is one Tab stop. Enter opens the tile.
2. Left and Right move one tile in the sequence of the list. They go across lines. Left on the first tile and Right on the last tile do not move.
3. Up and Down move one line, to the tile in the same column. When that line has no tile in that column, the focus goes to the last tile of that line. Up on the first line and Down on the last line do not move.
4. Home goes to the first tile, and End goes to the last tile.
5. The app reads the lines from the layout at each key press.
6. The focused tile scrolls into view.

A heading line and a status line have no tile. Up and Down go over them to the subsequent line with a tile.

### 2. The Rows view keeps its keys

In the Rows view, each line has one row. Up and Down move one row (R-LISTKEY-1 item 2). In the Rows view of Releases, Right and Left keep the singles group step of [ADR 0094](0094-one-fold-header.md).

In the Tiles view of Releases, Right and Left move as item 2 of Decision 1 says. They do not open or close a singles group. Enter on a group tile does what a click on it does.

### 3. Page Up and Page Down

R-GRIDKEY-1 gives no Page Up and no Page Down. In a grid of tiles, they keep their step in the sequence of the list. Main session choice, 2026-11-15. It stays open for the orchestrator.

### 4. The key model owns the move

The key model of the view model calculates each move from the lines that the layout gives. The renderer calculates no move (the Durable Set: typed action state). The key model tests the vectors of musicindex ADR 0078 with 10 tiles in 4 columns.

## How This Decision Is Checked

The operator found the fault on 2026-11-15. Thus the key model gets a test.

Mechanical, in [ADR 0102 task 001](../tasks/adr-0102-task-001-tiles-arrow-keys.md):

- A test of the key model with the 11 vectors of musicindex ADR 0078.
- A test that the lines come from the layout at each key press, at two window widths.

Visual, for the operator: open the Tiles view of Releases and of Artists. Move through each with the arrow keys only. Each key moves to the tile in its direction.

## Alternatives Considered

### Up and Down in the sequence

ADR 0092 did this. The operator asked for the tile in the direction of the arrow.

### A column count from the window width

The app can calculate the column count from the width and the tile width. It then gets the incorrect line in a group with fewer tiles than the width permits, and after a change of the UI scale. Item 5 of R-GRIDKEY-1 asks for the layout.

## Consequences

- The ADR 0092 task 001 walk correction of 2026-11-15 does not change the arrow keys. This ADR replaces that part.
- In the Tiles view of Releases, a singles group opens only with Enter or a click.
- A change to R-GRIDKEY-1 needs a superseding record in musicindex.
