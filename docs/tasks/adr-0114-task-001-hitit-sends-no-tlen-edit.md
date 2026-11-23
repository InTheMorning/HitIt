# ADR 0114 Task 001: HitIt Sends No `TLEN` Edit

## Status

Ready - 2026-11-22.

## Goal

HitIt sends no `TLEN` edit, and the scan of "Update n files" shows no `TLEN`
change. The crate `musicindex-tags` owns `TLEN`.

## Authority

[ADR 0114](../adr/0114-the-crate-owns-the-audio-length.md) Decisions 1 to 4
and its Invariant. Musicindex ADR 0093 R-TAG-11.
[ADR 0108](../adr/0108-the-tag-update-selects-each-change.md) Decision 3 for
the skip.

## Change

1. `id3v24_drag_copy_frame_is_writable` in `src/metadata.rs` gives `false` for
   `TLEN`. `auto_populated_pending_id3_edits` uses it, so
   `id3_edits_for_track_context` gives no `TLEN` edit. The download, the scan
   of "Update n files" and the compare grid use these functions.
   `pending_id3_edits_for_apply` removes each `TLEN` edit, so a manual drag
   onto the `TLEN` row writes nothing.
2. A comment at each change cites ADR 0114.
3. The `musicindex-tags` pin moves to musicindex-lib 1937944, which has
   R-TAG-11 (ADR 0114 Decision 4). A write keeps the `TLEN` of a file and
   gives the measured length to a file with no `TLEN`.
4. `src/audio_tags.rs` and `values_match` in
   `src/application/queries/tag_update.rs` do not change.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Writable frame check | extend | src/metadata.rs | `id3v24_drag_copy_frame_is_writable` controls the edit builder and the drag of the compare grid. |
| Stale skip delete | reuse | src/db/tag_update_skips.rs | The runtime deletes a stale skip after each scan. |
| Tag write | reuse | ~/build/musicindex-lib/tags/src/audio.rs | The crate owns `TLEN` (musicindex ADR 0093 R-TAG-11). |

## Acceptance Criteria

Mechanical:

1. A track with an RSS duration gives no `TLEN` edit from
   `id3_edits_for_track_context`, and `pending_id3_edits_for_apply` removes
   an explicit `TLEN` edit. Test
   `adr_0114_edit_builder_gives_no_tlen_edit`.
2. The scan gives no `TLEN` change for an MP3 file with `TLEN` `194185` and a
   stored duration of 187 s. Test
   `adr_0114_scan_shows_no_tlen_change`.
3. A stored skip of `TLEN` is stale after the scan. Test
   `adr_0114_scan_marks_a_tlen_skip_stale`.

Visual (manual, by the operator):

1. With the new binary, "Skipped tag changes (2)" goes away after the scan.
   The two Strange Love files keep `TLEN` `209484` and `194185`.

## Open

1. The visual check, after the operator starts the new binary.
