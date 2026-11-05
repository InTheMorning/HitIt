# ADR 0080 Task 012: The Language In `TLAN`

Status: Implemented 2026-11-05. Mechanical checks Green. Operator visual check open.

## Goal

A file holds the language of its track or of its feed in `TLAN`, as a three-letter code.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decisions 3, 4, 6 and 13.
- `id3v24_musicbrainz_rss_mapping.html` is the advisory map: `channel/language` goes to `TLAN`, with ISO 639-2 codes.

## Recorded Facts - 2026-11-04

- `Track` and `Feed` decode `language`. `src/views.rs` shows it, and `src/local_metadata.rs` stores it as a fact.
- `id3_frame_hint` has no row for language, and `source_value_for_metadata_field` has no value for it.
- `TLAN` is a writable text frame. `TagFieldId` has no typed value for it, so `Id3Text("TLAN")` gives no Vorbis key.
- A live feed gives `language` `en` on 2026-11-04.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) gives the display rules: fresh RSS comes before MusicIndex, and a screen shows the feed value as a separate "Feed language" value. A file has no such separate value, so ADR 0080 Decision 13 resolves one value for a file, as Decision 1 does. Screens do not change.

## Required Changes

1. Add a const table in a new `src/language_code.rs`. It maps each ISO 639-1 code to its ISO 639-2/T code. Add a module comment that names ADR 0080 Decision 13. Add no dependency.
2. A function takes the RSS value, trims it, lowers it, and takes the subtag before `-` or `_`. A two-letter subtag uses the table. A three-letter subtag of letters stays as it is. Each other value gives `None`.
3. Add the compare row "Language": the item value, otherwise the channel value, as the code. Its frame is `TLAN`.
4. Add `TagFieldId::Language`, with the Vorbis key `LANGUAGE` and the matching MP4 freeform key. Follow `TagFieldId::Publisher`.
5. The compare uses the value of the writer (Decision 3). The source text stays in the database, and the compare shows the code.

## Do Not Change

- No change to how a screen shows a language.
- No `Explicit` frame. ADR 0080 keeps that row with no frame.

## Acceptance Criteria

Mechanical:

- `en`, `EN`, `en-us` and `en_GB` give `eng`. `de` gives `deu`. `fra` gives `fra`.
- An empty value, `english` and `xx` give no frame.
- The tag edits hold the item language. Without it, they hold the channel language.
- A round-trip test: freshly built edits give no difference at the compare.
- A second write gives equal frames.
- A FLAC file and an MP4 file hold the value under the new key, and show no difference at the next scan.
- The table test: the table has each ISO 639-1 code, with one three-letter value.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: after "Update n files", an external tag reader shows `eng` as the language of a track of an English feed.

## Operator Visual Check

1. Run `cargo run --bin v4vmm` in a desktop session.
2. Open a Library track of a feed with `<language>en</language>`. Open "Update n files".
3. Wrong: the list does not show `TLAN: eng`.
4. Confirm the update. Run `ffprobe -hide_banner FILE`, with FILE as the path of the track.
5. Wrong: the output does not show `eng`.
6. Run the scan again. Wrong: the file shows a difference.

Cleanup: none. The tag rewrite is correct.

## Test Commands

- `cargo test --lib language_code`
- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed, the tests run, and the source of the table.

## Stop And Ask

Stop if `docs/schema/adr-0075-field-rules-artist-language-dates.md` states a language rule that this packet contradicts.
