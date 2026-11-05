# ADR 0076 Task 011: A New RSS Reader Reads Each Stored Feed One Time

Status: Implemented 2026-11-05. Mechanical checks Green. Operator visual check open.

## Goal

When the app reads a new value from an RSS document, the next Check RSS of each stored feed reads its full document one time. An unchanged feed then does not hide a new value behind HTTP `304`.

## Authority

- [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 2, amended 2026-11-05.
- [ADR 0016](../adr/README.md): the migration registry. This packet needs no schema change.
- [AGENTS.md](../../AGENTS.md): "Every fix gets a guard".

## Recorded Facts - 2026-11-05

- The operator saw "Documents received: 0. Not modified: 1." and `HTTP 304` for one feed in Check RSS, after the ADR 0080 packets.
- `check_feed` in `src/runtime/playlist_rss_check.rs` reads the validators with `read_request_validators`, and `NotModified` applies nothing.
- A local metadata fact key is free text, as ADR 0080 packet 010 proved.
- The guard `adr_0075_projection_views_combine_no_fact_and_column` forbids `db::local_metadata_fact(` in `src/rss/check_apply.rs`. Thus the read goes in `src/local_metadata.rs`, as `rss_license_facts` does.

## Required Changes

1. Add `RSS_READER_REVISION` in `src/rss/check_apply.rs`, with the value `1`. Its comment states the rule: a change that reads a new value from an RSS document raises this value.
2. Add the fact key `rss_reader_revision` in `src/local_metadata.rs`, and a reader for the revision of one feed. A missing or unreadable value gives `0`.
3. In `check_feed`, read the revision with the validators. When the revision is lower than `RSS_READER_REVISION`, pass `None` as the validators.
4. `apply_checked_document` writes the current revision as an `rss` fact of the feed, in its transaction. A document that does not apply stores no revision, so the next check sends no validators again.
5. `record_subscribed_document` writes the current revision.

## Do Not Change

- No schema change. No new control and no new report text.
- `read_request_validators` keeps its result, and `304` keeps its meaning.
- The stored validators stay. A response with `200` replaces them as it does today.
- No screen shows the new fact.

## Acceptance Criteria

Mechanical:

- `adr_0076_playlist_check_stored_validators_reach_the_second_request` stays true for a feed with the current revision.
- A feed with stored validators and no revision gets a request with no validators.
- A feed with a lower revision gets a request with no validators.
- After a document applies, the next request of that feed carries the validators.
- After HTTP `304`, the stored revision does not change.
- After a document that does not apply, the next request carries no validators.
- A subscribed feed holds the current revision, so its first check carries the validators.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: the first Check RSS of a playlist after this packet shows "Documents received" for each reachable feed. A second Check RSS shows "Not modified" for a feed that answers `304`.
- V2: after the first Check RSS of a feed with `podcast:license`, "Update n files" lists `WCOP` or `TXXX:MusicIndex License` for its tracks.

## Operator Visual Check

1. Run `cargo build --bin v4vmm`, and then start `./target/debug/v4vmm` in a desktop session.
2. Select **Music**, then **Playlists** in the left sidebar, and click a playlist in the list column.
3. Click **Check RSS** on the playlist page. Wait for the **RSS check** report to stop running.
4. Wrong: the report shows "Not modified" for a feed in this first check.
5. Click **Check RSS** again, and wait.
6. Wrong: the report shows "Documents received" for a feed that answered `304` before. A feed whose server sends no validators can correctly show "Documents received" again.

The first check of each feed can report differences, because the readers of ADR 0080 packet 008 read the document for the first time.

Cleanup: none. The stored revision is correct.

## Test Commands

- `cargo test --lib playlist_rss_check`
- `cargo test --lib rss::`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed, the tests run, and the requests that each test proves.

## Stop And Ask

Stop if a guard refuses the read or the write of the new fact.

Stop if a screen or a compare row starts to show the new fact.
