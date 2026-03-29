//! Optional interop with the `chrono` crate.
//!
//! Enable the `chrono` feature to get `From` conversions between
//! `AbsoluteTime` and `chrono::NaiveDateTime`.
//!
//! ```toml
//! [dependencies]
//! irig106-time = { version = "0.7", features = ["chrono"] }
//! ```
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | GAP-06      | chrono/time crate interop |

use crate::absolute::AbsoluteTime;

/// Convert `AbsoluteTime` to `chrono::NaiveDateTime`.
///
/// Requires the `year` field to be set. If year is `None`, January 1 of
/// year 1970 is assumed. Day-of-year is used for the conversion.
///
/// # Example
///
/// ```rust
/// use irig106_time::AbsoluteTime;
/// use irig106_time::chrono_interop; // ensure module is linked
/// use chrono::NaiveDateTime;
/// use chrono::Datelike;
///
/// let mut t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();
/// t.set_year(Some(2025));
/// let dt: NaiveDateTime = t.into();
/// assert_eq!(dt.year(), 2025);
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

/// Convert `chrono::NaiveDateTime` to `AbsoluteTime`.
///
/// Sets the year, day-of-year, and time-of-day fields. Month and
/// day-of-month are also populated from the chrono date.
impl From<chrono::NaiveDateTime> for AbsoluteTime {
    fn from(dt: chrono::NaiveDateTime) -> Self {
        use chrono::Datelike;
        use chrono::Timelike;
        let mut ns = dt.nanosecond();
        // Clamp nanoseconds that chrono may report >= 1B for leap seconds
        if ns >= 1_000_000_000 {
            ns = 999_999_999;
        }
        let mut abs = AbsoluteTime::new(
            dt.ordinal() as u16,
            dt.hour() as u8,
            dt.minute() as u8,
            dt.second() as u8,
            ns,
        )
        .unwrap();
        abs.set_year(Some(dt.year() as u16));
        abs.set_month(Some(dt.month() as u8));
        abs.set_day_of_month(Some(dt.day() as u8));
        abs
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
    fn chrono_to_absolute() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(); // day 100
        let time = NaiveTime::from_hms_nano_opt(12, 30, 25, 340_000_000).unwrap();
        let dt = NaiveDateTime::new(date, time);
        let abs: AbsoluteTime = dt.into();
        assert_eq!(abs.year(), Some(2025));
        assert_eq!(abs.day_of_year(), 100);
        assert_eq!(abs.hours(), 12);
        assert_eq!(abs.minutes(), 30);
        assert_eq!(abs.seconds(), 25);
        assert_eq!(abs.nanoseconds(), 340_000_000);
        assert_eq!(abs.month(), Some(4));
        assert_eq!(abs.day_of_month(), Some(10));
    }

    #[test]
    fn round_trip() {
        let mut original = AbsoluteTime::new(200, 8, 15, 45, 123_456_789).unwrap();
        original.set_year(Some(2024));
        let dt: NaiveDateTime = original.into();
        let back: AbsoluteTime = dt.into();
        assert_eq!(back.year(), Some(2024));
        assert_eq!(back.day_of_year(), 200);
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
}
