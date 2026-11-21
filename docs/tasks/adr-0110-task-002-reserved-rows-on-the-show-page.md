# ADR 0110 Task 002: Reserved Rows On The Show Page

## Status

Ready after task 001 - 2026-11-21. Implementation has not started.

## Goal

The Event section of the Show page lets the operator add a reserved event.
Its dialog shows the relay command with a suggested token path. A reserved
row has no Replace. Attach asks before it replaces a different event on the
target. Forget tells the operator when it keeps the token file.

## Authority

[ADR 0110](../adr/0110-a-reserved-event-enters-the-registry.md) Decisions 2,
3, 5 and 7, the GUI part. [ADR 0063](../adr/0063-show-dashboard-layout.md)
gives the inspector.

## Change

1. "Add reserved event" (Decision 2). The Event inspector
   (`render_event_detail`, `src/ui/composites/show_detail_panel.rs`) gets the
   action. It opens a dialog with three fields: the event ID, the token file
   path and an optional label. The dialog calls `add_reserved_event` of task
   001 and shows the failed check, or the stored row.
2. The relay command (Decision 3). The dialog has a field for the show name.
   It shows the command `tools/relay-event reserve SHOW_NAME TOKEN_PATH`,
   where TOKEN_PATH is `TOKEN_FOLDER/reserved/SHOW_NAME.token` from
   `tokens::default_token_directory`. A "Copy" control copies the command.
   When the operator types the show name, HitIt puts the suggested path in
   the token file field, while the operator has not changed that field.
3. No Replace (Decision 5). `src/view_models/show.rs`: a Dead Reserved row
   gives Check and no Replace. Its text tells the operator to reserve a new
   event with the relay command and to add it.
4. The row shows its kind, Created or Reserved, in the event list and in the
   inspector.
5. Attach (Decision 7). When the configured target carries a different
   event, Attach opens a confirmation that shows the event ID on the target
   and the selected event ID. Only "Replace event" in the confirmation
   continues. The CLI rule of task 001 does not change.
6. Forget (Decision 6). When Forget keeps the token file, the result line
   says that the file stays and why.
7. Change these documents in the same change: the AGENTS.md paragraph of
   ADR 0110, the ADR 0110 row of `docs/adr/README.md` if its text changes,
   this Status line, and a new section in `docs/pending-human-checks.md` with
   the visual criteria below.

Main session choices, 2026-11-21:

- Use the existing dialog and text field primitives of the Show page and of
  Settings. Do not make a new dialog component.
- The dialog uses no personal value as a default. The show name field is
  empty at the start.
- If a step needs a decision that this task file and ADR 0110 do not give,
  the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Event inspector | extend | src/ui/composites/show_detail_panel.rs | `render_event_detail` gets "Add reserved event" and the kind of the row. |
| Event actions view model | extend | src/view_models/show.rs | `EventActionAvailability` gives no Replace for a Reserved row. |
| Show commands | extend | src/app/show.rs | The event registry commands get Add reserved, the Attach confirmation and the Forget result. |
| Event registry | reuse | src/broadcast/registry.rs | Task 001 gives `add_reserved_event` and the Forget result. |
| Token folder | reuse | src/broadcast/tokens.rs | `default_token_directory` gives TOKEN_FOLDER. |

## Mechanical Criteria

1. A view model test: a Dead Reserved row gives Check, no Replace, and the
   text of Decision 5. A Dead Created row keeps Replace.
2. A view model test: the suggested path is
   `TOKEN_FOLDER/reserved/SHOW_NAME.token` for a test token folder, and the
   command text contains it.
3. A view model test: Attach with a different event on the target gives the
   confirmation with both IDs. Attach with no event or the same event gives no
   confirmation.
4. A view model test: the Forget result with a kept file gives the reason
   text.
5. The commit checks of AGENTS.md pass.

## Visual Criteria

These go in a new section of [pending human checks](../pending-human-checks.md).

- V1: the Event inspector shows "Add reserved event". The dialog shows the
  relay command with the suggested path, and "Copy" copies it.
- V2: a reserved row shows the kind Reserved. When it is Dead, it shows Check
  and no Replace.
- V3: Attach on a target with a different event asks first and shows both
  IDs.

## Report

Not started.
