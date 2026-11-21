# ADR 0106 Task 002: A Load Opens The Show

## Status

Implemented - 2026-11-19. The mechanical criteria pass. The visual criteria wait for a person, in section 46 of the pending human checks.

## Goal

"Load into Show cue" opens the Show page. The empty Show page lists My playlists, each with "Load into Show cue".

## Authority

[ADR 0106](../adr/0106-the-cue-changes-during-a-show.md) Decision 4.

## Change

1. After a load succeeds (`src/library.rs:790`), the app selects Show (`src/app.rs:439`). A failed load stays in Music with its error.
2. When the cue is empty, the Show view model gives the text "Select a playlist for your show.". It also gives the rows of My playlists in the Curator Order, each with "Load into Show cue". This replaces "No active show" and "Show playback is idle." (`src/view_models/show.rs:1864`).
3. The row action "Open playlists" of ADR 0105 opens this list. When ADR 0105 task 001 is not built, this item waits.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Show cue load | reuse | src/library.rs | The load command does not change. |
| Destination select | extend | src/app.rs | The success of a load selects Show. |
| Show view model | extend | src/view_models/show.rs | The empty state gives the text and the playlist rows. |
| Playlist list rows | reuse | src/ui/composites/list_row.rs | The rows of My playlists. |

## Mechanical Criteria

1. A test: a successful load selects Show. A failed load keeps Music and gives the error.
2. A view model test: an empty cue gives the text and one row for each playlist in the Curator Order, each with "Load into Show cue".
3. No source file has the text "Show playback is idle.".
4. The commit checks of AGENTS.md pass.

## Visual Criteria

These are in section 46 of [pending human checks](../pending-human-checks.md).

- V4: "Load into Show cue" on a playlist opens Show with that cue.
- V5: with an empty cue, Show lists the playlists. "Load into Show cue" on a row loads it.

## Report

Items 1, 2 and 3 are built. ADR 0105 task 001 (commit 2139387) built item 3.

Changed files:

- `src/library.rs`: the new event `LibraryAppEvent::ShowCueLoadFailed(String)`.
- `src/library/app_impl.rs`: the new method `LibraryApp::load_show_cue`. The Music row menu and the Show page both call it. It uses the same commands as before. A failed load emits `ShowCueLoadFailed`.
- `src/view_models/library.rs`: the method `status_text` of the Library view model.
- `src/app.rs`: `ShowCueLoaded` selects Show through `tab_after_show_cue_event`. `ShowCueLoadFailed` shows its message on Show only when Show is open. The new field `show_playlists`.
- `src/app/show.rs`: `RefreshShowPage` reads `db::playlists_list`. This is the read of My playlists. The Show page calls `load_show_cue` through the new slot.
- `src/view_models/show.rs`: the empty state gives "Select a playlist for your show." when the cue has no entry. The text "No active show" and the text "Show playback is idle." are removed from this view model. The new type `ShowPlaylistRowDisplay`. The new field `cue_playlists`.
- `src/view_models/playlist_curation.rs`: `show_playlist_rows` uses `ordered_playlists` in the Curator Order.
- `src/ui/shells/show.rs`: the new slot `on_load_playlist`. The rows are `ListRow` with a "Load into Show cue" button.

Tests for the mechanical criteria:

1. `app::tests::adr_0106_a_successful_load_selects_show_and_a_failed_load_keeps_the_tab` proves the select of Show from Music and from Show, and the kept tab after a failure. `view_models::library::tests::adr_0106_a_failed_show_cue_load_gives_the_error` proves the error text.
2. `view_models::playlist_curation::tests::adr_0106_show_playlist_rows_follow_the_curator_order` proves the order and the action label. `view_models::show::tests::adr_0106_empty_cue_gives_the_text_and_the_playlist_rows` proves the text. `view_models::show::tests::adr_0106_a_cue_with_entries_gives_no_playlist_rows` proves that a cue with entries gives no rows.
3. A search of `src` and `tests` finds no "Show playback is idle.".
4. The commit checks pass (see the report to the main session).

Deviations and open points:

- The ADR 0060 guard forbids `crate::db` in the Show view model. The row projection is in `playlist_curation.rs`, and the Show view model takes the finished rows.
- `ShowEmptyStateDisplay.subtitle` is now `Option<&'static str>`.
- `ShowPageVm::is_active` uses the new field `active`. It did use `empty_state.is_none()`.
- A stopped Show with cue entries has no empty state now. Before, it showed "No active show". The task does not say what to show. This is an open point for the operator.
- The row action of the Show page says "Load into Show cue". The Music menu item says "Load into Show Cue". The task text gives the first form. The two labels differ in case.
- The status bar still says "No active show" (`src/view_models/live_status.rs`). It is a separate text.
- A failed load from the Show page shows its error in the status line of Show (`settings_status`).
- No test covers the render of the rows or the click. V4 and V5 stay visual checks.

Main session review:

- The empty state now uses the full cue, not the rows after the text filter. A filter that hides each row does not show the playlist list. The test `adr_0106_a_cue_with_entries_gives_no_playlist_rows` covers it.
- Open: the Music menu says "Load into Show Cue" and "Replace Show Cue". ADR 0106 and the Show row say "Load into Show cue". The operator decides the case.
- Open: a stopped Show with cue entries has no empty state. The status bar still says "No active show".

Item 3, main session check after ADR 0105 task 001:

- The on air row "Show cue" gives "Open playlists" when the cue is empty or an entry is not ready (`src/view_models/show.rs`, the test of `with_on_air_band`). `TopApp::on_air_row_action` in `src/app/show.rs` selects Show and opens the cue panel. With an empty cue, the Show page lists My playlists below the band. No code change is necessary.
- No unit test covers this app link. Step 7 of section 46 examines it.
