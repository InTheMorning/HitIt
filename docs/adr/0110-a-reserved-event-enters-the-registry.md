# ADR 0110: A Reserved Event Enters The Registry

## Status

Accepted - 2026-11-21. The operator accepted it as written.

- Bar: crosses repositories. The relay command `tools/relay-event` makes the
  token file, and HitIt reads it. It is also an opposite choice by mistake.
  ADR 0059 lets `Forget` remove the token file of each row. Without this
  record, an agent can remove the token file of a reserved event.
- Changes: [ADR 0059](0059-broadcast-control-surface.md) §This App Owns The
  Event Registry. The registry also holds reserved events that the operator
  made. A `404` is not always a dead event. `Forget` keeps a token file that
  a target uses or that HitIt did not write.
- Changes: [ADR 0059](0059-broadcast-control-surface.md) §Attachment Names The
  Configured Target. Attach asks before it replaces a different event on the
  target.

Uses:

- Item 1 of the [live rehearsal goal request](../plans/live-rehearsal-goal-request.md),
  with the operator choice and the relay answer of 2026-11-21.
- Requests 1 to 4 of the
  [reserved event safety request](../plans/reserved-event-safety-request.md).

Terms:

- **Reserved event**: a relay event that survives a relay restart and the
  idle time limit (relay ADR 0001). Only the relay operator makes one. This
  is an established term of the relay.
- **Created event**: an event that HitIt made with `Create` (ADR 0059). It is
  not reserved. This is a term of this record.

## Context

The show of the rehearsal uses a reserved event. Only a reserved event can
send track images (publisher ADR 0008). HitIt can make only a created event.

Operator choice of 2026-11-21: the operator reserves each event with one relay
command, also the events of friends. HitIt does not reserve events and does
not hold the admin token. HitIt enters a reserved event from its event ID and
its token file.

The relay command is `tools/relay-event reserve SHOW_NAME TOKEN_FILE`. It
writes the event ID to stdout. It writes the broadcaster token to a new file
with mode `600`, as one line with a newline. It refuses a path that exists,
also a symlink. It makes a missing folder with mode `700`, and stops when the
folder exists with a different mode.

The registry of ADR 0059 has these defects for a reserved event:

1. Each `404` from the metadata route marks an event Dead. A reserved event
   has no snapshot after a relay restart until its next publish. The relay
   then gives `404 metadata_not_found`. Only `404 event_not_found` means that
   the event does not exist (relay `docs/runbooks/reserved-live-items.md`).
2. A Dead event shows Replace. A replacement is a new created event with a new
   ID, and listeners must tune again.
3. `Forget` removes the token file of the row
   (`src/broadcast/registry.rs:70`). A publisher target can use that file.
4. Attach is available when the configured target does not carry the selected
   event. The target can carry a different event. Attach then replaces it with
   no question (`src/view_models/show.rs:3140`).

## Decision

### 1. A row has a kind

Each registry row has the kind Created or Reserved. Each row that exists
before this record has the kind Created. `Create` and `Replace` make the kind
Created. Decision 2 makes the kind Reserved.

### 2. The operator adds a reserved event

HitIt adds a reserved event from three values: the event ID, the path of the
token file, and an optional label.

- The GUI has the action "Add reserved event" in the Event section of the Show
  page.
- The CLI has `hitit broadcast events add-reserved EVENT_ID --token-file PATH
  [--label TEXT] --json`.

Before HitIt stores the row, it does these checks in this order:

1. The event ID is not empty, and no row has it.
2. The path is absolute.
3. The path is a regular file, not a symlink.
4. On Unix, the file mode is `600`.
5. The file has one line that is not empty. A newline at the end is
   permitted.
6. HitIt checks the event at the relay with the rules of Decision 4.

When a check of steps 1 to 5 fails, HitIt stores no row and names the failed
check. When the relay answers that the event does not exist, HitIt stores no
row. When the relay does not answer, HitIt stores the row with the status
Unknown and shows the failure.

HitIt does not copy, move or change the token file. It stores the path that
the operator gave. It does not keep the token text after the check.

### 3. HitIt shows the relay command

The dialog of "Add reserved event" shows the relay command with a suggested
token path. HitIt makes the path at run time:
`TOKEN_FOLDER/reserved/SHOW_NAME.token`. TOKEN_FOLDER is the token folder of
HitIt (`tokens::default_token_directory`). SHOW_NAME is the text that the
operator types in the dialog.

HitIt does not make the folder `reserved`. The relay command makes it with
mode `700`. HitIt does not run the relay command.

### 4. Only `event_not_found` means Dead

This decision applies to each row, Created or Reserved.

The route is `GET /v1/liveitems/{event_id}/metadata`. The `404` body is one
JSON object with the key `error`, for example `{"error":"event_not_found"}`
(relay `src/lib.rs` and `tests/api.rs`, answer of 2026-11-21 through the
orchestrator). An older relay build can send a different body. A body with no
`error` key that HitIt can read is the row "with no code".

| Relay answer | Status |
| --- | --- |
| Metadata | Live |
| `404` with the code `metadata_not_found` | Live. The event exists and has no snapshot. |
| `404` with the code `event_not_found` | Dead |
| `404` with a different code, or with no code | The status does not change. The check shows a failure. |
| A transport failure | The status does not change. The check shows a failure. |

### 5. A reserved row has no Replace

A Dead reserved row shows Check and no Replace. Its text tells the operator to
reserve a new event with the relay command and to add it.

### 6. `Forget` keeps a token file that HitIt must not remove

`Forget` always removes the row. It removes the token file only when each of
these conditions is true:

1. The row has the kind Created.
2. HitIt read the target list of the configured host, and no target names the
   token file.

In each other case, `Forget` keeps the file and says why. A failed read of the
target list keeps the file.

### 7. Attach asks before it replaces a different event

When the configured target carries a different event, Attach shows the event
ID on the target and the selected event ID. It asks before it changes the
target.

The CLI command `hitit broadcast targets attach` refuses the same case. It
continues only with `--replace-event EVENT_ID`, where EVENT_ID is the event on
the target.

## Invariants

- HitIt does not reserve an event and does not hold the relay admin token.
- HitIt does not write, move or remove the token file of a reserved row.
- A `404` with the code `metadata_not_found` does not make a row Dead.
- No Replace is available for a reserved row.

## Alternatives

- **HitIt runs the relay command.** The operator rejected this on 2026-11-21.
  HitIt does not hold the admin token.
- **HitIt copies the token file to its own folder.** Then two copies of one
  token exist, and a removal of one copy does not remove the other. Rejected.
- **A Dead reserved row offers Replace with a created event.** A created event
  cannot send track images and does not survive a relay restart. Rejected.

## Consequences

- The table `broadcast_events` gets the column for the kind, with an upgrade.
- The protection of instance `mixxx` is not in this record. Item 2 of the
  live rehearsal goal request owns it, after the publisher ADR that changes
  publisher ADR 0016.

## Tasks

- [Task 001](../tasks/adr-0110-task-001-reserved-rows-in-the-registry.md):
  Decisions 1, 2, 4, 6 and the CLI part of Decision 7.
- [Task 002](../tasks/adr-0110-task-002-reserved-rows-on-the-show-page.md):
  the GUI part of Decisions 2, 3, 5 and 7.
