# ADR 0109: Check All Feeds Reads No RSS

## Status

Proposed - 2026-11-21. The operator agreed to the two changes on 2026-11-21, after the measurement in the Context.

- Bar: crosses modules, and opposite choice by mistake. The change touches the feed commands, the feed service and the ADR 0075 observation guards. [ADR 0075 task 039](../tasks/adr-0075-task-039-feed-check-and-update-observation-retention.md) and its guard tell an agent that the feed update must merge RSS.
- Changes: the scope of ADR 0075 task 039. The feed update keeps its MusicIndex observations and records no RSS observation.

## Context

- The operator wants HitIt to read RSS only on request, for example in the playlist RSS check of [ADR 0076](0076-playlist-rss-check-for-stale-musicindex-records.md) before a show. "Check all feeds" must use the musicindex API.
- "Check all feeds" (`CheckFeedsAndRepairRoutes`, `src/application/commands/feed.rs`) has four steps:
  1. A staleness check: one musicindex feed request for each feed, in sequence.
  2. An update of each changed feed (`apply_feed_updates`, `src/feed_service.rs`): one feed request and one track detail request for each downloaded track.
  3. The route repair.
  4. The tag scan.
- In step 2, `merge_track_context_with_recorder` reads the RSS of the feed for each downloaded track. HitIt discards the merged result. The read only records a provider observation in the `metadata_*` tables. The stored value projection, `rss_license_facts`, the tag scan and the screen text do not read these rows.
- A timing run on 2026-11-21 with the data of the operator: 91 feeds, 0 changed feeds. Step 1 took 8.7 s, step 3 took 0.1 s and step 4 took 0.3 s. Step 2 did not run. Thus about 0.1 s for each feed in step 1 is most of the time.
- HitIt has a pattern for bounded parallel requests: `fetch_feed_singles_set` (`src/application/queries/name_lists.rs`) and `fetch_recent_feed_result_rows` (`src/application/queries/feed.rs`). Each uses `PARALLEL_DETAIL_READS = 4`, an index of the work and results in the input order.

## Decision

### 1. The feed update reads no RSS

1. `apply_feed_updates` does not call the RSS enrichment. It keeps its musicindex feed request, its track detail requests, the stored musicindex values and the update marker.
2. These paths continue to read RSS:
   - The playlist RSS check of ADR 0076.
   - The subscription of a feed.
   - The track detail of the Library reader.
3. The RSS observations of a feed stay as they are until one of these paths reads the RSS again.

### 2. The staleness check sends up to four requests at the same time

1. Step 1 uses the bounded pattern of `fetch_feed_singles_set`, with `PARALLEL_DETAIL_READS`.
2. The list of changed feeds keeps the order of the input feeds. Step 2 continues to update the feeds one after the other.
3. Each call has its own observation recorder and its own receipt list. The command joins the receipts after the calls. A shared recorder can attach the receipt of one feed to the response of a different feed.
4. A cancel stops the start of new calls. A call that started continues until it ends or until its timeout.

## Invariants

- "Check all feeds" sends no RSS request.
- The changed feeds have the same order as the input feeds.

## Alternatives

- **Keep the RSS merge and make only step 1 parallel.** A run with changed feeds still reads the RSS once for each feed, with no effect on the stored values. Rejected.
- **A musicindex endpoint that checks many feeds in one request.** It can be faster than four parallel requests, but it needs work in the musicindex repository. It stays possible later. Not selected now.
- **More than four requests at the same time.** HitIt uses four for each bounded read. A higher number puts more load on the musicindex server. Rejected.

## Tasks

- None. A task file follows when the operator accepts this ADR.
