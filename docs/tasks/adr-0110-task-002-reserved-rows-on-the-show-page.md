# ADR 0110 Task 002: Reserved Rows On The Show Page

## Status

Implemented - 2026-11-21. The mechanical criteria pass. The main session reviewed the diff and ran the checks. The agent walked V1 and V2 on Xvfb with a scratch configuration, and they passed for the walked steps. The "Copy" step of V1 and all of V3 are open for the operator in [pending human checks](../pending-human-checks.md) section 50.

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
6. Forget (Decision 6). Main session choice, 2026-11-21: the Show page gets
   no Forget action. ADR 0059 keeps UI Forget out of scope. The CLI of task 001
   meets this item.
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
4. Main session choice, 2026-11-21: the CLI test of task 001 for the Forget
   result with a kept file meets this criterion. The GUI has no Forget.
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

Files changed: `src/view_models/show.rs`, `src/view_models/show/event_report.rs`, `src/view_models/startup/capabilities.rs`, `src/app/show.rs`, `src/app.rs`, `src/ui/composites/show_detail_panel.rs`, `src/ui/composites/confirmation_dialog.rs`, `src/ui/composites/mod.rs`, the dialog literals in `src/ui/shells/`, `docs/pending-human-checks.md` and `AGENTS.md`.

Mechanical criteria:

1. `adr_0110_dead_reserved_row_gives_check_and_no_replace` and `adr_0110_replace_command_refuses_a_dead_reserved_row`.
2. `adr_0110_suggested_path_and_relay_command`.
3. `adr_0110_attach_asks_only_for_a_different_event_on_the_target`.
4. Met by the CLI test of task 001 (main session choice). The Show page has no Forget action.
5. The commit checks of AGENTS.md. See the report of the main session.
6. Go on air: `adr_0110_go_on_air_attach_step_refuses_a_different_event`.

Also tested:  `adr_0110_the_picker_and_the_selection_name_the_kind`, `adr_0110_add_reserved_outcome_lines`, `adr_0110_add_reserved_action_availability` and `adr_0110_add_reserved_command_stores_selects_and_keeps_the_token_file`.

Main session choices, 2026-11-21:

- Forget has no GUI action. The CLI of task 001 meets criterion 4.
- A show name has only letters, digits, `-`, `_` and `.`, and does not start with a dot. Otherwise no path and no command show. A path with special characters gets shell quotes in the command.
- The Go on air step "Attach" does not replace a different event (ADR 0110 Decision 7). When the configured target carries a different event, the step fails. The error names the event on the target and the selected event. It tells the operator to use Attach in the Live Metadata inspector. The failure goes to the Live Metadata card as other attach failures do. When `HitIt` cannot read the target list, the step runs as before.
- The recovery retry of Attach keeps its behavior. The operator already confirmed the first Attach in the dialog, and a refusal on retry would block the recovery.

Choices of the implementer:

- The confirmation dialog composite got an optional `form`. It has text fields, a text with a Copy button, a result line and a "keeps open" flag. No new dialog component exists.
- The command shows the suggested path. The token file field can hold a different path.
- "Add" selects the new row. The dialog stays open to show the result.
