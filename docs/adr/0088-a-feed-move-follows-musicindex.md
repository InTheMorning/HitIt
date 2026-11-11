# ADR 0088: A Feed Move Follows MusicIndex

## Status

Accepted - 2026-11-11. The operator agreed on 2026-11-04 that a new ADR decides this subject. On 2026-11-11 the operator selected the MusicIndex method of this record and accepted it. [Task 001](../tasks/adr-0088-task-001-rss-check-follows-a-move.md) implements it.

## Context

v4vmm keys a stored feed on `feeds.feed_url`. The RSS check of ADR 0076 reads that URL. On 2026-11-04, a search of `src/` found no reader of `itunes:new-feed-url`, and no production statement that changes `feeds.feed_url`.

Three other parts of the chain changed on 2026-11-04:

- Studio ADR 0128 gives each move an `itunes:new-feed-url` in the feed on the new host, with the URL of that host. The pointer file at the old host also names the new host. The pointer file keeps the feed and its items (`pointer_bytes` in `crates/core/src/pipeline.rs` of Studio).
- Studio ADR 0127 gives a changed feed a new `lastBuildDate`.
- Stophammer ADR 0083 moves a record to a new URL. A return to an earlier URL needs a tag at each end, and it has a wait that grows. Release 0.18.1 holds this rule.

Thus a moved feed can stay readable at its old URL. The RSS check of v4vmm then compares the tags of a file with an old copy of the feed, and it reports no change. No message tells the curator.

The node already follows the tag. In the stored contract of Stophammer candidate v0.19.0-rc.7, `GET /v1/feeds/{guid}` gives the current `feed_url` of a feed. v4vmm stores `feeds.feed_guid` and already sends this request.

The research note `research_notes/aggregation/itunes-new-feed-url.md` in `../musicindex-research` gives the facts about the tag. The tag gives no identity, and two URLs can name each other. The research binds nothing.

## Decision

### 1. MusicIndex Gives The Candidate

For each stored feed with a `feed_guid`, the RSS check reads `feed_url` from `GET /v1/feeds/{guid}`.

- A `feed_url` that names the stored URL is not a move. Two URLs are equal when they are equal after a lowercase scheme and host, and with no trailing `/`.
- Each other `feed_url` is a candidate URL.
- v4vmm reads no `itunes:new-feed-url`. Stophammer ADR 0083 owns the use of the tag, its loop guard and its wait before a return to an earlier URL.

A stored feed with no `feed_guid` does not move. A MusicIndex answer of 404, or no answer, gives no candidate. The check then continues as before.

### 2. RSS Proves The Same Feed

MusicIndex is a cache of RSS, and RSS is the only provenance (ADR 0075 Decision I). Thus MusicIndex gives only the address. v4vmm fetches the candidate URL with the shared HTTP client and the host throttle. v4vmm moves the feed only when each of these is true:

- The candidate document parses as RSS.
- The `podcast:guid` of the candidate equals the stored `feed_guid`.
- No other row of `feeds` holds the candidate URL.

### 3. A Move Is Reported

The RSS check report states the old URL, the new URL, the feed title and the recorded time. A refused candidate is also reported with its reason, for example a different GUID. The report follows the "Write reports for the operator" rule of AGENTS.md.

### 4. A Move Needs No Confirmation

The GUID proof and the report are sufficient, and a move writes no file. The owner of a feed can move the stored URL. That owner can also change each value of the feed today, so the risk is not new.

### 5. A Move Writes No Tags

A move changes `feeds.feed_url` only. v4vmm keeps no copy of the old URL. The report states it. The next compare of the check reads the new document. "Update n file(s)" writes tags only after the operator confirms (ADR 0076 Decision 8).

## Alternatives Considered

- v4vmm reads the tag, fetches the candidate and follows one URL in each check. This was the proposal of 2026-11-04. Rejected: it repeats the work of Stophammer ADR 0083 in a second place, and the two apps can then show different URLs.
- Follow the MusicIndex `feed_url` with no RSS proof. Rejected: MusicIndex is a cache of RSS (ADR 0075 Decision I). The RSS check reads the candidate in each case, so the proof adds almost no cost.
- Find the GUID of a stored feed with no GUID through `GET /v1/feeds/by-url`. Not selected: the candidate then has no stored GUID to prove against.
- No change. Rejected: Studio now moves feeds with a tag, and the tags of a file can then go stale with no message.

## Consequences

- A stored feed gets its new URL at the first RSS check after the node reads the move.
- A return to an earlier URL waits for the Stophammer wait of ADR 0083. v4vmm and the node show the same URL.
- A feed that the node does not hold never moves. ADR 0038 of Stophammer limits the public lists to `medium = music`. Task 001 examines whether `GET /v1/feeds/{guid}` gives a playlist feed.
- Each moved feed costs one more RSS request at that check. Each stored feed with a GUID costs one MusicIndex request, unless the check already reads that record.
- `feeds.feed_url` is no longer constant, so each cache that uses the URL as a key must clear the old entry (`invalidate_feed_document` in `src/rss/enrich.rs`).
- A GUID change of Stophammer ADR 0052 is a different event. A 404 with `superseded_by` is not a move, and this record does not decide it.

## Invariants

- MusicIndex gives only the candidate URL. The RSS document proves the identity.
- A stored feed moves only to a document with the same `podcast:guid`.
- A `feed_url` that names the stored URL moves nothing.

## Verification

Mechanical criteria, phrased at the owning layer:

- A MusicIndex `feed_url` that names the stored URL gives no candidate, also with a different case in the host or a trailing `/`.
- A candidate with the same GUID changes `feeds.feed_url` and clears the retained document of the old URL.
- A candidate with a different GUID, no GUID or a URL that another row holds changes nothing, and the report holds the reason.
- A stored feed with no GUID, a MusicIndex 404 and no MusicIndex answer each give no candidate and no request to a candidate.
- The report of a move holds the old URL, the new URL and the recorded time.
- No production code reads `itunes:new-feed-url`.

Visual criteria, for the operator:

- After a move in Studio, and after the node reads it, the RSS check report in v4vmm names the two URLs.
- A later edit in Studio reaches the compare.
