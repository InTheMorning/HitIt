# ADR 0080 Task 013: An MP4 Date Settles After One Write

Status: Ready - 2026-11-04. The operator visual check stays open until a person walks it.

## Goal

An MP4 file with a release date settles after one "Update n files" write. The next scan shows no `TYER` difference.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 6: a write is idempotent.
- [ADR 0080 task 003](archive/adr-0080-task-003-vorbis-date-shares-one-key.md) corrected the same defect for Vorbis. Read it first, and follow its method where it applies.

## Recorded Facts

From the [ADR 0080 plan](../plans/adr-0080-tag-frames-phase-plan.md), 2026-10-03 and 2026-10-16:

- The writer stores `TDRC` and `TYER` in two freeform atoms on an MP4 file. The reader gives both values as `TDRC`.
- Thus the scan always reports `TYER` on an MP4 file with a date. The reader label does not name the atom, so a key comparison cannot correct it.
- The operator saw `TYER: 2025 (file: 2025-10-19)` and `TYER: 2025 (file: 2025-10-03)` on two files, "Ethereal Realm ( 528hz )" and "Whispers". It is not confirmed that the two files are MP4 files, or that a write changes the stored date.

From a reading of the code on 2026-11-04:

- `vorbis_storage_key` in `src/application/queries/tag_update.rs` gives the scan the storage key of a Vorbis label.
- `TagFieldId::from_id3_label` maps `TDRC` and `TYER` to `TagFieldId::Date`.

## Required Changes

1. Write a failing test first: an MP4 file written with `TDRC` and `TYER` edits, then scanned. Confirm the recorded behavior. If the test does not fail, stop and report it.
2. Make the date of an MP4 file one value. A write stores one date atom. The scan treats `TDRC` and `TYER` as that one value, with a year equal when the file date starts with it.
3. A file with two earlier date atoms keeps one after one write.
4. Find the format of the two files that the operator named, if the Library holds them, through a read-only operator command. Add the result to the report. If they are not MP4 files, the 2026-10-16 observation is a different defect, so report it.

## Do Not Change

- No change to the ID3 path of `TDRC` and `TYER`.
- No change to the Vorbis path that packet 003 corrected.

## Acceptance Criteria

Mechanical:

- The test of Required Change 1 passes after the change.
- An MP4 file with a date shows no `TYER` difference after one write.
- A second write gives equal atoms.
- A file with two earlier date atoms holds one after a write.
- The ADR 0080 packet 003 tests pass.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: an MP4 file leaves the "Update n files" list after one write, and stays out at the next scan.

## Operator Visual Check

1. Find one MP4 file in the Library: run `sqlite3 -readonly DB "SELECT file_path FROM tracks WHERE file_path LIKE '%.m4a' LIMIT 1"`, with DB as the path of the app database. Read the column name in the schema if it differs.
2. Run `cargo run --bin v4vmm` in a desktop session.
3. Open "Update n files" on the playlist of that file.
4. Confirm the update. Run the scan again.
5. Wrong: the file still shows `TYER` or `TDRC`.
6. Run `ffprobe -hide_banner FILE`, with FILE as the path of the file.
7. Wrong: the output shows no date, or two different dates.

Cleanup: none. The tag rewrite is correct.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed, the tests run, and the result of Required Change 4.

When this packet is implemented, change the "Known open defects" line in `AGENTS.md`.

After the operator check passes, remove the "Open Defect" section of the ADR 0080 plan.

## Stop And Ask

Stop if a player or the broadcast producer reads the `TYER` atom of an MP4 file. Search `src/broadcast/` first.
