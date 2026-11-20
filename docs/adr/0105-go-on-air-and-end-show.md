# ADR 0105: "Go On Air" And "End Show"

## Status

Accepted - 2026-11-19. The orchestrator session relayed the operator choice of 2026-11-19: the operator selected each proposal of the show host analysis. This record decides row 2, "Going on air", and the "End show" part of row 3.

- Bar: crosses modules. One sequence controls the Show playback owner, the Live Metadata services, the event registry and the stream encoder. It is also an opposite choice by mistake. ADR 0069 states that Start, Stop and Attach stay separate operations. An agent that reads only ADR 0069 does not put them in one command.
- Changes: [ADR 0069](0069-grouped-settings-and-selective-presets.md) §Settings Configures Resources; Show Operates Them. One "Go on air" and one "End show" run Start, Stop, Attach, Connect and Disconnect in a fixed order after a confirmation. The separate controls stay.
- Changes: [ADR 0063](0063-show-dashboard-layout.md). The Show page has the on air band above the cards.
- Changes: [ADR 0068](0068-show-cue-and-audition-isolation.md) §Show Owns Its Transport. After the last entry ends while the stream is connected, the Show page shows the end band of Decision 6.
- Changed by: [ADR 0107](0107-the-cards-hold-the-on-air-status.md), 2026-11-19. Decisions 1, 2, 4 and 5: the band has no status text, the cards and their inspectors show the status, and "End show" stops only the parts that HitIt started.

Uses:

- The analysis `research_notes/ui-design/journeys/hitit/analysis.md` in `~/build/musicindex-research` (commit `2eb54c7`, corrected in `bc8a4a7`), themes T3, T5, T6 and T10.
- The mockups `mockups/2b-air-proposed.html` and `mockups/3b-show-proposed.html` of that analysis. Their step counts are estimates.
- [ADR 0104](0104-the-show-player-and-the-sound-setup.md): the Show player and the result of the test sound.
- The orchestrator answer of 2026-11-19 about the relay: the relay (musicindex-live-relay `fcb4349`) and the publisher (`62523ea`) carry no audio. They give no level and no listener count.

Terms:

- **On air** is the condition in which the stream encoder reports a connection. This is a term of this record.
- The **on air band** is the area at the top of the Show page with the checklist, "Go on air" and "End show". This is a term of this record.
- **butt** is the stream encoder program that HitIt controls (ADR 0059). This is an established name.

## Context

- Going on air is 12 separate steps on three cards: Source, Live Metadata and Stream (`src/view_models/show.rs:2097`). The page gives no order and no confirmation.
- Show Stop stops the driver, the session and the drop file (`src/playback_owner.rs:430`). The last EOF does the same (`src/playback_owner.rs:514`). The two do not stop the stream or the services.
- The relay has no route that lists or deletes an event (ADR 0059 §Context). HitIt cannot end an event. The relay removes an idle event later.
- HitIt reads `butt -S`. It gives `AudioSignalState` (Present, Absent or Unknown) and a listener count (`src/broadcast/encoder.rs:163`).
- ADR 0059 §This App Is A Control Surface: the broadcast chain operates when the app is closed, except the `mpv` source.

## Decision

### 1. The Show page has one on air band

1. The on air band is above the cards of ADR 0063. The cards keep their order and their controls. They are the "Details" of the band.
2. The band is for the source `mpv`. For another source, the band shows only the stream and Live Metadata rows, and the host starts the source outside HitIt.
3. The band uses words for the host. A project word, such as "drop file", "Producer" or "Relay sender", shows only in the cards.

### 2. The checklist has rows in a fixed order

Each row shows a state in words and, when it is not ready, one action.

1. **Sound plays.** Ready when the Show player is mpv and `ping` succeeds. The row also says if the host heard the test sound of ADR 0104 in this run of the app. A missing test result is a warning, not a block. Action: "Play test sound".
2. **Show cue.** Ready when the cue has entries and each entry is ready under ADR 0059 and ADR 0076. The row gives the number of tracks. Action: "Open playlists" (ADR 0106).
3. **Stream.** Ready when the encoder is configured. The row says "Listeners use this address" and gives the address. Action: "Change".
4. **Show information (optional).** Ready when an event is selected and Live. The row says "Podcast apps show your show and the track names." When the host turns it off, the row says "Off" and the sequence does not do step 2 of Decision 3. Action: "Add".

The rows read the readiness that the cards already calculate. The band does not make a second readiness.

### 3. "Go on air" runs the parts in order

"Go on air" is available when rows 1 to 3 are ready, and row 4 is ready or off. It opens a confirmation:

- Title: "Go on air now?"
- Text: "Listeners hear "TITLE" after the stream connects." TITLE is the selected cue entry.
- When row 4 is off: "Show information: off. Podcast apps do not show this show."
- Buttons: "Cancel" and "Go on air".

After the confirmation, HitIt does these steps in this order:

1. It starts each Live Metadata service of the selected host that does not run.
2. It attaches the selected event, when row 4 is on.
3. It connects the stream and waits until the encoder reports the connection.
4. It plays the selected cue entry.

Each step waits for its result. When a step fails, the sequence stops. The row of that part shows the error and "Try again". The parts that started stay on, and "End show" stops them. A part that already runs is not started again.

Reason: the publisher must run before the first drop file, so that the relay gets the first track name. The stream connects before the music, so that listeners hear the first track from its start.

### 4. The band says what the stream gets

While on air, the band shows:

1. "Sound reaches the stream" when `AudioSignalState` is Present.
2. "The stream gets silence" as a warning when it is Absent.
3. "Sound at the stream is not known" when it is Unknown.
4. "N listeners" with the note "From the stream server", when butt gives a listener count.

Limit: Present proves sound at the input of butt only. HitIt cannot prove that a listener hears the show. It is an inference that butt reads the listener count from the stream server. The band does not say "Listeners hear the show".

### 5. "End show" stops each live part

1. While on air, the band shows the button "End show" with text. It opens a confirmation.
2. The confirmation lists each part that HitIt stops, and only the parts that are on:
   - "The music"
   - "The stream. Listeners hear nothing after this."
   - "The track names to the relay"
3. It also says: "The show information stays until the relay removes it. Podcast apps can show the show as live for some time." and "The Show cue stays as it is."
4. After the confirmation, HitIt stops the music, disconnects the stream and stops the Live Metadata services, in this order.
5. When a step fails, HitIt does the next steps. The row of each failed part shows its error.

The services stop only by this command or by their own controls. A close of the app does not stop them (ADR 0059).

### 6. The end of the last track opens the end band

1. When the last cue entry ends while on air, the Show page shows the band "The last track ended. The show ends in 60 seconds." with "End show now" and "Keep on air".
2. "Keep on air" closes the band. The stream and the services stay on.
3. When the host does not answer in 60 seconds, HitIt does Decision 5 step 4 without the confirmation.
4. The end of the music at the last entry stays as ADR 0068 states: the Show stops and keeps the cue.

Reason: an end with no question can stop a host who talks after the last track. A question with no time limit sends silence to listeners when the host is away. The band with a delay does both.

## Invariants

- "Go on air" and "End show" each have one confirmation that names the parts.
- The sequence runs the existing commands of the broadcast service layer. A screen does not run an encoder command (ADR 0059).
- The band never says that listeners hear the show.

## Alternatives

- **"Podcast apps show the show as ended."** The mockup says this after "End show". The relay has no route that ends an event, so the text is false. Changed to the text of Decision 5 step 3. A route that ends an event is an upstream request.
- **"Listeners hear the first track in approximately 10 seconds."** No measurement gives the delay of the stream server. Changed to "after the stream connects".
- **End the show at once after the last track.** Rejected. See the reason of Decision 6.
- **Stop the started parts when a step fails.** A stop can disconnect listeners when only the show information failed. Rejected: the host decides with "Try again" or "End show".

## Tasks

- [Task 001](../tasks/adr-0105-task-001-go-on-air.md): Decisions 1 to 4.
- [Task 002](../tasks/adr-0105-task-002-end-show-and-the-last-track.md): Decisions 5 and 6.
