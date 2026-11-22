# ADR 0112 Task 001: The Producer Kind Of A Host

## Status

Ready - 2026-11-21. No code starts until the operator says so.

## Goal

A broadcast host has a producer kind, `mixxx` or `liquidsoap`. The unit name
of the producer comes from the kind. A `liquidsoap` host has no show
instances, and HitIt makes no show instance on it.

## Authority

[ADR 0112](../adr/0112-a-station-host-with-a-liquidsoap-producer.md)
Decisions 1 to 3. [ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md)
Decisions 1 and 4. Publisher ADR 0022 §The Template Unit owns the unit name
`liquidsoap-now-playing@INSTANCE.service`.

## Change

1. Configuration (Decision 1). `src/config.rs`: a new enum `ProducerKind`
   with `Mixxx` and `Liquidsoap`, serialized as `mixxx` and `liquidsoap`.
   `BroadcastHostConfig` gets `producer: ProducerKind` with
   `#[serde(default)]`. The default is `Mixxx`. Do not write the field when
   it is `Mixxx`, so a configuration of today writes as before.
2. The host check (Decision 1). A `liquidsoap` host with one or more show
   instances is refused. Put the check where the host check of ADR 0111 task
   001 refuses a bad show instance name. The message names ADR 0112
   Decision 1.
3. Unit names (Decision 1). `src/broadcast/control.rs`:
   `UnitRef::producer(kind, instance)` gives
   `mixxx-now-playing@INSTANCE.service` for `Mixxx` and
   `liquidsoap-now-playing@INSTANCE.service` for `Liquidsoap`. It keeps the
   instance name check. Change each caller. A caller that has no host passes
   `ProducerKind::Mixxx` only where the unit is a Mixxx unit by its
   function, for example the producer switch.
4. Only the source adapter knows the kind (ADR 0113 §Sources Are
   Selectable). Shared display and control logic does not compare the kind
   with a text literal. Use the enum.
5. No show instance on a station host (Decision 3).
   `src/broadcast/setup_helper.rs` and "Make show instance"
   (`src/app/show_instance.rs`) refuse a `liquidsoap` host before any command
   runs. The message names ADR 0112 Decision 3. The view model gives no
   "Make show instance" action for a `liquidsoap` host.
6. The producer switch (`src/broadcast/producer_switch.rs`) does not include
   a `liquidsoap` host (ADR 0111 Decision 5).

If a step needs a decision that this task file and ADR 0112 do not give, the
implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Host configuration | extend | src/config.rs | `BroadcastHostConfig` gets `producer`. |
| Unit reference | extend | src/broadcast/control.rs | `UnitRef::producer` takes the kind. |
| Setup helper client | extend | src/broadcast/setup_helper.rs | It refuses a `liquidsoap` host. |
| Show instance command | extend | src/app/show_instance.rs | "Make show instance" refuses a `liquidsoap` host. |
| Producer switch | reuse | src/broadcast/producer_switch.rs | The switch stays for Mixxx producers only. |

## Mechanical Criteria

1. ADR 0112 criterion 1: a host with no `producer` field has the kind
   `Mixxx` and writes with no `producer` field. A host with
   `producer = "liquidsoap"` reads and writes. A `liquidsoap` host with a show
   instance is refused.
2. ADR 0112 criterion 2: `UnitRef::producer(ProducerKind::Liquidsoap, "my-station")`
   gives `liquidsoap-now-playing@my-station.service`.
   `UnitRef::producer(ProducerKind::Mixxx, "show-1")` gives
   `mixxx-now-playing@show-1.service`. The instance name check applies to the
   two kinds.
3. ADR 0112 criterion 3, the first part: the setup helper on a `liquidsoap`
   host gives an error, and a recording `CommandRunner` proves that no command
   ran. The view model gives no "Make show instance" action for a
   `liquidsoap` host.
4. The architecture test of ADR 0111 criterion 10 stays Green.
5. The commit checks of AGENTS.md pass.

## Visual Criteria

None. Task 002 has visual criterion 1 of ADR 0112.

## Report

Give each mechanical criterion and its test. Do not commit. The main session
reviews the diff, runs the checks and commits.
