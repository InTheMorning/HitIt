# Responsiveness Task 001: No UI Thread Database Wait

Status: Implemented - 2026-11-06. The operator ran downloads and removals on 2026-11-06 and reported that the app does not hang.

Owner items: [overhaul plan](../plans/design-and-cleanup-overhaul-plan.md#throughout-responsiveness-and-background-feedback), items 9, 13, 14, 34 and 35.

Authority: [ADR 0084](../adr/0084-music-column-browser.md) Decision 10 as amended 2026-10-24, task 039, and AGENTS "Async" and "Current-view state must update in place".

## Problem

A download or a removal froze the window. A capture of the main thread (`v4vmm-stacks.txt`, 30 samples, one each second) showed the main thread in `futex_wait` on the database lock in 15 of 23 active samples. The waits came from `reload_playlists` and from the broadcast readiness report. A capture of all threads (`v4vmm-stacks-all.txt`, 15 samples) showed the lock holders:

- the readiness report read the tags of each Library file under the lock, in the runtime actor and in the Inbox path.
- the Library tree load converted the metadata rows of each album under the lock.
- `subscribe_feed_body` made a MusicIndex request under the lock. The capture did not show this request, but the code did.

## Change

1. The MusicIndex feed read of a subscribe runs before the lock. `rss::musicindex_updated_at_for_body` reads it, and `rss::subscribe_feed_body` persists it. The callers are the subscribe service, the feed service and the public playlist copy.
2. The readiness report has a plan step and a file step. The plan step reads the rows and the stored facts under the lock. The file step reads the tags with no lock. The Inbox command `LoadBroadcastReadiness` and the runtime actor use the same plan.
3. Readiness loads, Library tree loads and playlist loads run one at a time. A request during a run sets one run-again flag. A result of an old run is dropped. `src/library/single_flight.rs` holds the rule.
4. `reload_playlists` reads the playlist list and the covers through `LoadPlaylistSidebar`. A step that reads the list after a reload runs in the completion, through `reload_playlists_then`.
5. The Library tree load locks once for each album. The metadata conversion runs after the lock is released.
6. The detail rows of a frame are computed once in `render_workspace_content`. A window group page no longer copies its rows.

## Mechanical Criteria

Each criterion names the test or guard that checks it.

| Criterion | Check |
|---|---|
| The subscribe body makes no MusicIndex request and the callers read it before the lock | Guard `adr_0084_task_039_subscribe_persists_without_a_musicindex_request` in `tests/architecture_tests.rs` |
| The readiness file step takes no database connection and runs with the lock held by a test | Test `adr_0084_task_039_readiness_files_are_read_with_the_lock_free` in `src/application/queries/broadcast.rs` |
| The Library loads do not run on the UI thread and read files only after the lock | Guard `adr_0084_task_039_readiness_and_playlists_load_off_the_ui_thread` in `tests/architecture_tests.rs` |
| A burst of requests gives one more run, and a stale run is dropped | Tests `adr_0084_task_039_burst_of_requests_gives_one_more_run` and `adr_0084_task_039_only_the_current_run_is_applied` in `src/library/single_flight.rs` |
| A run of a gone session never overwrites a newer state | Test `adr_0084_task_039_a_run_of_a_gone_session_never_overwrites_a_newer_one` in `src/library/single_flight.rs` |
| The last request of a run gets a result | Test `adr_0084_task_039_requests_during_a_live_run_keep_it_current` in `src/library/single_flight.rs` |
| Each Library load starts through `SingleFlight` | Guard `adr_0084_task_039_library_loads_coalesce_through_single_flight` in `tests/architecture_tests.rs` |
| The package builds, passes clippy with `-D warnings`, and passes the tests | Gate in the run record of 2026-11-06 |

## Visual Checks

These checks need a person in a desktop session. They are open. The operator owns the list in [pending human checks](../pending-human-checks.md).

1. Start a download of one release. The window keeps its response to a click while the download runs.
2. Remove one track from the Library. The window keeps its response, and the Inbox count updates in place.
3. Open the Inbox from the sidebar. The rows show after the load, and the page shows no empty state before the load.
4. Open a playlist after you create it. The playlist page opens with its name after the list reloads.

## Cleanup

No state is created by this task. The checks above use the Library you already have.

## Not Changed

- `build_tree` still reads the stored values and the identity facts of each album under one lock call.
- Other main-thread lock sites remain. They are listed in the run record of 2026-11-06.
