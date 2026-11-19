# ADR 0105 Task 001: "Go On Air"

## Status

Ready - 2026-11-19. It waits for [ADR 0104 task 001](adr-0104-task-001-mpv-default-and-test-sound.md), because row 1 reads the test result.

## Goal

The Show page has the on air band. It has a checklist in order and one "Go on air" with a confirmation. A sequence starts the parts in order, and the band shows the state of the stream.

## Authority

[ADR 0105](../adr/0105-go-on-air-and-end-show.md) Decisions 1 to 4. [ADR 0059](../adr/0059-broadcast-control-surface.md) §Invariants: encoder commands run only in the broadcast service layer.

## Change

1. `src/view_models/show.rs`: add an on air band view model. It gives the four rows of ADR 0105 Decision 2, each with a state, a text and one action. It reads the readiness of the cards. It does not calculate a second readiness.
2. The band view model gives the availability of "Go on air" and the text of the confirmation, with the title of the selected cue entry.
3. `src/app/show.rs`: add a sequence that runs the existing commands in the order of ADR 0105 Decision 3. Start each Live Metadata service that does not run. Attach the event when row 4 is on. Connect the stream and wait for the connection. Then play the selected entry. Each step waits for its result. A failure stops the sequence and puts the error on its row with "Try again".
4. While on air, the band gives the stream text of ADR 0105 Decision 4 from `AudioSignalState` and the listener count. It never says "Listeners hear the show".
5. `src/ui/shells/show.rs`: render the band above the cards. Project words show only in the cards.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Show view model | extend | src/view_models/show.rs | It gives the band, the rows and the confirmation. |
| Show commands | extend | src/app/show.rs | The sequence calls the existing service, attach, connect and play commands. |
| Service control | reuse | src/broadcast/control.rs | `start` and `show` give the service state. |
| Encoder control | reuse | src/broadcast/encoder.rs | Connect, the connection state, `AudioSignalState` and the listener count. |
| Confirmation dialog | reuse | src/ui/composites/confirmation_dialog.rs | The confirmation of "Go on air". |
| Show render | extend | src/ui/shells/show.rs | It renders the band above the cards. |

## Mechanical Criteria

1. A view model test: the rows come in the order Sound plays, Show cue, Stream, Show information. Each row gives its state text and its action.
2. A view model test: "Go on air" is not available when rows 1 to 3 are not ready. A missing test result gives a warning and does not block.
3. A sequence test with fake commands: the order is service start, attach, connect, play. A running service is not started again. With row 4 off, attach does not run.
4. A sequence test: a failed connect stops the sequence, play does not run, and the Stream row gives the error and "Try again".
5. A view model test: Present, Absent and Unknown give the three texts of ADR 0105 Decision 4. No text contains "Listeners hear".
6. The architecture test of ADR 0059 that keeps encoder commands out of screens passes with no change.
7. The commit checks of AGENTS.md pass.

## Visual Criteria

These are in section 45 of [pending human checks](../pending-human-checks.md).

- V1: the band is above the cards. The four rows show in order, in Light and Dark, at 1440 px and at 390 px.
- V2: "Go on air" opens the confirmation with the first track title. "Go on air" in the confirmation starts the parts, and the person hears the stream from the first track.
- V3: with the stream input muted, the band shows "The stream gets silence".

## Report

Empty until the work is done.
