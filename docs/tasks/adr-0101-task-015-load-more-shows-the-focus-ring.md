# ADR 0101 Task 015: "Load More" Shows The Focus Ring

Status: Implemented - operator visual checks V1 to V3 open, 2026-11-18.

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

## Report

Cause of the missing ring and the dimmed look:

- A key press that moves the focus to "Load more" scrolls the grid to its end.
- The scroll probe of the list sees the end of the grid and starts the next page load.
- "Load more" is then disabled, so it has opacity 0.4, no ring and no Tab stop.
- The cause is in the HitIt code of the scroll probe. It is not in the Button primitive, in GPUI or in `gpui-component`.

Cause of the Tab stop after a pointer click:

- The Releases view buttons gave the focus back to the list column. Artists, Playlists and Public playlists do not.
- The next Tab then went from the list to "Load more". The Releases code that was different from the other pages is removed. All list pages use the shared `list_page_head` code.

Change:

- `src/ui/focus_ring.rs` records the scroll offset of a key reveal. The probe in `content_list_scroll.rs` does not start a page load for that offset.
- `Button` has the method `focus_target_when_disabled`. A disabled button that has the focus gives it to that handle. Only the Releases status row sets it. It is the list column.
- `LoadMoreFocus` (`releases_everything.rs`) finds the first new row after "Load more". `settle_releases_load_more_focus` moves the focus and the cursor to that row.

Mechanical criteria:

- M1: `adr_0101_load_more_shows_the_focus_ring`. It did not pass before the change. The message was "Tab must reach "Load more" and show its ring".
- M2: `adr_0101_disabled_load_more_is_not_a_tab_stop` and `adr_0101_load_more_that_becomes_disabled_gives_the_focus_to_the_list`.
- M3: `adr_0101_a_click_on_a_view_button_keeps_tab_off_load_more`. It did not pass with the Releases refocus that the change removes.
- Focus after "Load more": `adr_0101_load_more_focuses_the_first_new_row` and two tests of `LoadMoreFocus`. The test runs the click handler of the button. It does not push Enter.
- M4: the assembly, the tests, clippy, the format check and the architecture tests pass.

A key reveal of a row near the end of a list does not start the next page load. "Load more" stays available when a key reveal scrolls to the end.

Visual criteria V1 to V3 stay open for the operator.

Operator choices. The main session recorded them on 2026-11-18, and the operator accepted them on 2026-11-19:

- Operator choice, accepted 2026-11-19: on each list with `attach_list_scroll_with`, a key that moves the cursor to a row near the end no longer loads the next page. The user presses "Load more". The packet did not ask for this change. The main session accepted it, because a load at that time disables the focused button.
- Operator choice, accepted 2026-11-19: the refocus of commit `0f15eee` gave the arrow keys to the list after a pointer click. The view buttons no longer do this. The scope buttons and Select still give the focus to the list, so a Tab after a click on them goes to "Load more". The packet named only the view buttons.
