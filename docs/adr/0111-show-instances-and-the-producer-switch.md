# ADR 0111: Show Instances And The Producer Switch

## Status

Accepted - 2026-11-21. The operator accepted it with one change: HitIt
stores on disk that the switch of a show stopped the private producer, so
"End show" starts it again also after a restart (Decision 7).

- Revised: 2026-11-21. Operator choice: a show that HitIt plays also uses a
  selected show instance (Decision 6). Decision 4 states what Replace is.

- Bar: crosses repositories. HitIt runs the units and the setup helper of
  publisher ADR 0021. It also crosses modules: the configuration, the unit
  control, the service watch and the Show page change together.
- Changes: [ADR 0105](0105-go-on-air-and-end-show.md) Decisions 3 and 5.
  A show that Mixxx plays has its own "Go on air" and "End show" steps.
- Changes: [ADR 0107](0107-the-cards-hold-the-on-air-status.md) Decision 3.
  "End show" starts the producer of the private stream again when the switch
  of this show stopped it.

Uses:

- Items 2 and 5 of the [live rehearsal goal request](../plans/live-rehearsal-goal-request.md),
  with the operator choices of 2026-11-21.
- Publisher ADR 0021, at publisher commit `4a38e09`, accepted by the operator
  on 2026-11-21. It owns the unit names, the Mixxx lock and the setup helper
  flags. This record cites it and does not restate its rules.
- [ADR 0110](0110-a-reserved-event-enters-the-registry.md) for the reserved
  event of a show.

Terms:

- **Private instance**: the publisher instance of the private stream of the
  operator. It is the `instance_name` of the configured host, default
  `mixxx`. This is a term of this record.
- **Show instance**: a publisher instance that HitIt made for shows. This is
  a term of this record.
- **Producer switch**: the stop of one Mixxx producer and the start of a
  different one, in the order of publisher ADR 0021 §The Switch. This is a
  term of this record.

## Context

Operator choices of 2026-11-21:

- HitIt gets full control of the publisher units and the producer units.
  HitIt makes an instance, starts it, stops it, switches between the private
  stream and a show, and shows the state.
- The publisher package owns the content of each unit file. HitIt writes no
  unit file. HitIt runs only `systemctl --user` and the setup helper.
- Only one Mixxx producer runs at a time. During a show, the private event
  gets no track data.
- The private instance is protected.
- The rehearsal uses Mixxx (item 4 of the request).
- HitIt commits no personal setting. The operator enters each host, path,
  event and token file in the configuration or in the GUI.

HitIt now has one producer unit name, `PRODUCER_UNIT`
(`src/app/show.rs:55`), with no instance. The configured host has one
`instance_name`. ADR 0105 "Go on air" plays the HitIt Show cue. It has no
step for Mixxx.

## Decision

### 1. Each unit has an instance

`src/broadcast/control.rs` makes the producer unit from an instance name, as
`UnitRef::publisher` makes the publisher unit. The unit name is
`mixxx-now-playing@INSTANCE.service` (publisher ADR 0021 §The Unit Names).

HitIt refuses an instance name that does not match
`^[a-z0-9][a-z0-9-]{0,31}$`. `PRODUCER_UNIT` goes.

### 2. The configuration holds the show instances

The configured host keeps `instance_name` as the private instance. The host
also has a list of show instances. Each entry has a name and a target name.
The target name has the default `default`.

HitIt writes a show instance entry only after the setup helper of Decision 3
succeeds.

### 3. HitIt makes a show instance with the setup helper

The action "Make show instance" has these inputs: the instance name, the
target name, and one reserved event of the registry (ADR 0110). HitIt runs:

```text
setup-mixxx-musicindex --instance NAME --target TARGET --endpoint ENDPOINT
    --event-id EVENT_ID --token-file TOKEN_PATH --display --no-start --json
```

- ENDPOINT is the relay endpoint of the registry row.
- EVENT_ID and TOKEN_PATH come from the registry row.
- `--display` lets the show send track images (publisher ADR 0008).
- HitIt reads the last line of stdout as the JSON object of publisher ADR 0021
  §The Setup Helper. It stores `instance`, `target`, `config_file`,
  `publisher_unit` and `producer_unit`.
- HitIt sends `--force` only after a confirmation. The confirmation names the
  instance and says that its configuration changes to the selected event.

### 4. The private instance is protected

HitIt refuses these actions on the private instance, in the GUI and in the
CLI:

- Make show instance, and each run of the setup helper.
- Attach, Detach and Replace. Replace is Attach with a replacement of the
  event on a target. The Replace of the event registry makes a new event,
  changes no target and is not refused.

HitIt does not stop or start the publisher unit of the private instance. HitIt
stops and starts the producer unit of the private instance only in a producer
switch.

### 5. The producer switch

A switch from instance A to instance B has these steps:

1. HitIt records whether the producer of A was active.
2. HitIt stops the producer of A.
3. HitIt starts the producer of B. The start also starts the publisher of B
   (publisher ADR 0021).
4. HitIt reads the state of the producer of B.

When the producer of B fails with exit status 3, a different producer holds
the Mixxx lock. HitIt shows "INSTANCE holds Mixxx", with the instance name
from the error. When the producer of B fails, HitIt starts the producer of A
again if step 1 recorded it as active. It then shows the error.

### 6. "Go on air" for a show that Mixxx plays

The Show page has a player setting: HitIt or Mixxx. The default is HitIt.

Operator choice of 2026-11-21: no show, with either player, changes the
private instance. The Show page has a selected show instance for both
players. Attach and Detach use the selected show instance. With HitIt, ADR
0105 Decision 3 applies, and its step Attach attaches the event to the target
of the selected show instance. When no show instance is selected, "Go on air"
is not available and the Show page tells the operator to make or select a
show instance.

With Mixxx, "Go on air" has these steps, in this order:

1. A producer switch from the private instance to the selected show instance.
2. A check of the event of the show instance (ADR 0110 Decision 4).
3. HitIt connects the stream and waits until the encoder reports the
   connection.
4. HitIt starts the recording of the encoder and stores the wall clock time
   of the start.

HitIt does not attach the event. The setup helper wrote it into the instance
configuration. HitIt does not play a track. The operator plays the tracks in
Mixxx. Each step waits for its result. When a step fails, the sequence stops,
as in ADR 0105 Decision 3.

The stored start time is for the episode export (item 6 of the request).

### 7. "End show" for a show that Mixxx plays

With Mixxx, "End show" has these steps, in this order:

1. HitIt stops the recording of the encoder, when HitIt started it.
2. HitIt disconnects the stream, when HitIt connected it (ADR 0107 Decision
   3).
3. HitIt stops the producer of the show instance.
4. HitIt stops the publisher of the show instance. The publisher then stops
   the keepalive of the show event.
5. HitIt starts the producer of the private instance, when the switch of step
   1 of Decision 6 stopped it.

When a step fails, HitIt does the next steps, as in ADR 0105 Decision 5.

HitIt stores the fact "the switch of this show stopped the private producer"
in its database, not in a repository file. HitIt stores it before step 2 of
Decision 5 stops the private producer. HitIt clears it when the switch fails
and the private producer starts again, and after step 5 of this decision.
Step 5 reads the stored fact. Thus "End show" starts the private producer
again also after a restart or a crash of HitIt during the show.

### 8. The Show page shows the state of each instance

The Live Metadata card shows, for the private instance and each show
instance:

- The state of the publisher unit.
- The state of the producer unit.
- The instance that holds Mixxx: the producer unit that runs, from
  `systemctl --user list-units 'mixxx-now-playing@*'`.

`src/runtime/broadcast_service_watch.rs` reads these units. It keeps the pace
rules of ADR 0059.

## Invariants

- HitIt writes no unit file.
- HitIt does not run the setup helper, Attach, Detach or Replace on the
  private instance.
- HitIt does not stop the publisher unit of the private instance.
- The source has no personal host name, user path, event ID, token or token
  file path.

## Alternatives

- **Publisher option A: a second producer unit with `Conflicts=`.** The
  operator rejected it on 2026-11-21. systemd cannot name each instance of a
  template in `Conflicts=` (publisher ADR 0021).
- **HitIt writes the unit files.** The operator rejected it on 2026-11-21.
- **"End show" leaves the private producer stopped.** Then the private
  stream stays without tracks after each show until the operator starts it.
  Rejected.
- **A show uses the private instance with a second target.** One producer
  writes one target, so the private event and the show event cannot get
  different tracks. Rejected.

## Consequences

- The installed package must have publisher ADR 0021, and the private stream
  must have done `setup-mixxx-musicindex --migrate-units`. Before that,
  `mixxx-now-playing@mixxx.service` does not exist. The Show page then shows
  the producer unit as not installed.
- The episode export of item 6 gets the wall clock start of the recording.

## Acceptance Criteria

### Mechanical

Each criterion is proved by a test.

1. `UnitRef` gives `mixxx-now-playing@show-1.service` for the instance
   `show-1`. It refuses `Show`, `-show`, an empty name, and a name of 33
   characters.
2. The configuration reads and writes a host with show instances. A host with
   no show instances reads as before.
3. The setup helper command for a show instance has the arguments of Decision
   3, in that order, with no `--force`. With the confirmation, it also has
   `--force`.
4. The parser of the setup helper output reads the last stdout line, and
   refuses an output with no JSON object on the last line.
5. Each refused action of Decision 4 on the private instance returns an error
   and runs no command. A test with a recording command runner proves that no
   command ran.
6. The switch model gives the steps of Decision 5 in order. On a failure of
   B with exit status 3, it gives the lock holder and the start of A.
7. The "Go on air" model for Mixxx gives the steps of Decision 6 in order, and
   stops at a failed step.
8. The "End show" model for Mixxx gives only the steps for the parts that
   HitIt started, in the order of Decision 7, and continues after a failed
   step.
9. The service watch snapshot has the publisher unit and the producer unit of
   each instance, and the instance that holds Mixxx.
10. A source check finds no `PRODUCER_UNIT` constant with a fixed unit name.
11. An "End show" model that HitIt makes again from the stored state after a
    restart still gives step 5 of Decision 7. After step 5, the stored fact
    is clear.
12. With the player HitIt and a selected show instance, the step Attach of
    "Go on air" attaches to the target of the selected show instance. With no
    selected show instance, "Go on air" is not available.

### Visual

A person must judge each criterion. An agent walk on Xvfb comes first.

1. The Live Metadata card shows the state of each instance and the instance
   that holds Mixxx.
2. "Make show instance" shows its inputs, and its confirmation for `--force`
   names the instance and the event.
3. The player setting shows HitIt and Mixxx, and "Go on air" shows the Mixxx
   steps when Mixxx is selected.
4. A refused action on the private instance shows why it is refused.

## Tasks

- [Task 001](../tasks/adr-0111-task-001-instance-units-and-the-setup-helper.md):
  Decisions 1 to 4, mechanical criteria 1 to 5 and 10. The refusal of
  Attach and Detach goes to task 003, so that it lands with the selection of
  a show instance.
- [Task 002](../tasks/adr-0111-task-002-the-producer-switch-and-the-watch.md):
  Decisions 5 and 8, mechanical criteria 6 and 9.
- [Task 003](../tasks/adr-0111-task-003-a-show-that-mixxx-plays.md):
  Decisions 6 and 7, the stored fact, the GUI of Decisions 3 and 4,
  the selected show instance for both players, the refusal of Attach and
  Detach on the private instance, mechanical criteria 5 (Attach and Detach),
  7, 8, 11 and 12, and the visual criteria.
