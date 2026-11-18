# ADR 0100: A Removal Moves The Audio File To The Audio Cache

## Status

Accepted - 2026-11-15. The operator selected the cache folder and the move of the old files on 2026-11-15.

- Bar: difficult to change. The decision gives each kept audio file a second root folder and a second table. Each reader of a stored path depends on these two roots.

## Context

[ADR 0060](0060-workflow-surface-structure.md) Decision "Vocabulary" says that a future ADR must define the cache policy. This ADR gives the cache part of that policy. The dump part stays open.

An examination of HitIt at commit `e62c0f6` found these facts:

- `execute_library_removal` in `src/application/library_removal.rs` writes only `is_in_library = 0`. It does no operation on the disk.
- The audio file stays in `<music_dir>/artists/`. Its `local_files` row stays also.
- `cached_tracks` in `src/db.rs` calls a track "cached" when it has a `local_files` row and is not in the Library. No cache folder exists.
- Only "Delete" in Settings, Cached files, deletes such a file.
- Thus the music folder holds the files of removed tracks. A program that reads the music folder, such as a player or a backup, gets music that the curator removed.

The operator reported this as a defect on 2026-11-15.

## Decision

### 1. The music folder holds only Library tracks

A file below `music_dir` belongs to a track that is in the Library. A removal from the Library moves each audio file of the removed tracks out of `music_dir`.

### 2. The audio cache is a folder in the cache directory of the app

The audio cache is the folder `audio` in the cache directory of the app. On Linux this is `$XDG_CACHE_HOME/hitit/audio`, usually `~/.cache/hitit/audio`. One function in `src/config.rs` gives the path. No setting changes it.

A file in the audio cache keeps its path relative to the root. The file `<music_dir>/artists/A/B/c.flac` moves to `<audio cache>/artists/A/B/c.flac`.

A different program can empty the cache directory. A lost cache file causes only a new download.

### 3. The table `cached_files` holds the cache files

A row of `local_files` is a file below `music_dir`. A row of the new table `cached_files` is a file below the audio cache. A file has a row in one table only.

Each reader of `local_files` thus sees a removed track as not downloaded. No reader gets a path with an incorrect root.

A playlist entry or a Show cue entry of a removed track reads "Not downloaded". The removal confirmation already names these references.

### 4. A removal is safe against a failure

A removal does these steps in this sequence:

1. It puts each file into the audio cache. It uses a hard link, or a copy when a hard link is not possible. The length of a copy is the length of its source.
2. It commits one database transaction. The transaction changes the membership, deletes each `local_files` row and adds each `cached_files` row.
3. It removes each file from `music_dir`, and then each empty parent folder below `music_dir`.

When step 1 or step 2 fails, the removal removes the files that it put into the audio cache and gives an error. The track then stays in the Library with its file.

A track with no file has no step 1 and no step 3. A `local_files` row with a file that is not on the disk is deleted, and no `cached_files` row replaces it.

### 5. A download of a cached track takes the file from the audio cache

A download first looks for a `cached_files` row of the track. When the cache file is on the disk, the download moves it back to its stored relative path below `music_dir`. It sends no request for the audio. The sequence of Decision 4 applies in the opposite direction.

When the cache file is not on the disk, the download deletes the row and gets the audio from the network.

### 6. Settings shows the audio cache

The list is in the section "Audio cache" of Settings, Library ([ADR 0093](0093-settings-has-a-tab-row-and-one-save-bar.md) Decision 7). The list reads `cached_files`. "Delete" and "Delete all" delete the file in the audio cache and its row. They change no file below `music_dir`.

### 7. Each start completes the move

After the migrations, each start does this step when `music_dir` is a folder:

- It moves the file of each `local_files` row whose track is not in the Library, by Decision 4. This moves the files of each removal before this ADR.
- It removes a file below `music_dir` that has a `cached_files` row, no `local_files` row and a cache file of the same length. This completes a removal that stopped after its step 2.

A failure of this step does not stop the start. The step writes each failure to the log and tries again at the next start. When `music_dir` is not a folder, the step does nothing, as [ADR 0064](archive/0064-local-file-addressing.md) decided for the path repair.

## Operator Choices

- 2026-11-15: the audio cache is `~/.cache/hitit/audio`. The operator did not select a hidden folder in `music_dir` or a setting.
- 2026-11-15: the app moves the files of earlier removals at a start. The operator did not select a button in Settings.

## Open Points

The main session selected these and the operator can change them:

- A download of a cached track uses the cache file and no network (Decision 5).
- A removal with a failed move gives an error and removes nothing (Decision 4).
- The start step writes its failures to the log only. The repair report surface of ADR 0064 task 002 does not exist yet.
- The audio cache has no size limit and no automatic purge.

## Verification

Mechanical, each with real files in a temporary folder:

- A track removal and a release removal leave no file of the removed tracks below `music_dir`. Each file is in the audio cache with a `cached_files` row.
- A removal whose step 1 fails changes no row and no file.
- A download of a cached track puts the file back and sends no request.
- A download of a cached track with a lost cache file deletes the row and continues to the network path.
- "Delete" of a cached file removes the cache file and its row.
- The start step moves the file of an earlier removal and removes a duplicate of a stopped removal. It does nothing when `music_dir` is not a folder.
- The migration adds `cached_files` and changes no other table.

Visual: the check in [pending human checks](../pending-human-checks.md) for [task 001](../tasks/adr-0100-task-001-removal-moves-the-file-to-the-audio-cache.md).
