# musicindex ADR 0043 Task 001: Icons And Menu Entries

Status: Implemented on 2026-11-06. The visual check is open.

## Goal

The two icons of R-LIST-7 exist in `musicindex-assets` and v4vmm. A menu entry has the text color, and only its check mark has the accent color (R-LIST-6 item 3).

## Files To Inspect

`../musicindex-assets/icons/lucide/`, `src/ui/icons.rs`, `src/ui/primitives/context_menu.rs`, `src/ui/primitives/button.rs`.

## Decisions

- The primitive owns the entry color, so each menu follows. A destructive entry stays red. The durable rule "no isolated visual tweaks" gives this owner.
- The icons come from `lucide-static` 1.43.0, as `../musicindex-assets/AGENTS.md` states.

## Acceptance Criteria

Mechanical:

- `python3 scripts/check.py verify` passes in `musicindex-assets`, and `compare` passes for v4vmm.
- `IconName` has the two icons, and a test proves that each one resolves to a copied asset.
- A test proves that a menu entry that is not destructive has the `Label` color and that its check mark has `Accent`.

Visual: the new section of pending human checks.

## Test Commands

```bash
cargo test --lib context_menu
cargo test --lib icons
```

## Prompt for lower-context coding model

The planning model does this packet, because it changes a shared primitive and a shared repository.

## Escalation Triggers

`lucide-static` 1.43.0 does not have an icon of this name.
