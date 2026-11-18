<!-- Recorded output of Claude inventory run D (sonnet) at commit c95aff3. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

Recorded output of Claude inventory run D at commit c95aff3a2dfd2910125146c5773ac7294b973798.

## 1. Reading this inventory

This inventory describes code paths. The app was not started. Files were not changed. The scope is run D: Show preparation, playback and live.

Quoted text preserves the code's spelling, capitalization and punctuation. Templates use braces for values that the code supplies at runtime, for example `{title}`.

References are repository-relative `file:line`. Line numbers come from the code at commit c95aff3. A reference with a function name and no number means the number was not recorded during the read.

Every count is **INFERENCE**. One click, one key, one typed field or one list selection is one step. A wait or an automatic action is zero steps. A native file selection is one step. A count starts at the start point that its job names.

**UNSURE** marks a fact that the code does not establish. Examples are native dialog text, mpv behavior, relay behavior and how a window looks.

Not all files in the scope were read. These were not read, so what they do is **UNSURE**: `src/broadcast/*` (control, encoder, producer, publisher_targets, registry, tokens, transport, live_value), `src/app/publisher_dispatch.rs`, `src/ui/shells/publisher.rs`, `src/view_models/publisher_page.rs`, `src/view_models/live_run.rs`, `src/view_models/live_status.rs`, `src/view_models/live_page.rs`, `src/ui/shells/library/live_page.rs`, and `src/app/show.rs` lines 830 to 2075 and 2846 to 3124 (only function names and error constructors were seen there).

Terms. "Show" is the project name for the live-performance screen and its stored play list (the "cue"). "Cue" is the ordered list of tracks that Show plays. "Event" is a live broadcast registration at the relay. "Producer" and "Relay sender" are two services that the Show page can start and stop. "Stream" is the audio encoder connection. These are project terms.

## 2. Summary of what exists and what does not

- The Show page is a tab. Its label is "Show" (`src/view_models/sidebar.rs:371`). The shortcut is "Ctrl+2" on Linux (`src/app/keyboard.rs:152`, registered as `cmd-2`, label "Show").
- The only way to put tracks in the cue is a playlist action: "Load into Show Cue" or "Replace Show Cue" (`src/view_models/library.rs:790`, `src/view_models/library.rs:792`).
- No control and no caller exists for "play one track" or "seek". `PlayTrack` and `SeekPlayback` exist as commands in `src/application/commands/playback.rs`. A search of `src` found no caller outside that file. The owner has a `seek` method (`src/playback_owner.rs`) with no UI path.
- No control is named "Go live", "Go Live", "End show" or "End the show". A search of `src` for these words found no label. Going live is a set of separate controls on the Show page (section 7). Ending the show is a set of separate controls (section 14).
- The stream recording has a state row ("Recording") but no start or stop recording control was found in the VM (`src/view_models/show.rs:2886`, `:2895` list only "Connect" and "Disconnect"). Whether "Connect" starts a recording is **UNSURE**.
- The default playback driver is `Null`. The config text says: `# Playback backend. The default "null" driver disables playback.` (`src/config.rs:1167`, default at `src/config.rs:115`). The walks record states that no walk played audio (`docs/research/2026-11-18-user-journeys/walks.md`). All playback, relay and stream paths below are Code only.

## 3. Show page layout and parts

The Show page has a summary, a card grid, a bottom log pane, a transport deck and a side panel. Source: `src/ui/shells/show.rs`.

| Part | Label or text | Evidence |
|---|---|---|
| Summary title | "Show" | `src/view_models/show.rs:1859` |
| Summary fields | now-playing title, artist, duration, state label | `src/ui/shells/show.rs` |
| Empty state | "No active show" and "Show playback is idle." | `src/view_models/show.rs:1859` to `:1865` |
| State labels | "Idle", "Starting", "Playing", "Paused", "Failed" | `src/view_models/show.rs:3020` to `:3024` |
| Not-checked state | state "Not checked", title "Show status unavailable", subtitle "App needs its background runtime to read show status. Use Check again in Background tools." | `src/view_models/show.rs:1778` to `:1782` |
| Card "Source" | state "Reachable", "Not reachable" or "Unknown"; primary is the host name; secondary is the readiness count label | `src/view_models/show.rs:1449` |
| Card "Live Metadata" | state "Ready" or "Not ready"; primary "Event: {badge label}", "{service label}: {badge label}", "Event, Producer, Relay sender: ready" or "Event: loading" | `src/view_models/show.rs:1450`, `:2124` |
| Card "Stream" | state is the connection label; secondary "{count} listeners" or "Listeners unknown" | `src/view_models/show.rs:1451` |
| Side panel | default open, "Cuelist" mode | `src/view_models/show.rs:1561` to `:1574` |
| Panel buttons | "Open" (a11y "Open show side panel"), "Close" (a11y "Close show side panel"), "Cuelist" (a11y "Return show side panel to cuelist") | `src/view_models/show.rs:1561` to `:1574` |
| Status line | red text under the summary, from `status_message` | `src/ui/shells/show.rs:331` to `:341` |

Card ids are "show-card-source", "show-card-live-metadata" and "show-card-stream" (`src/ui/shells/show.rs`). A click on a card selects it and opens its detail in the side panel. A click on the card that is already open closes the panel (`src/view_models/show.rs`, `select_card`).

When a dependency is missing, a card shows the text "Setup needs attention" as its state label and adds the capability text to its secondary line (`src/view_models/show.rs:1688` to `:1736`).

## 4. Queue and now playing (the Cuelist)

The Cuelist is the side panel in Cuelist mode. Source: `src/ui/composites/show_detail_panel.rs`, `src/ui/shells/queue_now_playing.rs`, `src/view_models/queue_now_playing.rs`.

| Item | Label | Evidence |
|---|---|---|
| Empty list text | "Queue is empty" | `src/view_models/queue_now_playing.rs:524` |
| Row accessibility label | "{Now playing, }{title}, by {artist}, {duration}" and ". Unavailable: {reason}" when unavailable | `src/view_models/queue_now_playing.rs` |
| Missing title | "Untitled track" | `src/view_models/queue_now_playing.rs` |
| Unavailable reasons | "Not in Library", "Not downloaded", "File missing" | `src/view_models/queue_now_playing.rs:192` to `:194` |
| Transport "Previous track" | icon button | `src/view_models/queue_now_playing.rs:308` |
| Transport play/pause | icon button; a11y "Pause Show", "Resume Show", "Start Show", "No Show cue is loaded", "Starting Show" | `src/view_models/queue_now_playing.rs:60` to `:64` |
| Transport "Next track" | icon button | `src/view_models/queue_now_playing.rs:313` |
| Transport "Stop Show" | icon button | `src/view_models/queue_now_playing.rs:318` |
| "Clear" | button, id "show-clear-cue", a11y "Clear Show cue", destructive style | `src/view_models/queue_now_playing.rs:148` |

Transport availability by state (`src/view_models/queue_now_playing.rs`, `src/app/queue_now_playing.rs`): Stopped enables Start and row Select. Playing enables Pause and Stop. Paused enables Resume and Stop. Failed enables Start and Stop. Empty and Starting enable no play/pause action. "Previous track" and "Next track" depend on the cue position. "Clear" is available only when a cue exists and the state is Stopped.

A row click calls row select only when the session is stopped (`can_select` is true only with no active session, `src/app/queue_now_playing.rs`). A running session ignores row clicks.

There is no control to reorder the cue. There is no drag handle in the cue rows. Playlist track reorder ("Move Up", "Move Down", drag) exists on the playlist page, not on the cue (`src/view_models/library.rs`, `move_playlist_track`). Whether a change to the playlist updates a loaded cue is **UNSURE**.

## 5. Playlist into the cue

Entry points:

| Control | Label | Evidence |
|---|---|---|
| Button on the playlist page | "Load into Show Cue" when no cue exists, else "Replace Show Cue" | `src/view_models/library.rs:790`, `:792`; `src/ui/shells/library/playlist_detail.rs:235` |
| Row menu item in the playlist list | the same two labels | `src/ui/shells/playlist.rs:350` to `:368` |

A11y labels: `Load the playlist "{name}" into the Show cue` and `Replace the Show cue with the playlist "{name}"`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open a playlist (Music, "My playlists", click a playlist). | The playlist page. | `src/view_models/sidebar.rs:367` |
| 2 | Click "Load into Show Cue" or "Replace Show Cue". | The cue is stored. The page stays on Music. A `ShowCueLoaded` event refreshes the Show page. | `src/library/app_impl.rs:2185` to `:2250`, `src/app.rs:439` |

Branches and errors:
- "Replace Show Cue" is disabled when a cue exists and playback is not stopped (`ReplaceUnavailable`, `src/view_models/library.rs:775` to `:800`).
- No confirmation for Load or Replace.
- Error in the library sidebar status: `The Show cue did not change: {error:#}` (`src/view_models/library.rs:3963`) and "The playlist has no tracks. The Show cue did not change." (`src/view_models/library.rs:3967`).
- An RSS check for the playlist runs after a load (`cue_load_rss_check_trigger`, `src/library/app_impl.rs`). The result is **UNSURE**.

End state: cue stored, Show page current, active tab unchanged. Step count: 2 from an open playlist page. **INFERENCE**.

Clear the cue: click "Clear" in the Cuelist. A confirmation appears: title "Clear the Show cue?", message "This removes the stored cue.", buttons "Cancel" (a11y "Keep the Show cue") and "Clear" (a11y "Clear the Show cue") (`src/app/show.rs:57` to `:86`). On error: `Show cue clear error: {error:#}` in `settings_status` (`src/app/show.rs:108`).

## 6. Playback: play, pause, resume, next, previous, stop

Entry points: the transport icon buttons (section 4) and these keys (`src/app/keyboard.rs:108` to `:155`):

| Key label | Keystroke in code | On Linux |
|---|---|---|
| "Show Play/Pause" | `cmd-alt-p` | "Ctrl+Alt+P" |
| "Next Show cue entry" | `cmd-alt-right` | "Ctrl+Alt+Right" |
| "Previous Show cue entry" | `cmd-alt-left` | "Ctrl+Alt+Left" |

The Linux conversion of `cmd` to `ctrl` follows the same rule as the Music key in run B. These keys are global (handlers near `src/app/keyboard.rs:368` to `:440`). In the Show and Settings tab, the active-pane handlers do nothing for arrow, Enter, Escape and Space (`AppTab::Show | AppTab::Settings => {}`, `src/app/keyboard.rs` near `:476`, `:551`, `:567`, `:583`, `:643`, `:660`, `:683`).

Main path:

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the play/pause icon (a11y "Start Show") or press "Ctrl+Alt+P". | State "Starting" until the first poll proves playback, then "Playing". The status line shows "Playing {title}" or "Starting {title}". | `src/app/playback_bar.rs`, `src/app.rs:1107` to `:1152` |
| 2 | Click "Next track". | The next entry is selected and, when playing, started. Message "Selected the next Show cue entry". | `src/app/playback_bar.rs` |
| 3 | Click "Previous track". | Message "Selected the previous Show cue entry". | `src/app/playback_bar.rs` |
| 4 | Click the play/pause icon while playing (a11y "Pause Show"). | State "Paused". Message "Paused {title}". | `src/view_models/queue_now_playing.rs:60` |
| 5 | Click the icon again (a11y "Resume Show"). | State "Playing". | `src/view_models/queue_now_playing.rs:61` |
| 6 | Click "Stop Show". | Message "Playback stopped". State "Idle" or Stopped. | `src/view_models/queue_now_playing.rs:318` |

Rules in the owner (`src/playback_owner.rs`):
- Start fails with "Show must be stopped before it starts a cue entry", "No Show cue is loaded" or "No Show cue entry is selected".
- A cue entry that cannot play gives: "Show cue entry {id} cannot play: the track is not in the library", "...the track is not downloaded" or "...the file is missing".
- When stopped, Next and Previous only select the neighbor entry. They do not play. When paused, they stop the driver and prepare the entry, and Resume loads it.
- No neighbor: "No adjacent Show cue entry".
- On end of file the poll advances to the next entry. At the last entry the session stops. If the next entry cannot play, the session stops and returns the error above. After a stop, the tag update scan runs again and playlists reload (`src/app.rs:1142` to `:1152`).

Errors and where they show:
- Command failure: `Playback error: {error:#}` (`src/app/playback_bar.rs:276`).
- Select failure: `Show selection error: {error:#}` (`src/app/playback_bar.rs:45`).
- No verified player: "App has no verified player. Check playback again." (`src/app/playback_bar.rs:115`). The failed action is retained and the play/pause label becomes "Retry".
- Missing mpv: the error starts with "mpv is not installed at " (`MPV_NOT_INSTALLED_PREFIX`, `src/playback_driver/mod.rs`). The app records `PlaybackProgramMissing` (`src/application/capability.rs:85`). The Show page then shows "Repair playback" in place of the play/pause label (`src/view_models/show.rs:3091`, a11y "Repair playback for the original track").
- A poll error sets the transport state to Failed (`src/app.rs:1126` to `:1131`).

All messages go to `settings_status`. The Show shell prints it in red under the summary (`src/ui/shells/show.rs:331`). The same text appears in Settings (see section 16).

Step counts from "Show open, cue loaded, state Stopped", **INFERENCE**: start 1, next 1, previous 1, pause 1, resume 1, stop 1.

Play one track: **not possible**. No control or caller exists (section 2). A user can build a playlist of one track and load it (Job 11 plus playlist creation).

Seek: **not possible**. No control or caller exists. `src/application/commands/playback.rs` defines `SeekPlayback`. Nothing calls it.

Reorder the queue: **not possible** on the cue (section 4).

## 7. Live: event, producer, relay sender, stream

All controls are on the Show page. A card click opens its detail in the side panel (section 3). Controls in the Live Metadata detail:

Event section (`src/view_models/show.rs:2530` to `:2575`, `:5868`):

| Control | Label | Available when |
|---|---|---|
| Event picker | trigger label is the event label or "Choose an event"; items "{label} · Selected · Configured" with description "{date} · {state} · {id}" | registry loaded |
| Primary button | "Create" | state None, registry loaded, no selected id, no events, no command working |
| Primary button | "Replace" | event state Dead |
| Primary button | "Attach" | event Live, publisher reachable, target list loaded, target "not attached", non-empty attach target name |
| Primary button | "Check" or "Retry check" | state Unknown or a failed check |
| Primary button | "Retry events" | registry load failed |
| Primary button | "Retry config" | event Live, no target list |
| Primary button, disabled | "Creating", "Replacing", "Attaching", "Detaching", "Checking", "Loading" | a command or load is running |
| "Detach" | menu item in "More" | event attached |
| "Check again" | menu item in "More"; a11y "Check whether the relay still has this event and read the publisher configuration again" | event selected |
| "Copy feed tag" | menu item in "More"; a11y when no host: "Copy listener feed tag. The stored relay endpoint of this event has no host." | endpoint has a host |
| "Logs" | opens the log pane | always |
| "More" | menu trigger, a11y "More event actions" | always |

The feed tag text is `<podcast:liveValue uri="..." protocol="socket.io"/>` (`src/view_models/show.rs:6084`).

Event badge labels: "Creating", "Replacing", "Attaching", "Detaching", "Loading", "Events unavailable", "Event unavailable", "No event", "Check failed", "Checking", "Dead", "Unknown", "Target not set", "Attached", "Not attached", "Target unknown", "Commands unavailable", "Not reachable", "Target read failed". Event state labels: "No event", "Unknown", "Live", "Dead".

Event messages: "Registering event…", "Event registered. Token file saved.", "Registration failed: {detail}", "Checking event…", "Event check complete.", "Event check failed: {detail}". Hints: "Token file is missing locally; copy it to the relay sender host.", "Could not save the event choice. Open Logs.", "Could not create the event. Open Logs.", "Publisher configuration action failed. Open Logs.", "Publisher configured for {id}. Attach replaces it."

Target labels: "not attached", "Target unknown", "Target commands unavailable", "Relay sender not reachable", "Target list failed". Target details: "No broadcast event selected.", "The configured relay sender target does not carry this event.", "Target list has not been read.", "Relay sender target commands are not installed.", "Target list cannot be read from the host.", "This target carries the selected event.", "Relay sender returned an empty target name."

Service rows "Producer" and "Relay sender" (`src/view_models/show.rs:2926` to `:2938`):

| Button | Label | Available when |
|---|---|---|
| Start | "Start" | state Inactive |
| Stop | "Stop" | state Active |
| Reset | "Reset" | state Failed |
| Logs | "Logs" | state not "Not reachable" |

Service state labels: "Active", "Inactive", "Starting", "Stopping", "Failed", "Not installed", "Not reachable", "Unknown", "Working". With no unit, badge and name show "Status unavailable". When the dependency is missing the three buttons read "Repair tool" (`src/view_models/show.rs:1688`) and event controls read "Repair action" (`:1718`).

Stream rows and buttons (`src/view_models/show.rs:2886`, `:2895`):
- Rows: "Server", "Connection" (Connected, Connecting, Disconnected, Working, Not installed, Not reachable, Unknown), "Signal" ("Audio present", "No audio signal", "Signal unknown"), "Recording" ("Recording", "Stopped", "Recording unknown"), "Listeners", and when present "Recording elapsed", "Recording file", "Encoder song", "Stream elapsed".
- "Connect": available only when Disconnected. "Disconnect": available only when Connected. Both are absent when the encoder is not configured or state is "Not installed". With a missing dependency both read "Repair encoder" (`:1732`, `:1735`).

Source detail rows: "Host", "Reachability", "Readiness", and a button "Open" (a11y "Open broadcast readiness issues in Music") when the problem count is above 0 (`src/app/show.rs:1136`, `open_broadcast_readiness_in_music`).

Command dispatch (`src/app/show.rs`): `run_event_control` (2855), `select_show_event` (2892), `read_event_targets` (2975), `dispatch_event_registry_command` (822), `dispatch_event_target_command` (941), `run_stream_encoder_command` (1096), `run_publisher_service_command` (509), `retry_show_action` (622), `finish_show_command` (704). A failed service command sets `settings_status`. "Relay sender command error: {error:#}" is set when no host or command can be built. After a change of the selected event, `refresh_show_page` runs a Check automatically (zero steps). Event reports go to the status bar through `record_show_reports` (ADR 0101 Decision 7). What each command does on the relay or the host is **UNSURE** (`src/broadcast/*` not read).

## 8. Log pane

The "Logs" button of an event or a service opens the bottom log pane. Texts: "Reading service logs…", "Reading logs", "No log lines returned.", "Log read failed", "{n} lines", "Saved event details", and unit name "Event · {event id}". Close button "Close" (a11y "Close logs"). Source: `src/ui/composites/show_log_pane.rs`, `src/view_models/show.rs`. Exact line numbers were not recorded.

## 9. Library "Live" list

The sidebar row "Live" (`src/view_models/sidebar.rs:369`) opens a list of current Index live items. It reads `GET /v1/live-items` with view=now (`src/library/live_page.rs`). Status rows: "Try again" and "Load more". This list is not a control of the user's own live show. Row text and the empty text are **UNSURE** (`src/ui/shells/library/live_page.rs` and `src/view_models/live_page.rs` not read).

## 10. Job 11: play a playlist. Start: app open

Current path:

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "My playlists" (or Ctrl+1 then the sidebar row). | The playlist list. | `src/view_models/sidebar.rs:367` |
| 2 | Click a playlist. | The playlist page. | `src/ui/shells/library/playlist_detail.rs` |
| 3 | Click "Load into Show Cue" (or "Replace Show Cue"). | The cue is stored. No tab change. | section 5 |
| 4 | Click "Show" or press "Ctrl+2". | The Show page with the cue in the Cuelist, state "Idle" or Stopped. | `src/app/keyboard.rs:152` |
| 5 | Click the play/pause icon (a11y "Start Show") or press "Ctrl+Alt+P". | "Starting", then "Playing". | section 6 |

Count: 5 steps. **INFERENCE**. The starting tab is not named, so step 1 can need zero or one extra step. Key "Ctrl+Alt+P" works from any tab, so step 4 can be skipped when the user does not need to see the Show page (4 steps). The load can also start from the playlist row menu (same label).

The path works only when the driver is "mpv" and mpv is installed. With the default "null" driver the result is **UNSURE**: the code says playback is disabled (`src/config.rs:1167`). The app then shows the capability text (for example "Repair playback"). The driver is set in Settings, Diagnostics, configuration repair (`playback.driver`, `playback.mpv_path`; `src/application/capability.rs:257`).

## 11. Job 12: go live with no event yet and music that plays. Start: app open

No single control exists. The path is a composition (order is not enforced by the code, **UNSURE**):

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 to 3 | Open a playlist and click "Load into Show Cue". | Cue stored. | section 5 |
| 4 | Click "Show". | Show page. | section 3 |
| 5 | Click the "Live Metadata" card, then click "Create" in the Event section. | "Registering event…", then "Event registered. Token file saved." | `src/view_models/show.rs:2530` |
| 6 | Click "Attach". | "Attaching", then badge "Attached". Requires a reachable publisher and a target list. | `src/view_models/show.rs:2562` |
| 7 | Click "Start" on "Producer". | Badge "Starting", then "Active". | `src/view_models/show.rs:2926` |
| 8 | Click "Start" on "Relay sender". | Badge "Starting", then "Active". | `src/view_models/show.rs:2926` |
| 9 | Click the "Stream" card, then click "Connect". | Connection "Connecting", then "Connected". | `src/view_models/show.rs:2886` |
| 10 | Click the play/pause icon. | "Playing". | section 6 |

Count: 11 steps with card clicks counted (Create needs the Live Metadata card open: 1, Stream card: 1). **INFERENCE**. Whether the Producer or the Relay sender needs a start depends on the host setup (**UNSURE**; `src/broadcast/*` not read). The readiness check (section 12) is not a gate. No code found blocks Connect when the readiness shows blockers.

## 12. MH3 Get show-ready: the pre-show check

Done when the app says "Ready" with zero blockers.

What the code shows:
- The string "Ready" exists as the state label of the "Live Metadata" card (`src/view_models/show.rs:2124`), together with "Not ready". It is true when the event, the Producer and the Relay sender are all fine.
- The Source card shows a readiness count label (`src/view_models/show.rs`): "Readiness unavailable", "No local tracks", "{N track(s)} ready", "{N tracks} not ready", "Checking readiness". Details for "not ready" are built from: "{n} removed from feed", "{n} with an out-of-date payment route", "{n} without payment routes", "{n} need relay sender routes", "{n} with a missing file", "{n} not downloaded". Detail for "ready": "Every scanned track carries payment routes."
- No single place says "Ready" for the whole show. The user must read three cards. That is a finding for the study, not a rule.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Show". | The three cards. The page refreshes by itself (zero steps). | `src/app/keyboard.rs:152` |
| 2 | Read the "Source", "Live Metadata" and "Stream" cards. | States as above. | `src/view_models/show.rs:1449` |
| 3 | If the Source card shows blockers, click the "Source" card, then "Open". | Music opens on the readiness issues. | `src/app/show.rs:1136` |
| 4 | If the event state is stale, click "Check" or "Check again". | "Checking event…", then "Event check complete." | `src/view_models/show.rs:2543` |

Count: 1 step when everything is ready; 3 to 5 with a blocker. **INFERENCE**. When the app has no background runtime, the page shows "Not checked" and the text "App needs its background runtime to read show status. Use Check again in Background tools." That control is in Settings (section 16 lists this as a result away from its control).

## 13. Job 13: go live with an event already attached. Start: app open

| Step | User action | What appears next |
|---|---|---|
| 1 | Click "Show". | Show page; the Live Metadata card should read "Ready". |
| 2 | Click "Start" on "Producer" if its badge is "Inactive". | "Active". |
| 3 | Click "Start" on "Relay sender" if its badge is "Inactive". | "Active". |
| 4 | Click the "Stream" card, then "Connect". | "Connected". |
| 5 | Click the play/pause icon. | "Playing". |

Count: 5 to 7 steps (card clicks included), plus 4 to 5 more when no cue is loaded (section 5 plus the "Show" click). **INFERENCE**. Evidence: sections 6 and 7. Whether the Producer and Relay sender are already active after a reboot is **UNSURE**.

## 14. Job 14: cue a different next track. Start: Show open

Impossible while a show is playing. Evidence: row select requires no active session (`can_select`, `src/app/queue_now_playing.rs`); the cue has no reorder control (section 4); the playlist reorder exists only on the playlist page, and "Replace Show Cue" is disabled while playback is not stopped (`src/view_models/library.rs:775` to `:800`).

What a user can do:
- While stopped: click a cue row. 1 step. The row becomes the entry that Start plays (`select_show_cue_entry`, `src/app/playback_bar.rs`). Message "Show cue entry selected".
- While playing: click "Next track" (1 step). The next entry is selected and started at once. The current track ends. This is not "cue the next track".
- Pick a different track by editing the playlist and replacing the cue: requires "Stop Show" (1), then the playlist page and "Replace Show Cue" (about 3, from section 5). **INFERENCE**.

## 15. Job 15: end the show. Start: Show open

No "End show" control exists. Ending is a set of single actions:

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Stop Show". | "Playback stopped". Music stops. No confirmation found. | `src/view_models/queue_now_playing.rs:318` |
| 2 | Click the "Stream" card, then "Disconnect". | Connection "Disconnected". | `src/view_models/show.rs:2895` |
| 3 | Click "Stop" on "Relay sender". | "Inactive". | `src/view_models/show.rs:2932` |
| 4 | Click "Stop" on "Producer". | "Inactive". | `src/view_models/show.rs:2932` |
| 5 (optional) | Click "Detach" in "More". | Event "Not attached". | `src/view_models/show.rs:2572` |

Count: 1 step to stop the music; 5 to 6 steps to stop the music and all live parts (card clicks included). **INFERENCE**. The code does not make "Stop Show" stop the stream or the services. Whether the relay marks the event "Dead" after a stop is **UNSURE**. The "Stop Show" button has no confirmation; the "Clear" button has one (section 5).

## 16. MH4 Play the show with the built-in player

Done when music plays in order, can be steered and ends cleanly.

- In order: yes. The poll advances to the next entry on end of file (section 6).
- Steered: Start, Pause, Resume, Next, Previous, Stop exist. Seek, jump to a chosen entry while playing and reorder do not exist.
- Ends cleanly: at the last entry the session stops and the Show page refreshes, the tag scan runs and playlists reload (`src/app.rs:1142` to `:1152`). A next entry that cannot play stops the session with the "cannot play" error text (section 6).
- Count from "Show open, cue loaded": 1 step to start, then 0 steps until the end. **INFERENCE**.
- mpv, audio device and end-of-file timing are **UNSURE** (not run).

## 17. MH5 Go live and come off air

- Count on: from "Show open, event attached, cue loaded": 4 steps (Producer "Start", Relay sender "Start", "Connect" with its card click, play) plus 1 more when "Connect" needs a card click: 5. With no event: add "Create" and "Attach" and the "Live Metadata" card click (about 3 more). **INFERENCE**.
- Count off: 5 to 6 steps (section 15). **INFERENCE**.
- There is no one-step go live and no one-step end.

## 18. Narrow layout (below 940 px)

- Below 940 px (`SHELL_TAB_BAR_BREAKPOINT`, `src/ui/layouts.rs:73`; used in `src/app.rs:2227`) a tab bar replaces the sidebar. It holds Search, Releases, Show and Inbox, plus "More" (`src/view_models/sidebar.rs:223` to `:237`, `:544` to `:556`). "Show" stays one click away.
- The Show card grid depends on window width, not on the tab bar: Compact below 712 px with one column, Medium below 1056 px with two columns, Wide with three (`src/view_models/show.rs:1603` to `:1612`).
- Which parts of the page differ in the Compact class (side panel, log pane, transport deck) is **UNSURE**; the layout code in `src/ui/shells/show.rs` was only partly read.
- The transport buttons are icon-only at all widths. The keys in section 6 work at all widths.

## 19. Controls with no handler, or a handler that does nothing

| Item | What happens | Evidence |
|---|---|---|
| "Select", "Pause", "Resume", "Start" in the Show actions list | Actions exist in the VM; no button of their own. Only the icons and row clicks carry them. | `src/view_models/queue_now_playing.rs:120` to `:148` |
| Play/pause icon when Empty or Starting | Disabled. | `src/view_models/queue_now_playing.rs:63`, `:64` |
| "PlayTrack", "SeekPlayback" | Commands with no caller. | `src/application/commands/playback.rs` |
| Arrow, Enter, Escape, Space on the Show tab | Do nothing in the active pane. | `src/app/keyboard.rs` near `:476` to `:683` |
| Cue rows while a session is active | A click does nothing. | `src/app/queue_now_playing.rs` |
| "Open" and "Close" panel icon buttons | Show no label text; a11y only. | `src/view_models/show.rs:1561` to `:1574` |
| A control with no handler set | Rendered disabled. | `src/ui/composites/show_detail_panel.rs` |
| Recording | State rows only, no control found. | `src/view_models/show.rs:2886` |

## 20. Labels that differ for one action

| Action | Labels | Evidence |
|---|---|---|
| Start, pause, resume | visible "Start", "Pause", "Resume", "Retry", "Starting"; a11y "Start Show", "Pause Show", "Resume Show", "Starting Show"; key "Show Play/Pause" | `src/view_models/queue_now_playing.rs:60` to `:64`, `src/app/keyboard.rs:112` |
| Stop | action "Stop"; a11y "Stop Show"; message "Playback stopped" | `src/view_models/queue_now_playing.rs:135`, `:318` |
| Next, previous | "Next track" and "Previous track" (icons); keys "Next Show cue entry" and "Previous Show cue entry"; messages "Selected the next Show cue entry" and "Selected the previous Show cue entry" | `src/view_models/queue_now_playing.rs:308`, `:313`, `src/app/keyboard.rs:118`, `:124` |
| Event check | "Check", "Retry check", "Check again", "Retry events", "Retry config" | `src/view_models/show.rs:2543`, `:2545` |
| Open panel, Open readiness | "Open" for both; a11y differ | `src/view_models/show.rs:1561` |
| Close panel, close logs | "Close" for both; a11y "Close show side panel", "Close logs" | `src/view_models/show.rs:1561` |
| Fix a missing dependency | "Repair tool", "Repair action", "Repair encoder", "Repair playback" | `src/view_models/show.rs:1688` to `:1735`, `:3091` |
| Load the cue | "Load into Show Cue", "Replace Show Cue" | `src/view_models/library.rs:790`, `:792` |
| Empty cue | "Queue is empty" in the list; "No Show cue is loaded" as a11y | `src/view_models/queue_now_playing.rs:524`, `:63` |
| Queue words | Panel "Cuelist"; code and list text "Queue", "Show cue" | `src/view_models/show.rs:1561`, `src/view_models/queue_now_playing.rs` |

## 21. Results or errors that show away from their control

| Cause | Where the text shows | Evidence |
|---|---|---|
| Any transport command, event command or service command | `settings_status`, shown in red under the Show summary and also in Settings | `src/ui/shells/show.rs:331` to `:341`, `src/app/playback_bar.rs:45`, `:276` |
| "Load into Show Cue" and "Replace Show Cue" errors | Library sidebar status, not the Show page | `src/view_models/library.rs:3963`, `:3967` |
| Event reports | Status bar | `src/app/show.rs` (`record_show_reports`) |
| Readiness blockers | "Open" in the Source detail moves the user to Music | `src/app/show.rs:1136` |
| "Check again in Background tools" | The control is in Settings, the text is on Show | `src/view_models/show.rs:1778` to `:1782` |
| Poll failure | Transport state "Failed" with no matching text at the control | `src/app.rs:1126` to `:1131` |
| Cue clear error | `settings_status` | `src/app/show.rs:108` |

## 22. UNSURE points that matter

- Whether any control starts or stops a stream recording. Only state rows were found.
- What the "Null" driver shows when the user clicks the play/pause icon. The config text says it disables playback.
- What each broadcast command does on the host and the relay (`src/broadcast/*` and `src/app/publisher_dispatch.rs` not read).
- Whether the Producer or Relay sender start is needed in a normal setup, and whether the order of the live steps matters.
- How the Show page changes below 712 px for the side panel, the log pane and the transport deck.
- Whether a change to a playlist updates a loaded cue.
- The text and the empty state of the Library "Live" list rows.
- Native dialog text, mpv behavior, audio output and timing. The app was not started.
