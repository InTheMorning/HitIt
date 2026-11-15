# ADR 0068 Task 002: The Stored Show Cue

## Status

Implemented - 2026-11-14. Items R0 to R7 of the Review Findings and items L1 to L3 and L5 to L10 of the last inspection are corrected. Item L4 moves to task 003. Mechanical checks are Green. The visual checks V1 to V4 are open in section 19 of [pending human checks](../pending-human-checks.md).

## Goal

The curator loads a playlist into one stored Show cue. The load starts no audio and publishes nothing. While nothing plays, Show shows the loaded cue.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "The Show Cue Exists Before Playback", "Persistence, Restart And CLI Compatibility" and its [revision of 2026-11-11](../adr/0068-show-cue-and-audition-isolation.md#revision-for-the-adr-0089-shell---2026-11-11).
- [ADR 0016](../adr/archive/0016-schema-migration-discipline.md): each schema change goes through the migration registry.
- [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 2: the RSS check runs when the curator selects a playlist for a show.
- [ADR 0089](../adr/archive/0089-the-shell-destinations-and-list-pages.md) Decision 6: adding to a playlist is in the item menu, and Play stays the main action.

## Recorded Facts - 2026-11-11

- Schema version 24 is current (`CURRENT_VERSION` in `src/db.rs:2867`). A migration is one `src/db/<name>.rs` with `pub(crate) fn apply` and one entry in `MIGRATIONS`.
- `playback_sessions` has no entry table. `local_track_id` cascades on a track delete. `playlist_id` is set to null on a playlist delete.
- The Show cuelist comes from `queue_now_playing_vm` in `src/app/queue_now_playing.rs`. It reads the session and the current playlist rows.
- Library removal (`src/application/library_removal.rs`) counts playlist references only.
- `db::tracks_in_use_by_show` (`src/db.rs:2815`) reads the session and its playlist. Its comment says that ADR 0068 is Proposed.
- The RSS check starts in the `PlayPlaylistAt` handler in `src/app.rs:418` with `RssCheckTrigger::PlaybackStart`. A guard in `src/application/commands/playlist.rs` requires this.

## Files To Inspect

- `src/db.rs` (`MIGRATIONS`, `playback_sessions`, `tracks_in_use_by_show`), `src/db/playlist_order.rs` as a model of a migration
- `src/application/commands/playlist.rs`, `src/application/queries/`
- `src/application/library_removal.rs`, `src/view_models/library_removal.rs`
- `src/app/queue_now_playing.rs`, `src/view_models/queue_now_playing.rs`, `src/app/show.rs`
- The item menu of a playlist row and of a playlist page in `src/view_models/library.rs`

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| SQLite migration registry | extend | `src/db.rs` | Adds migration 25 and its read contract. |
| Playlist command bus | extend | `src/application/commands/playlist.rs` | Adds typed cue commands. |
| Playlist sidebar query | extend | `src/application/queries/playlist.rs` | Reads whether a cue exists. |
| Show queue projection | extend | `src/app/queue_now_playing.rs` | Reads the stored cue while playback is stopped. |
| Playlist row menu | extend | `src/view_models/library.rs` | Adds the cue load action and label. |
| Show cue storage | new | `src/db/show_cue.rs` | Searched `src/db.rs` and `src/db/`; no stored cue component exists. |

## Scope

1. Migration 25 adds the cue:
   - `show_cue`: one row with the selected entry, the time of the load, and the source playlist id and name as provenance.
   - `show_cue_entries`: an entry id of its own, the position, the track id, and the source playlist id and position.
   - `playback_sessions.cue_entry_id`, nullable. Packet 003 writes it.
   A track that occurs two times gets two entries. Existing playlists, sessions and migration records stay. No migration makes a cue from an old session.
2. Typed commands `LoadShowCue`, `ReplaceShowCue` and `ClearShowCue` in one transaction each. An empty playlist does not replace a cue. A failed load keeps the old cue. Replace and Clear are not available while the session plays or is paused.
3. A query gives the cue entries with their track, their file state and a typed reason when an entry cannot play. An entry with a missing file stays in the cue.
4. The item menu of a playlist gets "Load into Show Cue", or "Replace Show Cue" when a cue exists. The view model gives the label and the availability.
5. While the session is stopped or absent, the Show cuelist shows the loaded cue. While a session plays, the projection of today stays. Packet 003 removes it.
6. Library removal counts the cue entries of a track and names the cue in its confirmation.
7. `tracks_in_use_by_show` also gives the tracks of the cue. Correct its comment.
8. The RSS check starts on Load and on Replace with a new trigger `RssCheckTrigger::CueLoad`. The start on `PlayPlaylistAt` stays until packet 003 removes that event. Change the guard in `src/application/commands/playlist.rs` to require the new trigger.

## Do Not Touch

- `PlaybackOwner`, the driver and the producer. Packet 003 changes the owner.
- The Show transport and the keys.
- The CLI.

## Mechanical Criteria

- M1 of ADR 0068: Load and Replace are atomic, start no player and write no session. A failed or empty load keeps the old cue. Two occurrences of a track have two entry ids.
- M2 of ADR 0068: the cue keeps its order after an edit, a reorder or a delete of its source playlist, and after a restart. An entry with a missing file stays visible with its reason. Library removal counts the cue entries.
- M10 of ADR 0068: migration 25 keeps the playlists, the tracks, the bindings, the sessions and the migration records. A session from before the migration makes no cue.
- The view model gives "Load into Show Cue" and "Replace Show Cue" with their availability.
- A Load starts one RSS check with `RssCheckTrigger::CueLoad`.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: the item menu of a playlist has "Load into Show Cue". After a Load, Show shows the cue, and no audio plays.
- V2: with a loaded cue, the menu shows "Replace Show Cue".
- V3: a Load of an empty playlist shows "The playlist has no tracks. The Show cue did not change."
- V4: while a playlist plays, "Replace Show Cue" is not available. After a stop, it is available again (item L2).

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo test --test architecture_tests
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```

## Report

Migration 25 creates `show_cue` as a singleton with its selected entry, load time, and source playlist id and name.
It creates ordered `show_cue_entries` rows with unique ids, track ids, and source playlist positions.
It adds nullable `playback_sessions.cue_entry_id` and adds `cue_load` to the RSS check trigger values.
The migration preserves existing playlists, tracks, bindings, sessions, and migration records.

The change extends these existing components:

- The SQLite migration registry in `src/db.rs`.
- The playlist command bus in `src/application/commands/playlist.rs`.
- The playlist sidebar query in `src/application/queries/playlist.rs`.
- The Show queue projection in `src/app/queue_now_playing.rs`.
- The playlist row menu in `src/view_models/library.rs`.
- The Library removal plan and confirmation in `src/application/library_removal.rs` and `src/view_models/library_removal.rs`.
- The RSS check trigger path in `src/db/rss_check_runs.rs` and `src/view_models/playlist_rss_check.rs`.
- The database startup migration path in `src/view_models/startup/database.rs`.

The change adds cue storage in `src/db/show_cue.rs` because no stored cue component existed.

Corrections for the Review Findings of 2026-11-14:

- R1: `show_cue` reads one row for each cue entry. The file of an entry is the file that `db::track_row_by_id` gives for the track. This is the existing read of one track. Its query has no `ORDER BY`. The test `cue_entries_select_one_local_file_for_each_cue_entry` proves one entry for each cue row.
- R2: `show_cue` gives `MissingFile` for a track with no binding. The Show queue projection in `src/app/queue_now_playing.rs` resolves the bound path against the music folder and gives `MissingFile` when the file is not on the disk. The query does no disk access. The test `stored_cue_marks_a_bound_file_absent_from_the_disk_as_missing` proves the disk check.
- R3: after a Load, `finish_show_cue_load` reads the playback session. Replace is available only when no session exists or the session is `stopped`.
- R4: the playlist page gets the cue item from `playlist_row_menu_with_cue`, the function of the row menu.
- R5: the removal confirmation counts the cue entries of a track as "cue entries". A track in a playlist counts each playlist one time.
- R6: the tests in `src/db/show_cue.rs` cover M1, M2 and M10.
- R7: the test `adr_0068_successful_cue_loads_start_one_rss_check_each` proves that a Load and a Replace each start one check with `RssCheckTrigger::CueLoad`. An empty Load and a Load that gives an error start no check. The remaining guard checks the ADR 0076 `PlaybackStart` rule, so it stays.

Files changed:

- `AGENTS.md`
- `docs/architecture/source-map.md`
- `docs/pending-human-checks.md`
- `docs/tasks/adr-0068-task-002-stored-show-cue.md`
- `src/app.rs`
- `src/app/playback_bar.rs`
- `src/app/queue_now_playing.rs`
- `src/app/show.rs`
- `src/application/commands/playlist.rs`
- `src/application/library_removal.rs`
- `src/application/queries/playlist.rs`
- `src/db.rs`
- `src/db/maintenance/upgrade.rs`
- `src/db/playlist_order.rs`
- `src/db/rss_check_runs.rs`
- `src/db/show_cue.rs`
- `src/db/upgrades.rs`
- `src/library.rs`
- `src/library/app_impl.rs`
- `src/ui/composites/track_row_menu.rs`
- `src/ui/shells/library/detail.rs`
- `src/ui/shells/library/playlist_detail.rs`
- `src/ui/shells/playlist.rs`
- `src/ui/shells/queue_now_playing.rs`
- `src/view_models/library.rs`
- `src/view_models/library_removal.rs`
- `src/view_models/live_status.rs`
- `src/view_models/playlist_rss_check.rs`
- `src/view_models/queue_now_playing.rs`
- `src/view_models/release_page.rs`
- `src/view_models/release_row_actions.rs`
- `src/view_models/show.rs`
- `src/view_models/startup/database.rs`

These commands pass:

- `cargo fmt -- --check`
- `cargo test --locked --offline --quiet`
- `cargo test --test architecture_tests`
- `cargo clippy --locked --offline --quiet --all-targets -- -D warnings`
- `cargo build --locked --offline --quiet --bin hitit`
- `git diff --check`
- `python3 /home/citizen/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics --lines 599:617 docs/pending-human-checks.md`

## Review Findings - 2026-11-14

An independent review and the orchestrating session found these items in the first implementation. The orchestrating session corrected item R0. Items R1 to R7 are open.

- R0, corrected: migration 25 rebuilds `rss_check_runs`. With foreign keys on, the drop deleted each row of `rss_check_differences` and cleared `rss_field_holds.run_id`. The rebuild now sets these rows aside and restores them. The test `migration_25_keeps_the_rss_check_history` proves it.
- R1: `show_cue` in `src/db/show_cue.rs` joins each `local_files` row of a track. A track with two bindings gives each entry two times. Give one row for each cue entry. Use the rule that the existing code uses to select the file of a track, and name that rule in the Report.
- R2: an entry whose file is not on the disk gets no reason, because the query examines only a null path. Use the existing file readiness path of the Library to give `CueUnavailableReason::MissingFile`. Keep the query free of disk access if the existing path does the check in the view model.
- R3: `src/library/app_impl.rs` calls `set_show_cue_state(true, true)` after a Load. While a session plays or is paused, Replace must stay unavailable. Read the session state after the load.
- R4: the item menu of a playlist page (`src/view_models/library.rs`, the page actions near Rename, Delete and Check RSS) has no "Load into Show Cue" or "Replace Show Cue". Add them through the same view model function as the row menu.
- R5: the removal confirmation (`src/view_models/library_removal.rs`) calls the cue entry count "tracks". Two entries of one track give "2 tracks". Name them as cue entries.
- R6: add tests at the layer that owns each rule:
  - M1: Replace and Clear fail while the session is `playing` and while it is `paused`. A successful Replace and a successful Clear. A failed or empty Load keeps the old cue and writes no session row.
  - M2: a cue of two different tracks keeps its order after an edit, a reorder and a delete of its source playlist. It also keeps its order after a close and an open of the database file. Library removal counts the cue entries of a track. `tracks_in_use_by_show` gives a cue track with no session.
  - M10: a database at version 24 with a playlist, a session and the ledger keeps each value after migration 25, and gets no cue.
- R7: the guard in `src/application/commands/playlist.rs` reads two strings. Add a behavior test: a Load and a Replace each start one RSS check with `RssCheckTrigger::CueLoad`, and a failed or empty Load starts none. Keep the guard only if it checks a rule that the behavior test does not check.

## Last Inspection - 2026-11-14

A second agent examined the full diff and found these items.

Corrected items:

- L1: the album removal plan counted cue entries. It counts each Library track one time, as the playlist count does. The text gives the singular or the plural for each count. The tests `album_removal_counts_each_cue_track_one_time` and `album_removal_message_names_each_count_with_its_number_form` prove it.
- L2: a stop that the playback poll finds did not refresh the cue menu. The `SessionStopped` arm in `src/app.rs` reloads the playlists. No test covers this arm. The manual check is open.
- L3: `show_cue_entries.id` uses `AUTOINCREMENT`. A Replace does not use an entry id again. The test `replace_never_gives_an_old_entry_id_again` proves it.
- L5: a Load or a Replace that gives an error shows "The Show cue did not change" and reloads the cue menu. Before, it showed a playlist load error.
- L9, part: `refresh_show_cue_state` had the same body as `reload_playlists`. The change removes it.
- L6: a Library track with no file binding gets "Not downloaded". A bound file that is not on the disk gets "File missing". The tests `stored_cue_reports_not_downloaded_without_a_file_binding` and `unavailable_reasons_have_distinct_labels` prove it.
- L7: `queue_now_playing_input` reads the session and the cue in the database lock. `queue_now_playing_vm` examines the cue files after the lock. The playlist sidebar query reads `db::show_cue_exists`. The test `show_cue_exists_tracks_load_and_clear` proves the new read.
- L8: a Load of an empty playlist shows "The playlist has no tracks. The Show cue did not change." The test `empty_show_cue_load_reports_that_the_cue_did_not_change` proves the text.
- L9: `db::show_cue_can_change` owns the rule that the cue changes only while playback is stopped. Load, Clear and the sidebar query use it. `finish_show_cue_load` reloads the playlists and reads no session. The two reason enums stay, because only the view model enum has the disk reason.
- L10: the tests `failed_replace_rolls_back_cue_rows_and_entry_ids` and `cue_sequence_and_source_positions_survive_source_reorder` are new. The RSS check test keeps its closure, because `finish_show_cue_load` needs a GPUI context. The test and the method share `cue_load_rss_check_trigger`.

Open item:

- L4: `ClearShowCue` has no entry point. ADR 0068 puts the Clear item on the Show page. Operator decision of 2026-11-14: task 003 adds the Clear item and its test.

## Corrections For The Open Items - 2026-11-14

This section was the scope of the correction change. A ChatGPT run made the change, and the orchestrating session examined the diff and ran the checks. Item L4 stays with task 003.

Operator choice: the orchestrating session selected each correction below. The operator gave no different instruction.

### C6: Two Reasons For A Track With No Playable File (L6)

- In `src/db/show_cue.rs`, rename `CueUnavailableReason::MissingFile` to `NotDownloaded`. `show_cue` gives it for a Library track with no file binding. The query does no disk access.
- In `src/view_models/queue_now_playing.rs`, add `QueueUnavailableReason::NotDownloaded` with the label "Not downloaded". `MissingFile` keeps the label "File missing".
- In `src/app/queue_now_playing.rs`, map `NotDownloaded` to `NotDownloaded`. Give `MissingFile` only when a binding exists and the file is not on the disk.
- Tests: `show_cue` gives `NotDownloaded` for a Library track with no binding. The projection gives `NotDownloaded` for that track and `MissingFile` for a bound file that is not on the disk. The view model gives the two labels.

### C7: No File Read In The Database Lock (L7)

- Divide `queue_now_playing_vm` in `src/app/queue_now_playing.rs` into two steps. Step 1 reads the session and the cue or the session tracks from the connection. Step 2 examines the cue files on the disk and builds the view model. Step 2 gets no connection.
- In `RefreshShowPage::execute` in `src/app/show.rs`, do step 1 and `load_event_section` in the lock block. Do step 2 after the block ends.
- Add `db::show_cue_exists(conn) -> Result<bool>` in `src/db/show_cue.rs`. It reads only `SELECT EXISTS(SELECT 1 FROM show_cue WHERE id = 1)`. `LoadPlaylistSidebar` in `src/application/queries/playlist.rs` uses it, and does not call `db::show_cue`. `load_show_cue` also uses it for its check of an existing cue.
- Test: the existing test `stored_cue_marks_a_bound_file_absent_from_the_disk_as_missing` calls the two steps and passes. A new test proves `show_cue_exists` before a Load, after a Load and after a Clear.

### C8: An Empty Playlist Gives A Message (L8)

- Add `report_empty_show_cue_load` to the Library view model in `src/view_models/library.rs`, adjacent to `fail_show_cue_change`. It sets the status to "The playlist has no tracks. The Show cue did not change."
- `finish_show_cue_load` in `src/library/app_impl.rs` calls it when `loaded` is `false`.
- Test: a view model test proves the status text.

### C9: One Owner Of The Rule That The Cue Changes Only While Playback Is Stopped (L9)

- Add `db::show_cue_can_change(conn) -> Result<bool>` in `src/db/show_cue.rs`. It gives `true` when there is no session `default`, or when the session state is `stopped`. Give it a doc comment that names ADR 0068.
- `load_show_cue` (for a Replace), `clear_show_cue` and `LoadPlaylistSidebar` use this function. Remove their own state comparisons.
- `finish_show_cue_load` does not read the session. After a load that changes the cue, it calls `self.reload_playlists(cx)`, and the sidebar load sets the cue menu state. Keep the RSS check start and the `ShowCueLoaded` event. This also removes one database lock on the UI thread.
- The two reason enums stay. After C6 they are different: the database enum has no disk reason, and the view model enum adds `MissingFile`. Write this in a doc comment on `QueueUnavailableReason`.
- The existing tests of Replace and Clear in the states `playing` and `paused` must pass without a change.

### C10: Missing Tests (L10)

- Rollback: with a cue from playlist A, a Replace from playlist B gets an error after its first write. Make the error with a `TEMP TRIGGER` that aborts an insert into `show_cue_entries` at position 1. The cue of playlist A stays, with the same entry ids and the same sequence.
- Source reorder: load a cue of three different tracks, then reorder the source playlist with the existing playlist reorder function. The cue keeps its sequence and its `source_position` values.
- The test `adr_0068_successful_cue_loads_start_one_rss_check_each` keeps its closure. `finish_show_cue_load` needs a GPUI context, and the test and the method share `cue_load_rss_check_trigger`. Add a comment in the test that says so.

### Checks And Report

- Run each command of "Test Commands". Each command must pass.
- Follow `/home/citizen/.agents/skills/asd-ste100/SKILL.md` for each comment and each document line.
- Do not commit. Do not change `AGENTS.md`, the ADRs or `docs/pending-human-checks.md`.
- Do not change this task file. In the last message, give for each of C6 to C10 the files that changed and the name of each new test.
- Stop and report if a correction needs a change in `PlaybackOwner`, the driver, the producer, the Show transport or the CLI.

## Stop And Ask

Stop if a different path uses the session playlist as the list of tracks in use. `tracks_in_use_by_show` is the known path.
