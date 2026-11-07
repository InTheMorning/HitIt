# ADR 0090 Task 002: The List Bar Stays In View

## Status

Implemented - 2026-11-06. Visual checks are open.

This task is packet 002 of the [ADR 0090 phase plan](../plans/adr-0090-list-bar-phase-plan.md).
It implements Decision 5 of [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) on the
two group pages of packet 001.

## Goal

The list bar of the singles group page and the window group page stays in
view while the rows scroll. A curator can act on a selection on a long page
without a scroll back to the top.

## Scope

- Only the singles group page and the window group page change.
- No scroll refactor. The detail column keeps its scroll handle and its
  scrolling container.
- No sort or view control is added.

## Method

GPUI has no sticky style. The bar keeps its place in the rows. A second copy
of the same bar draws as an overlay at the top of the viewport. The copy shows
only while the in-flow bar is above the viewport.

- The in-flow bar carries an empty probe. Each paint, the probe records where
  the bar sits, relative to the viewport top at zero offset.
- A render reads the current scroll offset and that position. The view model
  rule decides "pinned". A scroll shows or hides the copy in the same frame.
- The probe asks for one more redraw only when the pinned state differs from
  the state that the last render drew.
- Both copies come from `render_list_bar`, with the same view model and the
  same handler. The pinned copy has its own element ids.

## Files Changed

- `src/view_models/detail_selection.rs`: `list_bar_is_pinned`, and its tests.
- `src/ui/composites/music_column_scroll.rs`: `ListBarPin`, the probe and the
  pinned state.
- `src/ui/composites/selection_action_bar.rs`: `ListBarCopy`. `render_list_bar`
  takes the copy. The pinned copy has `pinned-` ids and no tab stop.
- `src/ui/primitives/button.rs`: `Button::out_of_tab_sequence`, an opt-out from
  the Tab sequence. The default is unchanged.
- `src/ui/composites/playlist_popover.rs`: `trigger_out_of_tab_sequence`, for
  the "Add to playlist" trigger of the pinned copy.
- `src/ui/composites/mod.rs`: re-export of `ListBarCopy`.
- `src/ui/shells/library/singles_group.rs`, `window_group.rs`: the probe, the
  pinned copy and the overlay wrapper.
- `src/ui/shells/library/detail.rs`: passes `ListBarPin` to the two pages.
- `src/library.rs`, `src/library/app_impl.rs`: the `ListBarPin` field of
  `LibraryApp`.
- `tests/architecture_tests.rs`: guard
  `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer`, and the call
  string of `adr_0090_group_pages_have_no_frame_header_action_or_bottom_bar`.

## Mechanical Criteria

Each criterion is checked by a unit test or a guard.

- The pinned state is false when the bar is in view, and true once its top is
  above the viewport. The boundary is not pinned. Test:
  `adr_0090_list_bar_is_pinned_only_after_its_top_scrolls_above_the_view`.
- A page that does not scroll is never pinned. Test:
  `adr_0090_list_bar_is_not_pinned_on_a_page_that_does_not_scroll`.
- A bar with no recorded position is not pinned. Test:
  `adr_0090_list_bar_without_a_position_is_not_pinned`.
- Both group pages draw the pinned copy and the in-flow bar through
  `render_list_bar`, and no second bar renderer exists. Guard:
  `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer`.
- The group pages keep no frame header action and no bottom bar. Guard:
  `adr_0090_group_pages_have_no_frame_header_action_or_bottom_bar`.

The operator run of 2026-11-06 found that a click on an empty part of the pinned copy reached a hidden row and changed the selection. The pinned copy now blocks that click, and the guard `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer` holds this.

## Keyboard

- Only the in-flow copy takes part in the Tab sequence. The "Select" or
  "Done" button of the in-flow copy is the one tab stop. The pinned copy
  adds no tab stop.
- The ids of the pinned copy start with `pinned-`. The ids of the in-flow
  copy do not change.
- Ctrl+A, Space, Escape and the row marks do not change.
- Focus: a focus on the in-flow button stays there when the copy shows. A key
  that moves the focus to the button scrolls the column to show it (ADR 0090
  Invariants). The buttons of the pinned copy cannot take focus.
- A click on a pinned control runs the same command as the in-flow control.
  A click on "Select" or "Done" moves the focus to the rows, as before.
- "Add to playlist" in the pinned copy opens its own popover. The popover has
  its own keyed state, so it does not open the in-flow popover. The popover
  closes when the pinned copy hides.

## Redraw

- A scroll wheel event already redraws the view that owns the scroll element,
  as before. This task adds no redraw to a scroll.
- The pinned copy adds one redraw only when the pinned state changes between
  two paints. The redraw is a notify of `LibraryApp`.

## Visual Criteria

These criteria need a person in a desktop session. They are open.

- Scroll a long singles group page. The bar stays at the top of the rows.
- The pinned bar has the same look as the in-flow bar. It has an opaque
  background, so the rows do not show through its text.
- The rows do not jump when the pinned bar shows or hides.
- In selection mode, the pinned bar shows the selection background.
- "Add to playlist" in the pinned bar opens its popover, and the popover
  is readable and closes.
- A click on "Select", a row action and "Done" in the pinned bar works.
- Scroll back to the top. The pinned bar hides, and the in-flow bar shows.
- Scroll a window group page, and check the same points.

## Known Limits

- The pinned copy uses the same right inset as the rows, so the scrollbar
  stays clear. The top inset is the page padding. An operator check decides
  whether the inset looks right.
- The position of the bar is recorded at paint. On a navigation to another
  group page, the first frame may use the position of the previous page. The
  next paint corrects it. The operator check covers this.

## Gates

- `touch src/lib.rs src/main.rs tests/architecture_tests.rs`, `cargo fmt`:
  format check passes.
- `cargo clippy --all-targets -- -D warnings`: Green.
- `cargo test --no-fail-fast`: 2867 unit tests passed, 407 architecture tests
  passed, and 0 failed.
- `cargo test --test architecture_tests`: 407 passed, 0 failed.
- `cargo build --bin v4vmm`: finished.
- The app was not run. The visual list above is open.
