# ADR 0084 Task 041: The Row Reveal Clears The Pinned Bar And Runs At Entry

Status: Implemented - operator visual checks open, 2026-11-16.

## Goal

On a detail page with a track list, the row with the keyboard cursor is always in view and not below the pinned list bar. This applies after Tab into the list, and after a key that moves the cursor.

## Authority

- [ADR 0084](../adr/0084-music-column-browser.md) Decision 10 and packet 018: the detail column shows the row with the keyboard cursor (`ScrollReveal`).
- [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) Decision 5: the pinned list bar.
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Invariants: the list shows the cursor row when it takes the focus.
- ADR 0089 Decision 9 and musicindex ADR 0026 R-FOCUS-1: only keyboard input shows the focus.

Operator choice line, 2026-11-16: the operator chose to correct the defects of the walk of sections 15 to 17 before the walk continues. Defects D and E come from the walk of [ADR 0084 task 040](adr-0084-task-040-release-page-draws-only-the-rows-in-view.md).

## Recorded Facts

- 2026-11-16, agent walk on Xvfb, release binary, Index release "GLOBAL FEDPILLED INDUSTRIAL COMPLEX" (491 tracks):
  - Defect D: Tab from "Reverse" gives the focus to the track list and puts the cursor on row 1. The column does not scroll, so the ring is out of view. Down then moves the ring to row 2 and the column scrolls to it.
  - Defect E: from the last row, Home puts the cursor on row 1. The column scrolls row 1 to the top of the viewport, and the pinned list bar covers it.
- 2026-11-16: `sync_detail_keyboard_focus` (`src/library/detail_selection.rs`) calls `DetailSelection::enter_with_keyboard` (`src/view_models/detail_selection.rs`). That call can move the cursor, but no code asks for a reveal after it. `src/app/detail_selection.rs:330` calls `sync_detail_keyboard_focus` in the render.
- 2026-11-16: a key that moves the cursor calls `request_row(index)` for a track row and `request()` for other rows (`src/library/detail_selection.rs`, the `move_focus` match). `DetailPageRows::track_row_index` gives the index.
- 2026-11-16: `ScrollReveal::probe` and `ScrollReveal::probe_row` (`src/ui/composites/music_column_scroll.rs`) call `reveal_offset_y` with the full viewport of the scroll handle. They do not subtract the pinned list bar.
- 2026-11-16: `ListBarPin` (`src/ui/composites/music_column_scroll.rs`) records the top of the in-flow list bar (`rest_top`) at each paint. It does not record the bar height. `list_bar_is_pinned` (`src/view_models/detail_selection.rs:1027`) decides when the pinned copy shows.
- 2026-11-16: `LibraryApp` makes `detail_scroll_reveal` and `list_bar_pin` together (`src/library/app_impl.rs:1248` and `:1249`).

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Row reveal | extend | src/ui/composites/music_column_scroll.rs | `ScrollReveal` gets a top inset from the pin. `probe` and `probe_row` use one shared function. |
| Pinned list bar values | extend | src/ui/composites/music_column_scroll.rs | `ListBarPin` also records the bar height. |
| Reveal offset | extend | src/ui/composites/music_column_scroll.rs | `reveal_offset_y` stays as it is. The caller gives a viewport with its top moved down by the inset. |
| Detail keyboard entry | extend | src/library/detail_selection.rs | `sync_detail_keyboard_focus` asks for a reveal when the entry moves the cursor. |
| Detail entry step | extend | src/view_models/detail_selection.rs | `enter_with_keyboard` returns the key that got the cursor. |

## Required Changes

1. `ListBarPin::probe` records the height of the in-flow bar at each paint, adjacent to `rest_top`. Add `ListBarPin::pinned_inset(&self, offset_y: f32, max_offset_y: f32) -> f32`. It gives the bar height when `list_bar_is_pinned` gives `true` for that offset, and 0 in all other conditions.
2. `ScrollReveal` gets an optional `ListBarPin`. Add a builder `ScrollReveal::below_pin(self, pin: ListBarPin) -> Self`. In `src/library/app_impl.rs`, make `detail_scroll_reveal` with the pin that `list_bar_pin` holds. The two values must use the same cells of the pin. Do not make a second set of cells.
3. Move the body of `probe` and `probe_row` into one private function. It calculates the offset with `reveal_offset_y`. The viewport top is the scroll viewport top plus the inset of the current offset. If the new offset gives a larger inset, calculate again with that inset. Keep the windowed estimate of task 040 as it is. The probe of the shown row does the last correction.
4. `DetailSelection::enter_with_keyboard` returns `Option<DetailRowKey>`: the key that got the cursor, or `None`. Update its callers. `src/view_models/search_results/mod.rs` can ignore the value.
5. In `sync_detail_keyboard_focus`, when the entry gives a key, ask for a reveal. Use `request_row(index)` when `rows.track_row_index(&key)` gives an index. Use `request()` for other rows.
6. Add tests:
   - `reveal_offset_y` with a viewport top moved down by an inset scrolls a row that is below the inset into view.
   - `pinned_inset` gives 0 when the bar is not pinned and the bar height when it is pinned.
   - `enter_with_keyboard` returns the key of the first row for a list with no cursor and `shows` set to `true`. It returns `None` for a list with a cursor.
   - A gpui test in `src/ui/composites/music_column_scroll.rs` with a pinned area of 50 px at the top of the viewport. A reveal of row 0 from a scrolled position puts the top of row 0 at or below the bottom of that area.

## Do Not Change

- No change to `list_bar_is_pinned`, to the pinned bar paint or to the in-flow list bar.
- No change to `focus_reveal_layer`. It shows controls, not rows.
- No change to the windowed section of task 040. Only its call to the shared reveal function can change.
- No change to the keys in the list. ADR 0092 and ADR 0102 own them.
- No change to entry by a pointer click. Only keyboard input shows the focus (R-FOCUS-1).

## Acceptance Criteria

Mechanical:

- A test proves that `pinned_inset` gives the bar height only when the bar is pinned.
- A test proves that a row reveal puts the row below the pinned area.
- A test proves that `enter_with_keyboard` returns the key that got the cursor.
- The commit checks of AGENTS.md pass.

Visual, listed apart:

- V1: on "GLOBAL FEDPILLED INDUSTRIAL COMPLEX", Tab from "Reverse" puts the ring on row 1, and row 1 is in view.
- V2: push End, then Home. Row 1 has the ring and is fully below the pinned list bar.
- V3: from row 5, push Up until row 1. Each row with the ring is fully below the pinned list bar.

## Test Commands

- `cargo test --lib ui::composites::music_column_scroll`
- `cargo test --lib view_models::detail_selection`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`

## Report

Give the files changed and the tests added. Tell how the reveal and the pin use the same cells.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the Index release page (`src/app/search_dispatch.rs`) uses a pin that is not the `list_bar_pin` of `LibraryApp`.
- Stop if the reveal needs a change to `list_bar_is_pinned`.

## Implementation Report

- `ListBarPin` records the bar height at each paint. `pinned_inset` gives the height only when `list_bar_is_pinned` gives `true`.
- `ScrollReveal::below_pin` holds a clone of the pin. `LibraryApp` makes `detail_scroll_reveal` from a clone of the one `list_bar_pin`, so the two share the same cells.
- `probe` and `probe_row` call one function, `reveal_row`. It adds the inset of the current offset to the viewport top. It calculates again when the new offset needs a larger inset.
- `enter_with_keyboard` returns `Option<DetailRowKey>`. `sync_detail_keyboard_focus` calls `request_row` or `request` for that key.
- Mechanical tests added in `music_column_scroll.rs` and `view_models/detail_selection.rs`. The visual checks V1 to V3 are open.

The primary agent made two corrections after the delegate change:

- `probe` and `probe_row` add the margin `Spacing::SM` around the row area, the margin of `focus_reveal_layer`. Without it, the viewport edge cut the line of the ring.
- The pinned copy of the list bar covers the bar height and its top padding. `ListBarPin::pinned_probe` records the height of the pinned copy at each paint. `pinned_inset` uses that height, and the in-flow bar height before the first paint of the pinned copy. Without it, the pinned copy covered the top line of the ring.

## Walk Result - 2026-11-16

The agent walked V1 to V3 on an Xvfb display with the release binary, on the Index release "GLOBAL FEDPILLED INDUSTRIAL COMPLEX" (491 tracks). Section 24 of `docs/pending-human-checks.md` gives the steps.

- V1: Tab from "Reverse" put the ring on row 1, and the column scrolled to show the full ring. Pass.
- V2: End, then Home, put the ring on row 1, fully below the pinned list bar. Pass.
- V3: from row 15, Up until row 1. Row 6, row 2 and row 1 kept the full ring below the pinned list bar. Pass.

The operator confirmation stays open.

