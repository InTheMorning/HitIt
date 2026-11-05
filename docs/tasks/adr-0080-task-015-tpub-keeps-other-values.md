# ADR 0080 Task 015: A Write Keeps A `TPUB` Value From Another Source

Status: Implemented 2026-11-05. Mechanical checks Green. It has no visual check.

## Goal

A write keeps each `TPUB` value of a file, and adds each confirmed label that the file does not hold.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decisions 6, 12 and 13, amended 2026-11-05.
- Open point 1 of the [run record](../reviews/orchestrated-run-2026-11-05.md). The operator accepted the recommendation on 2026-11-05.

## Recorded Facts - 2026-11-05

- The writer replaces the `TPUB` key, so a write removed a value from another source.
- A test recorded this result as an open point. A test also recorded the same result for `TLAN`.
- Before the write of 2026-11-05, 0 of the 231 Library files held a publisher, label or language value.

## Required Changes

1. `merge_label_edits` in `src/audio_tags.rs` gives one `TPUB` edit. It holds each file value, and then each label of the edits that the file does not hold.
2. `write_mp3_edits` and `write_lofty_edits` read the file values and call this function before each cleanup pass.
3. `values_match` in `src/application/queries/tag_update.rs` finds each expected label among the values of the file.
4. The `TLAN` test states the ruling of Decision 13. The earlier `TPUB` test is deleted, because the new test replaces it.

## Do Not Change

- No change to `TLAN`: a write replaces each other value.
- No change to the compare rows.
- No marker for the source of a value.

## Acceptance Criteria

Mechanical:

- `adr_0080_label_value_from_another_source_stays_on_mp3_flac_and_mp4`: the file holds the other value and the label. A second write gives equal fields, and the scan lists nothing.
- `adr_0080_label_merge_keeps_file_values_and_adds_one_edit`: NUL and " · " split a value, a held label does not repeat, and the result holds one edit.
- `adr_0080_language_write_over_a_foreign_tlan_value_on_mp3_flac_and_mp4`: the file holds only the new code.
- The commit checks of AGENTS.md pass.

Visual: none. The packet changes no screen. The tests read the written files. The Library holds no `TPUB` value from another source, so a walk has no file to read.

## Known Limit

The file holds no marker for the source of a value. Thus a label that the app wrote earlier stays when RSS no longer confirms it. A person must remove such a value with a different tool.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release`
- `cargo build --bin v4vmm`
