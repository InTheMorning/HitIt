# ADR 0115 Task 001: Export A Playlist As musicL

## Status

Done - 2026-11-22. The main session ran the gate.

- Operator choice, 2026-11-22: the operator accepted ADR 0115 with no change.
- Agent choice, 2026-11-22: the stored column is `playlists.musicl_guid`. A
  unique index keeps each GUID on one playlist.

## Goal

The command `hitit playlist export-musicl PLAYLIST_ID PATH` writes a
playlist as a `musicL` file (ADR 0115 Decisions 1 to 6).

## Authority

- ADR 0115 Decisions 1 to 6 and its two invariants.
- Musicindex ADR 0092 R-STATION-3 items 1 and 2.

## Change

1. `Cargo.toml`: a direct dependency on `uuid` with the feature `v4`.
2. New `src/db/playlist_guid.rs`: migration 34 adds `playlists.musicl_guid`
   and its unique index. The module reads the export rows of a playlist and
   gives the stored GUID. It makes and stores a GUID when the playlist has
   none.
3. New `src/application/commands/musicl_export.rs`: the selection of the
   songs, the XML text and the file write.
4. `src/cli.rs`: the command `playlist export-musicl PLAYLIST_ID PATH`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Schema migration registry | extend | src/db.rs | Migration 34 is one more entry. |
| Playlist read contract | extend | src/db/playlist_order.rs | The contract of version 34 adds one column to the playlists columns. |
| Web URL check | reuse | src/metadata.rs | `web_url_value` gives the feed URL only for http or https. |
| XML writer | reuse | Cargo.toml | `quick-xml` is a current dependency. |
| musicL export | new | src/application/commands/musicl_export.rs | Searched src for `musicL`, `remoteItem` writes and `quick_xml::Writer`: HitIt writes no feed. Not searched: ~/build/musicindex-lib, because the agent does not read other suite repositories. The orchestrator names the station crate of musicindex-lib as a reader of the file, not as a writer. |
| Feed parser | reuse | Cargo.toml | The test reads the file with `stophammer_parser::profile::stophammer()`. |

## Acceptance Criteria

Mechanical:

1. Each `podcast:remoteItem` has `feedGuid` and `itemGuid`, in the playlist
   sequence. Test `musicl_export::tests::each_remote_item_has_both_guids`.
2. Two exports give the same `podcast:guid`, also after a rename. Test
   `musicl_export::tests::two_exports_and_a_rename_keep_the_guid`.
3. A track with no feed GUID and a track from a feed with a stored `podcast:medium` value other than
   `music` are left out and reported. Test
   `musicl_export::tests::left_out_tracks_are_reported_with_the_cause`.
4. `feedUrl` is written only for an http or https URL. Test
   `musicl_export::tests::feed_url_only_for_a_web_url`.
5. An item gets no `medium` when its feed has no stored `podcast:medium` value. Test
   `musicl_export::tests::no_medium_when_the_feed_has_none`.
6. When no track remains, no file is written. Test
   `musicl_export::tests::no_file_when_no_track_remains`.
7. The stophammer parser reads an exported file with no error and returns
   each remote item. Test
   `musicl_export::tests::stophammer_parser_reads_the_export`.
8. Migration 34 is in the registry, and each existing migration test passes.

Visual: none.

## Checks

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --bin hitit`
