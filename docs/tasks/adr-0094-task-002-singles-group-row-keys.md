# ADR 0094 Task 002: Singles Group Row Keys

## Status

Ready - 2026-11-10. Implementation has not started. The operator decides the priority.

This task implements Decision 9 of [ADR 0094](../adr/0094-one-fold-header.md). It does not need task 001.

## Goal

Prove the keys Right and Left on the singles group row of Releases with a key test. Give the open condition in the a11y label of the row.

## Current State On 2026-11-10

- Right and Left are bound to `OpenFocusedGroup` and `CloseFocusedGroup` (`src/app/keyboard.rs`). These commands use `disclose_focused_group` in `src/library/app_impl.rs`.
- `releases_disclosure_step` in `src/view_models/library.rs` has unit tests. No test presses Right or Left.
- The row a11y label in `src/view_models/singles_group.rs` gives the artist, the count, the download state and the level. It does not give the open condition. Only the chevron label gives it: "Show singles" or "Hide singles".
- The chevron in `src/ui/shells/library/destination_lists.rs` has no Tab stop.

## Scope

1. Add the open condition to the row a11y label in `src/view_models/singles_group.rs`, for example "{artist}, {count} singles, {download}, {level}, Open". The view model gives the text.
2. Add a key test on Releases with a singles group row in focus. Right opens the group. Left closes it. Left on a single of an open group gives the focus to its group row (musicindex ADR 0061 R-LISTKEY-1 item 3).
3. The test also asserts that Tab does not stop on the chevron.
4. Enter on the group row keeps opening the group page. The test asserts it.

## Do Not Touch

- The key bindings and `disclose_focused_group`, unless the test finds a fault. If it finds a fault, report it before the change.
- The chevron look and its pointer click.
- `FoldHeader` and task 001.

Do not run the app as an agent. The Xvfb screenshot script of the operator is permitted.

## Mechanical Criteria

Each criterion is checked by a unit test or a key test.

- The row a11y label gives "Open" for an open group and "Closed" for a closed group.
- Right opens and Left closes the focused group row on Releases.
- `releases_disclosure_step` gives `FocusGroup` with the group key for Left on a single of an open group, and `None` for Right on that single. On 2026-11-10 the test `adr_0084_selection_right_and_left_open_and_close_a_group` in `src/view_models/library.rs` asserts this. Keep the test, and add the same key to the key test.
- Tab does not stop on the chevron.
- Enter on the group row opens the group page.

## Visual Criteria

No visual criterion. The look does not change.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin v4vmm
git diff --check
```
