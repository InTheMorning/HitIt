# ADR 0090 List Bar Phase Plan

## Status

Active - 2026-11-06. This plan is advisory. It states no rule. [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) owns the rules.

## Goal

Each list of selectable rows has one list bar directly above its rows. The bar stays in view while the rows scroll.

## Non-Goals

- A change to the selection model of ADR 0084 Decision 10.
- A change to the sort, view or reverse rules of musicindex ADR 0043.

## Sequence

| Packet | Content | State |
|---|---|---|
| [001](../tasks/adr-0090-task-001-group-page-list-bar.md) | The singles group page and the window group page | Complete |
| [002](../tasks/adr-0090-task-002-list-bar-stays-in-view.md) | The bar stays in view during a scroll (Decision 5), on the pages of packet 001 | Implemented |
| 003 | The release page | Not started |
| 004 | The feed page and the name level page | Not started |
| 005 | The playlist page | Not started |
| 006 | The search results page and the saved search page | Not started |
| 007 | Releases and the Inbox. First check musicindex ADR 0027 R-LIST-1 item 4. A change to that rule needs a record in `../musicindex` | Not started |

Each packet removes the band or the title row button, and the bottom bar, of its pages. The last packet deletes the old bar paths and the guards that state them.

## Risks

- GPUI has no sticky style. Packet 002 proves or rejects the pinned copy before the other pages move.
- The bar styles repeat part of the bottom bar until packet 007 deletes it.
