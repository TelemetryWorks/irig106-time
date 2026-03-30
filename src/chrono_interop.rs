//! Optional interop with the `chrono` crate.
//!
//! Enable the `chrono` feature to get `From` conversions between
//! `AbsoluteTime`, `CalendarTime`, and `chrono::NaiveDateTime`.
//!
//! ```toml
//! [dependencies]
//! irig106-time = { version = "0.7", features = ["chrono"] }
//! ```
//!
//! # Conversion Summary
//!
//! | From | To | Notes |
//! |------|----|-------|
//! | `AbsoluteTime` | `NaiveDateTime` | Uses DOY. Year defaults to 1970 if absent. |
//! | `CalendarTime` | `NaiveDateTime` | Uses month/day directly. |
//! | `NaiveDateTime` | `CalendarTime` | chrono always provides full date. |
//!
//! If you only have an `AbsoluteTime` and need a `NaiveDateTime`, the
//! conversion uses day-of-year (which is always present). If you need
//! to go from `NaiveDateTime` back to `AbsoluteTime`, convert through
//! `CalendarTime` first: `let abs: AbsoluteTime = CalendarTime::from(dt).into();`
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | GAP-06      | chrono/time crate interop |

use crate::absolute::{AbsoluteTime, CalendarTime};

/// Convert `AbsoluteTime` to `chrono::NaiveDateTime`.
///
/// Uses day-of-year for the date component. If year is `None`, January 1 of
/// year 1970 is assumed.
///
/// # Example
///
/// ```rust
/// use irig106_time::AbsoluteTime;
/// use chrono::NaiveDateTime;
/// use chrono::Datelike;
///
/// let mut t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();
/// t.set_year(Some(2025));
/// let dt: NaiveDateTime = t.into();
/// assert_eq!(dt.year(), 2025);
/// assert_eq!(dt.ordinal(), 100);
/// ```
impl From<AbsoluteTime> for chrono::NaiveDateTime {
    fn from(abs: AbsoluteTime) -> Self {
        let year = abs.year().unwrap_or(1970) as i32;
        let date = chrono::NaiveDate::from_yo_opt(year, abs.day_of_year() as u32)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
        let time = chrono::NaiveTime::from_hms_nano_opt(
            abs.hours() as u32,
            abs.minutes() as u32,
            abs.seconds() as u32,
            abs.nanoseconds(),
        )
        .unwrap_or_default();
        chrono::NaiveDateTime::new(date, time)
    }
}

/// Convert `CalendarTime` to `chrono::NaiveDateTime`.
///
/// Uses the validated year, month, and day-of-month directly.
///
/// # Example
///
/// ```rust
/// use irig106_time::CalendarTime;
/// use chrono::NaiveDateTime;
/// use chrono::Datelike;
///
/// let ct = CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 0).unwrap();
/// let dt: NaiveDateTime = ct.into();
/// assert_eq!(dt.year(), 2025);
/// assert_eq!(dt.month(), 4);
/// assert_eq!(dt.day(), 10);
/// ```
impl From<CalendarTime> for chrono::NaiveDateTime {
    fn from(ct: CalendarTime) -> Self {
        let year = ct.year().unwrap_or(1970) as i32;
        let date =
            chrono::NaiveDate::from_ymd_opt(year, ct.month() as u32, ct.day_of_month() as u32)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap());
        let time = chrono::NaiveTime::from_hms_nano_opt(
            ct.hours() as u32,
            ct.minutes() as u32,
            ct.seconds() as u32,
            ct.nanoseconds(),
        )
        .unwrap_or_default();
        chrono::NaiveDateTime::new(date, time)
    }
}

/// Convert `chrono::NaiveDateTime` to `CalendarTime`.
///
/// chrono always provides year, month, and day, so this always produces a
/// fully validated `CalendarTime`. To get an `AbsoluteTime` instead, use
/// `.into_absolute_time()` on the result.
impl From<chrono::NaiveDateTime> for CalendarTime {
    fn from(dt: chrono::NaiveDateTime) -> Self {
        use chrono::Datelike;
        use chrono::Timelike;
        let mut ns = dt.nanosecond();
        if ns >= 1_000_000_000 {
            ns = 999_999_999;
        }
        CalendarTime::from_parts(
            dt.year() as u16,
            dt.month() as u8,
            dt.day() as u8,
            dt.ordinal() as u16,
            dt.hour() as u8,
            dt.minute() as u8,
            dt.second() as u8,
            ns,
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

    #[test]
    fn absolute_to_chrono() {
        let mut t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();
        t.set_year(Some(2025));
        let dt: NaiveDateTime = t.into();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.ordinal(), 100);
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 25);
        assert_eq!(dt.nanosecond(), 340_000_000);
    }

    #[test]
    fn calendar_to_chrono() {
        let ct = CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap();
        let dt: NaiveDateTime = ct.into();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 4);
        assert_eq!(dt.day(), 10);
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 25);
        assert_eq!(dt.nanosecond(), 340_000_000);
    }

    #[test]
    fn chrono_to_calendar_time() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 10).unwrap();
        let time = NaiveTime::from_hms_nano_opt(12, 30, 25, 340_000_000).unwrap();
        let dt = NaiveDateTime::new(date, time);
        let ct: CalendarTime = dt.into();
        assert_eq!(ct.year(), Some(2025));
        assert_eq!(ct.day_of_year(), 100);
        assert_eq!(ct.month(), 4);
        assert_eq!(ct.day_of_month(), 10);
        assert_eq!(ct.hours(), 12);
        assert_eq!(ct.minutes(), 30);
        assert_eq!(ct.seconds(), 25);
        assert_eq!(ct.nanoseconds(), 340_000_000);
    }

    #[test]
    fn round_trip_via_calendar_time() {
        let ct = CalendarTime::from_parts(2024, 7, 18, 200, 8, 15, 45, 123_456_789).unwrap();
        let dt: NaiveDateTime = ct.into();
        let back: CalendarTime = dt.into();
        assert_eq!(back.year(), Some(2024));
        assert_eq!(back.day_of_year(), 200);
        assert_eq!(back.month(), 7);
        assert_eq!(back.day_of_month(), 18);
        assert_eq!(back.hours(), 8);
        assert_eq!(back.minutes(), 15);
        assert_eq!(back.seconds(), 45);
        assert_eq!(back.nanoseconds(), 123_456_789);
    }

    #[test]
    fn no_year_defaults_to_1970() {
        let t = AbsoluteTime::new(50, 6, 0, 0, 0).unwrap();
        let dt: NaiveDateTime = t.into();
        assert_eq!(dt.year(), 1970);
        assert_eq!(dt.ordinal(), 50);
    }

    #[test]
    fn calendar_to_absolute_drops_month_day() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 10).unwrap();
        let time = NaiveTime::from_hms_nano_opt(12, 0, 0, 0).unwrap();
        let dt = NaiveDateTime::new(date, time);
        let ct: CalendarTime = dt.into();
        let abs: AbsoluteTime = ct.into();
        assert_eq!(abs.year(), Some(2025));
        assert_eq!(abs.day_of_year(), 100);
        // AbsoluteTime has no month() or day_of_month() — type safety enforced
    }
}
