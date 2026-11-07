# ADR 0090: One List Bar Above The Rows

## Status

Accepted - 2026-11-06. The operator selected this option from the [mockups](https://claude.ai/artifact/G7nujJr3pXwU77Y3oeiQ9E), walked the singles group page and the window group page, and accepted it.

Each other page moves in a packet of the [phase plan](../plans/adr-0090-list-bar-phase-plan.md). Until its packet, a page keeps the position that [ADR 0084](0084-music-column-browser.md) Decision 10 and [ADR 0089](0089-the-shell-destinations-and-list-pages.md) Decision 6 give it.

## Context

The "Select" button of an item page is alone in a band above the page title. The operator found this odd on 2026-11-06. The operator then gave two more faults of the options that keep "Select" in a header:

- "Select" competes for space with the sort menu, the view icons and the reverse icon (musicindex ADR 0043 R-LIST-6).
- The mouse moves from the top right to the bottom left to act, then back to the top right for "Done".

These facts come from a search on 2026-11-06:

- Carbon: "The batch action bar appears at the top of the table." Selection starts from a row checkbox.
- Gmail: the actions are in the top toolbar after a row checkbox.
- GNOME HIG: "An action bar should show available actions for the selection, at the bottom of the screen." This is against this decision.
- No source states that sort stays beside the actions, that the bar stays in view during a scroll, or that "Select" is in the bar. No study measures target distance for this choice. This decision is thus a design of this app, and the walk is its main evidence.

## Decision

1. A list of selectable rows on a page has one list bar, directly above its rows.
2. In normal mode, the left end has "Select". The right end has the sort menu, the rows icon, the tiles icon and the reverse icon, in the sequence of R-LIST-6. A control that the list does not have is absent.
3. In selection mode, the left end has "Done", the count, "Select all", the actions for the selection and "Clear". The right end does not change, so a curator can sort while selecting. Selection is by item, so a new order keeps it.
4. The bar has the selection background in selection mode, and each row shows its checkbox. These two facts show the mode change.
5. The bar stays in view while the rows scroll.
6. In a narrow window, the bar wraps. The two ends stay on separate lines, so a destructive action does not move next to the sort menu.
7. A destructive action keeps its role (musicindex ADR 0042 R-ACT-4) and its confirmation.
8. The page has no selection band and no selection bar at its bottom.

## Alternatives Considered

- **The title row, beside "⋯".** Same place as a list page today. It crowds the title row, and it keeps the mouse travel.
- **The title row, with "Done" also in the bottom bar.** It shortens the end of the path only, and it gives two "Done" buttons.
- **A section heading with "Select" only.** It leaves the sort controls in another place, and it keeps the mouse travel.
- **An item in the "⋯" menu.** It hides an essential function, and "Done" cannot be in a menu.
- **The top toolbar.** Each page shares the toolbar, so "Select" comes and goes there. It keeps the mouse travel.
- **The list bar with the sort controls hidden while selecting.** The operator rejected this: it makes sorting and selecting exclusive.
- **A permanent bottom bar.** It has no competition and no travel. It costs a strip on each page, in a place where people may not look for "Select".

## Consequences

- This record supersedes the bar position of ADR 0084 Decision 10 and the "Select" position of ADR 0089 Decision 6. Each list moves to this bar in its own packet.
- A list page has its title row above its rows. A later packet decides if that row is the list bar, and checks musicindex ADR 0027 R-LIST-1 item 4 first. A change to that rule needs a record in `../musicindex`.
- The bar replaces the band above the title, so the page gains that height.

## Invariants

- The view model gives the bar its controls, labels and availability. The renderer decides none of them.
- Each control keeps its accessibility label and its keyboard reach. "Select" and "Done" give the keyboard focus to the rows (ADR 0084 Decision 10).

## Non-Goals

- A change to the sort, view or reverse rules of musicindex ADR 0043.
- Sort or view controls on a group page. The first page shows the controls that the page has today.

## Verification

Mechanical: the view model exposes the controls of each mode, and a test holds them. A guard holds that the first page has no frame header action.

Visual, for the operator, on the first page:

- The bar is directly above the rows, and the page has no band above its title.
- Select, act and Done need almost no mouse travel.
- The mode change is clear.
- The bar stays in view during a scroll.
- A narrow window wraps the bar with its two ends on separate lines.
