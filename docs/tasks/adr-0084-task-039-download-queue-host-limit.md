# ADR 0084 Task 039: The Download Queue Waits For A Host That Sends HTTP 429

Status: Implemented - 2026-10-25. The visual checks V1 to V4 stay open in [pending human checks](../pending-human-checks.md#8-packets-038-and-039---implemented-2026-10-25). The operator answered Q1 to Q3 on 2026-10-24. The orchestrator applied the recommendation for Q4. The step 1 change of ADR 0084 Decision 10 landed before the code.

## Goal

The download queue sends one RSS request at a time to each host, with a minimum interval
between two requests. When a host sends HTTP 429 (Too Many Requests), the queue waits and tries
the job again a limited number of times. The progress item tells the operator that the app waits
for the host, and until when. A failure line names the host, the HTTP status and what the app
did.

## Authority

- [ADR 0084](../adr/0084-music-column-browser.md) Decision 10, amended 2026-10-20: one
  download queue, at most three downloads at the same time, one progress item. This packet
  amends Decision 10. The operator decided the amendment (Q1 to Q4).
- [ADR 0058](../adr/0058-outbound-http-client-policy.md): all blocking HTTP clients are built in
  `src/http_client.rs`. Its "Consequences" section says: "Retry, backoff, and per-host policy
  remain unowned." No ADR owns a host policy for the download queue today.
- [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 2 and
  "Accepted Values": the host policy of the playlist RSS check only. It is the model for this
  packet. It does not bind the download queue.
- [ADR 0040](../adr/0040-async-vm-runtime.md): background work runs through `present_command`
  and the runtime. A screen never calls `cx.spawn`.
- [AGENTS.md](../../AGENTS.md): "Write reports for the operator", "Every fix gets a guard" and
  "Current-view state must update in place".

## Recorded Facts - 2026-10-24

### The defect

- The operator walk of 2026-10-24, at 18:13 local time. The operator selected one singles group
  of 38 `MusicIndex` singles and clicked "Download". Each single is a Wavlake feed,
  `https://wavlake.com/feed/music/<uuid>`.
- The progress item showed "Downloaded 20 of 38, 18 failed".
- Each failure line has this text:

  ```text
  The app could not download <title>: feed command failed: HTTP error for https://wavlake.com/feed/music/<uuid>: HTTP status client error (429 Too Many Requests) for url (…)
  ```

- The operator also saw that the app was slow while the download ran.
- ADR 0076 "Context" records that Wavlake throttles crawlers. The Stophammer Wavlake import
  waits 2 seconds or more between requests and waits 300 seconds after HTTP 429.
- It is not known if Wavlake sends a `Retry-After` header with its HTTP 429 response. The
  current code does not read it.

### The path of one single from a group row

1. `src/library/downloads.rs:115-123` `pump_downloads` starts each job that
   `DownloadQueue::start_next` gives. It has no time gate.
2. `src/library/download_queue.rs:32` sets `MAX_IN_FLIGHT` to 3. The rule
   `next_startable_index` (`:534-544`) compares only the `FeedKey` of the jobs. It does not
   compare the host. Thus three jobs to `wavlake.com` run at the same time. When one job ends,
   the next job starts at once, with no interval.
3. `src/library/downloads.rs:393-427` `read_index_release_job` reads the feed from MusicIndex
   (`FetchIndexFeedDetail`). This request goes to the MusicIndex host, not to Wavlake. Only
   after this read does the job know its RSS URL and thus its host.
4. `src/library/downloads.rs:343-389` `run_index_release_job` sends `SubscribeFeed`.
5. `src/application/ports/download_manager.rs:183-195` calls the recovery owner and changes
   each error to `DownloadError::Failed(String)` (`:67-72`).
6. `src/subscribe_service.rs:139-141` `subscribe_feed_retaining` locks the database mutex and
   then calls `rss::subscribe_feed`. The track path does the same at `:384-386`.
7. `src/rss/subscribe.rs:331-339` `subscribe_feed` sends one GET with
   `crate::http_client::document()`. It calls `error_for_status` and adds the context
   "HTTP error for {feed_url}". It reads no status and no `Retry-After` header. It does not try
   again.
8. `src/application/commands/feed.rs:743-748` changes the error to
   `CommandError::Feed(String)`. `src/application/errors/command.rs:182` adds the text
   "feed command failed:".
9. `src/library/download_queue.rs:394` records a `DownloadOutcome::Failed` as one failure. The
   queue continues with the next job (module comment, `:13-14`).
10. `src/view_models/download_progress.rs:250-258` makes the failure line "The app could not
    download {label}: {error}".

### The root cause

- The queue has no per-host limit and no interval between two requests to one host
  (`src/library/download_queue.rs:32` and `:534-544`).
- The RSS fetch keeps no typed HTTP status and no `Retry-After` value
  (`src/rss/subscribe.rs:336-337`). The error becomes a string at
  `src/application/ports/download_manager.rs:194`. Thus the queue cannot tell HTTP 429 from
  another failure.
- No code waits or tries again after HTTP 429. `src/http_client.rs` owns only the timeouts, and
  ADR 0058 leaves retry and host policy unowned.

### The slow app during the download

- The same code is a possible cause. `src/subscribe_service.rs:139-141` and `:384-386` keep the
  database mutex locked during the HTTP GET of `rss::subscribe_feed`. That GET can take a maximum
  of 30 seconds (`OPERATION_TIMEOUT`, `src/http_client.rs`).
- `LibraryApp` locks the same mutex on the main thread, for example at
  `src/library/app_impl.rs:1541` and `:1642`. Thus the main thread can wait for a Wavlake
  response.
- No measurement confirms this cause. The overhaul plan list "Throughout: Responsiveness And
  Background Feedback" records the lag. Q3 asks if this packet also moves the GET out of the
  lock.

### Code that the fix can use

- `src/runtime/playlist_rss_check.rs:52-62` holds the ADR 0076 values `MIN_HOST_INTERVAL`
  (2 seconds) and `MAX_RETRY_AFTER` (60 seconds).
- `src/runtime/playlist_rss_check.rs:1113-1129` `retry_after` reads a number of seconds or an
  HTTP date. It reads them from a captured header in base64.
- `src/runtime/playlist_rss_check.rs:1131-1140` `feed_host` gives the lowercase host of an HTTP
  or HTTPS URL.
- `src/presentation/delayed_presenter.rs:14` `present_after` runs a closure on an entity after
  a delay.

## Design

The design is option A of Q1. The Decisions below describe it.

### The typed throttle failure

`rss::subscribe_feed` reads the HTTP status before `error_for_status`. For status 429 it returns
a typed error, `HostThrottled { host, retry_after }`, in the `anyhow` chain. `retry_after` is
the parsed `Retry-After` value, or `None`. Each other status keeps its current error.

`DownloadError` gets the variant `HostThrottled { host, retry_after }`. The download manager
port finds the typed error with `downcast_ref` and does not change it to a string.
`CommandError` gets the same variant, and `feed_download_error` and the track command keep it.
`present_command` gives it to the failure closure of the job.

### The host gate of the queue

`DownloadQueue` stays a pure scheduler with no I/O and no GPUI type. It gets one map of host
states, from the lowercase host to:

- `busy`: one job sends its RSS request to this host now.
- `not_before`: the earliest time of the next RSS request to this host.
- `stopped`: the reason when the queue sends no more requests to this host in this batch.

Before a job sends `SubscribeFeed` or `SubscribeTrack`, `LibraryApp` asks the queue for the host
of the RSS URL. The queue gives one of three answers:

- `Go`: the job sends its command now. The queue sets `busy`.
- `WaitUntil(time)`: the job keeps its slot and gets the phase `WaitingForHost`. `LibraryApp`
  asks again after the wait with `present_after`.
- `Stopped(reason)`: the job fails with the reason.

When the command ends, the queue clears `busy` and sets `not_before` to the end time plus the
minimum interval. After HTTP 429, it sets `not_before` to the end time plus the wait of Q2, for
all jobs of that host. It puts the job back at the front of the queue and adds one to its try
count. After the last try, the job fails.

### The progress text

`DownloadPhase` gets `WaitingForHost { until }`. The progress item shows the subject, the action
and the object, with the recorded time. Examples:

- Title: "Downloading 21 of 38. Waiting for wavlake.com until 18:14:05".
- Detail: "wavlake.com sent HTTP 429 (Too Many Requests) at 18:13:05. The app tries again at
  18:14:05. Try 2 of 3."
- Failure line: "The app could not download Rise Up: wavlake.com sent HTTP 429 (Too Many
  Requests) 3 times. The app stopped the download of this release."

The times come from the queue records. The renderer adds no time.

## Decisions

- **D1.** The queue owns the host gate. Reason: the queue owns the progress item and "Stop
  queued". A wait inside a blocking command would hide the wait from the operator.
- **D2.** The RSS fetch returns a typed throttle failure. Reason: the queue must tell HTTP 429
  from other failures. A match on the error text is not permitted (typed enums in the
  application layer, AGENTS.md "Errors").
- **D3.** The host rule applies to the RSS request only. The MusicIndex read, the MusicIndex
  track reads and the enclosure downloads keep their current behavior. Reason: only the RSS
  host sent HTTP 429 in the walk.
- **D4.** A waiting job keeps its slot. Reason: the slot count stays the rule of Decision 10.
  A batch of one host thus runs one RSS request at a time.
- **D5.** "Stop queued" also removes each job that waits for a host. Reason: the operator can
  then stop a long wait.
- **D6.** `src/http_client.rs` gets no retry. Reason: it owns client construction only (ADR
  0058), and a retry there would hide the wait from the queue.

## Questions

- **Q1.** Where does the host wait live?

  - Option A: in the download queue, as the Design describes. The progress item shows the wait.
    "Stop queued" removes a waiting job.
  - Option B: in the RSS fetch, as one blocking wait for each host in a shared map. It is less
    code. The progress item cannot show the wait, and a stop cannot end it.

  Recommendation: option A.

- **Q2.** Which values apply to the download queue?

  - The minimum interval between two RSS requests to one host. Recommendation: 2 seconds, the
    ADR 0076 value and the Stophammer Wavlake value.
  - The RSS requests to one host at the same time. Recommendation: 1.
  - The wait after HTTP 429 with a `Retry-After` value of 60 seconds or less. Recommendation:
    that value.
  - The wait after HTTP 429 with no `Retry-After` value. Recommendation: 60 seconds. The
    Stophammer import waits 300 seconds, but a 5-minute wait in the app looks like a stopped
    download.
  - A `Retry-After` value of more than 60 seconds. Recommendation: the queue stops the host for
    the batch, as ADR 0076 does. Each remaining job of that host fails with a line that names
    the host and the requested wait.
  - The number of tries of one job. Recommendation: 3, the first request and two more.

- **Q3.** Does this packet also move the HTTP GET of `rss::subscribe_feed` out of the database
  lock?

  - Option A: yes. The function fetches the document first. Then it locks the database and
    stores the document. The fix changes the same function, and the main thread does not wait
    for a Wavlake response.
  - Option B: no. A separate responsiveness packet takes it.

  Recommendation: option A. No measurement confirms that the lock causes the lag. Step 4 of the
  operator check gives a first observation.

- **Q4.** Does the host rule also apply to the other callers of `rss::subscribe_feed`, for
  example the feed refresh in `src/feed_service.rs:369`?

  Recommendation: no, not in this packet. The typed failure helps these callers, but they keep
  their behavior. ADR 0058 "Follow-Up Work" keeps a general retry policy as a separate decision.

## Operator Answers - 2026-10-24

- Q1: A. The host wait lives in the download queue. The progress item shows the wait, and "Stop queued" removes a waiting job.
- Q2: the values of ADR 0076:
  - The minimum interval between two RSS requests to one host is 2 seconds.
  - The queue sends 1 RSS request at a time to one host.
  - After HTTP 429 with a `Retry-After` value of 60 seconds or less, the queue waits for that value.
  - After HTTP 429 with no `Retry-After` value, the queue waits 60 seconds.
  - After HTTP 429 with a `Retry-After` value of more than 60 seconds, the queue stops the host for the batch.
  - One job has 3 tries.
- Q3: yes. This packet moves the HTTP GET of `rss::subscribe_feed` out of the database lock.
- Q4: the operator was not asked. The orchestrator applied the recommendation. The host rule applies to the download queue only. The other callers of `rss::subscribe_feed` keep their behavior.

## Required Changes

1. `docs/adr/0084-music-column-browser.md`: amend Decision 10 with the answers to Q1 to Q4. Add
   one "Amended 2026-10-24" line at the top. Name ADR 0058 and say that Decision 10 now owns the
   host rule of the download queue. Update the ADR 0084 row of `docs/adr/README.md`.
2. Move `feed_host` and the text parse of `retry_after` out of
   `src/runtime/playlist_rss_check.rs` to one shared owner, for example `src/rss/host.rs`. The
   RSS check and the download queue use the same functions. The RSS check keeps its behavior.
3. `src/rss/subscribe.rs`: add the typed error `HostThrottled { host, retry_after }`. Return it
   for HTTP 429. Fetch the document before the caller locks the
   database (Q3). Change `src/subscribe_service.rs:139-141` and `:384-386` to match.
4. `src/application/ports/download_manager.rs`: add `DownloadError::HostThrottled`. Keep the
   typed error in `subscribe_feed` and `subscribe_track`.
5. `src/application/errors/command.rs` and `src/application/commands/feed.rs`: add
   `CommandError::HostThrottled`. Keep it in `feed_download_error` and in the track command.
   Its display text names the host, the status and the requested wait.
6. `src/library/download_queue.rs`: add the host map and the answers `Go`, `WaitUntil` and
   `Stopped`. Add the try count of each job and `DownloadOutcome::Throttled`. Add the values of
   Q2 as named constants. "Stop queued" removes waiting jobs. Update the module comment.
7. `src/library/downloads.rs`: ask the queue for the host before each `SubscribeFeed` and
   `SubscribeTrack`. Wait with `present_after`. Give a throttle failure to the queue as
   `DownloadOutcome::Throttled`. Update the module comment.
8. `src/view_models/download_progress.rs`: add `DownloadPhase::WaitingForHost`, the wait title,
   the wait detail and the throttle failure line. Each text names the host, the status, the
   recorded time and the next action.
9. `tests/architecture_tests.rs`: add the guard R39-09.
10. Add each test of "Mechanical Acceptance Criteria".
11. Remove the item of the download lag from "Throughout: Responsiveness And Background
    Feedback" only if the operator finds no lag in step 4. Else keep it, and add the result.

## Mechanical Acceptance Criteria

Each criterion here is mechanical. A unit test beside its owning code has the prefix
`adr_0084_task_039_`. A guard is in `tests/architecture_tests.rs` and has the case number in its
name. No test sends a network request. The values in the cases are the values of Q2.

| Case | Required proof |
|---|---|
| R39-01 | Mechanical, unit test in `src/library/download_queue.rs`. With two queued jobs of the host `wavlake.com`, the first `Go` answer goes to the first job. The second job gets `WaitUntil` while the first job is busy |
| R39-02 | Mechanical, unit test, same file. After the first job of a host ends at time `t`, the next job of that host gets `WaitUntil(t + 2 s)`. At `t + 2 s` it gets `Go` |
| R39-03 | Mechanical, unit test, same file. A job of a second host gets `Go` while a job of `wavlake.com` is busy |
| R39-04 | Mechanical, unit test, same file. A `Throttled` result with `Retry-After` 30 seconds comes at time `t`. Then the job is first in the queue and its try count is 2. Each job of the host gets `WaitUntil(t + 30 s)`. With no `Retry-After` the wait is 60 seconds |
| R39-05 | Mechanical, unit test, same file. A third `Throttled` result of one job records one failure. The batch failure text names the host, HTTP 429 and the try count |
| R39-06 | Mechanical, unit test, same file. A `Retry-After` value of 61 seconds stops the host. Each queued job of that host fails with a text that names the host and the requested wait. Jobs of other hosts continue |
| R39-07 | Mechanical, unit test, same file. "Stop queued" removes a job in the phase `WaitingForHost`, and the batch counts it as stopped |
| R39-08 | Mechanical, unit test in `src/rss/subscribe.rs` with a local fixture server. A 429 response with `Retry-After: 30` gives `HostThrottled` with the host and 30 seconds. A 404 response gives no `HostThrottled` |
| R39-09 | Mechanical, guard. Situational: it cites ADR 0084 Decision 10 as amended by this packet, and is deleted when that decision is superseded. `src/library/downloads.rs` asks the queue for the host before each `SubscribeFeed::new` and `SubscribeTrack::new`. Its failure message names ADR 0084 Decision 10, this packet and the fix |
| R39-10 | Mechanical, unit test in `src/view_models/download_progress.rs`. A batch with one job in `WaitingForHost` gives the title "Downloading 21 of 38. Waiting for wavlake.com until 18:14:05" and a detail that names HTTP 429, the recorded time and the try count |
| R39-11 | Mechanical, unit test in `src/application/ports/download_manager.rs`. A typed `HostThrottled` error from the recovery owner stays `DownloadError::HostThrottled`. It does not become `Failed` |
| R39-12 | Mechanical, unit test in the shared host owner of step 2. The parse of `Retry-After` reads `30` and an HTTP date. `feed_host` gives `wavlake.com` for `https://WavLake.com/feed/music/x`. The tests of the RSS check pass with no change |
| R39-13 | Mechanical, unit test in `src/subscribe_service.rs`. The fixture server holds its response until the test locks the database mutex from a second thread. The lock succeeds while the GET waits |

The failure message of R39-09 has this form:

```text
ADR 0084 Decision 10, task 039: a download must ask the queue for the RSS host
before it sends SubscribeFeed or SubscribeTrack. Without the host gate, the queue
sends requests to one host too fast and the host sends HTTP 429. Call the host
gate in this job.
```

## Visual Acceptance Criteria

Each criterion here is visual. Only a person can examine it. No test proves it.

- V1: a download of one singles group of 30 or more Wavlake singles completes with no failure
  line that names HTTP 429.
- V2: while the queue waits for a host, the progress item names the host and the time of the
  next try. The text changes in place when the wait ends.
- V3: a failure line, if one shows, names the host, the HTTP status and what the app did. It
  does not show "feed command failed".
- V4: "Stop queued" during a wait removes the waiting jobs, and the progress item shows them as
  stopped.

## Exclusions

- No change to the MusicIndex requests or to the enclosure downloads.
- No retry for a status other than 429.
- No change to the feed refresh or to the RSS check, other than the shared functions of step 2.
- No change to `src/http_client.rs`.
- No schema change and no migration.

## Files To Inspect

- [AGENTS.md](../../AGENTS.md) and the [source map](../architecture/source-map.md).
- `src/library/download_queue.rs` (`MAX_IN_FLIGHT`, `start_next`, `next_startable_index`,
  `finish`, `stop_queued`).
- `src/library/downloads.rs` (`pump_downloads`, `run_index_release_job`,
  `read_index_release_job`, `run_index_track_job`, `read_index_track_job`).
- `src/rss/subscribe.rs` (`subscribe_feed`).
- `src/subscribe_service.rs` (`subscribe_feed_retaining` and the track path).
- `src/application/ports/download_manager.rs`, `src/application/commands/feed.rs` and
  `src/application/errors/command.rs`.
- `src/view_models/download_progress.rs` (`DownloadPhase`, `DownloadProgressDisplay::project`).
- `src/runtime/playlist_rss_check.rs` (`MIN_HOST_INTERVAL`, `MAX_RETRY_AFTER`, `retry_after`,
  `feed_host`).
- `src/presentation/delayed_presenter.rs` (`present_after`).

## Checks

```bash
cargo test
cargo test --test architecture_tests
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
touch src/main.rs && cargo build --bin hitit
```

## Rollback

Revert the commit. This packet adds no migration and no stored data. A rollback returns the
queue to three jobs with no host rule.

## Operator Visual Check

This check needs a Linux desktop session. It needs a connection to `https://api.musicindex.org`
and to `https://wavlake.com`. It needs one singles group of 30 or more Wavlake singles that are not
in the Library. An example is the group of the walk of 2026-10-24 at 18:13.

1. Build and start the debug binary:

   ```bash
   cd ~/build/HitIt
   cargo build --bin hitit
   nm -C target/debug/hitit | grep -c TestDispatcher
   ./target/debug/hitit
   ```

   The `nm` command must give `0`.

2. In Music, open Releases with the scope "Everything". Click "Select", select the singles group
   row, and click "Download". Record the time.

3. V2: look at the progress item while the download runs.
   **Wrong if:** the item says that it waits, but does not name the host or the time of the next
   try. **Wrong if:** the time of the next try is in the past and the text does not change.

4. Look at the app while the download runs. Scroll the Releases list and open one release page.
   **Wrong if:** the list or the page stops for more than one second. Record the time and what
   you did.

5. V1 and V3: when the progress item shows the finished batch, open its failure lines.
   **Wrong if:** a line names HTTP 429 after fewer tries than the answer to Q2. **Wrong if:** a
   line shows "feed command failed" or does not name the host.

6. V4: start a second download of another group of Wavlake singles. While the progress item
   shows a wait, click "Stop queued".
   **Wrong if:** a waiting job starts after the stop, or the progress item does not count it as
   stopped.

Cleanup: steps 2 and 6 add singles to the Library. If the Library must stay as before the
check, select those singles in Releases with the scope "Library" and remove them.

## Implementation Notes - 2026-10-25

The deviations below keep the decided rules. The operator confirms deviations 1 and 3 at the next walk.

1. A busy host (R39-01). The packet names `WaitUntil` while a second RSS request to the host runs. The queue gives `HostTurn::Busy`. A busy host has no end time. The download keeps its slot and asks again after `HOST_BUSY_RECHECK` (1 second). A `WaitUntil` with an invented time would show an incorrect time in the progress item.
2. No `DownloadOutcome::Throttled` (step 6). `DownloadQueue::host_throttled` takes the download and returns it to the queue. `DownloadOutcome` derives `Eq` and `PartialEq`, and `DownloadJob` holds an `api::Feed`, which cannot derive them.
3. The Library downloads also wait for their host. Step 7 names `src/library/downloads.rs` only. `run_library_feed_job` and `run_library_track_job` in `src/library/app_impl.rs` also send `SubscribeFeed` or `SubscribeTrack`. The two runners belong to the download queue, so the host rule applies to them. Without the gate, the rule would not hold for Library releases. Guard R39-09 covers the two files.
4. A stopped host fails each queued download when that download starts. The host gate answers `Stopped` at the start. The text and the count match the packet.
5. A waiting download shows as queued in release and row states. `DownloadPhase::WaitingForHost` maps to the queued state in `src/view_models/library.rs` and `src/view_models/releases_everything.rs`. The packet names no release state for a wait. The progress item shows the wait.
6. A retry keeps the feed. A retried Index release download keeps its `api::Feed`, so the next try reads no MusicIndex again.
7. Shared host functions (R39-12). `feed_host` and the `Retry-After` parse moved to `src/rss/host.rs`. The playlist RSS check keeps its tests and its behavior.
8. The feed GET runs before the database lock (Q3). `rss::fetch_feed_document` runs before the lock at each of the two calls in `src/subscribe_service.rs`. `rss::subscribe_feed` keeps its signature for `src/feed_service.rs`.

Tests in this packet:

- `src/library/download_queue.rs`: R39-01 to R39-07 (`adr_0084_task_039_r39_01_*` to `adr_0084_task_039_r39_07_*`).
- `src/rss/host.rs`: R39-12 and the typed error. `src/rss/subscribe.rs`: R39-08, with a local server, one test for HTTP 429 and one for HTTP 404.
- `src/subscribe_service.rs`: R39-13, the database lock is free during the GET.
- `src/view_models/download_progress.rs`: R39-10. `src/application/ports/download_manager.rs`: R39-11.
- `tests/architecture_tests.rs`: R39-09, `adr_0084_task_039_r39_09_download_jobs_ask_the_queue_for_the_host`. It fails when a copy drops the gate.

Gate on 2026-10-25: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are Green. The suite ran 2576 library tests and 397 architecture tests.

Responsiveness item 32 stays open. No measurement exists. Step 4 of the operator check gives the first observation.
