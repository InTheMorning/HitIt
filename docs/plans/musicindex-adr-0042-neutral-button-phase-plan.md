# musicindex ADR 0042 Neutral Button Phase Plan

Status: Open. Started 2026-11-06. The packets are implemented, and the visual check is open.

Owner of the rules: musicindex ADR 0042 R-ACT-4 to R-ACT-6. This plan states no rule.

## Goal

v4vmm gets the neutral button of Studio. Each button with a text label has a fill and one of four roles.

## Non-Goals

- The website changes. Its light values need a deploy, and the operator did not select them in this run.
- A change to the red token or the search field radius.
- A change to icon buttons.

## Assumptions

- A second agent edits many of the same files. The work stays small, and each edit reads the file first.
- The operator commits. This run makes no commit.

## Affected Modules

- `src/assets/tokens.json`, `src/ui/tokens.rs`, `src/ui/theme_profiles.rs`, `src/ui/contrast.rs`
- `src/ui/control_styles.rs`, `src/ui/primitives/button.rs`
- The call sites that packet 003 lists

## Sequence

| Packet | Content | Worker |
|---|---|---|
| [001](../tasks/musicindex-adr-0042-task-001-shared-tokens.md) | The tokens in `musicindex-assets` and each app copy | Planning model |
| [002](../tasks/musicindex-adr-0042-task-002-neutral-role.md) | The tokens in Rust, the role, the primitive and the guards | Planning model |
| [003](../tasks/musicindex-adr-0042-task-003-call-site-roles.md) | The role of each call site by its result | Coding model |

Packet 002 changes most buttons from one owner, so packet 003 changes only the sites where the result of the command sets the role.

## Schema And API Implications

None.

## Risk Areas

- A wider button can move to a new row. The test of ADR 0074 found this for the pill shape.
- A test can fail because of the work of the second agent. Examine each failed test before a change.

## Test Strategy

Mechanical: the full gate, the token test and the contrast test. Visual: section 12 of [pending human checks](../pending-human-checks.md).

## Rollback Strategy

Revert the working tree changes of the packets. The stored data does not change.

## Open Questions

None.
