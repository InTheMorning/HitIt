# ADR 0107: The Cards Hold The On Air Status

## Status

Accepted - 2026-11-19. The [agent walk of 2026-11-19](../reviews/agent-walk-2026-11-19.md#operator-requests---2026-11-19) records the operator choice for items 7 and 8. This record is first. It removes the text above the cards and makes "End show" safe. A second record gives the list of shows.

- Bar: opposite choice by mistake. ADR 0105 Decisions 2 and 4 tell an agent to put the checklist and the stream lines above the cards. ADR 0105 Decision 5 tells an agent to disconnect each connected stream. Without this record, an agent keeps the two rules.
- Changes: [ADR 0105](0105-go-on-air-and-end-show.md) Decisions 1, 2, 4 and 5. The band has no status text. The cards and their inspectors show the status. "End show" stops only the parts that HitIt started.

Terms:

- The **inspector** is the panel that shows the data of one card (`ShowPanelMode::Detail`, ADR 0063). This is a project term.
- **butt** is the stream encoder program that HitIt controls (ADR 0059). This is an established name.

## Context

- In the operator check of 2026-11-19, the text lines of the on air band used too much space. The operator found that they do not help the host.
- During that check, the operator had a stream through Mixxx and butt. HitIt did not start that stream. HitIt read the connection of butt as "on air", and "End show" was available. A click on it disconnects butt and stops the stream of the operator.
- Row 3 of ADR 0105 Decision 2 gives the address of the listeners. HitIt has no field for this address. The field `address` of the encoder configuration is the control address of butt.
- Row 1 of ADR 0105 Decision 2 warns until the host answers the test sound question of ADR 0104. The operator asks to remove that question (walk record, item 1).

## Decision

### 1. The band has no status text

1. Above the cards, the Show page shows one row of buttons: "Go on air" or "End show". It shows no checklist rows, no signal line and no listener line.
2. The end band of ADR 0105 Decision 6 stays. It asks for an answer in 60 seconds, so it is not status text.
3. The switch "Show information" moves to the inspector of the Live Metadata card.

### 2. The cards hold the status

Each readiness condition of ADR 0105 Decision 2 stays. It continues to control "Go on air". The text moves to the card that owns the part:

| Status | Card | Inspector text |
| --- | --- | --- |
| mpv is not the Show player, or `ping` fails | Source | The cause and its action |
| The cue has no entries, or an entry is not ready | Source | The cause and "Open playlists" |
| The encoder is not configured | Stream | The cause and "Change" |
| The stream gets silence | Stream | "The stream gets silence", in the warning color |
| Sound at the stream is not known | Stream | "Sound at the stream is not known" |
| The event is not selected or not Live | Live Metadata | The cause and "Add" |
| A step of "Go on air" or "End show" fails | The card of that part | The error and "Try again" |

1. A card with a problem uses the Attention state of ADR 0063, or the Failed state for an error. The card adds no new marker. A new marker needs the shared rule check of musicindex ADR 0045 R-SYSTEM-1 first.
2. While the stream is connected, the Stream card shows the signal state and the listener count of ADR 0105 Decision 4 as its summary. The limit of ADR 0105 Decision 4 stays: no text says that listeners hear the show.
3. A missing test sound result gives no warning.
4. No card shows an address for the listeners.
5. While "Go on air" is not available, the button shows a short cause, for example "Stream is not ready". The cause names the card.

### 3. "End show" stops only what HitIt started

1. HitIt records each part that it starts or connects in this run of the app: the music, the stream connection, and each Live Metadata service. "Go on air" records a part. The controls of the cards also record a part.
2. "End show", and the end of ADR 0105 Decision 6 step 3, stop only the recorded parts that are on. The confirmation lists only those parts.
3. A stream that was connected when HitIt found it is not a recorded part. The Stream card shows "Connected outside HitIt". "End show" does not disconnect it.
4. "End show" is available when one or more recorded parts are on.
5. The Disconnect control of the Stream card stays. It is a different command of the host (ADR 0069).

Limit: the record is for one run of the app. After a restart of HitIt during a show, "End show" does not stop the stream. The host uses the Disconnect control of the Stream card.

## Invariants

- Above the cards, only the end band gives a status.
- HitIt does not disconnect a stream that it did not connect. Only the Disconnect control of the Stream card can do this.

## Alternatives

- **Keep the checklist, with less text.** The operator rejected the text above the cards. Rejected.
- **Keep the record of the started parts after a restart.** After a crash, or after a change that HitIt did not make, a saved record can be incorrect. Then "End show" can stop a stream that a different program owns. Rejected for this record. The record of the list of shows can give each show its own owner.
- **A question to the host, when HitIt did not connect the stream.** It adds a question to each "End show" of a stream that HitIt does not own. Rejected: the Disconnect control does the same with no new question.

## Tasks

- Task 001 (to write): Decisions 1 to 3.
