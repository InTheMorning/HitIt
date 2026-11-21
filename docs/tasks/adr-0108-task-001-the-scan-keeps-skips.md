# ADR 0108 Task 001: The Scan Keeps Skips

## Status

Open - 2026-11-21.

## Goal

HitIt stores each skip of the tag update. The scan marks each change that is a skip, and the count does not include skips. The scan deletes each skip that is not current. A confirm writes only the changes that it gets, and it records the skips that it gets. Task 002 adds the check boxes. This task adds no new text to the screen. A skip is not in the count and not in the popup.

## Authority

[ADR 0108](../adr/0108-the-tag-update-selects-each-change.md) Decisions 1.3, 2.2 and 3. It changes [ADR 0076](../adr/0076-playlist-rss-check-for-stale-musicindex-records.md) Decision 8. [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 6 gives the shared storage keys. ADR 0040 gives the runtime actor.

## Terms

- A **change** is one `TagFrameChange` of one file. The key of a change is the track id, the frame label and the stored value (`expected_value`). This is a project term.
- A **change group** is the set of changes of one file that share one stored key (`CompareKey::Stored`). A change with no shared key is a group of one. The check box of ADR 0108 Decision 1.3 selects one group. This is a project term.
- A **skip** is a stored record with a track id, a frame label and a stored value. This is a project term.

## Change

1. Database. Add migration 29 `tag_update_skips` in `src/db.rs`. Set `CURRENT_VERSION` to 29. The table has the columns `track_id`, `frame_label`, `stored_value` and `recorded_at`. The primary key is (`track_id`, `frame_label`, `stored_value`). When the Library removes a track, its skips go.
   - Use the method that the other tables of the track use, for example `ON DELETE CASCADE` or the removal plan. Name the method in the report.
2. Add `src/db/tag_update_skips.rs` on the pattern of `src/db/rss_field_holds.rs`:
   - `record_tag_update_skips(conn, &[TagUpdateSkip])`. A record that exists does not change, and it is not an error.
   - `tag_update_skips(conn) -> Vec<TagUpdateSkip>`.
   - `delete_tag_update_skips(conn, &[TagUpdateSkip])`.
   - `TagUpdateSkip` has `track_id: i64`, `frame_label: String` and `stored_value: String`.
3. Scan model (`src/application/queries/tag_update.rs`):
   - `TagFrameChange` gets `group: usize` and `skipped: bool`. The group index is unique in one file.
   - `shared_key_edits` keeps each edit with the group index of the changed frames of its key. The edits that `TagUpdateFile::edits()` gives do not change.
   - `plan_tag_update_scan` reads the skips with the other stored values. It does not write.
   - `compare_planned_files` sets `skipped` for each change that has a skip with the same key. A group is a skip only when each change of the group has a skip. In the other condition, no change of the group is skipped.
   - `TagUpdateScan` gets `stale_skips: Vec<TagUpdateSkip>`. A skip is stale when its track has a readable compared file and no change of that file has the key of the skip. Thus a skip goes when the stored value changes (Decision 3.3) and when the file value becomes equal to the stored value (Decision 3.4). A skip of a file that HitIt cannot read, or of a track that the scan did not compare, is not stale.
   - `TagUpdateScan::count()` counts each unreadable file and each file with one or more changes that are not skipped. Add `skip_count()`, the number of skipped groups.
   - Add `TagUpdateFile::with_groups(&self, keep: &BTreeSet<usize>) -> Option<TagUpdateFile>`. The result has only the changes and the shared key edits of the kept groups. It is `None` when no change stays, and for an unreadable file.
   - Add `TagUpdateFile::skips_for_groups(&self, groups: &BTreeSet<usize>) -> Vec<TagUpdateSkip>`, one skip for each change of the groups.
4. Write command (`src/application/commands/tag_update.rs`): add `record_skips(conn: &Mutex<Connection>, skips: &[TagUpdateSkip])` and `delete_stale_skips(conn: &Mutex<Connection>, skips: &[TagUpdateSkip])`. `write_tag_updates` does not change. It writes the edits of the files that it gets.
5. Runtime (`src/runtime/tag_update.rs`):
   - `TagUpdateHandle::confirm` and `TagUpdateMessage::Confirm` get `skips: Vec<TagUpdateSkip>`.
   - A confirm first records the skips in its own `spawn_blocking` call. Then it writes the files in the existing `spawn_blocking` call. Keep the text `tokio::task::spawn_blocking(move || {\n                write_tag_updates(` that the ADR 0076 guard looks for.
   - If HitIt cannot record the skips, the confirm writes no file. The snapshot gets the error in `write_error`.
   - After `compare_planned_files`, the scan deletes the stale skips while it holds the database lock. If the delete fails, the error goes to `scan_error`. The snapshot keeps the scan.
6. Callers. `LibraryApp::confirm_tag_update` (`src/library/app_impl.rs`) sends each file with its groups that are not skipped (`with_groups`), and no skips. Task 002 replaces this with the selection of the host.
7. View model (`src/view_models/tag_update.rs`): the popup lists only the files and the lines of the changes that are not skipped. A file with only skipped changes is not in the popup. The button count uses `count()`.
8. Change the ADR 0076 guard text in `tests/architecture_tests.rs` only if a required text moves. The guard must continue to fail on a tag write in the scan.

Main session choices, 2026-11-21:

- The scan does not write. The runtime deletes the stale skips after the compare step. The ADR 0076 guard forbids a tag write in the scan. A database delete in the runtime is not a tag write.
- A skip of an unreadable file stays. The scan cannot know if the file value changed.
- If a step needs a decision that this task file and ADR 0108 do not give, the implementer stops and reports.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Migration registry | extend | src/db.rs | Migration 29 adds the skip table. |
| Stored record module | reuse | src/db/rss_field_holds.rs | The pattern for a small table with record, read and delete functions. |
| Skip store | new | src/db/tag_update_skips.rs | Searched src/db/, src/db.rs, src/application/queries/tag_update.rs and src/rss/check_apply.rs. No table records a frame value that the host refused. `rss_field_holds` holds RSS field values for a feed, not frame changes of a file. |
| Tag update scan | extend | src/application/queries/tag_update.rs | `TagFrameChange`, `TagUpdateScan` and `TagUpdateFile` get groups, skips and the selection functions. |
| Tag update write | extend | src/application/commands/tag_update.rs | It records and deletes skips. `write_tag_updates` does not change. |
| Tag update actor | extend | src/runtime/tag_update.rs | The confirm records skips. The scan deletes stale skips. |
| Tag update view model | extend | src/view_models/tag_update.rs | The popup and the button ignore skipped changes. |

## Mechanical Criteria

1. A migration test: after migration 29, the table `tag_update_skips` exists with its primary key. The migration registry guards of `tests/architecture_tests.rs` pass.
2. A `tag_update_skips` test: record, read and delete give the correct rows. A second record of the same skip is not an error and gives one row.
3. A test: when the Library removes a track, its skips go.
4. A scan test: a change with a matching skip is skipped. A file with only skipped changes is not in `count()`, and `skip_count()` is 1.
5. A scan test: when the stored value is different from the stored value of the skip, the change is not skipped, and the skip is in `stale_skips`.
6. A scan test: when the file value is equal to the stored value, the skip is in `stale_skips`.
7. A scan test: a skip of an unreadable file is not in `stale_skips`.
8. A scan test with a FLAC or Ogg file: two changes that share one stored key have the same group. The group is skipped only when the two changes have a skip.
9. A `with_groups` test: the result keeps only the changes and the shared key edits of the kept groups. It is `None` when no group stays. `skips_for_groups` gives one skip for each change of the groups.
10. A runtime test: a confirm with one skip and one file records the skip and writes the file. The next snapshot shows the change as skipped, and `count()` does not include it.
11. A runtime test: after a scan, a stale skip is not in the database.
12. A view model test: the popup does not list a skipped change, and the button count does not include a file with only skipped changes.
13. The ADR 0076 guard `adr_0076_tag_update_checks_and_scans_write_no_tag` passes.
14. The commit checks of AGENTS.md pass.

## Visual Criteria

None. Task 002 gives the visual criteria.

## Report

Write the report here when the task is complete. Name each changed file, the method that removes the skips of a removed track, and each criterion with its test name.
