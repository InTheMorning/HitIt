# ADR 0105 Task 002: "End Show" And The Last Track

## Status

Ready - 2026-11-19. It waits for [task 001](adr-0105-task-001-go-on-air.md).

## Goal

One "End show" with a confirmation stops each live part. When the last track ends on air, the end band asks the host and ends the show after 60 seconds.

## Authority

[ADR 0105](../adr/0105-go-on-air-and-end-show.md) Decisions 5 and 6. [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md) §Show Owns Its Transport keeps the stop at the last entry.

## Change

1. The band view model gives "End show" while on air. The confirmation lists only the parts that are on, with the texts of ADR 0105 Decision 5. It also gives the two notes about the show information and the cue.
2. `src/app/show.rs`: the end sequence stops the music, disconnects the stream and stops the Live Metadata services, in this order. A failed step does not stop the next steps. Each failed part shows its error on its row.
3. When the playback owner stops at the last entry (`src/playback_owner.rs:514`) while on air, the band view model gives the end band. The band has the text "The last track ended. The show ends in 60 seconds.", "End show now", "Keep on air" and a count of the seconds left.
4. After 60 seconds with no answer, the app runs the end sequence without the confirmation. "Keep on air" closes the end band and starts nothing.
5. The 60 seconds come from one constant with the name of ADR 0105 Decision 6.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| On air band view model | extend | src/view_models/show.rs | Task 001 makes it. This task adds "End show" and the end band. |
| Show commands | extend | src/app/show.rs | The end sequence calls the existing stop and disconnect commands. |
| Playback owner | reuse | src/playback_owner.rs | Its stop at the last entry is the signal for the end band. Its behavior does not change. |
| Confirmation dialog | reuse | src/ui/composites/confirmation_dialog.rs | The confirmation of "End show". |

## Mechanical Criteria

1. A view model test: the confirmation lists only the parts that are on, and always gives the two notes.
2. A sequence test with fake commands: the order is stop music, disconnect, stop services. A failed disconnect does not stop the service stop.
3. A test: the last EOF while on air gives the end band. The last EOF while not on air gives no end band.
4. A test with a fake clock: after 60 s with no answer, the end sequence runs. "Keep on air" before 60 s runs nothing.
5. A test: the cue is the same after the end sequence.
6. The commit checks of AGENTS.md pass.

## Visual Criteria

These are in section 45 of [pending human checks](../pending-human-checks.md).

- V4: "End show" opens the confirmation with the parts and the two notes. After "End show", the stream stops for a listener.
- V5: at the end of the last track, the end band shows and counts down. "Keep on air" keeps the stream on. With no answer, the show ends after 60 seconds.

## Report

Empty until the work is done.
