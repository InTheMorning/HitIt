# ADR 0105 Task 002: "End Show" And The Last Track

## Status

Implemented - 2026-11-19. The mechanical criteria pass. The visual criteria wait for a person, in section 45 of the pending human checks.

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

Main session choice, 2026-11-19:

- "On air" means that the encoder reports a connection in the broadcast readiness snapshot. The app keeps no second on air flag. A part is "on" when its own state says so: the music when the Show transport plays or is paused, the stream when the encoder is connected, and each Live Metadata service when its unit is active.
- A pure model `EndShowSequence`, next to `OnAirSequence` in `src/view_models/show.rs`, holds the order. It takes the parts that are on and gives the steps. A failed step records its error on its row and the model gives the next step. Criterion 2 tests it.
- The end sequence uses the existing command paths: the Show stop of `src/app/playback_bar.rs`, `run_stream_encoder_command` with `StreamEncoderOperation::Disconnect`, and `run_publisher_service_command` with `PublisherServiceOperation::Stop`. It uses the same step id field and the same `advance_on_air` link as task 001. Only one run exists at a time: "Go on air" is not available while an end run exists, and "End show" is not available while a Go on air run exists. A step that cannot start fails at once, as in task 001.
- A pure model `EndBand` holds the start instant of the end band. A function with the current instant gives the seconds left, or the end. Criterion 4 tests it with fixed instants. The app updates the band with one timer of 1 s that GPUI runs while the band shows. The constant has the name `END_BAND_DELAY` and the value 60 s.
- The signal for the end band is the transport change to stopped at the last entry, as `src/playback_owner.rs` gives it. A pure function takes that change and the on air fact, and gives the end band or nothing. Criterion 3 tests it.
- If a step needs a decision that this task file and ADR 0105 do not give, the implementer stops and reports.

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

This task adds the end sequence and countdown to the Show view model and app flow.

The sequence reuses task 001 command paths. It continues after each error and shows each error on its part row.

The playback owner reports when the last entry ends. The end band counts down through the existing one-second playback watch.

The end sequence does not change the Show cue. Playback-owner tests cover cue retention.

The main session review made these changes:

- The new playback outcome `LastEntryEnded` also starts the tag update scan and reloads the playlists, as `SessionStopped` does (ADR 0076 packet 005, ADR 0068 task 002). The two arms call one helper, `release_show_files`. The ADR 0076 guard test now examines the two arms and the helper.
- When the playback advances again, the app closes the end band. Thus the end band does not stop music that the host started again.
- The end band uses the playback watch of each `PLAYBACK_POLL_INTERVAL`. It does not have its own timer of 1 s. The stop at the last entry itself does not change.

The full checks pass. V4 and V5 wait for a person in section 45.
