use super::*;
use crate::absolute::AbsoluteTime;
use crate::correlation::ReferencePoint;
use crate::rtc::Rtc;
use alloc::vec;

fn make_ref(ch: u16, rtc_raw: u64, doy: u16, h: u8, m: u8, s: u8) -> ReferencePoint {
    ReferencePoint {
        channel_id: ch,
        rtc: Rtc::from_raw(rtc_raw),
        time: AbsoluteTime::new(doy, h, m, s, 0).unwrap(),
    }
}

#[test]
fn empty_refs() {
    let q = compute_quality(&[]);
    assert_eq!(q.total_refs, 0);
    assert_eq!(q.channel_count, 0);
    assert!(q.max_rtc_gap_ns.is_none());
    assert!(q.ref_density_per_sec.is_none());
    assert!(q.rtc_span_ns.is_none());
}

#[test]
fn single_ref() {
    let refs = [make_ref(1, 10_000_000, 100, 12, 0, 0)];
    let q = compute_quality(&refs);
    assert_eq!(q.total_refs, 1);
    assert_eq!(q.channel_count, 1);
    assert!(q.max_rtc_gap_ns.is_none());
    assert!(q.ref_density_per_sec.is_none());
    assert!(q.rtc_span_ns.is_none());
}

#[test]
fn two_refs_one_channel() {
    let refs = [
        make_ref(1, 10_000_000, 100, 12, 0, 0),
        make_ref(1, 20_000_000, 100, 12, 0, 1),
    ];
    let q = compute_quality(&refs);
    assert_eq!(q.total_refs, 2);
    assert_eq!(q.channel_count, 1);
    assert_eq!(q.refs_per_channel, vec![(1, 2)]);
    // 10M ticks = 1 second = 1_000_000_000 ns
    assert_eq!(q.max_rtc_gap_ns, Some(1_000_000_000));
    assert_eq!(q.min_rtc_gap_ns, Some(1_000_000_000));
    assert_eq!(q.rtc_span_ns, Some(1_000_000_000));
    // 2 refs over 1 second = 2.0 refs/sec
    assert!((q.ref_density_per_sec.unwrap() - 2.0).abs() < 0.01);
}

#[test]
fn multi_channel() {
    let refs = [
        make_ref(1, 10_000_000, 100, 12, 0, 0),
        make_ref(2, 15_000_000, 100, 12, 0, 0),
        make_ref(1, 20_000_000, 100, 12, 0, 1),
        make_ref(2, 25_000_000, 100, 12, 0, 1),
    ];
    let q = compute_quality(&refs);
    assert_eq!(q.total_refs, 4);
    assert_eq!(q.channel_count, 2);
}

#[test]
fn drift_ppm_populated_for_multi_ref_channels() {
    // Perfect sync: 1 second RTC = 1 second absolute
    let refs = [
        make_ref(1, 10_000_000, 100, 12, 0, 0),
        make_ref(1, 20_000_000, 100, 12, 0, 1),
    ];
    let q = compute_quality(&refs);
    assert_eq!(q.drift_ppm_per_channel.len(), 1);
    assert!(q.drift_ppm_per_channel[0].1.abs() < 0.01);
}

#[test]
fn drift_ppm_not_populated_for_single_ref_channel() {
    let refs = [make_ref(1, 10_000_000, 100, 12, 0, 0)];
    let q = compute_quality(&refs);
    assert!(q.drift_ppm_per_channel.is_empty());
}
