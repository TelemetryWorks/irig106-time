use super::*;
use crate::absolute::AbsoluteTime;
use crate::rtc::Rtc;

#[test]
fn new_streaming_correlator_empty() {
    let sc = StreamingTimeCorrelator::new(10_000_000_000);
    assert!(sc.is_empty());
    assert_eq!(sc.len(), 0);
    assert_eq!(sc.total_evicted(), 0);
    assert!(sc.latest_rtc().is_none());
}

#[test]
fn add_and_correlate_single_ref() {
    let mut sc = StreamingTimeCorrelator::new(60_000_000_000);
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    assert_eq!(sc.len(), 1);

    let result = sc.correlate(Rtc::from_raw(10_000_000), None).unwrap();
    assert_eq!(result.hours, 12);
    assert_eq!(result.minutes, 0);
}

#[test]
fn correlate_by_channel() {
    let mut sc = StreamingTimeCorrelator::new(60_000_000_000);
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    sc.add_reference(
        2,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 13, 0, 0, 0).unwrap(),
    );

    let ch1 = sc.correlate(Rtc::from_raw(10_000_000), Some(1)).unwrap();
    assert_eq!(ch1.hours, 12);

    let ch2 = sc.correlate(Rtc::from_raw(20_000_000), Some(2)).unwrap();
    assert_eq!(ch2.hours, 13);
}

#[test]
fn correlate_no_ref_returns_error() {
    let sc = StreamingTimeCorrelator::new(60_000_000_000);
    assert!(sc.correlate(Rtc::from_raw(10_000_000), None).is_err());
}

#[test]
fn eviction_removes_stale_refs() {
    // Window of 10 seconds = 10_000_000_000 ns = 100_000_000 ticks
    let mut sc = StreamingTimeCorrelator::new(10_000_000_000);

    // Insert ref at RTC 10M (1 second)
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    assert_eq!(sc.len(), 1);

    // Insert ref at RTC 200M (20 seconds) — the first ref is now 19 seconds old
    // With a 10-second window, it should be evicted
    sc.add_reference(
        1,
        Rtc::from_raw(200_000_000),
        AbsoluteTime::new(100, 12, 0, 20, 0).unwrap(),
    );
    assert_eq!(sc.len(), 1); // old one evicted
    assert_eq!(sc.total_evicted(), 1);
}

#[test]
fn eviction_preserves_recent_refs() {
    let mut sc = StreamingTimeCorrelator::new(60_000_000_000); // 60 sec window

    // Insert 10 refs, 1 second apart
    for i in 0..10u64 {
        sc.add_reference(
            1,
            Rtc::from_raw((i + 1) * 10_000_000),
            AbsoluteTime::new(100, 12, 0, i as u8, 0).unwrap(),
        );
    }
    // All within 9 seconds — none evicted
    assert_eq!(sc.len(), 10);
    assert_eq!(sc.total_evicted(), 0);
}

#[test]
fn multi_channel_eviction() {
    let mut sc = StreamingTimeCorrelator::new(5_000_000_000); // 5 sec window

    // Channel 1 at RTC 10M
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    // Channel 2 at RTC 20M
    sc.add_reference(
        2,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );
    assert_eq!(sc.len(), 2);

    // Jump to RTC 200M — both should be evicted
    sc.add_reference(
        1,
        Rtc::from_raw(200_000_000),
        AbsoluteTime::new(100, 12, 0, 20, 0).unwrap(),
    );
    assert_eq!(sc.len(), 1); // only the new one
    assert_eq!(sc.total_evicted(), 2);
}

#[test]
fn channel_ids_reflects_active() {
    let mut sc = StreamingTimeCorrelator::new(60_000_000_000);
    sc.add_reference(
        5,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    sc.add_reference(
        3,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );

    let ids = sc.channel_ids();
    assert!(ids.contains(&3));
    assert!(ids.contains(&5));
    assert!(!ids.contains(&1));
}

#[test]
fn channel_len_per_channel() {
    let mut sc = StreamingTimeCorrelator::new(60_000_000_000);
    sc.add_reference(1, Rtc::from_raw(10_000_000), AbsoluteTime::new(100, 12, 0, 0, 0).unwrap());
    sc.add_reference(1, Rtc::from_raw(20_000_000), AbsoluteTime::new(100, 12, 0, 1, 0).unwrap());
    sc.add_reference(2, Rtc::from_raw(30_000_000), AbsoluteTime::new(100, 12, 0, 2, 0).unwrap());

    assert_eq!(sc.channel_len(1), 2);
    assert_eq!(sc.channel_len(2), 1);
    assert_eq!(sc.channel_len(99), 0);
}

#[test]
fn latest_rtc_tracks_maximum() {
    let mut sc = StreamingTimeCorrelator::new(60_000_000_000);
    sc.add_reference(1, Rtc::from_raw(50_000_000), AbsoluteTime::new(100, 12, 0, 0, 0).unwrap());
    assert_eq!(sc.latest_rtc(), Some(Rtc::from_raw(50_000_000)));

    // Earlier RTC doesn't update latest
    sc.add_reference(1, Rtc::from_raw(10_000_000), AbsoluteTime::new(100, 11, 0, 0, 0).unwrap());
    assert_eq!(sc.latest_rtc(), Some(Rtc::from_raw(50_000_000)));

    // Later RTC updates
    sc.add_reference(1, Rtc::from_raw(100_000_000), AbsoluteTime::new(100, 13, 0, 0, 0).unwrap());
    assert_eq!(sc.latest_rtc(), Some(Rtc::from_raw(100_000_000)));
}
