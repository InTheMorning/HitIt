# Musicindex ADR 0086 Task 001: The Tag Crate And The Retag

## Status

In progress - 2026-11-22. Commits 942baf8, 49fcef5, 2440498 and 89384ed hold
steps 1 to 3. Step 4 applies musicindex ADR 0090. The main session ran the gate
for each commit. Three items are open. The Open section gives them.

## Goal

HitIt uses the crate `musicindex-tags` and writes only the frame names of
musicindex ADR 0086 R-TAG-2. The operator can rewrite the library files with
the new names.

## Authority

[musicindex ADR 0086](../../../musicindex/docs/adr/0086-one-feed-parser-and-one-tag-crate-for-songs-and-episodes.md)
R-FEED-4 item 5, R-TAG-1 and R-TAG-2. Musicindex ADR 0089 R-TAG-6.
Musicindex ADR 0090 (commit 6920ed8) R-TAG-7 item 5, R-TAG-8 and R-TAG-10.
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
4. Step 4, musicindex ADR 0090: `musicindex-tags` moves to musicindex-lib
   fad5343. ADR 0080 records that R-TAG-8 item 6 replaces Decision 6 for bad
   frames, frames of an older version and ID3v1. A download writes the tag one
   time, also with no edit, so the crate repairs it. The retag also rewrites
   each file with an ID3 tag to repair, and reports a file with a bad tag
   header as unreadable.
5. Step 4 also: an MP3 write gives one `TDRC` frame and no `TYER`
   frame (R-TAG-8 item 1). Thus the scan of "Update n files" compares the
   date of an MP3 file as one value, the same as the date of an MP4 file.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Tag read and tag write | reuse | ~/build/musicindex-lib/tags/src/audio.rs | The crate owns the frame code (R-TAG-1). |
| Rename table | extend | src/audio_tags.rs | `RENAMED_FRAMES` serves migration 33 and the retag. |
| Stored route of a write | reuse | src/metadata_service.rs | `with_stored_route_frame` gives the route frame (ADR 0076 Decision 9). |
| Library file list | reuse | src/library_service.rs | `library_tracks` gives each track with its stored path. |
| ID3v2 frame loop | reuse | ~/build/musicindex-lib/tags/src/id3v2.rs | `read_id3v2` gives the version, the bad frames and the ID3v1 tag of the retag plan (ADR 0090 R-TAG-7). |
| Download tag write | extend | src/subscribe_service/materialization.rs | The one download write runs also with no edit (ADR 0090 R-TAG-10 item 1). |
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
7. An ID3v2.3 file with two `PRIV` frames of size 0 is a rewrite. After the
   write, a strict `id3` read gives ID3v2.4 with no `PRIV` frame. Test
   `adr_0090_retag_rewrites_a_v23_file_with_empty_priv_frames`.
8. A correct ID3v2.3 file with no podcast frame and an ID3v1 tag is a
   rewrite. The write gives ID3v2.4 and removes the ID3v1 tag. Test
   `adr_0090_retag_converts_a_v23_file_with_no_podcast_frame`.
9. A file with a bad tag header is unreadable, and the write does not change
   it. Test `adr_0090_retag_reports_a_bad_tag_header_and_keeps_the_file`.
10. A download of a file with a bad tag gives a tag that a strict `id3` read
    accepts. Test
    `adr_0090_download_of_a_bad_tag_gives_a_tag_that_a_strict_read_accepts`.

Visual (manual, by the operator):

1. After the retag, HitIt shows the tags of the two Strange Love files
   (musicindex ADR 0090 R-TAG-10 item 2).

## Open

1. The operator runs the dry run and the write with the release binary.
2. The visual check of the Strange Love files. The orchestrator reports that
   they have an ID3v2.3 tag with two `PRIV` frames of size 0. The new crate
   reads them, and the retag removes the two frames.
3. The episode part of musicindex ADR 0089 R-TAG-6. HitIt has no path that
   writes episode tags. The HitIt ADR for the station selects the action that
   writes them. Do not start it before that ADR. It also needs the push of
   stophammer-parser d105468.

## Findings

- Operator choice, 2026-11-22: no fix for the two copies of the migration
  lines of the dry run. The upgrade to schema 33 calls `upgrades::verify_target` with the
  current version two times, after the migration and after the reopen. Each
  call makes a reference database in memory and runs each migration on it.
  Migrations 16, 17 and 26 write their counts to the error stream, so each
  call prints them with 0 counts. The operator database does not run these
  migrations again, and the lines occur only during an upgrade.
