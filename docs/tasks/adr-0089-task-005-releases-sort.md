# ADR 0089 Task 005: The Sort Of Releases And Artists

Status: Ready - 2026-11-05.

Depends on: [packet 004](adr-0089-task-004-list-page-state.md).

## Goal

Releases sorts by Newest or A to Z (ADR 0089 Decision 5). Newest gives the window groups of ADR 0086. A to Z gives the groups of ADR 0084 Decision 6 and a letter bar (R-LIST-1 item 2). The "New releases" view of Discover goes, because Releases takes it (Decision 2). Artists uses A to Z, and this packet examines its Newest source (The Contract).

## Parts

The orchestrator divided this packet on 2026-11-05, because one agent cannot complete it safely in one pass. Each part keeps the gate Green and has its own commit. The design steps below name the owner of each part.

- Part A: steps 1, 3, 6, 7 and 8. The sort state and its menu, the check mark, the letter bar, `medium` on the two requests, and A to Z in All and In Library. The sort Newest shows the present name list until Part B.
- Part B: steps 2 and 10. Newest with window groups in All and In Library, with the sort in the cache key.
- Part C: steps 4 and 5. The tile view for each sort, the window group item, its mark and its selection.
- Part D: step 9. The "New releases" view and its paths go, with their guards.

Result of step 7 (2026-11-05): `GET /v1/feeds/recent` returns `FeedResponse` rows with no name key and no release count, and it gives only release feeds with the default `medium`. `GET /v1/artist-names` has only the sort `sort_name`. Thus Artists has one sort, A to Z, and its menu has one item.

## Files To Inspect

`src/view_models/window_groups.rs`, `src/view_models/window_group_page.rs`, `src/view_models/releases_everything.rs`, `src/view_models/artists_everything.rs`, `src/view_models/index_name_list.rs`, `src/view_models/library.rs` (`refresh_new_releases_layout`, `releases_list`, `cached_releases_rows`, `content_list_page`), `src/view_models/detail_selection.rs` (`discover_*`), `src/library/music_scopes.rs`, `src/library/detail_selection.rs`, `src/library/app_impl.rs` (`show_discover_view`, `replace_recent_music_content`, `recent_music_page`, `render_discover_view_detail`), `src/ui/shells/library/content_list.rs`, `src/ui/shells/library/window_groups.rs`, `src/ui/shells/library/destination_lists.rs`, `src/api.rs` (`fetch_recent_feeds`, `fetch_feeds_by_name`), `src/application/queries/name_lists.rs`, `tests/architecture_tests.rs`.

## Design

1. `ReleasesListKey` gains the sort. The cache stays one cached merge for one key (ADR 0084 packet 037).
2. Newest in All: the source is the recent pager that New releases uses (`GET /v1/feeds/recent` with `medium`). Library releases merge by the date of the Index or the stored feed date. A row that is in the Library keeps its state. Newest in In Library: the source is the Library tree, in date order. The rows group with the window of ADR 0086 through `NewReleasesLayout`. Change its input so it does not depend on `ContentListPageSource::RecentMusic`.
3. A to Z in All: the present path (`GET /v1/feeds/by-name`). A to Z in In Library: the Library tree with the groups of ADR 0084 Decision 6.
4. `ReleasesListItemVm` gains the window group item. A window group row and tile open the page of ADR 0086 Decision 4, and the selection of a group keeps its rule. The window group page keeps `WindowGroupSource`.
5. Tiles and rows: both views exist for each sort. Reuse the Discover tile and row. The virtual list renders rows, and the tile view uses the present grid. Each view keeps Load more.
6. The letter bar: `LetterBarVm` and a shared composite. A letter sets `from` on the by-name request in All, and scrolls to the first row of that letter in In Library. It shows only with A to Z.
7. Artists: A to Z with the letter bar. Examine a Newest source. If `GET /v1/feeds/recent` cannot give a name row, Artists has one sort, the menu has one item, and the run record states the result.
8. `fetch_recent_feeds` and `fetch_feeds_by_name` take `medium`. This packet passes the default `music`. Packet 009 passes `musicL`.
9. "New releases" goes. Delete `DiscoverView`, `FrameNavigationEntry::DiscoverView`, `MusicDestination::default_child_page`, `render_discover_new_releases_detail`, `LibraryDetail::Discover` and the Discover selection paths in `src/view_models/detail_selection.rs` and `src/library/detail_selection.rs`. The Discover destination stays with its saved searches list until packet 006. Move the selection of a window group row to the list page selection. Delete the guards of Discover views and keep the guards of window groups, with their paths changed. Delete the Settings preference of the view mode if no page reads it.
10. A saved search keeps its groups (ADR 0086 Decision 6).

## Do Not Touch

The row layout (packet 014). The saved searches list (packet 006).

## Acceptance Criteria

Mechanical:

- The Releases list view model gives window groups with Newest and Decision 6 groups with A to Z, in All and in In Library. Unit tests hold each of the four.
- The sort menu gives Newest and A to Z, with the check mark on the current sort.
- The letter bar view model is present only with A to Z, and a letter sets `from`.
- `DiscoverView` does not exist.
- The gate is Green.

Visual:

- V1: Releases opens with Newest and shows window groups. A group row opens the group page.
- V2: A to Z shows the letter bar and the singles groups. A letter moves the list.
- V3: Tiles and rows show the same items in each sort.
- V4: Load more works in each sort of All.
- V5: The list stays smooth at 1000 rows, as in ADR 0084 packet 037.

## Test Commands

`cargo test window_groups`, `cargo test releases`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if one window group needs rows that the recent pager does not hold and that ADR 0086 did not cover.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-005-releases-sort.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decisions 2 and 5, The Contract)
- docs/adr/0086-new-releases-singles-window.md
- the files that the task lists

Goal:
- Releases has the sorts Newest and A to Z, with their groups and the letter bar. The New releases view goes.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The row layout and the saved searches list.

Acceptance criteria:
- The mechanical criteria of the task.

Test commands:
- cargo fmt -- --check; cargo clippy --all-targets -- -D warnings; cargo test; cargo build --bin v4vmm

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
