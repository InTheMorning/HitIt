# Musicindex ADR 0086 Task 001: The Tag Crate And The Retag

## Status

In progress - 2026-11-22. Commits 942baf8, 49fcef5, 2440498 and 89384ed hold
steps 1 to 3. The main session ran the gate for each commit. Two items are
open. The Open section gives them.

## Goal

HitIt uses the crate `musicindex-tags` and writes only the frame names of
musicindex ADR 0086 R-TAG-2. The operator can rewrite the library files with
the new names.

## Authority

[musicindex ADR 0086](../../../musicindex/docs/adr/0086-one-feed-parser-and-one-tag-crate-for-songs-and-episodes.md)
R-FEED-4 item 5, R-TAG-1 and R-TAG-2. Musicindex ADR 0089 R-TAG-6.
[ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 9 for the
route frame.

## Change

1. Step 1, commit 942baf8: the fee values of R-FEED-4 item 5 in
   `src/rss/value_routes.rs`.
2. Step 2, commit 49fcef5: HitIt uses `musicindex-tags`. `src/audio_tags.rs`
   gives the crate the `ReferenceSource` of the app. Migration 33 renames
   the old frame labels in the tag update skips.
3. Step 3, commit 2440498: `hitit library retag-podcast-frames`, in
   `src/application/commands/podcast_frames.rs`. The runbook section "Podcast
   Frame Retag Workflow" gives the steps.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Tag read and tag write | reuse | ~/build/musicindex-lib/tags/src/audio.rs | The crate owns the frame code (R-TAG-1). |
| Rename table | extend | src/audio_tags.rs | `RENAMED_FRAMES` serves migration 33 and the retag. |
| Stored route of a write | reuse | src/metadata_service.rs | `with_stored_route_frame` gives the route frame (ADR 0076 Decision 9). |
| Library file list | reuse | src/library_service.rs | `library_tracks` gives each track with its stored path. |
| Library path | reuse | src/library_path.rs | `LibraryRelativePath::resolve` gives the file path and the backup path (ADR 0064). |

## Acceptance Criteria

Mechanical:

1. The retag gives each old frame its new name and removes the old frames.
   Test `adr_0086_retag_renames_the_old_frames_and_keeps_a_backup`.
2. The `SYLT` times stay the same. Test
   `adr_0086_retag_keeps_the_transcript_times`.
3. A second run writes nothing. Test
   `adr_0086_retag_second_run_changes_nothing`.
4. The dry run writes nothing. Test `adr_0086_retag_dry_run_writes_nothing`.
5. Old Vorbis keys in capital letters get the new names. Test
   `adr_0086_retag_renames_old_vorbis_comments`.
6. A route frame of a track with no stored route is removed, and a second
   dry run gives no file to rewrite. Test
   `adr_0086_retag_removes_a_route_frame_with_no_stored_route`.

Visual: none.

## Open

1. The operator runs the dry run and decides the real run.
2. The episode part of musicindex ADR 0089 R-TAG-6. HitIt has no path that
   writes episode tags. The HitIt ADR for the station selects the action that
   writes them. Do not start it before that ADR. It also needs the push of
   stophammer-parser d105468.
