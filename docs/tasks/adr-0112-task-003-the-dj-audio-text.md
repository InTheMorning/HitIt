# ADR 0112 Task 003: The DJ Audio Text

## Status

Ready - 2026-11-21. It needs task 001. No code starts until the operator says
so.

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

Give each mechanical criterion and its test, and the result of the visual
walk. Do not commit. The main session reviews the diff, runs the checks and
commits.
