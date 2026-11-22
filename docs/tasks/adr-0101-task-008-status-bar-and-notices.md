# ADR 0101 Task 008: The Status Bar And The Notices

Status: Implemented - 2026-11-16. The orchestrator session chose A in the Stop Report. The operator visual checks are open.

## Goal

The window has one status bar at the bottom, the same on each section. It replaces the live status band. The result of an action shows where the action was. An event that is not the result of a visible action goes to the status bar. HitIt shows no toast.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 7 owns the status bar. Read Decision 7 fully before the first edit:
  - Left: the latest event of one bounded app event list. The list holds the show event reports (ADRs 0059 and 0063) and the events with no visible action of musicindex ADR 0026 R-NOTICE-1 item 2: a feed check, a download or a tag write that fails. Each event keeps the words of its owner and its recorded time (Revised 2026-11-16).
  - Right: small items with a simple icon and a short word. Each item opens its detail. The items are feed state, files to update, reports with a count, and show state.
  - The show item appears only while a show is active. With no active show, Music and Settings show no broadcast state ([ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Decision 18).
  - The reports item opens the normal-shell notice of [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md). Its subjects, repair actions and retained actions stay reachable there and in Settings.
- musicindex ADR 0026 R-NOTICE-1 (`../musicindex/docs/adr/0026-cross-app-controls-and-feedback.md` lines 81 to 109, at musicindex `785ae28`):
  1. The result of an action shows where the action was. A copy button shows "Copied". The bar of an editor shows "Saved, not written".
  2. An event that is not the result of a visible action goes to one status area of the app, the status bar of HitIt.
  3. No app shows a toast: a message that slides in and goes away by itself.
- musicindex ADR 0028 R-PAGE-2 item 4 (`../musicindex/docs/adr/0028-cross-app-item-pages.md` lines 57 to 73): after a copy, the "…" button of the header shows "Copied".
- [ADR 0059](../adr/archive/0059-broadcast-control-surface.md) and ADR 0063 own the words of a report.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ba43138`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `build_live_status_strip` (`src/app/show.rs:293`) builds the band only on Music and Settings. Examined.
- `TopApp` render (`src/app.rs:2181` to `:2222`) puts the capability notice below the toolbar, then the live status band, then `download_status_view`, then the section content. Examined.
- `LiveStatusStrip::render` (`src/ui/composites/live_status_strip.rs:65`) owns the band. `LiveStatusDisplay::from_projection` (`src/view_models/live_status.rs:175`) makes it active with playback, a broadcast or a recording. Examined. It also gives "No active show" and the action "Open Show".
- `capability_report` and `capability_notice` (`src/ui/composites/startup_report.rs:27` and `:135`) own the Settings report and the compact normal-shell notice. Its max height is 160 px (`src/ui/layouts.rs:37`).
- Other notice code: `src/ui/shells/publisher.rs:175`, `src/ui/shells/name_level_page.rs:118` and `src/ui/shells/search_results_inspector.rs:668`.
- `src/view_models/show/event_report.rs:92` gives the time of an event report.
- This is an inference: no toast exists in HitIt.
- Tests: `live_status_no_show_is_absent` (`src/view_models/live_status.rs:245`), `adr_0066_normal_notice_preserves_workspace_height` (`src/ui/composites/startup_report.rs:346`) and `adr_0066_optional_notices_have_one_normal_report_owner` (`src/app/bootstrap.rs:642`). Guard: `adr_0060_live_status_strip_contract_and_mount_are_guarded` (`tests/architecture_tests.rs:14306`).

Main session choice, 2026-11-15: the download status item stays where ADR 0098 puts it. The operator decides at the walk if it moves into the status bar.

Main session choice, 2026-11-15: in the narrow shell, the status bar is above the tab bar.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Live status projection | extend | `src/view_models/live_status.rs` | It gives the show state. The status bar view model uses it for the show item. |
| App event list | new | `src/view_models/app_events.rs` | Searched `src/view_models/show/event_report.rs` (`EventReport` keeps only the latest show result for each operation), the `VecDeque` users in `src/` (download queue, backfill queue, startup capabilities) and `~/build/musicindex-lib` (fit, search and value only). No component holds a bounded list of app events. |
| Status bar view model | new | `src/view_models/status_bar.rs` | Searched `src/view_models/` in HitIt. `live_status.rs` gives only the show. No view model gives the latest event and the right items. |
| Status bar composite | extend | `src/ui/composites/live_status_strip.rs` | It is the one band. Change it into the status bar, and rename the file and the type. |
| Normal-shell notice | reuse | `src/ui/composites/startup_report.rs` | `capability_notice` is the notice that the reports item opens. |
| Shell layout | extend | `src/app.rs` | It mounts the band. Mount the status bar at the bottom there. |
| Mount guard | extend | `tests/architecture_tests.rs` | The ADR 0060 guard covers the band. Change it to the status bar. |

## Required Changes

1. **The event list.** `app_events.rs` holds one bounded list of app events, newest first. Each event has its owner, its words and its recorded time. The list keeps at most 50 events. The show event reports go into the list when `EventSectionInput::record_event_report_for` records them. A feed check that fails, a download that fails and a tag write that fails also go into the list. Each event keeps the words of its owner. This packet adds no new words. Report the source of each event.
1a. **The view model.** `status_bar.rs` gives the latest event of the list with its time, and the right items: feed state, files to update, reports with a count, and show state. The show item exists only while a show is active. Each item gives its icon, its word, its accessibility label and the detail that it opens. Find the source of each item, and report it.
2. **The composite.** The status bar draws the view model in one row at the bottom of the window. The event text truncates. The right items keep their place.
3. **The shell.** The status bar shows on each section. The live status band and its mount go away. The capability notice no longer shows below the toolbar. The reports item opens it.
4. **The copy result.** After a copy, the "…" button of a page header shows "Copied". Find each copy action, and give it this result. A copy in a row menu shows "Copied" in the row.
5. **The other notices.** Examine the notices of `publisher.rs:175`, `name_level_page.rs:118` and `search_results_inspector.rs:668`. A notice that is the result of a visible action stays in its place. A notice of an event moves to the status bar. Report each one.
6. **The guards.** Change the ADR 0060 guard so that one composite owns the status bar and no screen renders a second band. Its message names ADR 0101 Decision 7 and the fix. Delete each guard and test of the band that no longer applies.

## Do Not Change

- No change of the words of a report. ADRs 0059 and 0063 own them.
- No change of the Settings report.
- No change of the download status item, other than its place in the code.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test proves that the status bar view model gives no show item when no show is active, and gives one when a show is active.
- M2: a test proves that the latest event gives its words and its recorded time.
- M2a: a test proves that the list keeps at most 50 events, newest first, and that a show report and a failed feed check both go into it.
- M3: a test proves that the reports item gives the count and opens the normal-shell notice.
- M4: the changed guard proves one composite owner and no second status band.
- M5: a test proves that a copy action gives "Copied" on its button.
- M6: a guard proves that no code builds a notice that closes itself on a timer. Its message names R-NOTICE-1 item 3.
- M7: the fit tests of ADR 0097 pass, with the status bar at 390 px and at 940 px.
- M8: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M, at normal and narrow widths:

- V1: the status bar shows at the bottom of Music, Show and Settings. It reads clearly.
- V2: with no active show, the bar shows no show item. Start a show. The show item appears.
- V3: the reports item opens the notice with its repair actions.
- V4: copy the feed URL on a release page. The "…" button shows "Copied".

## Test Commands

- `cargo test --lib view_models::status_bar`
- `cargo test --lib view_models::live_status`
- `cargo test --lib ui::composites::startup_report`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M6 by name. Give the source of each item of step 1. Give each copy action of step 4 and each notice of step 5 with its result.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if an item of the right side has no source in the code.
- Stop if a failed feed check, download or tag write has no words of its owner in the code.
- Stop if the Show page needs the band for a control that the status bar does not give.
- Stop if a repair action of the notice is not reachable after the change.

## Stop Report - 2026-11-16

The delegate (ChatGPT) stopped before the first edit. The primary agent examined the stop.

- ADR 0101 Decision 7 names ADRs 0059 and 0063 as the owners of the words of the latest event. These ADRs own the show reports only. `src/view_models/show/event_report.rs` is the one event report in the code.
- musicindex ADR 0026 R-NOTICE-1 item 2 sends each event that is not the result of a visible action to the status bar. Examples are a feed check that fails, a download that fails and a tag write that fails. These events have no show report.
- With no active show, ADR 0101 Decision 7 permits no broadcast state on Music and Settings. Thus a show event alone can leave the left side empty for a user with no show.
- The right items have a source: feed state (`RssCheckRunState`, `src/runtime/playlist_rss_check.rs`), files to update (`src/audio_tags.rs`), reports (`src/ui/composites/startup_report.rs`) and the show (`src/view_models/live_status.rs`).

Question: which events go to the left side of the status bar?

Orchestrator choice, 2026-11-16: option A, because musicindex ADR 0026 R-NOTICE-1 item 2 requires it. One bounded app event list collects the show event reports and the events with no visible action. Each event keeps the words of its owner. musicindex ADR 0026 R-NOTICE-1 item 2 binds HitIt, and option B breaks it. ADR 0101 has a Revised line for Decision 7. The main session chose the limit of 50 events.

## Implementation Report

Status: the delegate finished the code and the checks. The visual criteria V1 to V4 are open for the operator. The delegate did not commit.

### Files changed

- New: `src/view_models/app_events.rs`, `src/view_models/status_bar.rs`, `src/ui/composites/status_bar.rs` (renamed from `live_status_strip.rs`), `src/app/status_bar.rs`.
- Changed: `src/app.rs`, `src/app/show.rs`, `src/app/fit_check/mod.rs`, `src/library.rs`, `src/library/app_impl.rs`, `src/library/download_queue.rs`, `src/library/downloads.rs`, `src/runtime/playlist_rss_check.rs`, `src/ui/composites/mod.rs`, `src/ui/primitives/context_menu.rs`, `src/ui/shells/release_page.rs`, `src/ui/shells/track_page.rs`, `src/view_models/{album_page,download_progress,live_status,mod,playlist_rss_check,release_page,tag_update,track_detail,track_page}.rs`, `src/view_models/show/event_report.rs`, `src/view_models/startup/capabilities.rs`, `tests/architecture_tests.rs`.
- Deleted: `src/ui/composites/live_status_strip.rs` (the band composite), `build_live_status_strip`, and the band types and tests of `live_status.rs` (health, recording, heading, open-show button).

### Tests

- M1: `view_models::status_bar::tests::adr_0101_task_008_show_item_exists_only_while_a_show_is_active`.
- M2: `view_models::status_bar::tests::adr_0101_task_008_latest_event_gives_words_and_recorded_time` and `view_models::app_events::tests::adr_0101_task_008_latest_event_keeps_owner_words_and_time`.
- M2a: `view_models::app_events::tests::adr_0101_task_008_list_keeps_fifty_events_newest_first`, `...show_report_and_failed_feed_check_enter_the_list` and `view_models::playlist_rss_check::tests::adr_0101_task_008_failed_feed_gives_the_words_of_the_report_row`.
- M3: `view_models::status_bar::tests::adr_0101_task_008_reports_item_gives_count_and_opens_the_notice`.
- M4: `adr_0101_decision_7_one_composite_owns_the_status_bar_and_no_screen_renders_a_second_band` in `tests/architecture_tests.rs`. It replaces `adr_0060_live_status_strip_contract_and_mount_are_guarded`.
- M5: `view_models::album_page::tests::adr_0101_task_008_copy_action_gives_copied_for_its_button` and `ui::primitives::context_menu::tests::adr_0101_task_008_selected_item_stores_its_result_label`.
- M6: `adr_0101_task_008_no_code_builds_a_notice_that_closes_on_a_timer` in `tests/architecture_tests.rs`. It is a heuristic: it limits the timer users in the UI layers to a list, and it fails when a timer closure names a notice field.
- M7: `app::fit_check::adr_0097_the_status_bar_fits_at_the_two_widths`. At 390 px it found a real fault (four items in one row). The bar now wraps its items.

### Source of each event (step 1)

- Show event reports: `EventSectionInput::report_events` (`src/view_models/show/event_report.rs`). `reproject_show_page` (`src/app/show.rs`) records the new entries after each recording flow. This is a change from the packet text: a hook in `record_event_report_for` would lose or double events, because the input is cloned and replaced in several flows.
- Failed feed check: `view_models::playlist_rss_check::failure_words`. The words are the report row words with the feed name before them, and the run error. `LibraryApp` emits them when a run moves from Running to ended.
- Failed download: `DownloadFailure::words` (`src/view_models/download_progress.rs`), the same words as the progress item. `DownloadQueue` keeps a new-failure outbox. `pump_downloads` emits it.
- Failed tag write: `view_models::tag_update::failure_words` (write error, and each file with a write error) and `request_failure_words` (a confirm that the runtime did not accept). The tag update bridge emits them when a write ends.

### Source of each right item

- Feed state: `PlaylistRssCheckSnapshot` (`running_playlists`, and the latest run with failed feeds). It opens the playlist page.
- Files to update: `view_models::tag_update::button`. It opens the tag update popup.
- Reports: `CapabilityReportVm::report_count`. It toggles the normal-shell notice `reports_open` (on Settings it runs `OpenReport`).
- Show: `LiveStatusDisplay::from_show_page`. It shows only while `ShowPageVm::is_active`. It opens the Show section.

### Copy actions (step 4)

- Library release page "..." menu, Index release page "..." menu, Library track page "..." menu: "Copied" on the button (`result_label`, cleared when the menu opens, no timer).
- Index track page "..." menu: the result comes when the feed read ends. See the orchestrator choice in the Review Corrections section.
- No row menu has a copy action. Other clipboard writes (Nostr icon, podping row button, log copies, report copies) are not in a "..." menu or a row menu and stay unchanged.

### Other notices (step 5)

- `publisher.rs` `unavailable_notice`: the result of the page load, a page state. It stays.
- `name_level_page.rs` `render_matched_feed_notice`: a page state with an open action. It stays.
- `search_results_inspector.rs` `empty_state_content`: the empty or failed state of a search that the operator ran. It stays.

### Limits and inferences

- Inference: the status bar view is a separate entity so that a library change redraws only the bar. The top app pushes its facts at the start of its render.
- A download that the queue fails through a host stop goes through `fail_running` and is in the outbox. A feed check or tag scan error is not an event: `scan_error` is not a tag write.
- Tag update confirm results also show in the existing report. The event is an addition.

### Checks

- `cargo fmt -- --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test`: library 3244 passed, 0 failed (the final run is below in the main report), `architecture_tests` 417 passed, 0 failed.
- `cargo build --bin hitit`: pass.

## Review Corrections - 2026-11-16

An independent review (ChatGPT, `gpt-6-astra`, effort high) examined the delegate change. The primary agent made these corrections.

- A show report that left the 50-event list entered the list again. `AppEventList` now keeps the time of the newest show report that it recorded, and the words at that time. An older report, or the same report at that time, does not enter again. Test: `adr_0101_task_008_a_dropped_show_report_does_not_enter_again`.
- A feed check that failed before the next render gave no event. `LibraryApp` now examines each ended run of the snapshot. It records the start time of each run that it reported, and ignores the runs that started before the session.
- A tag write that failed before the next render gave no event. `TagUpdateSnapshot` now has a `writes_ended` count. The bridge emits an event when the count changes.
- The status bar items used `on_click`. They now use `on_activate`, so Enter and Space open the detail with the key-down handler of the button (ADR 0069).
- Main session choice: the bar has a minimum height of `Size::MinHitTarget`. With no event and no item, the bar was approximately 9 px high, and the content moved when the first event came.

Orchestrator choice, 2026-11-16: the "..." button of an Index track page shows the result of "Copy Feed URL" when the feed read ends. Reason: musicindex ADR 0026 R-NOTICE-1 item 1 says that the result of an action shows where the action was. The copy is a visible action, so item 2 (the status line) does not apply.

- While the read runs, the button is busy (ADR 0101 task 007): it shows the spinner and "Copying…", and takes no input.
- When the read succeeds, the button shows "Copied".
- When the read fails, the button shows "Copy failed", or "No feed URL" when Music Index gives no feed URL. Main session choice: the full failure words are in the hover help of the button, because a long error does not fit on the button.
- The result stays until the menu opens again. The status line no longer gets the words of this copy.
- `IndexFeedCopy` and `TrackPageMenuResult` (`src/view_models/track_page.rs`) hold the state and the result. `ContextMenu::owner_result` and `ContextMenu::on_open` (`src/ui/primitives/context_menu.rs`) show a result that the owner holds, and tell the owner when the menu opens. Test: `adr_0101_task_008_index_copy_shows_its_result_on_the_menu_button`.

### Checks After The Corrections

- `cargo fmt -- --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test`: library 3245 passed, `architecture_tests` 417 passed, 0 failed.

## Walk Result - 2026-11-16

The agent walked the debug binary on an Xvfb display at M, after the review corrections.

- V1: Pass on Music (Light) and Settings (Light and Dark, at 1200 px and 700 px). The bar is at the bottom with a height of approximately 44 px when it is empty. At 700 px it is above the bottom tab bar. No event came during the walk, so the text of an event is not examined.
- V2: Pass for the first half. With no active show, the bar shows no show item. The test data has no show, so the second half is open.
- V3: Not walked. The test data has no report.
- V4: Pass. After Copy Feed URL on "Waltz Tunes Volume 1", its "…" button shows "Copied". The "…" button of "Fox Trot Tunes Volume 2" shows no result.

The operator confirmation stays open.

Walk of the orchestrator choice, 2026-11-16, debug binary, Light, M, on the Index track page "Golden Age" (Let Em Riot):

- After "Copy Feed URL", the "…" button shows "Copied", and the clipboard holds the feed URL. Pass.
- When the menu opens again, the button shows only its icon. Pass.
- The busy state is not seen: the feed read ended before the first screenshot. The failure words are not walked: the walk cannot make the feed read fail. The view model test proves the two states.

## Operator Walk Correction - 2026-11-17

The operator walk of pending human checks section 24 found a defect. On the Index release page "GLOBAL FEDPILLED INDUSTRIAL COMPLEX", Tab went from "Reverse" to the status bar item "Update 28 files" before the rows. Cause: the items got Tab index 0 with `on_activate`, and the detail list has Tab index 1 (ADR 0092 task 002). Correction: each item has `Button::tab_index(1)`, so Tab reaches the bar after the page and its list. Test: `adr_0101_task_008_tab_reaches_the_list_before_the_status_bar`.

Orchestrator choice, 2026-11-17: the status bar shows the time of the latest event in the local time zone. It uses the time format of the system language, with no "UTC". Reason: musicindex ADR 0023 R-DATE-1 item 1 formats a date in the language of the system. A time of day follows the same idea. R-DATE-1 has no rule for a time of day yet. The app event list keeps the time in UTC, and only the display converts it. `fmt_time_in` (`src/view_models/format.rs`) and `StatusEventDisplay::from_event_in` (`src/view_models/status_bar.rs`) give the time. Test: `adr_0101_task_008_event_time_is_local_in_the_language_format`.
