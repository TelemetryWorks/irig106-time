//! Unit tests for the `correlation` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `new_correlator_empty` | New correlator has 0 refs | L3-COR-002 |
//! | `add_reference_sorted` | References maintained in RTC order | L3-COR-003 |
//! | `correlate_exact_match` | RTC matching a ref returns that time | L3-COR-004 |
//! | `correlate_interpolation_forward` | RTC after ref extrapolates forward | L3-COR-004 |
//! | `correlate_nearest_point` | Uses closest ref of two | L3-COR-004 |
//! | `correlate_channel_filter` | Only uses refs from specified channel | L3-COR-005 |
//! | `correlate_no_ref_returns_error` | Empty correlator returns NoReferencePoint | L3-COR-002 |
//! | `correlate_no_channel_ref_returns_error` | No refs for channel returns error | L3-COR-005 |
//! | `detect_no_jump` | Stable time has no jumps | L3-COR-007 |
//! | `detect_gps_lock_jump` | Time jump detected after GPS lock | L3-COR-007 |
//! | `detect_jump_threshold` | Jump below threshold not flagged | L3-COR-007 |

use super::*;

fn abs(doy: u16, h: u8, m: u8, s: u8, ns: u32) -> AbsoluteTime {
    AbsoluteTime::new(doy, h, m, s, ns).unwrap()
}

#[test]
fn new_correlator_empty() {
    let c = TimeCorrelator::new();
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
}

#[test]
fn add_reference_sorted() {
    let mut c = TimeCorrelator::new();
    // Insert out of order
    c.add_reference(1, Rtc::from_raw(300), abs(1, 0, 0, 0, 0));
    c.add_reference(1, Rtc::from_raw(100), abs(1, 0, 0, 0, 0));
    c.add_reference(1, Rtc::from_raw(200), abs(1, 0, 0, 0, 0));

    let refs = c.references();
    assert_eq!(refs[0].rtc, Rtc::from_raw(100));
    assert_eq!(refs[1].rtc, Rtc::from_raw(200));
    assert_eq!(refs[2].rtc, Rtc::from_raw(300));
}

#[test]
fn correlate_exact_match() {
    let mut c = TimeCorrelator::new();
    let rtc = Rtc::from_raw(10_000_000); // 1 second
    let time = abs(100, 12, 30, 25, 0);
    c.add_reference(1, rtc, time);

    let result = c.correlate(rtc, None).unwrap();
    assert_eq!(result, time);
}

#[test]
fn correlate_interpolation_forward() {
    let mut c = TimeCorrelator::new();
    let ref_rtc = Rtc::from_raw(10_000_000);
    let ref_time = abs(100, 12, 30, 25, 0);
    c.add_reference(1, ref_rtc, ref_time);

    // Target: 15 ms later = 150_000 ticks later
    let target = Rtc::from_raw(10_150_000);
    let result = c.correlate(target, None).unwrap();

    // Expected: 12:30:25.015_000_000
    assert_eq!(result.hours, 12);
    assert_eq!(result.minutes, 30);
    assert_eq!(result.seconds, 25);
    assert_eq!(result.nanoseconds, 15_000_000);
}

#[test]
fn correlate_nearest_point() {
    let mut c = TimeCorrelator::new();
    // Ref 1 at RTC=1000, time=12:00:00.000
    c.add_reference(1, Rtc::from_raw(1_000), abs(1, 12, 0, 0, 0));
    // Ref 2 at RTC=100_000_000 (10 sec later), time=12:00:10.000
    c.add_reference(1, Rtc::from_raw(100_000_000), abs(1, 12, 0, 10, 0));

    // Target close to ref 2
    let target = Rtc::from_raw(99_999_000);
    let result = c.correlate(target, None).unwrap();
    // Should use ref 2 (closer), interpolate backward slightly
    assert_eq!(result.hours, 12);
    assert_eq!(result.minutes, 0);
    // 100 ticks = 10_000 ns = 10 µs before 10.0s
    assert_eq!(result.seconds, 9);
}

#[test]
fn correlate_channel_filter() {
    let mut c = TimeCorrelator::new();
    // Channel 1: RTC=1000, 12:00:00
    c.add_reference(1, Rtc::from_raw(1_000), abs(1, 12, 0, 0, 0));
    // Channel 2: RTC=2000, 13:00:00 (different time source)
    c.add_reference(2, Rtc::from_raw(2_000), abs(1, 13, 0, 0, 0));

    // Correlate using only channel 2
    let target = Rtc::from_raw(2_000);
    let result = c.correlate(target, Some(2)).unwrap();
    assert_eq!(result.hours, 13);

    // Correlate using only channel 1
    let result = c.correlate(Rtc::from_raw(1_000), Some(1)).unwrap();
    assert_eq!(result.hours, 12);
}

#[test]
fn correlate_no_ref_returns_error() {
    let c = TimeCorrelator::new();
    let result = c.correlate(Rtc::from_raw(100), None);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::NoReferencePoint => {}
        other => panic!("expected NoReferencePoint, got {other:?}"),
    }
}

#[test]
fn correlate_no_channel_ref_returns_error() {
    let mut c = TimeCorrelator::new();
    c.add_reference(1, Rtc::from_raw(1_000), abs(1, 12, 0, 0, 0));

    // Ask for channel 99 which has no refs
    let result = c.correlate(Rtc::from_raw(1_000), Some(99));
    assert!(result.is_err());
}

#[test]
fn detect_no_jump() {
    let mut c = TimeCorrelator::new();
    // Two refs 1 second apart, consistent
    let rtc1 = Rtc::from_raw(10_000_000);
    let rtc2 = Rtc::from_raw(20_000_000); // 1 sec later
    c.add_reference(1, rtc1, abs(1, 12, 0, 0, 0));
    c.add_reference(1, rtc2, abs(1, 12, 0, 1, 0)); // exactly 1 sec later

    let jumps = c.detect_time_jump(1, 1_000_000); // 1ms threshold
    assert!(jumps.is_empty());
}

#[test]
fn detect_gps_lock_jump() {
    let mut c = TimeCorrelator::new();
    // Before GPS lock: internal clock says 12:00:00
    c.add_reference(1, Rtc::from_raw(10_000_000), abs(1, 12, 0, 0, 0));
    // 1 second later by RTC, but GPS corrects to 12:00:05 (5 second jump)
    c.add_reference(1, Rtc::from_raw(20_000_000), abs(1, 12, 0, 5, 0));

    let jumps = c.detect_time_jump(1, 1_000_000_000); // 1 sec threshold
    assert_eq!(jumps.len(), 1);
    assert_eq!(jumps[0].channel_id, 1);
    // Expected: 12:00:01, Actual: 12:00:05 → delta = +4 sec = +4_000_000_000 ns
    assert!(jumps[0].delta_nanos > 0);
}

#[test]
fn detect_jump_threshold() {
    let mut c = TimeCorrelator::new();
    c.add_reference(1, Rtc::from_raw(10_000_000), abs(1, 12, 0, 0, 0));
    // 1 second later by RTC, absolute time is 1.0005 sec later (500 µs drift)
    c.add_reference(
        1,
        Rtc::from_raw(20_000_000),
        abs(1, 12, 0, 1, 500_000), // 500 µs extra
    );

    // With 1ms threshold, 500µs drift should NOT be flagged
    let jumps = c.detect_time_jump(1, 1_000_000);
    assert!(jumps.is_empty());

    // With 100µs threshold, it SHOULD be flagged
    let jumps = c.detect_time_jump(1, 100_000);
    assert_eq!(jumps.len(), 1);
}
