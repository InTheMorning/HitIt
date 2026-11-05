# ADR 0080 Task 015: A Write Replaces Each Other `TPUB` Or `TLAN` Value

Status: Complete 2026-11-05. Mechanical checks Green. It has no visual check.

## Goal

A frame that holds a confirmed label or a language does not grow. A write replaces each other value.

## Authority

- [ADR 0080](../../adr/0080-tag-frames-follow-their-owner.md) Decisions 12 and 13, amended 2026-11-05.
- Open point 1 of the [run record](../../reviews/orchestrated-run-2026-11-05.md). The operator decided: "overwrite for consistancy, do not bloat the id3 with forever-growing number of fields".

## Recorded Facts - 2026-11-05

- The writer replaces the `TPUB` and `TLAN` keys. Two tests recorded this result as an open point.
- Commit `402300a` kept a `TPUB` value from another source. The operator rejected that rule, and this packet removes the code.

## Required Changes

1. The writer stays as it was before `402300a`.
2. The two tests state the rulings of Decisions 12 and 13.

## Do Not Change

- With no confirmed label, a write does not change `TPUB`.
- No change to the compare rows.

## Acceptance Criteria

Mechanical:

- `adr_0080_label_write_over_a_foreign_tpub_value_on_mp3_flac_and_mp4`: the file holds only the confirmed label.
- `adr_0080_language_write_over_a_foreign_tlan_value_on_mp3_flac_and_mp4`: the file holds only the new code.
- The commit checks of AGENTS.md pass.

Visual: none. The packet changes no screen, and the tests read the written files.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release`
- `cargo build --bin v4vmm`
