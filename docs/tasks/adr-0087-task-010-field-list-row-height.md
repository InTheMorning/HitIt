# ADR 0087 Task 010: Each Field List Row Keeps Its Height

Status: Ready - 2026-10-24.

Depends on: [packet 008](archive/adr-0087-task-008-field-list-stays-open.md).

## Problem

The final walk of 2026-10-24 found this defect (run record, walk log, "ADR 0087 packet 008, new
finding"). The screenshots are in `~/.claude/v4vmm-shots/20261024-182128/`. The window is
1440 x 845.

- `04-fieldlist3-v1-after-enter.png`, `06-fieldlist3-v2-in-index-space.png` and
  `11-fieldlist3-v3-after-click.png`: the full list of 11 fields shows after a term and a space.
  Each row is too short. The field name, for example `title:`, overlaps the hint line of the row
  above, for example "an artist name  artist:jsmovie". The list stops in the row `is:`, so `is:`
  shows only in part. The rows `playlist:` and `has:` do not show.
- `09-fieldlist3-v3-kind-values.png`: the list of 3 values of `kind:` is correct. Each row shows
  its name and its description with no overlap.

The same defect caused two earlier failed checks:

- Packet 004 V1 failed on 2026-10-23: "The list shows 9 of 11 rows and stops in the `is:` row.
  `playlist:` and `has:` cannot be reached, and the list has no scroll bar. Each description is
  nearer to the next field name than to its own" (run record of 2026-10-23).
- Packet 005 V7 failed on 2026-10-23: "A long artist name wraps to three lines, but its row keeps
  one line of height. Its "Library" mark is drawn into the next row".

Packet 008 did not cause the defect. Packet 008 shows the full list more frequently, so the
operator sees the defect more frequently.

This packet repairs the layout only: the overlap, the cut rows and the access to each row. The
visual style of the list stays the same. The visual overhaul (Phase 7) owns the style.

## Authority

- [ADR 0087](../adr/0087-search-fields-and-live-results.md) Decision 3: "Each row gives one field,
  its values and one example". "A shared composite under `src/ui/composites` draws the list".
- ADR 0076 Decision 8, as `src/ui/composites/confirmation_dialog.rs:138-147` restates it: an item
  in a scrolling column carries `flex_shrink_0()`, so that it keeps its own height.
- [ADR 0084](../adr/0084-music-column-browser.md) Decision 1: the guard
  `adr_0084_columns_list_rows_do_not_shrink_in_a_scrolling_column`
  (`tests/architecture_tests.rs:21878-21890`) holds the same rule for `ListRow`. Its incident is
  from 2026-10-12.
- [AGENTS.md](../../AGENTS.md): "Token discipline", "No isolated visual tweaks", "Every fix gets a
  guard", "Column text does not call `truncate()`" and "An acceptance criterion states how it is
  checked".

## Recorded Facts - 2026-10-24

An agent read each fact below on 2026-10-24 in the working tree at commit `78afacc`.
`$REG` is `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`.

### The cause: the rows shrink in the scrolling row list

- `src/ui/composites/search_field_list.rs:72-79`: the row list is a flex column with
  `.max_h(Size::ColumnRegular.scaled(cx))`, `.overflow_y_scroll()` and
  `.track_scroll(&scroll)`.
- `src/ui/tokens.rs:898` and `:900`: `Size::ColumnRegular` is 320 px. `Size::RowMd` is 36 px.
- `src/ui/composites/search_field_list.rs:178-203` `render_row`: each row is a flex column with
  two lines, `title_line` and `detail_line`. It has `.min_h(Size::RowMd.scaled(cx))`. It has no
  `flex_shrink_0()` and no `flex_none()`.
- A flex child shrinks by default. When the sum of the row heights is more than the `max_h` of
  the column, the layout shrinks each row to its `min_h`, 36 px. The two lines of a row need more
  than 36 px. Thus the second line paints below the row bounds, on the next row.
- The screenshots agree with this cause. The pitch of the 11 rows is 36 px. The visible list is
  320 px high, and 320 / 36 = 8.9 rows. Thus 8 rows and a part of `is:` show.
- The 3 rows of `kind:` do not fill 320 px. They keep their content height, and their pitch is
  52 px in `09-fieldlist3-v3-kind-values.png`. Thus the value rows are correct only because the
  list is short. A long list of artist names shrinks in the same way (packet 005 V7).
- After the shrink, the content is 11 x 36 = 396 px. Thus the list scrolls by 76 px only. The
  list shows no scroll bar, so the curator does not see that more rows are below.

### The same defect and its repair in other composites

- `src/ui/composites/confirmation_dialog.rs:138-147`: "A flex child with `overflow_hidden()` and
  no `flex_shrink_0()` has a minimum height of zero. When the items are taller than `max_h`, the
  layout shrinks each item to zero height. `flex_shrink_0()` keeps each item at its own height."
- `src/ui/composites/confirmation_dialog.rs:289-330` R76-8-01 and R76-8-02: a
  `TestAppContext` test draws 40 items and compares each height with the height of one item. It
  reads the bounds with `debug_selector` and `VisualTestContext::debug_bounds`.
- `tests/architecture_tests.rs:21878-21890`: `ListRow` keeps `.flex_shrink_0()`.

### The scroll and the keys

- `src/app/search_field_list.rs:219-224`: `FieldListKeyOutcome::Select(index)` calls
  `self.scroll.scroll_to_item(index)`. The row list holds only the rows. The notices are outside
  it (`src/ui/composites/search_field_list.rs:89-95`). Thus the index of a row is the index of
  its child.
- `src/ui/composites/music_column_scroll.rs:56-72` `music_column_tracked_scrollbar` puts the
  `gpui_component` scroll bar on a parent of a tracked scroll container with
  `.vertical_scrollbar(scroll)`. That helper is for a Music column only.
- `$REG/gpui-component-0.6.1/src/scroll/scrollable.rs:33-35`: `vertical_scrollbar` takes a
  `ScrollbarHandle`. `ScrollHandle` is one.

### Candidates that the facts exclude

- The row height token. `Size::RowMd` is a minimum, not a fixed height. It gives the 36 px only
  because the layout shrinks the row to its minimum.
- A uniform item height of a gpui `list`. The row list is a `div` with children, not a `list` or
  a `uniform_list`.
- The `anchored` and `deferred` overlay. `snap_to_window_with_margin` moves the surface. It does
  not set a height. The visible surface ends at about y 365, far from the bottom edge at 845.
- `truncate()`. No text of the composite calls it.

## Design

### The rule

Each row of the field list keeps its content height. It never shrinks below the height of its
two lines. When the rows are taller than the maximum height of the row list, the row list
scrolls. This is the rule of ADR 0076 Decision 8 and of `ListRow`.

### The shared owner

The fix is in the composite `src/ui/composites/search_field_list.rs`, in `render_row` and in
the row list. It is not in a screen and not in `src/app/search_field_list.rs`. The fix adds no
raw literal. Each size comes from a `Size` token.

### What does not change

- The rows, the edits, the notices, the keys and the view model of packets 004, 005 and 008.
- The width of the list, `Size::ColumnTall`.
- The fonts, the colors, the padding, the gaps, the border and the fill of a row.
- The two lines of a row. The visual overhaul can change them.

## Orchestrator Decisions

- **D1. The fix is `flex_shrink_0()` on the row element.** It is the repair of ADR 0076
  Decision 8 and of `ListRow`. Keep `.min_h(Size::RowMd.scaled(cx))` as the minimum.
- **D2. The layout test is the guard, and it is situational.** It cites ADR 0087 Decision 3 and
  the walk of 2026-10-24. A source-text guard in `tests/architecture_tests.rs` is not necessary,
  because the test measures the result. When an ADR replaces Decision 3, delete the test with
  that ADR.
- **D3. The key Down scrolls the selected row into view.** This is the current behavior
  (`scroll_to_item`). R87-10-02 proves it with rows that keep their height.

## Questions

These questions need an operator answer. ADR 0087 does not decide them. The design of this
packet follows each recommendation.

- **Q1. Which token gives the maximum height of the row list?** With the fix, a field row is
  about 52 px high, as in `09-fieldlist3-v3-kind-values.png`. The 11 fields are about 572 px high.
  - A: `Size::ColumnRegular`, 320 px, as today. About 6 rows show. The curator scrolls to see
    `duration:`, `is:`, `playlist:` and `has:`.
  - B: `Size::ColumnTall`, 480 px. About 9 rows show. The curator scrolls to see `playlist:` and
    `has:`. No new token.
  - C: a new token `Size::FieldListMaxHeight` of 640 px. The 11 fields show with no scroll in a
    window of 845 px. A long list of artist names still scrolls. In a short window,
    `snap_to_window_with_margin` moves the surface, and the bottom rows can go out of the window.
  - Recommendation: B. It uses a token that exists, and a scrolling column of 480 px already
    uses it. More of the field help shows than today. It does not depend on the window height.
    The visual overhaul can change the token.
- **Q2. Does the row list show a scroll bar?**
  - A: yes. The row list gets the `gpui_component` scroll bar with `.vertical_scrollbar(&scroll)`
    on a parent, as `music_column_tracked_scrollbar` does. Do not call the Music helper.
  - B: no. The list scrolls with the wheel and with Down and Up. The visual overhaul adds the
    scroll bar.
  - Recommendation: A. Packet 004 V1 failed partly because "the list has no scroll bar". With
    Q1 A or B, rows are below the visible area. The Apple HIG asks for a clear state, and a
    scroll bar tells the curator that more rows are below.

## Required Changes

1. `src/ui/composites/search_field_list.rs` `render_row`: add `.flex_shrink_0()` to the row
   element. Keep `.min_h(Size::RowMd.scaled(cx))`.
2. `src/ui/composites/search_field_list.rs` row list: set `.max_h(..)` to the token of the
   answer to Q1. When the answer is C, add `Size::FieldListMaxHeight` to `src/ui/tokens.rs`
   with a doc comment and to the token list of `src/ui/tokens.rs:1613-1616`.
3. When the answer to Q2 is A: put the row list in a parent with `.relative()` and call
   `.vertical_scrollbar(&scroll)` on that parent. Keep `.overflow_y_scroll()` and
   `.track_scroll(&scroll)` on the row list. Keep `FieldListVm::LIST_ID` on the row list.
4. Add `.debug_selector(..)` with the `element_id` to each row, and with `FieldListVm::LIST_ID`
   to the row list. The tests read the bounds through these selectors, as R76-8-01 does.
5. Change the module doc comment of `src/ui/composites/search_field_list.rs`. Add one paragraph:
   each row carries `flex_shrink_0()`, so it keeps its height, and the row list scrolls (ADR
   0087 Decision 3, ADR 0076 Decision 8).
6. Add each test of "Mechanical Acceptance Criteria".
7. Change no other source file. `src/app/search_field_list.rs` changes only for the test
   R87-10-02.

The orchestrator then makes these document changes in the same commit:

- [Pending human checks](../pending-human-checks.md): retire packet 004 V1 and packet 005 V7
  into V1 and V3 of this packet. Add V1 to V5 of this packet.
- The [overhaul plan](../plans/design-and-cleanup-overhaul-plan.md) line 187: remove "ADR 0087
  packet 004 V1 (9 of 11 field rows show)" and "ADR 0087 packet 005 V7 (a long name overlaps the
  next row)", and name this packet.
- The [run record](../reviews/orchestrated-run-2026-10-24.md) and the
  [ADR 0087 phase plan](../plans/adr-0087-search-fields-phase-plan.md): name this packet.
- [Packet 004](adr-0087-task-004-field-list.md) V1 and
  [packet 005](adr-0087-task-005-field-list-values.md) V7: mark them as walked again by this
  packet.

## Mechanical Acceptance Criteria

Each criterion is mechanical. A `gpui::TestAppContext` test proves it. Use the prefix
`adr_0087_field_list_` for each test. Each test draws the window, then reads bounds with
`VisualTestContext::debug_bounds`. "Open" means a `FieldListState` with `focused` `true` and
`opening` `Open`.

| Case | Owner | Required proof |
|---|---|---|
| R87-10-01 | `src/ui/composites/search_field_list.rs` | The composite draws the `FieldListVm` that `FieldListVm::build` gives for an empty text, Open (11 `Field` rows). For each row `i` from 0 to 9, the top of row `i + 1` is not above the bottom of row `i`. The height of the row `artist:` is equal to its height in a `FieldListVm` that holds only that row |
| R87-10-02 | `src/app/search_field_list.rs` | With the setup of R87-4-17, focus the empty field and press `down` 11 times. The row `has:` is selected. Its bounds are inside the bounds of `FieldListVm::LIST_ID`. The height of the row list is not more than the token of Q1, scaled |
| R87-10-03 | `src/ui/composites/search_field_list.rs` | The composite draws a `FieldListVm` of 30 `Value` rows with fixed labels, as for `artist:`. Each row height is equal to the height of the same row in a `FieldListVm` of one row. No two rows overlap |

R87-10-01 is the regression test of ADR 0087 Decision 3. Its name is
`adr_0087_field_list_r87_10_01_rows_keep_their_height`. Its doc comment names "ADR 0087
Decision 3: each row gives one field, its values and one example", ADR 0076 Decision 8 and the
walk of 2026-10-24. Each assertion has this failure message:

```text
ADR 0087 Decision 3 (each row gives one field, its values and one example): row "<label>" is <h> px high and overlaps the next row.
A row in the scrolling field list must keep its content height. Keep `.flex_shrink_0()` on the row element in render_row of src/ui/composites/search_field_list.rs.
```

## Visual Acceptance Criteria

Each criterion is visual. Only a person can examine it in the running app. No test proves it.

- V1 (packet 004 V1 again): click in the empty search field. Each row shows the field name, then
  its values and its example under the name. No text of a row touches the next row. Scroll to
  the bottom. The last rows `is:`, `playlist:` and `has:` show in full.
- V2: type `haleen ` with a space. The full list shows with the layout of V1.
- V3 (packet 005 V7 again): type `artist:`. The names show with no overlap. A long name wraps,
  and its row grows to hold each line and its mark.
- V4: with the full list open, press Down 11 times. The selection moves to `has:`, and the list
  scrolls so that `has:` shows.
- V5: Light and Dark themes, a normal width and a narrow width. The list stays in the window. When
  the answer to Q2 is A, the scroll bar shows at the right edge of the rows and covers no text.

## Exclusions

- No change to the fonts, the colors, the spacing, the two lines of a row, the mark or the width.
  The visual overhaul (Phase 7) owns the style.
- No change to `src/view_models/search_field_list.rs` or to `src/view_models/search_query.rs`.
- No change to the keys, the edits or the open state of packet 008.
- No change to gpui-kit, gpui-component, the popover primitive or `music_column_scroll.rs`.
- No change to the confirmation dialog or `ListRow`.

## Files To Inspect

- [AGENTS.md](../../AGENTS.md) and the [source map](../architecture/source-map.md).
- [ADR 0087](../adr/0087-search-fields-and-live-results.md) Decision 3.
- [Packet 004](adr-0087-task-004-field-list.md), [packet 005](adr-0087-task-005-field-list-values.md)
  and [packet 008](archive/adr-0087-task-008-field-list-stays-open.md).
- `src/ui/composites/search_field_list.rs`.
- `src/ui/composites/confirmation_dialog.rs` `confirmation_items` and R76-8-01, as the pattern.
- `src/ui/composites/music_column_scroll.rs` `music_column_tracked_scrollbar`, as the pattern of
  the scroll bar, to read only.
- `src/app/search_field_list.rs` `handle_key` and the test R87-4-17.
- `src/ui/tokens.rs` `Size`.

## Checks

```bash
cargo test --lib adr_0087_field_list_
cargo test --lib adr_0087_field_values_
cargo test --lib adr_0076_confirmation_list_
cargo check --all-targets
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --test architecture_tests
touch src/main.rs && cargo build --bin v4vmm
```

## Rollback

Revert the commit. This packet adds no migration, no stored data and no request.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `AGENTS.md` and `docs/architecture/source-map.md`
- This packet: `docs/tasks/adr-0087-task-010-field-list-row-height.md`
- ADR 0087 Decision 3
- Each file in "Files To Inspect"

Goal:
- Make each change in "Required Changes" items 1 to 7.

Constraints:
- Follow the rust-dev skill and the conventions in `AGENTS.md`.
- Write each comment in ASD-STE100 Simplified Technical English. Use the shared skill at
  `~/.agents/skills/asd-ste100/SKILL.md`.
- Follow each decision in "Orchestrator Decisions" and the answers to Q1 and Q2.
- Use only `Size`, `Spacing` and the other named tokens. Add no raw pixel value.
- No text calls `truncate()` (ADR 0063).
- Tests use fixed text. No test sends a request or opens a database.
- Never run `git add`, `git rm`, `git commit`, `git checkout`, `git restore`, `git stash` or
  `git reset`.
- Do not run the app: no `cargo run`, no `xvfb-run` and no display attempt.

Do not touch:
- `src/view_models/search_field_list.rs`, `src/view_models/search_query.rs` and
  `src/app/tab_bar.rs`.
- `src/ui/composites/music_column_scroll.rs`, `src/ui/composites/confirmation_dialog.rs` and
  `src/ui/composites/list_row.rs`.
- gpui-kit, gpui-component and `Cargo.toml`.
- Any ADR and any document other than this packet.

Acceptance criteria:
- Each case R87-10-01 to R87-10-03 has a passing test.
- R87-10-01 fails when `.flex_shrink_0()` is removed from the row element. Make this check one
  time, then put the call back.
- Each command in "Checks" is Green.
- The packet has an "Implementation Result - <date>" section.

At the end, report: files changed, tests run, behavior changed, deviations, unresolved concerns,
and the document changes that the orchestrator must make.

Stop and report the problem, and do not guess, when:
- The operator did not answer Q1 or Q2.
- A recorded fact of this packet is not true in the working tree.
- `debug_bounds` gives no bounds for a row in the `deferred` overlay.
- An architecture guard rejects `.vertical_scrollbar(..)` outside the Music scroll helper.
- A change needs a file in "Do not touch".

## Operator Visual Check

The check needs a desktop session, the built app and a Library with one or more artists. V3
needs an artist with a long name, or MusicIndex for more names.

1. Build the app: `cd ~/build/v4vmm && cargo build --release`.
2. Start the app: `./target/release/v4vmm`.
3. Open the Music tab.
4. V1: click in the empty toolbar search field.
   - Look at: each row shows the field name, and under it the values and the example. Scroll the
     list to the bottom with the wheel. The rows `is:`, `playlist:` and `has:` show in full.
   - Wrong: a field name touches the line above it, a row shows only in part at the bottom, or
     `playlist:` or `has:` cannot be reached.
5. V2: type `haleen ` with the space.
   - Look at: the full list shows with the layout of step 4.
   - Wrong: the rows overlap.
6. V3: delete the text. Type `artist:`. Wait for the names.
   - Look at: no name touches the next row. A long name wraps, and its row grows. Its mark
     "Library" or "MusicIndex" stays in its own row.
   - Wrong: a mark or a second line of a name is drawn on the next row.
7. V4: delete the text. Press Down 11 times.
   - Look at: the selection moves to `has:`. The list scrolls, and `has:` shows in full.
   - Wrong: the selected row is out of view.
8. V5: in Settings, change the theme to Light, then to Dark. Make the window narrow, then wide.
   Do step 4 again each time.
   - Look at: the list stays in the window. When a scroll bar shows, it covers no text.
   - Wrong: a row is cut at the right edge, or the scroll bar covers text.

Cleanup:

1. Delete the text in the search field.
2. Set the theme and the window width back to their values before the check.
3. Close the app.

The check changes no file and no stored data.
