# musicindex ADR 0043 Task 003: The Reverse Control And Local Lists

Status: Implemented on 2026-11-06. The visual check is open in section 13 of pending human checks.

## Goal

The title row shows the sort menu, the rows icon, the tiles icon and the reverse icon, and the lists with only local rows reverse all their rows.

## Files To Inspect

`src/ui/composites/list_page_head.rs`, `src/view_models/list_page.rs`, `src/library/music_scopes.rs`, `src/view_models/search_results/sort.rs`, `src/view_models/library.rs`.

## Constraints

- The reverse icon renders from `ListReverseVm`. No renderer decides its availability.
- A list gets `reverse: Some` only when each of its rows is local: Releases and Artists with the scope In Library, My playlists, and Results with the scope In Library.
- A view icon that is not current has `SecondaryLabel`. Each other icon button stays as it is.
- The mounted list updates in place when the state changes.
- A reverse keeps the selected letter and the Select state. Only a new sort clears the letter.
- The keys of the letter bar follow the reversed sequence of the letters.
- Delete the three `expect(dead_code)` attributes of packet 002 in `list_page.rs`. Map `ListReverseIcon` to `IconName::SortUsual` and `IconName::SortReversed` in the renderer.
- The reverse icon has the role `ControlStyle::ToolbarIcon`. When it is not pressed it has `SemanticColor::SecondaryLabel`. When it is pressed it uses `.selected(true)`. It has an accessibility label and a tooltip from the view model.
- If a list cannot give each of its rows from the local database, it keeps `reverse: None`. Report it, and do not guess.

## Acceptance Criteria

Mechanical:

- A test proves that the reverse command changes the sequence of the rows of each local list.
- A test proves that each list from the Index gives `reverse: None`.
- A test proves the sequence of the controls in the view model.

Visual, in a new section of pending human checks:

- The control looks like the Sort Control Mockup in the dark and the light theme.
- The reverse icon moves no other control.

## Test Commands

```bash
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test architecture_tests
```

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/musicindex-adr-0043-task-003-reverse-control.md`
- `/home/citizen/build/musicindex/docs/adr/0043-a-list-sort-can-be-reversed.md`
- The files in the section "Files To Inspect"
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`

Goal:
- The goal of the packet.

Constraints:
- The constraints of the packet. Another agent edits this tree. Do not run git stash, git checkout, git reset, git add, git commit or cargo fmt. Never run the app.

Do not touch:
- `src/ui/primitives/`, `src/ui/tokens.rs`, the Index query code.

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

A list mixes local rows and Index rows in a way that the table of the phase plan does not name.
