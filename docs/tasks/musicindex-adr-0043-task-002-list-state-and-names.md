# musicindex ADR 0043 Task 002: List State And Reversed Names

Status: Implemented on 2026-11-06.

## Goal

Each list page keeps its own reversed state, and the view model gives each name of R-LIST-7 item 5.

## Files To Inspect

`src/view_models/list_page.rs`, `docs/tasks/musicindex-adr-0043-task-001-icons-and-menu.md`.

## Files Likely To Change

`src/view_models/list_page.rs` only.

## Do Not Touch

Screens, queries and `src/ui/`.

## Constraints

- A view model carries no renderer type.
- A new sort keeps the reversed state of the list. A change of list does not change the state of another list.
- The state starts false.

## Implementation Steps

1. Add `reversed: bool` to the list state, with `set_reversed` beside `set_sort`.
2. Add `ListSort::name(reversed)`: Newest or Oldest, A to Z or Z to A.
3. The sort button and each menu option use that name.
4. Add `ListReverseVm`: its id, the label "Reverse the Order", its pressed state and its icon name as a typed enum.
5. `ListPageHeadVm` gets `reverse: Option<ListReverseVm>`. It is `None` for each list in this packet.
6. Keep the letter bar order a value of the view model: Z to A when reversed.

## Acceptance Criteria

Mechanical:

- A test proves each reversed name.
- A test proves that each list keeps its own state.
- A test proves the order of the letters.

Visual: none.

## Test Commands

```bash
cargo clippy --all-targets -- -D warnings
cargo test --lib list_page
```

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/musicindex-adr-0043-task-002-list-state-and-names.md`
- `/home/citizen/build/musicindex/docs/adr/0043-a-list-sort-can-be-reversed.md`
- `src/view_models/list_page.rs`
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`

Goal:
- The steps of the packet.

Constraints:
- The constraints of the packet. Another agent edits this tree. Do not run git stash, git checkout, git reset, git add, git commit or cargo fmt. Never run the app.

Do not touch:
- The section "Do Not Touch" of the packet.

Acceptance criteria:
- The mechanical criteria of the packet.

Test commands:
- The commands of the packet.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Escalation Triggers

A caller outside `list_page.rs` needs a change to compile.
