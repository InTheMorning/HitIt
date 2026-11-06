# ADR 0089 Task 016: The Now-Playing Panel

Status: Waits - 2026-11-05. The operator decides [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md) first (ADR 0089, Answer 2). Do not start this packet before that decision. Rewrite its design from that decision.

Depends on: ADR 0068, and [packets 013](adr-0089-task-013-narrow-tab-bar.md) and [014](adr-0089-task-014-row-rules.md).

## Goal

The right side holds the panel of musicindex ADR 0027 R-SHELL-1 item 5, only while something plays or a show is live (ADR 0089 Decision 7). The main action of an album or a track in the Library is Play (Decision 6). Play stays on the current page and opens the panel.

## Known Scope

- The panel holds the current track, the transport, a seek control and Up Next. The transport and Up Next move there from the Show page.
- The curator can expand and collapse the panel. In a narrow window, it is a bar above the tab bar.
- The row action Play needs the route of ADR 0068. The guard `music_flow_map_dead_artist_view_and_track_play_action_stay_deleted` keeps the old `EntityActionKind::Play` deleted, and the operator decision of 2026-10-20 owns that guard.

## Acceptance Criteria

Mechanical:

- The panel view model is absent while nothing plays and no show is live.
- The row view model exposes Play as the main action of an album or a track in the Library, with typed availability.

Visual:

- V1: The panel opens on Play, and the page does not change.
- V2: The panel is absent when nothing plays.

## Prompt for lower-context coding model

No prompt exists, because the design waits for ADR 0068. The orchestrator writes it after the operator decision.
