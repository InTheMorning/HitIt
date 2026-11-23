# ADR 0080 Task 019: People Frames Read Back The Same

## Status

Done - 2026-11-22. The main session ran the gate.

- Visual check 1 passed, 2026-11-22: the operator used the release binary
  of 3e7210a on "What You Mean To Me" by Strange Love. The closed
  "Contributors" row showed "= 3 contributors" in the file column. The
  operator did not download the track again.
The `musicindex-tags` pin is musicindex-lib commit e2e8a20, with the public
builder `involved_people_value`.

- Orchestrator choice, 2026-11-22 (the operator lets the orchestrator make
  choices for suite efficiency): `musicindex-tags` owns the text of `TIPL`
  and `TMCL` (musicindex ADR 0086 R-TAG-1 item 4). HitIt did not change its
  `TMCL` join to " / " as an interim step.
- Agent choice, 2026-11-22: only the download shows the crate error, as a
  part of its tag warning. The download writes with no review by the user.
  The grid and the scan of "Update n files" have no warning text. For such a
  track they stage no `TMCL` edit, so they write no bad frame.
- Agent choice, 2026-11-22: `merge_joined_values` continues to join the
  `TMCL` text of two performer rows with " / ". Each part is a text from the
  crate builder.
- Fact, 2026-11-22: the crate accepts the name "Alex Nicol /", because it
  reads back the same. The error test uses the name "/ Alex Nicol", which
  makes " / " after the ": " of its pair.

## Goal

A file that HitIt wrote shows no difference on the "Contributors" row at the
next compare. The row shows its compare marker. A `TMCL` frame with two persons
on one instrument reads back as two pairs.

## Authority

[ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 3: the
compare checks each frame against the value that the writer would write. A
file that the app wrote shows no difference at the next compare.
Musicindex ADR 0086 R-TAG-1 item 4: the crate owns the text of `TIPL` and
`TMCL`.

## Recorded Facts

The operator screen of 2026-11-22 shows Strange Love "Dirt and Sand", a file
that HitIt downloaded. The "Contributors" row shows "3 contributors" for RSS
and "5 contributors" for the file. The row shows no marker.

From a round-trip test print on 2026-11-22 (the fixture of
`adr_0080_task_018_downloaded_file_stages_no_edit`, with two vocal credits):

- The writer sent `TMCL="bass: Josh Moroz / drums: Alex Nicol / guitar:
  Anthony Columbro / keyboards: Anthony Columbro / vocals: Josh Moroz ·
  vocals: Alex Nicol"`.
- `format_tmcl_value` (`src/metadata.rs`) joins the pairs of one instrument
  with " · " through `join_values`. The crate function
  `parse_involved_people_list` (`tags/src/audio.rs`, rev 1937944) splits the
  pairs on " / ", and then each pair on ": ". The file gets one pair with the
  role "vocals" and the name "Josh Moroz · vocals: Alex Nicol".
- The "Contributors" row compared the RSS credits with the union of
  `TXXX:Podcast Contributors`, `TMCL`, `TIPL`, `TPE1`, `TPE2` and other
  people frames (`id3_contributor_values`). The status was `Different`.
- The expandable cell shows the summary "n contributors" and not the
  display value, so the marker does not show (`expandable_cell_summary`).

## Change

### Part A: The Two Sides Read The Same Source

1. The file side of the "Contributors" row reads only
   `TXXX:Podcast Contributors`. The writer fills this frame from the RSS
   credits (`musicindex_contributors_id3_value`), and the RSS side reads the
   same credits.
2. Remove the special "Contributors" path from `id3_value_for_field`,
   `id3_compare_value_for_field` and `grouped_id3_frame_keys`. Remove each
   function that no entry point then reaches.
3. A people frame that no row reads, for example `TCOM` with no RSS composer,
   shows as its own frame row in its group.

### Part B: The Summary Keeps Its Marker

1. `TrackMetadataGridVm::expandable_cell_summary` takes the compare glyph and
   puts it before the summary, as `display_with_glyph` does for other cells.
2. The library cell passes the display value with no glyph and the glyph
   apart, so a fallback summary does not show the glyph two times.

### Part C: The Crate Builds `TMCL` (After The Pin)

1. Move the `musicindex-tags` pin to the commit that the orchestrator sends.
2. Remove `format_tmcl_value`. The `TMCL` edit text comes from the crate
   builder of (role, name) pairs.
3. When the builder gives an error, HitIt shows the error to the user and
   writes no `TMCL` frame. HitIt does not write a bad frame.

## Existing Files

A file that HitIt wrote before this task keeps its bad `TMCL` pair until
HitIt writes the file again. The grid then shows a difference on the
performer row, because the RSS side gives one pair for each person.

## Open

- `TXXX:Podcast Contributors` also uses " / " between entries. A credit name
  that contains " / " does not read back the same. The orchestrator decides
  the owner of this text subsequently.

## Do Not Change

- The writer of `TXXX:Podcast Contributors` and the scan of "Update n files".
- The rows "Artist", "Album artist", "Involved people" and the performer rows.
- `src/assets/tokens.json` and the musicindex-lib crate.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Frame value of a grid field | reuse | src/metadata.rs | `id3_value_for_field` reads `TXXX:Podcast Contributors` through its `TXXX` path with the needles of "Contributors". |
| Contributor compare | reuse | src/metadata.rs | `normalized_contributors_compare_value` compares the two serialized lists. |
| Expandable cell summary | extend | src/view_models/track_metadata_grid.rs | `expandable_cell_summary` gets the glyph. |
| Glyph text | reuse | src/view_models/track_metadata_grid.rs | `display_with_glyph` puts the glyph before the value. |
| Involved people text | reuse | src/metadata.rs | Part C uses the crate builder and deletes `format_tmcl_value`. |

## Acceptance Criteria

Mechanical:

1. Part A: for a file that `id3_edits_for_track_context` wrote with RSS
   credits and an artist, `aligned_compare_rows` gives the "Contributors" row
   the status `ComparisonStatus::Match`. Test
   `adr_0080_task_019_written_file_matches_its_contributors`.
2. Part A: the file value of the "Contributors" row is the value of
   `TXXX:Podcast Contributors` only. A `TPE1` value is not in it. Test
   `adr_0080_task_019_contributors_reads_one_frame`.
3. Part B: `expandable_cell_summary("Contributors", raw, display, Some("≠"),
   ...)` gives "≠ 3 contributors" for a raw value with three names. A
   fallback summary has the glyph one time. Test
   `adr_0080_task_019_summary_keeps_its_marker`.
4. Part C: a write of a `TMCL` with two persons on one instrument reads back
   through the crate as two pairs. Test
   `adr_0080_task_019_tmcl_two_people_read_back`.
5. Part C: a builder error gives a user message and no `TMCL` edit. Test
   `adr_0080_task_019_tmcl_builder_error_writes_no_frame`.
6. Each existing test passes. A changed assertion is listed in the report
   with its cause.

Visual (manual, by the operator):

1. After part A and part B, on "Dirt and Sand", the "Contributors" row shows
   "= n contributors" on the file side.

## Checks

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --bin hitit`
