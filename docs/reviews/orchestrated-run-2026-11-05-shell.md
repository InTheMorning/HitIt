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
