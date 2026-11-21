# Agent Walk Of The Open Checks - 2026-11-19

Status: record of an agent walk. This walk closes no gate. Each visual gate stays open until a person walks it.
[Pending human checks](../pending-human-checks.md) owns each check, and this record gives the agent results.

## Method

- Binary: the release build of HEAD `89f0883`.
- Two walker agents used two Xvfb displays (:91 and :92) with openbox. Each walker had its own scratch `HOME`,
  `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_CACHE_HOME`. No walker touched the Library of the operator.
- `allowed_domains` had `api.musicindex.org`, `cdn.kolomona.com` and `sirlibre.com`.
- Walker A walked [ADR 0098 task 001](../tasks/adr-0098-task-001-each-named-feed-keeps-its-own-title.md),
  [ADR 0098 task 002](../tasks/adr-0098-task-002-a-role-row-dot-has-the-color-of-its-first-role.md),
  [ADR 0101 task 015](../tasks/adr-0101-task-015-load-more-shows-the-focus-ring.md), section 16 V3 and the tile ring.
- Walker B walked sections 37, 38 and 39, section 25 V1 for a list row at 390 px, and the count of a singles group.
- The main agent examined the shots of each defect and of these Pass results: task 001 V1 and V2, task 002 V1 in
  Light, and task 015 V1 in Light. This folder keeps only the shots of the defect and the open points, in
  [screenshots/agent-walk-2026-11-19](screenshots/agent-walk-2026-11-19/).

## Defects Found

1. **Tab moves the focus to a hidden button of the restore band at 390 px.** Section 37 V2, in Dark and Light. Tab
   from "Review restore" goes to "Upgrade backup" and then to "Copy database report". The band does not scroll, and
   no ring shows. The wheel scrolls the band (V1 passes).

   The main agent confirmed that the shot shows no ring after
   the Tab. Screenshots: `restore-band-tab-hidden-dark.png`, `restore-band-tab-hidden-light.png`.

   Preparation: the buttons are disabled in a new state. The walker typed two scratch paths in the Restore fields and
   chose "Review restore". The app refused the review, and then "Upgrade backup" and "Copy database report" were
   enabled. The walker did not choose "Restore database".

## Open Points

1. **A singles group gets a "Matched by name" row after a download.** Before the download, the group page of
   "Jon Ross · 8 singles" lists 8 singles and has no "Matched by name" section. After the download of
   "Never Be Another You", the title is "Jon Ross · 9 singles". A "Matched by name" section shows "Rock The Halls".

   The button still says "Download 8 Singles". The main agent confirmed the shots. Screenshots:
   `singles-group-before-download.png`, `singles-group-after-download.png`.

   An inference is that the Library track lets the name match find a by-name single. The count then includes that
   row, and the button counts only the artist feed singles. The operator decides if the count or the button is
   incorrect.
2. **Task 015 V3: the walk cannot prove which tile is new.** Enter on "Load more" loads more tiles, and the ring goes
   to a tile in the second row of the new part. In the sort Newest, the new tiles can sort between the old tiles. The
   code selects the first tile in screen order that the list did not show before the press, and
   `adr_0101_load_more_focuses_the_first_new_row` tests it. Screenshot: `load-more-enter-ring-dark.png`.
3. **A release page title breaks inside a word at 390 px.** "Red Pill Revolution" shows as "Revoluti" and "on". This
   is a detail page, not a list page, so section 39 does not include it. Only walker B saw it, with no shot in this
   folder.

## Results By Item

| Item | Pass | Fail or open |
| --- | --- | --- |
| ADR 0098 task 001 | V1 and V2 after the download of "Motive to Kill": "Label: Sir Libre Records", and the page title "Sir Libre Records" | - |
| Section 16 V3 | The Sir Libre Records page has "Role not confirmed" rows with `rel="recordLabel"` | - |
| ADR 0098 task 002 | V1: the "Ph@fe" row dot and page dot are `#875CCF` in Dark and `#8D62D5` in Light. V2: the Feed dot and the Artist dot keep their colors | - |
| ADR 0101 task 015 | V1 in Light and Dark: the 2 px ring, the fill of Select, and no load in 6 s. V2 in Light and Dark: after a click on Tiles, Tab goes to Select | V3: open point 2 |
| Tile ring | Tab into the Tiles grid shows the ring on the first tile at once, in Light and Dark | - |
| Section 25 V1 | A list row at 390 px has the full 2 px ring, in Light and Dark | - |
| Section 37 | Step 3, V1 and V3 in Light and Dark | V2: defect 1 |
| Section 38 | V1 to V4 in Light and Dark. The status bar has no Tab stop at 390 px, so Tab goes from the list to the tab bar | - |
| Section 39 | V1 to V4 in Light and Dark | - |
| Singles count | The download worked with the allowed hosts | Open point 1 |

Only a walker saw the Pass results that the main agent did not examine. The Method section names the results that the
main agent examined.

## Section 43 Walk - 2026-11-19, Afternoon

The main agent did this walk itself, with the release build of commit `3532954` on Xvfb display `:85` at 1440 px. The configuration had `musicindex_endpoint = "http://127.0.0.1:9"`. The shots are in the scratch folder of the session and are not kept.

| Item | Pass | Fail or open |
| --- | --- | --- |
| Section 43 V1 | In Dark and Light: "Copy report", then the closed fold "Details" with a chevron that points right. The report does not show | - |
| Section 43 V2 | In Dark and Light: a click opens the report below the fold, and the chevron points down. A second click closes it. In Dark, Enter on the focused fold opens it, with the focus ring | - |
| Tab order | Shift+Tab from "Details" goes to "Copy report", and Tab goes back | - |
| Task 016 item 3 | The log has the line "App could not complete a Music Index search: ..." | - |

Finding 4: **the Releases page shows raw request text in its message.** With the same endpoint, Releases shows "Recent music unavailable. GET /v1/feeds/recent: error sending request for url (...): client error (Connect): tcp connect error: Connection refused (os error 111)". musicindex ADR 0085 R-MSG-1 item 4 does not permit raw text from a server or a library in a message. Task 016 changed only the search notice. A subsequent task can put this text in a "Details" fold, as task 016 did.

## Sections 44 To 46 Walk - 2026-11-19, Evening

The main agent did this walk itself, with the debug build of commit `e362af2` on Xvfb display `:85`, at 1440 px and at 390 px. The Library had 11 audio files and the playlists Walk (10 tracks) and Mix (14 tracks). The sandbox has no sound device. For the cue steps, the scratch file `mpv/mpv.conf` had `ao=null`, so that mpv plays each file at its usual speed with no sound. The shots are in the scratch folder of the session and are not kept.

The walk found these defects. Commits `04c24ee` and `e362af2` correct them.

1. The first start opened an empty Settings page.
2. Settings said "mpv is not installed" for each mpv error. Each mpv check gave an error, because HitIt read the version from an incorrect field.
3. Between two files, the mpv poll gave the error "property unavailable".
4. The on air band was tall, and the cards and "Go on air" were off the screen.
5. Row 1 of the band said "mpv is not ready" after each start that was not the first.
6. The band showed the stream sound and listener lines when the show was not on air.

| Item | Pass | Fail or open |
| --- | --- | --- |
| Section 44 steps 1, 3 and 4 | The group "Audio" in Dark and Light. The first start opens "Set up sound", and "Continue" opens Music | Step 2: the tone needs a person |
| Section 45 V1 | In Dark and Light at 1440 px, and in Dark at 390 px: the band is above the cards, with the four rows in order. Each row that is not ready has one action. "Play test sound" opens Settings, Audio. "Change" opens Settings, Diagnostics, Configuration. The switch "Show information" sets row 4 to "Off". "Go on air" is not available while a row is not ready | Steps 2 to 5 need a stream server and a listener |
| Section 46 V1 | A drag of row 4 to position 3 moves it. The track plays on | - |
| Section 46 V2 | "Play next" puts the row directly after the playing row. When the track ends, that row plays | - |
| Section 46 V3 | A drag of the playing row and of a played row does not move them | - |
| Section 46 V4 and V5 | "Load into Show Cue" on Mix opens Show with 14 tracks. After "Clear", Show lists the playlists, and a load from that list works. "Open playlists" on the band keeps that list | - |

Open points:

- In the cue list, rows 3 and after show "Play next" and "Actions" at all times. At 1440 px the title then shows only five letters, for example "Growi…".
- After a click on Play, the tooltip "Start Show" stays on the button, which is then Pause.
- At 740 px with an empty cue, the cards area gets almost no height.
- In Settings, the status line "The player played the test sound." stays on the other groups.

## Operator Requests - 2026-11-19

The operator gave these requests during the check of section 44. Each item changes an accepted ADR, so a new ADR must come before the code.

1. "Play test sound" has no "Yes" and "No" question after the tone. While the tone plays, the button shows an animation. A click on the button during the tone stops it. This changes ADR 0104 Decision 4 items 4 and 5.

   ADR 0105 Decision 2 item 1 reads the answer: row 1 of the band warns until the host clicks "Yes". The new ADR must give the condition that clears that warning.
2. The Show player items are "mpv", "Mixxx" and "No sound". The label "Speakers of this computer (mpv)" is removed, because the operator finds it incorrect and not necessary. This changes ADR 0104 Decision 2 item 2. At this time, "Mixxx" is not a Show player. ADR 0059 gives Mixxx only as a type of stream source. The new ADR must say what HitIt does when the host selects "Mixxx".
3. Each row of the Show cue list shows the album art of its track. Before the code, find the ADR that owns the cue row, and the musicindex ADR that owns the art of a list row.
4. The Show page does not use its space correctly. The cue list shows which track plays and the total time of the cue.
5. The transport has a seek bar and the album art, as a usual music player. Subsequently, a mood bar can replace the seek bar. The played time and the time left are larger.
6. While a show plays, the transport shows on each page, not only on Show. The operator asks to examine small status icons for Source, Live Metadata and Stream in the transport.

Items 4 to 6 change the layout of ADR 0105 Decision 1 and the Show page of ADR 0063. One ADR for the Show page and the transport is necessary.

7. The text lines above the cards are removed. The cards and their inspectors hold each warning and each status. For example, when the stream gets silence, the Stream card shows a warning icon, and its inspector shows "The stream gets silence" in the warning color.
8. HitIt has a list of shows in a root window, and the host selects one show. Each show has its own source and its own stream. Examples: a Mixxx show through butt, an mpv show of HitIt, and liquidsoap shows on a remote computer. In principle, one Mixxx show, one mpv show and a number of liquidsoap shows can operate at the same time.

   At the time of the check, the operator had a 24-hour stream through Mixxx and butt. HitIt took mpv as the source of that stream, and that is incorrect. "End show" disconnects butt, so it can stop a stream that HitIt did not start.

Items 7 and 8 change ADR 0105 Decisions 1 to 6, and the source model of ADR 0059. Item 8 is a decision that crosses modules, and a subsequent change is hard. It needs an ADR before the code.

The operator check of section 46 step 1 on 2026-11-19 passed. On that screen, the agent found these defects:

- The listener line said "1 listeners". Corrected: one listener gives "1 listener".
- Row 3 said "Stream address is not available." while the stream was on. The row reads the field `address` of the encoder configuration. That field is the control address of butt, not the address of the listeners. HitIt has no field for the address of the listeners. The operator must say where this address comes from, or if the row gives a different text.
- The status bar showed "Relay https://api.musicindex.org reported event ribQc9QzWWD5BvAw6aC17A missing (HTTP 404). App saved event ribQc9QzWWD5BvAw6aC17A as Dead." The message gives a URL and the key of a live event to the host. Compare it with musicindex ADR 0085 R-MSG-1 before a change.

Operator choices of 2026-11-19 for items 7 and 8:

- Section 45 steps 4 and 5 close as replaced. The new ADR gives its own checks.
- The sequence: first a short ADR for item 7, and for an "End show" that does not disconnect an encoder that HitIt did not connect. Then a second ADR for the list of shows.
- A Mixxx show: HitIt shows the status of butt, and it can also connect and disconnect butt for that show.
- The list of shows: the host adds each show with a form. The form gives a name, the source type, the control address of butt or of the remote source, and an optional live event. HitIt keeps the shows in its configuration.
- Operator choice, 2026-11-19, after ADR 0107: in the root window, the host selects a show and configures it. The on air band, the cards and their inspectors show the selected show.

Agent walk of section 47 of the human checks, 2026-11-20. The results are in that section. These points stay open:

- "Open playlists" shows in the Source inspector at all times. ADR 0107 Decision 2 gives it only for a Source cause.
- The cause "Select mpv in Settings, Audio" has no action button. ADR 0107 Decision 2 gives each cause its action.
