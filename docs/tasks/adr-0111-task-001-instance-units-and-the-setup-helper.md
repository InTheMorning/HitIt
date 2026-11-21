# ADR 0111 Task 001: Instance Units And The Setup Helper

## Status

Implemented - 2026-11-21. Mechanical criteria are Green. The main session reviewed the diff and ran the checks. This task has no visual criteria. The refusal of Attach and Detach moved to [task 003](adr-0111-task-003-a-show-that-mixxx-plays.md).

## Goal

Each Mixxx producer unit has an instance name. The configuration holds the
show instances. HitIt makes a show instance with the publisher setup helper.
HitIt refuses the setup helper on the private instance.

## Authority

[ADR 0111](../adr/0111-show-instances-and-the-producer-switch.md) Decisions
1 to 4. Publisher ADR 0021 owns the unit names, the instance name pattern and
the setup helper flags and output.

## Change

1. Unit names (Decision 1). `src/broadcast/control.rs`: a new constructor
   `UnitRef::producer(instance)` gives `mixxx-now-playing@INSTANCE.service`.
   `UnitRef::producer` and `UnitRef::publisher` refuse an instance name that
   does not match `^[a-z0-9][a-z0-9-]{0,31}$`. Use one function for the
   check. Do not add a regex dependency.
2. `src/app/show.rs`: remove `PRODUCER_UNIT`. `broadcast_service_unit` uses
   `UnitRef::producer(&host.instance_name)` for the producer role.
3. Configuration (Decision 2). `src/config.rs` `BroadcastHostConfig` gets
   `show_instances: Vec<ShowInstanceConfig>`, with `#[serde(default)]`.
   `ShowInstanceConfig` has `name`, `target` (default `default`),
   `config_file`, `publisher_unit` and `producer_unit`. A configuration with
   no show instances reads and writes as before.
4. Setup helper (Decision 3). A new module `src/broadcast/setup_helper.rs`:
   - A function that builds the arguments of Decision 3 in that order, with
     `--force` only when the caller gives the confirmation.
   - A function that runs the command through `CommandRunner` and the
     `Transport` of the host, as `publisher_targets` does.
   - A parser that reads the last line of stdout as the JSON object of
     publisher ADR 0021 and refuses an output with no JSON object on that
     line.
   - A function that adds the `ShowInstanceConfig` to the host only after the
     helper succeeds.
5. Protection (Decision 4). The setup helper function refuses the private
   instance (`host.instance_name`) with an error that names ADR 0111
   Decision 4. It runs no command. Operator choice of 2026-11-21 (option B):
   the refusal of Attach and Detach goes to task 003, with the selection of a
   show instance. Then no commit makes "Go on air" of a show that HitIt plays
   fail.
6. Change these documents in the same change: the AGENTS.md paragraph of
   ADR 0111, and this Status line.

Main session choices, 2026-11-21:

- The protection is in the broadcast layer, not in the view model, so the
  GUI and the CLI get the same refusal.
- The CLI gets no new command to make a show instance in this task. Task 003
  adds the GUI action.
- Use no personal value in a test or a default. Use `mixxx`, `default` and
  test names such as `show-1`.
- If a step needs a decision that this task file and ADR 0111 do not give,
  the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Unit reference | extend | src/broadcast/control.rs | `UnitRef` gets `producer` and the instance name check. |
| Command runner | reuse | src/broadcast/control.rs | `CommandRunner` and `ProcessCommandRunner` run the setup helper. |
| Command transport | reuse | src/broadcast/transport.rs | `Transport` runs the helper locally or over SSH. |
| Host configuration | extend | src/config.rs | `BroadcastHostConfig` gets `show_instances`. |
| Publisher targets | extend | src/broadcast/publisher_targets.rs | `attach_event` and `detach_target` refuse the private instance. |
| Setup helper client | new | src/broadcast/setup_helper.rs | Searched src/broadcast/, src/cli.rs and ~/build/musicindex-lib/src. No code runs `setup-mixxx-musicindex` or reads its JSON output. |

## Mechanical Criteria

1. ADR 0111 criterion 1: `UnitRef::producer("show-1")` gives
   `mixxx-now-playing@show-1.service`. It refuses `Show`, `-show`, an empty
   name, and a name of 33 characters. `UnitRef::publisher` refuses the same
   names.
2. ADR 0111 criterion 2: a host with show instances reads and writes. A host
   with no show instances reads as before.
3. ADR 0111 criterion 3: the arguments are in the order of Decision 3 with
   no `--force`. With the confirmation, they also have `--force`.
4. ADR 0111 criterion 4: the parser reads the last stdout line. It refuses an
   output with no JSON object on the last line.
5. ADR 0111 criterion 5, the setup helper part: the setup helper on the
   private instance gives an error. A recording `CommandRunner` proves that no
   command ran.
6. ADR 0111 criterion 10: an architecture test finds no constant with the
   text `mixxx-now-playing.service` in `src/`. Its failure message names
   ADR 0111 Decision 1 and `UnitRef::producer`.
7. The commit checks of AGENTS.md pass.

## Visual Criteria

None. Task 003 has the visual criteria of ADR 0111.

## Report

Each mechanical criterion and its test:

1. Criterion 1: `adr_0111_producer_unit_has_an_instance_and_the_name_is_checked`
   in `src/broadcast/control.rs`.
2. Criterion 2: `adr_0111_host_reads_and_writes_show_instances` in
   `src/config.rs`. The test `adr_0111_host_refuses_a_bad_show_instance` covers
   the host check of show instance names.
3. Criterion 3: `adr_0111_setup_arguments_follow_decision_3_and_force_needs_confirmation`
   in `src/broadcast/setup_helper.rs`.
4. Criterion 4: `adr_0111_parser_reads_the_last_line_and_refuses_other_output`
   in `src/broadcast/setup_helper.rs`.
5. Criterion 5: these tests use a recording command runner or refuse before a
   command can run.
   - `adr_0111_setup_helper_refuses_the_private_instance` in
     `src/broadcast/setup_helper.rs`.
   - Test `adr_0111_host_changes_only_after_the_helper_succeeds` proves that
     the host gets a show instance only after the helper succeeds.
6. Criterion 6: `adr_0111_no_producer_unit_name_without_an_instance` in
   `tests/architecture_tests.rs`.
7. Criterion 7: see the check results in the report of the main session.

Choices that this task file did not give:

- The registry Replace action makes a new created event and changes no
  publisher target. It has no instance, so it has no refusal. The orchestrator
  accepted this reading on 2026-11-21 (ADR 0111 Decision 4).
- `--force` is the last argument.
- `BroadcastHostConfig::validate` refuses a show instance with a bad name, with
  the name of the private instance, or with a name that occurs twice.
- `UnitRef::publisher` no longer trims the instance name. It refuses a name
  with spaces.
- Test literals of `mixxx-now-playing.service` in `src/` changed to
  `mixxx-now-playing@mixxx.service`.
