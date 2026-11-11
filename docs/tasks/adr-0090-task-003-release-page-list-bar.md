# ADR 0090 Task 003: Release Page List Bar

## Status

Implemented - 2026-11-07. Visual checks are open.

This task is packet 003 of the [ADR 0090 phase plan](../plans/adr-0090-list-bar-phase-plan.md).
It implements Decision 1 of [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) on the
release page. The release page has two forms: the Library form and the Index form. Both
forms use one list bar renderer. The two forms cannot drift apart.

## Goal

On the release page, show one list bar above the track rows. Remove the band above the
title and the bottom action bar. The bar stays in view during a scroll.

## Scope

- Only the release page changes. Its Library form and its Index form change.
- The feed page, the name level page, the playlist page, the search pages and the list
  pages keep their band above the title. They also keep their bottom bar. Their packets
  move them.
- The bar uses the parts of packets 001 and 002: `has_list_bar`, `ListBarVm`,
  `ListBarCommand`, `render_list_bar` and the pinned copy. The helper
  `render_group_list_bar_copies` moved to `render_list_bar_copies` in
  `src/ui/composites/selection_action_bar.rs`, because its name was false for a release
  page.
- No sort or view control is added.

## Position Of The Bar

The bar sits after the action row and before the track section. The track section
has a heading ("Tracks" and the count). The bar sits above that heading, so the heading
stays with its rows. The release page has no heading rule. The release details and the
panels keep their place, so the other sections do not move.

musicindex ADR 0028 R-PAGE-1 puts the Tracks section after the header and the notices.
The bar is not a section of that rule, so the sequence of R-PAGE-1 is kept, and no
change to musicindex is needed.

## Files Changed

- `src/view_models/detail_selection.rs`: `has_list_bar` gives `true` for the release
  page, and its tests.
- `src/library/app_impl.rs`: builds the list bar of a Library release page.
- `src/library/detail_selection.rs`: `list_bar_pin`, the accessor of the pinned state
  for the top app.
- `src/ui/composites/selection_action_bar.rs`: `render_list_bar_copies`, the in-flow
  copy and the pinned copy. The pinned copy blocks clicks with `.occlude()`.
- `src/ui/composites/mod.rs`: re-export of `render_list_bar_copies`.
- `src/ui/composites/release_detail_surface.rs`: the list bar slot, before the track
  section, and the pinned copy over the scroll container.
- `src/ui/shells/entity.rs`: passes the list bar slots to the surface.
- `src/ui/shells/release_page.rs`: `ReleasePageImages` gets the list bar and the pinned
  copy.
- `src/ui/shells/library/feed_detail.rs`: the Library release page. It has
  `render_library_list_bar_copies` and the handler that emits `LibraryAppEvent::ListBar`.
- `src/ui/shells/library/detail.rs`: passes the list bar to the Library release page.
- `src/ui/shells/library/singles_group.rs`, `window_group.rs`: call
  `render_library_list_bar_copies`. Their own copy helpers moved to the composite.
- `src/app/detail_selection.rs`: `top_list_bar_handler`. A command of the Index release
  page runs through `run_list_bar_command`, the same path as the Library form.
- `src/app/search_dispatch.rs`: the Index release page builds its list bar and its
  pinned copy.
- `tests/architecture_tests.rs`: guards `adr_0090_list_bar_pages_have_no_frame_header_action_or_bottom_bar`
  and `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer`.

## Mechanical Criteria

Each criterion is checked by a unit test or a guard.

- The release page has the list bar, and the other pages do not. Test:
  `adr_0090_the_group_and_release_pages_have_the_list_bar`.
- A release with no selectable row has no list bar and no "Select". A release with one
  selectable track has "Select" with the label "Select tracks". Test:
  `adr_0090_release_page_list_bar_needs_a_selectable_row`.
- The list bar of a release page gives "Download", "Add to playlist" and "Remove" while
  selecting. Test: `adr_0090_release_page_list_bar_gives_the_track_actions`.
- The group pages and the release page have no frame header action and no bottom bar.
  Guard: `adr_0090_list_bar_pages_have_no_frame_header_action_or_bottom_bar`.
- The list bar sits after the action row and before the track section. Guard: the same
  guard, with the order of `ReleaseDetailSurface`.
- Each list bar page draws its list bar through `render_list_bar_copies`. No shell calls
  `render_list_bar` and names no second root id. Guard:
  `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer`.
- The pinned copy blocks a click on the rows that it hides (`.occlude()`). Its
  "Add to playlist" trigger stays out of the Tab sequence. Same guard.
- The release page draws the pinned copy over its scroll container. Same guard.

## Visual Criteria

These criteria need a person in a desktop session. They are open.

- Open a Library release with tracks. The bar is directly above the "Tracks" heading.
  The heading and the rows are together. The page has no band above its title and no
  bottom bar.
- Open an Index release with tracks. The same points apply.
- "Select", the actions and "Done" need short mouse travel.
- "Add to playlist" in the bar opens its popover. The same action in the pinned copy opens
  its popover. Each popover is readable and closes.
- Scroll a long release page. The bar stays at the top of the rows. Scroll back to the
  top. The pinned bar hides, and the in-flow bar shows.
- In a narrow window, the bar wraps. Its two ends stay on two lines.
- The description and the release details keep their places.

## Keyboard

- The "Select" or "Done" button of the in-flow bar is the one tab stop of the bar. The
  pinned copy adds no tab stop.
- Tab, Ctrl+A, Space, Escape and the row marks keep the same paths. `sync_page_keyboard_focus`
  and `detail_page_rows` keep their owners.
- The code check reads these paths. A person checks them in a desktop session. This check
  is open.

## Changed Old Tests And Guards

- Guard `adr_0090_group_pages_have_no_frame_header_action_or_bottom_bar` is renamed to
  `adr_0090_list_bar_pages_have_no_frame_header_action_or_bottom_bar`. Its name was false
  for the release page. It now checks the release page too.
- Test `adr_0090_only_the_group_pages_have_the_list_bar` is renamed to
  `adr_0090_the_group_and_release_pages_have_the_list_bar`. Its check of the release page
  changed from `false` to `true`.
- Guard `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer` checks the shared
  helper in `selection_action_bar.rs`. It no longer checks `render_group_pinned_list_bar`
  in `singles_group.rs`, which moved.
- No test asserted the frame header action of the release page by name. The view model
  test of `detail_header_action` checks the shared function, which is still used. It
  stays unchanged.

## Gates

- `touch src/lib.rs src/main.rs tests/architecture_tests.rs`, `cargo fmt`: run. The
  format check passes.
- `cargo clippy --all-targets -- -D warnings`: Green.
- `cargo test --no-fail-fast`: 2869 unit tests passed, 407 architecture tests passed,
  and 0 failed. The first run had one failure. A guard needle had a space that the
  compact source does not keep. The needle is fixed, and the guard passes.
- `cargo test --test architecture_tests`: 407 passed, 0 failed.
- `touch src/main.rs src/lib.rs && cargo build --bin hitit`: finished.
- The app was not run. The visual list above is open.
- The shared STE linter reports 40 findings, the same count as the finished task 002.
  The findings are dictionary and heuristic checks. The sentence length of the position
  section is corrected.
