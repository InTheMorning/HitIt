# ADR 0105 Task 001: "Go On Air"

## Status

Implemented - 2026-11-19. The mechanical criteria pass. The visual criteria wait for a person, in section 45 of the pending human checks.

## Goal

The Show page has the on air band. It has a checklist in order and one "Go on air" with a confirmation. A sequence starts the parts in order, and the band shows the state of the stream.

## Authority

[ADR 0105](../adr/0105-go-on-air-and-end-show.md) Decisions 1 to 4. [ADR 0059](../adr/0059-broadcast-control-surface.md) §Invariants: encoder commands run only in the broadcast service layer.

## Change

1. `src/view_models/show.rs`: add an on air band view model. It gives the four rows of ADR 0105 Decision 2, each with a state, a text and one action. It reads the readiness of the cards. It does not calculate a second readiness.
2. The band view model gives the availability of "Go on air" and the text of the confirmation, with the title of the selected cue entry.
3. `src/app/show.rs`: add a sequence that runs the existing commands in the order of ADR 0105 Decision 3. Start each Live Metadata service that does not run. Attach the event when row 4 is on. Connect the stream and wait for the connection. Then play the selected entry. Each step waits for its result. A failure stops the sequence and puts the error on its row with "Try again".
   Main session choice, 2026-11-19, for the link between the sequence and the commands:

   - A pure state machine `OnAirSequence`, with no GPUI, holds the order. It takes the start facts: the Live Metadata services that do not run, and row 4 on or off. It gives the next step. It takes the result of each step and gives the next step, or a stop with the row and the error. Criteria 3 and 4 test it.
   - `TopApp` keeps one `Option` of the run: the state machine and the `ShowCommandId` of the step that runs. "Go on air" is not available while a run exists.
   - Each step uses the existing dispatch path: `run_publisher_service_command` for a service start, `dispatch_event_target_command` with `EventTargetOperation::Attach`, `run_stream_encoder_command` for connect, and the `StartShow` path of `src/app/playback_bar.rs` with the selected entry. To get the id of the step, a dispatch function can return its `ShowCommandId`. Do not add a second command path.
   - `finish_show_command`, `finish_event_target` and the finish of `run_playback_command` call one function, `advance_on_air`. When the finished id is the id of the step, that function gives the result to the state machine and dispatches the next step. A different id does nothing.
   - The connect step ends when the connect command returns, and then the encoder state shows a connection. The sequence reads that state from the broadcast readiness snapshot that the app already polls. When no connection shows in 15 s, the step fails with "The stream did not connect.".
   - The row of an error: a service start or an attach goes to row 4, Show information. A connect goes to row 3, Stream. A play goes to row 2, Show cue. A refusal of `refuse_show_play` goes to row 1, Sound plays.
   - "Try again" on a row starts the sequence again from the start facts of that time. A part that runs is not started again.
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

Changed files: `src/app.rs`, `src/app/show.rs`, `src/app/playback_bar.rs`, `src/ui/shells/show.rs`, `src/view_models/show.rs`, `src/view_models/queue_now_playing.rs` and this task file.

Findings of the main session review and their fixes:

1. Row 4 switch. `ShowInformationSwitch` (On by default) lives in `TopApp`. The band has a button that turns it on or off. With the switch on, row 4 is ready only for a selected Live event, and "Go on air" needs it. With the switch off, row 4 says "Off", the sequence does no attach, and the confirmation gives the off text.
2. Row actions. Each action is a button. A row shows its action only when it is not ready. Row 1 also shows "Play test sound" and a warning when the test sound is not confirmed. "Play test sound" opens Settings, Audio. "Open playlists" opens the Show tab with the cue panel. "Change" opens Settings, Diagnostics, Configuration. "Add" opens the Live Metadata detail of the Show page. "Try again" starts the sequence again. All go through `on_air_row_action`.
3. A step that cannot start. After each dispatch, `dispatch_next_on_air_step` checks that the step set a step id. If not, the step fails with `settings_status` or "This step could not start.".
4. Play step. Only `start_on_air_cue` sets `on_air_play_pending`. `run_playback_command` takes that flag, so a playback command of the host does not advance the run. A refusal of `refuse_show_play`, a missing playback owner and a missing player go to row 1. Other play errors go to row 2.
5. Row 2 reads all cue rows through `cue_len` and `cue_all_available` of `QueueNowPlayingPageVm`.
6. Tests. See the list below.

Mechanical criteria and their tests (all in `src/view_models/show.rs`):

- Criterion 1: `on_air_rows_come_in_order_with_state_text_and_action`.
- Criterion 2: `on_air_missing_test_result_warns_and_does_not_block_go` and `on_air_show_information_row_blocks_go_only_when_on_and_not_ready`.
- Criterion 3: `on_air_services_to_start_leave_out_a_running_service` and `on_air_sequence_runs_only_requested_parts_in_order`.
- Criterion 4: `on_air_sequence_stops_on_connect_failure_and_does_not_play` and `on_air_sequence_fail_names_the_row`.
- Criterion 5: `on_air_stream_texts_follow_decision_four`.
- Finding 5: `on_air_cue_row_counts_rows_that_a_text_filter_hides`.
- Criterion 6 and 7: the architecture tests and the commit checks in the check results below.

Check results: `cargo build` and `cargo test` passed (3343 unit tests, 417 architecture tests, one version test, 0 failed). `cargo clippy --all-targets -- -D warnings` passed. `cargo fmt -- --check` passed. The visual checks stay open in section 45 of [pending human checks](../pending-human-checks.md).

## Main Session Review - 2026-11-19

The first build passed the gate, but it had these defects against ADR 0105 or the main session choice of Change item 3. The Report gives the fix of each. The main session read the fixed diff and ran the gate.

1. Row 4 has no switch. ADR 0105 Decision 2 item 4 says that the host turns the row off. Add a host switch "Show information" with the states on and off, kept in `TopApp`, on by default. When it is on, the row is ready only when an event is selected and Live, and "Go on air" needs the row ready. When it is off, the row says "Off", the sequence does no attach, and the confirmation gives the off text.
2. The row actions are labels. Each row shows one action only when it is not ready (ADR 0105 Decision 2), and the action is a button that works. "Play test sound" and the row 1 warning open Settings, Audio. "Open playlists" opens the Show playlists list of ADR 0106 (task 002 item 3 delivers the full row action later; here open the Show tab with the cue panel). "Change" opens Settings, Diagnostics, Configuration. "Add" opens the Show event detail. "Try again" on the failed row starts the sequence again.
3. A step that cannot start makes the sequence wait for all time. This occurs when `run_publisher_service_command`, `dispatch_event_target_command`, `run_stream_encoder_command` or the play path returns before a dispatch. After each dispatch call, when no step id is set for a step that needs one, fail the step with `settings_status` or a fixed text.
4. The play step matches by step type, not by command. Give the play command an id in the same way, or keep one flag that only `start_on_air_cue` sets. A refusal of `refuse_show_play` and a missing player go to row 1, Sound plays, as the main session choice states. Other play errors go to row 2.
5. Row 2 counts the filtered rows. Use all the cue rows, as `QueueNowPlayingPageVm::cue_is_empty` does.
6. Tests. Criterion 2: with rows 1 to 3 ready and no test result, "Go on air" is available and row 1 gives a warning. With row 4 on and not ready, it is not available; with row 4 off, it is available. Criterion 3: the selection of the services to start is a pure function, and a test proves that a running service is not in the list. Criterion 5: a test for the three stream texts, and no band text contains "Listeners hear". Criterion 4: a test proves that the play step does not come after a connect failure.
7. The switch "Show information" stayed locked after a run with an error. The main session changed this. After an error, the host can change the switch and try again without the attach.

Open point: the app-level links (the dispatch of each step, the play flag and the row actions) have no unit test. Only the view model and the sequence model have tests. The visual walk of section 45 examines the links.
