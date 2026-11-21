# ADR 0111 Task 003: A Show That Mixxx Plays

## Status

Ready after task 002 - 2026-11-21. Implementation has not started.

## Goal

The Show page has a player setting, HitIt or Mixxx. With Mixxx, "Go on air"
switches the producer, checks the event, connects the stream and starts the
recording. "End show" stops each part that HitIt started and starts the
private producer again, also after a restart of HitIt. The Show page has the
action "Make show instance" and shows the state of each instance.

## Authority

[ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md) Decisions
3, 4 (GUI), 6, 7 and 8 (render), and the stored fact of Decision 7. It
changes [ADR 0105](../adr/0105-go-on-air-and-end-show.md) Decisions 3 and 5
and [ADR 0107](../adr/0107-the-cards-hold-the-on-air-status.md) Decision 3.

## Change

1. Player setting (Decision 6). `src/view_models/show.rs`: a new type
   `ShowPlayer` with `HitIt` (default) and `Mixxx`, and the selected show
   instance. The Show page shows the setting in the Source inspector. With
   HitIt, `OnAirSequence` does not change.
2. "Go on air" with Mixxx (Decision 6). `OnAirSequence` gets the steps
   `SwitchProducer`, `CheckEvent`, `ConnectStream` and `StartRecording`, in
   that order, for Mixxx. It has no `AttachEvent` and no `PlayCue` for
   Mixxx. A failed step stops the sequence. `StartRecording` calls
   `encoder::start_recording` and stores the wall clock time of the start.
3. "End show" with Mixxx (Decision 7). `EndShowSequence` gets the steps
   `StopRecording`, `StopShowProducer`, `StopShowPublisher` and
   `StartPrivateProducer`, in the order of Decision 7, with
   `DisconnectStream` as step 2. Each step is in the sequence only for a part
   that HitIt started. A failed step does not stop the sequence.
   `StartedParts` records the recording and the show instance.
4. The stored fact (Decision 7). HitIt stores "the switch of this show
   stopped the private producer" in its database with the name of the show
   instance and the name of the private instance. HitIt stores it before the
   switch stops the private producer. HitIt clears it when the switch fails
   and the private producer starts again, and after `StartPrivateProducer`.
   At start, HitIt reads the stored fact. When it is set, "End show" is
   available and its sequence has `StopShowProducer`,
   `StopShowPublisher` and `StartPrivateProducer`.
5. "Make show instance" (Decision 3, GUI). The Live Metadata inspector gets
   the action. Its dialog has the instance name, the target name (default
   `default`) and a list of the Reserved rows of the registry. It calls the
   setup helper function of task 001. A second dialog asks for `--force` when
   the helper reports that the instance exists. It names the instance and the
   event.
6. Protection (Decision 4, GUI). An action that task 001 refuses on the
   private instance is not available in the GUI. Its tooltip or its
   inspector text says why, with the name of the private instance.
7. State (Decision 8, render). The Live Metadata card and its inspector
   render the instance lines of task 002.
8. Change these documents in the same change: the AGENTS.md paragraph of
   ADR 0111, this Status line, the ADR 0111 row of `docs/adr/README.md` if
   its text changes, and a new section in `docs/pending-human-checks.md` with
   the visual criteria below.

Main session choices, 2026-11-21:

- The stored fact is one row in the database, in a new migration. It is not
  a file in the repository and not in the configuration file.
- The player setting is not saved in this task. It starts as HitIt at each
  start of HitIt, unless the stored fact is set. Then it starts as Mixxx with
  the stored show instance.
- The recording start time is in `StartedParts` and in the stored row. Item
  6 of the live rehearsal request reads it later.
- If a step needs a decision that this task file and ADR 0111 do not give,
  the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Go on air sequence | extend | src/view_models/show.rs | `OnAirSequence` and `OnAirStep` get the Mixxx steps. |
| End show sequence | extend | src/view_models/show.rs | `EndShowSequence` and `EndShowStep` get the Mixxx steps. |
| Started parts record | extend | src/view_models/show.rs | `StartedParts` gets the recording and the show instance. |
| Show commands | extend | src/app/show.rs | The sequences run the new steps. |
| Encoder control | reuse | src/broadcast/encoder.rs | `start_recording` and `stop_recording` exist and have no caller. |
| Producer switch | reuse | src/broadcast/mod.rs | Task 002 adds the module `producer_switch` with the switch. |
| Setup helper client | reuse | src/broadcast/mod.rs | Task 001 adds the module `setup_helper` with the helper call. |
| Database migrations | extend | src/db.rs | A new migration for the stored fact. |
| Show inspector | extend | src/ui/composites/show_detail_panel.rs | The Source and Live Metadata inspectors get the setting, the action and the instance lines. |

## Mechanical Criteria

1. ADR 0111 criterion 7: the "Go on air" model for Mixxx gives the steps of
   Decision 6 in order and stops at a failed step.
2. ADR 0111 criterion 8: the "End show" model for Mixxx gives only the steps
   for the parts that HitIt started, in the order of Decision 7, and
   continues after a failed step.
3. ADR 0111 criterion 11: an "End show" model that HitIt makes again from
   the stored state after a restart still gives `StartPrivateProducer`.
   After that step, the stored fact is clear.
4. A test: a failed switch with a start of the private producer clears the
   stored fact.
5. A view model test: with HitIt as player, the sequences are as before.
6. The commit checks of AGENTS.md pass.

## Visual Criteria

These go in a new section of [pending human checks](../pending-human-checks.md).
The agent walks each one on Xvfb first.

- V1 (ADR 0111 visual 1): the Live Metadata card shows the state of each
  instance and the instance that holds Mixxx.
- V2 (ADR 0111 visual 2): "Make show instance" shows its inputs, and its
  confirmation for `--force` names the instance and the event.
- V3 (ADR 0111 visual 3): the player setting shows HitIt and Mixxx, and
  "Go on air" shows the Mixxx steps when Mixxx is selected.
- V4 (ADR 0111 visual 4): a refused action on the private instance shows why
  it is refused.

## Report

Not started.
