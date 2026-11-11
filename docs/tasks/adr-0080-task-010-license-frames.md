# ADR 0080 Task 010: The RSS License In A File

Status: Implemented 2026-11-05. Mechanical checks Green. Operator visual check open.

## Goal

The RSS license reaches a file. Its URL goes into `WCOP` or `TXXX:LICENSE`, and its text goes into `TXXX:MusicIndex License`.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decisions 3, 4, 6, 8 and 11.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) Decision I: RSS values and MusicBrainz values stay apart.
- [ADR 0054](../adr/0054-local-metadata-source-fact-persistence.md): feed and track metadata facts persist by source.

## Recorded Facts - 2026-11-04

- Studio writes `podcast:license` on the channel and on each item, with its text and an optional `url` (`License` in `crates/core/src/feed/elements.rs` of Studio).
- The Stophammer parser does not read the tag, and the API gives no license. Open request 12 asks for it. This packet does not wait for that request, because HitIt reads the RSS document.
- `src/rss/` of HitIt does not read the tag.
- `musicbrainz_license_row` in `src/metadata.rs` writes one MusicBrainz license to `WCOP`, and more than one to `TXXX:LICENSE`.
- HitIt stores no MusicBrainz lookup result, so a write that finds a `WCOP` value cannot name its source.
- The Studio corpus holds these forms: the text `All rights reserved` with no `url`, and the text `my-podcast-license-v1` with a `url`.

## Required Changes

1. Read `podcast:license` of a channel and of an item in the subscribe reader and in the RSS check. Store the text and the URL as local metadata facts with new `fact_key` values. The keys need no migration, as `language` and `explicit` need none. Confirm this in `src/local_metadata.rs`.
2. Add the fields to `Track` and `Feed` as app-only values, and read them in `src/feed_service.rs`. The contract field guard must stay Green, so read how that guard treats a field that the contract does not declare. Use a separate carrier if the guard refuses such a field.
3. Add the compare rows "License" and "License name" for the RSS value. Keep `musicbrainz_license_row` for the MusicBrainz value. Show the two sources on their own rows.
4. Add the resolved edits in `id3_edits_for_track_context`: the item value, otherwise the channel value.
5. Writer rule for the URL, in `src/audio_tags.rs`:
   - The file holds no license URL: write `WCOP`.
   - `WCOP` holds the same URL: change nothing.
   - Packet 016 replaced this step: the URL of the write replaces each `WCOP` and `TXXX:LICENSE` value.
6. The text goes into `TXXX:MusicIndex License`, and into the `MUSICINDEX LICENSE` custom key of a Vorbis or MP4 file.
7. The compare counts the RSS URL as equal when `WCOP` or `TXXX:LICENSE` holds it.

## Do Not Change

- No `USER` frame and no `TCOP` frame (ADR 0080 "Alternatives Considered").
- No license display on a screen, other than the compare rows.

## Acceptance Criteria

Mechanical:

- The tag edits for a track with its own license hold its URL and its text. Without it, they hold the channel values.
- A license with text and no `url` gives only `TXXX:MusicIndex License`.
- A `url` that is not an `http` or `https` URL gives no URL frame.
- A file with a different `WCOP` holds only the RSS URL in `WCOP` after one write, and no `TXXX:LICENSE`.
- A second write gives equal frames.
- The compare of each state above reports no difference after the write.
- A MusicBrainz license stays on its own row, and its write follows ADR 0080 Decision 8.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: the compare grid shows the RSS license and a MusicBrainz license on their own rows.
- V2: after "Update n files", an external tag reader shows the license URL and the text.

## Operator Visual Check

1. Add `<podcast:license url="https://creativecommons.org/licenses/by/4.0/">cc-by-4.0</podcast:license>` to a test feed in Studio. Publish the change.
2. Run `cargo run --bin hitit` in a desktop session.
3. Run the RSS check of the playlist with that feed. Open "Update n files".
4. Wrong: the list does not show `WCOP` and `TXXX:MusicIndex License`.
5. Confirm the update. Run `exiftool -a -G1 -s FILE`, with FILE as the path of the track.
6. Wrong: the output does not show the URL in `CopyrightURL` and the text in `MusicIndexLicense`.
7. Run the scan again. Wrong: the file shows a difference.

Cleanup: remove the license from the test feed in Studio, and publish it.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test --lib rss::`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin hitit`

## Report

Give the files changed, the tests run, and the frames of each writer state.

## Stop And Ask

Stop if local metadata facts need a schema change. Schema version 21 is current, and the migration registry of ADR 0016 owns each change.
