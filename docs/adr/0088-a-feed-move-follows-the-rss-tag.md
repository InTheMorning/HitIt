# ADR 0088: A Feed Move Follows The RSS Tag

## Status

Proposed - 2026-11-04. The operator agreed on 2026-11-04 that a new ADR decides this subject. The operator has not decided the questions in "Open Questions". The recommendation of this record is the proposed value of each.

## Context

v4vmm keys a stored feed on `feeds.feed_url`. The RSS check of ADR 0076 reads that URL. On 2026-11-04, a search of `src/` found no reader of `itunes:new-feed-url`, and no production statement that changes `feeds.feed_url`.

Three other parts of the chain changed on 2026-11-04:

- Studio ADR 0128 gives each move an `itunes:new-feed-url` in the feed on the new host, with the URL of that host. The pointer file at the old host also names the new host. The pointer file keeps the feed and its items (`pointer_bytes` in `crates/core/src/pipeline.rs` of Studio).
- Studio ADR 0127 gives a changed feed a new `lastBuildDate`.
- Stophammer ADR 0083 moves a record to a new URL. A return to an earlier URL needs a tag at each end, and it has a wait that grows. Release 0.18.1 holds this rule.

Thus a moved feed can stay readable at its old URL. The RSS check of v4vmm then compares the tags of a file with an old copy of the feed, and it reports no change. No message tells the curator.

The research note `research_notes/aggregation/itunes-new-feed-url.md` in `../musicindex-research` gives these facts.
Its fetched sample of 2026-11-04 has 621 tags. Of these, 559 tags name the URL of their feed with the same characters. Another 13 tags name a different spelling of that URL.

The tag gives no identity. Two URLs can name each other. The sample selects hosts that write the tag, so the fractions do not describe all feeds. The research binds nothing.

## Decision

### 1. The RSS Check Reads The Tag

The RSS check reads the channel `itunes:new-feed-url` of each document that it fetches.

- A tag that names the URL of the document, in any spelling that normalizes to the same URL, is not a move. This is the normal form.
- A tag with no valid `http` or `https` URL is not a move. The app keeps it as evidence only.
- Each other tag gives a candidate URL.

### 2. The Candidate Must Prove The Same Feed

The tag gives no identity, so v4vmm fetches the candidate URL with the shared HTTP client. v4vmm moves the feed only when each of these is true:

- The candidate document parses as RSS.
- The `podcast:guid` of the candidate equals the stored `feed_guid`. A stored feed or a candidate with no GUID does not move.
- No other row of `feeds` holds the candidate URL.

MusicIndex is not a proof. A difference between the stored URL and the MusicIndex `feed_url` is not a candidate, because MusicIndex is a cache of RSS (ADR 0075 Decision I).

### 3. One Move In Each Check

A check moves a feed by one URL. The check does not follow the tag of the candidate. A later check reads it.
Two URLs that name each other thus give at most one move in each check, and no loop in one check.

### 4. A Move Is Reported

The RSS check report states the old URL, the new URL, the feed title and the recorded time. A refused candidate is also reported with its reason, for example a different GUID. The report follows the "Write reports for the operator" rule of AGENTS.md.

### 5. A Move Writes No Tags

A move changes `feeds.feed_url` only. The next compare of the check reads the new document. "Update n file(s)" writes tags only after the operator confirms (ADR 0076 Decision 8).

## Open Questions

1. **Does a move need a confirmation?** Recommendation: no. The GUID proof and the report are sufficient, and a move writes no file. The owner of a feed can move the stored URL with a tag. That owner can also change each value of the feed today, so the risk is not new.
2. **Does v4vmm follow a return to an earlier URL at once?** Recommendation: yes. v4vmm stores one URL, and Decision 3 limits the rate to one move for each check. The Stophammer wait of ADR 0083 protects a public index that signs events, and v4vmm has no such event. The two apps can thus show different URLs for some time.
3. **Does v4vmm keep the old URL?** Recommendation: no. The report states it, and no reader needs a stored copy.

## Alternatives Considered

- Follow the MusicIndex `feed_url`. Rejected: MusicIndex is a cache of RSS, and the stored feed can then change with no RSS proof.
- Follow the tag with no fetch. Rejected: the tag gives no identity, and any feed can name any URL.
- Follow a chain to its end in one check. Rejected: it needs loop detection and a limit of hops, and one hop for each check gives the same result in time.
- No change. Rejected: Studio now moves feeds with a tag, and the tags of a file can then go stale with no message.

## Consequences

- A stored feed reaches its new host at the next RSS check.
- Each moved feed costs one more request at that check.
- `feeds.feed_url` is no longer constant, so each cache that uses the URL as a key must clear the old entry (`invalidate_feed_document` in `src/rss/enrich.rs`).

## Invariants

- A stored feed moves only to a document with the same `podcast:guid`.
- A check moves a feed at most one time.
- A tag that names its own document moves nothing.

## Verification

Mechanical criteria, phrased at the owning layer:

- A tag that names the URL of its document gives no candidate, also with a different case in the host.
- A candidate with the same GUID changes `feeds.feed_url` and clears the retained document of the old URL.
- A candidate with a different GUID, no GUID or a URL that another row holds changes nothing, and the report holds the reason.
- Two documents that name each other give one move in one check.
- The report of a move holds the old URL, the new URL and the recorded time.

Visual criteria, for the operator:

- After a move in Studio, the RSS check report in v4vmm names the two URLs, and a later edit in Studio reaches the compare.
