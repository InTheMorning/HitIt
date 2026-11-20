# ADR 0107 Task 001: The Cards Hold The On Air Status

## Status

Not started - 2026-11-19.

## Goal

The on air band above the Show cards has only one row of buttons. The Source, Stream and Live Metadata cards and their inspectors show each status. "End show" stops only the parts that HitIt started or connected in this run of the app.

## Authority

[ADR 0107](../adr/0107-the-cards-hold-the-on-air-status.md) Decisions 1, 2 and 3. It changes [ADR 0105](../adr/0105-go-on-air-and-end-show.md) Decisions 1, 2, 4 and 5. ADR 0105 Decisions 3 and 6 stay. [ADR 0063](../adr/0063-show-dashboard-layout.md) gives the card states and the inspector (`ShowPanelMode::Detail`).

## Change

1. Band (Decision 1). `OnAirBandDisplay` (`src/view_models/show.rs`) and `render_on_air_band` (`src/ui/shells/show.rs`) lose the checklist rows, the signal line, the listener line, the warnings and the error lines. The band keeps one row with "Go on air" or "End show", and the end band of ADR 0105 Decision 6.
2. The switch "Show information" (`ShowInformationSwitch`) moves from `render_on_air_buttons` to the inspector of the Live Metadata card. Its behavior does not change.
3. Cards (Decision 2). Each readiness condition stays a fact that controls "Go on air". `sound_row`, `cue_row`, `stream_row` and `information_row` stop making band rows. Their facts go to the card and the inspector in the table of ADR 0107 Decision 2.
4. A card with a problem uses `ShowCardStateKind::Attention`. A card with a failed step uses `ShowCardStateKind::Failed`. The inspector shows the cause and the action of the table. No new marker.
5. The Stream card, while the stream is connected: the primary line is the signal state (`StreamSignalState::display`), and the secondary line is the listener count. No text says that listeners hear the show.
6. Remove the test sound warning (`sound_row`) and the listener address text (`stream_row`). The test sound result of ADR 0104 does not change.
7. While "Go on air" is not available, the band row shows a short cause adjacent to the button. The cause names the card, for example "Stream is not ready". The accessible label of the button includes the cause.
8. Started parts (Decision 3). A new pure model `StartedParts`, in `src/view_models/show.rs`, holds the parts that HitIt started in this run: the stream connection and each Live Metadata service role. `TopApp` keeps one value. It is not saved.
9. `src/app/show.rs`: a successful Connect of the stream, or a successful Start of a service, records the part. This applies to the steps of "Go on air" and to the controls of the cards. A Disconnect or a Stop removes the part.
10. `start_end_show_sequence` gives `EndShowSequence` only the recorded parts that are on. The `EndShowConfirmation` lists only those parts.
11. A stream that is connected but not recorded: the Stream card state label is "Connected outside HitIt". "End show" does not disconnect it. The Disconnect control of the Stream card stays (`stream_actions`, `render_stream_detail`).
12. Change these documents in the same change:
    - The ADR 0107 status line and its Tasks line.
    - The AGENTS.md paragraph of ADR 0107, and the ADR 0107 row of `docs/adr/README.md`.
    - `docs/pending-human-checks.md`: a new section 47 with the visual criteria below. In section 45, a line closes step 1 as replaced by section 47.

Main session choices, 2026-11-19:

- The music is always a part that HitIt started. The Show transport belongs to HitIt (ADR 0068), so no other program can start it. `StartedParts` does not hold the music. "End show" stops the music when the Show transport plays or is paused, as now.
- "On air" now means "one or more recorded parts are on, or the music plays". `stream_is_connected` stays for the stream facts, but it no longer decides "on air". "End show" is available, and the end band of ADR 0105 Decision 6 can start, only when on air in this meaning. Otherwise the band row shows "Go on air".
- A step of "Go on air" for a part that is already on does not run and does not record the part. Thus a stream connected outside HitIt stays outside after "Go on air".
- When the watch snapshot shows a recorded part as off, `StartedParts` removes it. Thus a stream that disconnected and later connected outside HitIt is not recorded.
- Silence at the stream gives `Attention` on the Stream card. "Sound at the stream is not known" gives the inspector text only, and the card state does not change.
- A failure of "Go on air" or "End show" goes to one card:
  - The Sound plays, Show cue and music steps go to Source.
  - The stream steps go to Stream.
  - The Show information and service steps go to Live Metadata.
- The inspector of that card shows the error and "Try again".
- `OnAirRow` can stay as the key of a failure, but no band row uses it. If a type has no user after the change, delete it.
- If a step needs a decision that this task file and ADR 0107 do not give, the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| On air band view model | extend | src/view_models/show.rs | `OnAirBandDisplay` and `ShowPageVm::with_on_air_band` lose the status rows and get the cause of the button. |
| Show card view model | extend | src/view_models/show.rs | `ShowCardDisplay`, `source_card_state`, `live_metadata_card_state` and `stream_card_state` get the readiness facts. |
| Show inspector | extend | src/ui/composites/show_detail_panel.rs | `render_source_detail`, `render_publisher_detail`, `render_event_detail` and `render_stream_detail` show the causes, the errors and the switch. |
| Show shell band | extend | src/ui/shells/show.rs | `render_on_air_band` and `render_on_air_buttons` keep only the button row and the end band. |
| End show sequence | reuse | src/view_models/show.rs | `EndShowSequence` takes the parts as now. Only its input changes. |
| Show commands | extend | src/app/show.rs | `run_stream_encoder_command`, `run_publisher_service_command` and `start_end_show_sequence` record and read the started parts. |
| Started parts record | new | src/view_models/show.rs | Searched src/view_models/show.rs, src/app/show.rs, src/runtime/broadcast_service_watch.rs and src/broadcast/encoder.rs. No type records which part HitIt started. The watch snapshot gives only the current state. |

## Mechanical Criteria

1. A view model test: `OnAirBandDisplay` has no checklist rows, no signal line, no listener line and no error lines. It has the button and the end band.
2. A view model test for each row of the ADR 0107 Decision 2 table: the fact gives the state and the inspector text on the correct card.
3. A view model test: a missing test sound result gives no warning and does not block "Go on air".
4. A view model test: no card and no inspector text contains the encoder `address`.
5. A view model test: while the stream is connected, the Stream card summary is the signal state and the listener count.
6. A view model test: while "Go on air" is not available, the band gives a cause that names the card.
7. A `StartedParts` test: Connect and Start record a part. Disconnect, Stop and a snapshot with the part off remove it.
8. A test: a stream that is connected and not recorded gives "Connected outside HitIt". "End show" is not available, the confirmation does not list the stream, and `EndShowSequence` has no disconnect step.
9. A test: with a recorded stream and a service that is on but not recorded, the end sequence disconnects the stream and does not stop the service.
10. Delete the tests that assert the replaced rules: `on_air_rows_come_in_order_with_state_text_and_action`, `on_air_missing_test_result_warns_and_does_not_block_go`, `on_air_stream_texts_follow_decision_four` and `on_air_band_has_no_stream_lines_while_not_on_air`. Change `on_air_show_information_row_blocks_go_only_when_on_and_not_ready`, `end_show_confirmation_lists_only_active_parts_and_keeps_both_notes` and `end_show_sequence_keeps_order_and_continues_after_failure` to the new rules.
11. The commit checks of AGENTS.md pass.

## Visual Criteria

These go in section 47 of [pending human checks](../pending-human-checks.md).

- V1: above the cards, the Show page shows only the button row, at 1440 px and at 390 px, in Light and Dark.
- V2: with no encoder configured, the Stream card shows Attention. Its inspector shows the cause and "Change". "Go on air" shows "Stream is not ready".
- V3: the switch "Show information" is in the inspector of the Live Metadata card.
- V4: with a stream from Mixxx and butt that HitIt did not connect, the Stream card shows "Connected outside HitIt", and "End show" is not available.
- V5: after "Go on air", "End show" stops the stream for a listener.

## Report

Not started.
