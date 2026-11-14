# ADR 0097 Task 002: The List Pages Fit At The Two Widths

Status: Implemented - 2026-11-14. The test found faults at 390 px. [Task 007](adr-0097-task-007-list-rows-truncate-long-names.md) corrects them, and the test then passes. The test opens the search results after the queued destination event. Without this sequence, that event replaces the results page. This is phase 2 of the [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md).

## Goal

A GPUI layout test draws each list page of the sidebar destinations with long fixture rows at 390 px and at 940 px. The fit probes find no fault of musicindex ADR 0065 R-FIT-3. The pages are Releases, Artists, My playlists, Public playlists, Live, Search and the Inbox.

The packet also makes the fixture adapter that tasks 003 to 005 use.

## Authority

- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md) Decision 2 (the two widths), Decision 4 (the probes and the faults), Decision 5 (the failure message) and Decision 6 (the phases).
- musicindex ADR 0065 R-FIT-2 item 4: each page gets the check. R-FIT-3 gives the faults.
- [ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decision 1: the destinations and the tab bar below 940 px.

## Recorded Facts

These facts come from a ChatGPT code search of 2026-11-14. The main session examined the facts marked "examined".

- `src/app/fit_check.rs` mounts `TopApp` with an in-memory database, opens a destination with `TopApp::open_sidebar_row` and reads `fit_probe::faults` (examined).
- The harness gives no runtime host. `LibraryApp` then makes `AsyncCommandRunner::unavailable` (`src/library/app_impl.rs:1143`, examined). The Library tree load runs as the command `LoadLibraryTracksTree` (`src/library/app_impl.rs:1586`, examined). Thus rows in the database alone do not go into the Releases or Artists rows in this harness.
- The `SidebarDestination` variants are `Releases`, `Artists`, `MyPlaylists`, `PublicPlaylists`, `Live`, `Search` and `Inbox` (`src/view_models/sidebar.rs:18`).
- `LibraryViewModel::replace_tree` (`src/view_models/library.rs:3311`) and `replace_playlists` (`:3879`) take prepared data. `LibraryApp.vm` is private.
- The database test helpers (`setup_test_db`, `create_named_feed`, `insert_track_full` and others) are private in `db::tests` (`src/db.rs:4296`). `playlist_create` (`src/db.rs:2222`), `playlist_append` (`:2316`) and `insert_saved_search` (`src/db/saved_searches.rs:241`) are production functions.
- The Index rows go into these view model setters. Each one is in a private field of `LibraryApp`:
  - Releases Newest: `RecentFeedsPageVm::load_recorded_page`, test only (`src/view_models/recent_feeds.rs:327`, examined).
  - Releases A to Z, Public playlists and Live: `IndexNameListPage::begin_load` and `finish_load` (`src/view_models/index_name_list.rs:201`, `:282`).
  - Artists: `ArtistGroupPages::loaded` (`src/view_models/artist_groups.rs:216`).
  - Search results: `replace_index_results` (`src/view_models/search_results/mod.rs:569`).
- The Inbox rows come from the broadcast readiness report. `LibraryViewModel::show_inbox_report` (`src/view_models/library.rs:3415`) takes a report.
- `library.set_list_view(ListPageKey, ListView, cx)` selects Rows or Tiles (`src/library/music_scopes.rs:303`). `set_music_list_scope` selects the scope (`:72`).
- The Search destination lists the saved searches. Construction reloads them from the database (`src/library/saved_searches.rs:41`).

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Shell fit harness | extend | src/app/fit_check.rs | Move it to `src/app/fit_check/mod.rs`. Keep `init`, `top_app`, `draw_and_read`, `WIDTHS` and `LONG_TITLE` there for each phase. |
| Fit probe and fault finder | reuse | src/ui/primitives/fit_probe.rs | `clear`, `count`, `faults` and `figtree_test_context` give the check. |
| List page view model setters | reuse | src/view_models/index_name_list.rs | `begin_load` and `finish_load` take an Index batch. |
| Artists groups | reuse | src/view_models/artist_groups.rs | `ArtistGroupPages::loaded` takes the rows of a group. |
| Releases Newest page | reuse | src/view_models/recent_feeds.rs | `load_recorded_page` takes a recorded batch. |
| Search results | reuse | src/view_models/search_results/mod.rs | `replace_index_results` takes the Index rows. |
| Library tree and playlists | reuse | src/view_models/library.rs | `replace_tree`, `replace_playlists` and `show_inbox_report` take prepared data. |
| Fit fixture adapter | new | src/library/fit_fixtures.rs | Searched src/db.rs tests, src/app/search_field_list.rs tests, src/presentation/runtime_host.rs and the view model setters above. No test code in a module out of `src/library` gives `LibraryApp` prepared rows. |

## Required Changes

1. Move `src/app/fit_check.rs` to `src/app/fit_check/mod.rs`. Keep the phase 1 tests unchanged. Put the phase 2 test in `src/app/fit_check/list_pages.rs`.
2. Add `src/library/fit_fixtures.rs` as a `#[cfg(test)]` module. It gives methods on `LibraryApp` that put fixture rows into its view model state through the setters of Recorded Facts. It does not send a network request and does not start a command.
3. Make the fixture data with long names: each title, artist name, playlist name and saved search name is 40 characters with no space (`LONG_TITLE`). Give each list a minimum of 3 rows. Give one row the longest text of each column, for example the type label and the count "14 albums · 38 singles".
4. Draw each page at each width of `WIDTHS`:
   - Releases and Artists: the scope Library and the scope All, each in Rows and in Tiles.
   - My playlists, Public playlists, Live and the Inbox: their one view.
   - Search: the list of saved searches, and a results page with Index rows.
5. Give each fault message the page, the scope and the view. The message of `fit_probe` already gives the width, the element and the fix (ADR 0097 Decision 5).
6. Require `fit_probe::count() > 0` for each draw, as the phase 1 test does.
7. In the same change, set the phase 2 row of the phase plan to Done with the date.

## Do Not Change

- No change to a layout, a token or a primitive. A fault that the test finds gets its own task (phase plan, Non-Goals).
- No change to a production code path. The adapter is test code only.
- No new runtime host in the harness.

## Acceptance Criteria

Mechanical:

- The test `adr_0097_the_list_pages_fit_at_the_two_widths` draws each page and each state of Required Changes item 4 at 390 px and at 940 px.
- Each draw records one or more probes.
- Each draw of a page with fixture rows records a probe that holds the text `LONG_TITLE`. This proves that the rows were drawn.
- The test finds no fault, or the report lists each fault (Stop And Ask).
- The phase 1 tests pass with no change.
- The commit checks of AGENTS.md pass.

Visual: none. The fit test is the check. The visual items of ADR 0097 stay in section 14 of [pending human checks](../pending-human-checks.md).

## Test Commands

- `cargo test --lib app::fit_check`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests added. Give the full text of each fault message that the test found. Name each page text that does not go through `Label` or `Button`, because phase 6 adds the probe for that text.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the test finds a fault. Do not change the layout. Report each message. The main session writes the fix task.
- Stop if a page cannot get fixture rows without a change to a production code path. Report the page and the change that it needs.
- Stop if a page has a horizontal scroll area. The fault finder must ignore that area first (phase plan, Risks).
