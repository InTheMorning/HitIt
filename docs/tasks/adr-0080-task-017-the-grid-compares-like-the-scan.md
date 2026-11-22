# ADR 0080 Task 017: The Grid Compares Like The Scan

## Status

Done - 2026-11-22, except the visual check. The main session ran the gate.

- Agent choice, 2026-11-22: criteria 2 and 3 have no separate tests. The
  test `adr_0080_task_018_downloaded_file_stages_no_edit` covers them on the
  real path: the file has `TDRC` and no `TYER`, and the grid stages no edit.

## Goal

The compare grid ("Compare ID3") and the scan of "Update n files" give the
same result for the track number and the release year. The grid stages no
`TYER` edit for a file that has a date.

## Authority

[ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 3: the tag
compare checks each frame against the value that the writer would write.
Musicindex ADR 0090 R-TAG-8 item 1: an MP3 write gives one `TDRC` frame and
no `TYER` frame.

- Operator choice, 2026-11-22: the track number compares its number only. A
  track total, such as the 6 of "1/6", is not a difference. The RSS total is
  the episode count of the feed, so a feed that grows changes it. A compare
  of the total would cause a rewrite of each earlier file.

## Recorded Facts

From the operator screen of 2026-11-22, Strange Love "Dirt and Sand", an MP3
file with an ID3v2.4 tag:

- The "Track #" row shows the RSS value "1/6", the frame `TRCK` and the file
  value with the orange marker "≠ 1". The scan shows no `TRCK` change.
- The "Release year" row shows the frame `TYER` with a border and the value
  "= 2026". A border means a staged edit
  (`src/ui/shells/library/track_detail_metadata_cells.rs:220`). The file has
  no `TYER` frame.

From a reading of the code on 2026-11-22:

- The grid uses `compare_id3_field_values` in `src/metadata.rs`. It compares
  the full text of "Track #".
- The scan uses `values_match` in `src/application/queries/tag_update.rs`.
  It compares `TRCK` and `TPOS` by `leading_number`.
- `aligned_compare_rows` in `src/metadata.rs` gets the file value of the
  "Release year" row from `id3_value_for_field("Release year", result)`. With
  no `TYER` frame, the value is empty, so `auto_populated_pending_id3_edits`
  stages the RSS year.
- The scan treats `TYER` as the year of the one file date (`year_matches`).

## Change

1. One function in `src/metadata.rs` gives the number compare of a track or
   disc number: two values are equal when their leading numbers are equal.
   `values_match` uses it for `TRCK` and `TPOS`, and `leading_number` in
   `tag_update.rs` goes. `compare_id3_field_values` uses it for the fields
   "Track #" and "Disc #", if the grid has a "Disc #" field.
2. In `aligned_compare_rows`, when the file has no `TYER` value, the file
   value of the "Release year" row is the year of the file date. Use
   `release_year_from_value` on the file value of "Release date". The cell
   is then not empty, so the grid stages no `TYER` edit.
3. A comment at each change cites ADR 0080 Decision 3 and this task.

## Do Not Change

- The edit builder (`id3_edits_for_track_context`), the download write and
  the scan rules for `TDRC` and `TYER`. The tests at
  `src/application/queries/tag_update.rs` lines 1453, 1503 and 1515 stay as
  they are.
- `format_track_slash_total`, `format_slash_number_frame` and the display
  text of the "Track #" row. The grid continues to show "1/6".
- `src/assets/tokens.json` and the musicindex-lib crate.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Number compare of a track or disc number | extend | src/application/queries/tag_update.rs | `leading_number` moves to `src/metadata.rs` as one function for the scan and the grid. |
| Grid compare | extend | src/metadata.rs | `compare_id3_field_values` uses the shared number compare for "Track #". |
| Year of a date | reuse | src/metadata.rs | `release_year_from_value` gives the year of the file date. |
| Grid auto-stage | reuse | src/metadata.rs | `auto_populated_pending_id3_edits` stages only empty cells, so a filled "Release year" cell gets no edit. |

## Acceptance Criteria

Mechanical:

1. `compare_id3_field_values("Track #", Some("1/6"), Some("1"))` gives
   `ComparisonStatus::Match`. With `Some("2/6")` and `Some("1")` it gives
   `ComparisonStatus::Different`. Test
   `adr_0080_task_017_track_number_compares_its_number`.
2. Use a file with `TDRC` "2026-05-21", no `TYER` and an RSS date in 2026.
   For it, `aligned_compare_rows` gives the "Release year" row the file
   value "2026" and the status `ComparisonStatus::Match`. Test
   `adr_0080_task_017_release_year_reads_the_file_date`.
3. `auto_populated_pending_id3_edits` for the rows of criterion 2 gives no
   edit for the frame `TYER`. Test
   `adr_0080_task_017_grid_stages_no_tyer_for_a_file_date`.
4. Each existing test passes with no change to its assertions.

Visual (manual, by the operator):

1. On "Dirt and Sand", `Compare ID3` shows "= 1/6" or "= 1" on the "Track #"
   row, and the "Release year" row has no border.

## Checks

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --bin hitit`

## Rules For The Delegate

- Write each comment in Simplified Technical English. The skill is at
  `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it.
- Write each failing test first, and confirm that it fails before the change.
- If criterion 4 needs a change to an existing assertion, stop and report the
  test name and the reason.
