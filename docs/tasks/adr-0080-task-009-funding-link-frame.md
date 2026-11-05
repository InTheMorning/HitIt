# ADR 0080 Task 009: The Funding Link In `WPAY`

Status: Ready - 2026-11-04. It needs packet 006. The operator visual check stays open until a person walks it.

## Goal

A file holds one funding link in `WPAY`. The database keeps each funding link with its owner and its label.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decisions 3, 4, 6, 8 and 10.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) section 4: a feed fact never becomes a track assertion in storage.
- Stophammer ADR 0082 owns the `funding` rows of `source_links`.
- [AGENTS.md](../../AGENTS.md): "Secrets", for the rule that RSS and MusicIndex responses are untrusted input.

## Recorded Facts - 2026-11-04

- Three Studio corpus feeds with the tag give one `funding` row each from the live API: `phafe-music-the-singles`, `jimmyv4v-music-jimmy-v-music` and `behindthesch3m3s-music-pixel-palace`.
- Stophammer ADR 0082 gives a null `url` for a funding value with no `http` or `https` scheme.
- `persist_source_links` in `src/identity_ingest.rs` stores `raw_json` of each link. After packet 006, that text holds `label`. No new column is necessary.
- `source_link_from_local` in `src/feed_service.rs` does not read `raw_json`.
- `rss_feed_link_inputs` and `rss_track_link_inputs` in `src/rss/subscribe.rs` write the website, the page and the transcript links. They write no funding link.
- `website_from_links` and `transcript_from_links` in `src/metadata.rs` select by `link_type`. A funding row does not enter `WOAR` or `WOAF`.
- `WPAY` is a writable URL frame. `UrlKind::Payment` maps to `PAYMENTWEBPAGE`.
- `WEBSITE_FRAME_LABELS` in `src/audio_tags.rs` holds only `WOAR` and `WOAF`.

## Required Changes

1. Read each `podcast:funding` of a channel and of an item in the subscribe reader and in the RSS check.
   - Write one link input for each tag.
   - Use `link_type` `funding` and the `extraction_path` of Stophammer ADR 0082.
   - Keep the node text in `raw_json`.
2. Read `label` from `raw_json` in `source_link_from_local`. A row with no label gives `None`.
3. Add a `funding_url_from_links` function in `src/metadata.rs`. It gives the first row with `link_type` `funding` and a URL that `format_id3_website_url` accepts.
4. Add two compare rows, "Funding" and "RSS feed funding", as "Website" and "RSS feed website" are. Each row shows the value of its owner.
5. Add one resolved edit in `id3_edits_for_track_context`: the item value, otherwise the channel value, in `WPAY`. Follow the resolution of the Nostr key in `apply_resolved_nostr_key`.
6. The compare of `WPAY` uses the same resolution (Decision 3).
7. A write replaces the `WPAY` value that the app supplied, and keeps one frame. A `WPAY` value that no source supplied stays when RSS supplies no funding link. When RSS supplies a link, the frame holds that link, because ID3v2.4 permits one `WPAY`. Decision 10 states this limit of Decision 6.

## Do Not Change

- No database migration.
- No funding action or link on any screen. How an app shows a funding link needs a shared rule of `../musicindex`. Stophammer ADR 0082 section 5 leaves it to the apps.
- No label in a frame.

## Acceptance Criteria

Mechanical:

- The tag edits for a track with an item funding link hold it in `WPAY`.
- Without it, they hold the first channel funding link.
- A value with no `http` or `https` scheme gives no frame, and the next channel value is used.
- A round-trip test: freshly built edits give no difference at the compare.
- A second write gives equal frames, and one `WPAY` frame.
- A funding row gives no `WOAR`, `WOAF` or transcript value.
- A label from the API and a label from RSS both survive a store and a read of the local identity link.
- A FLAC file and an MP4 file hold the value under the key of `UrlKind::Payment`, and show no difference at the next scan.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: "Update n files" lists `WPAY` for a track of a feed with a funding link.
- V2: after the update, an external tag reader shows the URL with no label text, and the next scan shows no difference.

## Operator Visual Check

1. Run `cargo run --bin v4vmm` in a desktop session.
2. Open a Library track of a feed with `podcast:funding`. `phafe-music-the-singles` has one, if you have that feed.
3. Run the RSS check of its playlist. Open "Update n files".
4. Wrong: the list does not show `WPAY`, or its value holds label text.
5. Confirm the update. Run `ffprobe -hide_banner FILE`, with FILE as the path of the track.
6. Wrong: the output has no payment URL, or it holds label text.
7. Run the scan again. Wrong: the file shows a difference.

Cleanup: none. The tag rewrite is correct.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test --lib identity_ingest`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed, the tests run, and the frame output for one sample feed.

## Stop And Ask

Stop if a screen starts to show the funding rows as a side effect of this packet.
