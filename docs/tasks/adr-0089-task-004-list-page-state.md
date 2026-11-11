# ADR 0089 Task 004: List Page State

Status: Implemented - 2026-11-05. V1 to V4 are open. The checks are in section 11 of [pending human checks](../pending-human-checks.md).

Depends on: [packet 003](adr-0089-task-003-one-sidebar.md).

## Goal

Each list page has its own sort, view and scope state, with the defaults of ADR 0089 Decision 5. A list page has a title row: the title at the left, then sort, view and Select at the right (musicindex ADR 0027 R-LIST-1 item 4, ADR 0089 Decision 6). The scope "All | In Library" is below the title at the left, and All is the default.

## Files To Inspect

`src/view_models/index_name_list.rs` (`MusicListScope`, `MusicScopeControlDisplay`), `src/view_models/workspace/chrome.rs` (`ContentViewMode`, `FrameHeaderActionDisplay`), `src/ui/composites/frame_shell.rs`, `src/library/music_scopes.rs`, `src/library/app_impl.rs` (`render_list_column`, `set_content_view_mode`), `src/view_models/library.rs` (`releases_list`, `artists_list`, `header_action_for_list_column`, `PlaylistSort`), `src/view_models/release_download_state.rs`, `src/library/download_queue.rs`, `src/app/settings.rs`, `src/ui/composites/segmented_control.rs`, `src/ui/shells/library/destination_lists.rs`.

## Design

1. A new module `src/view_models/list_page.rs` owns `ListPageKey` (`Releases`, `Artists`, `MyPlaylists`, and later keys), `ListSort` (`Newest`, `AToZ`), `ListView` (`Rows`, `Tiles`), `ListScope` (`All`, `InLibrary`) and `ListPageState`. `ListPageState::default_for(key)` gives the table of Decision 5. A later packet adds its own key and its default.
2. `LibraryViewModel` holds one state for each key. Replace `releases_scope`, `artists_scope` and the global use of `content_view_mode` for these pages. Discover keeps its present view mode until packet 005. The Settings preference of the view mode goes if no page reads it after packet 005, and packet 005 deletes it then.
3. `ListPageHeadVm` gives the title, the scope control (absent if the page has none), the sort button text and its menu (R-LIST-1 item 1: "Newest ⌄", a check mark on the current sort), the view icons (R-LIST-1 item 3: the current icon has a tint) and the Select action. A shared composite `src/ui/composites/list_page_head.rs` draws it. It is the one owner of this row. Delete the scope and header action of the frame shell for Music.
4. In this packet the sort button shows only the current sort of each page. Packet 005 adds the sort behavior. If the page has one sort, the menu has one item. My playlists has no sort button (Decision 5). Delete `PlaylistSort` and its cycle: the curator sets the sequence (Decision 5).
5. The words: "Library" and "Everything" become "In Library" and "All". Rename `MusicListScope` to `ListScope` and its variants.
6. Scope model (`ListScope::holds`). In Library holds a release that is downloaded, partly downloaded, downloading, queued or failed. All holds the Index items and the Library items. Each Library item keeps its state (Decision 5).
7. The failed fact: add `ReleaseDownloadState::Failed` from the outcomes of the download queue. It is session state and has no migration. The download icon shows the failed state with the danger status and a word in its accessibility label. If the queue has no retained outcome for each feed, add it there.
8. A removed release is not in the Library, so it shows only in All. A test holds this.
9. In Library shows its count. All shows no count (Decision 5, Answer 5). The view model gives these texts.
10. The default scope All sends the name request when the page first shows, also at start. The Library rows show before the response. Reuse the merge of `releases_everything` and `artists_everything`.
11. A destination keeps its state when the curator goes to another destination and back. The change of a scope clears the selection, as today.

## Do Not Touch

The sort source (packet 005). The results page (packet 008). Rows (packet 014).

## Acceptance Criteria

Mechanical:

- `ListPageState::default_for` gives each row of Decision 5 that exists in this packet. A unit test compares the table.
- The state of one page does not change when another page changes. A unit test holds this.
- `ListScope::holds` puts a downloaded, downloading or failed release in In Library, and a removed release only in All.
- `ListPageHeadVm` gives a count for In Library and no count for All.
- The gate is Green.

Visual:

- V1: Releases and Artists open with All selected. The Library rows show at once, and the Index rows follow.
- V2: The title row has the title at the left, and the sort button, the view icons and Select at the right. The scope is below the title.
- V3: A change of scope on Releases does not change Artists.
- V4: A failed download shows in In Library with its danger icon.

## Test Commands

`cargo test list_page`, `cargo test --test architecture_tests`, then the full gate.

## Escalation

Stop if the merge of All needs a new Stophammer endpoint.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/adr-0089-task-004-list-page-state.md
- docs/plans/adr-0089-shell-phase-plan.md (section Rules For Each Packet)
- docs/adr/0089-the-shell-destinations-and-list-pages.md (Decisions 5 and 6)
- ../musicindex/docs/adr/0027-cross-app-shell-lists-and-toolbar.md (R-LIST-1)
- the files that the task lists

Goal:
- Own state and defaults for each list page, one title row composite, and the scope All and In Library.

Constraints:
- The task and the rules of the phase plan. Do not commit. Do not run the app.

Do not touch:
- The sort source, the results page and the row content.

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
