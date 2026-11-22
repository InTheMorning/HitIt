# ADR 0112 Task 002: The Station Units On The Show Page

## Status

Ready - 2026-11-21. It needs task 001. No code starts until the operator says
so.

- Operator choice, open: the line and the controls of a station show when
  the station host is the selected host (`broadcast.selected_host`). This
  agrees with ADR 0113 §Lists From The Start, which shows one selection at a
  time. The other option is a station line on the Show page for each
  `liquidsoap` host at all times. The main session chose the first option
  because ADR 0112 does not decide it. The operator can change it before
  the implementation starts.

## Goal

When the selected host is a station host, the service watch reads the two
station units. The Show page shows one line for the station instance. Start,
Stop and Reset act on the station units. Stop asks first.

## Authority

[ADR 0112](../adr/0112-a-station-host-with-a-liquidsoap-producer.md)
Decisions 3 and 4. [ADR 0113](../adr/0113-broadcast-control-surface.md)
§Remote Control Uses SSH and §The Service Watch Reads At The Pace Of Its
Viewer. [ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md)
Decision 8.

## Change

1. The watch (Decision 4). `src/app/show.rs` `broadcast_service_units` and
   `service_instance`: for a `liquidsoap` host, the instance is
   `instance_name`, and the producer unit comes from
   `UnitRef::producer(ProducerKind::Liquidsoap, instance)`. The watch reads
   the two units with one `systemctl show` call, as for each instance. The
   pace rules do not change.
2. The Mixxx holder (Decision 4). The station producer does not change
   `mixxx_holder`. The holder read stays the read of `mixxx-now-playing@*`.
3. The line (Decision 4). `PublisherSectionDisplay::instances` has one
   `ShowInstanceLine` for the station instance. Its text names the instance
   and the state of the two units. It has no Mixxx mark.
4. The commands (Decision 4). `refuse_private_service_instance`: for a
   `liquidsoap` host, Start, Stop and Reset act on `instance_name`. ADR 0111
   Decision 4 applies to a Mixxx host only. Attach, Detach and Replace of a
   target stay refused on a station instance (Decision 3).
5. The Stop confirmation (Decision 4, operator answer 8). Stop of a station
   unit opens a confirmation before the command runs. The text names the
   station instance and says that the listeners pay nobody until the next
   start. Use the confirmation component that "End show" uses. Start and
   Reset have no confirmation.
6. SSH (Decision 4). HitIt runs `systemctl --user` through the `Transport`
   of the host, as the account in the SSH destination. HitIt adds no `-M`.

If a step needs a decision that this task file and ADR 0112 do not give, the
implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Service watch | reuse | src/runtime/broadcast_service_watch.rs | It reads the units that the app gives it. |
| Watch unit list | extend | src/app/show.rs | `broadcast_service_units` takes the producer kind. |
| Service command | extend | src/app/show.rs | `refuse_private_service_instance` permits the station instance. |
| Instance line | reuse | src/view_models/show.rs | `ShowInstanceLine` shows the station instance. |
| Confirmation | reuse | src/view_models/show.rs | `EndShowConfirmation` and its view are the pattern for the Stop confirmation. |
| Command transport | reuse | src/broadcast/transport.rs | `Transport` runs `systemctl --user` over SSH. |

## Mechanical Criteria

1. ADR 0112 criterion 3, the second part: Stop of a station unit gives the
   confirmation with the station instance name. A recording `CommandRunner`
   proves that no command runs before the confirmation. Start and Reset run
   with no confirmation. Attach and Detach are not available on a station
   instance.
2. ADR 0112 criterion 4: a watch test with fixed `systemctl` text gives a
   snapshot with the units `musicindex-live-publisher@my-station.service` and
   `liquidsoap-now-playing@my-station.service`. The Mixxx holder does not
   change.
3. The view model has one instance line for the station instance with no
   Mixxx mark.
4. The SSH command of a station unit has `systemctl --user` and no `-M`.
5. The commit checks of AGENTS.md pass.

## Visual Criteria

1. ADR 0112 visual criterion 1: with a station host selected, the Show page
   shows the line of the station instance with the state of the two units.
   The agent walks this check on Xvfb with a configuration that has a
   station host on a fixture transport. No agent check uses a real radio
   host.

## Report

Give each mechanical criterion and its test, and the result of the visual
walk. Do not commit. The main session reviews the diff, runs the checks and
commits.
