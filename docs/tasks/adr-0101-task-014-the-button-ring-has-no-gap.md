# ADR 0101 Task 014: The Button Ring Touches The Button

Status: Ready, 2026-11-16.

## Goal

The focus ring of a button touches the shown button. Today a dark space shows between the ring and the button.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 1: the focus ring is 2 px with no space between the ring and the control. musicindex ADR 0073 R-FOCUS-2 gives this ring to HitIt.
- ADR 0069 (situational guard `adr_0069_keyboard_buttons_show_focus_without_layout_shift`): the focus ring does not change the layout.
- musicindex ADR 0026 R-FOCUS-1: only keyboard input shows the ring.

Operator choice line, 2026-11-16: this is a defect correction of ADR 0101 Decision 1. The agent walk of [ADR 0101 task 001](adr-0101-task-001-shared-tokens.md) V1 found it.

## Recorded Facts

- 2026-11-16, agent walk on Xvfb, debug binary, Dark, M: on "Download album" of an Index release page, a dark space of approximately 4 px is between the ring and the button fill.
- `Button::render` (`src/ui/primitives/button.rs`) makes two boxes. The hit target (`hit_target_base`) has `min_h(min_hit_target)`, 44 px at M. The shown button (`visual_base`) has `h(self.height(cx))`, which is smaller.
- `keyboard_button_focus` puts a transparent border of `CONTROL_FOCUS_RING_WIDTH` on the hit target and sets its color to `SemanticColor::Focus` in `.focus(...)`. Thus the ring is at the edge of the hit target, not at the edge of the shown button.
- `focus_for_reveal` calculates `focus.is_focused(window) && focus_ring_shows(cx)` at each render.
- `tests/architecture_tests.rs` has two source guards that read the text of `keyboard_button_focus`: `adr_0069_keyboard_buttons_show_focus_without_layout_shift` and an ADR 0066 guard that requires `button.track_focus(&handle.tab_index(tab_index).tab_stop(true))`.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Button focus ring | extend | src/ui/primitives/button.rs | `keyboard_button_focus` keeps the Tab stop and the key context. The ring moves to the shown button. |
| Keyboard ring state | reuse | src/ui/focus_ring.rs | `focus_ring_shows` tells if keyboard input shows the ring (R-FOCUS-1). |
| Ring width | reuse | src/ui/layouts.rs | `CONTROL_FOCUS_RING_WIDTH`, scaled with `layout::scaled_dimension`. |

## Required Changes

1. Paint the ring on the shown button. When the button has the focus and `focus_ring_shows(cx)` gives `true`, the shown button gets one child: an absolute layer with `inset_0`, the radius of the button, and a border of `CONTROL_FOCUS_RING_WIDTH` (scaled) in `SemanticColor::Focus` for the appearance. The shown button gets `relative()`. The layer takes no pointer input.
2. Remove the ring color from the hit target. Keep the dimensions of the hit target and of the shown button, so no layout moves. If the transparent border of the hit target goes, give the cause in the report and prove that no fit test changes.
3. Keep the Tab stop, `tab_index`, the key context and the activation of `keyboard_button_focus` as they are.
4. Update the two source guards in `tests/architecture_tests.rs` for the new code. Each guard keeps its function: the ring uses `CONTROL_FOCUS_RING_WIDTH` and `SemanticColor::Focus`, the ring does not change the layout, and a supplied focus handle stays a Tab stop. The failure message names ADR 0101 Decision 1 and the fix.
5. Add a gpui test in `src/ui/primitives/button.rs`. Focus a button with the keyboard. The bounds of the ring layer are equal to the bounds of the shown button. Focus it with a click. No ring layer shows.

## Do Not Change

- No change to the ring of a list row or a tile. They have their own code.
- No change to the button dimensions, the hit target dimensions or the button colors.
- No change to `CONTROL_FOCUS_RING_WIDTH` or to the focus color.

## Acceptance Criteria

Mechanical:

- M1: a test proves that the ring layer of a button with keyboard focus has the bounds of the shown button.
- M2: a test proves that a click focus shows no ring layer.
- M3: the source guards of ADR 0069 and ADR 0066 pass with the new code.
- M4: the fit tests of ADR 0097 pass.
- M5: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, listed apart:

- V1: on an Index release page, Tab to "Download album". The ring touches the blue fill, with no dark space.
- V2: Tab to "Website" and to an icon button such as "⋯". The ring touches each button.

## Test Commands

- `cargo test --lib ui::primitives::button`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests added. Tell if the hit target keeps its transparent border.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if the ring hides a necessary border of a button style. Give the style.
- Stop if the ring layer changes the dimensions of a button in a fit test.
