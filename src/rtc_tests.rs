//! Unit tests for the `rtc` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `zero_constant` | `Rtc::ZERO` is 0 | L3-RTC-004 |
//! | `max_constant` | `Rtc::MAX` is 2^48 − 1 | L3-RTC-005 |
//! | `from_le_bytes_all_zeros` | Zero bytes → zero RTC | L3-RTC-006 |
//! | `from_le_bytes_known_value` | Known byte pattern decodes correctly | L3-RTC-006 |
//! | `from_le_bytes_max` | All 0xFF bytes → MAX | L3-RTC-006 |
//! | `from_raw_masks_upper_bits` | Bits above 48 are cleared | L3-RTC-007 |
//! | `from_raw_preserves_lower` | Lower 48 bits preserved | L3-RTC-007 |
//! | `as_raw_round_trip` | `from_raw → as_raw` round-trips | L3-RTC-008 |
//! | `elapsed_ticks_simple` | Forward elapsed ticks | L3-RTC-009 |
//! | `elapsed_ticks_wrap_around` | 48-bit wrap-around handling | L3-RTC-009 |
//! | `elapsed_ticks_same_value` | Elapsed between equal RTC is 0 | L3-RTC-009 |
//! | `elapsed_nanos_simple` | Elapsed nanos = ticks × 100 | L3-RTC-010 |
//! | `to_nanos_zero` | Zero ticks → 0 ns | L3-RTC-011 |
//! | `to_nanos_one_tick` | 1 tick → 100 ns | L3-RTC-011 |
//! | `to_nanos_one_second` | 10M ticks → 1 second in ns | L3-RTC-011 |
//! | `ordering` | Rtc ordering by raw value | L3-RTC-012 |
//! | `debug_display` | Debug impl exists | L3-RTC-013 |
//! | `clone_copy_eq` | Clone, Copy, PartialEq, Eq | L3-RTC-013 |

use super::*;

#[test]
fn zero_constant() {
    assert_eq!(Rtc::ZERO.as_raw(), 0);
}

#[test]
fn max_constant() {
    assert_eq!(Rtc::MAX.as_raw(), 0x0000_FFFF_FFFF_FFFF);
}

#[test]
fn from_le_bytes_all_zeros() {
    let rtc = Rtc::from_le_bytes([0, 0, 0, 0, 0, 0]);
    assert_eq!(rtc, Rtc::ZERO);
}

#[test]
fn from_le_bytes_known_value() {
    // 0x00_00_01_00_00_00 in LE = [0x00, 0x00, 0x00, 0x01, 0x00, 0x00]
    // = 0x0000_0001_0000_00 = 16_777_216
    let rtc = Rtc::from_le_bytes([0x00, 0x00, 0x00, 0x01, 0x00, 0x00]);
    assert_eq!(rtc.as_raw(), 0x0000_0001_0000_00);
}

#[test]
fn from_le_bytes_max() {
    let rtc = Rtc::from_le_bytes([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(rtc, Rtc::MAX);
}

#[test]
fn from_raw_masks_upper_bits() {
    let rtc = Rtc::from_raw(0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(rtc.as_raw(), 0x0000_FFFF_FFFF_FFFF);
}

#[test]
fn from_raw_preserves_lower() {
    let rtc = Rtc::from_raw(12345);
    assert_eq!(rtc.as_raw(), 12345);
}

#[test]
fn as_raw_round_trip() {
    let val = 0x0000_ABCD_1234_5678;
    let rtc = Rtc::from_raw(val);
    assert_eq!(rtc.as_raw(), val);
}

#[test]
fn elapsed_ticks_simple() {
    let earlier = Rtc::from_raw(1_000_000);
    let later = Rtc::from_raw(2_000_000);
    assert_eq!(earlier.elapsed_ticks(later), 1_000_000);
}

#[test]
fn elapsed_ticks_wrap_around() {
    // Simulate 48-bit wrap: later has a smaller raw value due to counter overflow.
    let earlier = Rtc::from_raw(0x0000_FFFF_FFFF_FFF0); // Near MAX
    let later = Rtc::from_raw(0x0000_0000_0000_000F); // Wrapped around
    // Expected: 0x10 + 0x0F = 0x1F = 31 ticks
    assert_eq!(earlier.elapsed_ticks(later), 31);
}

#[test]
fn elapsed_ticks_same_value() {
    let rtc = Rtc::from_raw(42);
    assert_eq!(rtc.elapsed_ticks(rtc), 0);
}

#[test]
fn elapsed_nanos_simple() {
    let earlier = Rtc::from_raw(0);
    let later = Rtc::from_raw(10); // 10 ticks
    assert_eq!(earlier.elapsed_nanos(later), 1_000); // 10 * 100 ns
}

#[test]
fn to_nanos_zero() {
    assert_eq!(Rtc::ZERO.to_nanos(), 0);
}

#[test]
fn to_nanos_one_tick() {
    assert_eq!(Rtc::from_raw(1).to_nanos(), 100);
}

#[test]
fn to_nanos_one_second() {
    // 10 MHz = 10_000_000 ticks/sec → 10_000_000 * 100 ns = 1_000_000_000 ns = 1 sec
    let rtc = Rtc::from_raw(10_000_000);
    assert_eq!(rtc.to_nanos(), 1_000_000_000);
}

#[test]
fn ordering() {
    let a = Rtc::from_raw(100);
    let b = Rtc::from_raw(200);
    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, Rtc::from_raw(100));
}

#[test]
fn debug_display() {
    let rtc = Rtc::from_raw(42);
    let dbg = alloc::format!("{:?}", rtc);
    assert!(dbg.contains("42"));
}

extern crate alloc;

#[test]
fn clone_copy_eq() {
    let rtc = Rtc::from_raw(999);
    let cloned = rtc.clone();
    let copied = rtc; // Copy
    assert_eq!(rtc, cloned);
    assert_eq!(rtc, copied);
}
