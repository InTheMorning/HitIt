# ADR 0089 Task 017: The Copy Reads Feeds With musicindex-fetch

## Status

Done - 2026-11-23. The main session ran the gate. The download part is
open. It waits for a change of `musicindex-fetch` (see Open Item).

- Agent choice, 2026-11-23: `musicindex-tags` and `musicindex-value` move
  to rev `5db5c80`, so the lock file has one version of each lib crate.
- Agent choice, 2026-11-23: the pin stays at `5db5c80`. The orchestrator
  names rev `40ef57a` for the gzip and Brotli decode after the operator
  pushes it.

## Goal

The copy of a public playlist reads feeds with the `FeedReader` of
`musicindex-fetch`. HitIt keeps no second copy of the reader or of the host
items (musicindex ADR 0092 R-STATION-1 item 5).

## Authority

- Musicindex ADR 0092 R-STATION-1 items 3 and 5.
- ADR 0089 Decision 1, packet 010, for the copy.

## Change

1. `Cargo.toml`: `musicindex-fetch` at `5db5c80`. `stophammer-parser` at
   `81fc50b`. `musicindex-tags` and `musicindex-value` at `5db5c80`.
2. `src/public_playlist_copy.rs`: the private `FeedReader` and
   `RSS_HOST_INTERVAL` are deleted. The copy calls `FeedReader::resolve`
   of the crate after the database lookup. Then it stores the body one time
   for each feed URL. An item with no feed URL gets its URL from MusicIndex
   by the feed GUID.
3. `src/http_client.rs`: `feed_fetch()` gives the crate `HttpFetch` with
   the HitIt user agent.
4. `src/rss/host.rs`: a re-export of `HostThrottled`, `feed_host` and
   `parse_retry_after` from the crate. The copy and its two tests are
   deleted.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Feed reader | reuse | Cargo.toml | `musicindex_fetch::FeedReader` replaces the private reader of the copy. |
| Host items | reuse | src/rss/host.rs | The module re-exports the crate items. |
| HTTP client values | extend | src/http_client.rs | `feed_fetch()` gives the crate fetch with the HitIt user agent. |
| Feed store | reuse | src/rss/subscribe.rs | `subscribe_feed_body` stores the body, as before. |

## Acceptance Criteria

Mechanical:

1. The five copy tests of ADR 0089 packet 010 pass with the crate reader.
   Tests `public_playlist_copy::tests::adr_0089_*`.
2. An item with no feed URL gets the URL from MusicIndex. Test
   `public_playlist_copy::tests::adr_0092_copy_gets_a_missing_feed_url_from_musicindex`.
3. An item that is not in its feed is skipped. Test
   `public_playlist_copy::tests::adr_0092_copy_skips_an_item_that_is_not_in_its_feed`.

Visual: none.

## Open Item

`download_enclosure` of HitIt stays. The crate download follows redirects
with the `reqwest` redirect layer. That layer does not follow a `Location`
value with a raw space. The HitIt redirect code encodes the space
(ADR 0056). The test
`track_compare::tests::follows_enclosure_redirect_instead_of_saving_redirect_body`
fails with the crate download and gives HTTP 301. The lib session owns the
change of the crate.
