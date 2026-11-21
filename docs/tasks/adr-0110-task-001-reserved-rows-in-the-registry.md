# ADR 0110 Task 001: Reserved Rows In The Registry

## Status

Implemented - 2026-11-21. Mechanical criteria are Green. The main session reviewed the diff and ran the checks. This task has no visual criteria.

## Goal

The event registry holds reserved events that the operator made with the
relay command. A `404` with the code `metadata_not_found` does not make a row
Dead. `Forget` keeps a token file that HitIt must not remove. The CLI command
`attach` refuses to replace a different event without `--replace-event`.

## Authority

[ADR 0110](../adr/0110-a-reserved-event-enters-the-registry.md) Decisions 1,
2, 4, 6 and the CLI part of Decision 7. It changes
[ADR 0059](../adr/0059-broadcast-control-surface.md).

## Change

1. Kind (Decision 1). `src/db.rs`: a new migration after the last migration
   adds the column `kind` to `broadcast_events`, `TEXT NOT NULL DEFAULT
   'created'`. The values are `created` and `reserved`. `BroadcastEventRow`
   and `BroadcastEventInput` get a field `kind: BroadcastEventKind`.
   `create_event` and the Replace path write Created.
2. Add (Decision 2). `src/broadcast/registry.rs`: a new method
   `add_reserved_event(event_id, token_path, label)`. It does the checks of
   Decision 2 in the given order and names the failed check in the error. It
   reads the token file only to check it. It does not keep the token text, and
   it does not copy, move or change the file. It stores the endpoint of the
   registry. When the relay does not answer, it stores the row with the status
   Unknown and returns the failure with the row.
3. The CLI: `hitit broadcast events add-reserved EVENT_ID --token-file PATH
   [--label TEXT] --json` in `src/cli.rs`, in the same form as
   `broadcast events create`.
4. The relay answer (Decision 4). `src/api` `fetch_live_metadata_optional`
   returns a type that tells `metadata_not_found`, `event_not_found` and a
   `404` with a different code or with no readable code. The body is one
   JSON object with the key `error` (ADR 0110 Decision 4). `check_event`
   applies the table of Decision 4. Each other caller of
   `fetch_live_metadata_optional` keeps its present behavior for a `404`.
   Find each caller with `grep`.
5. Forget (Decision 6). `forget_local_event` always removes the row. It
   removes the token file only for a Created row when the target list of the
   configured host was read and no target has a `token_file` equal to the row
   path. A failed read keeps the file. The result says if the file was kept
   and why. The CLI JSON output of `forget` gets the field
   `token_file_kept` and a reason. The GUI caller in `src/app/show.rs` gives
   the target list or the read failure. Tests use a fake target list. Make
   the target list an argument, so that the registry does not run SSH.
6. Attach (CLI part of Decision 7). `hitit broadcast targets attach` reads the
   configured target first. When the target carries a different event, the
   command stops with an error that names both event IDs. It continues only
   with `--replace-event EVENT_ID`, and EVENT_ID must be the event on the
   target.
7. Change these documents in the same change: the AGENTS.md paragraph of
   ADR 0110, and this Status line.

Main session choices, 2026-11-21:

- The registry check that a file is a regular file uses `symlink_metadata`.
  The mode check runs only on Unix (`std::os::unix::fs::PermissionsExt`).
- If a step needs a decision that this task file and ADR 0110 do not give,
  the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Event registry | extend | src/broadcast/registry.rs | `BroadcastRegistry` gets `add_reserved_event`. `forget_local_event` and `check_event` change. |
| Event table | extend | src/db.rs | `broadcast_events` gets the column `kind` in a new migration. |
| Relay client | extend | src/api.rs | `fetch_live_metadata_optional` tells the `404` codes apart. |
| Publisher targets | reuse | src/broadcast/publisher_targets.rs | `list_targets` gives the target list for Forget and for Attach. |
| Token files | reuse | src/broadcast/tokens.rs | The token helpers stay. HitIt writes no token file for a reserved row. |
| CLI parser | extend | src/cli.rs | New subcommand `add-reserved` and the option `--replace-event`. |

## Mechanical Criteria

1. A database test: a row from before the migration has the kind Created. A
   new reserved row has the kind Reserved.
2. A registry test for each check of Decision 2: the failed check gives an
   error that names it, and no row is stored. The cases are: an empty ID, a
   duplicate ID, a relative path, a symlink, a folder, mode `644`, an empty
   file, a file with two lines.
3. A registry test: a valid file stores a Reserved row, and the token file
   has the same bytes and mode after the add.
4. A relay test for each row of the Decision 4 table, with a local test
   server: `metadata_not_found` gives Live, `event_not_found` gives Dead, a
   different code and a body with no `error` key leave the status and give a
   failure.
5. A Forget test: a Reserved row keeps its file. A Created row that a target
   names keeps its file. A Created row with a failed target read keeps its
   file. A Created row that no target names loses its file.
6. A CLI test: `attach` refuses a target with a different event, names both
   IDs, and continues with the correct `--replace-event`. A wrong
   `--replace-event` is refused.
7. The commit checks of AGENTS.md pass.

## Visual Criteria

None. Task 002 has the visual criteria of ADR 0110.

## Report

The registry has the kind column (migration 31), `add_reserved_event`, the
`404` code table and the new `forget_local_event`. The CLI has `add-reserved`,
`forget [--json]` and `attach --replace-event`. The GUI has no `Forget` caller,
so `src/app/show.rs` changed only for the new `kind` field.

| Criterion | Test |
|---|---|
| 1 | `db::tests::adr_0110_broadcast_event_kind_defaults_to_created_and_stores_reserved` |
| 2 | `registry::tests::adr_0110_add_reserved_event_names_the_failed_check_and_stores_no_row` |
| 3 | `registry::tests::adr_0110_add_reserved_event_stores_a_reserved_row_and_keeps_the_file` |
| 4 | `registry::tests::adr_0110_check_event_tells_the_404_codes_apart` and `registry::tests::adr_0110_add_reserved_event_handles_the_relay_answers` |
| 5 | `registry::tests::adr_0110_forget_keeps_a_token_file_that_hitit_must_not_remove` |
| 6 | `cli::tests::adr_0110_attach_refuses_a_different_event_without_the_matching_replace_option` |

Open points: `--replace-event` is also refused when the target carries no
different event. `fetch_live_metadata_optional` keeps its signature. The new
method `fetch_live_metadata_classified` gives the code, so the other callers
keep their behavior.
