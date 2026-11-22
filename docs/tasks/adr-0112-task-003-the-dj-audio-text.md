# ADR 0112 Task 003: The DJ Audio Text

## Status

Implemented - 2026-11-21. Mechanical criteria are Green. The main session
reviewed the diff and ran the checks. The agent walked visual criterion 1 on
Xvfb. The result is in the Report.

## Goal

A DJ set is a show that Mixxx plays. When the selected event is the event on
a station target, the confirmation of "Go on air" tells the operator to
connect the DJ audio first.

## Authority

[ADR 0112](../adr/0112-a-station-host-with-a-liquidsoap-producer.md)
Decisions 5 and 6. [ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md)
Decision 6. [ADR 0110](../adr/0110-a-reserved-event-enters-the-registry.md)
for the reserved event.

## Change

1. The station events (Decision 6). HitIt reads the target list of the
   station instance of each `liquidsoap` host with
   `publisher_targets::list_targets` through the `Transport` of that host. It
   reads the list in a runtime actor or a command, never in a screen. It reads
   when the Show page opens and when the selected event changes. A failed
   read gives no station events and a log entry. It does not block
   "Go on air".
2. The fact. The app keeps the event IDs of the station targets as a fact of
   the Show page. `OnAirFacts` gets the fact that the selected event is on a
   station target.
3. The text (Decision 6). `mixxx_go_confirmation` in
   `src/view_models/show.rs`: when the fact is true, the confirmation starts
   with a sentence that tells the operator to connect the DJ audio to the
   station first. Example: "Connect the DJ audio to the station first. The
   station event plays the DJ only after the audio is connected." The text
   has no station name from the configuration in the source code.
4. No other change to "Go on air". The steps of ADR 0111 Decision 6 do not
   change.

If a step needs a decision that this task file and ADR 0112 do not give, the
implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Publisher target list | reuse | src/broadcast/publisher_targets.rs | `list_targets` reads the targets of an instance. |
| Command transport | reuse | src/broadcast/transport.rs | `Transport` reads the list over SSH. |
| On air facts | extend | src/view_models/show.rs | `OnAirFacts` gets the station event fact. |
| Go on air confirmation | extend | src/view_models/show.rs | `mixxx_go_confirmation` adds the DJ audio text. |

## Mechanical Criteria

1. ADR 0112 criterion 5: the "Go on air" confirmation has the DJ audio text
   when the selected event is the event on a station target. It has no DJ
   audio text in these conditions: the selected event is on no station
   target, no event is selected, or the read of the station targets failed.
2. A test with a fixture transport proves that the read runs `list_targets`
   for the station instance. It also proves that the read does not attach,
   detach or replace a target.
3. The commit checks of AGENTS.md pass.

## Visual Criteria

1. ADR 0112 visual criterion 2: the "Go on air" confirmation of a DJ set on
   the station event shows the DJ audio text. The agent walks this check on
   Xvfb with fixture data. No agent check uses a real radio host or starts a
   real unit.

## Report

1. Criterion 1:
   `adr_0112_go_confirmation_has_the_dj_audio_text_only_for_a_station_event`
   in `src/view_models/show.rs`, and
   `adr_0112_selected_event_fact_needs_a_station_target_and_a_good_read` in
   `src/app/show.rs`.
2. Criterion 2: `adr_0112_station_read_runs_list_targets_for_the_station_instance_only`
   in `src/app/show.rs`.
3. Criterion 3: the commit checks pass. Lib tests: 3466. Architecture tests:
   418.

The read is the command `ReadStationEvents` in `src/app/show.rs`. It runs at
app start, when the selection of the Show page changes, and when the operator
selects an event. The fact is `ShowPlayerFacts.selected_event_on_station`.

Visual criterion 1: the agent walked it on Xvfb. The configuration had a DJ
host and a station host with SSH destinations on the reserved domain
`.invalid`. A stub `ssh` answered only `target list` for `my-station`, and
refused each other command. The relay endpoint was a closed local port. The
agent added the reserved event `event-station` with a fake token file,
selected the player Mixxx and the show instance `show-1`, and clicked
"Go on air". The confirmation started with "Connect the DJ audio to the
station first." The agent clicked Cancel.

Then the stub gave a different event ID for the station target. After a new
selection, the confirmation had no DJ audio text. The stub received only
`systemctl --user show`, `systemctl --user list-units` and `target list`.

Open items:

- The read does not run when the operator only opens the Show tab. A change
  of a station target on the radio host shows after the next selection or
  start.
- A failed read writes one line to standard error. The project has no log
  crate.
