//! Hand arithmetic: 106-24R1 §11.2.1.1 i (10 MHz, 48-bit free-running RTC),
//! printed 11-7/11-8; §g ERTC is 1 ns. No encoder-generated oracle.
use irig106_time::{NanosDuration, Rtc, TimeError, rtc::*, timeline::PacketKey};
/// Requirements: L1-RTC-001, L2-RTC-101, L3-RTC-101
#[test]
fn checked_width_never_silently_masks() {
    for raw in [0, 1, (1 << 47), (1 << 48) - 1] {
        assert_eq!(checked_rtc(raw).unwrap().as_raw(), raw);
    }
    for raw in [1 << 48, u64::MAX] {
        assert!(matches!(checked_rtc(raw), Err(TimeError::CounterWidth { value }) if value == raw));
    }
}
/// Requirements: L1-RTC-003, L2-RTC-103, L3-RTC-103
#[test]
fn modular_signed_direction_context_and_sessions() {
    let stamp = |session, rtc| SessionRtc { session, rtc };
    let max = stamp(7, Rtc::MAX);
    let zero = stamp(7, Rtc::ZERO);
    assert_eq!(max.delta_to(zero, CounterOrder::Nearest).unwrap().get(), 1);
    assert_eq!(zero.delta_to(max, CounterOrder::Nearest).unwrap().get(), -1);
    assert_eq!(zero.delta_to(zero, CounterOrder::Nearest).unwrap().get(), 0);
    let half = stamp(7, Rtc::from_raw(1 << 47));
    assert!(matches!(
        zero.delta_to(half, CounterOrder::Nearest),
        Err(TimeError::AmbiguousCounterDelta)
    ));
    assert_eq!(
        zero.delta_to(half, CounterOrder::ForwardWithinOneWrap)
            .unwrap()
            .get(),
        1 << 47
    );
    assert_eq!(
        zero.delta_to(half, CounterOrder::BackwardWithinOneWrap)
            .unwrap()
            .get(),
        -(1 << 47)
    );
    assert!(matches!(
        zero.delta_to(stamp(8, Rtc::ZERO), CounterOrder::Nearest),
        Err(TimeError::CrossSessionCounterDelta)
    ));
    assert!(
        zero.delta_to(stamp(0, Rtc::ZERO), CounterOrder::Nearest)
            .is_err()
    );
    assert_eq!(
        zero.delta_to(max, CounterOrder::ForwardWithinOneWrap)
            .unwrap()
            .get(),
        (1 << 48) - 1
    );
}
/// Requirements: L1-RTC-004, L2-RTC-104, L3-RTC-104
#[test]
fn signed_duration_boundaries_and_shared_magnitude() {
    for (ticks, ns) in [
        (0, 0),
        (1, 100),
        (-1, -100),
        (150_000, 15_000_000),
        (-150_000, -15_000_000),
    ] {
        let d = SignedRtcTicks::new(ticks).to_nanos().unwrap();
        assert_eq!(d.get(), ns);
        assert_eq!(
            d.magnitude().unwrap(),
            NanosDuration::new(ns.unsigned_abs() as u64)
        );
    }
    let limit = i128::MAX / 100;
    assert_eq!(
        SignedRtcTicks::new(limit).to_nanos().unwrap().get(),
        limit * 100
    );
    assert_eq!(
        SignedRtcTicks::new(-limit).to_nanos().unwrap().get(),
        -limit * 100
    );
    for ticks in [limit + 1, -limit - 1, i128::MIN, i128::MAX] {
        assert!(matches!(
            SignedRtcTicks::new(ticks).to_nanos(),
            Err(TimeError::QuantityOverflow { .. })
        ));
    }
    assert_eq!(
        SignedNanos::new(u64::MAX as i128)
            .magnitude()
            .unwrap()
            .get(),
        u64::MAX
    );
    for ns in [u64::MAX as i128 + 1, i128::MIN] {
        assert!(matches!(
            SignedNanos::new(ns).magnitude(),
            Err(TimeError::QuantityOverflow { .. })
        ));
    }
    assert!(SignedNanos::new(-1).to_unsigned().is_err());
    assert_eq!(SignedNanos::new(1).to_unsigned().unwrap().get(), 1);
}
/// Requirements: L1-RTC-003, L2-RTC-103, L3-RTC-103, L1-RTC-004, L2-RTC-104, L3-RTC-104
#[test]
fn unwrapped_packet_boundaries_are_checked() {
    let key = |session, ticks| PacketKey {
        session,
        ticks,
        packet: 1,
    };
    assert_eq!(
        key(2, 200)
            .delta_to(key(2, 50))
            .unwrap()
            .to_nanos()
            .unwrap()
            .get(),
        -15_000
    );
    assert!(matches!(
        key(2, 0).delta_to(key(3, 0)),
        Err(TimeError::CrossSessionCounterDelta)
    ));
    assert!(matches!(
        key(2, i128::MIN).delta_to(key(2, i128::MAX)),
        Err(TimeError::QuantityOverflow { .. })
    ));
}
