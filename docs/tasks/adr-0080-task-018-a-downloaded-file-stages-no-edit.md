# ADR 0080 Task 018: A Downloaded File Stages No Edit

## Status

Done - 2026-11-22, except the visual check. The main session ran the gate.
With task 017, both tests of this task pass. The `WOAR` cause is not known:
the step 2 test failed first on `TIPL`, which is its first assertion. The
visual check shows if the `WOAR` edit stays.

## Goal

Find which rows of the compare grid stage an edit for a file that the
download wrote. The test uses the real path through the crate
`musicindex-tags`.

## Authority

[ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 3: a file
that the app wrote shows no difference at the next compare.

## Recorded Facts

On 2026-11-22 the operator downloaded Strange Love track 05 "Come Back to Me
Now" with the release binary of commit 5acd7e6. `Compare ID3` showed 10
staged edits:

| Grid field | Frame | Staged value |
| --- | --- | --- |
| Involved people | `TIPL` | `producer: Anthony Columbro` |
| Performer [bass] | `TMCL` | `bass: Josh Moroz` |
| Performer [drums] | `TMCL` | `drums: Alex Nicol` |
| Performer [guitar] | `TMCL` | `guitar: Anthony Columbro` |
| Performer [keyboards] | `TMCL` | `keyboards: Anthony Columbro` |
| Performer [vocals] | `TMCL` | `vocals: Josh Moroz / Alex Nicol` (inference from the screen) |
| Publisher | `TXXX:V4V_PUBLISHER` | `Strange Love` |
| Release year | `TYER` | `2026` |
| Album description | `COMM:Podcast Feed Description` | the channel description |
| RSS feed website | `WOAR` | the channel website |

The item has no description. The file has `TDRC` `2026-02-11`. The grid
showed no staged edit for `TIT2`, `TALB`, `TRCK`, `TPE1`, `TPE2`, `TLAN`,
`TDRC`, `TXXX:Podcast Image`, `SYLT`, `USLT`, `TXXX:RSS Nostr Handle`,
`TXXX:Podcast Value Routes` and the two guid frames.

A grid row gets an automatic edit only when its file value is empty
(`auto_populated_pending_id3_edits` in `src/metadata.rs`). A source reading
on 2026-11-22 explains only the `TYER` row: the grid reads `TYER`, and the
crate keeps `TDRC` and drops `TYER`.

## Step 1 (Done): The Test

1. Write one test in `src/metadata_service.rs`, in the test module, near
   `adr_0080_round_trip_edits_read_back_unchanged`. Use the same helpers.
   - Make a `TrackContext` with the values of the table: five contributors
     with roles (one producer, four performers with the roles bass, drums,
     guitar, keyboards and vocals), a publisher, a channel website, a
     channel description, no item description and a release date.
   - Get the edits from `id3_edits_for_track_context`.
   - Write them to an MP3 fixture through `crate::audio_tags::write_id3v24_edits`.
   - Read the file back with the function that `Compare ID3` uses
     (`compare_downloaded_track_path` in `src/subscribe_service.rs`, or the
     functions that it calls).
   - Build the grid rows with the function that the grid uses
     (`aligned_compare_rows` in `src/metadata.rs`).
   - Call `auto_populated_pending_id3_edits` with no explicit edit and the
     format `Some(AudioFormat::Mp3)`.
   - Assert that it gives no edit.
2. Name the test `adr_0080_task_018_downloaded_file_stages_no_edit`.
3. Do not fix the defect. The test must fail. Keep it in the working tree.

## Step 2: The Stored Context Gets The Channel Values

Cause, from a reading of the code on 2026-11-22. The download and the scan
use `track_row_to_track_context_with_local_identity` in
`src/feed_service.rs`. `Compare ID3` uses a fresh context from the MusicIndex
API. The stored context has these gaps:

- `track_row_to_feed` and `hydrate_feed_identity` do not set
  `feed.description` and `feed.publisher_text`.
- `track_context_with_values` calls `crate::api::track_with_feed_defaults`
  before `hydrate_feed_identity` loads the channel credits. Then
  `hydrate_track_identity` sets `track.source_contributors` to `Some` of the
  item credits, also when that list is empty. Thus a track with channel
  credits only gets no credits.
- The channel website (`WOAR`): no cause is known. Find it with the test.

Changes:

1. `hydrate_feed_identity` sets `feed.description` and `feed.publisher_text`
   from the stored values of the feed (`stored_values::feed_values`), in the
   form that the stored value projection selects.
2. The stored context applies the feed defaults after the feed and the track
   have all their stored values. An empty item credit list is no credit
   list, so the channel credits apply. Use `track_with_feed_defaults`; do
   not copy its rules.
3. A comment at each change cites ADR 0076 Decision 10, ADR 0080 Decision 3
   and this task.

## Do Not Change

- `src/metadata.rs` and `src/application/queries/tag_update.rs`. Task 017
  changes them at the same time.
- The fresh context of `Compare ID3` and `crate::api::track_with_feed_defaults`.
- The assertion of `adr_0080_task_018_downloaded_file_stages_no_edit`. It
  continues to fail on `TYER` until task 017 is done.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Download edit builder | reuse | src/metadata_service.rs | `id3_edits_for_track_context` gives the edits of the download. |
| Tag write | reuse | src/audio_tags.rs | `write_id3v24_edits` calls the crate. |
| Grid rows | reuse | src/metadata.rs | `aligned_compare_rows` and `auto_populated_pending_id3_edits` build the grid. |
| Compare read | reuse | src/subscribe_service.rs | `compare_downloaded_track_path` reads the file for `Compare ID3`. |
| Stored context | extend | src/feed_service.rs | `hydrate_feed_identity` and `track_context_with_values` get the channel values. |
| Feed defaults | reuse | src/api.rs | `track_with_feed_defaults` gives the fallback from feed to track. |
| Stored feed values | reuse | src/application/queries/stored_values.rs | `feed_values` gives the projected description and publisher of the feed. |

## Acceptance Criteria

Mechanical:

1. The test exists and runs with
   `cargo test adr_0080_task_018_downloaded_file_stages_no_edit`.
2. The report gives the failure output. For each staged edit, it gives the
   row id, the frame, the staged value and the file value that the grid
   row holds.
3. For each staged row, the report gives the cause with file:line
   evidence. The cause is one of these: the edit builder sends no edit, the
   crate writes a different frame, the read gives a different label, or the
   grid row looks for a different label. Mark each inference.

4. Step 2. Use a test database with a feed that has a description, a
   publisher, a website link and credits with roles. Its track has no item
   description, no item publisher and no item credits. For this track,
   `id3_edits_for_track_context` of
   `track_row_to_track_context_with_local_identity` gives edits for `TIPL`,
   `TMCL`, `TXXX:V4V_PUBLISHER`, `COMM:Podcast Feed Description` and `WOAR`.
   Test `adr_0080_task_018_stored_context_gets_the_channel_values`, in the
   test module of `src/feed_service.rs`. It fails before the change.
5. Each existing test passes with no change to its assertions, except
   `adr_0080_task_018_downloaded_file_stages_no_edit`, which fails on `TYER`
   only.
6. If the `WOAR` edit is missing after changes 1 and 2, the report gives the
   cause with file:line evidence. Do not change the link storage.

Visual (manual, by the operator):

1. Re-download a Strange Love track. `Compare ID3` shows no staged edit.

## Checks

- `cargo test adr_0080_task_018` (the step 1 test fails on `TYER` only)
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

## Rules For The Delegate

- Write each comment in Simplified Technical English. The skill is at
  `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Keep the change in the working tree and report it.
- Write each failing test first, and confirm that it fails before the change.
- If an existing assertion must change, stop and report the test name and
  the reason.
