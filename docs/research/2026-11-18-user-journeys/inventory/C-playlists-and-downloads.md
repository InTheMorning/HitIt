<!-- Recorded output of Claude inventory run C (sonnet) at commit 79c23c1. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

Recorded output of Claude inventory run C at commit 79c23c1a95a817a3d2ff2f4aba1cd9b6dd423391.

## 1. Reading this inventory

This inventory describes code paths. The app was not started. No file was changed. It gives no design proposal and no rule.

Scope: playlists (My playlists and Public playlists) and downloads (queue, progress, stop, notices, failures, audio cache, "Remove from the Library", confirmations, cached files in Settings).

Conventions:

- Labels are quoted as the source has them. A template uses braces, for example "Add {n} {noun} to playlist".
- Paths are repository-relative. A reference is `file:line` where I read the line. A reference with no line number names the function only.
- **INFERENCE** marks a count or a statement that I derived from code and did not observe.
- **UNSURE** marks a fact that is not in the code (native dialog text, mpv behavior, timing) or that I did not read to the end.
- A step is one click, one key, one typed field or one list selection. A wait or an automatic action is zero steps.
- "Music open" is the start state of each baseline job. Prefix steps to reach a search result come from the sibling inventory `B-music-search-and-item-pages.md` section 6 and are INFERENCE here.
- Files that I did not read to the end: `src/library/download_queue.rs` (queue internals), `src/audio_cache.rs`, `src/ui/shells/playlist.rs` (page shell below the row level), `src/app/keyboard.rs` (only the "NewPlaylist" binding), `src/ui/composites/track_drag.rs`. Statements about them are UNSURE.

## 2. Entry points

### 2.1 Playlist entry points

| Entry point | Quoted label | Evidence |
|---|---|---|
| Sidebar destination heading | "Playlists" | `src/view_models/library.rs` (`playlist_sidebar`, ~3990) |
| Playlists page head action | "New Playlist" (id `playlists-add`) | `src/library/app_impl.rs:6396` |
| Inline new-playlist row | placeholder "New playlist name…", button "Add" | `src/library/app_impl.rs:1124`, `src/ui/shells/library/my_playlists.rs` |
| Keyboard | "cmd-n" (NewPlaylist) opens Playlists and focuses the input | `src/app/keyboard.rs:140`, `src/library/app_impl.rs:1579` |
| Playlist row (list) | row a11y label "Playlist: {name}"; row type tag "Playlist"; count "1 track" or "{n} tracks" | `src/ui/shells/library/destination_lists.rs`, `src/view_models/library.rs` |
| Playlist row menu | "Rename", "Move Up", "Move Down", "Delete…", "Load into Show Cue" or "Replace Show Cue" | `src/view_models/library.rs:705-760`, `src/library/app_impl.rs:2107` |
| Playlist page bar | "Delete…", "Rename", the Check RSS label from the view model, Show Cue | `src/view_models/library.rs:6950-7000`, `src/ui/shells/playlist.rs` |
| Track row menu | "Add to Playlist", "Download", "Inspect Sources", "Remove…", "Move Up", "Move Down", "Remove from Playlist", "Remove from Library…"; trigger tooltip "More actions" | `src/view_models/release_page.rs:47-65` |
| Release page action | "Add to Playlist" (popover id `album-feed-add:{feed_id}`) | `src/view_models/album_page.rs:35-140`, `src/ui/shells/release_page.rs:160-240` |
| Release row menu | "Download {Kind}", "Add to Playlist", "Remove {Kind}…" | `src/view_models/release_row_actions.rs:211-260` |
| Selection bar | "Add {n} {noun} to playlist"; playlist bar "Add {selected} to playlist" | `src/view_models/selection_action_bar.rs` |
| Track drag onto a playlist row | no label; spring-load opens "Playlists" | `src/ui/shells/library/my_playlists.rs` (`add_tracks_to_playlist`) |
| Public playlist row menu | "Copy to My Playlists…" (tooltip "More actions") | `src/view_models/public_playlists.rs:224`, `:885` |

### 2.2 Download entry points

| Entry point | Quoted label | Evidence |
|---|---|---|
| Release page plain button | "Download {kind}" (disabled while Queued or Downloading) | `src/view_models/album_page.rs:35-140` |
| Release row menu | "Download {Kind}" | `src/view_models/release_row_actions.rs:234` |
| Track row menu | "Download" | `src/view_models/release_page.rs:47-65` |
| Selection bar | "Download {n} {noun}"; playlist bar "Download {n} track(s)" | `src/view_models/selection_action_bar.rs` |
| Playlist page, RSS difference row | `on_download_added_track` slot calls `download_library_tracks` | `src/ui/shells/library/playlist_detail.rs:245`, `:268-279` |
| Progress item | "Stop queued", "Dismiss" | `src/view_models/download_progress.rs:287`, `:305` |
| Settings, Library, section "Audio cache" | "Cached files" or "Cached files ({count})", "Delete All Cached", per row "Delete" | `src/view_models/cached_files.rs:78-129`, `src/view_models/settings.rs:77` |
| Removal | "Remove {Kind}…", "Remove…", "Remove from Library…", "Remove {n} {noun}…" | see section 9 |

## 3. Journey: make a playlist

### 3.1 From the Playlists page

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Playlists" in the sidebar | The playlist list opens | `src/view_models/library.rs` (`playlist_sidebar`) |
| 2 | Click "New Playlist" (or press cmd-n, which also does step 1) | The inline row opens with the input, placeholder "New playlist name…", and the button "Add" | `src/library/app_impl.rs:6396`, `:1579`, `:1124-1135` |
| 3 | Type the name | The input shows the text | `src/library/app_impl.rs:1124-1135` |
| 4 | Click "Add" | The inline row closes, the list reloads, and the new playlist page opens (`PlaylistDetail(id)`, `CurrentPath`) | `src/library/app_impl.rs:2010` |

INFERENCE: 4 steps from Music open (3 when cmd-n is used, not counting the key as a second step: key 1, type 1, click 1 = 3).

Branches and errors:

- Enter in the new-playlist input does nothing. Only the rename input subscribes to PressEnter. Evidence: `src/library/app_impl.rs:1124-1135`.
- A blank name returns with no text and does not clear the input. Evidence: `src/library/app_impl.rs:2010` (`create_playlist`).
- On error the status text is "Error creating playlist: {e}". It shows in the sidebar footer (section 18).
- End state: the new playlist is open and empty.

### 3.2 From the playlist popover ("New Playlist", "Create & Add")

See section 7. This path makes the playlist and adds the item in one step.

### 3.3 From a public playlist

See section 6.

## 4. Journey: add items to a playlist

### 4.1 One track (Library row or Index row on a release page)

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Hover or focus the row, click the "⋯" cell (tooltip "More actions") | The row menu opens | `src/view_models/release_page.rs:47-65` |
| 2 | Click "Add to Playlist" | The menu shows the playlist level: "Back", the playlist names, "New Playlist" | `src/ui/composites/track_row_menu.rs` |
| 3 | Click a playlist name | The command runs; the status shows "Downloading {n} track(s)..." when a download is needed, then "Added {appended} of {total} to {name}" with " (downloaded {n})" and "; {n} failed" when they apply | `src/library/app_impl.rs` (`add_track_to_playlist`, `spawn_subscribe_then_append`) |

INFERENCE: 3 steps from the page that holds the row. The "⋯" cell is out of the Tab sequence and shows only on hover or focus (UNSURE: keyboard route to the menu, other than the right-click menu that the playlist page has).

Index row: the add first downloads the track (synchronous and sequential inside the command), then appends it. Evidence: `src/library_service.rs:130-190` (`subscribe_then_append_with`). Index error texts go to `settings_status`: "Cannot add track without a Music Index feed id", "Error preparing feed: {error}", "Cannot add track until the feed is indexed locally", "Feed has no tracks to add", "Error adding to playlist: {error}"; success is "Added {n} track(s), downloaded {m}". Evidence: `src/app/search_dispatch.rs:2017-2100`.

### 4.2 One release (whole album)

- Library release page, InLibrary: click "Add to Playlist" (1), click a playlist (1) = 2 steps. Evidence: `src/view_models/album_page.rs`, `src/ui/shells/release_page.rs:160-240`.
- Library release page, RemoteOnly: only "Download" appears. Queued or Downloading: a disabled "Download". Removing: a disabled "Add to Playlist".
- Index release page: only the plain button "Download {kind}", and "Copy Feed URL" in the "⋯" menu. There is no "Add to Playlist". The `AddReleaseToPlaylist(_)` and `CreatePlaylistForRelease(_)` run commands are empty handlers, but the view model does not offer them for the Index. Evidence: `src/view_models/album_page.rs`, `src/app/search_dispatch.rs:1618-1628`.
- Release row menu in a list: "Add to Playlist" if the row state allows it. Evidence: `src/view_models/release_row_actions.rs:211-260`. UNSURE: which row states allow it (I did not read `ReleaseRowAvailability`).
- Messages: "Album has no tracks"; "Error loading album tracks: {e}".

### 4.3 Several selected rows

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Select" in the list bar | Checkboxes appear. The label changes to "Done". The row menu cell is absent | `src/ui/composites/selection_action_bar.rs` (`render_list_bar`) |
| 2..n | Click each row or checkbox | The count label shows "{n} {noun} selected" | `src/view_models/selection_action_bar.rs` |
| n+1 | Click "Add {n} {noun} to playlist" | The playlist popover opens | `src/app/search_selection.rs:300`, `src/app/detail_selection.rs:603-655` |
| n+2 | Click a playlist, or "New Playlist" then type, then "Create & Add" | The tracks are added | `src/ui/composites/playlist_popover.rs` |

Rules from the code:

- In a Tracks selection, the "Add" action counts only rows with `library_track_id`. A selection of Index-only rows gives no "Add … to playlist" action. Evidence: `src/view_models/detail_selection.rs:600`, `src/app/detail_selection.rs:661` (`detail_library_ids` drops Index rows).
- The playlist choices leave out the open playlist on a playlist page.
- "Select all" exists in the bar. "Clear" clears the selection.

### 4.4 Drag a track onto a playlist row

A track drag onto a playlist row in the Playlists list adds the track (`add_tracks_to_playlist`). Hovering over the "Playlists" destination opens it (spring-load). The Tiles view has no drag and no menu. Evidence: `src/ui/shells/library/my_playlists.rs`. UNSURE: the hover delay and what a drag of an Index row does (I did not read `track_drag.rs`).

### 4.5 Playlist popover details

See section 7.

## 5. Journey: manage a playlist

### 5.1 Rename

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open the playlist page (click the row) | The page bar shows "Delete…", "Rename" | `src/ui/shells/playlist.rs` |
| 2 | Click "Rename" (a11y label "Rename playlist"), or "Rename" in the row menu | The rename input appears with the placeholder "Playlist name", and "Save" and "Cancel" | `src/view_models/library.rs:6950-7000`, `src/library/app_impl.rs:2156` |
| 3 | Type the new name | The input shows the text | |
| 4 | Click "Save" or press Enter | The page shows the new name | `src/library/app_impl.rs:2091` (`submit_playlist_rename`) |

INFERENCE: 4 steps (3 from the list row menu if the page is already open).

A blank name is ignored with no text. The command layer fails a blank name with "playlist command failed". The error text is "Error renaming: {e}". Evidence: `src/library/app_impl.rs:2040`, `src/application/commands/playlist.rs`.

### 5.2 Delete

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Delete…" (page bar or row menu) | A confirmation opens with the title `Delete the playlist "{name}"?` and the message "It has {n} entry/entries. The tracks stay in the Library." | `src/view_models/library.rs` (`playlist_delete_confirmation`), `src/ui/shells/library/playlist_detail.rs:224` |
| 2 | Click "Delete" (or "Cancel", a11y "Keep the playlist") | The playlist goes. Error text: "Error deleting: {e}" | `src/library/app_impl.rs:2276` |

INFERENCE: 2 steps.

### 5.3 Reorder tracks

- Drag the row by the handle (tooltip "Drag to reorder playlist track"). The handle appears on hover or focus and is replaced by a checkbox in selection mode. A drop onto another row reorders. A drop from another playlist is rejected (line-through style). Evidence: `src/ui/shells/playlist.rs`, `src/view_models/library.rs`. The drag works in Curator Order (UNSURE: the sort condition; I recorded it as such from the playlist reorder code and did not re-read the guard).
- Row menu "Move Up" and "Move Down": 2 steps for each move (the "⋯" cell, then the item). Evidence: `src/view_models/release_page.rs:47-65`, `src/library/app_impl.rs` (`move_playlist_track`).
- Selection bar "Move to top" and "Move to bottom". Evidence: `src/view_models/selection_action_bar.rs` (`for_playlist`), `src/library/app_impl.rs:2416`.
- Error text: "Error reordering: {e}".
- Reorder of the playlists in the list: row drag, `move_playlist_to`, error "Error moving the playlist: {e}". Row menu "Move Up" and "Move Down".

### 5.4 Remove a track from a playlist

- Row menu "Remove from Playlist" (1 + 1 = 2 steps). Evidence: `src/ui/shells/library/playlist_detail.rs:421`.
- Selection bar "Remove {selected} from playlist" (Select, check, action).
- No confirmation for these two (no dialog call in `remove_playlist_track_at` or `remove_selected_playlist_entries`).
- A removed track (RSS check) gives "Remove from all playlists", which opens a confirmation that names each playlist. Evidence: `src/library/app_impl.rs:2320`, `src/ui/shells/library/playlist_detail.rs:281-289`. UNSURE: the dialog text (`src/ui/shells/playlist_removal_confirmation.rs` not read).
- Error text: "Error removing track: {e}".

### 5.5 Show Cue (load a playlist into the show)

The page bar has a Show Cue button. It runs `LoadShowCue` (label "Load into Show Cue" or "Replace Show Cue"). Errors: "The Show cue did not change: {e}" and "The playlist has no tracks. The Show cue did not change." Evidence: `src/ui/shells/library/playlist_detail.rs:235`, `src/library/app_impl.rs:2156`.

### 5.6 Check RSS

The "Check RSS" button (label from the view model) starts a feed check of the playlist. The report shows on the page. The actor error is "The RSS check actor stopped, or it has too many waiting requests. Try again." Evidence: `src/ui/shells/library/playlist_detail.rs:257`, `src/application/commands/playlist.rs`.

## 6. Journey: public playlists, copy to My Playlists

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the "Public playlists" destination | The list loads, with a loading row, an error row or "Load more" | `src/view_models/public_playlists.rs:50-61`. UNSURE: the exact sidebar label. |
| 2 | Click "More actions" on a row | The menu shows "Copy to My Playlists…". The menu is absent while a copy runs | `src/view_models/public_playlists.rs:224-235` |
| 3 | Click "Copy to My Playlists…" | The dialog "Copy to My Playlists" opens. Body: "Copy {title} to a new local playlist. The copy downloads no file." Input placeholder "Name of the new playlist". Buttons "Copy" and "Cancel" | `src/view_models/public_playlists.rs:30-36`, `:255` |
| 4 | Type a name | | |
| 5 | Click "Copy" or press Enter | The row shows "Copying to My Playlists…". The report appears | `src/library/public_playlists.rs:388-484` |

INFERENCE: 5 steps from Music open, plus the destination click.

Messages (all from `src/view_models/public_playlists.rs`):

- Blank name: "Enter a name for the playlist."
- Name taken: `A playlist named "{name}" already exists. Enter another name.` (it also shows when the command returns `NameTaken`).
- Success: "Copied {copied} of {total} {noun} to {name}. {n} could not be read."
- No readable tracks: the constant `PUBLIC_PLAYLIST_NO_READABLE_TRACKS`, "The app could not read a track of this playlist, so it made no playlist."
- Other errors: `set_error_status` with the command error text.

The copy downloads no file. An added track later downloads when it is added to a playlist from the Index (section 4.1) or when "Download" runs. UNSURE: whether a copied track is playable without a download.

## 7. The playlist popover (shared)

File: `src/ui/composites/playlist_popover.rs`.

| Mode | Control | Label | Line |
|---|---|---|---|
| List | empty text | "No playlists yet" | 357 |
| List | one button per playlist | the playlist name | |
| List | create button (only when `on_create` is set) | "New Playlist" with the Add icon | 364-370 |
| Create | back | "Back" | ~404 |
| Create | input | placeholder "Playlist name" | 191 |
| Create | confirm | "Create & Add" | 408-430 |

- A blank or whitespace name returns with no text (lines 414-417).
- Open: click, or Enter or Space (`on_key_activate`, 243-249). Close: Escape or an outside click (`overlay_closable`).
- Accessibility labels in the bar: "Create a new playlist", "Back to playlist choices", "Create playlist and add the selection". On the release page: "Create playlist and add album".
- The row menu "Add to Playlist" uses the same names in its own level: "Back", the choices, "New Playlist", "Create & Add".
- Playlist row menus pass empty select and create handlers (the playlist level is empty). Evidence: `src/library/app_impl.rs:2132-2135`.

## 8. Journey: downloads (queue, progress, stop, finish)

### 8.1 Start

Each "Download" control (section 2.2) queues a job. A Library release or track job and an Index release job have separate queue keys. Evidence: `src/library/downloads.rs`, `src/app/search_dispatch.rs:1974`. UNSURE: the order and concurrency of jobs (`download_queue.rs` not read to the end).

### 8.2 Progress item

Mount: `TopApp::render` mounts the item through `DownloadStatusView` on each section. Evidence: `src/library/column_view.rs:60-81`, `src/library/downloads.rs:261`. The item appears only while a batch is open.

| State | Title | Other lines | Actions |
|---|---|---|---|
| Running | "Downloading {position} of {total}" | the oldest job in flight; "{n} failed" text; one line per failure | "Stop queued" when one or more jobs wait; a11y "Stop {queued} queued download(s)" |
| Running, host wait | "Downloading {position} of {total}. Waiting for {host} until {HH:MM:SS}" | "{host} sent HTTP 429 (Too Many Requests) at {time}. The app tries again at {time}. Try {n} of {max}." or "The app waits for {host} until {time}. Try {n} of {max}." | "Stop queued" |
| Finished | "Downloaded {done} of {total}" plus ", {n} failed" and ", {n} stopped", then ". Finished {HH:MM}" | failure lines | "Dismiss" (a11y "Dismiss the download report") |

Evidence: `src/view_models/download_progress.rs:160-210`, `:287-305`, `:380-425`.

- "Stop queued" removes each waiting job. A job in flight keeps writing its file. Evidence: `src/library/downloads.rs:250`.
- A failure line reads "The app could not download {label}: {error}". Evidence: `src/view_models/download_progress.rs:210`.
- There is no cancel for a job in flight (INFERENCE from the stop code and the comment on partial files).
- "Dismiss" removes a finished report. A running batch cannot be dismissed.
- Release state labels: "Not downloaded", "Queued", "Downloading", "Partly downloaded", "Downloaded", "Failed". Evidence: `src/view_models/release_download_state.rs:56-61`.
- Removal refusal: "The app did not remove {release_name}: a download of this release is queued or running. Remove it after the download ends." Evidence: `src/library/download_queue.rs:544`.

### 8.3 Track subscribe and conversion

`SetTrackLibraryMembership` gives "Subscribed track" or "Unsubscribed track". `finish_track_subscribe` gives "Downloaded track: {path}". Retry of a conversion gives "Original track retry finished". Evidence: `src/application/commands/download.rs`, `src/view_models/library.rs`.

## 9. Journey: remove from the Library

### 9.1 Entry points and confirmation rule

| Entry | Label | Confirmation | Evidence |
|---|---|---|---|
| Release page "⋯" menu | "Remove {kind}…" | Always | `src/library/app_impl.rs:3411` |
| Release row menu | "Remove {Kind}…" (a11y "Remove the {noun} {title} from the Library…") | Always, UNSURE (the row menu route to `remove_album_after_confirmation` was not traced) | `src/view_models/release_row_actions.rs:250` |
| Track row menu (Library row) | "Remove…" (only when downloaded with no work in progress) | Always | `src/library/app_impl.rs:946` |
| Playlist row menu | "Remove from Library…" | Always | `src/library/app_impl.rs:946` |
| Track page | "Remove track…" | Always (`remove_track_after_confirmation`, line 3500) | `src/library/app_impl.rs:3500` |
| Track page subscription toggle | the Remove action of `TrackSubscriptionAction::Remove` | `WhenReferenced`: the dialog opens only if a playlist or the Show cue holds the track | `src/library/app_impl.rs:3937`, `src/application/library_removal.rs:75` |
| Selection bar | "Remove {n} {noun}…"; playlist bar "Remove from Library…" | dialog "Remove {count} {noun} from the Library?" | `src/view_models/library_removal.rs:258` |

### 9.2 Dialogs

- Track: title "Remove Track from Library?", buttons "Cancel" (a11y "Cancel removing track from library") and "Remove" (a11y "Remove track from library"). Evidence: `src/view_models/library_removal.rs:62-69`.
- Album: title "Remove Album from Library?", buttons "Cancel" and "Remove" (a11y "Remove album from library"). Evidence: `src/view_models/library_removal.rs:72-79`.
- Feed: cancel a11y "Cancel removing feed from library" (`src/ui/shells/library_removal_confirmation.rs:81`).
- Selection: title "Remove {count} {noun} from the Library?", a11y "Cancel removing the selection from the library" and "Remove the selection from the library".
- Impact text for a track: "This track is in no playlist. Removing it takes it out of the library." / "This track is in {n} {playlist(s)}. Removing it from the library will make it unavailable for playlist playback." / "The Show cue has {n} {entry} for this track. Removing it from the library makes it unavailable for playback." / the combined form. For an album: "No track from this album is in a playlist. Removing the album takes its tracks out of the library." plus other forms. Evidence: `src/view_models/library_removal.rs:95-150`.

### 9.3 What the command does

`execute_library_removal` moves the local files to the audio cache (`audio_cache::put_files`) and moves the database rows to the cache. It does not delete them. Evidence: `src/application/library_removal.rs:213-270`. Settings text: "Files that a Library removal moved out of the music folder. Delete removes a file for good." Evidence: `src/view_models/settings.rs:199`.

For a release, the removal first checks the download queue (refusal in 8.2) and shows "Removing feed..." (`begin_feed_removal`). On success the list reloads. Errors: `set_error_status` ("Error: {e}") or, for a selection, the report "The app removed {removed} of {total} {noun}{s}. It could not remove: {title} ({error}), …." A full success is silent. Evidence: `src/view_models/library_removal.rs:363`.

## 10. Journey: delete cached files (Settings)

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Settings" in the sidebar | Settings opens | `src/view_models/sidebar.rs` |
| 2 | Open the "Library" group (INFERENCE: it is a group of Settings) | Sections "Music Index", "Files", "Audio cache" | `src/view_models/settings.rs:65-82` |
| 3 | Click "Delete All Cached" (a11y "Delete all cached music files"), or "Delete" on a row (a11y "Delete cached file for {title}") | The command runs. The status line shows "Deleted {n} cached file(s)". The list reloads | `src/app/settings.rs:315-365`, `src/app.rs:1529-1565`, `src/application/commands/download.rs:317` |

INFERENCE: 3 steps from Music open for one file or all files.

- No confirmation dialog for either action (no dialog call in `delete_cached_files`).
- Row title "Cached files" or "Cached files ({count})". Rows list artist names and tracks. "Delete All Cached" shows only when the count is above 0.
- Disabled when the command runner is not available, or (per row) when the file has no path.
- Notices: "App has not read the cached-file list.", "App is reading the cached-file list.", "App could not read the cached-file list because background tools are unavailable. Choose Check again in Diagnostics, Background tools.", "App could not read the cached-file list from the database. Reopen Settings to try again.", "No cached files.", and " App is showing the last loaded list." Evidence: `src/view_models/cached_files.rs:128-150`.
- The result text goes to `settings_status`. It shows on the Settings page when not in the workspace form (`src/app/settings.rs:224`). Error text: "Error: {error}".

## 11. Baseline Job 5: add an Index track found by search to a playlist

Start: **Music open**.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the search field (placeholder "Search Library and Index") | Focus | B inventory section 6 |
| 2 | Type the query | Results after about 300 ms | B inventory section 6 |
| 2a | Press Enter (optional) | Search runs at once | B inventory section 6 |
| 3 | Click the "Tracks" tab | The track rows show | `src/view_models/search_results/tabs.rs:50` |
| 4 | Click the Index track result | The Index track page opens | `src/app/search_dispatch.rs:739-770` |
| 5 | Click the album name link | The Index release page opens | `src/app/search_dispatch.rs:1745-1760` |
| 6 | Click "⋯" on the track row | The row menu opens | `src/view_models/release_page.rs:47-65` |
| 7 | Click "Add to Playlist" | The playlist level opens | `src/ui/composites/track_row_menu.rs` |
| 8 | Click a playlist (or "New Playlist", type, "Create & Add" = 3 steps) | The track downloads, then is appended | `src/library_service.rs:130-190` |

**INFERENCE count: 8** (7 with the typed Enter omitted, 9 if the typed query needs Enter; 10 with a new playlist: steps 8a, 8b, 8c).

Facts that decide the path:

- A search row has no row menu with "Add to Playlist". Evidence: no `add_to_playlist` in `src/view_models/search_results/*.rs`.
- Search selection mode gives no "Add … to playlist" action for Index-only tracks (section 4.3).
- The Index track page offers no action: `DownloadTrack`, `AddToPlaylist`, `MusicBrainzLookup`, `InspectSources` and `RemoveTrack` are empty handlers there, and the view model offers the plain "Download Release". Evidence: `src/app/search_dispatch.rs:1680-1692`, `src/view_models/track_page.rs:977`.
- Step 5 relies on the album name link of the track page. UNSURE: that the link shows for each Index track.
- If the track is already a Library track, the result opens a Library track page (`open_library_track_result`). The count there is UNSURE (the Library track page offers "Add to Playlist" in the TrackPageAction set, which I did not read).
- Result and errors show away from the control (section 18).

## 12. Baseline Job 6: create a playlist and add an album

Start: **Music open**. Assume the release is in the Library as InLibrary.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Releases" | The release list | B inventory section 13 |
| 2 | Click the release row | The Library release page | `src/app/search_dispatch.rs:739` |
| 3 | Click "Add to Playlist" | The popover opens | `src/ui/shells/release_page.rs:160-240` |
| 4 | Click "New Playlist" | Create mode | `src/ui/composites/playlist_popover.rs:364` |
| 5 | Type the name | | |
| 6 | Click "Create & Add" | The playlist is made. The album tracks are appended | `src/library/app_impl.rs` (`create_playlist_and_add_albums`) |

**INFERENCE count: 6.** Alternative from the release row menu: "⋯", "Add to Playlist", "New Playlist", type, "Create & Add" = 6 from the list.

If the release is not in the Library (an Index release):

- The Index release page has no "Add to Playlist". The path is "Download {kind}" (1 step), then wait until the state is "Downloaded", then steps 3 to 6 above. **INFERENCE count: 7 from the Index release page** (the wait is 0 steps).
- The Index release page also can use the per-track path (section 4.1): 3 steps for the first track plus the playlist creation (5 steps for the first track), then 3 steps each for the other tracks.
- The playlist page does not open after "Create & Add" (UNSURE; the code opens the new page for the inline "Add" path in 3.1 only).

## 13. Baseline Job 7: add 3 tracks of a release to a playlist

Start: **Music open**. Assume a Library release page is open (steps to open it are 2: "Releases", click the row).

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Select" | The checkboxes appear. The label is "Done" | `src/ui/composites/selection_action_bar.rs` |
| 2 | Click the first track | "1 track selected" (the bar count label) | `src/view_models/selection_action_bar.rs` |
| 3 | Click the second track | | |
| 4 | Click the third track | | |
| 5 | Click "Add 3 tracks to playlist" | The popover opens | `src/app/detail_selection.rs:603-655` |
| 6 | Click a playlist | The tracks are added | |

**INFERENCE count: 6 from the release page, 8 from Music open.**

Other paths:

- Per-track menu: 3 steps for each track, 9 steps for three.
- On an Index release page, selection gives no "Add … to playlist" action (only Library rows count). The per-track menu path is the only path: 9 steps for three tracks (the playlist is chosen each time), plus a downloaded track after each add.
- Shift or Ctrl click ranges in selection mode: UNSURE (`click_detail_row` takes modifiers; I did not read the rules). They could reduce steps 2 to 4.

## 14. Baseline Job 10: drop a release and free its disk space

Start: **Music open**. Assume the release is downloaded in the Library.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Releases" | The list | B inventory section 13 |
| 2 | Click "⋯" of the row | The row menu | `src/view_models/release_row_actions.rs:211-260` |
| 3 | Click "Remove {Kind}…" | The dialog "Remove Album from Library?" | `src/view_models/library_removal.rs:72` |
| 4 | Click "Remove" | The files move to the audio cache. The list reloads | `src/application/library_removal.rs:213-270` |
| 5 | Click "Settings" | Settings opens | |
| 6 | Open the "Library" group (INFERENCE) | "Audio cache" shows | `src/view_models/settings.rs:65-82` |
| 7 | Click "Delete All Cached" | "Deleted {n} cached file(s)" | `src/application/commands/download.rs:317` |

**INFERENCE count: 7.** Steps 1 to 4 drop the release from the Library, but the disk space is not free yet: `execute_library_removal` moves the files to the audio cache. Steps 5 to 7 delete them for good. Settings text: "Delete removes a file for good."

Branches:

- A queued or running download of the release refuses the removal with the text in 8.2. The text shows in the sidebar footer (section 18).
- "Delete All Cached" deletes all cached files, not only those of this release. A per-file "Delete" is a 1 step action per track, with the name list in the same section (INFERENCE: so a release of 10 tracks needs 10 clicks). Rows list tracks under the artist name and no album name (`CachedFileRow::Artist` and `Track` only).
- No confirmation for step 7.
- From the release page: "⋯" (1), "Remove {kind}…" (1), "Remove" (1) = 3 steps from the release page.
- UNSURE: the cache folder is `config::audio_cache_dir()`; whether the user can free space by any other control, and whether the app empties the cache by itself.

## 15. MH2 Build a show playlist

Goal: the playlist exists, is ordered, and each track can be got.

### 15.1 Make the playlist

Counts from Music open: 4 steps ("Playlists", "New Playlist", type, "Add"). With cmd-n: 3 steps. With the popover: the first add makes the playlist (section 7).

### 15.2 Add each track (count for each add)

| Source | Steps for each track | Evidence |
|---|---|---|
| Library track on a release page or a playlist page | 3 ("⋯", "Add to Playlist", playlist) | section 4.1 |
| Library tracks, selected | 1 (Select) + 1 for each track + 2 (bar action, playlist) in total | section 4.3 |
| Index track | 3 on the Index release page, plus the steps to reach that page (search: 4 to 5) | section 11 |
| Whole Library release | 2 (section 4.2) | |
| Public playlist | 5 for the whole playlist (section 6) | |
| Drag onto a playlist row | 1 drag, with the "Playlists" list open | section 4.4 |

The playlist page does not offer a control to add a track from inside the page. Its own row menu has "Add to Playlist" for the other playlists only (the open playlist is left out). Evidence: `src/ui/shells/library/playlist_detail.rs:124-139`.

### 15.3 Order

- Appended tracks go to the end. UNSURE: the order of a multi-select add (selection order or list order).
- Drag handle (tooltip "Drag to reorder playlist track"): 1 step for each move. UNSURE: the sort condition for the drag.
- "Move Up" and "Move Down" in the row menu: 2 steps for each place moved. "Move to top" and "Move to bottom" in the selection bar: Select (1), check (1), action (1) = 3 steps.
- The position label shows "{n}.".

### 15.4 Each track can be got

- Adding an Index track to a playlist downloads it first (section 4.1). The result string names the failed count: "Added {appended} of {total} to {name}" and "; {n} failed". The names of failures are in the command result strings ("{title}: {err}", "track {id} not found", "append {title}: {err}") but not on a page. UNSURE: whether the sidebar status shows the names.
- A track that is not local shows "Unavailable" in the playlist row (the row availability label). The playlist page has "Download" in the row menu and in the selection bar ("Download {n} track(s)").
- "Check RSS" and its report mark a track that left the feed (removed rows with "Remove from all playlists" and a confirm step). Evidence: `src/ui/shells/library/playlist_detail.rs:281`.
- "Load into Show Cue" or "Replace Show Cue" loads the playlist into the show. It fails with "The playlist has no tracks. The Show cue did not change." for an empty playlist.

MH2 is possible now. It has no single control that makes an ordered playlist from a release or a search with all tracks got. The count depends on the source in the table.

## 16. Controls with no handler or a handler that does nothing

| Control or command | Behavior | Evidence |
|---|---|---|
| `ReleasePageCommand::AddReleaseToPlaylist(_)`, `CreatePlaylistForRelease(_)`, `RemoveRelease` on an Index release page | Empty handler. The view model does not offer them | `src/app/search_dispatch.rs:1618-1628` |
| `TrackPageAction::DownloadTrack`, `AddToPlaylist`, `MusicBrainzLookup`, `InspectSources`, `RemoveTrack`, and `TrackPageCommand::AddToPlaylist(_)`, `CreatePlaylist(_)` on an Index track page | Empty handler. The page does not offer them | `src/app/search_dispatch.rs:1680-1692` |
| `TrackRowMenuAction::AddToPlaylist`, `MoveUp`, `MoveDown`, `RemoveFromPlaylist`, `CopyToMyPlaylists`, `RenamePlaylist`, `DeletePlaylist`, `LoadShowCue` in the Library row menu action runner | `return` with no effect. Other runners handle them | `src/library/app_impl.rs:949-957` |
| Playlist row menu select and create handlers | Empty (the playlist level is empty) | `src/library/app_impl.rs:2132-2135` |
| Enter in the new-playlist input | No subscription. Nothing happens | `src/library/app_impl.rs:1124-1135` |
| "Create & Add" and "Add" with a blank name | Return with no text and no clear | `src/ui/composites/playlist_popover.rs:414-417`, `src/library/app_impl.rs:2010` |
| Rename with a blank name | Return with no text | `src/library/app_impl.rs:2040`, `:2091` |
| Tiles view of playlists | No drag and no menu | `src/ui/shells/library/my_playlists.rs` |
| "Add to Playlist" of a release that is not InLibrary | Absent or disabled (not a no-op handler) | `src/view_models/album_page.rs` |

## 17. Labels that differ for one action

| Action | Labels | Evidence |
|---|---|---|
| Add to a playlist | "Add to Playlist" (menu, release page), "Add {n} {noun} to playlist" (bar), "Add {selected} to playlist" (playlist bar) | `src/view_models/release_page.rs`, `src/view_models/selection_action_bar.rs` |
| Make a playlist | "New Playlist" (page head, popover), "Add" (inline button), "Create & Add" (popover), a11y "Create playlist and add album", "Create playlist and add the selection", "Create a new playlist" | `src/library/app_impl.rs:6396`, `src/ui/composites/playlist_popover.rs` |
| Placeholders for a playlist name | "New playlist name…" (inline), "Playlist name" (popover and rename), "Name of the new playlist" (copy dialog) | `src/library/app_impl.rs:1124`, `src/view_models/public_playlists.rs:36` |
| Remove from the Library | "Remove…" (track menu), "Remove from Library…" (playlist row menu, playlist bar), "Remove {Kind}…" (release), "Remove {n} {noun}…" (bar), "Remove track…" (track page) | section 9.1 |
| Dialog buttons for removal | "Remove" (Library removal), "Delete" (playlist), "Copy" (public copy) | `src/view_models/library_removal.rs`, `src/view_models/library.rs`, `src/view_models/public_playlists.rs:32` |
| Remove from a playlist | "Remove from Playlist" (row menu), "Remove {selected} from playlist" (bar) | |
| Delete files | "Delete" (row), "Delete All Cached" (Settings), "Remove…" (Library, moves to cache) | `src/view_models/cached_files.rs` |
| Download | "Download", "Download {kind}", "Download {Kind}", "Download Album", "Download Release", "Download {n} {noun}", "Download {n} track(s)" | section 2.2 |
| Stop a download | "Stop queued" (visible), "Stop {n} queued download(s)" (a11y) | `src/view_models/download_progress.rs:448` |
| Copy a public playlist | "Copy to My Playlists…" (menu), "Copy to My Playlists" (dialog title), "Copy" (button) | `src/view_models/public_playlists.rs:30-34` |
| Playlist name unavailable | "[untitled]", "Untitled playlist", "Untitled item", "Unknown" | `src/view_models/library.rs`, `src/view_models/public_playlists.rs:44-46` |
| Show cue | "Load into Show Cue", "Replace Show Cue" (same command with a state) | `src/view_models/library.rs` |

## 18. Results and errors that show away from their control

| Action | Result or error | Where it shows | Evidence |
|---|---|---|---|
| Add a track or release to a playlist (Library) | "Downloading {n} track(s)...", "Added {appended} of {total} to {name}", "Error adding to playlist: {e}" | `LibraryViewModel.status`, in the sidebar footer (`render_sidebar_column`), not near the row or popover | `src/library/app_impl.rs:5843-5910` |
| Add an Index track to a playlist | "Cannot add track…", "Error preparing feed: {error}", "Added {n} track(s), downloaded {m}", "Error creating playlist: {error}" | `settings_status`. It shows on the Settings page (`src/app/settings.rs:224`) and as the Show status message (`src/app/show.rs:475`). Not near the control | `src/app/search_dispatch.rs:2017-2100` |
| Create, rename, delete, reorder, remove in the Library | "Error creating playlist: {e}", "Error renaming: {e}", "Error deleting: {e}", "Error reordering: {e}", "Error removing track: {e}" | sidebar footer status | `src/library/app_impl.rs` |
| Library removal | refusal text, "Removing feed...", errors | sidebar footer status; a selection report also shows there | `src/library/app_impl.rs:3561-3600` |
| Public playlist copy | Report and error | the row of the playlist (`PublicPlaylistCopyReport`) and the sidebar footer status | `src/library/public_playlists.rs:463-484` |
| Delete cached file | "Deleted {n} cached file(s)", "Error: {error}" | `settings_status`, shown at the bottom of the Settings content, after the cached list (the Delete All button is in a row above) | `src/app/settings.rs:224`, `src/app.rs:1529-1565` |
| Download job | progress, failures, stop and finish text | the progress item mounted on each section, not at the control | `src/library/column_view.rs:60-81` |
| Playlist track download from a feed difference | the progress item | the progress item | `src/ui/shells/library/playlist_detail.rs:268-279` |
| Show Cue | "The Show cue did not change: {e}" | UNSURE: where it renders | `src/library/app_impl.rs:2156` |

Fact from the code: the Library status line sits in the sidebar footer. Below `SHELL_TAB_BAR_BREAKPOINT` (940 px) the window shows the tab bar in place of the sidebar. So the `LibraryViewModel.status` text may have no place on screen in the narrow layout (INFERENCE; UNSURE whether another element shows it. I found no other render of `status_snapshot` outside `render_sidebar_column`).

## 19. Narrow layout (below 940 px)

- The window shows `render_narrow_tab_bar` at the bottom, with no sidebar. The "Playlists" destination opens through the tab bar rows or the More menu. Evidence: `src/app.rs:2227-2231`, `src/app/tab_bar.rs`. UNSURE: which destinations are tabs and which are in the More menu (not read).
- The inline new-playlist row, the playlist page, the popover and the selection bar use the same code. The list bar wraps in a narrow window (comment at `src/ui/composites/selection_action_bar.rs:395`). The download progress item text wraps (`src/ui/composites/download_status_item.rs:9`).
- The playlist status text (`LibraryViewModel.status`) may not show (section 18).
- Spring-load on the "Playlists" destination (section 4.4) needs the sidebar. INFERENCE: it is not possible below 940 px.
- Playlist page repair controls must not overlap the identity at narrow widths (comment `src/ui/shells/playlist.rs:1289`). UNSURE: the effect.

## 20. UNSURE points that matter

1. Where the Library status text shows in the narrow layout (section 18, 19).
2. Index-add errors go to `settings_status`, which shows only in Settings or the Show page (section 18). I did not check if the page shows it on a Music page. By the code at `src/app/settings.rs:224`, it does not.
3. Whether the album name link of every Index track page opens the Index release page (Job 5 step 5).
4. Which release row states allow "Add to Playlist" in a list row menu (`ReleaseRowAvailability`).
5. Order of a multi-select add and the sort condition for the drag reorder.
6. Whether the removal of a release from the release row menu always confirms (the track and album menu paths do).
7. Queue order, concurrency and mid-flight cancel (`download_queue.rs` not read to the end).
8. The text of the "Remove from all playlists" dialog (`playlist_removal_confirmation.rs` not read).
9. Whether a path other than "Delete All Cached" and per-track "Delete" frees disk space after a Library removal, and whether the cache is emptied by the app.
10. Native dialog text and mpv behavior: none of the journeys here use a native dialog or mpv, so no text is missing for them.
