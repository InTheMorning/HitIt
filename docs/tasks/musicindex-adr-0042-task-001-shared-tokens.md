# musicindex ADR 0042 Task 001: Shared Tokens

Status: Complete on 2026-11-06.

## Goal

`button-fill` and `button-fill-hover` are in `tokens/tokens.json` of `musicindex-assets`, and each app copy has the same bytes.

## Acceptance Criteria

Mechanical:

- `python3 scripts/check.py verify` in `musicindex-assets` passes.
- `python3 scripts/check.py compare` passes for the Studio and HitIt copies.
- The Studio tests `text-styles.test.mjs` and `type-dot.test.mjs` pass.

Visual: none.

## Result

Each check passed. The website keeps its CSS. Its check passed with the new file.
