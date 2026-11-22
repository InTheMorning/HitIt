# ADR 0112: A Station Host With A Liquidsoap Producer

## Status

Accepted - 2026-11-21. The operator accepted it as written. The publisher
answered Open Questions 1 to 7 at publisher commit `8e139b0`. The operator
answered Open Questions 8 to 11.

- Operator choice, 2026-11-21, relayed by the orchestrator: each answer to
  Open Questions 8 to 11 is the recommendation of this record.

- Bar: crosses repositories. Publisher ADR 0022 owns the station units and
  ADR 0023 owns the DJ takeover, and HitIt controls those units. It is
  also an opposite choice by mistake. Without this record, an agent can make
  HitIt write the slot text or copy a token file to the radio host. An agent
  can also make HitIt attach an event to the station instance.

Uses:

- Publisher ADR 0022, ADR 0023 and ADR 0024, at publisher commits `6eb01dc`,
  `0c69ae6` and `f9d248f`, accepted by the operator on 2026-11-21. They own
  the producer, the units, the state file, the takeover and the payment rule.
  This record cites them and does not restate their rules.
- The research tutorial
  `research_notes/azuracast/musicindex-producer-tutorial.md`, research commit
  `233aacb`. It is advisory.
- [ADR 0113](0113-broadcast-control-surface.md),
  [ADR 0110](0110-a-reserved-event-enters-the-registry.md) and
  [ADR 0111](0111-show-instances-and-the-producer-switch.md).

Terms:

- **Station**: a radio station that Liquidsoap plays, with or without
  AzuraCast. Liquidsoap selects the tracks. This is an established term.
- **Radio host**: the server of the station. This is a term of this record.
- **Station instance**: the publisher instance of a station on the radio
  host. It has the units `musicindex-live-publisher@INSTANCE.service` and
  `liquidsoap-now-playing@INSTANCE.service`. This is a term of this record.

## Context

The orchestrator relayed these facts on 2026-11-21:

- The two station units are user units of one account with lingering on the
  radio host. Lingering is the systemd function that keeps the user services
  of an account in operation with no login. This is an established term.
- The producer reads `/var/lib/musicindex/azuracast/STATION/state.json`.
- Liquidsoap selects the tracks, not HitIt. The operator has not decided if
  HitIt can send tracks to Liquidsoap.
- A live DJ takes over the station event. The DJ starts the DJ producer, for
  example Mixxx, when the DJ audio is connected, and stops it at the end. The
  DJ publisher needs a copy of the token of the station event.

HitIt has these parts at this time:

- A broadcast host has a transport, local or SSH, and one `instance_name`,
  the private instance of ADR 0111.
- `UnitRef::producer` makes only `mixxx-now-playing@INSTANCE.service`
  (`src/broadcast/control.rs`).
- Attach passes a token file path to the publisher on the host of the
  instance (`src/broadcast/publisher_targets.rs`). The file must be on that
  host before the Attach. HitIt copies no token file.
- ADR 0113 §Remote Control Uses SSH needs no control API in the publisher.
  Publisher ADR 0022 adds no control API.

## Decision

### 1. A host has a producer kind

A `[[broadcast.hosts]]` entry gets the field `producer`, with the values
`mixxx` and `liquidsoap`. The default is `mixxx`, so each entry in a configuration
of today keeps its meaning.

- For `liquidsoap`, `instance_name` is the station instance.
- A `liquidsoap` host has no show instances. HitIt refuses the configuration
  if it has one.
- `UnitRef::producer` takes the kind. For `liquidsoap`, the unit is
  `liquidsoap-now-playing@INSTANCE.service`.
- Only the source adapter knows the kind (ADR 0113 §Sources Are Selectable).
- The station instance name is the AzuraCast short name with each `_`
  changed to `-` (publisher ADR 0022 §The Template Unit, commit `8e139b0`).
  HitIt applies the name rule of ADR 0111 Decision 1. The operator enters
  the instance name. HitIt does not change a short name.

### 2. HitIt does not set up the station

The publisher runbook owns the slot text, the bind mount, the packages and
the account with lingering. It also owns the target and the token file on
the radio host.
The publisher has no setup helper for a station. The runbook uses the target
`default`. HitIt runs no setup helper on a station host. HitIt copies no file
to the radio host.

### 3. The station instance is protected

HitIt applies ADR 0111 Decision 4 to the station instance: no "Make show
instance", no Attach, no Detach and no Replace of a target. A producer
switch does not include a station instance, because the switch is for Mixxx
producers only (ADR 0111 Decision 5).

### 4. HitIt shows the station units

The service watch reads the two station units over SSH, with the pace rules
of ADR 0113. The SSH destination is the account with lingering on the radio
host. HitIt runs `systemctl --user` as that account, not with `-M` from a
different account (publisher answer 4). The Show page shows one line for the station instance, as
ADR 0111 Decision 8 does for each instance. The station producer does not
change the Mixxx holder of ADR 0111.

Start, Stop and Reset of a station unit are available. Stop has a
confirmation. It names the station and says that the listeners pay nobody
until the next start (operator answer 8).

### 5. The station event is a reserved event

The station event is a reserved event of ADR 0110. A station plays all day,
and a created event stops at the idle time limit or at a relay restart. The
operator adds the row with a local copy of the token file. HitIt does not
read the token file on the radio host.

### 6. A DJ set is a show that Mixxx plays

A DJ set with HitIt is a show of ADR 0111 Decision 6. Its show instance on
the DJ computer attaches the station event. "Go on air" starts the DJ
producer through the producer switch. "End show" stops it. This is the DJ
side of publisher ADR 0023 §The DJ Side Stays Manual.

HitIt cannot see if the DJ audio is connected to the station. When the
selected event is the event on the station target, the confirmation of
"Go on air" tells the operator to connect the DJ audio first. HitIt reads the
station target with the target list of the radio host. The publisher has no command
that gives the lock state (publisher answer 5). The operator asked the
publisher for one as a future item (operator answer 11). Until it exists,
the confirmation text is the guard.

### 7. HitIt sends no tracks to Liquidsoap

This record gives HitIt no command to Liquidsoap (operator answer 9).

## Invariants

- HitIt writes no slot text, no unit file and no state file, and it copies
  no file to a radio host.
- HitIt does not attach, detach or set up a station instance.
- HitIt reads no `state.json`. The publisher owns its format.
- The source has no personal host name, account name, station name, path,
  event ID or token.

## Open Questions

The publisher session answered these questions on 2026-11-21, at publisher
commit `8e139b0`:

1. The instance name is the short name with each `_` changed to `-`. The
   rule of publisher ADR 0021 applies. For example, `my_station` gives
   `my-station`.
2. No setup helper exists. The configuration file is
   `~/.config/musicindex-live-publisher/INSTANCE/config.toml`. The runbook
   uses the target `default`.
3. The producer writes `%t/musicindex-live-publisher/%i/nowplaying/TARGET.json`.
4. An SSH login as the account with lingering gets `XDG_RUNTIME_DIR` from
   `pam_systemd`. This is an inference that the runbook test checks with
   `systemctl --user is-system-running`.
5. No command gives the producer state or the lock state.
6. The runbook puts the token file on the radio host with mode `0600`, and
   the operator runs `target add` there. The DJ computer needs its own copy.
7. The station producer takes no Mixxx lock. It shares only the producer
   lock with the other producers of its instance.

The operator answered these questions on 2026-11-21:

8. Stop of a station unit: available, with a confirmation that names the
   station and says that the listeners pay nobody until the next start.
9. HitIt sends tracks to Liquidsoap: not in this record.
10. The paired GUID rule of publisher ADR 0024: no HitIt change at this time. A
    different HitIt record follows only if the operator asks for it.
11. A lock state command in the publisher: yes, as a future item. The
    orchestrator tells the publisher. Until the command
    exists, the confirmation text of Decision 6 is the guard.

## Alternatives

- **Wait for a control API in the publisher.** ADR 0059 planned it, and
  ADR 0113 replaces that plan.
  Rejected: publisher ADR 0022 adds none, and HitIt needs only the unit state,
  the start and the stop.
- **HitIt sets up the station.** Rejected: the setup needs root on the radio
  host and changes AzuraCast. The publisher runbook owns it.
- **HitIt copies the token file to the radio host.** Rejected: a token file
  is irreplaceable (ADR 0113). A copy over SSH adds a new path for a secret.
- **A different list of stations in the configuration.** Rejected: a station
  host has a transport and an instance, as each host has. The producer kind
  is the only difference.

## Consequences

- The operator enters the radio host, the account and the station instance
  in the configuration. No default names them.
- A DJ set uses the show instances and the producer switch of ADR 0111 with
  no new switch.
- A DJ producer that starts before the DJ audio connects can put a DJ
  block back over the station dead block (publisher ADR 0023 §Risks). Until the
  lock state command of operator answer 11 exists, only the confirmation
  text guards it.

## Acceptance Criteria

### Mechanical

1. A configuration test: a host with no `producer` field has the kind
   `mixxx`. A host with `producer = "liquidsoap"` and a show instance is
   refused.
2. A unit test: `UnitRef::producer` gives
   `liquidsoap-now-playing@INSTANCE.service` for the kind `liquidsoap`.
3. A view model test: Make show instance, Attach and Detach are not
   available on a station instance. Stop of a station unit gives the
   confirmation with the station name.
4. A watch test with fixed `systemctl` text: the snapshot has the two station
   units, and the Mixxx holder does not change.
5. A view model test: the "Go on air" confirmation has the DJ audio text when
   the selected event is the event on the station target, and not in other
   conditions.

### Visual

1. The Show page shows the line of the station instance with the state of
   the two units.
2. The "Go on air" confirmation of a DJ set on the station event shows the
   DJ audio text.

## Tasks

Do these tasks in this order. No code starts until the operator says so.

1. [Task 001](../tasks/adr-0112-task-001-the-producer-kind-of-a-host.md):
   the producer kind of a host.
2. [Task 002](../tasks/adr-0112-task-002-the-station-units-on-the-show-page.md):
   the station units on the Show page.
3. [Task 003](../tasks/adr-0112-task-003-the-dj-audio-text.md): the DJ audio
   text.
