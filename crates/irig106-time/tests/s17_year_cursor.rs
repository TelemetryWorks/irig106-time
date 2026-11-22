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
    for ticks in [i128::MIN, i128::MAX, i128::MAX / 100, -(i128::MAX / 100)] {
        let mut huge = r.clone();
        huge.key.packet = 5;
        huge.key.ticks = ticks;
        assert!(matches!(
            c.observe(huge),
            Err(TimeError::QuantityOverflow { .. })
        ));
    }
    let mut safe = r.clone();
    safe.key.packet = 5;
    safe.key.ticks = 1;
    assert!(c.observe(safe).is_ok()); // Overflow errors did not consume packet5.
    assert!(
        SessionYearCursor::new(
            1,
            OfflinePolicy {
                version: 0,
                ..Default::default()
            }
        )
        .is_err()
    );
    let mut other = SessionYearCursor::new(
        1,
        OfflinePolicy {
            jump_threshold_ns: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    let (other, _) = other.observe(r.clone()).unwrap();
    assert!(f.resolve(other).is_err());

    let mut retry = f.replay();
    let mut next = r.clone();
    next.key.packet = 5;
    assert!(retry.observe(next.clone()).is_err());
    let _ = retry.observe(r.clone()).unwrap();
    let (past_end, _) = retry.observe(next.clone()).unwrap();
    assert!(f.resolve(past_end).is_err());
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

#[test]
fn native_year_gap_closes_resolver_and_retains_partial_context() {
    let mut first = raw(0, 0, &[0x41, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    first.input = first
        .input
        .with_year_context(YearContext {
            caller: Some(2023),
            recording: None,
        })
        .unwrap();
    let mut later = raw(
        9,
        800 * 86400 * 10_000_000,
        &[0x41, 0, 0, 0, 0, 0, 0, 0, 1, 1],
    );
    later.input = later
        .input
        .with_year_context(YearContext {
            caller: Some(2023),
            recording: None,
        })
        .unwrap();
    let mut cursor = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    let (a, _) = cursor.observe(first.clone()).unwrap();
    let (b, closed) = cursor.observe(later.clone()).unwrap();
    let old = closed.unwrap();
    assert!(old.resolve(b.clone()).is_err());
    assert!(old.resolve(a.clone()).is_ok());
    let new = cursor.freeze().unwrap();
    assert!(new.resolve(a).is_err());
    let (resolved, findings, _) = new.resolve(b).unwrap();
    assert_eq!(resolved.input.year_basis.continuity_segment(), 1);
    assert_eq!(
        resolved.input.year_basis.gap_ticks(),
        Some(SignedRtcTicks::new(later.key.ticks))
    );
    assert!(resolved.input.year_basis.selected().is_none());
    assert!(resolved.input.year_basis.year_issue().is_some());
    assert!(findings.iter().any(|f| f.code == 20));
    let mut replay = new.replay();
    let (b, closed) = replay.observe(later).unwrap();
    assert!(closed.is_none());
    let (again, _, _) = new.resolve(b).unwrap();
    assert_eq!(again, resolved);
}

#[test]
fn replay_preserves_first_observed_origin_for_late_day366() {
    // Q14: first observed2017 day1 at RTC1s; late day366 at RTC0 is2016.
    let mut refs = vec![
        raw(0, 10_000_000, &[0x41, 0, 0, 0, 0, 0, 0, 0, 1, 0]),
        raw(1, 0, &[0x41, 1, 0, 0, 0, 0x59, 0x59, 0x23, 0x66, 3]),
    ];
    for r in &mut refs {
        r.input = r
            .input
            .clone()
            .with_year_context(YearContext {
                caller: Some(2017),
                recording: None,
            })
            .unwrap();
    }
    let expected = oracle(&refs);
    let mut c = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    for r in &refs {
        c.observe(r.clone()).unwrap();
    }
    let f = c.freeze().unwrap();
    let mut replay = f.replay();
    for (r, want) in refs.iter().zip(&expected.sessions()[0].references) {
        let (s, _) = replay.observe(r.clone()).unwrap();
        let (got, _, _) = f.resolve(s).unwrap();
        assert_eq!(got.input.value, want.input.value);
        assert_eq!(got.input.year_basis, want.input.year_basis);
    }
}

#[test]
fn resolver_keeps_dmy_evidence_skipped_by_the_phase_clock() {
    // A valid2017June30 leap label is forbidden in the phase clock's2016
    // year. It still carries explicit DMY evidence, as offline's second pass
    // records; it must not disappear from contradictory year candidates.
    let refs = vec![
        raw(0, 0, &[0x41, 3, 0, 0, 0, 0, 0, 0, 1, 1, 0x16, 0x20]),
        raw(
            1,
            10_000_000,
            &[0x41, 2, 0, 0, 0, 0x60, 0x59, 0x23, 0x30, 6, 0x17, 0x20],
        ),
        raw(2, 20_000_000, &[0x41, 1, 0, 0, 0, 0, 0, 0, 2, 0]),
    ];
    let want = oracle(&refs);
    let mut c = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    for r in &refs {
        c.observe(r.clone()).unwrap();
    }
    let f = c.freeze().unwrap();
    let mut replay = f.replay();
    for (r, want) in refs.iter().zip(&want.sessions()[0].references) {
        let (s, _) = replay.observe(r.clone()).unwrap();
        let (got, _, _) = f.resolve(s).unwrap();
        assert_eq!(got.input.year_basis, want.input.year_basis);
    }
}

// Rebuild from raw inputs; carry forward ONLY newly rejected leap labels.
// Bounded test driver retains inputs; production callers may spool them to disk.
fn cursor_resolve(original: &[TimelineReference]) -> Vec<TimelineReference> {
    let mut inputs = original.to_vec();
    for _ in 0..=inputs.len() {
        let mut cursor = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
        let mut stamps = Vec::new();
        let mut resolvers = Vec::new();
        for r in &inputs {
            let (stamp, closed) = cursor.observe(r.clone()).unwrap();
            stamps.push(stamp);
            if let Some(closed) = closed {
                resolvers.push(closed);
            }
        }
        resolvers.push(cursor.freeze().unwrap());
        let mut output = Vec::new();
        let mut changed = false;
        for (i, stamp) in stamps.into_iter().enumerate() {
            let (r, _, bad) = resolvers
                .iter()
                .find_map(|resolver| resolver.resolve(stamp.clone()).ok())
                .unwrap();
            if bad && !matches!(inputs[i].input.value, ReferenceValue::Malformed) {
                inputs[i] = r.clone();
                changed = true;
            }
            output.push(r);
        }
        if !changed {
            return output;
        }
    }
    panic!("monotone rejected labels did not converge");
}
fn assert_equivalent(refs: &[TimelineReference]) {
    let got = cursor_resolve(refs);
    let timeline = oracle(refs);
    let want = &timeline.sessions()[0].references;
    assert_eq!(got, *want);
    for r in want {
        if matches!(r.input.value, ReferenceValue::Readable(..)) {
            assert!(!r.reasons.contains(&Rejection::Malformed), "{r:?}");
        }
    }
}
#[test]
fn rebuild_does_not_transport_other_malformed_rejections() {
    // Figs11-13/14: DOY366 23:58:50..56, RTC0..6s, then a dated
    // 2016-01-01 12:58:59 anchor 13h later. Pin both the eight-packet
    // ordinary case and its leap-labelled variant, which requires a rebuild.
    let mut refs: Vec<_> = (0..7)
        .map(|i| {
            raw(
                i,
                i128::from(i) * 10_000_000,
                &[0x41, 1, 0, 0, 0, 0x50 + i as u8, 0x58, 0x23, 0x66, 3],
            )
        })
        .collect();
    refs.push(raw(
        7,
        (6 + 13 * 3600) * 10_000_000,
        &[0x41, 3, 0, 0, 0, 0x59, 0x58, 0x12, 1, 1, 0x16, 0x20],
    ));
    assert_equivalent(&refs);
    refs[6] = raw(
        6,
        60 * 10_000_000,
        &[0x41, 1, 0, 0, 0, 0x60, 0x59, 0x23, 0x66, 3],
    );
    assert_equivalent(&refs);
}
#[test]
fn seeded_bounded_cursor_offline_equivalence() {
    // Fixed seed, 192 eight-reference sessions; Figs11-13/14 decimal fields.
    let mut seed = 0x1062_4013u32;
    for _ in 0..192 {
        let mut refs = Vec::new();
        for i in 0..7 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let day = [1u16, 181, 182, 365, 366][(seed as usize) % 5];
            let seconds = if day != 1 && seed & 8 == 0 {
                0x60
            } else {
                0x50 + i as u8
            };
            let day_bcd = ((day % 10) | ((day / 10 % 10) << 4)) as u8;
            refs.push(raw(
                i,
                i128::from(i) * 10_000_000,
                &[
                    0x41,
                    u8::from(day == 366),
                    0,
                    0,
                    0,
                    seconds,
                    0x59,
                    0x23,
                    day_bcd,
                    (day / 100) as u8,
                ],
            ));
        }
        refs.push(raw(
            7,
            (6 + 13 * 3600) * 10_000_000,
            &[0x41, 3, 0, 0, 0, 0x59, 0x58, 0x12, 1, 1, 0x16, 0x20],
        ));
        assert_equivalent(&refs);
    }
}
#[test]
fn gap_closed_resolver_uses_final_evidence_not_phase_anchors() {
    // Explicit2017 DMY leap label is skipped by the2016 phase clock but
    // remains final evidence; then >half-year context gap closes the resolver.
    let refs = vec![
        raw(0, 0, &[0x41, 3, 0, 0, 0, 0, 0, 0, 1, 1, 0x16, 0x20]),
        raw(
            1,
            10_000_000,
            &[0x41, 2, 0, 0, 0, 0x60, 0x59, 0x23, 0x30, 6, 0x17, 0x20],
        ),
        raw(2, 20_000_000, &[0x41, 1, 0, 0, 0, 0, 0, 0, 2, 0]),
    ];
    let mut cursor = SessionYearCursor::new(1, OfflinePolicy::default()).unwrap();
    let mut last = None;
    for r in &refs {
        last = Some(cursor.observe(r.clone()).unwrap().0);
    }
    let stamp = last.unwrap();
    let expected = cursor.freeze().unwrap().resolve(stamp.clone()).unwrap();
    let (_, closed) = cursor
        .observe(raw(
            3,
            800 * 86400 * 10_000_000,
            &[0x41, 0, 0, 0, 0, 0, 0, 0, 1, 0],
        ))
        .unwrap();
    assert_eq!(closed.unwrap().resolve(stamp).unwrap(), expected);
    assert!(
        expected
            .0
            .input
            .year_basis
            .candidates()
            .iter()
            .any(|c| c.supplied == 2017)
    );
}
