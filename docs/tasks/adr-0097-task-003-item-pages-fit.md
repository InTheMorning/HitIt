# ADR 0097 Task 003: The Item Pages Fit At The Two Widths

Status: Ready - 2026-11-14. It waits for [task 002](adr-0097-task-002-list-pages-fit.md), which makes the fixture adapter. This is phase 3 of the [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md).

## Goal

A GPUI layout test draws each item page with long fixture data at 390 px and at 940 px. The fit probes find no fault of musicindex ADR 0065 R-FIT-3. The pages are the release, track, feed, name level, playlist, singles group and window group pages.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) Decisions 2, 4, 5 and 6.
- musicindex ADR 0065 R-FIT-2 item 4 and R-FIT-3.
- [ADR 0084](../adr/archive/0084-music-column-browser.md) Decision 4: one release page and one track page for the Library and the Index.

## Recorded Facts

These facts come from a ChatGPT code search of 2026-11-14. The implementer examines each fact before it uses the fact.

- Release: `open_release_in_frame(frame_id, &ReleaseKey, label, cx)` (`src/app/search_dispatch.rs:865`). A local feed gives the Library page. Other keys get the Index page from a request.
- Track: `open_track_in_frame(frame_id, &TrackKey, label, cx)` (`src/app/search_dispatch.rs:1000`). A database track gives the Library page.
- Feed: `open_publisher_page(feed_guid, PublisherPageContext::{Library, Index}, cx)` (`src/app/publisher_dispatch.rs:103`). The Library context reads local data and also sends an Index request. A failed request sets `other_albums_failure`.
- Name level: `open_artist_name(ArtistNameKey, start, cx)` (`src/app/name_level_dispatch.rs:46`). If only one Library feed matches, the feed page opens. The unknown artist key sends no Index request.
- Playlist: `open_page` with `PlaylistDetail(playlist_id)` (`src/library/app_impl.rs:1803`). `select_playlist` reads the tracks from the database.
- Singles group: `open_singles_group(&SinglesGroupKey, cx)` (`src/library/app_impl.rs:714`). It reads the Library members and requests the feed singles.
- Window group: `open_window_group(&WindowGroupKey, cx)` (`src/library/app_impl.rs:702`).
- The harness sends each Index request to `http://127.0.0.1:9`, so each request fails.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Shell fit harness | reuse | src/app/fit_check.rs | Task 002 moves it to `src/app/fit_check/mod.rs`. Use its `init`, `top_app`, `draw_and_read`, `WIDTHS` and `LONG_TITLE`. |
| Fit fixture adapter | extend | src/library/app_impl.rs | Task 002 adds `src/library/fit_fixtures.rs`. Add the methods that put the item page state. |
| Fit probe and fault finder | reuse | src/ui/primitives/fit_probe.rs | `clear`, `count`, `faults` and `figtree_test_context` give the check. |
| Singles group rows | reuse | src/view_models/singles_group.rs | The phase 1 test makes `SinglesGroupPageEntryVm` rows with `LONG_TITLE`. |

## Required Changes

1. Put the phase 3 test in `src/app/fit_check/item_pages.rs`.
2. Give each page long fixture data: titles, artist names, contributor names, role labels and URLs. A URL uses `Label::break_anywhere` only where the page already sets it.
3. Draw each page at each width of `WIDTHS`:
   - The release page and the track page, from the Library and from the Index.
   - The feed page in the context Library and in the context Index.
   - The name level page with two or more Library feeds.
   - The playlist page with a minimum of 3 entries, and one entry of a removed track.
   - The singles group page and the window group page.
4. Draw each page also in the state of a failed Index request, because the harness request fails. Its failure text must fit.
5. For an Index page, put the Index data through the adapter. Do not send a request to a test server.
6. Give each fault message the page and its source.
7. In the same change, set the phase 3 row of the phase plan to Done with the date.

## Do Not Change

- No change to a layout, a token or a primitive. A fault that the test finds gets its own task.
- No change to a production code path. The adapter is test code only.

## Acceptance Criteria

Mechanical:

- The test `adr_0097_the_item_pages_fit_at_the_two_widths` draws each page and each state of Required Changes items 3 and 4 at 390 px and at 940 px.
- Each draw records one or more probes, and one probe holds the text `LONG_TITLE`.
- The test finds no fault, or the report lists each fault (Stop And Ask).
- The tests of phases 1 and 2 pass with no change.
- The commit checks of AGENTS.md pass.

Visual: none. The fit test is the check.

## Test Commands

- `cargo test --lib app::fit_check`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests added. Give the full text of each fault message. Name each page text that does not go through `Label` or `Button`.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the test finds a fault. Do not change the layout. Report each message.
- Stop if a page cannot get its Index data without a change to a production code path. Report the page and the change that it needs.
- Stop if a page has a horizontal scroll area.
