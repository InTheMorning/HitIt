# ADR 0092 Task 003: Keyboard Entry Puts The Cursor On The Marked Row

Status: Implemented - operator visual checks open, 2026-11-16.

## Goal

When a list with a Back mark gets the keyboard focus, the cursor goes to the marked row. The list scrolls to show that row. Today the list shows no cursor and does not scroll. The first Down goes to a row that the operator cannot predict.

## Authority

- [ADR 0092](../adr/0092-the-keys-of-a-list-row.md) Decision 1: a list with rows is one Tab stop with the keys of musicindex ADR 0061 R-LISTKEY-1.
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Invariants: the list shows the cursor row when it takes the focus.
- ADR 0089 Decision 9 and musicindex ADR 0026 R-FOCUS-1: only keyboard input shows the focus.
- [ADR 0092 task 001](adr-0092-task-001-artists-list-focus.md) Walk Correction of 2026-11-15, V2.

Operator choice line, 2026-11-16: keyboard entry into a list with a Back mark and no cursor puts the cursor on the marked row. The list shows that row. The Back mark stays as it is.

## Recorded Facts

- 2026-11-16, agent walk on Xvfb, release binary, check 17 V2: on Artists in the Tiles view, open an artist, then push Back. Tab into the grid. No tile shows the focus outline and the grid does not scroll. The first Down puts the cursor on "Crash Landing". The same fault occurs in the Rows view.
- 2026-11-16: `keyboard_entry_step` (`src/view_models/list_selection.rs:461`) gives `FocusStep::Home` only if no row has the cursor and no row has a Back mark. With a Back mark it gives `None`.
- 2026-11-16: `mark_row_for_keyboard_entry` (`src/library/app_impl.rs:4648`) returns when `keyboard_entry_step` gives `None`. `reveal_focused_row` (`src/library/app_impl.rs:4698`) returns when no row has the cursor. Thus a list with a Back mark and no cursor gets no cursor and no scroll.
- 2026-11-16: `marked_entry` is a `FrameNavigationEntry` (`src/view_models/workspace/nav.rs:70`). `src/app.rs:1925` sets it through `set_marked_entry` (`src/library/saved_searches.rs:354`).
- 2026-11-16: the tests `adr_0098_list_entry_scrolls_to_the_cursor_row` and `r_focus_1_a_page_header_control_hides_the_row_ring` do not include a list with a Back mark.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Keyboard entry step | extend | src/view_models/list_selection.rs | `keyboard_entry_step` decides the entry move. Extend it so that a Back mark with no cursor gives a move to the marked row. |
| Keyboard entry of a list | extend | src/library/app_impl.rs | `mark_row_for_keyboard_entry` applies the step for Releases, the Inbox, My playlists and Artists. |
| Cursor row reveal | reuse | src/library/app_impl.rs | `reveal_focused_row` and `reveal_artists_tile` show the cursor row or tile. |
| Back mark match | reuse | src/view_models/library.rs | Each `with_marked` finds the row that a `FrameNavigationEntry` marks. Find the marked row with the same match. Do not write a second copy. |

## Required Changes

1. Change `keyboard_entry_step`, or add a typed result adjacent to it. For a list with the focus ring, no cursor and a Back mark, the result is "put the cursor on the marked row". Keep `Home` for a list with no cursor and no mark. Keep `None` for a list with a cursor.
2. In `mark_row_for_keyboard_entry`, apply the new result for Releases, the Inbox, My playlists and Artists. Find the marked row with the match of `with_marked` for that list. Set the cursor on that row and show it, in the Rows view and in the Tiles view.
3. If the scope hides the marked row, use `Home`.
4. Update the test `adr_0089_keyboard_entry_marks_the_first_row_only_for_the_keyboard` for the new result.
5. Add a test for Artists and a test for Releases. With a Back mark and no cursor, keyboard entry puts the cursor on the marked row. It also asks for a scroll to that row or tile.

## Do Not Change

- No change to the Back mark. `set_marked_entry` and the row mark paint stay as they are.
- No change to entry by a pointer click. Only keyboard input shows the focus (R-FOCUS-1).
- No change to the arrow keys. ADR 0092 and ADR 0102 own them.
- No change to the Tab sequence. ADR 0092 task 002 owns the detail column.

## Acceptance Criteria

Mechanical:

- A unit test proves that `keyboard_entry_step` (or its new result) gives the marked row for a list with the focus ring, no cursor and a Back mark.
- A unit test proves that the result stays `Home` with no cursor and no mark, and stays `None` with a cursor.
- A test proves that keyboard entry into Artists with a Back mark and no cursor sets the cursor on the marked artist. The test also proves that it asks for a scroll, in the Rows view and in the Tiles view.
- A test proves the same for Releases.
- The commit checks of AGENTS.md pass.

Visual, listed apart:

- V1: on Artists in the Tiles view, open an artist and push Back. Tab into the grid. The tile of that artist shows the focus outline and is in view.
- V2: the same in the Rows view, and on Releases.

## Test Commands

- `cargo test --lib view_models::list_selection`
- `cargo test --lib library`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests added. Tell how each list finds its marked row.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if no code finds the row of a list from a `FrameNavigationEntry`.
- Stop if the change makes a pointer click show the focus ring.

## Walk Result - 2026-11-16

The agent walked V1 and V2 on an Xvfb display with the release binary. Section 22 of `docs/pending-human-checks.md` gives the steps.

- Releases, Rows view: entry scrolled to the marked row "The Dark Plains - 00001" and showed the ring. Pass.
- Artists, Rows view: entry scrolled to "2140" and showed the ring. Pass.
- Artists, Tiles view: entry scrolled to "888childish" and showed the outline. Pass.

The operator confirmation stays open.
