# ADR 0092 Task 002: The Detail List Follows The Page Controls

Status: Implemented - operator visual checks open, 2026-11-16.

## Goal

On a detail page with a track list, Tab reaches the page controls first and the track list after them. Today the track list is the first Tab stop of the page, before the first name link.

## Authority

- [ADR 0092](../adr/0092-the-keys-of-a-list-row.md) Decision 1: a list with rows is one Tab stop with the keys of musicindex ADR 0061 R-LISTKEY-1.
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Invariants: each control of the list bar keeps its keyboard reach.
- [ADR 0092 task 001](adr-0092-task-001-artists-list-focus.md) Walk Correction of 2026-11-15: the same fault on the Artists list, and its correction with `tab_index(1)`.

## Recorded Facts

- 2026-11-16, agent walk on Xvfb: on the Index release page "Waltz Tunes Volume 2", Tab goes from the name link "minimalizman" to "Download album", "⋯", "Website", "RSS", "Select", "Reverse", and then to the sidebar. Shift+Tab from the name link puts the focus on the track list. The operator reported the fault: "i can't reach the tracks with tab".
- 2026-11-16: `detail_column_focus` is a plain `cx.focus_handle()` with Tab index 0 (`src/library/app_impl.rs:1240`).
- 2026-11-16: `render_detail_column_frame` makes the detail column div the Tab stop of the list (`src/app/detail_selection.rs:361` to `:365`). The div holds the page header, the list bar and the rows.
- 2026-11-16: GPUI `gpui-pre` 0.3.1 at `gpui-kit` revision `9e2392b` sorts Tab stops by the Tab index path, then by the sequence of insertion (`patches/gpui-pre/src/tab_stop.rs:52`). A div inserts its own handle before its children (`patches/gpui-pre/src/elements/div.rs:2495`).
- 2026-11-16: commit `3ab3edb` added `Button::tab_index` (`src/ui/primitives/button.rs:224`). The test `adr_0092_a_button_in_the_list_follows_the_list` proves the sequence for a list with index 1.
- 2026-11-16: the controls in the detail column use index 0. These are the name links, the release buttons, the release menu, "Website", "RSS", "Select" and "Reverse".

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Detail column focus handle | extend | src/library/app_impl.rs | Give the handle Tab index 1, as `list_column_focus` has. |
| Button Tab index | reuse | src/ui/primitives/button.rs | `Button::tab_index` sets the index of a control. |
| Detail column frame | reuse | src/app/detail_selection.rs | `render_detail_column_frame` keeps the Tab stop rule. |
| Tab sequence test pattern | reuse | src/ui/primitives/button.rs | `adr_0092_a_button_in_the_list_follows_the_list` is the pattern for the new test. |

## Required Changes

1. Give `detail_column_focus` Tab index 1.
2. Find each control that a detail page paints after its rows in the detail column. An example is the contributors panel of a release page (`after_section`). Give each of these controls Tab index 1, so Tab reaches it after the list. Give each control in the report.
3. Add a test that proves the sequence on a release page with rows: the last page control before the rows, then the track list, then a control after the rows if one exists.
4. Examine the other detail pages that call `render_detail_column_frame` with rows: the playlist page, the singles group page and the artist page. Tell in the report if each one has the same sequence after the change.

## Do Not Change

- No change to the keys in the list. ADR 0092 and ADR 0102 own them.
- No change to a row menu. It stays out of the Tab sequence (ADR 0098 Invariants).
- No change to the Releases, Inbox, My playlists or Artists list columns.
- No change to the controls of a pinned list bar. They stay out of the Tab sequence.

## Acceptance Criteria

Mechanical:

- `detail_column_focus` has Tab index 1.
- A test proves that `focus_next` from the last page control before the rows of a release page gives the focus to the track list.
- A test proves that `focus_prev` from the first page control does not give the focus to the track list.
- The commit checks of AGENTS.md pass.

Visual, listed apart:

- V1: on an Index release page, Tab from "Reverse" puts the focus ring on the first track row. Down moves it.
- V2: on a Library release page, the same.

## Test Commands

- `cargo test --lib ui::primitives::button`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed, the tests added and each control that got Tab index 1. Tell the result of Required Change 4 for each page.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a control in the detail column must come before the list but GPUI paints it after the rows.
- Stop if the change moves the Tab sequence of a page that has no rows.

## Implementation Report - 2026-11-16

The page-column focus handle has Tab index 1. The release contributor Website and Nostr buttons also have Tab index 1.

The new test checks that Shift+Tab from the first page control does not focus the list. It also checks that Tab from the last page control focuses the list, then reaches the post-row control.

The playlist, singles group and artist pages use the same detail frame. Their page controls come before the list, and the list follows them. These pages have no controls after their rows.

The visual checks V1 and V2 remain open. A person must examine the Index and Library release pages with the commands in section 21 of `docs/pending-human-checks.md`.

### Operator visual check

1. Compile HitIt with `cargo build --bin hitit`.
2. Start HitIt with `target/debug/hitit` in a Linux desktop session.
3. Open an Index release page with tracks.
4. Push Tab until “Reverse” has focus. Push Tab again. Make sure the first track row has the focus ring.
5. Push Down. Make sure keyboard focus moves in the track list.
6. Repeat steps 3 to 5 on a Library release page with tracks.
7. Close HitIt. Remove scratch folders that you created for this check.

The check fails if Tab skips the first track row, or Down does not move keyboard focus in the list.
