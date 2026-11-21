# ADR 0109 Task 001: Changed Feeds Get One RSS Check

## Status

Open - 2026-11-21. ADR 0108 task 001 must be complete first, because it adds migration 29.

## Goal

"Check all feeds" sends a maximum of four musicindex requests at the same time. A feed that musicindex marks as changed gets one RSS check of ADR 0076 after its musicindex update. The check applies the RSS values. The RSS merge for each track goes.

## Authority

[ADR 0109](../adr/0109-check-all-feeds-reads-rss-of-changed-feeds.md) Decisions 1 and 2. It changes the scope of [ADR 0075 task 039](adr-0075-task-039-feed-check-and-update-observation-retention.md). [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) gives the RSS check. ADR 0040 gives the runtime actor.

## Change

1. Staleness check (Decision 2.1). `check_feed_batch_for_updates` (`src/application/commands/feed.rs`) uses the pattern of `fetch_feed_singles_set` (`src/application/queries/name_lists.rs`): `std::thread::scope`, `PARALLEL_DETAIL_READS` workers, an atomic work index and results in the input order.
   - Each call gets its own `ProviderObservationRecorder` and its own receipt list. The function joins the receipts in the input order after the calls.
   - Each worker examines the cancel token before it takes the next feed.
   - A provider storage failure stops the start of new calls. The function returns the failure of the first feed in the input order that failed.
2. Feed update (Decisions 1.3 and 2.2). `apply_feed_updates` (`src/feed_service.rs`):
   - Remove the call of `merge_track_context_with_recorder`. If the function then has no caller, delete it. Remove `invalidate_feed_document` only if no other code needs it.
   - The track detail requests run with the same bounded pattern. The database writes (`persist_musicindex_feed`, `persist_musicindex_track`) stay in the input order of the tracks, after the requests.
   - The changed feeds stay in sequence, one after the other.
3. RSS check of a changed feed (Decisions 1.1, 1.2, 1.4 and 1.5).
   - `CheckFeedsAndRepairRoutesResult` and the result of `ApplyFeedUpdates` give the ids of the feeds that the update changed without an error.
   - In `src/library/app_impl.rs`, the completion callbacks of `check_all_feeds` and `apply_all_feed_updates` send these ids to the RSS check actor. Send nothing when the list is empty.
   - Add `PlaylistRssCheckMessage::StartFeeds { feed_ids, trigger }` in `src/runtime/playlist_rss_check.rs`. It uses the existing host queue, the conditional GET, `check_feed`, `record` and `apply_checked_document`. Do not copy them.
   - Add the trigger `RssCheckTrigger::FeedUpdate`, with the token `feed_update`.
   - Migration 30 in `src/db.rs`: `rss_check_runs.playlist_id` accepts `NULL`, and the trigger check accepts `feed_update`. SQLite cannot change a column constraint. Rebuild the table, keep each row and keep the foreign keys of `rss_check_feed_results`. Set `CURRENT_VERSION` to 30.
   - A run with no playlist does not show in the playlist snapshot. When it ends, the actor reloads the marks of each loaded playlist that has one of the feeds. Then the differences show where the ADR 0076 differences show now (Decision 1.6).
4. Tag scan. The RSS check can change stored values. When a feed run of item 3 ends, `LibraryApp` starts the tag update scan again (`scan_tag_updates`). The existing scan after the musicindex update stays.
5. The status text of "Check all feeds" does not change in this task.
6. Tests and guards of ADR 0075 task 039. Change these to the new rule. Do not delete a test that has a different rule in it:
   - `adr_0075_feed_observation_update_preserves_requests_files_and_markers`: the update sends no `/feed.xml` request, and the receipts do not include an RSS receipt.
   - `adr_0075_feed_observation_retained_bodies_survive_a_database_reopen`: move the RSS part to a test of the feed run of item 3, or delete only the RSS part.
   - `adr_0075_feed_observation_storage_failures_stop_dependent_work`: remove the RSS storage failure case. Keep the musicindex cases.
   - `adr_0075_feed_observation_ordinary_failures_keep_skips_and_messages`: change the request and receipt counts.
   - `adr_0075_feed_observation_cancellation_keeps_cause_and_receipts` and `adr_0075_feed_observation_combined_root_keeps_receipts_through_repair`: change the exact counts only if the parallel calls change them. Each test keeps its rule.
   - The guard `adr_0075_feed_observation_roots_and_consumers_are_guarded` (`tests/architecture_tests.rs`): it must not require `merge_track_context_with_recorder` in `apply_feed_updates`. It must fail when `apply_feed_updates` calls the RSS enrichment. The failure message names ADR 0109 Decision 1.3 and the fix.
7. Change these documents in the same change:
   - The status line and the report of this task file.
   - A line in ADR 0075 task 039 that names ADR 0109 for the RSS merge of the update.
   - `docs/pending-human-checks.md`: a new section 49 with the visual criteria below.

Main session choices, 2026-11-21:

- The Library app sends the feed ids to the actor after the command ends. The command is synchronous and cannot wait for the actor. Thus the RSS check runs after the musicindex update (Decision 1.4).
- A feed run uses the same host queue as a playlist run. If a playlist run and a feed run read one host at the same time, the host interval applies to the two runs. If the queue cannot do this without a large change, stop and report.
- If a step needs a decision that this task file and ADR 0109 do not give, the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Bounded parallel reads | reuse | src/application/queries/name_lists.rs | `fetch_feed_singles_set` gives the worker pattern with `PARALLEL_DETAIL_READS`. |
| Parallel read limit | reuse | src/application/queries/mod.rs | `PARALLEL_DETAIL_READS = 4`. |
| Feed commands | extend | src/application/commands/feed.rs | The staleness check runs in parallel, and the results give the changed feed ids. |
| Feed service | extend | src/feed_service.rs | `apply_feed_updates` drops the RSS merge and runs the track detail requests in parallel. |
| RSS check actor | extend | src/runtime/playlist_rss_check.rs | It gets a feed run with the existing host queue, conditional GET and apply. |
| RSS check run store | extend | src/db/rss_check_runs.rs | A run can have no playlist and the trigger `feed_update`. |
| Migration registry | extend | src/db.rs | Migration 30 rebuilds `rss_check_runs`. |
| Library feed callbacks | extend | src/library/app_impl.rs | They send the changed feed ids and scan the tags after the feed run. |

## Mechanical Criteria

1. A test with 9 feeds and a fake musicindex server: the stale list has the input order, and the server sees a maximum of 4 requests at the same time.
2. A test: with a cancel after the first feeds, no new request starts, and the receipts of the started calls stay.
3. A test: the receipts of each feed have the observation of that feed.
4. A test: `apply_feed_updates` sends no RSS request. The track writes have the input order of the tracks.
5. A test: the results of `CheckFeedsAndRepairRoutes` and `ApplyFeedUpdates` give the ids of the changed feeds, and not the ids of the feeds that failed.
6. A runtime test: `StartFeeds` with two feeds sends one conditional GET for each feed, records a run with no playlist and the trigger `feed_update`, and applies the RSS values.
7. A runtime test: `StartFeeds` with an empty list records no run.
8. A migration test: after migration 30, each earlier run row stays, a run with no playlist is valid, and the trigger `feed_update` is valid. The migration registry guards pass.
9. A test: a feed run reloads the marks of a loaded playlist that has one of the feeds.
10. The tests and the guard of item 6 pass with the new rules.
11. The commit checks of AGENTS.md pass.

## Visual Criteria

These go in section 49 of [pending human checks](../pending-human-checks.md).

- V1: "Check all feeds" with no changed feed ends in a shorter time than the 8.8 s of the timing run of 2026-11-21. The agent can measure this only with the data of the operator.
- V2: after a feed changes in musicindex, "Check all feeds" updates the feed. The RSS differences of that feed show in the playlist of the feed.

## Report

Write the report here when the task is complete. Name each changed file, the method of the host interval between two runs, and each criterion with its test name.
