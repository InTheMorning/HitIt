//! Pure formatting helpers shared by view-models.
//!
//! These were previously defined inside `search.rs`. They are pure
//! string formatters with no GPUI dependency, so they belong in the
//! domain-adjacent layer where view-models can use them without
//! pulling in the entire screen module. `search.rs` now re-exports
//! these names for backward compatibility with the rest of the UI.

#![warn(clippy::pedantic)]

use chrono::{Datelike, FixedOffset, Offset, TimeZone, Timelike};
use icu_datetime::fieldsets::zone::LocalizedOffsetShort;
use icu_datetime::fieldsets::{MD, T, YMD};
use icu_datetime::input::{Date, Time, UtcOffset, ZonedDateTime};
use icu_datetime::{DateTimeFormatter, NoCalendarFormatter};
use icu_locale_core::Locale;

use crate::view_models::name_order::system_locale;

/// The text of a total time: `"41 min"` under one hour and `"1 h 12 min"`
/// from one hour (musicindex ADR 0023 R-DATE-1 item 3).
fn total_time_text(total_secs: i64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    if hours > 0 {
        format!("{hours} h {minutes} min")
    } else {
        format!("{minutes} min")
    }
}

/// Formats a total time in seconds as `"41 min"`, or as `"1 h 12 min"`
/// from one hour (R-DATE-1 item 3).
#[must_use]
pub fn fmt_runtime(total_secs: i32) -> String {
    total_time_text(i64::from(total_secs))
}

/// Formats a total time in seconds as `"41 min"`, or as `"1 h 12 min"`
/// from one hour (R-DATE-1 item 3). The result is `None` when the total is
/// zero or less. The screen then shows no duration row.
#[must_use]
pub fn fmt_total_time(total_secs: i64) -> Option<String> {
    (total_secs > 0).then(|| total_time_text(total_secs))
}

/// Formats the duration of a track as `"m:ss"`, or as `"h:mm:ss"` from one
/// hour (R-DATE-1 item 2). A negative duration gives `"0:00"`.
#[must_use]
pub fn fmt_duration(secs: i64) -> String {
    let secs = secs.max(0);
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// The UTC offset of the local time zone at `ts`, in seconds. A time out of
/// range gives zero. Unit tests use UTC, so that their dates do not depend on
/// the machine. The tests of the `_in` functions give an offset.
pub(crate) fn local_offset_secs(ts: i64) -> i32 {
    if cfg!(test) {
        return 0;
    }
    chrono::Local
        .timestamp_opt(ts, 0)
        .single()
        .map_or(0, |time| time.offset().fix().local_minus_utc())
}

/// The time `ts` in the time zone with the given offset.
fn local_time(ts: i64, offset_secs: i32) -> Option<chrono::DateTime<FixedOffset>> {
    Some(
        chrono::DateTime::from_timestamp(ts, 0)?
            .with_timezone(&FixedOffset::east_opt(offset_secs)?),
    )
}

/// The calendar day of a local time as an ICU date. The macro keeps the
/// calendar type of the date out of the signatures, because the app does
/// not depend on the calendar crate.
macro_rules! civil_date {
    ($local:expr) => {
        match (u8::try_from($local.month()), u8::try_from($local.day())) {
            (Ok(month), Ok(day)) => Date::try_new_iso($local.year(), month, day).ok(),
            _ => None,
        }
    };
}

thread_local! {
    /// The formatter of the medium date of the system language. Each thread
    /// builds its own, because the formatter is not `Sync`.
    static SYSTEM_DATE: Option<DateTimeFormatter<YMD>> =
        DateTimeFormatter::try_new((&system_locale()).into(), YMD::medium()).ok();

    /// The formatter of the day and the short month of the system language.
    static SYSTEM_MONTH_DAY: Option<DateTimeFormatter<MD>> =
        DateTimeFormatter::try_new((&system_locale()).into(), MD::medium()).ok();
}

/// Formats a Unix epoch timestamp as the medium date of the system
/// language, for example `"Nov 15, 2026"` in `en-US`. The day is the day in
/// the local time zone (R-DATE-1 item 1). Returns `None` if the timestamp
/// is out of range.
#[must_use]
pub fn fmt_date(ts: i64) -> Option<String> {
    let local = local_time(ts, local_offset_secs(ts))?;
    let date = civil_date!(local)?;
    SYSTEM_DATE.with(|formatter| Some(formatter.as_ref()?.format(&date).to_string()))
}

/// Formats a Unix epoch timestamp as the medium date of `locale`, for the
/// day in the time zone with the offset `offset_secs`. Tests give the
/// locale and the offset.
#[must_use]
pub fn fmt_date_in(ts: i64, locale: &Locale, offset_secs: i32) -> Option<String> {
    let local = local_time(ts, offset_secs)?;
    let date = civil_date!(local)?;
    let formatter = DateTimeFormatter::try_new(locale.into(), YMD::medium()).ok()?;
    Some(formatter.format(&date).to_string())
}

/// Formats a Unix epoch timestamp as the day and the short month of the
/// system language, with no year, for example `"Oct 8"` in `en-US`. The day
/// is the day in the local time zone, as in [`fmt_date`]. Returns `None`
/// when the timestamp is out of range (ADR 0086 Decision 3).
#[must_use]
pub fn fmt_month_day(ts: i64) -> Option<String> {
    let local = local_time(ts, local_offset_secs(ts))?;
    let date = civil_date!(local)?;
    SYSTEM_MONTH_DAY.with(|formatter| Some(formatter.as_ref()?.format(&date).to_string()))
}

/// Formats a Unix epoch timestamp as the day and the short month of
/// `locale`, for the day in the time zone with the offset `offset_secs`.
#[must_use]
pub fn fmt_month_day_in(ts: i64, locale: &Locale, offset_secs: i32) -> Option<String> {
    let local = local_time(ts, offset_secs)?;
    let date = civil_date!(local)?;
    let formatter = DateTimeFormatter::try_new(locale.into(), MD::medium()).ok()?;
    Some(formatter.format(&date).to_string())
}

/// Formats a Unix epoch timestamp as the medium date of `locale`, the time
/// in hours and minutes and the name of the time zone with the offset
/// `offset_secs`, for example `"Nov 15, 2026, 10:30 PM GMT+1"` in `en-US`.
#[must_use]
pub fn fmt_date_time_zone_in(ts: i64, locale: &Locale, offset_secs: i32) -> Option<String> {
    let local = local_time(ts, offset_secs)?;
    let date = civil_date!(local)?;
    let time = Time::try_new(
        u8::try_from(local.hour()).ok()?,
        u8::try_from(local.minute()).ok()?,
        u8::try_from(local.second()).ok()?,
        0,
    )
    .ok()?;
    let zone = UtcOffset::try_from_seconds(offset_secs).ok()?;
    let formatter = DateTimeFormatter::try_new(
        locale.into(),
        YMD::medium().with_time_hm().with_zone(LocalizedOffsetShort),
    )
    .ok()?;
    Some(
        formatter
            .format(&ZonedDateTime { date, time, zone })
            .to_string(),
    )
}

/// Formats a Unix epoch timestamp as the time of day of `locale`, with
/// hours, minutes and seconds, in the time zone with the offset
/// `offset_secs`. For example `"7:12:48 AM"` in `en-US` and `"07:12:48"` in
/// `de-DE`. Returns `None` when the timestamp is out of range.
///
/// musicindex ADR 0023 R-DATE-1 has no rule for a time of day yet. The
/// orchestrator choice of ADR 0101 task 008 applies the idea of item 1.
#[must_use]
pub fn fmt_time_in(ts: i64, locale: &Locale, offset_secs: i32) -> Option<String> {
    let local = local_time(ts, offset_secs)?;
    let time = Time::try_new(
        u8::try_from(local.hour()).ok()?,
        u8::try_from(local.minute()).ok()?,
        u8::try_from(local.second()).ok()?,
        0,
    )
    .ok()?;
    let formatter = NoCalendarFormatter::try_new(locale.into(), T::hms()).ok()?;
    Some(formatter.format(&time).to_string())
}

/// Formats a Unix epoch timestamp as the medium date, the time and the time
/// zone name of the system language and the local time zone.
#[must_use]
pub fn fmt_date_time_zone(ts: i64) -> Option<String> {
    fmt_date_time_zone_in(ts, &system_locale(), local_offset_secs(ts))
}

/// Push `(key, value)` into `rows` only when `value` is `Some` and not
/// empty. Lifted from the deleted `ui_common::optional_row`.
pub fn optional_row(rows: &mut Vec<(String, String)>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        if !value.is_empty() {
            rows.push((key.into(), value));
        }
    }
}

/// English plural suffix: empty for `1`, otherwise `"s"`. Lifted from
/// the deleted `ui_common::plural`.
#[must_use]
pub fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_runtime_under_one_hour() {
        assert_eq!(fmt_runtime(0), "0 min");
        assert_eq!(fmt_runtime(59), "0 min");
        assert_eq!(fmt_runtime(60), "1 min");
        assert_eq!(fmt_runtime(3599), "59 min");
    }

    #[test]
    fn fmt_runtime_one_hour_or_more() {
        assert_eq!(fmt_runtime(3600), "1 h 0 min");
        assert_eq!(fmt_runtime(3660), "1 h 1 min");
        assert_eq!(fmt_runtime(7325), "2 h 2 min");
    }

    /// 2026-11-15T22:30:00 UTC
    const NIGHT: i64 = 1_794_781_800;

    fn locale(text: &str) -> Locale {
        text.parse().expect("a locale")
    }

    /// R-DATE-1 item 1: the date is in the language of the reader.
    #[test]
    fn r_date_1_date_follows_the_locale() {
        assert_eq!(
            fmt_date_in(NIGHT, &locale("en-US"), 0).as_deref(),
            Some("Nov 15, 2026")
        );
        assert_eq!(
            fmt_date_in(NIGHT, &locale("fr-FR"), 0).as_deref(),
            Some("15 nov. 2026")
        );
        assert_eq!(fmt_date_in(i64::MAX, &locale("en-US"), 0), None);
    }

    /// R-DATE-1 item 1: the day is the day in the local time zone.
    #[test]
    fn r_date_1_day_is_the_local_day_near_midnight() {
        let english = locale("en-US");
        let hour = 3600;
        assert_eq!(
            fmt_date_in(NIGHT, &english, 0).as_deref(),
            Some("Nov 15, 2026")
        );
        assert_eq!(
            fmt_date_in(NIGHT, &english, 2 * hour).as_deref(),
            Some("Nov 16, 2026")
        );
        assert_eq!(
            fmt_date_in(NIGHT, &english, -23 * hour).as_deref(),
            Some("Nov 14, 2026")
        );
    }

    #[test]
    fn adr_0086_window_groups_fmt_month_day_gives_month_and_day() {
        assert_eq!(
            fmt_month_day_in(NIGHT, &locale("en-US"), 0).as_deref(),
            Some("Nov 15")
        );
        assert_eq!(
            fmt_month_day_in(NIGHT, &locale("fr-FR"), 0).as_deref(),
            Some("15 nov.")
        );
        assert_eq!(fmt_month_day_in(i64::MAX, &locale("en-US"), 0), None);
    }

    /// R-DATE-1 item 1: the live schedule has the date, the time and the
    /// time zone name.
    #[test]
    fn r_date_1_date_time_zone_has_the_zone_name() {
        let text = fmt_date_time_zone_in(NIGHT, &locale("en-US"), 3600).expect("a text");
        assert!(text.starts_with("Nov 15, 2026"), "{text}");
        assert!(text.contains("11:30"), "{text}");
        assert!(text.ends_with("GMT+1"), "{text}");
    }

    /// R-DATE-1 item 2.
    #[test]
    fn r_date_1_duration_is_minutes_or_hours() {
        assert_eq!(fmt_duration(187), "3:07");
        assert_eq!(fmt_duration(3723), "1:02:03");
        assert_eq!(fmt_duration(0), "0:00");
        assert_eq!(fmt_duration(-5), "0:00");
        assert_eq!(fmt_duration(3599), "59:59");
    }

    /// R-DATE-1 item 3.
    #[test]
    fn r_date_1_total_time_is_minutes_or_hours() {
        assert_eq!(fmt_total_time(2460).as_deref(), Some("41 min"));
        assert_eq!(fmt_total_time(4320).as_deref(), Some("1 h 12 min"));
        assert_eq!(fmt_total_time(3600).as_deref(), Some("1 h 0 min"));
        assert_eq!(fmt_total_time(0), None);
        assert_eq!(fmt_total_time(-1), None);
    }

    #[test]
    fn plural_returns_empty_for_one() {
        assert_eq!(plural(0), "s");
        assert_eq!(plural(1), "");
        assert_eq!(plural(2), "s");
        assert_eq!(plural(99), "s");
    }

    #[test]
    fn optional_row_skips_none_and_empty() {
        let mut rows: Vec<(String, String)> = Vec::new();
        optional_row(&mut rows, "A", None);
        optional_row(&mut rows, "B", Some(String::new()));
        optional_row(&mut rows, "C", Some("ok".into()));
        assert_eq!(rows, vec![("C".to_string(), "ok".to_string())]);
    }
}
