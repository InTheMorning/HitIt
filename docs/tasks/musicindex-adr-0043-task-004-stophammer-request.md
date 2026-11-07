# musicindex ADR 0043 Task 004: The Stophammer Request

Status: Complete on 2026-11-06. Request 15 is in both copies. Stophammer owns the answer.

## Goal

The [open Stophammer requests](../plans/v4vmm-open-requests.md) ask for a sort direction, in both copies of that file.

## Request Text

MusicIndex lists need a sort direction. Reason: musicindex ADR 0043 R-LIST-7 reverses a whole list, and an app must not reverse only the loaded rows.

| Route | Needed |
|---|---|
| `/v1/feeds/recent` | The oldest item first, with a cursor in that direction |
| `/v1/feeds/by-name` | Z to A, with `from` and a cursor in that direction |
| `/v1/artist-names` | Z to A, with `from` and a cursor in that direction |
| `/v1/search` | The same two directions, if its results have a stable sequence |

Stophammer owns the name and the form of each parameter.

## Acceptance Criteria

Mechanical: request 15 has equal bytes in the two copies. The copies differ elsewhere, because draft request 14 is only in v4vmm.

Visual: none.

## Prompt for lower-context coding model

The planning model does this packet, because it writes to two repositories.

## Escalation Triggers

The other copy differs for a reason that this packet does not name.
