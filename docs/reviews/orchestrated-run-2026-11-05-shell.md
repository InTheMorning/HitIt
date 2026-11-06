# Orchestrated Run Of 2026-11-05: The Shell

## Status

Open - 2026-11-05. The run implements the [ADR 0089 phase plan](../plans/adr-0089-shell-phase-plan.md), packets 001 to 015. Packet 016 waits for the operator decision on ADR 0068.

This record is advisory. It states no rule. Each rule stays in its ADR or its packet.

## Operator Decisions - 2026-11-05

- The operator asked for the phase plan and the packets, then for the orchestration, with a commit between packets.
- The run starts although sections 8 and 9 of [pending human checks](../pending-human-checks.md) are open. The orchestrator reads the request as this decision. The two sections stay open.

## Orchestrator Decisions

Each entry names the step, the decision and the reason.

- Plan: the orchestrator commits the uncommitted ADR 0089 documents with the plan, as one commit, because each packet builds on them.
- Plan: "the show playlist" is the source playlist of the active playback session. The app has no other value with this meaning. The Show cuelist is projected from that playlist.
- Plan: the row action Play moves from packet 014 to packet 016. Music has no Play action, and ADR 0068 owns Music playback.
- Plan: a removed release needs no state. A release that is not in the Library shows only in All.
- Plan: the failed state of a release is session state, from the outcomes of the download queue.
- Plan: open points of ADR 0089 keep their present behavior, as the phase plan lists.
- Plan: the toolbar and the title row of a list page each show the title, because R-TOOLBAR-1 and R-LIST-1 each ask for it. The final walk asks the operator about this.
- Packet 001, a page with no content: a Library page with no loaded data and no load shows its list root, and a load shows the skeleton. The stack entry stays. A reload with `ResetDetail` thus shows the list root under a page entry, and Back then stays on that list. The final walk checks this.
- Packet 001, focus: the visible frame takes the keyboard focus when it changes. The toolbar search field keeps the focus for a list root, because a live run changes the list root while the curator types. A page takes the focus from the field.
- Packet 001, Discover: "New releases" opens as a page, and Back shows the Discover views list. Packet 006 removes this result.
- Packet 002, Back at the root of a path: `go_back` with no earlier entry moves to the root of the path. This holds R-NAV-2 item 3. Back is thus enabled on each page over a root.
- Packet 002, scroll: the list scroll stays in long-lived view state (`ListState` or `ScrollHandle`), not in the navigation entry. The packet added handles for My playlists and the Discover views. The mechanical criterion "restored scroll" thus has no model test, and the final walk checks it.
- Packet 002, marks: a Library track, a name match and an artist by name in search results get no mark, because the row does not hold the data of its page entry. Needs attention rows get no mark, because packet 012 replaces them.
- Packet 002, Show and Settings: their frames lost Back and Forward, and the toolbar shows each control only when its history exists.
- Packet 003, frames: `SourceList` goes. `Detail` stays, because a removal changes `MusicColumns` and the shell widths. A later packet removes it when it is small.
- Packet 003, icons: Show uses Lucide `RadioTower`, and Settings uses Lucide `Settings`.
- Packet 003, keyboard: the Show and Settings rows take a click and the section shortcuts, as the old tabs did. A sidebar row has no Enter key action.
- Packet 004, My playlists: the page has no sort, and its rows stay in name sequence. Decision 5 says that the curator sets the sequence. A sequence needs stored positions and a move control, and ADR 0089 puts the moves of My playlists in Non-Goals. The final walk asks the operator which holds.
- Packet 004, view icons: no page has two views yet, so the icons are absent until packet 005.
- Packet 004, the In Library count: it counts each release that the scope holds, so a singles group of five counts five.
- Packet 005: the orchestrator divided it into Parts A to D, because the first agent stopped on its size before any edit.
- Packet 005, Artists: Artists has one sort, A to Z. The packet states the evidence.
- Packet 005 Part A: a first letter outside ASCII counts as "#". A later rule for sort keys can change this.
