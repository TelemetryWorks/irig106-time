//! Unit tests for the `bcd` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `day_fmt_decode_known` | Known DOY time decodes correctly | L3-BCD-003 |
//! | `day_fmt_midnight_day1` | Midnight DOY 001 | L3-BCD-003, L3-BCD-005 |
//! | `day_fmt_max_day` | DOY 366 at 23:59:59.990 | L3-BCD-003 |
//! | `day_fmt_invalid_bcd_digit` | Nibble > 9 is rejected | L3-BCD-001 |
//! | `day_fmt_reserved_bit_set` | Reserved bit non-zero rejected | L3-BCD-002 |
//! | `day_fmt_day_zero_rejected` | Day=0 rejected | L3-BCD-006 |
//! | `day_fmt_buffer_too_short` | <8 bytes rejected | L3-BCD-003 |
//! | `day_fmt_to_absolute` | Conversion to AbsoluteTime | L3-BCD-007 |
//! | `dmy_fmt_decode_known` | Known DMY time decodes correctly | L3-BCD-004 |
//! | `dmy_fmt_to_absolute_with_date` | Conversion includes year/month/day | L3-BCD-007 |
//! | `dmy_fmt_invalid_month_zero` | Month=0 rejected | L3-BCD-006 |
//! | `dmy_fmt_buffer_too_short` | <10 bytes rejected | L3-BCD-004 |
//! | `millisecond_resolution_10ms` | MS field has 10ms granularity | L3-BCD-005 |

use super::*;

/// Encode a known DOY time into the wire format for testing.
///
/// Day 100, 12:30:25.340
/// Word 0: Tmn=4, Hmn=3, Sn=5, TSn=2, rsv=0
///   = 0b0_010_0101_0011_0100 = 0x2534
/// Word 1: Mn=0, TMn=3, rsv=0, Hn=2, THn=1, rsv=0
///   = 0b00_01_0010_0_011_0000 = 0x1230
/// Word 2: Dn=0, TDn=0, HDn=1, rsv=0
///   = 0b000000_01_0000_0000 = 0x0100
/// Word 3: 0x0000
fn day100_123025_340_bytes() -> [u8; 8] {
    let w0: u16 = 0x2534;
    let w1: u16 = 0x1230;
    let w2: u16 = 0x0100;
    let w3: u16 = 0x0000;
    let mut buf = [0u8; 8];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[2..4].copy_from_slice(&w1.to_le_bytes());
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    buf[6..8].copy_from_slice(&w3.to_le_bytes());
    buf
}

#[test]
fn day_fmt_decode_known() {
    let buf = day100_123025_340_bytes();
    let t = DayFormatTime::from_le_bytes(&buf).unwrap();
    assert_eq!(t.day_of_year, 100);
    assert_eq!(t.hours, 12);
    assert_eq!(t.minutes, 30);
    assert_eq!(t.seconds, 25);
    assert_eq!(t.milliseconds, 340);
}

#[test]
fn day_fmt_midnight_day1() {
    // Day 001, 00:00:00.000
    // Word 0: all BCD digits 0 → 0x0000
    // Word 1: all BCD digits 0 → 0x0000
    // Word 2: Dn=1, TDn=0, HDn=0 → 0x0001
    let mut buf = [0u8; 8];
    buf[4] = 0x01; // w2 LE low byte: Dn=1
    let t = DayFormatTime::from_le_bytes(&buf).unwrap();
    assert_eq!(t.day_of_year, 1);
    assert_eq!(t.hours, 0);
    assert_eq!(t.minutes, 0);
    assert_eq!(t.seconds, 0);
    assert_eq!(t.milliseconds, 0);
}

#[test]
fn day_fmt_max_day() {
    // Day 366, 23:59:59.990
    // Word 0: Tmn=9, Hmn=9, Sn=9, TSn=5 → 0x5999
    // Word 1: Mn=9, TMn=5, Hn=3, THn=2 → 0x2359
    // Word 2: Dn=6, TDn=6, HDn=3 → 0x0366
    let w0: u16 = 0x5999;
    let w1: u16 = 0x2359;
    let w2: u16 = 0x0366;
    let mut buf = [0u8; 8];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[2..4].copy_from_slice(&w1.to_le_bytes());
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    let t = DayFormatTime::from_le_bytes(&buf).unwrap();
    assert_eq!(t.day_of_year, 366);
    assert_eq!(t.hours, 23);
    assert_eq!(t.minutes, 59);
    assert_eq!(t.seconds, 59);
    assert_eq!(t.milliseconds, 990);
}

#[test]
fn day_fmt_invalid_bcd_digit() {
    // Set Tmn nibble to 0xA (invalid BCD)
    let mut buf = [0u8; 8];
    buf[0] = 0x0A; // w0 low byte: Tmn=0xA
    buf[4] = 0x01; // valid day
    let result = DayFormatTime::from_le_bytes(&buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::InvalidBcdDigit { nibble, .. } => assert_eq!(nibble, 0xA),
        other => panic!("expected InvalidBcdDigit, got {other:?}"),
    }
}

#[test]
fn day_fmt_reserved_bit_set() {
    // Set word 0 bit 15 (reserved)
    let mut buf = [0u8; 8];
    buf[1] = 0x80; // w0 high byte bit 7 → word bit 15
    buf[4] = 0x01; // valid day
    let result = DayFormatTime::from_le_bytes(&buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::ReservedBitSet { .. } => {}
        other => panic!("expected ReservedBitSet, got {other:?}"),
    }
}

#[test]
fn day_fmt_day_zero_rejected() {
    // All zeros → day_of_year=0 → OutOfRange
    let buf = [0u8; 8];
    let result = DayFormatTime::from_le_bytes(&buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::OutOfRange { field, .. } => assert_eq!(field, "day_of_year"),
        other => panic!("expected OutOfRange for day_of_year, got {other:?}"),
    }
}

#[test]
fn day_fmt_buffer_too_short() {
    let buf = [0u8; 6];
    let result = DayFormatTime::from_le_bytes(&buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::BufferTooShort {
            expected: 8,
            actual: 6,
        } => {}
        other => panic!("expected BufferTooShort, got {other:?}"),
    }
}

#[test]
fn day_fmt_to_absolute() {
    let buf = day100_123025_340_bytes();
    let t = DayFormatTime::from_le_bytes(&buf).unwrap();
    let abs = t.to_absolute();
    assert_eq!(abs.day_of_year(), 100);
    assert_eq!(abs.hours(), 12);
    assert_eq!(abs.minutes(), 30);
    assert_eq!(abs.seconds(), 25);
    assert_eq!(abs.nanoseconds(), 340_000_000);
    assert_eq!(abs.month(), None);
}

#[test]
fn dmy_fmt_decode_known() {
    // March 15, 2025, 08:45:30.120
    // Word 0: Tmn=2, Hmn=1, Sn=0, TSn=3 → 0x3012
    // Word 1: Mn=5, TMn=4, Hn=8, THn=0 → 0x0845
    // Word 2: Dn=5, TDn=1, On=3, TOn=0 → 0x0315
    // Word 3: Yn=5, TYn=2, HYn=0, OYn=2 → 0x2025
    let w0: u16 = 0x3012;
    let w1: u16 = 0x0845;
    let w2: u16 = 0x0315;
    let w3: u16 = 0x2025;
    let mut buf = [0u8; 10];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[2..4].copy_from_slice(&w1.to_le_bytes());
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    buf[6..8].copy_from_slice(&w3.to_le_bytes());
    let t = DmyFormatTime::from_le_bytes(&buf).unwrap();
    assert_eq!(t.year, 2025);
    assert_eq!(t.month, 3);
    assert_eq!(t.day, 15);
    assert_eq!(t.hours, 8);
    assert_eq!(t.minutes, 45);
    assert_eq!(t.seconds, 30);
    assert_eq!(t.milliseconds, 120);
}

#[test]
fn dmy_fmt_to_absolute_with_date() {
    let w0: u16 = 0x3012;
    let w1: u16 = 0x0845;
    let w2: u16 = 0x0315;
    let w3: u16 = 0x2025;
    let mut buf = [0u8; 10];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[2..4].copy_from_slice(&w1.to_le_bytes());
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    buf[6..8].copy_from_slice(&w3.to_le_bytes());
    let t = DmyFormatTime::from_le_bytes(&buf).unwrap();
    let abs = t.to_absolute();
    assert_eq!(abs.year(), Some(2025));
    assert_eq!(abs.month(), Some(3));
    assert_eq!(abs.day_of_month(), Some(15));
    // March 15 in non-leap 2025 = 31+28+15 = 74
    assert_eq!(abs.day_of_year(), 74);
}

#[test]
fn dmy_fmt_invalid_month_zero() {
    // Month = 0 via BCD = On=0, TOn=0
    let w2: u16 = 0x0015; // Dn=5, TDn=1, On=0, TOn=0
    let w3: u16 = 0x2025;
    let mut buf = [0u8; 10];
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    buf[6..8].copy_from_slice(&w3.to_le_bytes());
    let result = DmyFormatTime::from_le_bytes(&buf);
    assert!(result.is_err());
}

#[test]
fn dmy_fmt_buffer_too_short() {
    let buf = [0u8; 8];
    assert!(DmyFormatTime::from_le_bytes(&buf).is_err());
}

#[test]
fn millisecond_resolution_10ms() {
    // Tmn=5 (tens of ms), Hmn=2 (hundreds of ms) → 250 ms
    // Word 0 low nibbles: [3:0]=5, [7:4]=2
    let w0: u16 = 0x0025; // Sn=0, TSn=0, Hmn=2, Tmn=5
    let mut buf = [0u8; 8];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[4] = 0x01; // day=1
    let t = DayFormatTime::from_le_bytes(&buf).unwrap();
    assert_eq!(t.milliseconds, 250);
}
