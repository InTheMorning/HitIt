# musicindex ADR 0042 Task 002: The Neutral Role

Status: Implemented on 2026-11-06. The visual check is open.

## Goal

The `Button` primitive draws R-ACT-5, and a centered plain button with a text label is Neutral (R-ACT-6).

## Files To Inspect

`src/ui/primitives/button.rs`, `src/ui/control_styles.rs`, `src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/contrast.rs`.

## Files Likely To Change

The files above, and `src/ui/composites/action_button.rs` for its comment.

## Do Not Touch

Screens and view models. Packet 003 owns them.

## Constraints

- Keep the id, the focus and the activation behavior of each button.
- Do not add a raw color. The tokens own each value.

## Implementation Steps

1. Add `SemanticColor::ButtonFill` and `ButtonFillHover` with the values of `tokens.json`.
2. Add `ButtonVariant::Neutral`: the `Label` color on `ButtonFill`, with `ButtonFillHover` for the pointer.
3. In the primitive, a `Plain` button with a text label and centered content draws as Neutral. A role `Unfilled` is the exception (R-ACT-6 item 2).
4. In the primitive, a `DestructiveRowAction` with a text label and centered content draws as Destructive.
5. Rename the role `Ghost` to `Neutral`. Remove the accent foreground of `Neutral`, `RowAction` and `ToolbarIcon`. An icon button keeps the accent from `Plain`.
6. Add the role `CompactAction`: `Tinted`, `Sm`, `Caption`.
7. Remove the border and the accent of `MetadataAction`.

## Acceptance Criteria

Mechanical:

- A test compares `ButtonFill` and `ButtonFillHover` with `tokens.json`.
- The contrast test covers the Neutral label on each surface in each profile, at rest and with the pointer.
- A test proves that each role with a text label has a fill, except `Unfilled`, and that a menu item has none.

Visual: section 12 of [pending human checks](../pending-human-checks.md).

## Test Commands

```bash
cargo test --lib button
cargo test --lib contrast
cargo test --lib tokens
```

## Prompt for lower-context coding model

The planning model does this packet, because it decides the owner of the role.

## Escalation Triggers

A test that fails for a reason outside this packet.
