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
//! | `with_date_valid` | Attach DMY fields | L3-ABS-003 |
//! | `with_date_month_zero_rejected` | month=0 is invalid | L3-ABS-003 |
//! | `with_date_month_13_rejected` | month=13 is invalid | L3-ABS-003 |
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
    assert_eq!(t.month(), None);
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
fn with_date_valid() {
    let t = AbsoluteTime::new(45, 10, 30, 0, 0)
        .unwrap()
        .with_date(2025, 2, 14)
        .unwrap();
    assert_eq!(t.year(), Some(2025));
    assert_eq!(t.month(), Some(2));
    assert_eq!(t.day_of_month(), Some(14));
}

#[test]
fn with_date_month_zero_rejected() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    assert!(t.with_date(2025, 0, 1).is_err());
}

#[test]
fn with_date_month_13_rejected() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    assert!(t.with_date(2025, 13, 1).is_err());
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
