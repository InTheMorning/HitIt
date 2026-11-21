# ADR 0109: Check All Feeds Reads RSS Of Changed Feeds

## Status

Accepted - 2026-11-21. The operator gave the direction on 2026-11-21: keep RSS current when musicindex signals a change, and make the requests fast and efficient. The operator accepted the ADR on 2026-11-21.

- Bar: crosses modules, and opposite choice by mistake. The change touches the feed commands, the feed service, the RSS check of ADR 0076 and the ADR 0075 observation guards. [ADR 0075 task 039](../tasks/adr-0075-task-039-feed-check-and-update-observation-retention.md) and its guard tell an agent that the feed update merges RSS for each track.
- Changes: the scope of ADR 0075 task 039. The RSS read of the feed update moves to the RSS check of [ADR 0076](0076-playlist-rss-check-for-stale-musicindex-records.md).

## Context

- "Check all feeds" (`CheckFeedsAndRepairRoutes`, `src/application/commands/feed.rs`) has four steps:
  1. A staleness check: one musicindex feed request for each feed, in sequence.
  2. An update of each changed feed (`apply_feed_updates`, `src/feed_service.rs`): one feed request and one track detail request for each downloaded track, in sequence.
  3. The route repair.
  4. The tag scan.
- A feed is changed when its musicindex `updated_at` is more recent than the stored value. This is the signal of musicindex.
- In step 2, `merge_track_context_with_recorder` reads the RSS of the feed. The first track downloads the document. The other tracks use the cached document. HitIt discards the merged result. The read only records a provider observation in the `metadata_*` tables. No stored value, no tag and no screen text changes.
- The RSS check of ADR 0076 (`src/runtime/playlist_rss_check.rs`) reads one document for each feed. It sends a conditional GET with the stored `ETag` and `Last-Modified`. It waits 2 s between two requests to one host, and it reads a maximum of 4 hosts at the same time. It applies the RSS values to the stored values (`apply_checked_document`) and records the differences. A run belongs to one playlist (`rss_check_runs.playlist_id` is `NOT NULL`).
- A timing run on 2026-11-21 with the data of the operator: 91 feeds, 0 changed feeds. Step 1 took 8.7 s, step 3 took 0.1 s and step 4 took 0.3 s. Step 2 did not run.
- HitIt has a pattern for bounded parallel requests: `fetch_feed_singles_set` (`src/application/queries/name_lists.rs`) and `fetch_recent_feed_result_rows` (`src/application/queries/feed.rs`). Each uses `PARALLEL_DETAIL_READS = 4`, an index of the work and results in the input order.

## Decision

### 1. A changed feed gets one RSS check

1. When step 1 finds a changed feed, HitIt checks the RSS of that feed with the RSS check of ADR 0076. The check sends one conditional GET for the feed, keeps the host interval and applies the RSS values to the stored values.
2. An unchanged feed gets no RSS request.
3. `apply_feed_updates` does not call the RSS merge for each track. It keeps its musicindex feed request, its track detail requests, the stored musicindex values and the update marker.
4. The RSS check runs after the musicindex update of the feed. Thus the RSS values, which own the provenance (ADR 0075 Decision I), are the last values that HitIt stores.
5. The RSS check of a feed is not part of a playlist. Its run has no playlist and a trigger that names "Check all feeds". The task gives the storage of this run.
6. The differences that the RSS check records show where the differences of the ADR 0076 RSS check show now.

### 2. Requests run in parallel with a limit

1. Step 1 sends a maximum of 4 musicindex requests at the same time. It uses the pattern of `fetch_feed_singles_set` with `PARALLEL_DETAIL_READS`.
2. In step 2, the track detail requests of one feed run a maximum of 4 at the same time with the same pattern. The database writes stay in the input order.
3. The RSS checks of Decision 1 use the host queue of ADR 0076. They do not add a second limit.
4. Each parallel call has its own observation recorder and its own receipt list. The command joins the receipts after the calls. A shared recorder can attach the receipt of one feed to the response of a different feed.
5. The list of changed feeds keeps the order of the input feeds.
6. A cancel stops the start of new calls. A call that started continues until it ends or until its timeout.

## Invariants

- "Check all feeds" sends no RSS request for a feed that musicindex does not mark as changed.
- "Check all feeds" sends at most one RSS request for each changed feed.
- The changed feeds have the same order as the input feeds.

## Alternatives

- **No RSS read in "Check all feeds".** It is the fastest, but the stored RSS values stay old until a playlist RSS check. Rejected by the operator.
- **Keep the RSS merge for each track.** It reads RSS, but it changes no stored value. Rejected.
- **A musicindex endpoint that checks many feeds in one request.** It can be faster than four parallel requests, but it needs work in the musicindex repository. It stays possible later. Not selected now.
- **More than four requests at the same time.** HitIt uses four for each bounded read. A higher number puts more load on the musicindex server. Rejected.

## Tasks

- [Task 001](../tasks/adr-0109-task-001-changed-feeds-get-one-rss-check.md): the RSS check of changed feeds and the parallel requests.
