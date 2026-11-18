# Observations

This page gives facts for a subsequent analysis. It gives no design proposal and no priority.
Each item has an evidence mark from `README.md`. The walk IDs refer to `walks.md`. The inventory references
refer to the files in `inventory/`, with the section number.

## Defects

**O1. A Sir Libre release shows a GUID as its label after a download.** Shot W02, W06, W10.

Before the download, the release page of "Motive to Kill" shows "Label: Sir Libre Records". After the download,
it shows "Label: 34bf795f-f551-4db1-8877-ad19aa5f89c8". The label page shows the title "Motive to Kill".
The third agent walk of 2026-11-18 found the same defect.

**O2. The search page has no control that runs a failed search again.** Shot W05.

The text says "Check your connection and the Music Index endpoint in Settings, then run the search again."
"Try again" is only on "Releases".

**O3. "Edit endpoint" does not open the endpoint field of Settings.** Shot W05.

The band says "Edit endpoint opens musicindex_endpoint in Settings." The control opens the text of
"Configuration repair". The field "Music Index endpoint" of Settings, Library is a different control.

**O4. A playlist can get the same tracks two times with no warning.** Shot W07.

The walk added 3 tracks that were already in "Mix". "Mix" changed from 10 to 13 tracks.

**O5. Return in a new playlist name field does not make the playlist.** Code, C §16. Walker report W06.

The inline field has no Enter handler (`src/library/app_impl.rs:1124`). In the popover, the walker saw that
Return closed the popover and made no playlist. No shot shows this.

## Setup And The Audio Player

**O6. The first start has no welcome page and no setup page.** Shot W01.

The first page is "Releases". The walk saw no page about the music folder, the player or Music Index.

**O7. The audio player is a typed configuration value.** Shot W01. Code, A §7 and §11.

No Settings page has a playback group. The path is "Diagnostics", "Configuration", "Edit configuration",
"playback.driver", a typed value and "Save correction". The text box does not show the current value or the
permitted values. The default driver is "null", and only a comment in the config file says that.

**O8. The setup checks do not prove that a service works.** Code, A §13.

"Check endpoint" does not test a search response. "Check player" does not play audio. "Check encoder" does not
connect the stream (`src/view_models/startup/capabilities.rs:688`). No screen shows that the setup is complete.

## Items And Their Actions

**O9. The same item has different actions in different places.** Shot W02, W06, W08. Code, B §20, C §16.

- The cloud icon of a release row is a state with hover help "Not downloaded". A click opens the release page.
- The track rows of the search results have no menu and no action. A right click has no effect (W08).
- An Index track page has no "Add to Playlist". Its "..." menu has only "Copy Feed URL" (W08).
- A release that is not in the Library has no "Add to Playlist" (W06).
- When one track of a release is in the Library, the release page has no action for the missing tracks (W04).
  A singles group page has "Download {missing_count} Single{s}" (B §15).

**O10. One action has different words.** Code, A §15.2, B §21, C §17, D §20.

- Download: "Download Album", "Download EP", "Download Single", "Download Release", "Download Track",
  "Download {count} releases" and "Download {n} {noun}".
- Remove from the Library: "Remove…", "Remove Track…", "Remove from Library…", "Remove {Kind}…" and
  "Remove {count} tracks…". The confirmation says "Remove Album from Library?" for each kind.
- Make a playlist: "New Playlist", "Add", "Create & Add". The name field has the placeholders
  "New playlist name…", "Playlist name" and "Name of the new playlist".
- Library membership: "In library", "In Library" and "Downloaded".
- Show start: "Start", "Pause", "Resume", "Retry", with the accessible names "Start Show" and the key name
  "Show Play/Pause".

**O11. One selection gives three different counts.** Shot W04.

With "Select all" on a release with 3 of 7 tracks in the Library, the bar shows "Remove 3 tracks...",
"Download 4 tracks" and "Add 7 tracks to playlist".

**O12. An add to a playlist downloads the track with no question.** Shot W08. Code, C §4.

## Removal And Disk Space

**O13. Remove does not free the disk space.** Shot W09. Code, C §9.

"Remove" moves the audio files to the audio cache (`src/application/library_removal.rs:239`). The dialog text
does not tell about the files. "Delete All Cached" in Settings, Library deletes the files with no confirmation.

## Results That Show Away From Their Control

**O14. Many results go to a status text that the user does not see.** Code, A §15.3, B §22, C §18, D §21.

- The Library status ("Added {appended} of {total} to {name}", "Error adding to playlist: {e}" and the
  errors of create, rename, delete and reorder) shows in the sidebar foot.
- `settings_status` shows on Settings, General and Library and on the Show page. The add of an Index track,
  the stream errors, the transport errors and some search navigation failures go there.
- **Inference:** below 940 px the window has no sidebar, so the Library status has no place on the screen
  (C §18). No walk examined it.
- Walker report W08: the result "Added 1 track, downloaded 1" showed only on Settings, General. No shot shows this.

**O15. Some actions give no message.** Shot W06, W07, W10.

"Create & Add" and the add from a playlist menu close the menu with no message. The count of the playlist changes.

**O16. A failed search shows in three places.** Shot W05.

The text on "Releases" at the start, the text below the search counts and "1 report" in the status bar.

## Show, Playback And Live

These items are Code only. No walk played audio or used a relay or a stream target.

**O17. The show has no single control to go live or to end.** Code, D §2, §7 and §15.

Going live uses separate cards: the event ("Create", "Attach"), the Producer ("Start"), the Relay sender
("Start"), the Stream ("Connect") and the play control. "Stop Show" stops the music only, with no
confirmation. It does not stop the stream or the services.

**O18. The cue cannot change while a show plays.** Code, D §4 and §14.

The cue has no reorder control. A row click selects only when no session is active.

**O19. Built commands with no control.** Code, D §19.

`PlayTrack` and `SeekPlayback` have no caller outside `src/application/commands/playback.rs`. No control starts
or stops a recording.

**O20. "Ready" is the state of one card.** Code, D §12.

The word is the state of the "Live Metadata" card. No single check combines the Show cards and the Inbox.

## Layout

**O21. The narrow layout adds a step for five destinations.** Shot W10. Code, B §2.

The tab bar has "Search", "Releases", "Show", "Inbox" and "More". "Artists", "My playlists",
"Public playlists", "Live" and "Settings" are in "More".

**O22. The narrow layout cuts text.** Shot W10.

The search result buttons "Releases 1" and "Tracks 10" are cut. Release names are cut, for example
"Waltz Tunes Volu...".
