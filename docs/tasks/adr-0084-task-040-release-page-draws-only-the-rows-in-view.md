# ADR 0084 Task 040: The Release Page Shows Only The Rows In View

Status: Ready - 2026-11-16.

## Goal

A release page with many tracks scrolls without lag. Today each frame lays out and paints all track rows. On a release with 491 tracks, one scroll step takes approximately 300 ms.

## Authority

- [ADR 0084](../adr/0084-music-column-browser.md) Decision 4 and packet 021: one release page view model and one shell for the Library and the Index.
- ADR 0084 Decision 9: Back and Forward give the stored scroll offset of a detail page.
- ADR 0084 Decision 10 and packet 018: the detail column shows the row with the keyboard cursor (`ScrollReveal`).
- ADR 0084 task 037 (archived): the virtual list of the Releases column, `MusicColumnList`.
- [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) Decision 5: the pinned list bar. ADR 0090 invariant "keyboard reach": the focus reveal layer shows each control that takes the focus.
- [ADR 0039](../adr/0039-dynamic-type-ramp.md): the height of a track row does not change with the title length (test `adr_0039_track_row_height_is_independent_of_title_length`, `src/ui/composites/track_row.rs:792`).

Operator choice line, 2026-11-16: the operator chose to correct this defect before the walk of sections 15 to 17 continues.

## Recorded Facts

- 2026-11-16, operator: "a very long release such as Global Fedpilled Industrial Complex brings performance to its knees". The release binary also has the lag.
- 2026-11-16, agent measurement on Xvfb with the release binary and timing lines in a scratch worktree, on the Index release "GLOBAL FEDPILLED INDUSTRIAL COMPLEX" (491 tracks):
  - The render function of the app takes 5 to 7 ms. `render_index_release_page` takes 2 to 3 ms, and of this the view model takes approximately 1.5 ms.
  - The next frame starts approximately 304 ms after the render starts.
  - On "Waltz Tunes Volume 2" (21 tracks), the next frame starts approximately 32 ms after the render starts.
  - Thus the time is in the layout and the paint of the rows, not in the view model. A cache of the view model does not correct the lag.
- 2026-11-16: `render_release_page` (`src/ui/shells/release_page.rs:110` to `:125`) makes an element for each row. `track_section` (`src/ui/composites/release_detail_surface.rs:205`) puts all rows in one column in the scroll container of the detail column.
- 2026-11-16: the detail column scrolls with a `ScrollHandle`. These parts read that handle: the pinned list bar (`ListBarPin`), the focus reveal layer (`focus_reveal_layer`), the row reveal (`ScrollReveal`) and the stored offset of ADR 0084 Decision 9. The first three are in `src/ui/composites/music_column_scroll.rs`.
- 2026-11-16: `render_release_detail_shell` (`src/ui/shells/entity.rs:69`) has one caller, `render_release_page`. `render_release_page` has two callers: `render_library_feed_detail` (`src/app/feed_detail.rs:78`) and `render_index_release_page` (`src/app/search_dispatch.rs:1510`).
- 2026-11-16: a key that moves the cursor calls `self.detail_scroll_reveal().request()` (`src/library/detail_selection.rs:550`). Only the row with the cursor carries the reveal probe (`detail_row_probe`, `src/library/detail_selection.rs:255`).

## Decisions

- **D1.** The detail column keeps its `ScrollHandle` and its one scroll container. The track section shows only the rows in view and a margin of rows above and below. An empty block above and an empty block below give the full height of the rows that it does not show. Reasons:
  - The page header, the list bar and the controls after the rows stay in the element tree. Thus Tab reaches them at each scroll position (ADR 0090 invariant "keyboard reach").
  - `ListBarPin`, the focus reveal layer and the stored offset of ADR 0084 Decision 9 keep their current code.
  - A gpui `list` for the full column puts the header in one item. When the header scrolls away, the list does not show it, and its controls go out of the Tab sequence.
- **D2.** All rows have one height. The track section measures the height of one shown row at each paint and keeps it. Before the first measurement, it shows the first 40 rows.
- **D3.** A row reveal for a row that the section does not show calculates the offset. It uses the row index, the row height, the row space and the top of the track section. The section records its top at each paint, as `ListBarPin` records the bar position.
- **D4.** The view model stays as it is. The row elements, the thumbnails and the reveal probes are made only for the rows that the section shows.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Scroll parts of a Music column | extend | src/ui/composites/music_column_scroll.rs | Add the windowed row section adjacent to `ListBarPin` and `ScrollReveal`. Reuse `reveal_offset_y` for D3. |
| Release detail surface | extend | src/ui/composites/release_detail_surface.rs | `track_section` gets the windowed rows. It no longer gets a full list of row elements. |
| Release detail shell slots | extend | src/ui/shells/entity.rs | `ReleaseDetailBehaviorSlots::track_rows` gives a row count and a row function. It no longer gives all row elements. |
| Release page shell | extend | src/ui/shells/release_page.rs | `render_release_page` makes a row element only when the section asks for it. |
| Virtual Music list | reuse | src/ui/composites/music_column_scroll.rs | `MusicColumnList` is the pattern for the measured item height (`item_height`, `hinted_width`). It is not the container, because of D1. |
| Detail row reveal | extend | src/library/detail_selection.rs | The reveal also gets the row index, for D3. |

## Required Changes

1. Add the windowed row section to `src/ui/composites/music_column_scroll.rs`. It takes the `ScrollHandle`, the row count, the row space and a function that makes the row element for an index. It shows the rows in view, a margin of 10 rows on each side, and the two empty blocks.
2. Record the top of the section and the measured row height at each paint. When a recorded value changes, ask for one redraw, as `ListBarPin::probe` does.
3. Change `ReleaseDetailBehaviorSlots::track_rows` and `track_section` to use the section. Keep the title and the summary line of the track section.
4. In `render_release_page`, make the row element, the thumbnail and the probe of a row only in the row function. Do the same in `render_library_feed_detail` and `render_index_release_page` for the thumbnails and the probes. The Library and the Index each keep their own image source.
5. Do D3: when the cursor row is not shown, the reveal sets the offset from the row index. Then the probe of the shown row does the last correction.
6. Add tests:
   - The windowed section shows only the rows in view and the margin, for 491 rows in a 600 px viewport.
   - The height of the content is the same as with all rows shown.
   - A reveal of row 400 from the top scrolls so row 400 is in view.
   - With the column at row 400, Tab from "Reverse" reaches the track list. Shift+Tab from the track list reaches "Reverse".

## Do Not Change

- No change to `ReleasePageVm` or `release_page_vm`.
- No change to the Releases, Inbox, My playlists or Artists list columns.
- No change to the playlist page, the singles group page or the artist page. Tell in the report if each one has the same lag.
- No change to `ListBarPin`, `focus_reveal_layer` or the stored offset of ADR 0084 Decision 9. Only a read of the section top is permitted.
- No change to the keys in the list. ADR 0092 and ADR 0102 own them.

## Acceptance Criteria

Mechanical:

- A test proves that the section makes row elements only for the rows in view and the margin.
- A test proves that the content height is the same as the height of all rows.
- A test proves that a reveal of a row that is not shown scrolls the column to show it.
- A test proves the Tab sequence of the list with the column scrolled into the rows.
- The commit checks of AGENTS.md pass.

Visual, listed apart:

- V1: on "GLOBAL FEDPILLED INDUSTRIAL COMPLEX", the mouse wheel scrolls without lag, and the scrollbar thumb has the same length from the top to the end.
- V2: End in the track list moves the cursor to the last row and the column shows it. Home moves it to the first row.
- V3: Back from a track page gives the same scroll position.
- V4: the pinned list bar shows when the list bar scrolls away, and goes away when it comes back.

## Test Commands

- `cargo test --lib ui::composites::music_column_scroll`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed, the tests added, the measured row height and the margin. Give the result for the playlist, singles group and artist pages.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if two track rows of one release page have different heights.
- Stop if an architecture test forbids the change of `ReleaseDetailBehaviorSlots`. Give the test name.
- Stop if D1 cannot keep the pinned list bar or the stored offset without a change to their code.
