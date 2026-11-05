# ADR 0088 Task 001: The RSS Check Follows A Feed Move

Status: Waits for the operator - 2026-11-04. ADR 0088 is Proposed. Do not write code until the operator accepts it and answers its open questions.

## Goal

The RSS check reads `itunes:new-feed-url`. A stored feed moves to its new URL when the document at that URL proves the same `podcast:guid`, and the check report states the move.

## Authority

- [ADR 0088](../adr/0088-a-feed-move-follows-the-rss-tag.md) owns each rule of this packet.
- [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md): the RSS check and its report.
- [AGENTS.md](../../AGENTS.md): "Write reports for the operator", and the construction of a blocking HTTP client only in `src/http_client.rs`.

## Recorded Facts - 2026-11-04

- A search of `src/` finds no reader of `itunes:new-feed-url`.
- A search of `src/` finds no production statement that changes `feeds.feed_url`. Read the two statements in `src/application/queries/library.rs` first, because they appear to be test setup.
- `invalidate_feed_document` and the retained document in `src/rss/enrich.rs` use the feed URL as their key.
- `check_rss_document` in `src/rss/enrich.rs` and `src/rss/check_apply.rs` own the check and its comparison.
- `src/rss/host.rs` holds the host throttle. ADR 0084 packet 039 owns the limit for each host.
- Studio keeps the items of a feed in its pointer file, so a moved feed can stay readable at its earlier URL.

## Required Changes

1. Read the tag in the channel read of the RSS check.
2. Compare the tag with the URL of the document. Lowercase the scheme and the host, and ignore one trailing `/`. An equal URL gives no candidate.
3. Fetch the candidate with the check path, through the host throttle. Do not follow the tag of the candidate.
4. Apply the proof of ADR 0088 Decision 2 in one function in `src/rss/check_apply.rs`.
5. Change `feeds.feed_url` in one `db` function, in one transaction with the check result. Clear the retained document of the earlier URL.
6. Add typed report items for a move and for a refused candidate. The view model gives the text. The report holds the two URLs, the feed title, the reason and the recorded time.
7. Apply the answers of the operator to the open questions of ADR 0088.

## Do Not Change

- No tag write. "Update n file(s)" stays the only write (ADR 0076 Decision 8).
- No use of the MusicIndex `feed_url` as a proof or as a candidate.
- No chain in one check.
- No schema change, unless an open question requires one.

## Acceptance Criteria

Mechanical:

- A test for each criterion of ADR 0088 "Verification", with a local test server.
- The check sends one request for the candidate, and no request for the tag of the candidate.
- A candidate on a private address follows the rule of `src/http_client.rs`. No test sends a request to a public host.
- The view model gives each report item with its two URLs and its recorded time.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: after a move in Studio, the RSS check report names the old and the new URL.
- V2: a later edit in Studio reaches the compare in v4vmm.

## Operator Visual Check

This check needs Studio and two file hosts for a test feed.

1. Publish a test feed in Studio, and subscribe to it in v4vmm.
2. Run `cargo run --bin v4vmm` in a desktop session.
3. Move the feed to the second host in Studio.
4. Run the RSS check of the playlist with that feed.
5. Wrong: the report does not name the two URLs, or the report names a different feed.
6. Change the title of a track of the feed in Studio, and publish it.
7. Run the RSS check again.
8. Wrong: the compare does not show the new title.

Cleanup: remove the subscription from the Library, and delete the test feed in Studio.

## Test Commands

- `cargo test --lib rss::`
- `cargo test --lib db::`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed, the tests run, and the answer that each open question applied.

## Stop And Ask

Stop if a different table or a path on disk uses the stored URL as a key. One transaction must update each key.
