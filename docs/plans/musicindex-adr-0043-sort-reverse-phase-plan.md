# musicindex ADR 0043 Sort Reverse Phase Plan

Status: Open. Created 2026-11-06. Packets 001 to 004 are complete or implemented. Packet 005 waits for the Stophammer answer.

Owner of the rules: musicindex ADR 0043 R-LIST-6 and R-LIST-7. This plan states no rule.

## Goal

Each v4vmm list with a sort has the approved control. A list that can reverse all its rows has the reverse icon.

## Non-Goals

- Reversing only the loaded rows (R-LIST-7 item 9).
- A change to the view icons outside the sort control.

## Current State

| List | Scope | Source of the rows | Reverse icon |
|---|---|---|---|
| Releases | In Library | Local database | Packet 003 |
| Artists | In Library | Local database | Packet 003 |
| My playlists | None | Local database | Packet 003 |
| Results | In Library | Local database | Packet 003, if each row is local |
| Releases | All | `/v1/feeds/recent`, `/v1/feeds/by-name` | Absent until packet 005 |
| Artists | All | `/v1/artist-names` | Absent until packet 005 |
| Public playlists | None | `/v1/feeds/recent`, `/v1/feeds/by-name` | Absent until packet 005 |
| Results | All | `/v1/search` and pages of local rows | Absent until packet 005 |
| Live | None | No sort | No control |

The MusicIndex API 0.18.1 has no sort direction on these routes, and its cursors go forward.

## Sequence

| Packet | Content | Worker | Waits for |
|---|---|---|---|
| [001](../tasks/musicindex-adr-0043-task-001-icons-and-menu.md) | The two icons and the menu entry color | Planning model | Nothing |
| [002](../tasks/musicindex-adr-0043-task-002-list-state-and-names.md) | The reversed state and the reversed names in the view model | Coding model | 001 |
| [003](../tasks/musicindex-adr-0043-task-003-reverse-control.md) | The control and the local lists | Coding model | 002 |
| [004](../tasks/musicindex-adr-0043-task-004-stophammer-request.md) | The Stophammer request | Planning model | Complete: request 15 |
| 005 | The Index lists | Written after the Stophammer answer | 004 |

## Schema And API Implications

No schema change. Packet 004 asks for an API change.

## Risk Areas

- Most people start on Releases, All. The icon is absent there until packet 005.
- The menu entry color is in the shared `ContextMenu` primitive, so each menu in the app changes.
- A second agent edits this tree. The search view models are in its current changes.

## Test Strategy

Mechanical: the criteria of each packet and the full gate. Visual: a new section of [pending human checks](../pending-human-checks.md), written with packet 003.

## Rollback Strategy

Revert the commit of each packet. The stored data does not change.

## Open Questions

None for packets 001 to 003. Stophammer owns the answer to packet 004.
