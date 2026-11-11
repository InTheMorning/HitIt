# ADR 0068 Task 002: The Stored Show Cue

## Status

Ready - 2026-11-11. Implementation has not started.

## Goal

The curator loads a playlist into one stored Show cue. The load starts no audio and publishes nothing. While nothing plays, Show shows the loaded cue.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "The Show Cue Exists Before Playback", "Persistence, Restart And CLI Compatibility" and its [revision of 2026-11-11](../adr/0068-show-cue-and-audition-isolation.md#revision-for-the-adr-0089-shell---2026-11-11).
- [ADR 0016](../adr/archive/0016-schema-migration-discipline.md): each schema change goes through the migration registry.
- [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 2: the RSS check runs when the curator selects a playlist for a show.
- [ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decision 6: adding to a playlist is in the item menu, and Play stays the main action.

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

Give the schema of migration 25, the files changed and the tests.

## Stop And Ask

Stop if a different path uses the session playlist as the list of tracks in use. `tracks_in_use_by_show` is the known path.
