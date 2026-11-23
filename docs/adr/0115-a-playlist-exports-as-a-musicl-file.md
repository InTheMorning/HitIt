# ADR 0115: A Playlist Exports As A musicL File

## Status

Proposed - 2026-11-22.

- Bar: crosses repositories. The station of musicindex ADR 0092 reads the file that HitIt writes. A change of the format changes the input of a different repository.
- Uses: musicindex ADR 0092 R-STATION-3 items 1 and 2 (musicindex commit 708daf9). Stophammer ADR 0054 §4 for the feed URL.

Terms:

- **musicL** is a value of `podcast:medium` in the Podcasting 2.0 namespace. A `musicL` feed is a list of songs. Each song is one `podcast:remoteItem`. This is an established term.
- A `podcast:remoteItem` names an item of a different feed by its feed GUID and its item GUID. This is an established term.
- The **playlist GUID** is the `podcast:guid` of the exported feed. This is a project term.

## Context

- Musicindex ADR 0092 R-STATION-3 item 1: the song input of the station is one `musicL` feed. Each `remoteItem` of the feed is one song.
- R-STATION-3 item 2: HitIt exports a local playlist as a `musicL` file, and a HitIt ADR owns that export. ADR 0092 gives no rule for the channel elements, the playlist GUID, the attributes or the file location.
- R-STATION-1 item 3: for a `remoteItem` with no feed URL, the crate gets the feed URL from MusicIndex by the feed GUID. Thus the feed URL is optional, and the two GUIDs are necessary.
- The `playlists` table has no GUID column. In `tracks`, `item_guid` is NOT NULL. In `feeds`, `feed_guid` and `podcast_medium` can be NULL.

## Decision

1. The command `hitit playlist export-musicl PLAYLIST_ID PATH` writes the playlist to the file `PATH`. The operator gives `PATH`. The command replaces a file at `PATH`.
2. The file is an RSS 2.0 document with the Podcasting 2.0 namespace. The channel has these elements:
   - `title`: the playlist name.
   - `description`: the playlist description, only when it has a value.
   - `podcast:guid`: the playlist GUID.
   - `podcast:medium`: `musicL`.
3. The playlist GUID is a random UUID. HitIt makes it at the first export and stores it with the playlist. Each subsequent export gives the same GUID. A rename of the playlist does not change it.
4. Each track of the playlist gives one `podcast:remoteItem`, in the playlist sequence. It has these attributes:
   - `feedGuid`: the GUID of the track feed.
   - `itemGuid`: the GUID of the track.
   - `feedUrl`: the stored feed URL, only when it is an http or https URL (Stophammer ADR 0054 §4).
   - `medium`: the stored `podcast:medium` value of the track feed, only when it has a value.
5. The export leaves out a track and reports it in these conditions:
   - The track feed has no feed GUID.
   - The track feed has a stored `podcast:medium` value that is not `music`. R-STATION-3 item 1 permits only songs.
6. The command prints the path, the playlist GUID, the number of written items and each track that it left out, with the cause. When no track remains, the command writes no file and stops with an error.

## Invariants

- Each `podcast:remoteItem` of an export has a `feedGuid` and an `itemGuid`.
- Two exports of one playlist give the same `podcast:guid`.

## Alternatives

- **A GUID from the playlist name.** A rename gives a different GUID, and the station sees a different feed. Rejected.
- **A new GUID for each export.** The station sees a different feed for each export. Rejected.
- **`medium="music"` for each item.** It states a value that HitIt does not know for a feed with no stored `podcast:medium` value. Rejected.
- **A file location that HitIt selects.** The operator puts the file where the station reads it. Rejected.

## Consequences

- The `playlists` table gets a column for the playlist GUID. A migration adds it.
- HitIt gets a direct dependency on the `uuid` crate for the random UUID.
- A track from a feed with no feed GUID is not in the export. The operator sees it in the report.
