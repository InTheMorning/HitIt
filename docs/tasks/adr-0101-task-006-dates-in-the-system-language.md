# ADR 0101 Task 006: Dates In The System Language

Status: Implemented - operator visual checks open, 2026-11-16.

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

Each fact is from HitIt `ba43138`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

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

Operator choice, 2026-11-15: HitIt uses `icu_datetime` at `~2.1`, the family of `icu_collator`, with its compiled data. A date is the medium date length of the locale, for example "Nov 15, 2026" in `en-US` and "15 nov. 2026" in `fr-FR`. The day is the day in the local time zone of the system.

Operator choice, 2026-11-15: the report times stay. A recorded time of a report, for example `2026-11-15 22:27:35 UTC`, is not the date of an item. (open point 3 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points)). The live schedule of `src/view_models/live_page.rs:228` is the date of an item, so it changes.

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

## Implementation Report - 2026-11-16

The change is in the working tree. It is not committed.

Files changed: `Cargo.toml`, `Cargo.lock`, `src/view_models/format.rs`, `src/view_models/name_order.rs`, `src/view_models/live_page.rs`, `src/view_models/track.rs`, `src/view_models/library.rs`, `src/view_models/queue_now_playing.rs`, `src/view_models/playlist_rss_check.rs`, `src/view_models/show.rs`, `src/metadata.rs`, `tests/architecture_tests.rs`.

- `Cargo.toml` names `icu_datetime` at `~2.1`. `icu_locale_core` stays at 2.1.1. The feature `clock` of `chrono` is on, because the local time zone needs it. The crate `iana-time-zone` was already in `Cargo.lock`.
- `system_locale` and `locale_from_read` are in `name_order.rs`. A failed read gives `en-US`.
- `format.rs` has `fmt_date`, `fmt_month_day`, `fmt_date_time_zone`, `fmt_duration` and `fmt_total_time`. The `_in` variants take a locale and a UTC offset for tests.
- Unit tests use `en-US` and UTC through `cfg!(test)`, so that existing date tests do not depend on the machine.

Tests: M1 `r_date_1_date_follows_the_locale`. M2 `r_date_1_day_is_the_local_day_near_midnight`. M3 `r_date_1_duration_is_minutes_or_hours`. M4 `r_date_1_total_time_is_minutes_or_hours`. M5 `r_date_1_locale_read_falls_back_to_en_us`. M6 `adr_0101_dates_guard_no_view_model_formats_a_month_name_outside_format_rs`.

Text changes: `fmt_total_runtime_clock` is now `fmt_total_time`. The playlist and album totals change from "5:05" and "1h 23m" to "5 min" and "1 h 23 min". The now-playing duration was a track duration. It uses `fmt_duration` and keeps "m:ss". The live schedule changes from "Mar 13, 01:00 UTC" to the date of the locale, the time and a GMT offset name. A track of one hour or more changes from "62:05" to "1:02:05".

Percentages in `src/view_models/`: one, `value_route_split_label` in `track_metadata_grid.rs`. It gives the split with "%" and no decimal place. Task 004 owns it, so it is not changed. `src/metadata.rs:878` has "{}% local" for a match score. It is outside `src/view_models/` and is not changed.

Open points: a zero offset gives "GMT+0" and not "GMT" in the live schedule. `src/view_models/show.rs` has `%m-%d %H:%M` date texts that this guard does not cover.

Release binary: 95633448 bytes with the change, built in a separate folder. The base `target/release/hitit` is 92857984 bytes from an earlier commit. The growth is 2775464 bytes (2.65 MiB).

Checks: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

## Primary Agent Check - 2026-11-16

- The primary agent read the diff and ran the gate: fmt, clippy, 3174 lib tests, 413 architecture tests and `cargo build --bin hitit` pass.
- The `clock` feature of `chrono` was on before this task, through a different dependency. The lock file adds only the `icu_datetime` family.
- Unit tests use `en-US` and UTC through `cfg!(test)` in `system_locale` and `local_offset_secs`. The tests of the `_in` functions give the locale and the offset.
- The live schedule shows "GMT+0" for an offset of zero. This is the text of the ICU zone format.
- Open point: `src/view_models/show.rs` has two `%m-%d %H:%M` patterns. They have no month name, so the guard does not include them. This packet does not include them.
- Agent walk on the Xvfb display, debug binary, `LANG=fr_FR.UTF-8`: the Releases list shows "4 nov. to 13 nov.", and the release page "Waltz Tunes Volume 2" shows "12 nov. 2026" and "1 h 16 min". V2: Pass. Before this task, the English page showed "Nov 13, 2026". After this task, the day is the local day, and the time zone of this machine is west of UTC.
