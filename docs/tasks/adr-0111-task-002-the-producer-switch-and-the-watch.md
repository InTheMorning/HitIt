# ADR 0111 Task 002: The Producer Switch And The Watch

## Status

Ready after task 001 - 2026-11-21. Implementation has not started.

## Goal

HitIt switches the one Mixxx producer between two instances in the order of
publisher ADR 0021. The service watch reads the units of each instance and
the instance that holds Mixxx.

## Authority

[ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md) Decisions
5 and 8. [ADR 0059](../adr/0059-broadcast-control-surface.md) gives the pace
rules of the watch.

## Change

1. Switch model (Decision 5). A pure model, `ProducerSwitch`, in a new module
   `src/broadcast/producer_switch.rs`. It takes instance A, instance B and
   the fact "the producer of A is active". It gives the steps in order: stop
   the producer of A, start the producer of B, read the state of B. On a
   failure of B, it gives the start of A when A was active. On exit status
   3, it gives the instance that holds Mixxx.
2. The instance that holds Mixxx. When the producer of B fails with exit
   status 3, HitIt reads it from `systemctl --user list-units
   'mixxx-now-playing@*'`. Read the property `ExecMainStatus` of the unit of
   B for the exit status.
3. A function runs the switch through `ServiceControl` and `Transport`. The
   function returns a typed result: the switch is done, or the switch failed
   with the error, the lock holder when known, and the result of the start of
   A.
4. Watch (Decision 8). `src/runtime/broadcast_service_watch.rs`: the
   snapshot has the publisher unit and the producer unit of the private
   instance and of each show instance of the host, and the instance that
   holds Mixxx. It reads them in one `systemctl` call when possible. It keeps
   the pace rules of ADR 0059. A unit that is not installed gives the state
   "not installed".
5. `src/view_models/show.rs`: the Live Metadata card data gets one line for
   each instance: the name, the publisher state, the producer state, and a
   mark on the instance that holds Mixxx. Task 003 renders it.
6. Change these documents in the same change: the AGENTS.md paragraph of
   ADR 0111, and this Status line.

Main session choices, 2026-11-21:

- `ProducerSwitch` has no I/O. Its tests need no `systemctl`.
- A unit with `LoadState=not-found` is "not installed". This is the state of
  `mixxx-now-playing@mixxx.service` before the operator runs
  `--migrate-units`.
- If a step needs a decision that this task file and ADR 0111 do not give,
  the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Service control | reuse | src/broadcast/control.rs | `ServiceControl::start`, `stop` and `show` run the units. |
| Service watch | extend | src/runtime/broadcast_service_watch.rs | The snapshot gets the units of each instance and the lock holder. |
| Live Metadata card model | extend | src/view_models/show.rs | The card data gets the line of each instance. |
| Producer switch model | new | src/broadcast/producer_switch.rs | Searched src/broadcast/, src/view_models/show.rs and src/app/show.rs. No code stops one unit and starts a different one. `OnAirSequence` orders the steps of one show and has no undo step. |

## Mechanical Criteria

1. ADR 0111 criterion 6: the switch model gives the steps of Decision 5 in
   order. On a failure of B with exit status 3, it gives the lock holder and
   the start of A. With A not active, it gives no start of A.
2. ADR 0111 criterion 9: the watch snapshot has the publisher unit and the
   producer unit of each instance, and the instance that holds Mixxx. A test
   with a fake `systemctl` output proves it, also for a unit that is not
   installed.
3. A view model test: the Live Metadata card data has one line for each
   instance, and the mark is on the lock holder only.
4. The commit checks of AGENTS.md pass.

## Visual Criteria

None. Task 003 has the visual criteria of ADR 0111.

## Report

Not started.
