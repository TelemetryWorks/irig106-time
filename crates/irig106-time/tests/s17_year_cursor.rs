//! Bounded year-replay infrastructure clauses; no whole requirement marker.
use irig106_time::{
    edition::EditionEvidence, format1::decode_qualified_time_f1, scale::ScalePolicy, timeline::*,
    years::*, *,
};
fn raw(id: u64, ticks: i128, bytes: &[u8]) -> TimelineReference {
    let input = ReferenceInput::qualified_format1(
        decode_qualified_time_f1(
            bytes,
            bytes.len(),
            EditionEvidence::default(),
            ScalePolicy::default(),
        )
        .unwrap(),
    )
    .unwrap();
    let admitted = matches!(input.value, ReferenceValue::Readable(..));
    TimelineReference {
        key: PacketKey {
            session: 1,
            packet: id,
            ticks,
        },
        channel: 7,
        declared_timein: true,
        declared_external: false,
        input,
        admitted,
        reasons: if admitted {
            vec![]
        } else {
            vec![Rejection::NeedsContext]
        },
    }
}
fn oracle(refs: &[TimelineReference]) -> OfflineTimeline {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for r in refs {
        b.observe(
            PacketObservation {
                id: r.key.packet,
                channel: 7,
                rtc: Rtc::from_raw(r.key.ticks as u64),
                kind: PacketKind::Time,
                declared_timein: true,
                declared_external: false,
                flags: 0,
                reference: Some(r.input.clone()),
            },
            false,
        )
        .unwrap();
    }
    b.finish().unwrap()
}
#[test]
fn cursor_replay_matches_offline_future_dmy_rollover_and_conflict() {
    let refs = vec![
        raw(0, 0, &[0x41, 0, 0, 0, 0, 0x59, 0x59, 0x23, 0x65, 3]),
        raw(1, 10_000_000, &[0x41, 1, 0, 0, 0, 0, 0, 0, 1, 0]),
        raw(
            2,
            20_000_000,
            &[0x41, 3, 0, 0, 0, 1, 0, 0, 1, 1, 0x24, 0x20],
        ),
        raw(
            3,
            30_000_000,
            &[0x41, 2, 0, 0, 0, 2, 0, 0, 1, 1, 0x25, 0x20],
        ),
    ];
    let expected = oracle(&refs);
    let mut cursor = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    for r in &refs {
        let (_, closed) = cursor.observe(r.clone()).unwrap();
        assert!(closed.is_none());
    }
    let resolver = cursor.freeze().unwrap();
    let mut replay = resolver.replay();
    for (raw, expected) in refs.iter().zip(&expected.sessions()[0].references) {
        let (stamp, _) = replay.observe(raw.clone()).unwrap();
        assert_eq!(stamp.reference().key, raw.key);
        let (resolved, findings, rebuild) = resolver.resolve(stamp).unwrap();
        assert!(!rebuild);
        assert_eq!(resolved.input.value, expected.input.value);
        assert_eq!(resolved.input.year_basis, expected.input.year_basis);
        assert!(findings.iter().any(|f| f.code == 32));
    }
}
#[test]
fn cursor_scope_order_and_replay_origin_errors_do_not_consume_state() {
    let r = raw(4, 0, &[0x41, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    assert!(SessionYearCursor::new(0, OfflinePolicy::default()).is_err());
    let mut c = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    assert!(c.freeze().is_none());
    let mut wrong = r.clone();
    wrong.key.session = 2;
    assert!(c.observe(wrong).is_err());
    let (stamp, _) = c.observe(r.clone()).unwrap();
    assert!(c.observe(r.clone()).is_err());
    let f = c.freeze().unwrap();
    let mut retry = f.replay();
    let mut next = r.clone();
    next.key.packet = 5;
    assert!(retry.observe(next.clone()).is_err());
    let _ = retry.observe(r.clone()).unwrap();
    let mut foreign = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    let (foreign, _) = foreign.observe(next).unwrap();
    assert!(f.resolve(foreign).is_err());
    assert!(f.resolve(stamp).is_ok());
    fn controller<T: Clone + core::fmt::Debug + Send + Sync>() {}
    controller::<SessionYearCursor>();
    controller::<SessionYearResolver>();
}
#[test]
fn future_year_rebuild_drops_forbidden_leap_credit() {
    // Owner2026-11-15/RDG-010: day365 second60 is forbidden once2016 known.
    let mut refs = vec![
        raw(0, 0, &[0x41, 1, 0, 0, 0, 0x59, 0x59, 0x23, 0x65, 3]),
        raw(
            1,
            10_000_000,
            &[0x41, 1, 0, 0, 0, 0x60, 0x59, 0x23, 0x65, 3],
        ),
        raw(2, 20_000_000, &[0x41, 1, 0, 0, 0, 0, 0, 0, 0x66, 3]),
        raw(
            3,
            30_000_000,
            &[0x41, 3, 0, 0, 0, 1, 0, 0, 0x31, 0x12, 0x16, 0x20],
        ),
    ];
    let expected = oracle(&refs);
    for pass in 0..2 {
        let mut c = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
        for r in &refs {
            c.observe(r.clone()).unwrap();
        }
        let f = c.freeze().unwrap();
        let mut replay = f.replay();
        let mut saw = false;
        for (i, r) in refs.iter_mut().enumerate() {
            let (s, _) = replay.observe(r.clone()).unwrap();
            let (resolved, _, bad) = f.resolve(s).unwrap();
            if pass == 0 && bad {
                *r = resolved.clone();
                saw = true;
            }
            if pass == 1 {
                assert_eq!(
                    resolved.input.value,
                    expected.sessions()[0].references[i].input.value
                );
                assert_eq!(
                    resolved.input.year_basis.observed_leap_adjustment(),
                    SignedNanos::new(0)
                );
            }
        }
        assert_eq!(saw, pass == 0);
    }
}
