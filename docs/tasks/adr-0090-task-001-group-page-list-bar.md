# ADR 0090 Task 001: Group Page List Bar

## Status

Complete - 2026-11-06. The operator walked it on 2026-11-06.

This task is the first page of [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md).
The first page is the singles group page and the window group page.

## Goal

On the singles group page and the window group page, show one list bar directly
above the rows. Remove the band above the title and the bottom action bar.

## Scope

- Only the singles group page and the window group page change.
- The other pages keep their band above the title and their bottom bar.
- No sort or view control is added to a group page.

## Files Changed

- `src/view_models/selection_action_bar.rs`: `ListBarCommand`, the commands of
  the list bar.
- `src/view_models/detail_selection.rs`: `ListBarVm`, `has_list_bar`, and their
  unit tests.
- `src/ui/composites/selection_action_bar.rs`: `render_list_bar`,
  `list_bar_playlists` and `ListBarHandler`.
- `src/ui/composites/mod.rs`: re-exports of the list bar items.
- `src/library.rs`: `LibraryAppEvent::ListBar`.
- `src/library/app_impl.rs`: builds the list bar of a group page from the view
  model.
- `src/ui/shells/library/detail.rs`, `singles_group.rs`, `window_group.rs`: the
  bar sits after the page action row and before the rows.
- `src/app.rs`: the subscription that runs `LibraryAppEvent::ListBar`.
- `src/app/detail_selection.rs`: a group page has no frame header action and no
  bottom bar. `run_list_bar_command` sends each command to the existing path.
- `tests/architecture_tests.rs`: guard
  `adr_0090_group_pages_have_no_frame_header_action_or_bottom_bar`.

## Mechanical Criteria

Each criterion is checked by a unit test or a guard.

- The view model gives "Select" with the accessibility label "Select singles"
  in `Normal`. It gives "Done" with "Done selecting singles" in `Selecting`.
  Test: `adr_0090_list_bar_gives_select_done_and_the_selection_actions`.
- The view model gives the selection actions only in `Selecting`. It omits
  "Select all" when each row is selected, and it omits "Remove" when the rows
  have no Library facts. Same test.
- A group page with no selectable row has no list bar. Same test.
- Only the singles group page and the window group page have the list bar.
  Test: `adr_0090_only_the_group_pages_have_the_list_bar`.
- A group page has no frame header action, and its bottom bar is not drawn.
  Guard: `adr_0090_group_pages_have_no_frame_header_action_or_bottom_bar`.

## Visual Criteria

These criteria need a person in a desktop session. They are open.

- The bar is directly above the rows. The page has no band above its title.
- "Select", the actions and "Done" need short mouse travel.
- The mode change is clear. The selection background shows only in `Selecting`.
- In a narrow window, the bar wraps. Its two ends stay on two lines.
- The bar stays in view while the rows scroll. This is open. See
  [Scroll Result](#scroll-result).

## Scroll Result

Not done. The bar scrolls with the rows.

The page renders in `LibraryApp`, in its scroll container. The frame and
the bar commands are in `TopApp`. A pinned copy needs one of these options:

- `LibraryApp` keeps the scroll offset and the bar position in its own state.
  It draws the pinned copy in its own detail pane. `TopApp` does not redraw.
- A scroll listener on the detail container. It tells `LibraryApp` when the bar
  leaves the viewport.

Neither option is a small change. The operator decides which option to take. This
is open under ADR 0090 Decision 5.

## Wrap

The bar uses a wrapping row. In a narrow window, each control wraps to the next
line. The right end is empty on these pages, so no control sits next to a
destructive action.

## Keyboard

No key path changed. `detail_page_rows`, `sync_page_keyboard_focus`, and the
Ctrl+A, Space and Escape paths read the same rows as before. The "Select" or
"Done" button is in the detail column, before the first row. Tab reaches it
first. A click runs `toggle_detail_selection_mode`, which moves the focus to
the rows. This check reads the code. It was not run in a desktop session. It is
open in the visual list above.

## Gates

- `cargo fmt`: run.
- `cargo clippy --all-targets -- -D warnings`: blocked. It stops at one lint in
  `src/broadcast/encoder.rs:341` (`doc_markdown`). That file belongs to another
  session, so this task does not edit it. Clippy with pedantic lints on, and
  without that one lint, gives no warning in the files this task touches.
- `cargo test --no-fail-fast`: 2864 unit tests passed, 406 architecture tests
  passed, and 0 failed.
- `cargo test --test architecture_tests adr_0090`: 1 test passed.
- `cargo build --bin hitit`: finished.
