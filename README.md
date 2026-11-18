<p align="center">
  <img src="docs/assets/hitit-logo.png" alt="HitIt logo" width="640">
</p>

# HitIt

HitIt (`hitit`) is a Linux-first GPUI desktop app for working with
MusicIndex-backed music feeds and a local managed audio library.

The current tool is not a generic roadmap project or a future media player
shell. It is a desktop operator app that already does four concrete jobs:

- discover artists, feeds, and tracks from MusicIndex
- subscribe feeds by pulling their RSS into a local SQLite database
- download track files into a managed library under `music_dir/artists/...`
- compare and update embedded metadata on local files, with optional MusicBrainz-assisted staging

## What The App Does Today

- The Music sidebar has Search, Releases, Artists, My playlists, Public playlists and Live.
  Show, Inbox and Settings follow below a line.
- Search finds artists, releases and tracks in MusicIndex and in the local library.
- Releases and Artists have the scope `All` for MusicIndex and the scope `In Library`
  for the local library.
- `Download Release` stores the RSS metadata of a feed locally and tries to
  download each track. A release page gives the release type in the label, for
  example `Download Album`.
- `Download Track` stores the parent feed if necessary, downloads the file, and
  marks that track as part of the managed library.
- `Remove Track…` and `Remove from Library…` move the audio file from the music
  folder to the audio cache. The database rows of the feed and the track stay.
- `Settings`, `Library`, `Audio cache`, `Cached files` lists the files in the
  audio cache. `Delete` and `Delete All Cached` remove them from the disk.
- Local track inspectors can compare embedded ID3 data against RSS-derived
  metadata and stage explicit ID3v2.4 edits.
- Local track and album inspectors can run metadata-based MusicBrainz lookups.
  They can use the results to stage more ID3 edits.
- `Check all feeds`, at the bottom of the sidebar, checks the subscribed feeds
  for newer `updated_at` values from MusicIndex and applies tag refreshes to
  downloaded files.

## Current Scope

The app is built around four data sources:

- RSS feed data imported into SQLite
- MusicIndex API detail data
- embedded file metadata, currently centered on ID3 workflows
- MusicBrainz metadata lookups for local enrichment

It is not currently a full playback app, a fingerprint scanner, or a general-purpose tag editor.

## Run

```bash
cargo run
```

### Optional dependency: `flac`

When a feed declares MP3 but actually ships WAV (a known RSS quirk), HitIt
silently re-encodes the download to FLAC so it can be tagged. It uses the
standard `flac` CLI. When `flac` is missing or fails, it uses `ffmpeg`.
Install one of them with your package manager:

- Debian/Ubuntu: `sudo apt install flac`
- Fedora: `sudo dnf install flac`
- Arch: `sudo pacman -S flac`
- macOS (Homebrew): `brew install flac`

Without `flac` and `ffmpeg`, WAV downloads are kept as WAV and left untagged. To
set a custom `flac` path, use `Settings`, `Library`, `Files`, `Audio converter`,
or the `flac_path` config key.

Build the desktop binary:

```bash
cargo build
```

Install it locally:

```bash
cargo install --path .
```

## Configuration

On first run the app creates a config file, then ensures the required directories exist.

Default locations on Linux-style systems:

```toml
# ~/.config/hitit/config.toml
music_dir = "/home/<user>/V4Vmusic"
db_path = "/home/<user>/.local/share/hitit/hitit.sqlite"
musicindex_endpoint = "https://api.musicindex.org"
```

The `Settings` tab lets you update:

- `musicindex_endpoint`
- `music_dir`
- `flac_path` (optional override. Blank means use `flac` from `$PATH`)

Downloads are organized under:

```text
<music_dir>/artists/<artist>/<release-or-feed>/<track file>
```

The app keeps a thumbnail cache in the config directory
(`~/.config/hitit/thumbnail-cache`) and an audio cache in `~/.cache/hitit/audio`.

## Metadata Behavior

The app is provenance-first. It keeps RSS, MusicIndex, embedded tags, and
MusicBrainz values separate. It does not silently collapse them into one
inferred truth.

Current behavior:

- RSS import stores source feed and item metadata locally.
- RSS enrichment can pull transcript URLs and nostr handles directly from feed
  XML when MusicIndex detail data is missing them.
- Local compare views line up RSS, ID3, and optionally MusicBrainz values side by side.
- Applying tag changes writes explicit ID3v2.4 frames only.
- MusicBrainz lookup is metadata-based. There is no acoustic fingerprinting flow in this app.

Current limitations:

- embedded metadata inspection and editing are centered on ID3-backed files
- the library download layer recognizes more than MP3, but the richest
  compare/edit flows are still MP3/ID3-first
- search-side download/compare code still follows an MP3-oriented path

## Keyboard Shortcuts

Global shortcuts. Ctrl is Cmd on macOS.

- `Ctrl+1`: switch to `Music`
- `Ctrl+2`: switch to `Show`
- `Ctrl+3`: switch to `Settings`
- `Ctrl+F` or `Ctrl+Alt+F`: focus the search field
- `Ctrl+N`: make a new playlist
- `Ctrl+R`: refresh the library
- `Ctrl+Alt+P`: play or pause the Show
- `Ctrl+Alt+Right` and `Ctrl+Alt+Left`: go to the next or the previous Show cue entry
- `Ctrl+,`: open `Settings`
- `Ctrl+Q`: close the app

Keys of the active list:

- `Esc`: go back
- `Up`, `Down`, `Page Up`, `Page Down`, `Home` and `End`: move the selection
- `Enter`: open the selected item
- `Space`: select the focused row
- `Ctrl+A`: select all visible rows
- `Right` and `Left`: open or close the focused group

## Docs

- [Docs index](docs/README.md)
- [App overview](docs/architecture/app-overview.md)
- [Library and discovery workflows](docs/runbooks/workflows.md)
- [Storage and metadata model](docs/schema/storage-and-metadata.md)
- ADRs remain in [`docs/adr/`](docs/adr/)
