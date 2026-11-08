# ADR 0092: The Keys Of A List Row

## Status

Accepted - 2026-11-08. The orchestrator session made this decision on 2026-11-08 to apply the keyboard reach invariant of [ADR 0090](0090-one-list-bar-above-the-rows.md). The orchestrator session relayed it to this session, and on that date no person checked the keys. Implemented on 2026-11-08 for My playlists, Releases and the Inbox. A list with no list focus (Artists, Public playlists, Live, Search) has the keys when it has a list focus.

A pointer menu owns its keys while it is open. The first entry takes the focus. Up and Down move through the entries, and Home and End go to the first and the last entry. Return and Space run the entry. No key of the list goes to the pane.

## Context

ADR 0090 Invariants require that each control keeps its keyboard reach. The agent screenshot run of 2026-11-08 found these faults:

1. My playlists has no row in the Tab sequence. Tab goes from "New Playlist" back to the sidebar. A keyboard user cannot open a playlist, its row menu, "Move up" or "Move down".
2. A row menu opens only from its "…" button, which shows on a pointer hover. The menu of a list row stays out of the Tab sequence (ADR 0090 Invariants), so a keyboard user cannot open it.
3. The Curator Order of the [ADR 0089 amendment of 2026-11-08](0089-the-shell-destinations-and-list-pages.md#amendment---2026-11-08-the-stored-curator-order) has a drag, and the row menu has "Move up" and "Move down". With faults 1 and 2, no key moves a playlist.

## Decision

1. **My playlists takes the list focus of Releases.** Tab reaches the list. Up and Down move the focused row. Enter opens the focused row. The focused row shows the focus ring, as on Releases.
2. **Shift+F10 and the Menu key open the row menu of the focused row.** These are the platform keys for a context menu. They apply to each list with a row menu on its rows. Escape closes the menu and gives the focus back to the row.
3. **Alt+Up and Alt+Down move the focused row in Curator Order.** They apply to My playlists with the sort Curator Order, not reversed, in the view Rows. The drag and "Move up" and "Move down" apply in the same condition. The focus stays on the moved row. In each other condition the keys do nothing.
4. The view model gives the availability of each key action. The renderer decides none of them (the Durable Set: typed action state).

## Invariants

- Each row that has a row menu can open it from the keyboard.
- Each move that a drag can make, a key can also make.

## Non-Goals

- A key to move a track of a playlist. The playlist page has its own ADRs.
- A change of the Tab sequence of a page header.

## Verification

Mechanical: view-model tests give the availability of each key action in each state. Visual: the main agent checks each key on its own Xvfb display, and a person checks the keys in a desktop session.
