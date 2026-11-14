# ADR 0092 Task 001: Artists Takes The List Focus

Status: Ready - 2026-11-14. The operator set the sequence on 2026-11-14: this task first, then one task for Public playlists, Live and Search. Implementation has not started.

## Goal

The Artists list is one Tab stop in its Rows view and in its Tiles view, as Releases is. Up and Down move the focused artist. Enter opens it. After "Show more", the focus goes to the first new row of that group.

## Authority

- [ADR 0092](../adr/0092-the-keys-of-a-list-row.md) Decision 1 and Status: a list with no list focus has the keys of R-LISTKEY-1 when it gets a list focus. musicindex ADR 0061 R-LISTKEY-1 owns the keys.
- [ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decision 9: a list is a Tab stop, and keyboard entry marks a row.
- [ADR 0085](../adr/0085-artists-by-confirmation.md) amendment of 2026-11-08: the Artists list has three groups, and each group has a "Show more" button (musicindex ADR 0057 R-SITE-15).
- [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) Invariants: keyboard reach.

## Recorded Facts

- 2026-11-14: the list column gives `has_row_focus` only to Releases, the Inbox and My playlists (`src/library/app_impl.rs:5922` to `:5928`). Artists gets `(false, 0)`, so `list_is_tab_stop` is false.
- 2026-11-14: `render_artists_tile` (`src/ui/shells/library/releases_tiles.rs:283`) sets `on_click` only. It does not call `SelectableTile::focused`.
- 2026-11-14: the Releases tiles call `.focused(row.selection.focused)` (`src/ui/shells/library/releases_tiles.rs:163` and `:250`).
- 2026-11-14: `show_more_artist_group` (`src/library/music_scopes.rs:496`) has this comment: "The Artists list has no list focus, so the focus does not move to the first new row (musicindex ADR 0057 R-SITE-15 item 4)."
- 2026-11-14 (inference, not tested): after "Show more", a group can remove its status row (`src/view_models/artist_groups.rs:446` to `:463`). The focused button then leaves the tree, and the focus goes to the sidebar.
- 2026-11-14: `reveal_artists_tile` (`src/library/app_impl.rs:4572`) scrolls a tile into view.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| List Tab stop rule | reuse | src/view_models/list_selection.rs | `list_is_tab_stop` and `keyboard_entry_step` give the stop and the entry mark. |
| List column focus | extend | src/library/app_impl.rs | Add the Artists arm to the `has_row_focus` match and to `mark_row_for_keyboard_entry`. |
| Tile focus outline | reuse | src/ui/composites/selectable_tile.rs | `SelectableTile::focused` draws the focus outline. |
| Artists rows and groups | extend | src/view_models/artist_groups.rs | Add the focused row and the row sequence across the three groups. |
| Show more handler | extend | src/library/music_scopes.rs | Move the focus to the first new row after the rows load. |
| Tile reveal | reuse | src/library/app_impl.rs | `reveal_artists_tile` scrolls a tile into view. |

## Required Changes

1. Give the Artists view model a focused row. The sequence of rows goes through the three groups in their display sequence. Group headings and "Show more" buttons are not rows.
2. Add Artists to the `has_row_focus` match with its row count, in each scope.
3. Give Up, Down, Home, End and Enter to the Artists list. Use the key path of Releases. Do not write a second copy of it.
4. Show the focused row in the Rows view and in the Tiles view. In the Tiles view, call `SelectableTile::focused`, and reveal the tile with `reveal_artists_tile`.
5. After "Show more", put the focus on the first new row of that group, and show it. If the next page loads after the press, move the focus when its rows arrive. If no new row arrives, keep the focus on the list.
6. Remove the comment at `src/library/music_scopes.rs:496` that says the list has no list focus.
7. Change the ADR 0092 Status text in the same change: Artists has the list focus. Public playlists, Live and Search continue to differ.
8. In the same change, remove the ADR 0085 Status sentence that says the focus does not move to the first new row.

## Do Not Change

- No change to Public playlists, Live or Search. The next task owns them.
- No new key that R-LISTKEY-1 or ADR 0092 does not give. In the Tiles view, Up and Down move through the same sequence as the Rows view.
- No change to the three groups, their headings or their page sizes.
- No change to the row menu. ADR 0092 Decision 2 applies only when an Artists row has a row menu.

## Acceptance Criteria

Mechanical:

- The view model gives a focused row for Artists.
- A test proves that Down moves to the first row of the next group from the last row of a group.
- A test proves that `has_row_focus` is true for Artists with one or more rows, in each scope.
- A test proves that "Show more" sets the focused row to the first new row of its group, also when the page loads after the press.
- A test proves that "Show more" with no new row keeps a focused row in the list.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: on Artists in the Rows view, Tab reaches the list in one stop. Up and Down move the focus ring. Enter opens the artist.
- V2: the same in the Tiles view. The focused tile shows the focus outline and is in view.
- V3: in the scope All, press "Show more" with the keyboard. The focus goes to the first new row of that group, not to the sidebar.

## Operator Visual Check

1. Run `cargo run --bin hitit` in a desktop session.
2. Open Artists in the scope All.
3. Do V1, V2 and V3 in sequence.
4. Wrong: Tab skips the list, the focus ring does not show, or the focus goes to the sidebar.

## Test Commands

- `cargo test --lib view_models::artist_groups`
- `cargo test --lib view_models::list_selection`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests added. Tell if the loss of the focus after "Show more" had the cause in Recorded Facts.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the key path of Releases cannot take a second list without a copy. Report the shape of the change that it needs.
- Stop if an Artists row has a row menu. Decision 2 then also applies, and the scope grows.
