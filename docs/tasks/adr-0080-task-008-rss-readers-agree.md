# ADR 0080 Task 008: The RSS Readers Agree On Track Number And Artwork

Status: Implemented 2026-11-05. Mechanical checks Green. Operator visual check open.

## Goal

The subscribe reader and the enrichment reader give one track number and one artwork URL for one document. They apply the artwork sequence of ADR 0075.

## Authority

- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md), "Feed-artwork extraction order" and "Track-artwork extraction order", with the amendment of 2026-11-04 for `purpose`.
- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 9: `TXXX:MusicIndex Image` holds the item image, otherwise the channel image.
- Studio ADR 0079 writes `podcast:episode` and `itunes:episode` together. Studio ADR 0060 keeps the cover form of a feed.

## Recorded Facts - 2026-11-04

| Value | `src/rss/subscribe.rs` | `src/rss/enrich.rs` | Stophammer parser |
|---|---|---|---|
| Track number | `podcast:episode` only (line 211) | `podcast:episode` only (line 944) | `itunes:episode`, then `podcast:episode` |
| Channel image | `podcast:image`, `itunes:image`, `<image>` (line 128) | `itunes:image`, `<image>` (line 924) | `itunes:image`, `<image>` |
| Item image | `itunes:image` (line 218) | Read it in this packet | `itunes:image` |

- `find_ext_attr` returns the first `podcast:image`, with no check of `purpose`.
- The Studio corpus holds `purpose` values `artwork`, `banner`, `icon` and `social`. Studio models only the `href` of a `podcast:image`, and a read keeps each other attribute.
- A feed with only `itunes:episode` gives no track number in HitIt. The same feed gives a track number from the API.

## Required Changes

1. Add one helper in `src/rss/helpers.rs` that gives the first artwork `podcast:image`. It applies the `purpose` rule of ADR 0075: no `purpose`, or a `purpose` that holds the word `artwork`.
2. Channel image: `src/rss/subscribe.rs` and `src/rss/enrich.rs` apply `podcast:image`, `itunes:image`, then `<image>`.
3. Item image: the two readers apply `podcast:image`, then `itunes:image`.
4. Track number: the two readers apply `podcast:episode`, then `itunes:episode`. Keep `podcast:episode` first, because the code states it as the canonical sequence. Report a document where the two values differ only if a test shows that the app needs it.
5. Report conflicting ties as ADR 0075 states only where the code reports them today. This packet adds no new report surface.

## Do Not Change

- No change to the Stophammer parser. The API value and the RSS value stay separate facts (ADR 0075 Decision I).
- No change to the frame names or to the writer.
- No database migration.

## Acceptance Criteria

Mechanical:

- A test with one document runs each reader, and gives equal track number, channel image and item image.
- A channel with `podcast:image purpose="banner"` first and an `itunes:image` gives the `itunes:image`.
- A channel with `podcast:image purpose="artwork social"` gives that `podcast:image`.
- A channel with `podcast:image` and no `purpose` gives that `podcast:image`.
- A DeMu form channel, with only `<image><url>`, gives that URL.
- An item with only `itunes:episode` gives that number.
- An item with both elements gives `podcast:episode`.
- The ADR 0080 image tests pass.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: a feed with a banner `podcast:image` shows its cover on the release page, not the banner.

## Operator Visual Check

1. In a desktop session, run `cargo run --bin hitit`.
2. Subscribe to a test feed with `<podcast:image purpose="banner" href="...">` before its `itunes:image`. Use a feed that you host, because the feed needs this exact sequence.
3. Open its release page.
4. Wrong: the page shows the banner as the cover.
5. Open a track of that feed with no image of its own. Open "Update n files".
6. Wrong: `TXXX:MusicIndex Image` holds the banner URL.

Cleanup: remove the test subscription from the Library.

## Test Commands

- `cargo test --lib rss::`
- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin hitit`

## Report

Give the files changed, the tests run, and each reader that changed its result.

## Stop And Ask

Stop if the `rss` crate does not keep the `purpose` attribute of each `podcast:image`.

Stop if a change of the channel image in `src/rss/enrich.rs` gives a new conflict in an existing provider test.
