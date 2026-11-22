//! Partial CLI-104/native span infrastructure; no whole Requirements marker.
use irig106_time::{
    Rtc,
    session::{SessionClock, SessionStart},
    stream::*,
    timeline::*,
};

fn packet(id: u64, ticks: u64) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(ticks),
        kind: PacketKind::Data,
        declared_timein: false,
        declared_external: false,
        flags: 0,
        reference: None,
    }
}
#[test]
fn native_clock_matches_both_engines_at_late_boundary_wrap_and_reset() {
    // Native default1100ms: exactly11_000_000 ticks is late, not a reset;
    // one extra100ns tick crosses the strict bound. A short48-bit wrap is forward.
    for (infer, late_ns) in [
        (false, 1_100_000_000),
        (true, 1_100_000_000),
        (true, 200_000_000),
    ] {
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let p = OfflinePolicy {
                infer_sessions: infer,
                late_ns,
                ..OfflinePolicy::default()
            };
            let mut clock = SessionClock::new(&p).unwrap();
            assert_eq!(clock.late_ns(), late_ns);
            assert_eq!(clock.infer_sessions(), infer);
            let mut offline = TimelineBuilder::new(p.clone()).unwrap();
            let mut sp = StreamPolicy::new(
                1,
                BufferCaps {
                    packets: 2,
                    bytes: 1024,
                    references: 2,
                },
            );
            sp.offline = p;
            sp.mode = mode;
            let mut stream = StreamTimeline::new(sp, None).unwrap();
            for (id, (ticks, boundary)) in [
                (20_000_000, false),
                (9_000_000, false),
                (8_999_999, false),
                ((1u64 << 48) - 10, true),
                (10, false),
                (5, false),
                (100, true),
            ]
            .into_iter()
            .enumerate()
            {
                let event = clock
                    .observe(id as u64, Rtc::from_raw(ticks), boundary)
                    .unwrap();
                let expected = offline.observe(packet(id as u64, ticks), boundary).unwrap();
                let batch = stream
                    .observe(packet(id as u64, ticks), id as u64, 24, boundary)
                    .unwrap();
                assert_eq!(event.key, expected);
                assert_eq!(batch.answers.last().unwrap().basis.key, expected);
            }
        }
    }
}
#[test]
fn spans_keep_high_water_low_water_counts_and_explicit_basis() {
    let mut clock = SessionClock::new(&OfflinePolicy::default()).unwrap();
    assert!(clock.active().is_none());
    assert_eq!(clock.late_ns(), 1_100_000_000);
    assert!(clock.infer_sessions());
    for (id, ticks) in [(0, 20_000_000), (1, 30_000_000), (2, 25_000_000)] {
        assert!(
            clock
                .observe(id, Rtc::from_raw(ticks), false)
                .unwrap()
                .closed
                .is_none()
        );
    }
    let event = clock.observe(3, Rtc::from_raw(1), false).unwrap();
    let closed = event.closed.unwrap();
    assert_eq!(closed.start, SessionStart::Initial);
    assert_eq!(closed.first.packet, 0);
    assert_eq!(closed.last.packet, 2);
    assert_eq!(closed.first.ticks, 20_000_000);
    assert_eq!(closed.last.ticks, 25_000_000);
    assert_eq!(
        (closed.low_ticks, closed.high_ticks, closed.packets),
        (20_000_000, 30_000_000, 3)
    );
    assert_eq!(event.key.session, 2);
    assert_eq!(clock.active().unwrap().start, SessionStart::Inferred);
    let event = clock.observe(4, Rtc::from_raw(2), true).unwrap();
    assert_eq!(event.closed.unwrap().packets, 1);
    assert_eq!(clock.active().unwrap().start, SessionStart::Explicit);
    assert_eq!(clock.active().unwrap().first, event.key);
    // Cursor itself stores no packets/references/closed-session collection.
    assert!(std::mem::size_of::<SessionClock>() < 512);
    assert!(!std::mem::needs_drop::<SessionClock>());
    let mut late = SessionClock::new(&OfflinePolicy::default()).unwrap();
    late.observe(0, Rtc::from_raw(20_000_000), false).unwrap();
    late.observe(1, Rtc::from_raw(9_000_000), false).unwrap();
    assert_eq!(
        (
            late.active().unwrap().low_ticks,
            late.active().unwrap().high_ticks
        ),
        (9_000_000, 20_000_000)
    );
}
#[test]
fn invalid_policy_identity_and_ambiguous_rtc_do_not_commit_state() {
    assert!(
        SessionClock::new(&OfflinePolicy {
            late_ns: u64::MAX,
            ..OfflinePolicy::default()
        })
        .is_err()
    );
    let mut clock = SessionClock::new(&OfflinePolicy::default()).unwrap();
    clock.observe(5, Rtc::ZERO, false).unwrap();
    let before = *clock.active().unwrap();
    for (id, ticks) in [(5, 1), (4, 1), (6, 1u64 << 47)] {
        assert!(clock.observe(id, Rtc::from_raw(ticks), false).is_err());
        assert_eq!(*clock.active().unwrap(), before);
    }
    assert_eq!(
        clock
            .observe(6, Rtc::from_raw(1), false)
            .unwrap()
            .key
            .packet,
        6
    );
    // Explicit boundaries bypass even an otherwise ambiguous half-wrap.
    assert_eq!(
        clock
            .observe(7, Rtc::from_raw((1u64 << 47) + 1), true)
            .unwrap()
            .key
            .session,
        2
    );
    fn traits<T: Send + Sync + Clone + std::fmt::Debug>() {}
    traits::<SessionClock>();
}
