//! Unit tests for the `absolute` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `absolute_time_valid` | Constructor accepts valid ranges | L3-ABS-002 |
//! | `absolute_time_day_zero_rejected` | day_of_year=0 is invalid | L3-ABS-002 |
//! | `absolute_time_day_367_rejected` | day_of_year=367 is invalid | L3-ABS-002 |
//! | `absolute_time_hours_24_rejected` | hours=24 is invalid | L3-ABS-002 |
//! | `absolute_time_minutes_60_rejected` | minutes=60 is invalid | L3-ABS-002 |
//! | `absolute_time_seconds_60_rejected` | seconds=60 is invalid | L3-ABS-002 |
//! | `absolute_time_nanos_overflow_rejected` | nanos=1B is invalid | L3-ABS-002 |
//! | `calendar_time_valid` | CalendarTime construction with validated date | L3-ABS-006 |
//! | `calendar_time_month_zero_rejected` | month=0 is invalid | L3-ABS-006 |
//! | `calendar_time_month_13_rejected` | month=13 is invalid | L3-ABS-006 |
//! | `calendar_time_requires_year` | AbsoluteTime without year is rejected | L3-ABS-006 |
//! | `calendar_time_from_parts` | from_parts convenience constructor | L3-ABS-006 |
//! | `calendar_time_into_absolute_time` | Convert CalendarTime to AbsoluteTime | L3-ABS-006 |
//! | `calendar_time_display` | Display formats YYYY-MM-DD | L3-ABS-006 |
//! | `calendar_time_feb_31_rejected` | Feb 31 is never valid | L3-ABS-006 |
//! | `calendar_time_feb_29_non_leap_rejected` | Feb 29 on non-leap year rejected | L3-ABS-006 |
//! | `calendar_time_feb_29_leap_accepted` | Feb 29 on leap year accepted | L3-ABS-006 |
//! | `calendar_time_apr_31_rejected` | Apr 31 is never valid | L3-ABS-006 |
//! | `calendar_time_doy_mismatch_rejected` | DOY inconsistent with date rejected | L3-ABS-006 |
//! | `calendar_time_doy_consistent` | DOY consistent with date accepted | L3-ABS-006 |
//! | `with_year_rejects_invalid` | Year > 9999 rejected | L3-ABS-002 |
//! | `with_year_accepts_valid_range` | Year 0-9999 and None accepted | L3-ABS-002 |
//! | `with_year_10000_rejected` | Year 10000 rejected | L3-ABS-002 |
//! | `add_nanos_subsecond` | Add nanos within same second | L3-ABS-004 |
//! | `add_nanos_carry_to_seconds` | Carry from nanos to seconds | L3-ABS-004 |
//! | `add_nanos_carry_to_minutes` | Carry from seconds to minutes | L3-ABS-004 |
//! | `add_nanos_carry_to_hours` | Carry to hours | L3-ABS-004 |
//! | `add_nanos_carry_to_days` | Carry to next day | L3-ABS-004 |
//! | `total_nanos_of_day_midnight` | Midnight = 0 ns | L3-ABS-005 |
//! | `total_nanos_of_day_noon` | Noon = 12h in ns | L3-ABS-005 |
//! | `ieee1588_from_le_bytes` | Parse IEEE-1588 from buffer | L3-1588-002 |
//! | `ieee1588_nanos_overflow_rejected` | nanos >= 1B rejected | L3-1588-004 |
//! | `ieee1588_to_nanos_since_epoch` | Conversion to total ns | L3-1588-003 |
//! | `ertc_from_le_bytes` | Parse ERTC from buffer | L3-ERTC-002 |
//! | `ertc_to_nanos` | Conversion to ns (u128) | L3-ERTC-003 |
//! | `ertc_max_no_overflow` | MAX u64 doesn't overflow u128 | L3-ERTC-003 |

use super::*;

#[test]
fn absolute_time_valid() {
    let t = AbsoluteTime::new(100, 12, 30, 45, 500_000_000).unwrap();
    assert_eq!(t.day_of_year(), 100);
    assert_eq!(t.hours(), 12);
    assert_eq!(t.minutes(), 30);
    assert_eq!(t.seconds(), 45);
    assert_eq!(t.nanoseconds(), 500_000_000);
}

#[test]
fn absolute_time_day_zero_rejected() {
    assert!(AbsoluteTime::new(0, 0, 0, 0, 0).is_err());
}

#[test]
fn absolute_time_day_367_rejected() {
    assert!(AbsoluteTime::new(367, 0, 0, 0, 0).is_err());
}

#[test]
fn absolute_time_hours_24_rejected() {
    assert!(AbsoluteTime::new(1, 24, 0, 0, 0).is_err());
}

#[test]
fn absolute_time_minutes_60_rejected() {
    assert!(AbsoluteTime::new(1, 0, 60, 0, 0).is_err());
}

#[test]
fn absolute_time_seconds_60_rejected() {
    assert!(AbsoluteTime::new(1, 0, 0, 60, 0).is_err());
}

#[test]
fn absolute_time_nanos_overflow_rejected() {
    assert!(AbsoluteTime::new(1, 0, 0, 0, 1_000_000_000).is_err());
}

#[test]
fn calendar_time_valid() {
    let t = AbsoluteTime::new(45, 10, 30, 0, 0)
        .unwrap()
        .with_year(Some(2025))
        .unwrap();
    let ct = super::CalendarTime::new(t, 2, 14).unwrap();
    assert_eq!(ct.year(), Some(2025));
    assert_eq!(ct.month(), 2);
    assert_eq!(ct.day_of_month(), 14);
    // Deref gives access to AbsoluteTime methods
    assert_eq!(ct.hours(), 10);
    assert_eq!(ct.day_of_year(), 45);
}

#[test]
fn calendar_time_month_zero_rejected() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0)
        .unwrap()
        .with_year(Some(2025))
        .unwrap();
    assert!(super::CalendarTime::new(t, 0, 1).is_err());
}

#[test]
fn calendar_time_month_13_rejected() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0)
        .unwrap()
        .with_year(Some(2025))
        .unwrap();
    assert!(super::CalendarTime::new(t, 13, 1).is_err());
}

#[test]
fn calendar_time_requires_year() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    // No year set — CalendarTime requires it
    assert!(super::CalendarTime::new(t, 1, 1).is_err());
}

#[test]
fn calendar_time_from_parts() {
    let ct = super::CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 0).unwrap();
    assert_eq!(ct.year(), Some(2025));
    assert_eq!(ct.month(), 4);
    assert_eq!(ct.day_of_month(), 10);
    assert_eq!(ct.day_of_year(), 100);
    assert_eq!(ct.hours(), 12);
}

#[test]
fn calendar_time_into_absolute_time() {
    let ct = super::CalendarTime::from_parts(2025, 4, 10, 100, 12, 0, 0, 0).unwrap();
    let abs: AbsoluteTime = ct.into();
    assert_eq!(abs.year(), Some(2025));
    assert_eq!(abs.day_of_year(), 100);
    // AbsoluteTime has no month() or day_of_month() — type safety enforced
}

#[test]
fn calendar_time_display() {
    use alloc::format;
    let ct = super::CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap();
    let s = format!("{}", ct);
    assert!(s.starts_with("2025-04-10"));
    assert!(s.contains("12:30:25"));
}

// ── Calendar correctness validation (team review regression tests) ──

#[test]
fn calendar_time_feb_31_rejected() {
    // 2025-02-31 does not exist — February has 28 days in 2025
    let result = super::CalendarTime::from_parts(2025, 2, 31, 59, 12, 0, 0, 0);
    assert!(result.is_err());
}

#[test]
fn calendar_time_feb_29_non_leap_rejected() {
    // 2025-02-29 does not exist — 2025 is not a leap year
    let result = super::CalendarTime::from_parts(2025, 2, 29, 60, 12, 0, 0, 0);
    assert!(result.is_err());
}

#[test]
fn calendar_time_feb_29_leap_accepted() {
    // 2024-02-29 is valid — 2024 is a leap year. DOY = 31 + 29 = 60
    let ct = super::CalendarTime::from_parts(2024, 2, 29, 60, 12, 0, 0, 0).unwrap();
    assert_eq!(ct.month(), 2);
    assert_eq!(ct.day_of_month(), 29);
    assert_eq!(ct.day_of_year(), 60);
}

#[test]
fn calendar_time_apr_31_rejected() {
    // 2025-04-31 does not exist — April has 30 days
    let result = super::CalendarTime::from_parts(2025, 4, 31, 121, 12, 0, 0, 0);
    assert!(result.is_err());
}

#[test]
fn calendar_time_doy_mismatch_rejected() {
    // 2025-04-10 = DOY 100, but we claim DOY 42 — rejected
    let result = super::CalendarTime::from_parts(2025, 4, 10, 42, 12, 0, 0, 0);
    assert!(result.is_err());
}

#[test]
fn calendar_time_doy_consistent() {
    // 2025-04-10 = DOY 100 — matches
    let ct = super::CalendarTime::from_parts(2025, 4, 10, 100, 12, 0, 0, 0).unwrap();
    assert_eq!(ct.day_of_year(), 100);
    assert_eq!(ct.month(), 4);
    assert_eq!(ct.day_of_month(), 10);
}

#[test]
fn with_year_rejects_invalid() {
    // u16::MAX = 65535, which exceeds 9999
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    assert!(t.with_year(Some(65_535)).is_err());
}

#[test]
fn with_year_accepts_valid_range() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    let t = t.with_year(Some(0)).unwrap();
    assert_eq!(t.year(), Some(0));
    let t = t.with_year(Some(9999)).unwrap();
    assert_eq!(t.year(), Some(9999));
    let t = t.with_year(None).unwrap();
    assert_eq!(t.year(), None);
}

#[test]
fn with_year_10000_rejected() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    assert!(t.with_year(Some(10_000)).is_err());
}

#[test]
fn add_nanos_subsecond() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 100_000_000).unwrap();
    let t2 = t.add_nanos(200_000_000);
    assert_eq!(t2.seconds(), 0);
    assert_eq!(t2.nanoseconds(), 300_000_000);
}

#[test]
fn add_nanos_carry_to_seconds() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 900_000_000).unwrap();
    let t2 = t.add_nanos(200_000_000); // 1.1 seconds total
    assert_eq!(t2.seconds(), 1);
    assert_eq!(t2.nanoseconds(), 100_000_000);
}

#[test]
fn add_nanos_carry_to_minutes() {
    let t = AbsoluteTime::new(1, 0, 0, 59, 0).unwrap();
    let t2 = t.add_nanos(1_000_000_000); // +1 second → 60 secs → 1 min
    assert_eq!(t2.minutes(), 1);
    assert_eq!(t2.seconds(), 0);
}

#[test]
fn add_nanos_carry_to_hours() {
    let t = AbsoluteTime::new(1, 0, 59, 59, 0).unwrap();
    let t2 = t.add_nanos(1_000_000_000); // +1 sec → 1:00:00
    assert_eq!(t2.hours(), 1);
    assert_eq!(t2.minutes(), 0);
    assert_eq!(t2.seconds(), 0);
}

#[test]
fn add_nanos_carry_to_days() {
    let t = AbsoluteTime::new(1, 23, 59, 59, 0).unwrap();
    let t2 = t.add_nanos(1_000_000_000); // +1 sec → next day
    assert_eq!(t2.day_of_year(), 2);
    assert_eq!(t2.hours(), 0);
    assert_eq!(t2.minutes(), 0);
    assert_eq!(t2.seconds(), 0);
}

#[test]
fn total_nanos_of_day_midnight() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    assert_eq!(t.total_nanos_of_day(), 0);
}

#[test]
fn total_nanos_of_day_noon() {
    let t = AbsoluteTime::new(1, 12, 0, 0, 0).unwrap();
    assert_eq!(t.total_nanos_of_day(), 12 * 3600 * 1_000_000_000);
}

#[test]
fn ieee1588_from_le_bytes() {
    // 500_000_000 ns = 0x1DCD_6500, 1000 seconds = 0x0000_03E8
    let buf: [u8; 8] = [0x00, 0x65, 0xCD, 0x1D, 0xE8, 0x03, 0x00, 0x00];
    let t = Ieee1588Time::from_le_bytes(&buf).unwrap();
    assert_eq!(t.nanoseconds, 500_000_000);
    assert_eq!(t.seconds, 1000);
}

#[test]
fn ieee1588_nanos_overflow_rejected() {
    // nanoseconds = 1_000_000_000 (too large)
    let buf: [u8; 8] = [0x00, 0xCA, 0x9A, 0x3B, 0x00, 0x00, 0x00, 0x00];
    assert!(Ieee1588Time::from_le_bytes(&buf).is_err());
}

#[test]
fn ieee1588_to_nanos_since_epoch() {
    let t = Ieee1588Time {
        nanoseconds: 500_000_000,
        seconds: 10,
    };
    assert_eq!(t.to_nanos_since_epoch(), 10_500_000_000);
}

#[test]
fn ertc_from_le_bytes() {
    let buf: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let e = Ertc::from_le_bytes(&buf).unwrap();
    assert_eq!(e.as_raw(), 1);
}

#[test]
fn ertc_to_nanos() {
    let e = Ertc::from_le_bytes(&[10, 0, 0, 0, 0, 0, 0, 0]).unwrap();
    assert_eq!(e.to_nanos(), 1_000); // 10 * 100
}

#[test]
fn ertc_max_no_overflow() {
    let buf = [0xFF; 8];
    let e = Ertc::from_le_bytes(&buf).unwrap();
    let nanos = e.to_nanos();
    assert!(nanos > 0);
    // u64::MAX * 100 fits in u128
    assert_eq!(nanos, (u64::MAX as u128) * 100);
}
