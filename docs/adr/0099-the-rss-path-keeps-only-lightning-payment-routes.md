# ADR 0099: The RSS Path Keeps Only Lightning Payment Routes

## Status

Accepted - 2026-11-15. The operator selected option A on 2026-11-15. The orchestrator session relayed the decision.

- Changes: [ADR 0076](0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 9. The stored route of that decision holds only Lightning routes.

## Context

[ADR 0076](0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 9 makes the stored route the source of each payment route in a file. The RSS path fills the stored route from the `podcast:value` block of a feed or an item.

[stophammer ADR 0098](../../../stophammer/docs/adr/0098-a-value-block-keeps-its-type-and-method.md) section 4 limits `include=payment_routes` to Lightning routes. The rule is in effect from Stophammer release 0.21.0. A typed block of a different payment system is available only through `include=value_blocks`.

An examination of HitIt at commit `635fc18` found these facts:

- `value_block_json` in `src/rss/helpers.rs` kept the first `podcast:value` element and did not read its `type`.
- `payment_routes_from_value_block` in `src/rss/value_routes.rs` kept each recipient and did not examine its type or its address.
- Thus a route of a `type="usdc"` block could go into `payment_routes_json`, into the file frame `TXXX:MusicIndex Value Routes` and into the drop file of the live publisher.
- A feed with a `usdc` block before its Lightning block lost its Lightning routes.
- HitIt has no payment step, so no code refused such a route.

The stored route, the file frame and the drop file have no field for a block type. A reader of these routes cannot tell a USDC address from a Lightning address.

## Decision

### 1. The parser keeps the first Lightning block

The RSS parser keeps the first `podcast:value` block of a feed or an item whose `type` is `lightning` or absent. It does not read a block of a different type.

A feed or an item with no such block has no stored block and no RSS payment route. Readiness then reports no RSS recipients.

### 2. The converter keeps only Lightning recipients

The converter of a stored block to the stored route keeps a recipient only in these conditions:

- Its type is `lnaddress`.
- Its type is `node` and its address has 66 hexadecimal characters.

A stored block of a different type gives an empty route. This is the rule of `is_lightning_route` in Stophammer, so the RSS path and `include=payment_routes` give the same set.

### 3. HitIt discards the data of other payment systems

HitIt does not store, write or publish a route of a payment system that is not Lightning. The database schema, the file frame and the drop file contract do not change.

### 4. A changed reader reads each stored feed again

`RSS_READER_REVISION` is 2. By ADR 0076 Decision 2, the next playlist RSS check of each stored feed sends no validators and reads the document with the new parser. This ADR adds no migration.

## Open Points

The orchestrator session or the implementing session resolved these points with the recommended option. They are not operator choices.

1. The converter refuses a `node` address that does not have 66 hexadecimal characters. Reason: a different rule gives a payment route difference in each RSS check.
2. No migration changes stored rows. A stored row changes at the next RSS check of its feed. Two classes of stored row do not change at that check:
   - A stored Lightning block with a recipient that Decision 2 discards. The block in the document is the same, so the check writes no route.
   - A track route that came from MusicIndex before Stophammer release 0.21.0, when the feed has no Lightning block. No RSS route replaces it.
3. The block type and the recipient type compare without case. A type of only spaces is an absent type.
4. The RSS comparison of two blocks reads each recipient of the Lightning block. A change in a discarded recipient gives a difference with no change of the stored route.

## Alternatives Considered

- Filter only in the converter. A feed with a `usdc` block before its Lightning block continues to lose its Lightning routes.
- Store each block with its type and method. This needs a schema change, a file frame change and a change of the drop file contract. It stays possible if the suite carries USDC routes at a later time.

## Consequences

The stored route, the file frame and the drop file hold only routes that a Lightning payment can use.
A feed with only a `usdc` block shows no RSS recipients, and its tracks are not ready for a show that needs a route.
The first playlist RSS check after this change sends one unconditional request for each feed of the playlist.
A file that holds a discarded route differs from the stored route after that check. It stays not ready until the operator confirms "Update n file(s)" (ADR 0076 Decision 9).
The route that MusicIndex gives is not filtered in HitIt. stophammer ADR 0098 owns that set.

## Verification

Mechanical, in `cargo test`:

- `adr_0099_parser_keeps_the_first_lightning_value_block` and `adr_0099_parser_reads_no_block_from_a_feed_with_only_a_usdc_block` in `src/rss/subscribe.rs` prove Decision 1.
- `adr_0099_block_that_is_not_lightning_gives_no_route`, `adr_0099_recipient_that_is_not_lightning_gives_no_route` and `adr_0099_discarded_recipient_with_an_invalid_split_does_not_fail_the_block` in `src/rss/value_routes.rs` prove Decision 2.
- The `adr_0076_reader_revision` tests in `src/runtime/playlist_rss_check.rs` prove Decision 4 for each value of the revision.

This ADR has no visual criterion.
