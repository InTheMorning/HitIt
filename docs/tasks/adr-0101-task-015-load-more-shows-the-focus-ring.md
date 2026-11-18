# ADR 0101 Task 015: "Load More" Shows The Focus Ring

Status: Ready - 2026-11-18. The orchestrator asked for this packet on 2026-11-18, with the approval of the operator.

## Goal

When "Load more" has the keyboard focus, it shows the focus ring and it does not look dimmed. After a pointer click on a view button of a list page, Tab does not go to "Load more" before the list.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 1, focus ring: 2 px with no gap (musicindex ADR 0073 R-FOCUS-2).
- [ADR 0092](../adr/0092-the-keys-of-a-list-row.md): Tab reaches the list, and the list is one Tab stop.
- [ADR 0092 task 002](adr-0092-task-002-detail-list-follows-the-page-controls.md): the list comes after the controls of its page.

## Defect

The [third agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-third.md) defect 3:

- In the Tiles view of Releases, after a pointer click on the Tiles button, the first Tab moved the grid to its end.
- Return then loaded more tiles, and no release opened.
- "Load more" shows no ring in Dark and in Light, and it looks dimmed.

Screenshot: `docs/reviews/screenshots/agent-walk-2026-11-18-third/load-more-focus-no-ring.png`.

## Recorded Facts

The main session read these facts in the code on 2026-11-18. The cause is not known. The implementer finds it with a test before a change.

- `action_button` (`src/ui/shells/library/index_list_status.rs`) draws "Load more" with `UiButton::styled(…, ControlStyle::Neutral)`, `.tab_index(1)` in a list with row focus, `.disabled(!display.available)` and `.on_click(…)`. It does not call `on_activate`.
- The comment of `list_page_head` says that `on_activate` gives the Tab stop, the ring and Enter or Space.
- `Button` (`src/ui/primitives/button.rs`) draws the ring only when the button has the focus, `focus_ring_shows(cx)` is true, and the button is not disabled or busy. A disabled or busy button has opacity 0.4.
- The list column has Tab index 1. "Load more" has Tab index 1 after the list. GPUI sorts the Tab stops by index, then by insertion order.
- The walk record gives an inference: a pointer click on Tiles does not move the focus to Tiles, and the focus stays on the list. Thus the next Tab goes to the next stop after the list, "Load more".

Candidate causes, not confirmed:

1. "Load more" is disabled or busy while a page loads, so it is dimmed and has no ring, but it keeps the focus.
2. The button has no focus handle of its own with `on_click` only, and a different element takes the focus.
3. `focus_ring_shows(cx)` is false after a pointer click until a different key event.

## Change

- Find the cause with a test. Record it in the report.
- "Load more" shows the ring of R-FOCUS-2 when it has the keyboard focus.
- A disabled "Load more" is not a Tab stop. If it gets the focus while it changes to disabled, the focus goes to the list.
- After a pointer click on a view button, the next Tab goes to the next control of the page header, as from the keyboard. If this needs a change of the view buttons, use the shared `list_page_head` code. Do not add a special case for Releases.
- Enter or Space on a focused "Load more" loads more rows. The focus then goes to the first new row (as ADR 0092 task 001 V3 gives for "Show more").

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Button | reuse | src/ui/primitives/button.rs | It draws the ring and the disabled look. Extend it only if the cause is in it. |
| Focus ring state | reuse | src/ui/focus_ring.rs | `focus_ring_shows` decides if a ring shows. |
| Index list status | extend | src/ui/shells/library/index_list_status.rs | `action_button` draws "Load more". |
| List page head | extend | src/ui/composites/list_page_head.rs | The view buttons of each list page. Change only if the click focus is the cause. |

## Mechanical Criteria

- A GPUI test opens Releases in Tiles with a "Load more" button, gives the keyboard focus to it with Tab, and reads the focused control probe of the button. Before the change, the test fails. The implementer records that failure.
- A test proves that a disabled "Load more" is not a Tab stop.
- A test clicks the Tiles button with the pointer, presses Tab, and proves that the focus is not on "Load more".
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: in Light and Dark, in the Tiles view of Releases, Tab to "Load more". It shows the focus ring and does not look dimmed.
- V2: click Tiles, then press Tab. The grid does not move to its end, and the focus goes to the next control of the header.
- V3: press Enter on "Load more". More tiles load, and the focus goes to the first new tile.
- Incorrect: a focused control shows no ring, or Tab moves the grid to its end.

## Stop And Ask

- Stop if the fix changes the Tab order of a page other than the list pages.
- Stop if the cause is in GPUI or in `gpui-component`.

A delegate does not commit. It leaves the change in the working tree and reports it. Apply the STE skill at `/home/citizen/.agents/skills/asd-ste100/SKILL.md` to each comment and report.
