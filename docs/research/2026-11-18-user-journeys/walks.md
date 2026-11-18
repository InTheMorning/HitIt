# Walks

Each walk operated the release binary of HitIt commit `c95aff3` on Xvfb with openbox. Two walker agents used
two displays. The window was 1440 × 740 px, or 1120 × 780 px after a start of the app, or 390 px wide for
the narrow walk. The data folders were scratch folders with an empty Library at the first start.

The network was the live Music Index API (`api.musicindex.org`). Audio downloads were possible only from the
hosts `cdn.kolomona.com` and `sirlibre.com`, because the sandbox allowed only those hosts. Thus the walks
used releases of Sir Libre Records and of Crash Landing.

No walk played audio. The sandbox has no audio device. The journeys that need audio, a relay or a stream
target are Code only. `step-counts.md` gives their counts from the inventories.

A count in a walk is the number of steps that the walk did. The step method is in `README.md`.
The main agent examined the screenshots that the results cite.

## W01 First Run To The Audio Player Setting

Data: empty. Screenshots: `shots/w01-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 0 | Start the app. | No welcome page, no setup page and no dialog shows. "Releases" shows "Loading recent MusicIndex releases". The sidebar has "Search", "Releases", "Artists", "My playlists", "Public playlists", "Live", "Show", "Inbox" and "Settings". The sidebar foot shows "0 library tracks" and "Check all feeds". | Shot `w01-01-first-start-loading` |
| 0 | Wait about 4 s. | "Releases" shows a list of recent releases with the filters "All" and "In Library". | Shot `w01-02-releases-list` |
| 1 | Click "Settings". | Settings shows the tabs "General", "Library" and "Diagnostics". General has "UI scale", "Theme", "Use Defaults..." and "Save". No control is for playback. | |
| 2 | Click "Library". | The page has "Music Index endpoint", "Music directory", "Audio converter" and "Audio cache". No control is for playback. | Shot `w01-03-settings-library-no-playback` |
| 3 | Click "Diagnostics". | The page shows the database tools and the version line. | |
| 4 | Select "Configuration" in the menu adjacent to "Diagnostics". | The page "Configuration repair" shows "Edit configuration". | |
| 5 | Click "Edit configuration". | A list of field buttons shows. It includes "playback.driver" and "playback.mpv_path". | Shot `w01-04-config-repair-fields` |
| 6 | Click "playback.driver". | The text "playback.driver: enter the value as text, without quotes. Other edited fields stay in this draft." and an empty text box show. "Test draft and paths" and "Save correction" are not available until the text changes. | Shot `w01-05-playback-driver-text-box` |

Results:

- The first start goes directly to "Releases". The walk saw no welcome page. The shots had an interval of
  0.7 s, so a page that shows for less than 1 s is possible. This is **Unsure**.
- No Settings page has a playback group. The audio player is a text value in "Configuration repair".
- The text box does not show the current value or the permitted values.
- The config file that the app wrote has the comment `# Playback backend. The default "null" driver disables playback.`
  The app does not show this text.
- The walk did not save a value and did not play audio.

## W02 Take A Release

Data: continued from W01. Screenshots: `shots/w02-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| — | Scroll "Releases" through about 45 rows. | No release of Sir Libre Records shows. Most rows are from Wavlake. | |
| 1 | Click the search field. | | |
| 2 | Type "Motive to Kill". | | |
| 3 | Push Return. | "Search Results" shows "Releases 1" and "Tracks 10". The row "Motive to Kill" has a cloud icon. | Shot `w02-01-search-motive-to-kill` |
| — | Hover the cloud icon. | The hover help is "Not downloaded". | Shot `w02-02-row-icon-not-downloaded` |
| 4 | Click the row. | The release page opens. | |
| 5 | Click "Download Album". | A band shows "Downloading 1 of 1", "Motive to Kill", "0 of 1 done". The button still shows "Download Album". | Shot `w02-03-downloading-banner` |
| 0 | Wait about 30 s. | The band shows "Downloaded 1 of 1. Finished 12:46" and "Dismiss". The button changes to "Add to Playlist". The sidebar foot shows "10 library tracks". | Shot `w02-04-downloaded` |

Count: 5 steps from Music, with a search. The walk could not take a release from the list of recent releases,
because no release of an allowed host was in that list.

Results:

- The cloud icon of a row is a state, not an action. A click on it opens the release page.
- The Releases rows show a "..." button on hover. The walk did not open it.
- After the download, the release page shows "Label: 34bf795f-f551-4db1-8877-ad19aa5f89c8" in place of
  "Label: Sir Libre Records". The label page shows the title "Motive to Kill". Shot `w02-05-label-page-title-differs`.
  The third agent walk of 2026-11-18 found the same defect.
- The search "Sir Libre" did not find the label. The search "Sir Libre Records" found the label only.
- The download band moves the page content down by 72 px.

## W03 Take 3 Chosen Tracks Found By Search

Data: continued. Screenshots: `shots/w03-*.png`.

No artist of an allowed host had 3 singles. The walk took 3 tracks of the album "The World Deserves to Know"
by Crash Landing (7 tracks).

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 to 3 | Click the search field, type "Crash Landing", push Return. | "Releases 4", "Tracks 9", "Artists 1". | Shot `w03-01-search-crash-landing` |
| 4 | Click the row "The World Deserves to Know". | The release page opens. | |
| 5 | Scroll down. | The "Tracks" list shows "7 total". "Select" is above it. | |
| 6 | Click "Select". | The bar shows "Done", "Select items" and "Select all". Each track has an empty check box. | Shot `w03-02-select-mode` |
| 7 to 9 | Click the check box of tracks 1, 2 and 3. | The bar shows "3 tracks selected", "Select all", "Download 3 tracks" and "Clear". | Shot `w03-03-three-selected` |
| 10 | Click "Download 3 tracks". | After about 20 s the band shows "Downloaded 3 of 3". The bar shows "Remove 3 tracks...", "Select all", "Add 3 tracks to playlist" and "Clear". | Shot `w03-04-three-downloaded` |

Count: 10 steps from Music. Without the scroll, 9 steps.

Results:

- After the download, "Remove 3 tracks..." shows in the position of "Download 3 tracks".
- The selection mode stays on after the download. "Done" ends it.
- The search results have a "Tracks" list. The walk did not take a track from it.

## W04 Take The Missing Tracks Of A Release

Data: continued. 3 of 7 tracks of "The World Deserves to Know" are in the Library. Screenshots: `shots/w04-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 to 3 | Search "Crash Landing". | As W03. | |
| 4 | Click the row "The World Deserves to Know". | The main button is "Add to Playlist". No button downloads the 4 missing tracks. | Shot `w04-01-partial-release-add-to-playlist` |
| — | Click "...". | The menu has "Copy Feed URL", "MusicBrainz Lookup" and "Remove Album...". No item downloads. | Shot `w04-02-more-actions-menu` |
| 5 | Scroll down. | Tracks 4 to 7 show the word "New". | |
| 6 | Click "Select". | | |
| 7 | Click "Select all". | "7 tracks selected", with "Remove 3 tracks...", "Download 4 tracks", "Add 7 tracks to playlist" and "Clear". | Shot `w04-03-select-all-download-4` |
| 8 | Click "Download 4 tracks". | After about 25 s the band shows "Downloaded 4 of 4". All 7 rows show the downloaded state. | Shot `w04-04-all-seven-in-library` |

Count: 8 steps from Music. The walk did not examine the artist page. It is **Unsure** if the artist page has an
action that takes all missing tracks.

Results:

- When one track of a release is in the Library, "Download Album" is not on the release page.
- One selection gives three different numbers: "Remove 3 tracks...", "Download 4 tracks" and
  "Add 7 tracks to playlist".

## W05 Recover From A Failed Search

Data: continued. The app started with `HTTPS_PROXY`, `HTTP_PROXY` and `ALL_PROXY` set to `http://127.0.0.1:9`.
Screenshots: `shots/w05-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 0 | Start the app. | "Releases" shows "MusicIndex did not answer the release list request. The list shows Library releases only.", the technical text and "Try again". The Library releases show below. | Shot `w05-01-releases-start-offline` |
| 1 to 3 | Click the search field, type "Motive", push Return. | "Search Results" shows the counts, then "Music Index search unavailable", "App could not get search results from Music Index.", "Your local library remains available." and "Check your connection and the Music Index endpoint in Settings, then run the search again." The buttons are "Show details" and "Copy report". The status bar shows "1 report". | Shot `w05-02-search-unavailable` |
| 4 | Click "Show details". | The time, "Configured Music Index endpoint: https://api.musicindex.org/" and "Technical details" show. The button changes to "Hide details". | Shot `w05-03-show-details` |
| 5 | Click "1 report" in the status bar. | A band at the top shows "1 retained action" and "Index search: Motive. App retained the original action. Its setup needs a successful check before it can run." The buttons are "View tools in Settings", "Edit endpoint" and "Dismiss". | Shot `w05-04-retained-action-report` |
| 6 | Click "Edit endpoint". | Settings opens on "Diagnostics", "Configuration repair", with the field buttons. The text says "musicindex_endpoint: enter the value as text, without quotes." | Shot `w05-05-edit-endpoint-opens-config-repair` |

Count: 3 steps to the failure. 1 more step to the details, and 2 more steps to the endpoint editor.

Results:

- The failure shows in three places: the text on "Releases" at the start, the text below the search counts and
  "1 report" in the status bar.
- The search page has no button that runs the search again. "Try again" is only on "Releases".
- "Edit endpoint" opens the endpoint text in "Configuration repair". It does not open the field
  "Music Index endpoint" of Settings, Library. The band text says "Edit endpoint opens musicindex_endpoint in Settings."
- The fault of this walk was the network, not the endpoint. The two fix paths of the page go to the endpoint.

## W06 Make A Playlist And Add A Release

Data: "Motive to Kill" is downloaded. Screenshots: `shots/w06-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Click "In Library" on "Releases". | The list shows the downloaded releases. | |
| 2 | Hover the row, then click "...". | The menu has "Add to Playlist" and "Remove Album...". | Shot `w06-01-row-menu` |
| 3 | Click "Add to Playlist". | The menu shows the playlists and "+ New Playlist". | |
| 4 | Click "+ New Playlist". | A popover with the field "Playlist name" opens. The field does not have the focus. | Shot `w06-02-new-playlist-form` |
| 5 | Click the field. | | |
| 6 | Type "Mix". | | |
| 7 | Click "Create & Add". | The popover closes. No message shows. The count of "My playlists" in the sidebar changes. The playlist "Mix" has 10 tracks. | Shot `w06-03-result` |

Count: 7 steps from "Releases".

Results:

- Before the download, the row menu has only "Download Album", and the "..." menu of the release page has only
  "Copy Feed URL". Thus "Add to Playlist" is not available for a release that is not in the Library. Shot `w06-04-undownloaded-menu`.
- Return in the field "Playlist name" closed the popover and made no playlist. This comes from the report of the walker only.
- The other path starts in "My playlists": "New Playlist", click the field "New playlist name...", type, "Add".
  It takes 5 steps to an empty playlist. Return did not make the playlist. The empty playlist shows
  "Empty — add tracks from the library or search".
- The filter "In Library" stayed on when the walker went back to "Releases".

## W07 Add 3 Tracks Of A Release To A Playlist

Data: continued. Screenshots: `shots/w07-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Click the release row. | The release page opens. | |
| 2 | Scroll, then click "Select". | Check boxes show. | |
| 3 to 5 | Click the check boxes of tracks 2, 3 and 4. | "3 tracks selected", "Remove 3 tracks...", "Select all", "Add 3 tracks to playlist" and "Clear". | Shot `w07-01-three-selected` |
| 6 | Click "Add 3 tracks to playlist". | A menu shows the playlists. | Shot `w07-02-playlist-menu` |
| 7 | Click "Mix". | No message shows. The selection stays. "Mix" changes from 10 to 13 tracks. | Shot `w07-04-mix-13-tracks` |

Count: 7 steps from "Releases".

Results:

- The 3 tracks were already in "Mix". The app added them again with no warning.
- The "..." menu of each track has "Add to Playlist" and "Download". That path takes about 3 steps for each track.

## W08 Add A Music Index Track Found By Search To A Playlist

Data: continued. The track "Moonlight Waltz" is not in the Library. Screenshots: `shots/w08-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 to 3 | Click the search field, type "Moonlight Waltz", push Return. | "All 1", "Releases 0", "Tracks 1". | Shot `w08-01-search-results` |
| 4 | Click the track row. | The track page shows "Download Release" and "...". | Shot `w08-02-track-page` |
| 5 | Click the release link "Waltz Tunes Volume 2". | The release page opens. | |
| 6 | Scroll to track 15, then click its "...". | The menu has "Add to Playlist". | |
| 7 | Click "Add to Playlist". | A menu shows the playlists. | Shot `w08-03-track-playlist-menu` |
| 8 | Click "Mix". | "Mix" has 14 tracks. The sidebar foot shows "11 library tracks". | Shot `w08-04-mix-14-tracks` |

Count: 8 steps from Music.

Results:

- The track row of the search results has no "..." and no action. A right click has no effect.
- The "..." menu of the track page has only "Copy Feed URL". The track page has no "Add to Playlist".
- The add downloaded the track with no question.
- The result "Added 1 track, downloaded 1" showed only on Settings, General. This comes from the report of the walker only.
- After the add, "In Library" shows "Waltz Tunes Volume 2" as "Single" with 3 min. Before the add it was an album
  with 21 tracks. Shot `w09-03-after-remove`.

## W09 Remove A Release And Free Its Disk Space

Data: continued. Screenshots: `shots/w09-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Hover the row in "In Library", then click "...". | The menu opens. | Shot `w09-01-row-menu` |
| 2 | Click "Remove Album...". | The dialog "Remove Album from Library?" says "10 tracks from this album are in playlists. Removing them from the library will make them unavailable for playlist playback." The buttons are "Remove" and "Cancel". | Shot `w09-02-confirm` |
| 3 | Click "Remove". | The sidebar foot shows "1 library track". | Shot `w09-03-after-remove` |
| 4 | Click "Settings". | | |
| 5 | Click "Library", then scroll. | "Cached files (10)" and "Files that a Library removal moved out of the music folder. Delete removes a file for good." | Shot `w09-04-cached-files` |
| 6 | Click "Delete All Cached". | No confirmation shows. "Cached files (0)" and "No cached files." | Shot `w09-05-cache-deleted` |

Count: 6 steps from "Releases" to the state with no audio files of the release.

Results:

- The walker listed the scratch folders before and after each step. After step 3, the 10 files of the release were
  not in the music folder. The same 10 files (40,452,192 bytes) were in `cache/hitit/audio/`. After step 6 that
  folder was empty.
- Remove does not free the disk space. The dialog text does not tell about the files or the cache.
- "Delete All Cached" deletes the files with no confirmation.

## W10 The Narrow Layout

Data: continued. The window was 390 × 740 px. Screenshots: `shots/w10-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 to 3 | Click the search field, type "Motive to Kill", push Return. | The result buttons "Releases 1" and "Tracks 10" are cut. | Shot `w10-01-releases-narrow` |
| 4 | Click the release row. | The release page shows the title on two lines. | Shot `w10-02-release-page` |
| 5 | Click "Download Album". | The band shows "Downloaded 1 of 1". | |
| 6 | Click "Add to Playlist". | The menu shows "Walk", "Mix" and "+ New Playlist". | Shot `w10-03-playlist-menu` |
| 7 | Click "Walk". | The menu closes. No message shows. | Shot `w10-04-after-add` |

Count: 7 steps from "Releases", the same as the wide layout for the same path.

Results:

- The tab bar at the bottom has "Search", "Releases", "Show", "Inbox" and "More". "More" opens "Artists",
  "My playlists", "Public playlists", "Live" and "Settings". Thus "My playlists" takes 2 steps.
  Shot `w10-05-my-playlists`.
- The head of "Releases" has three lines: the title, then "Newest", the view buttons and "Select", then
  "All" and "In Library".
- Release names are cut, for example "Waltz Tunes Volu...".
- The label shows the GUID in the narrow layout too.
