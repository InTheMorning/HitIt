# ADR 0106: The Cue Changes During A Show

## Status

Accepted - 2026-11-19. The orchestrator session relayed the operator choice of 2026-11-19: the operator selected each proposal of the show host analysis. This record decides the cue part of row 3 and the direction "From a playlist to the Show" (T8), which has no mockup.

- Bar: opposite choice by mistake. The database refuses each cue change while the Show plays or is paused (`src/db/show_cue.rs:280`). An agent that reads only ADR 0068 keeps that lock for each change.
- Changes: [ADR 0068](0068-show-cue-and-audition-isolation.md) §The Show Cue Exists Before Playback. The host can move a cue entry that did not play, also while the Show plays or is paused. After a load, the app opens Show.

Uses:

- The analysis `research_notes/ui-design/journeys/hitit/analysis.md` in `~/build/musicindex-research` (commit `2eb54c7`, corrected in `bc8a4a7`), themes T4 and T8.
- The mockup `mockups/3b-show-proposed.html` of that analysis. Its step counts are estimates.

Terms:

- The **current entry** is the cue entry of the Show session while the Show plays or is paused. This is a term of this record.
- An entry **after the current entry** has a larger position than the current entry. This is a term of this record.

## Context

- `show_cue_can_change` allows a change only when the session is stopped (`src/db/show_cue.rs:270`). Selection is also locked (`src/db/show_cue.rs:254`, `src/app/queue_now_playing.rs:45`).
- At EOF, the owner reads the next entry from the stored cue, by `ORDER BY e.position` (`src/db/show_cue.rs:164` and `:240`, `src/playback_owner.rs:510`).
- mpv loads one file at a time (`src/playback_driver/mpv.rs:258`). The owner loads the next file only after EOF. Thus a move before EOF changes the next track and does not stop the current track.
- "Load into Show cue" stays in Music and only refreshes Show (`src/app.rs:439`). The empty Show page says "No active show" and "Show playback is idle." (`src/view_models/show.rs:1864`).
- My playlists moves a row with Alt+Up and Alt+Down (ADR 0092). `src/db/playlist_order.rs:133` moves a playlist, and `src/ui/composites/track_drag.rs` gives the drag of a row.

## Decision

### 1. Entries after the current entry can move

1. While the Show plays or is paused, the host can move an entry after the current entry to another position after the current entry.
2. The current entry and the entries before it do not move. The rows show "Played" and "Playing" with the time left.
3. While the Show is stopped, the host can move each entry.
4. A move does not stop, restart or load the current track. The next EOF plays the entry that is then next in the cue.
5. The database function of the move examines the session state and the positions in one transaction. It refuses a move that touches the current entry or an entry before it.

### 2. The host moves an entry in three ways

1. A drag of the row.
2. Alt+Up and Alt+Down on the focused row, as ADR 0092 gives for My playlists.
3. "Play next" in the row menu and on the row. It moves the entry to the position directly after the current entry. It shows only on an entry that is not already next.

The text below the cue says: "Drag a row that did not play, or select "Play next". The playing track does not stop."

### 3. Other cue changes stay locked during a show

Replace, clear, add and remove stay unavailable while the Show plays or is paused, as ADR 0068 states. The selection lock stays.

### 4. A load opens the Show page

1. After "Load into Show cue" succeeds, the app opens Show.
2. When the cue is empty, the Show page lists My playlists. Each row has "Load into Show cue". The text is "Select a playlist for your show."
3. The row action "Open playlists" of ADR 0105 opens this list.

Reason: the host loads a cue to play it. A load that stays in Music hides the result and the next step.

## Invariants

- No move changes the current entry or an entry before it.
- A move never interrupts the current track.

## Alternatives

- **Remove the lock for each change.** A replace or a clear during a show removes the current entry from the cue. Rejected.
- **A second "next track" field beside the cue.** It gives two sources for the next track. Rejected: the cue order is the one source.
- **A separate decision for T8.** T8 changes the same section of ADR 0068 and the same page. One record is shorter.

## Tasks

- [Task 001](../tasks/adr-0106-task-001-move-entries-during-a-show.md): Decisions 1 to 3.
- [Task 002](../tasks/adr-0106-task-002-a-load-opens-the-show.md): Decision 4.
