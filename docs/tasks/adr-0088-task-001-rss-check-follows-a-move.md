# ADR 0088 Task 001: The RSS Check Follows A Feed Move

Status: Ready - 2026-11-11. The operator accepted ADR 0088 on 2026-11-11. Implementation has not started.

## Goal

The RSS check reads the MusicIndex `feed_url` of each stored feed with a GUID. A stored feed moves to that URL when the RSS document at that URL proves the same `podcast:guid`, and the check report states the move.

## Authority

- [ADR 0088](../adr/0088-a-feed-move-follows-musicindex.md) owns each rule of this packet.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) Decision I: MusicIndex is a cache of RSS.
- [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md): the RSS check and its report.
- [AGENTS.md](../../AGENTS.md): "Write reports for the operator", and the construction of a blocking HTTP client only in `src/http_client.rs`.

## Recorded Facts

- 2026-11-04: a search of `src/` finds no reader of `itunes:new-feed-url`.
- 2026-11-04: a search of `src/` finds no production statement that changes `feeds.feed_url`. Read the two statements in `src/application/queries/library.rs` first, because they appear to be test setup.
- 2026-11-04: `invalidate_feed_document` and the retained document in `src/rss/enrich.rs` use the feed URL as their key.
- 2026-11-04: `check_rss_document` in `src/rss/enrich.rs` and `src/rss/check_apply.rs` own the check and its comparison.
- 2026-11-04: `src/rss/host.rs` holds the host throttle. ADR 0084 packet 039 owns the limit for each host.
- 2026-11-11: `feeds.feed_guid` exists. `src/feed_service.rs` sends `GET /v1/feeds/{guid}`.
- 2026-11-11: in `tests/fixtures/musicindex-openapi-0.19.0.json`, `GET /v1/feeds/{guid}` gives `feed_url`. Its 404 can carry `superseded_by` for a GUID change.
- 2026-11-11: `GET /v1/feeds/{guid}` gives a feed of each medium, also a playlist feed (Stophammer ADR 0038 Decision B, and a live read by the orchestrator session).

## Required Changes

1. Find whether the RSS check already reads the MusicIndex record of each feed. If it does, use that `feed_url`. If it does not, request `GET /v1/feeds/{guid}` through the existing MusicIndex client.
2. Compare that `feed_url` with the stored URL. Lowercase the scheme and the host, and ignore one trailing `/`. An equal URL gives no candidate.
3. Fetch the candidate with the check path, through the host throttle.
4. Apply the proof of ADR 0088 Decision 2 in one function in `src/rss/check_apply.rs`.
5. Change `feeds.feed_url` in one `db` function, in one transaction with the check result. Clear the retained document of the earlier URL.
6. Add typed report items for a move and for a refused candidate. The view model gives the text. The report holds the two URLs, the feed title, the reason and the recorded time.

## Do Not Change

- No reader of `itunes:new-feed-url`. Stophammer ADR 0083 owns the tag.
- No move with no RSS proof.
- No move for a feed with no stored `feed_guid`, and no use of `GET /v1/feeds/by-url`.
- No tag write. "Update n file(s)" stays the only write (ADR 0076 Decision 8).
- No action for a 404 with `superseded_by`.
- No schema change.

## Acceptance Criteria

Mechanical:

- A test for each criterion of ADR 0088 "Verification", with a local test server for RSS and for MusicIndex.
- The check sends one request for each candidate, and no request when no candidate exists.
- A candidate on a private address follows the rule of `src/http_client.rs`. No test sends a request to a public host.
- The view model gives each report item with its two URLs and its recorded time.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: after a move in Studio, and after the node reads it, the RSS check report names the old and the new URL.
- V2: a later edit in Studio reaches the compare in HitIt.

## Operator Visual Check

This check needs Studio, two file hosts for a test feed and a node that reads the test feed.

1. Publish a test feed in Studio, and subscribe to it in HitIt.
2. Run `cargo run --bin hitit` in a desktop session.
3. Move the feed to the second host in Studio.
4. Wait until `GET /v1/feeds/{guid}` of the node gives the new URL.
5. Run the RSS check of the playlist with that feed.
6. Wrong: the report does not name the two URLs, or the report names a different feed.
7. Change the title of a track of the feed in Studio, and publish it.
8. Run the RSS check again.
9. Wrong: the compare does not show the new title.

Cleanup: remove the subscription from the Library, and delete the test feed in Studio.

## Test Commands

- `cargo test --lib rss::`
- `cargo test --lib db::`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests run. Tell whether the RSS check already read the MusicIndex record, or whether the packet added the request.

## Stop And Ask

- Stop if a different table or a path on disk uses the stored URL as a key. One transaction must update each key.
