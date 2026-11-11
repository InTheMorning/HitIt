# ADR 0089 Task 016: The Now-Playing Panel

Status: Waits - 2026-11-11. The operator accepted [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md) on 2026-11-11. This packet waits for the ADR 0068 packets that supply the cue and the two players. Rewrite its design from ADR 0068 and its revision.

Depends on: ADR 0068, and [packets 013](adr-0089-task-013-narrow-tab-bar.md) and [014](adr-0089-task-014-row-rules.md).

## Goal

The right side holds the panel of musicindex ADR 0027 R-SHELL-1 item 5, only while something plays or a show is live (ADR 0089 Decision 7). The main action of an album or a track in the Library is Play (Decision 6). Play stays on the current page and opens the panel.

## Known Scope

- The panel holds the current track, the transport, a seek control and Up Next. The transport and Up Next move there from the Show page.
- The curator can expand and collapse the panel. In a narrow window, it is a bar above the tab bar.
- The panel has a Show part and an Audition part (ADR 0068 Answer 1). On 2026-11-11 the orchestrator session found that musicindex ADR 0027 R-SHELL-1 items 5 and 6 permit this.
- R-SHELL-1 item 5 shows the panel "only while something plays". ADR 0089 Decision 7 adds "or a show is live". This packet reads a live show as a case of play, as the orchestrator session read it on 2026-11-11.
- This packet decides what the narrow bar shows when the two players play. ADR 0068 Answer 1 does not decide it.
- The row action Play needs the route of ADR 0068. The guard `music_flow_map_dead_artist_view_and_track_play_action_stay_deleted` keeps the old `EntityActionKind::Play` deleted, and the operator decision of 2026-10-20 owns that guard.

## Acceptance Criteria

Mechanical:

- The panel view model is absent while nothing plays and no show is live.
- The row view model exposes Play as the main action of an album or a track in the Library, with typed availability.

Visual:

- V1: The panel opens on Play, and the page does not change.
- V2: The panel is absent when nothing plays.

## Prompt for lower-context coding model

No prompt exists. The orchestrator writes it from ADR 0068 after the ADR 0068 packets.
