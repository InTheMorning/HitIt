# ADR 0090 Task 005: Each Music Column Shows The Focused Control

Status: Implemented - 2026-11-14. Mechanical checks are Green. The visual checks V1 to V4 are open in section 15 of [pending human checks](../pending-human-checks.md).

## Goal

Each Music column scrolls to show a control that gets the keyboard focus. This includes the destinations list, the Index detail panes, the search notice pages and Settings. After this task, no Music column scrolls without a held `ScrollHandle`.

## Authority

- [ADR 0090](../adr/archive/0090-one-list-bar-above-the-rows.md) Invariants, bullet 3: "A key that moves the focus to a button in a Music column scrolls the column to show that button, with its focus ring."
- [ADR 0084](../adr/archive/0084-music-column-browser.md) Decision 1: each Music column has its own scrollbar.
- [AGENTS.md](../../AGENTS.md) "Known open defects" lists this fault.

## Recorded Facts

- 2026-11-14: `music_column_scrollbar` (`src/ui/composites/music_column_scroll.rs:37`) calls `overflow_y_scrollbar()`. It holds no `ScrollHandle`, so `focus_reveal_layer` does not run.
- 2026-11-14: `music_column_tracked_scrollbar` (`src/ui/composites/music_column_scroll.rs:59`) adds `begin_focus_reveal` before the column and `focus_reveal_layer` after it.
- 2026-11-14: these calls use `music_column_scrollbar`:
  - the destinations list, `src/library/app_impl.rs:5570`
  - the Index detail panes, the `None` branch of `src/ui/composites/release_detail_surface.rs:185`
  - the search notice pages, `src/ui/shells/search_results_inspector.rs:343` and `:663`
- 2026-11-14: Settings (`src/ui/composites/settings.rs:105` to `:130`) calls `track_scroll(scroll_handle)` and `vertical_scrollbar`. It has a handle, but it has no reveal layer.
- 2026-11-14: `tests/architecture_tests.rs` names `music_column_scrollbar(` at lines 2184, 5295, 21513 and 22702.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Tracked Music column scroll | reuse | src/ui/composites/music_column_scroll.rs | `music_column_tracked_scrollbar` has the focus reveal layer. |
| Focus reveal layer | reuse | src/ui/composites/music_column_scroll.rs | `focus_reveal_layer` and `reveal_offset_y` scroll by the smallest distance. |
| Held scroll handles of the shell | extend | src/library/app_impl.rs | Add a held handle for each column that has none. Follow the handles that `LibraryApp` holds. |

## Required Changes

1. Give the destinations list a held `ScrollHandle`, and call `music_column_tracked_scrollbar` with it.
2. Give the Index detail panes a held handle. Remove the `None` branch in `src/ui/composites/release_detail_surface.rs`. If the function then always gets a handle, change its parameter from `Option` to a reference.
3. Give each search notice page a held handle, and call `music_column_tracked_scrollbar`. Two pages must not share one scroll position (task 014 Recorded Facts, cited at `src/ui/composites/music_column_scroll.rs:35`).
4. Move Settings to `music_column_tracked_scrollbar` with its existing handle. Keep its padding and its gutter.
5. Remove `music_column_scrollbar`, because no call remains.
6. Change the architecture tests that name `music_column_scrollbar(`. Each Music column must call `music_column_tracked_scrollbar`, or the virtual list of task 037. A failure message names ADR 0090 Invariants and tells the author to call `music_column_tracked_scrollbar` with a held handle.

## Do Not Change

- No change to `focus_reveal_layer` or to `reveal_offset_y`.
- No change to the pinned list bar or to `ListBarPin`.
- No change to the Settings rows or to the Settings group menu. ADR 0093 task 001 owns them.
- No scroll position that stays across a navigation change for a column that does not keep one now. A new handle starts at the top on each new page.

## Acceptance Criteria

Mechanical:

- No source file outside `tests/` contains `music_column_scrollbar(`.
- The changed architecture tests pass, and each failure message names ADR 0090 Invariants and the fix.
- The unit test `adr_0084_detail_selection_r18_09_reveal_scrolls_the_smallest_distance` passes without a change.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: in a short window, Tab through the destinations list. Each focused entry is in view with its focus ring.
- V2: on an Index detail pane, Tab to a button below the visible part. The pane scrolls to show it.
- V3: on a search notice page, Tab to the last button. The page scrolls to show it.
- V4: in Settings, Tab to a control below the visible part. The page scrolls to show it.

## Operator Visual Check

1. Run `cargo run --bin hitit` in a desktop session.
2. Make the window 600 px high.
3. Do V1 to V4 in sequence.
4. Wrong: a focused control is out of view, or its focus ring is cut at the edge of the column.

## Test Commands

- `cargo test --lib ui::composites::music_column_scroll`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Implementation Notes - 2026-11-14

- The destinations list holds `LibraryApp::destinations_scroll`.
- The Index detail panes use the held detail column handle, `LibraryApp::detail_scroll_handle`. It already gets the reset on a navigation.
- The search notice pages hold `search_notice_scroll` and `saved_search_notice_scroll` on `TopApp`. The notices of one search page share one handle. A notice is short, so a stale offset is improbable. No reset on a change of notice exists.
- `ReleaseDetailSurface` takes a `&ScrollHandle`, not an `Option`. Its only scrollable caller, `src/ui/shells/entity.rs`, gives the held handle.

## Report

Give the files changed, the new held handles and the tests changed. Tell if `release_detail_surface` still takes an `Option`.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a column must keep its scroll position across a navigation change and no held handle exists for that state.
- Stop if a column is a horizontal scroll area.
