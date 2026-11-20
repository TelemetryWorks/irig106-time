use super::*;
use crate::absolute::AbsoluteTime;
use crate::rtc::Rtc;
use alloc::string::ToString;
use irig106_types::NanosDuration;

#[test]
fn new_streaming_correlator_empty() {
    let sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(10).expect("bounded literal duration"),
    );
    assert!(sc.is_empty());
    assert_eq!(sc.len(), 0);
    assert_eq!(sc.total_evicted(), 0);
    assert!(sc.latest_rtc().is_none());
}

#[test]
fn add_and_correlate_single_ref() {
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    );
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    assert_eq!(sc.len(), 1);

    let result = sc.correlate(Rtc::from_raw(10_000_000), None).unwrap();
    assert_eq!(result.hours(), 12);
    assert_eq!(result.minutes(), 0);
}

#[test]
fn correlate_by_channel() {
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    );
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
    assert_eq!(ch1.hours(), 12);

    let ch2 = sc.correlate(Rtc::from_raw(20_000_000), Some(2)).unwrap();
    assert_eq!(ch2.hours(), 13);
}

#[test]
fn correlate_no_ref_returns_error() {
    let sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    );
    assert!(sc.correlate(Rtc::from_raw(10_000_000), None).is_err());
}

#[test]
fn eviction_removes_stale_refs() {
    // Window of 10 seconds = 10_000_000_000 ns = 100_000_000 ticks
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(10).expect("bounded literal duration"),
    );

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
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    ); // 60 sec window

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
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(5).expect("bounded literal duration"),
    ); // 5 sec window

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
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    );
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
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    );
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    sc.add_reference(
        1,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );
    sc.add_reference(
        2,
        Rtc::from_raw(30_000_000),
        AbsoluteTime::new(100, 12, 0, 2, 0).unwrap(),
    );

    assert_eq!(sc.channel_len(1), 2);
    assert_eq!(sc.channel_len(2), 1);
    assert_eq!(sc.channel_len(99), 0);
}

#[test]
fn latest_rtc_tracks_maximum() {
    let mut sc = StreamingTimeCorrelator::new(
        NanosDuration::from_secs(60).expect("bounded literal duration"),
    );
    sc.add_reference(
        1,
        Rtc::from_raw(50_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    assert_eq!(sc.latest_rtc(), Some(Rtc::from_raw(50_000_000)));

    // Earlier RTC doesn't update latest
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 11, 0, 0, 0).unwrap(),
    );
    assert_eq!(sc.latest_rtc(), Some(Rtc::from_raw(50_000_000)));

    // Later RTC updates
    sc.add_reference(
        1,
        Rtc::from_raw(100_000_000),
        AbsoluteTime::new(100, 13, 0, 0, 0).unwrap(),
    );
    assert_eq!(sc.latest_rtc(), Some(Rtc::from_raw(100_000_000)));
}

// Partial NTP-104 compatibility refusal; no whole-requirement coverage claim.
#[test]
fn format2_ntp_needs_context_without_inserting_a_reference() {
    // Ch11 Fig11-16 carries32-bit seconds/fraction, not an era declaration.
    // This legacy engine cannot retain RDG-012's per-session era/basis.
    let mut sc = StreamingTimeCorrelator::new(NanosDuration::new(1_000_000_000));
    let raw = crate::NetworkTime::Ntp(crate::NtpTime {
        seconds: 3_944_678_400, // 2025 only when era0 is established.
        fraction: 0,
    });
    let result = sc.add_reference_f2(
        7,
        Rtc::from_raw(10_000_000),
        &raw,
        &crate::LeapSecondTable::builtin(),
    );
    assert!(matches!(
        result,
        Err(TimeError::NeedsContext {
            context: "NTP era/basis required; use ntp::convert_ntp"
        })
    ));
    assert!(sc.is_empty());
    assert_eq!(sc.len(), 0);
    assert_eq!(sc.latest_rtc(), None);
    assert_eq!(sc.total_evicted(), 0);
}

// CLI-104 bounded-reference prerequisite only, not whole coverage. These are
// reader resource-policy fixtures, not additional IRIG wire behaviour.
#[test]
fn reference_limit_bounds_equal_rtc_duplicates_and_keeps_queries_usable() {
    let mut s = StreamingTimeCorrelator::with_reference_limit(
        NanosDuration::new(u64::MAX),
        core::num::NonZeroUsize::new(8).unwrap(),
    );
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    for _ in 0..1000 {
        s.add_reference(1, Rtc::from_raw(0), time);
        assert!(s.len() <= 8);
    }
    assert_eq!(s.len(), 8);
    assert_eq!(s.total_evicted(), 992);
    assert_eq!(s.correlate(Rtc::from_raw(0), Some(1)).unwrap(), time);
}

#[test]
fn reference_limit_is_global_and_age_eviction_still_applies() {
    let limit = core::num::NonZeroUsize::new(2).unwrap();
    let mut s = StreamingTimeCorrelator::with_reference_limit(NanosDuration::new(100), limit);
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    s.add_reference(2, Rtc::from_raw(0), time);
    s.add_reference(1, Rtc::from_raw(0), time);
    s.add_reference(3, Rtc::from_raw(0), time);
    // Capacity ties: smallest channel ID first; two global slots, not two/channel.
    assert_eq!(s.channel_ids(), [2, 3]);
    assert_eq!(s.total_evicted(), 1);
    assert!(s.correlate(Rtc::from_raw(0), Some(1)).is_err());
    s.add_reference(4, Rtc::from_raw(2), time); // 200 ns; old refs exceed 100 ns.
    assert_eq!(s.channel_ids(), [4]);
    assert_eq!(s.len(), 1);
    assert_eq!(s.total_evicted(), 3);
}

#[test]
fn evicted_channel_vectors_cannot_retain_unbounded_heap_capacity() {
    let cap = 16;
    let mut s = StreamingTimeCorrelator::with_reference_limit(
        NanosDuration::new(u64::MAX),
        core::num::NonZeroUsize::new(cap).unwrap(),
    );
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    // Keep a late anchor on each old channel while cycling through bursts.
    // Merely bounding len(), or dropping only empty channels, is insufficient.
    for channel in 0..64 {
        for rtc in 0..32 {
            s.add_reference(channel, Rtc::from_raw(rtc), time);
        }
        s.add_reference(channel, Rtc::from_raw(1000 + u64::from(channel)), time);
        assert!(s.len() <= cap);
        assert!(s.channel_refs.len() <= cap);
        let allocated: usize = s.channel_refs.values().map(VecDeque::capacity).sum();
        assert!(
            allocated <= 4 * cap,
            "{allocated} reference slots for cap {cap}"
        );
    }
}

#[test]
fn age_eviction_releases_empty_channel_storage_without_a_count_limit() {
    let mut s = StreamingTimeCorrelator::new(NanosDuration::new(100));
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    for channel in 0..64 {
        s.add_reference(channel, Rtc::from_raw(u64::from(channel) * 2), time);
    }
    assert_eq!(s.len(), 1);
    assert_eq!(s.channel_refs.len(), 1);
    assert_eq!(s.total_evicted(), 63);
}

#[test]
fn a_single_slot_retains_the_latest_equal_rtc_arrival() {
    let mut s = StreamingTimeCorrelator::with_reference_limit(
        NanosDuration::new(u64::MAX),
        core::num::NonZeroUsize::MIN,
    );
    s.add_reference(
        1,
        Rtc::from_raw(0),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    let newer = AbsoluteTime::new(100, 12, 0, 5, 0).unwrap();
    s.add_reference(1, Rtc::from_raw(0), newer);
    assert_eq!(s.len(), 1);
    assert_eq!(s.total_evicted(), 1);
    assert_eq!(s.correlate(Rtc::from_raw(0), Some(1)).unwrap(), newer);
}

#[test]
fn count_eviction_uses_oldest_rtc_not_channel_or_arrival_order() {
    let mut s = StreamingTimeCorrelator::with_reference_limit(
        NanosDuration::new(u64::MAX),
        core::num::NonZeroUsize::new(2).unwrap(),
    );
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    for (channel, raw) in [(1, 20), (2, 10), (3, 30)] {
        s.add_reference(channel, Rtc::from_raw(raw), time);
    }
    assert_eq!(s.channel_ids(), [1, 3]);
    assert_eq!(s.total_evicted(), 1);
}

#[test]
fn evicted_step_queries_are_honest_in_both_modes() {
    for count_limited in [false, true] {
        let mut s = if count_limited {
            StreamingTimeCorrelator::with_reference_limit(
                NanosDuration::new(u64::MAX),
                core::num::NonZeroUsize::new(2).unwrap(),
            )
        } else {
            StreamingTimeCorrelator::new(NanosDuration::new(1000))
        };
        let old = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
        let stepped = AbsoluteTime::new(100, 13, 0, 0, 0).unwrap();
        s.add_reference(1, Rtc::from_raw(0), old);
        s.add_reference(1, Rtc::from_raw(10), stepped);
        s.add_reference(1, Rtc::from_raw(20), stepped);
        for ch in [Some(1), None] {
            let error = s.correlate(Rtc::from_raw(0), ch).unwrap_err();
            assert_eq!(error, TimeError::QueryBeforeRetainedWindow);
            assert_eq!(
                error.to_string(),
                "query precedes retained reference window"
            );
        }
        // No loss on channel 2: pre-reference extrapolation is still allowed.
        s.add_reference(2, Rtc::from_raw(20), old);
        assert!(s.correlate(Rtc::from_raw(19), Some(2)).is_ok());
    }
}

#[test]
fn duplicate_rtc_uses_latest_arrival_in_both_modes() {
    for count_limited in [false, true] {
        let mut s = if count_limited {
            StreamingTimeCorrelator::with_reference_limit(
                NanosDuration::new(u64::MAX),
                core::num::NonZeroUsize::new(8).unwrap(),
            )
        } else {
            StreamingTimeCorrelator::new(NanosDuration::new(u64::MAX))
        };
        for second in 0..5 {
            s.add_reference(
                1,
                Rtc::from_raw(10),
                AbsoluteTime::new(100, 12, 0, second, 0).unwrap(),
            );
        }
        for ch in [None, Some(1)] {
            assert_eq!(s.correlate(Rtc::from_raw(10), ch).unwrap().seconds(), 4);
        }
    }
}

#[test]
fn seeded_retained_window_differential() {
    let mut full = StreamingTimeCorrelator::new(NanosDuration::new(u64::MAX));
    let mut bounded = StreamingTimeCorrelator::with_reference_limit(
        NanosDuration::new(u64::MAX),
        core::num::NonZeroUsize::new(8).unwrap(),
    );
    let mut seed = 0x106u32;
    let mut rtc = 0u64;
    let base = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    for i in 0..100 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        rtc += 1 + u64::from(seed % 100);
        let t = base.add_nanos(i * 1_000_000_000).unwrap();
        full.add_reference(1, Rtc::from_raw(rtc), t);
        bounded.add_reference(1, Rtc::from_raw(rtc), t);
        let oldest = bounded.channel_refs[&1][0].rtc.as_raw();
        for _ in 0..12 {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let q = Rtc::from_raw(oldest + u64::from(seed) % (rtc - oldest + 10));
            for ch in [None, Some(1)] {
                assert_eq!(bounded.correlate(q, ch), full.correlate(q, ch));
            }
        }
        if i >= 8 {
            assert!(bounded.correlate(Rtc::from_raw(oldest - 1), None).is_err());
        }
    }
}

#[test]
fn loss_history_survives_empty_channel_and_does_not_taint_other_channels() {
    let mut s = StreamingTimeCorrelator::with_reference_limit(
        NanosDuration::new(u64::MAX),
        core::num::NonZeroUsize::new(2).unwrap(),
    );
    let t = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    s.add_reference(1, Rtc::from_raw(0), t);
    s.add_reference(2, Rtc::from_raw(10), t);
    s.add_reference(2, Rtc::from_raw(20), t);
    assert_eq!(
        s.correlate(Rtc::from_raw(0), Some(1)),
        Err(TimeError::NoReferencePoint)
    );
    assert!(s.correlate(Rtc::from_raw(9), Some(2)).is_ok());
    assert_eq!(
        s.correlate(Rtc::from_raw(9), None),
        Err(TimeError::QueryBeforeRetainedWindow)
    );
    // Reappearing channel does not forget its earlier loss.
    s.add_reference(1, Rtc::from_raw(30), t);
    assert_eq!(
        s.correlate(Rtc::from_raw(29), Some(1)),
        Err(TimeError::QueryBeforeRetainedWindow)
    );
    assert!(s.correlate(Rtc::from_raw(30), Some(1)).is_ok());
}

#[test]
fn clear_channel_discards_only_its_epoch_and_restarts_latest_when_empty() {
    let mut s = StreamingTimeCorrelator::new(NanosDuration::new(30_000_000_000));
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    s.add_reference(1, Rtc::from_raw(400_000_000), time);
    s.add_reference(2, Rtc::from_raw(400_000_000), time);
    assert_eq!(s.clear_channel(1), 1);
    assert_eq!(s.channel_ids(), [2]);
    assert_eq!(s.correlate(Rtc::from_raw(400_000_000), Some(2)), Ok(time));
    assert_eq!(s.clear_channel(1), 0);
    assert_eq!(s.clear_channel(2), 1);
    assert_eq!(s.latest_rtc(), None);
    s.add_reference(1, Rtc::from_raw(10_000_000), time);
    assert_eq!(s.latest_rtc(), Some(Rtc::from_raw(10_000_000)));
    assert_eq!(s.len(), 1);
    assert_eq!(s.total_evicted(), 2);
    assert_eq!(s.correlate(Rtc::from_raw(10_000_000), Some(1)), Ok(time));
}

#[test]
fn clear_channel_keeps_loss_guard_for_new_low_rtc_window() {
    let mut s = StreamingTimeCorrelator::new(NanosDuration::new(30_000_000_000));
    let time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    s.add_reference(7, Rtc::from_raw(400_000_000), time);
    s.add_reference(7, Rtc::from_raw(410_000_000), time);
    assert_eq!(s.total_evicted(), 0); // clear is the only source of loss.
    assert_eq!(s.clear_channel(7), 2);
    assert_eq!(s.total_evicted(), 2);
    s.add_reference(7, Rtc::from_raw(10_000_000), time);
    s.add_reference(7, Rtc::from_raw(20_000_000), time);
    assert_eq!(
        s.correlate(Rtc::from_raw(9_999_999), Some(7)),
        Err(TimeError::QueryBeforeRetainedWindow)
    );
    assert_eq!(s.correlate(Rtc::from_raw(10_000_000), Some(7)), Ok(time));
}
