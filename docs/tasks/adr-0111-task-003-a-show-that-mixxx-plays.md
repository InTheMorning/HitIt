# ADR 0111 Task 003: A Show That Mixxx Plays

## Status

Implemented - 2026-11-21. The mechanical criteria pass. The main session reviewed the diff and ran the checks. The agent walked steps 1, 2, 4, 6, 7, 9 and 11 of section 51 of the pending human checks on Xvfb. The mark of step 1 and steps 3, 5, 8 and 10 are open for the operator.

## Goal

The Show page has a player setting, HitIt or Mixxx. With Mixxx, "Go on air"
switches the producer, checks the event, connects the stream and starts the
recording. "End show" stops each part that HitIt started and starts the
private producer again, also after a restart of HitIt. The Show page has the
action "Make show instance" and shows the state of each instance.

## Authority

[ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md) Decisions
3 (GUI), 4 (Attach, Detach and the GUI), 6, 7 and 8 (render), and the stored fact of Decision 7. It
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
6. Selected show instance (Decision 6, operator choice of 2026-11-21). The
   Show page has a selected show instance for both players. Attach and
   Detach, in the inspector and in the step Attach of "Go on air" with
   HitIt, use the target of the selected show instance. With no selected show
   instance, "Go on air" is not available, and the Show page tells the
   operator to make or select a show instance. The CLI command `attach` gets
   an option for the show instance and has no default to the private
   instance.
7. Protection (Decision 4). `attach_event` and `detach_target` of
   `src/broadcast/publisher_targets.rs` refuse the private instance
   (`host.instance_name`) with an error that names ADR 0111 Decision 4. They
   run no command. `EventTargetCommand` in `src/app/show.rs` and the CLI
   command `attach` give the same error. The registry Replace makes a new
   event, changes no target and is not refused. In the GUI, a refused action
   on the private instance is not available. Its tooltip or its inspector
   text says why, with the name of the private instance. These parts land in
   the same commit as item 6.
8. State (Decision 8, render). The Live Metadata card and its inspector
   render the instance lines of task 002.
9. The drop folder and the units of a show that HitIt plays (operator
   choice of 2026-11-21, option A, a small point in ADR 0111 Decision 6):
   - While a show instance is selected, the HitIt drop file goes into the
     drop folder of that instance: `$XDG_RUNTIME_DIR/musicindex-live-publisher/INSTANCE/nowplaying`,
     with the file name `TARGET.json`. Publisher facts, verified by the
     orchestrator in the publisher source: the template unit gives
     `--watch-dir %t/musicindex-live-publisher/%i/nowplaying`, and the setup
     helper JSON does not give the folder. HitIt makes the path from the
     environment variable and the instance name. No `/run/user` path goes
     into code, tests or configuration.
   - With no selected show instance, the drop file goes to
     `broadcast.drop_directory` as before. A show needs a selected show
     instance, so no show writes to the private folder.
   - The folder is local. When the configured host uses SSH, "Go on air"
     with HitIt as player is not available, and the Show page says why.
   - The step StartService of "Go on air", and Start, Stop and Reset of the
     Live Metadata inspector, act on the units of the selected show
     instance. On the private instance they are refused with an error that
     names ADR 0111 Decision 4. Only the producer switch starts and stops
     the private producer.
10. Change these documents in the same change: the AGENTS.md paragraph of
   ADR 0111, this Status line, the ADR 0111 row of `docs/adr/README.md` if
   its text changes, and a new section in `docs/pending-human-checks.md` with
   the visual criteria below.

Main session choices, 2026-11-21:

- The stored fact is one row in the database, in a new migration. It is not
  a file in the repository and not in the configuration file.
- The player setting is not saved in this task. It starts as HitIt at each
  start of HitIt, unless the stored fact is set. Then it starts as Mixxx with
  the stored show instance.
- The confirmation of "Go on air" with Mixxx and the Mixxx line of the
  Source inspector say that the show sends track data only from the next
  track that starts in Mixxx. A track that started before the switch gets no
  link (orchestrator answer of 2026-11-21, publisher behavior).
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
| Publisher targets | extend | src/broadcast/publisher_targets.rs | `attach_event` and `detach_target` refuse the private instance. |
| CLI parser | extend | src/cli.rs | `attach` takes the show instance and refuses the private instance. |
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
   The step Attach uses the selected show instance.
   The drop folder and the service units are those of the selected show
   instance.
6. ADR 0111 criterion 12: with HitIt and a selected show instance, Attach
   goes to the target of the show instance. With no selected show instance,
   "Go on air" is not available.
7. ADR 0111 criterion 5, the Attach and Detach part: Attach, Detach and
   Attach with a replacement on the private instance give an error that
   names ADR 0111 Decision 4. A recording `CommandRunner` proves that no
   command ran. The app command and the CLI give the same error.
8. The commit checks of AGENTS.md pass.

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
- V5: with no selected show instance, "Go on air" is not available and the
  Show page tells the operator to make or select a show instance.

## Report

Implemented on 2026-11-21. Test names:

1. Criterion 1: `adr_0111_go_on_air_for_mixxx_gives_the_steps_in_order_and_stops_at_a_failure` (`src/view_models/show.rs`).
2. Criterion 2: `adr_0111_end_show_for_mixxx_gives_only_started_parts_and_continues_after_a_failure` (`src/view_models/show.rs`).
3. Criterion 3: the tests of `src/app/show_mixxx.rs` for `started_parts_from_store` and the clear after `StartPrivateProducer`, and the criterion 2 test.
4. Criterion 4: `adr_0111_failed_switch_with_a_restart_clears_the_fact` (`src/app/show_mixxx.rs`).
5. Criterion 5: `adr_0111_hitit_sequence_is_unchanged_and_go_needs_a_selected_show_instance` (`src/view_models/show.rs`).
6. Criterion 6: `adr_0111_attach_uses_the_target_of_the_selected_show_instance` (`src/app/show.rs`) and the test of criterion 5.
7. Criterion 7: `adr_0111_attach_and_detach_refuse_the_private_instance` (`src/broadcast/publisher_targets.rs`), `adr_0111_event_target_command_refuses_the_private_instance` (`src/app/show.rs`) and `adr_0111_attach_needs_a_show_instance_and_refuses_the_private_one` (`src/cli.rs`).
8. Criterion 8: see the check results of the hand-back.

Item 9 (the drop folder and the units), tests in `src/app/show.rs` and `src/view_models/show.rs`:

- `adr_0111_show_drop_directory_is_below_the_runtime_folder`: the path function with a temp runtime folder.
- `adr_0111_producer_folder_follows_the_selection`: no selection keeps `broadcast.drop_directory`. A show instance gives its folder and target. No runtime folder gives an error.
- `adr_0111_ssh_host_has_no_drop_folder`.
- `adr_0111_drop_folder_problem_blocks_go_on_air_with_hitit_only` (view model): an SSH host makes "Go on air" with HitIt unavailable and the Source inspector says why.
- `adr_0111_service_commands_target_the_show_instance_and_refuse_the_private_one`: Start, Stop and Reset and the watch use the show instance units. The private instance and no selection are refused when the command is made, so no command object exists to run. The service commands call `control::start`, `control::stop` and `control::reset` directly, with no runner to record.
- `adr_0111_service_controls_are_refused_on_the_private_instance` (view model): the controls are unavailable with the reason.

Choices that this task file did not give:

- The drop file name keeps the producer suffix, so the file is `TARGET.nowplaying.json`.
- The role units follow the selected show instance for the watch, the logs and the commands. With none selected, the watch and the logs read the private units, and the commands are refused.
- A missing `XDG_RUNTIME_DIR` or an SSH host leaves the producer off. Go on air with HitIt is not available and the Source inspector tells why.
- The capability recovery of the producer rebuilds the producer from the configuration file, with no selection. A recovery of Producer while a show instance is selected resets the folder to `broadcast.drop_directory`. This is an open point.
- No `.producer.lock` is added.

- The private instance is selectable in the picker and has a mark. Then the refusal shows (V4).
- No show instance is selected at start, except from the stored fact and after "Make show instance".
- The CheckEvent step runs the registry Check on the selected event and needs the state Live.
- With Mixxx, "Go on air" needs a selected show instance, a ready stream and a selected event. It needs no mpv and no cue.
- "End show" never stops a unit of the private instance.
- `CLI attach` has the required option `--instance`.
- The dialog "Make show instance" lists the Reserved rows as check boxes.

Open points:

- The HitIt steps StartService and the Live Metadata controls Start, Stop and Reset still act on the units of the private instance. The task says that the HitIt sequence does not change.
- The helper has no typed error for an existing instance. The app finds it from `show_instances` or from the text of the helper error.
- A reload of the configuration drops the selected show instance.
