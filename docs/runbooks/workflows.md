# Library And Discovery Workflows

## Find Music Workflow

Search, Releases and Artists in the Music sidebar show MusicIndex items. The
scope `All` shows MusicIndex items. The scope `In Library` shows local items.

### Search And Inspect

- Enter a query in the search field to search MusicIndex and the local library.
- Open inspectors for artists, feeds, tracks, and publishers linked from feeds or tracks.
- Feed and track inspectors surface source metadata, contributors, and value routes when available.

### Download A Release

`Download Release` subscribes the feed. A release page gives the release type
in the label, for example `Download Album`. It does more than flip a local flag.

The app will:

1. fetch the RSS feed directly
2. upsert feed-level metadata into SQLite
3. upsert track rows for feed items with stable GUIDs
4. try to hydrate track details from MusicIndex where track GUIDs exist
5. enrich track metadata from raw RSS for transcript and nostr-related fields
6. download each supported track file into the managed music directory
7. write generated ID3v2.4 edits when the target file supports that path
8. mark downloaded tracks as part of the local library

If a track cannot be downloaded or matched back into the database, the subscription message reports it as skipped rather than pretending it succeeded.

### Download A Track

`Download Track` is a narrower version of the same flow:

1. ensure the parent feed has been imported from RSS
2. download the track file
3. apply any prepared ID3 edits
4. mark the matching database track as downloaded and in-library

Track subscription is the fastest path from discovery to a managed local file.

### Remove From The Library

- `Remove Track…` and `Remove from Library…` clear the in-library state of the track.
- A removal of a release also clears the subscribed state of the feed.
- A removal moves the audio file from the music folder to the audio cache
  `~/.cache/hitit/audio`. The feed and track rows stay in the database.

## Library Workflow

The scope `In Library` of Releases and Artists shows the local library.

### Browse Managed Tracks

- Album inspectors show per-track actions plus album-level MusicBrainz staging.
- Track inspectors show local-file-aware actions once a file exists on disk.

### Compare Embedded Metadata

On a downloaded track you can open the compare view to inspect:

- RSS-derived values
- embedded ID3 values from the local file
- optional MusicBrainz candidate values

The compare grid is provenance-first. It surfaces differences rather than collapsing them into one guessed answer.

### Stage And Apply ID3 Edits

The local track inspector can stage edits from the compare grid.

- auto-staged edits come from the aligned metadata rows
- manual staging can happen by drag-copying source values onto writable ID3 targets
- duplicate effective targets are treated as conflicts and must be resolved before apply
- apply writes explicit ID3v2.4 frames to the local file, then re-reads the file for verification

### MusicBrainz Lookup

MusicBrainz lookup is available from local downloaded content.

- single-track lookup uses local tags plus track context to build a metadata query
- album-level lookup stages candidate-based edits across multiple downloaded tracks
- results are presented as candidates, not silently applied truths

This is metadata matching, not fingerprinting.

### Audio Cache

`Settings`, `Library`, `Audio cache`, `Cached files` lists the files of removed tracks.

Use it to:

- delete one cached file with `Delete`
- delete all cached files with `Delete All Cached`

A delete removes the file from disk and deletes its `cached_files` entry. It does not remove the feed or track rows.

## Feed Refresh Workflow

The library can refresh already-subscribed feeds without re-importing everything from scratch.

### Check Feeds

`Check all feeds`, at the bottom of the sidebar, compares each subscribed feed's stored
`musicindex_updated_at` value with the current MusicIndex feed detail.

Feeds with newer remote timestamps are applied in the same press. After those
updates finish, the same command repairs downloaded tracks that still lack the
embedded payment-route tag.

### Apply Updates

When feeds have stale updates, the sidebar shows `Apply updates (n)` instead
of `Check all feeds`. The Updates section of the Inbox also has `Apply updates`.
It walks downloaded library tracks for those stale feeds and:

1. fetches fresh track and feed detail from MusicIndex
2. regenerates ID3 edits from the current metadata view
3. writes any new edits to local files
4. records the new `musicindex_updated_at` value in SQLite

The status line reports:

- number of tracks updated
- number of edits written
- any file-level ID3 write failures
- any feed-level failures

## Broadcast Readiness Repair Workflow

Use readiness repair when a downloaded library track lacks the embedded
`TXXX:Podcast Value Routes` tag.

```bash
hitit broadcast readiness --json
hitit broadcast repair-routes --json
hitit broadcast repair-routes <track-id> --json
```

`repair-routes` fetches MusicIndex track detail with `payment_routes` included,
applies the existing feed-level route fallback, and writes only a non-empty
Podcast Value Routes tag. In the app, `Check all feeds` runs the repair-all
path after any stale feed updates apply. The single-row `Fix routes` action and
the one-track CLI command ask upstream again for that track.

The JSON summary separates:

- `repaired`: local files that received the route tag
- `no_routes_upstream`: tracks MusicIndex cannot repair because neither the
  track nor the feed carries payment routes
- `failed`: tracks whose lookup or tag write failed
- `skipped`: tracks already ready when the command reached them

`NoRoutesUpstream` is not a failure. The readiness report keeps it separate so
the publisher can fix routes upstream before the local repair is run again.

## Podcast Frame Retag Workflow

[musicindex ADR 0086](../../../musicindex/docs/adr/0086-one-feed-parser-and-one-tag-crate-for-songs-and-episodes.md)
R-TAG-2 gives each `MusicIndex ...` frame a new `Podcast ...` name. HitIt reads
only the new names. Use this command one time to give the library files the new
names.

The same write also repairs each ID3 tag that is not a correct ID3v2.4 tag
(musicindex ADR 0090 R-TAG-8 and R-TAG-10 item 2). Examples are a bad frame, an
ID3v2.3 tag and an ID3v1 tag. A file with a bad tag header is unreadable. The
write does not change it (R-TAG-7 item 5).

1. Do the dry run. It changes no file. It lists each file with old frames, the
   old frames and the new frames, and then the counts. A line "repair:" names
   each repair of the ID3 tag.

   ```bash
   hitit library retag-podcast-frames
   ```

2. Read the counts. A route frame of a track with no stored route is removed
   and not written again (ADR 0076 Decision 9).
3. Select a backup folder that is not in the music folder. The write copies
   each file into the backup folder, at its library path, before it changes
   the file. It does not replace a backup that exists.
4. Do the write.

   ```bash
   hitit library retag-podcast-frames --write --backup-dir /absolute/backup/folder
   ```

5. Do the dry run again. "Files to rewrite" must be 0.

A second write does not change a file that has a correct tag. To get the
file of the time before the write, copy it from the backup folder.

## Playback Session Workflow

`PlaybackSession` is the authoritative now-playing state. Player adapters report
transport facts such as position and pause state into this model. They do not
define metadata identity.

### Session-Only CLI Mode

The one-shot CLI preserves ADR 0020 simulated playback behavior. Commands write
`playback_sessions` synchronously, JSON reflects the database write
immediately, and no audio process is started or controlled.

### Preview A Playlist Row

Use dry-run to inspect the JSON that a playlist row would produce:

```bash
hitit playlist play <playlist-id> --dry-run
hitit playlist play <playlist-id> --dry-run --position <zero-based-position>
```

Dry-run validates that the selected track has a local file binding and a feed
GUID. It prints `NowPlayingUpdate` JSON with `sequence` set to `0` and does not
write `playback_sessions`.

### Simulate Playlist Playback

Use `playlist play` without `--dry-run` to persist the selected playlist row as
the default playback session.

```bash
hitit playlist play <playlist-id>
hitit playlist play <playlist-id> --position <zero-based-position>
hitit playback next
hitit playback previous
hitit playback pause
hitit playback resume
hitit playback stop
hitit now-playing --json
```

`next` and `previous` move inside the playlist stored on the current playback
session. They fail when the current session was set to a loose track instead of
a playlist row, or when the requested playlist position does not exist.
`pause` and `resume` update the session state without changing metadata
identity.

### Set And Update The Current Session

Use lower-level playback commands when a loose track should become the current
session state:

```bash
hitit playback set-track <track-id>
hitit playback position <ms>
hitit playback pause
hitit playback resume
hitit playback stop
hitit now-playing --json
```

`set-track` validates the same source facts as dry-run, then persists the
default playback session. `position` updates the current position in
milliseconds. `pause` and `resume` set `playback_sessions.state` to `paused` or
`playing`. `stop` marks the session stopped. After stop,
`now-playing --json` reports no current playback session.

Every persisted state change increments the session `sequence`.

### Live-Driver Mode

Live playback is owned by a long-running desktop/TUI process, or a future
playback daemon. One-shot CLI commands do not spawn, poll, reuse, or cleanly
drop mpv across separate invocations unless a later daemon/RPC workflow defines
that boundary.

When `playback.driver = "mpv"` is enabled for the long-running owner:

- the owner creates one `MpvDriver` for the default session
- mpv starts lazily on first load and is reused across tracks
- playlist row play actions in the desktop library route through the owner
- the desktop transport controls route previous, pause/resume, next, and stop
  through the same owner
- commands routed to the owner call the driver, then reconcile observed state
  back into `playback_sessions`
- `playback position <ms>` seeks the driver and persists the accepted target
  immediately. Later polls may correct drift
- `playback next` and EOF use the same playlist advancement service
- `playback stop` stops the driver when live, then marks the session stopped
- pause is persisted as session state: `playing`, `paused`, or `stopped`

The mpv IPC socket is private to the current user, uses a process-specific or
randomized path, waits for readiness with a bounded timeout, and is removed on
normal shutdown. The owner should shut mpv down gracefully before killing it
after timeout. `Drop` is only best-effort cleanup.

Use the player ping debug command to validate the configured backend:

```bash
hitit player ping
```

Playback config defaults to session-only behavior:

```toml
[playback]
driver = "null" # "null" or "mpv"
mpv_path = "mpv" # optional; defaults to PATH
```

Missing `[playback]` uses `driver = "null"`. Unknown drivers fail config
validation. mpv availability is checked when live playback starts.

### Manage Broadcast Events

Broadcast event commands use the configured `musicindex_endpoint`, defaulting
to `https://api.musicindex.org`. Configure that endpoint before creating a
local test event.

```bash
hitit broadcast events list --json
hitit broadcast events create --json --label "Sunday set"
hitit broadcast events check <event-id> --json
hitit broadcast events forget <event-id>
```

`create` calls the relay, writes the broadcaster token to a local file with mode
`0600`, stores only the file path in SQLite, and prints the event row plus
`token_path`. It does not print the token text.

Give the event identifier and the token file to `musicindex-live-publisher`.
That service sends the live payloads. See
`docs/architecture/broadcast-chain.md`.

Use the live item probes when you need to test relay health or inspect the
latest listener-visible metadata for a known event:

```bash
hitit liveitem health [--endpoint <url>]
hitit liveitem latest <event-id> --json [--endpoint <url>]
```

## CLI Debug Workflow

The CLI exposes structured JSON inspection commands for backend state. These
commands are intended for debugging, scripts, and future UI contract checks.

```bash
hitit playlists list --json
hitit playlist tracks <playlist-id> --json
hitit library tracks --json
hitit track inspect <track-id> --json
```

`track inspect` uses the canonical local track identity service. It requires a
local file binding and feed GUID, matching the now-playing identity rules.

## State Model

The UI uses four related states that are worth keeping distinct:

- `subscribed feed`: the feed is tracked locally and participates in feed refresh checks
- `in library`: a track is part of the managed library view
- `cached file`: a local file exists on disk, whether or not the track is still marked in-library
- `playback session`: the CLI-visible canonical now-playing state derived from local track/feed facts
