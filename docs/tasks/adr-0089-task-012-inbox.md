# ADR 0089 Task 012: The Inbox

Status: Implemented - 2026-11-06. V1 to V4 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 008](adr-0089-task-008-results-rows.md).

## Goal

The Inbox holds what needs a decision (ADR 0089 Decision 3). It holds the tracks that a show cannot use as they are, the updates and the setup that a show needs. Needs attention goes (Decision 2). This packet completes the sidebar sequence of Decision 1.

## Files To Inspect

`src/application/queries/broadcast.rs` (`BroadcastReadinessReport`, `problem_tracks`), `src/view_models/library.rs` (`show_needs_attention_report`, `broadcast_readiness_rows`, `destination_count`), `src/library/app_impl.rs:1024-1070`, `src/app/show.rs:395-405` and `:1085-1128`, `src/view_models/show.rs:322`, `src/view_models/tag_update.rs`, `src/runtime/tag_update.rs`, `src/view_models/library.rs` (`feed_update_display`), `src/application/capability.rs`, `src/view_models/startup/capabilities.rs`, `src/ui/shells/library/content_list_row_actions.rs`, `src/view_models/sidebar.rs`, `tests/architecture_tests.rs`.

## Design

1. `MusicDestination::NeedsAttention` becomes `MusicDestination::Inbox`. Its row moves to the lower group, between Show and Settings. The lower rows now are Show, Inbox and Settings.
2. `src/view_models/inbox.rs` gives `InboxPageVm` with three sections, in this sequence: Tracks, Updates and Setup. A section with no item is absent. An Inbox with no section shows the empty state of R-EMPTY-1.
3. Tracks: each track of `problem_tracks`. The show playlist is the source playlist of the active playback session, the playlist of the cuelist. The tracks of that playlist come first, in its sequence. The other tracks follow in the present sequence. With no session, no track has this priority (Answer 3). Each row keeps its state word and its fix button (`ContentListRowActionKind`), and its selection.
4. Updates: the feed updates of `feed_update_display` and the file differences of the tag update snapshot. Each item gives its count and opens the existing action. The sidebar bottom keeps "Check All Feeds" and the tag button (Decision 1). The Inbox item opens the same confirmation, so no second command exists.
5. Setup: the setup items of the capability issues that a show needs. Use the present scoped capability list. Each item has its existing repair action. Do not add a new probe. If a capability does not state that a show needs it, it stays out, and the run record states the filter.
6. Count: `SidebarVm` shows the sum of the items. The accessibility label gives a sentence, for example "Inbox, 4 items need a decision". A nonzero count does not rely on color alone.
7. `SourceReadinessDisplay` in Show keeps its summary, and its action "Open" opens the Inbox.
8. The Inbox updates in place after each mutation, through the existing refresh paths. It is a Music destination in the navigation stack.
9. Add the test that the full `SidebarVm` sequence equals Decision 1. It is the guard of the sidebar. Change `adr_0084_destinations_d5_03_show_opens_needs_attention_destination` to the Inbox and cite ADR 0089.
10. The name "Needs attention" goes from `src/`, also in comments of live code.

## Do Not Touch

The readiness rules of `BroadcastReadinessState`. Show content, apart from the "Open" target.

## Acceptance Criteria

Mechanical:

- `InboxPageVm` lists each problem track, with the tracks of the session source playlist first, in its sequence. A unit test holds this, and a second test holds the case with no session.
- A section with no item is absent, and an empty Inbox gives the empty state.
- The sidebar count is the sum, and its label is a sentence.
- `SidebarVm` equals the sequence of Decision 1: Search, Releases, Artists, My playlists, Public playlists, Live, the line, Show, Inbox and Settings.
- The gate is Green.

Visual:

- V1: The Inbox row shows a count, and an empty Inbox shows its empty state.
- V2: The tracks of the playlist on the cuelist come first.
- V3: A fix action changes the row in place, and the count changes.
- V4: "Open" in Show opens the Inbox.

## Test Commands

`cargo test inbox`, `cargo test sidebar`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if the session source playlist is not readable from the Library layer without a new channel from `TopApp`. Then pass it through the event path that Show uses.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-012-inbox.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decisions 1 to 3, Answer 3)
- the files that the task lists

Goal:
- The Inbox with Tracks, Updates and Setup. Needs attention goes. The sidebar sequence is complete.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The readiness rules and Show content, apart from the Open target.

Acceptance criteria:
- The mechanical criteria of the task.

Test commands:
- cargo fmt -- --check; cargo clippy --all-targets -- -D warnings; cargo test; cargo build --bin HitIt

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns
