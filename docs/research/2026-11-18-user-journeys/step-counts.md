# Step Counts

This page gives the step count of each baseline journey at HitIt commit `79c23c1`. The baseline is the
v4vmm study at commit `d97da51` (2026-10-22): jobs 1 to 16 come from `02-as-is.md`, and MH1 to MH7 come from
`03a-must-have-journeys.md`. The step method is in `README.md`. It is the same method as the baseline.

Each current count has a source:

- **Shot**: a walk in `walks.md` did the steps.
- **Code**: the count comes from an inventory, with the section number. It is an inference from the code.

When a walk and an inventory give different counts, the table gives both and the reason.

## Jobs 1 To 16

| Job | Journey | Start | Baseline | Now | Source | Difference |
|---|---|---|---|---|---|---|
| 1 | First launch to real audio | Fresh install | Not possible without raw config edits | 14 + M, where M is the steps to get a playlist with a downloaded track | Code, A §11. Shot, W01 steps 1 to 6 | Possible now, with a typed config value. The player is the field "playback.driver" in "Configuration repair", not a form. The default driver is "null". No walk played audio. |
| 2 | Take a new release from the list of new releases | Music open | 2 | 3 | Code, B §13. Shot, W02 (5, with a search) | 1 more. "Discover" is now "Releases". Row menu "Download {Kind}". The walk used a search, because no release of an allowed host was in the list. |
| 3 | Search an artist, take 3 chosen singles | Music open | 7 | 9 | Code, B §14. Shot, W03 (10, for 3 tracks of one album) | 2 more: "Select" and one more selection step. The walk took tracks of an album, because no artist of an allowed host had 3 singles. |
| 4 | Take all missing singles of a known artist | Music open | 3, only if the artist is in the Library | n + 4 from Music, or 1 from an open singles group page | Code, B §15. Shot, W04 (8, for the missing tracks of an album) | The singles group page has "Download {missing_count} Single{s}". A release page has no action for its missing tracks. The walk used "Select all" and "Download 4 tracks". |
| 5 | Add a Music Index track found by search to a playlist | Music open | 9 | 8 | Code, C §11. Shot, W08 | 1 fewer. The track page still has no "Add to Playlist". The path goes through the release page. The add also downloads the track. |
| 6 | Make a playlist and add an album | Music open | 6 | 6, or 7 when the album is not downloaded | Code, C §12. Shot, W06 (7, with the filter "In Library") | Same. Return in the name field does not make the playlist. A release that is not downloaded has no "Add to Playlist". |
| 7 | Add 3 tracks of a release to a playlist | Music open | 7 | 8 | Code, C §13. Shot, W07 (7, from "Releases") | 1 more from Music: "Select" is a step. Each track menu: 9. |
| 8 | Check feeds, apply, write tags | Music open | 4 | 3 wide, 4 narrow | Code, B §16 | 1 fewer. "Check all feeds" and the tag control of the status bar. Pending updates: "Apply updates ({count})", 3 steps. |
| 9 | Fix a "Needs attention" item | Music open | 2, 4 or no fix | 2 to 4 for each type. "No upstream routes": no fix in the app. "Missing file" and "Not downloaded": 2 to the track page, no direct fix | Code, B §17 | "Needs attention" is now "Inbox". "Route out of date" has a fix in 3 steps. |
| 10 | Remove a release and free its disk space | Music open | About 8 | 7 | Code, C §14. Shot, W09 (6, from "Releases") | 1 fewer. Remove moves the files to the audio cache. "Delete All Cached" in Settings, Library deletes them with no confirmation. |
| 11 | Play a playlist | App open | 3, plus 1 to see Show | 5, or 4 with "Ctrl+Alt+P" | Code, D §10 | 1 or 2 more. The playlist goes into the Show cue first ("Load into Show Cue"). No walk played audio. |
| 12 | Go live with no event and music that plays | App open | About 10, plus an RSS edit outside the app | 11 | Code, D §11 | No "Go live" control. The steps are separate cards: event "Create" and "Attach", Producer "Start", Relay sender "Start", Stream "Connect", then play. No walk. |
| 13 | Go live with an event already attached | App open | 6 | 5 to 7, plus 4 to 5 with no cue | Code, D §13 | About the same. No walk. |
| 14 | Cue a different next track | Show open | 3 to 5 | Not possible while a show plays. About 4 after "Stop Show" | Code, D §14 | The cue has no reorder control. A row click selects only with no active session. |
| 15 | End the show | Show open | 6, and it never fully ends | 1 to stop the music. 5 to 6 to stop the music and the live parts | Code, D §15 | "Stop Show" exists, with no confirmation. It does not stop the stream or the services. No "End show" control. |
| 16 | Set up a stream encoder | Anywhere | About 6, plus raw TOML | 8, or 3 from a shown "Edit encoder settings" | Code, A §12 | The encoder is a typed value in "Configuration repair". |

## MH1 To MH7

| ID | Journey | Done when | Baseline | Now | Source | Difference |
|---|---|---|---|---|---|---|
| MH1 | Curate music: find, take, remove | A release is in the Library or out of it | 2 to 9 | Take: 1 from a track row or a singles row icon, 2 from a release row menu or an item page. Remove: 3 from a row menu, 4 from a search result | Code, B §5. Shot, W02, W09 | The cloud icon of a release row is a state, not an action (W02). The track rows of the search results have no action (W08). |
| MH2 | Build a show playlist | The playlist exists, is in order, and each track can be got | 6 to 9 for each add | Make: 4, or 3 with the key of "New Playlist". Add: 2 for a Library release, 3 for a track. Order: drag, "Move Up" and "Move Down", or "Move to top" and "Move to bottom" | Code, C §15. Shot, W06, W07 | Fewer steps for each add. An add can add a track two times with no warning (W07). |
| MH3 | Get show-ready | "Ready" with zero blockers | 3 places, 4 of 6 problems have no fix | Show: 1 step when all is ready, 3 to 5 with a blocker. Library: 2 wide, 3 narrow with no problems. 4 wide, 5 narrow with one tag write | Code, D §12 and B §18 | No single check gives "Ready". "Ready" is the state word of the "Live Metadata" card only. The Show cards and the Inbox are separate. |
| MH4 | Play the show with the built-in player | Music plays in order, the user can control it, and it ends cleanly | 3, no stop, ends in an error | 1 from "Show open, cue loaded". "Stop Show" exists | Code, D §16 | The session stops at the last entry with no error. No control plays one track or seeks. No walk played audio. |
| MH5 | Go live and come off air | Listeners hear it, then it is off | 6 to 10 on, 6 off, never ends | 5 on with an event attached and a cue loaded. 5 to 6 off | Code, D §17 | No single "Go live" or "End show" control. No confirmation before the stream goes on air. No walk. |
| MH6 | Get set up | Audio works, services are green | Raw config, silent player | No fixed total. Each item is a typed value in "Configuration repair": a field selection and a typed value, then "Save correction" | Code, A §13. Shot, W01 | No welcome page and no setup forms. The "Check" controls examine a tool, but do not play audio or connect a stream. |
| MH7 | Recover when something breaks | The problem is fixed without a search through Settings | Repair, notice, raw config | 3 from a notice with an edit control. 1 for a fresh check. Search failure: 1 to the details, then steps outside the app, then 1 to search again | Code, A §14. Shot, W05 | The search failure has "Show details" and "Copy report". "Edit endpoint" opens the endpoint text in "Configuration repair" (W05). |

## Journeys That The Baseline Did Not Count

| Journey | Start | Now | Source |
|---|---|---|---|
| First start to a list of releases | Empty data | 0 | Shot, W01 |
| From the list of releases to the player field | Releases | 6 | Shot, W01 |
| Make an empty playlist in "My playlists" | My playlists | 4 | Shot, W06. Code, C §3 |
| Take a release in the narrow layout and add it to a playlist | Releases at 390 px | 7 | Shot, W10 |
| Open "My playlists" in the narrow layout | Any page at 390 px | 2 ("More", then "My playlists") | Shot, W10. Code, B §2 |
| Remove a release, then delete its cached files | Releases | 6 | Shot, W09 |
