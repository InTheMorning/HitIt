# musicindex ADR 0042 Task 003: The Role Of Each Call Site

Status: Implemented on 2026-11-06. The visual check is open.

## Goal

Each call site that packet 002 cannot decide gets the role that the result of its command gives. Owner: musicindex ADR 0042 R-ACT-4 and R-ACT-6.

The roles are `ControlStyle::Secondary` (Action), `Neutral`, `Unfilled`, `CompactAction`, `Destructive` and `DestructiveRowAction`. `ControlStyle::by_result(changes_stored_data)` selects `Secondary` or `Neutral`.

## Files Likely To Change

Only the files in the tables and their view-model owner.

## Do Not Touch

`src/ui/primitives/button.rs`, `src/ui/control_styles.rs`, `src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/contrast.rs`. Do not revert or reformat other edits in a file, because another agent edits this tree.

## Static Sites

Search each site by its identifier. Line numbers can move.

| File | Site | New role |
|---|---|---|
| `src/ui/composites/letter_bar.rs` | Letter that is not selected | `Unfilled` |
| `src/ui/composites/tab_bar.rs` | Item and "More" that are not selected | `Unfilled` |
| `src/ui/shells/track.rs` | Artist or album name link | `Unfilled` |
| `src/ui/shells/entity.rs` | Album name link | `Unfilled` |
| `src/app/toolbar.rs` | Search button with a label | `Neutral`. The comment stays. |
| `src/ui/composites/download_status_item.rs` | "Dismiss" | `Neutral` |
| `src/ui/composites/confirmation_dialog.rs` | "Cancel" | `Neutral` |
| `src/ui/composites/live_status_strip.rs` | "Open Show" | `Neutral` |
| `src/ui/shells/search_results_inspector.rs` | "Show details" | `Neutral` |
| `src/ui/shells/track_page.rs` | Plain actions | `Neutral` |
| `src/ui/shells/tag_update_confirmation.rs` | "Update n files" | `Neutral` |
| `src/ui/shells/library/public_playlists.rs` | Copy "Cancel" | `Neutral` |
| `src/ui/composites/startup_report.rs` | Startup action that is not `OpenApp` | `Neutral` |
| `src/ui/composites/selection_action_bar.rs` | "Remove n from playlist" | `Destructive` |
| `src/ui/shells/playlist.rs` | `render_removed_track_action` | `DestructiveRowAction` |
| `src/ui/shells/playlist.rs` | "Check RSS", "Download" | `Secondary` |
| `src/ui/shells/playlist.rs` | Play control when it shows a word ("Repair playback") | `CompactAction`. An icon stays `RowAction`. |
| `src/ui/composites/list_page_head.rs` | Sort menu trigger | Add `.trigger_style(ControlStyle::Neutral)` |
| `src/ui/shells/track_detail_metadata.rs` | "Apply tags" | Add `.control_style(ControlStyle::CompactAction)` to the `action_button` result |

## Sites With A Role From The View Model

A renderer does not decide a role. Add `changes_stored_data(self) -> bool` to the action type in its view-model file, with a test. The renderer calls `ControlStyle::by_result`.

| Renderer | Action type | True for |
|---|---|---|
| `correction_button` in `maintenance_forms.rs` | `CorrectionAction` | Save correction, End app session |
| `session_button` in `maintenance_forms.rs` | `SessionAction` | End app session |
| `database_tools` in `maintenance_forms.rs`, when not destructive | `DatabaseAction` | Back up, Preserve, Upgrade backup, End app session |
| `capability_button` in `startup_report.rs` | Its action type | Pending-entry "Run original action" and its other `run_label` labels |
| Setup action in `inbox.rs` | Its action type | The same rule as `capability_button` |
| Update action in `inbox.rs` | `InboxUpdateKind` | `FeedUpdates` |

Each check that writes nothing is false. Other labels are false.

## Other Sites

- `SearchPageActionStyle`: add the variant `Action`, and map it to `ControlStyle::Secondary`. Map `Plain` to `ControlStyle::Neutral`. Give `Action` to "Save search" and "Save Query". Correct the comment of `Plain`.
- `settings_actions` in `settings.rs`: "Use Defaults" is `Secondary`. Keep `Save` as `Primary` and a selected choice as `SelectedTab`.
- `publisher_action_button` in `show_detail_panel.rs`: `CompactAction` for Start, Stop and Reset. "Logs" stays `RowAction`. Use the typed data of the display, not the label text.
- `detail_action_button` in `show_detail_panel.rs`: add a role parameter. `CompactAction` for Create, Replace, Attach, Detach, Connect and Disconnect. `RowAction` for other labels.

## Acceptance Criteria

Mechanical:

- Each `changes_stored_data` has a test for one true and one false action.
- `SearchPageActionStyle::Action` has a test: "Save search" has it.
- The gate commands pass, or a failure comes from a file that this packet does not change.

Visual: section 12 of [pending human checks](../pending-human-checks.md).

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
- `docs/tasks/musicindex-adr-0042-task-003-call-site-roles.md`
- `src/ui/control_styles.rs`
- `/home/citizen/.agents/skills/asd-ste100/SKILL.md`

Goal:
- Apply each table of the packet.

Constraints:
- Another agent edits this tree. Read a file just before you edit it. Do not run `git stash`, `git checkout`, `git reset`, `git add` or `git commit`. Do not run `cargo fmt`. Format only with `rustfmt --edition 2021 --check` on a leaf file, and make the correction by hand.
- Never run the app.

Do not touch:
- The files in the section "Do Not Touch".

Acceptance criteria:
- The criteria of the packet.

Test commands:
- The commands of the packet.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Escalation Triggers

- A site does not match its row.
- An architecture guard requires an old style string. Update the guard only when it names the role of this packet, and report it.
