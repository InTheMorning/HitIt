# ADR 0101 Task 010: The Selected Look

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks. [ADR 0093 task 001](adr-0093-task-001-settings-tab-row-and-rows.md) comes after this packet.

## Goal

A selected sidebar item, a selected row in a list and a selected segment or tab have one look. The look is a gradient, a 1 px inner edge and a 1 px inner highlight at the top. The label keeps the text color in semibold. Only an icon has the accent color.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decisions 2, 11 and 12 own this packet.
- musicindex ADR 0051 R-SELECT-1 (`../musicindex/docs/adr/0051-one-segmented-control-and-one-selected-look.md` lines 55 to 67, at musicindex `785ae28`):
  1. The item has a gradient from `selected-top` to `selected-bottom`. It has a 1 px inner edge in `selected-edge`, and a 1 px inner highlight at the top in `selected-highlight`.
  2. The look has no other fill and no outer shadow.
  3. A label keeps the text color and has the semibold weight. Only an icon has the accent color.
  4. Only this look shows that an item of these kinds is selected. A filled button is never a selected state (ADR 0069 R-ACT-1 item 3).
  5. This look does not apply to an icon button. ADR 0043 R-LIST-7 item 8 gives the current view icon.
- musicindex ADR 0073 R-FOCUS-2 and ADR 0101 Decision 1: the focus ring is 2 px with no gap, an inset border. It shows over the selected look.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `src/assets/tokens.json` lines 20 to 23 and 43 to 46 give `selected-top`, `selected-bottom`, `selected-edge` and `selected-highlight` for dark and light. Examined. No Rust code reads them.
- `ControlStyle::SelectedTab` (`src/ui/control_styles.rs:129`) is `ButtonVariant::Tinted` with `AccentOutline`. Examined. Its callers: `src/ui/composites/segmented_control.rs:125`, `src/ui/composites/tab_bar.rs:71` and `:101`, `src/ui/composites/letter_bar.rs:115`, `src/ui/composites/list_page_head.rs:316`, `src/ui/composites/settings.rs:144` and `src/ui/composites/maintenance_page.rs:120`.
- `ListRow` (`src/ui/composites/list_row.rs:348`) gives a selected row a fill and an accent border. The border is 2 px and always takes its space. Focus gives the focus color. Examined.
- The sidebar (`src/ui/shells/library/sidebar.rs:79`) gives a selected item `AccentTint` and `AccentOutline`. Focus replaces the outline color. Examined.
- Other selected fills: `src/ui/composites/selectable_tile.rs:151`, `src/ui/shells/playlist.rs:885` and `src/ui/composites/search_field_list.rs:272`.
- GPUI 0.3.1 has two stops in a gradient (ADR 0101 Decision 4).
- Tests: `selected_tab_has_a_tint_and_an_outline` (`src/ui/control_styles.rs:232`) and `adr_0084_selection_focus_outline_shows_on_each_focused_row` (`src/ui/composites/list_row.rs:663`).

Main session choice, 2026-11-15: a selected tile is not an item of R-SELECT-1. It keeps its look, and the packet reports it. A checked row of a selection is not a selected row. It keeps its check box (musicindex ADR 0063).

Main session choice, 2026-11-15: a high-contrast profile gives the four colors from its own values. The gradient has two equal stops, and the edge is opaque.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Color tokens | extend | `src/ui/tokens.rs` | `SemanticColor` gives each color. Add `SelectedTop`, `SelectedBottom`, `SelectedEdge` and `SelectedHighlight`. |
| Selected look | new | `src/ui/primitives/selected_look.rs` | Searched `src/ui/primitives/`, `src/ui/composites/` and `src/ui/style.rs` in HitIt. Each caller draws its own tint and outline. One function must give the look to each caller. |
| Control styles | extend | `src/ui/control_styles.rs` | `SelectedTab` gives the selected segment and tab. Change it to the look. |
| List row | extend | `src/ui/composites/list_row.rs` | It draws a selected row. Use the look. |
| Sidebar | extend | `src/ui/shells/library/sidebar.rs` | It draws a selected item. Use the look. |
| Segmented control | extend | `src/ui/composites/segmented_control.rs` | It draws a selected segment through `SelectedTab`. |

## Required Changes

1. **The tokens.** Add the four color tokens in each theme profile, with the values of `tokens.json`. A test compares them.
2. **The look.** One function gives the look to an element: the gradient, the 1 px inner edge and the 1 px inner highlight at the top. It gives no other fill and no shadow.
3. **The label.** A selected label has the text color and the semibold weight. A selected icon has the accent color.
4. **The callers.** Each caller of `SelectedTab`, the list row, the sidebar, the playlist row and the field list row use the look. Report each one.
5. **The focus.** The focus ring shows over the look. A row keeps its border space, so the look and the ring move no row.
6. **The guard.** A guard proves that no file other than the look function uses `AccentTint` or `AccentOutline` for a selected state. Its message names R-SELECT-1 and the fix. Delete `selected_tab_has_a_tint_and_an_outline`.

## Do Not Change

- No change of a tile, a check box or an icon button.
- No change of the focus ring.
- No change of the accent color.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test proves that the four tokens of each theme equal `tokens.json`.
- M2: a test proves that the look gives the gradient, the edge and the highlight, and no other fill.
- M3: a test proves that `SelectedTab` gives the look, the text color and the semibold weight, and is not a filled button.
- M4: a test of the list row proves that a selected and focused row shows the focus ring over the look.
- M5: the guard of step 6 passes.
- M6: the fit tests of ADR 0097 pass.
- M7: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark, Light and the two high-contrast profiles at M:

- V1: the selected sidebar item, a selected list row and a selected segment have the same look.
- V2: the label of a selected item has the text color, not the accent.
- V3: Tab to a selected row. The focus ring shows fully over the look.

## Test Commands

- `cargo test --lib ui::primitives::selected_look`
- `cargo test --lib ui::control_styles`
- `cargo test --lib ui::composites::list_row`
- `cargo test --lib ui::tokens`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M5 by name. Give each caller of step 4, and each selected look that you did not change with the cause.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the inner highlight needs a third gradient stop.
- Stop if a caller uses the selected look for a state that is not a selection.
