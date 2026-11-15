# ADR 0101 Task 006: Dates In The System Language

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks. The operator must approve the new crate `icu_datetime` before the first edit.

## Goal

The date of an item shows in the language of the system, with the day, the short month name and the year. One formatter gives each date, each track duration and each total time. A percentage has one decimal place.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decisions 11 and 12 own this packet.
- musicindex ADR 0023 R-DATE-1 (`../musicindex/docs/adr/0023-cross-app-words-links-and-messages.md` lines 101 to 109, at musicindex `785ae28`):
  1. A date is formatted in the language of the reader, which for HitIt is the language of the system. It shows the day, the short month name and the year.
  2. The duration of a track is `m:ss`, or `h:mm:ss` from one hour.
  3. A total time is "41 min", or "1 h 12 min" from one hour.
  4. A percentage has one decimal place (R-VALUE-1).
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) owns the dates of the new releases window groups (item map, ADR 0086 Decision 3).
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `fmt_date` (`src/view_models/format.rs:28`) gives `%b %-d, %Y` in UTC. `fmt_month_day` (`:36`) gives `%b %-d`. Examined.
- `src/metadata.rs:4038` has a second `fmt_date` with the same pattern. Examined.
- `fmt_total_runtime_clock` (`src/view_models/format.rs:55`) gives `M:SS` under one hour and `Hh Mm` from one hour. Examined. R-DATE-1 item 3 gives "41 min" and "1 h 12 min".
- `src/metadata.rs` near line 4030 gives "{h} h {m} min" and "{m} min". Line 4005 gives `m:ss`. Examined.
- Other `m:ss` formatters: `src/view_models/track.rs:21`, `src/view_models/playlist_rss_check.rs:553`, `src/view_models/library.rs:6244` and `:7115`. Examined. None gives `h:mm:ss` from one hour.
- The callers of `fmt_date` and `fmt_month_day`: `src/view_models/track_detail.rs:296`, `src/view_models/entity_detail.rs:803`, `src/view_models/library.rs:7321` and `:7326`, `src/view_models/singles_group.rs:1048`, `:1060` and `:1349`, and `src/view_models/window_groups.rs:337` to `:342`.
- `src/view_models/live_page.rs:228` gives a schedule as `%b %-d, %H:%M UTC`.
- `src/view_models/name_order.rs:46` reads the system language with `sys_locale::get_locale()` and parses it into an `icu_locale_core::Locale`. Examined. `Cargo.toml` names `icu_collator` and `icu_locale_core` at `~2.1` and `sys-locale` at 0.3.
- `Cargo.lock` has no `icu_datetime`. Examined.
- Tests pin the English text: `fmt_date_renders_ymd` (`src/view_models/format.rs:108`) and `adr_0086_window_groups_fmt_month_day_gives_month_and_day` (`:114`).

Main session choice, 2026-11-15: HitIt uses `icu_datetime` at `~2.1`, the family of `icu_collator`, with its compiled data. A date is the medium date length of the locale, for example "Nov 15, 2026" in `en-US` and "15 nov. 2026" in `fr-FR`. The day is the day in the local time zone of the system.

Main session choice, 2026-11-15: the report times stay. A recorded time of a report, for example `2026-11-15 22:27:35 UTC`, is not the date of an item. The report owners (ADRs 0059 and 0063) decide a change (open point 3 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points)). The live schedule of `src/view_models/live_page.rs:228` is the date of an item, so it changes.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Date formatters | extend | `src/view_models/format.rs` | `fmt_date` and `fmt_month_day` are the shared formatters. Change them there. |
| System language | extend | `src/view_models/name_order.rs` | It reads the system locale. Move the read to one function that both modules use. |
| Total time formatter | extend | `src/view_models/format.rs` | `fmt_total_runtime_clock` is the shared total time. Change its text to R-DATE-1 item 3. |
| Duration formatter | extend | `src/view_models/format.rs` | Add one `m:ss` and `h:mm:ss` formatter. The copies call it. |
| Second date copy | extend | `src/metadata.rs` | Its `fmt_date` and duration copies call the shared formatters. |

## Required Changes

1. **The locale.** One function gives the system locale, with `en-US` when the read fails. `name_order.rs` and the date formatter use it. Tests give the locale as an argument.
2. **The date.** `fmt_date` gives the medium date of the locale in the local time zone. `fmt_month_day` gives the day and the short month of the locale, with no year. Keep their names and their `Option` result.
3. **The duration.** One function gives `m:ss`, or `h:mm:ss` from one hour. Each `m:ss` copy of "Recorded Facts" calls it.
4. **The total time.** `fmt_total_runtime_clock` gives "41 min", or "1 h 12 min" from one hour. The copy in `src/metadata.rs` calls it. Rename it to a name that says total time, and change each caller.
5. **The live schedule.** The schedule of the live page gives the date of the locale and the time in the local time zone, with the time zone name.
6. **The percentage.** Find each percentage text in `src/view_models/`. Each one has one decimal place. Report each one. Task 004 owns the payment split text.
7. **The tests.** Change the English tests so that they give the locale. Add the tests of M1 to M5.

## Do Not Change

- No change of a report time, a log time or a stored date.
- No change of the date parse in `src/metadata.rs:2769` and `:3390`. It reads tags.
- No new crate other than `icu_datetime` at `~2.1`.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test gives one date in `en-US` and in `fr-FR`, and proves the two texts.
- M2: a test proves that the day is the local day near midnight, with a fixed time zone.
- M3: a test gives `3:07` for 187 s and `1:02:03` for 3723 s.
- M4: a test gives "41 min" for 2460 s and "1 h 12 min" for 4320 s.
- M5: a test proves that the locale read gives `en-US` when it fails.
- M6: a guard proves that no file in `src/view_models/` other than `format.rs` calls `format("%b`. Its message names R-DATE-1 and the fix.
- M7: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator:

- V1: with the system language English, a release page shows its date as before.
- V2: start HitIt with `LANG=fr_FR.UTF-8`. A release page shows the date in French.
- V3: a playlist of more than one hour shows "1 h 12 min" and not "1h 12m".

## Test Commands

- `cargo test --lib view_models::format`
- `cargo test --lib view_models::name_order`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M6 by name. Give each caller that changed text, and each percentage of step 6. Give the size change of the release binary after the new crate.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if `icu_datetime` `~2.1` needs a different version of `icu_locale_core`.
- Stop if the release binary grows by more than 5 MB.
- Stop if a caller uses the date text as a key or stores it.
