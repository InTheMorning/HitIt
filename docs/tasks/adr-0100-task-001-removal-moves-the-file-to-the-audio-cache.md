# ADR 0100 Task 001: A Removal Moves The File To The Audio Cache

Status: Complete - 2026-11-15. M1 to M13 pass. The operator passed the visual criteria V1 to V4 on 2026-11-15 with the build of commit `0c84f85`: the first start moved 6 files of earlier removals, and 215 and 6 files added up to the 221 files before the start. The operator did not examine the text "Not downloaded" of a playlist entry of a removed track.

Known limits, found in the review of the diff:

- A download of a cached WAV file whose conversion fails gives an error. The WAV file stays in the audio cache.
- A download with "redownload" does not use the cache file. The cache file and its row stay until "Delete" or the subsequent removal of the track.
- `src/subscribe_service.rs` and `src/library/app_impl.rs` call `config::audio_cache_dir()` and do not get the path as a value.

## Goal

A removal from the Library moves each audio file of the removed tracks from `music_dir` to the audio cache. A download of a cached track moves the file back. Each start moves the files of earlier removals.

## Authority

- [ADR 0100](../adr/0100-a-removal-moves-the-audio-file-to-the-audio-cache.md) owns each rule of this packet. Read it fully before the first edit.
- The migration registry of ADR 0016 owns each schema change. Its tests in `src/db.rs` and `src/db/upgrades.rs` enforce it.
- [AGENTS.md](../../AGENTS.md): the layers, the commit checks and the conventions.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from commit `e62c0f6`, read on 2026-11-15.

- `execute_library_removal` in `src/application/library_removal.rs` takes only a connection. `execute_track_removal` and `execute_feed_removal` write the membership and do no disk operation.
- `RemoveFromLibrary::execute` in `src/application/commands/library_removal.rs` is the only production caller.
- `local_files.path` is a `LibraryRelativePath` (`src/library_path.rs`). `resolve` joins it to a root folder.
- `cached_tracks` in `src/db.rs` joins `local_files` with `is_in_library = 0`.
- `delete_cached_file` and `cleanup_empty_parents` are in `src/library_service.rs`. `RemoveCachedFiles` in `src/application/commands/download.rs` calls `delete_cached_file`.
- `prepare_track_for_subscription_internal` in `src/subscribe_service.rs` uses an existing file before a download. `src/subscribe_service/materialization.rs` calls it with the stored path.
- `src/startup.rs` calls `db::repair_local_file_paths` after the migrations. `src/cli.rs` calls it also.
- `MIGRATIONS` in `src/db.rs` has 26 entries. `docs/schema/storage-and-metadata.md` describes the storage.
- `DownloadedTrack::promote` in `src/track_compare.rs` uses `fs::hard_link`. No helper does a hard link with a copy as the alternative.
- The config and data folders come from `ProjectDirs` in `src/config.rs`.
- `upsert_local_file` in `src/db.rs` is the only production writer of `local_files`, and only `mark_track_downloaded` calls it. `mark_track_downloaded` sets `is_in_library = 1` first. Thus no download keeps a file with a track that is out of the Library.
- `mark_feed_subscribed: false` of a search download in `src/subscribe_service.rs` is the subscription of the feed, not the membership of the track. The first Stop And Ask condition does not apply to it. The main session verified this on 2026-11-15.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Stored relative path | reuse | `src/library_path.rs` | `resolve` takes a root, so the same value addresses a file below `music_dir` and below the audio cache. |
| Empty parent cleanup | extend | `src/library_service.rs` | `cleanup_empty_parents` takes a root. Use it for `music_dir` and for the audio cache. |
| Cached file delete | extend | `src/library_service.rs` | `delete_cached_file` changes its root to the audio cache and its table to `cached_files`. |
| Migration registry | extend | `src/db.rs` | Migration 27 adds `cached_files`. |
| Startup repair call | extend | `src/startup.rs` | The start step of ADR 0100 Decision 7 runs after `repair_local_file_paths`. |
| App folders | extend | `src/config.rs` | Add one function for the audio cache folder from `ProjectDirs::cache_dir()`. |
| File move between two roots | new | `src/audio_cache.rs` | Searched `src/track_compare.rs`, `src/track_compare/retained.rs`, `src/config/user_data_move.rs`, `src/audio_tags.rs` and `~/build/musicindex-lib`. No helper does a hard link with a copy as the alternative and a removal of the source after a commit. |

## Required Changes

1. **The folder.** Add a function in `src/config.rs` that gives `ProjectDirs::cache_dir().join("audio")`. Give the path to each caller as a value. A test gives a temporary folder and does not read the environment.
2. **Migration 27.** Add the table `cached_files` with the columns of `local_files`: `id`, `path` (unique), `track_id`, `added_at`, `file_size_bytes`, `audio_duration_sec`, `checksum` and `extra_json`. Add an index on `track_id`. Change no other table. Update each test and each list that names the tables or the current version, and `docs/schema/storage-and-metadata.md`.
3. **The move module.** Add `src/audio_cache.rs`. It holds the three steps of ADR 0100 Decision 4 as separate functions: put the files into the destination root, and, after the commit of the caller, remove the sources and the empty parent folders. It also holds the undo of the first step. The same functions do the two directions.
4. **The removal.** `execute_library_removal` takes `music_dir` and the audio cache folder. A track removal and a release removal do the sequence of ADR 0100 Decision 4, with one transaction for each removal target. A `local_files` row with no file on the disk is deleted with no `cached_files` row.
5. **The download.** Before a network request, the download path looks for a `cached_files` row of the track (ADR 0100 Decision 5). It moves the file back to its stored relative path and gives it as the existing file. With no cache file on the disk, it deletes the row and continues.
6. **Settings, Cached files.** `cached_tracks` reads `cached_files`. `delete_cached_file` deletes below the audio cache. Each path that the Settings list shows or opens resolves below the audio cache.
7. **The start step.** Add the step of ADR 0100 Decision 7 after `repair_local_file_paths` in `src/startup.rs`. Each failure goes to the log, and the start continues. Do the same call in `src/cli.rs` only if the CLI path there prepares the same database for use.
8. **Other readers.** Search for each use of `is_in_library = 0` with `local_files`, and each caller of `cached_tracks`. Make each agree with ADR 0100 Decision 3.
9. **The task text.** In `docs/tasks/adr-0076-task-003-stored-payment-route-and-readiness.md`, replace the sentence `**"Remove from library" deletes the audio file.**` with `**"Remove from library" moves the audio file to the audio cache (ADR 0100).**`.

## Do Not Change

- No setting for the cache folder, no size limit and no automatic purge.
- No change of the removal confirmation, its text or its plan.
- No change of `LibraryRelativePath` validation.
- No new text in the interface, apart from an error message of a failed removal through the existing error path.
- No change of `src/assets/tokens.json`.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical. Each test uses real files in a `tempfile` folder and cites ADR 0100 in its name or its message:

- M1: a track removal leaves no file of the track below `music_dir`. The file is in the audio cache at the same relative path, with one `cached_files` row and no `local_files` row.
- M2: a release removal does the same for each track of the release, and removes the empty release and artist folders below `music_dir`.
- M3: a removal whose put step fails changes no row and no file, and gives an error. Use a file as the audio cache path to cause the failure.
- M4: a removal of a track with a `local_files` row and no file on the disk deletes the row, adds no `cached_files` row and succeeds.
- M5: a download of a cached track puts the file back below `music_dir`, restores the `local_files` row, deletes the `cached_files` row and sends no request.
- M6: a download of a cached track with no cache file on the disk deletes the `cached_files` row and reaches the network path.
- M7: `RemoveCachedFiles` deletes the cache file and its `cached_files` row, and changes no file below `music_dir`.
- M8: the start step moves the file of a `local_files` row whose track is not in the Library.
- M9: the start step removes a file below `music_dir` that has a `cached_files` row, no `local_files` row and a cache file of the same length. It keeps that file when the lengths differ.
- M10: the start step does nothing when `music_dir` is not a folder.
- M11: migration 27 adds `cached_files`, and the schema checks of the registry pass at version 27.
- M12: the copy path has a test of its own that does not need two file systems. Call the copy function directly.
- M13: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, listed apart, for the operator to walk:

- V1: after a removal of a downloaded track, its file is not below the music folder, and it is below `~/.cache/hitit/audio`.
- V2: Settings, Cached files, lists the removed track, and "Delete" removes its cache file.
- V3: a download of the removed track completes immediately, and the file is below the music folder again.
- V4: after the first start of the new build, the files of earlier removals are not below the music folder.

## Test Commands

- `cargo test --lib audio_cache`
- `cargo test --lib application::library_removal`
- `cargo test --lib application::commands`
- `cargo test --lib db::`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed, the new functions and the tests of M1 to M12 by name. Tell which readers step 8 found and what changed in each. Tell if `src/cli.rs` got the start step, and why.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a production path downloads a track and keeps it out of the Library on purpose. The start step would move its file.
- Stop if a reader needs the file of a removed track for playback or for a tag write.
- Stop if the removal runs on the UI thread and a copy of many files can block it.
- Stop if the migration registry cannot add a table without a change of a different table.
