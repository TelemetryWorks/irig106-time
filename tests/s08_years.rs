//! S08 clause evidence: Fig11-13/14 (11-25/PDF33), LY §11.2.3.2a,
//! Ch9 Table9-4 RI4 and contract5.3; D9 owner policy. No encoder oracle.
use irig106_time::{findings::ValueValidity, stream::*, timeline::*, years::*, *};
fn r(day: u16, ly: bool, ctx: YearContext) -> ReferenceInput {
    // Hand-positioned DOY decimal digits in word2; clock00:00:00.
    let word = (day / 100) << 8 | ((day / 10) % 10) << 4 | (day % 10);
    let bytes = [
        1,
        u8::from(ly),
        0,
        0,
        0,
        0,
        0,
        0,
        word as u8,
        (word >> 8) as u8,
    ];
    ReferenceInput::format1(
        &bytes,
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(ctx)
    .unwrap()
}
fn dmy() -> ReferenceInput {
    // 2024-01-01 00:00:01: DATE=1,LY=1; word2=0x0101,word3=0x2024.
    let b = [1, 3, 0, 0, 0, 1, 0, 0, 1, 1, 0x24, 0x20];
    ReferenceInput::format1(
        &b,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
}
fn p(id: u64, ticks: u64, reference: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 1,
        rtc: Rtc::from_raw(ticks),
        kind: if reference.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference,
    }
}
fn sp(mode: StreamMode) -> StreamPolicy {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 4,
            bytes: 400,
            references: 3,
        },
    );
    p.mode = mode;
    p.offline.channel = Some(1);
    p
}
#[test]
fn priority_candidates_conflicts_and_severity_override_keep_the_time() {
    for (ctx, expected, origin) in [
        (YearContext::default(), None, YearOrigin::Unknown),
        (
            YearContext {
                caller: None,
                recording: Some(2022),
            },
            Some(2022),
            YearOrigin::RecordingDate,
        ),
        (
            YearContext {
                caller: Some(2023),
                recording: Some(2022),
            },
            Some(2023),
            YearOrigin::Caller,
        ),
    ] {
        let mut policy = OfflinePolicy::default();
        policy.finding_levels.insert(32, Severity::Information);
        let mut b = TimelineBuilder::new(policy).unwrap();
        let k = b.observe(p(1, 0, Some(r(1, false, ctx))), false).unwrap();
        let t = b.finish().unwrap();
        let a = t.lookup(k).unwrap();
        assert_eq!(a.time.unwrap().year(), expected);
        let input = &t.sessions()[0].references[0].input;
        assert_eq!(input.year_origin, origin);
        assert_eq!(input.year_basis.context(), ctx);
        assert_eq!(input.year_basis.selected().map(|c| c.year), expected);
        if ctx.caller.is_some() {
            assert_eq!(
                input
                    .year_basis
                    .candidates()
                    .iter()
                    .map(|c| c.year)
                    .collect::<Vec<_>>(),
                vec![2023, 2022]
            );
            let f = t.findings().iter().find(|f| f.code == 32).unwrap();
            assert_eq!(f.severity, Severity::Information);
            assert_eq!(f.evidence.validity(), ValueValidity::Kept);
            assert_eq!(f.raw_rtc, Some(Rtc::from_raw(0)));
            for (name, y) in [
                ("year.selected", 2023),
                ("year.caller", 2023),
                ("year.ri4", 2022),
            ] {
                assert!(f.evidence.raw_fields().iter().any(|v| v.name == name
                    && v.observed == irig106_time::findings::RawValue::Unsigned(y)));
            }
        } else {
            assert!(!t.findings().iter().any(|f| f.code == 32));
        }
        assert_eq!(
            t.findings().iter().any(|f| f.code == 20),
            expected.is_none()
        );
    }
    let ctx = YearContext {
        caller: Some(2023),
        recording: Some(2022),
    };
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b.observe(p(1, 0, Some(r(1, true, ctx))), false).unwrap();
    b.observe(p(2, 10_000_000, Some(dmy())), false).unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), Some(2024));
    let input = &t.sessions()[0].references[0].input;
    assert_eq!(input.year_origin, YearOrigin::Packet);
    assert_eq!(
        input
            .year_basis
            .candidates()
            .iter()
            .map(|c| c.year)
            .collect::<Vec<_>>(),
        vec![2024, 2023, 2022]
    );
    assert!(
        t.findings()
            .iter()
            .any(|f| f.code == 32 && f.severity == Severity::Warning)
    );
    assert!(!t.findings().iter().any(|f| f.code == 20));
}
#[test]
fn rollover_advances_context_and_future_dmy_can_date_the_preceding_year() {
    for (year, last) in [(2023, 365), (2024, 366)] {
        let ctx = YearContext {
            caller: Some(year),
            recording: None,
        };
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let a = b
            .observe(p(1, 0, Some(r(last, year == 2024, ctx))), false)
            .unwrap();
        let z = b
            .observe(
                p(2, 86400 * 10_000_000, Some(r(1, year != 2024, ctx))),
                false,
            )
            .unwrap();
        let t = b.finish().unwrap();
        assert_eq!(t.lookup(a).unwrap().time.unwrap().year(), Some(year));
        assert_eq!(t.lookup(z).unwrap().time.unwrap().year(), Some(year + 1));
        assert!(!t.findings().iter().any(|f| f.code == 7));
    }
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b
        .observe(p(1, 0, Some(r(365, false, YearContext::default()))), false)
        .unwrap();
    b.observe(p(2, 86401 * 10_000_000, Some(dmy())), false)
        .unwrap();
    assert_eq!(
        b.finish().unwrap().lookup(k).unwrap().time.unwrap().year(),
        Some(2023)
    );
}
#[test]
fn ly_checks_are_independent_of_year_selection_and_invalid_calendar_is_not_repaired() {
    for (year, day, ly, warn, usable) in [
        (None, 366, false, true, true),
        (None, 366, true, false, true),
        (Some(2023), 365, true, true, true),
        (Some(2024), 365, false, true, true),
        (Some(2023), 366, false, true, false),
        (Some(2024), 366, true, false, true),
    ] {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let k = b
            .observe(
                p(
                    1,
                    0,
                    Some(r(
                        day,
                        ly,
                        YearContext {
                            caller: year,
                            recording: None,
                        },
                    )),
                ),
                false,
            )
            .unwrap();
        let t = b.finish().unwrap();
        assert_eq!(t.findings().iter().any(|f| f.code == 7), warn);
        assert_eq!(t.sessions()[0].references[0].admitted, usable);
        assert_eq!(t.lookup(k).unwrap().time.is_some(), usable);
        assert_eq!(t.findings().iter().any(|f| f.code == 4), !usable);
    }
}
#[test]
fn sessions_isolate_dmy_and_governing_recording_year_changes_remain_in_each_basis() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(p(1, 0, Some(dmy())), false).unwrap();
    let k = b
        .observe(p(2, 0, Some(r(1, false, YearContext::default()))), true)
        .unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), None);
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for (i, y) in [(1, 2022), (2, 2023)] {
        b.observe(
            p(
                i,
                i * 10_000_000,
                Some(r(
                    1,
                    false,
                    YearContext {
                        caller: None,
                        recording: Some(y),
                    },
                )),
            ),
            false,
        )
        .unwrap();
    }
    let t = b.finish().unwrap();
    for (r, y) in t.sessions()[0].references.iter().zip([2022, 2023]) {
        assert_eq!(r.input.year_basis.context().recording, Some(y));
        assert_eq!(r.input.year_basis.selected().unwrap().year, y);
    }
    assert!(
        dmy()
            .with_year_context(YearContext {
                caller: Some(10000),
                recording: None
            })
            .is_err()
    );
    assert!(
        dmy()
            .with_year_context(YearContext {
                caller: None,
                recording: Some(10000)
            })
            .is_err()
    );
}
#[test]
fn stream_late_year_evidence_dates_only_unreleased_answers_in_both_modes() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        let first = s
            .observe(
                p(1, 0, Some(r(1, true, YearContext::default()))),
                0,
                100,
                false,
            )
            .unwrap();
        assert!(first.findings.iter().any(|f| f.code == 20));
        let data = s.observe(p(2, 5_000_000, None), 0, 100, false).unwrap();
        let old = data.answers.clone();
        let next = s
            .observe(p(3, 10_000_000, Some(dmy())), 0, 100, false)
            .unwrap();
        if mode == StreamMode::Standard {
            assert_eq!(old[0].time.unwrap().year(), None);
        } else {
            assert!(old.is_empty());
            assert_eq!(next.answers[0].time.unwrap().year(), Some(2024));
            assert_eq!(
                next.answers[0]
                    .basis
                    .preceding
                    .as_ref()
                    .unwrap()
                    .input
                    .year_basis
                    .selected()
                    .unwrap()
                    .year,
                2024
            );
        }
        let future = s.observe(p(4, 10_000_000, None), 0, 100, false).unwrap();
        assert_eq!(future.answers[0].time.unwrap().year(), Some(2024));
        assert_eq!(old, data.answers);
        let reset = s
            .observe(
                p(5, 0, Some(r(1, false, YearContext::default()))),
                0,
                100,
                true,
            )
            .unwrap();
        assert!(reset.findings.iter().any(|f| f.code == 20));
    }
}

// Added after implementation: explicit evidence/override and bounded-anchor probes.
#[test]
fn stream_year_and_ly_findings_keep_factual_validity_under_level_overrides() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut policy = sp(mode);
        policy
            .offline
            .finding_levels
            .insert(7, Severity::Information);
        policy.offline.finding_levels.insert(32, Severity::Error);
        let mut s = StreamTimeline::new(policy, None).unwrap();
        let ctx = YearContext {
            caller: Some(2023),
            recording: Some(2022),
        };
        let b = s
            .observe(p(7, 1234, Some(r(1, true, ctx))), 0, 100, false)
            .unwrap();
        for (id, level) in [(7, Severity::Information), (32, Severity::Error)] {
            let f = b.findings.iter().find(|f| f.code == id).unwrap();
            assert_eq!(f.severity, level);
            assert_eq!(f.evidence.validity(), ValueValidity::Kept);
            assert_eq!(f.raw_rtc, Some(Rtc::from_raw(1234)));
            assert_eq!(f.session, 1);
            assert_eq!(f.packet, 7);
        }
        let a = s.observe(p(8, 1234, None), 0, 100, false).unwrap();
        assert_eq!(a.answers[0].time.unwrap().year(), Some(2023));
    }
}
#[test]
fn contradictory_dmy_packets_keep_own_year_and_remain_candidates_for_future_doy() {
    let bytes = [1, 2, 0, 0, 0, 2, 0, 0, 1, 1, 0x25, 0x20]; //2025-01-01 00:00:02, LY0
    let newer = ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let packets = [
        p(1, 10_000_000, Some(dmy())),
        p(2, 20_000_000, Some(newer)),
        p(3, 30_000_000, Some(r(1, true, YearContext::default()))),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for p in &packets {
        b.observe(p.clone(), false).unwrap();
    }
    let t = b.finish().unwrap();
    let refs = &t.sessions()[0].references;
    assert_eq!(refs[1].input.year_basis.selected().unwrap().year, 2025);
    assert_eq!(
        refs[2]
            .input
            .year_basis
            .candidates()
            .iter()
            .map(|c| c.year)
            .collect::<Vec<_>>(),
        vec![2024, 2025]
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for p in &packets[..2] {
            s.observe(p.clone(), 0, 100, false).unwrap();
        }
        let b = s.observe(packets[2].clone(), 0, 100, false).unwrap();
        let f = b
            .findings
            .iter()
            .find(|f| f.code == 32 && f.packet == 3)
            .unwrap();
        assert_eq!(f.related, Some(1));
        assert!(f.evidence.related().iter().any(|l| l.packet == Some(1)));
        assert!(f.evidence.related().iter().any(|l| l.packet == Some(2)));
        let b = s.observe(p(4, 30_000_000, None), 0, 100, false).unwrap();
        let a = &b.answers[0];
        assert_eq!(a.time.unwrap().year(), Some(2024));
        assert_eq!(
            a.basis
                .preceding
                .as_ref()
                .unwrap()
                .input
                .year_basis
                .candidates()
                .iter()
                .map(|c| c.year)
                .collect::<Vec<_>>(),
            vec![2024, 2025]
        );
    }
}
#[test]
fn rejected_dmy_is_not_an_anchor_and_late_year_end_keeps_its_original_phase() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut policy = sp(mode);
        policy.offline.late_ns = 2 * 86_400_000_000_000;
        let mut s = StreamTimeline::new(policy, None).unwrap();
        let ctx = YearContext {
            caller: Some(2023),
            recording: None,
        };
        s.observe(p(1, 0, Some(r(365, false, ctx))), 0, 100, false)
            .unwrap();
        s.observe(
            p(2, 86400 * 10_000_000, Some(r(1, true, ctx))),
            0,
            100,
            false,
        )
        .unwrap();
        // Dec31 00:00:00.50 arrives after Jan1 midnight: RTC/day agree.
        let late_ref = review2_wire(&[1, 0, 0, 0, 0x50, 0, 0, 0, 0x65, 3]);
        let late = s
            .observe(p(3, 5_000_000, Some(late_ref)), 0, 100, false)
            .unwrap();
        assert!(!late.findings.iter().any(|f| f.code == 7));
        let a = s
            .observe(p(4, 86400 * 10_000_000, None), 0, 100, false)
            .unwrap();
        assert_eq!(a.answers[0].time.unwrap().year(), Some(2024));
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        let mut bad = dmy();
        bad.source = Some(TimeF1Source::Source(TimeSource::None));
        s.observe(p(1, 0, Some(bad)), 0, 100, false).unwrap();
        s.observe(
            p(2, 10_000_000, Some(r(1, false, YearContext::default()))),
            0,
            100,
            false,
        )
        .unwrap();
        assert_eq!(
            s.observe(p(3, 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            None
        );
    }
}
#[test]
fn other_channel_dmy_supplies_year_without_changing_stream_channel_or_absolute_anchor() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        s.observe(
            p(1, 0, Some(r(1, true, YearContext::default()))),
            0,
            100,
            false,
        )
        .unwrap();
        let before = s.observe(p(2, 2_500_000, None), 0, 100, false).unwrap();
        let frozen = before.answers.clone();
        let mut dated = p(3, 10_000_000, Some(dmy()));
        dated.channel = 2;
        s.observe(dated, 0, 100, false).unwrap();
        assert_eq!(s.channel().unwrap().0, 1);
        let after = s.observe(p(4, 5_000_000, None), 0, 100, false).unwrap();
        let answers = if mode == StreamMode::Standard {
            after.answers
        } else {
            s.finish(0).unwrap().answers
        };
        for a in &answers {
            assert_eq!(a.time.unwrap().year(), Some(2024));
            assert_eq!(a.basis.preceding.as_ref().unwrap().channel, 1);
            assert_eq!(a.basis.preceding.as_ref().unwrap().key.packet, 1);
            assert_eq!(
                a.time.unwrap().nanoseconds(),
                if a.basis.key.packet == 2 {
                    250_000_000
                } else {
                    500_000_000
                }
            );
        }
        assert!(!answers.is_empty());
        assert_eq!(frozen, before.answers);
        if mode == StreamMode::Standard {
            assert_eq!(frozen[0].time.unwrap().year(), None);
        }
    }
}
#[test]
fn newly_dated_held_answer_keeps_year_when_the_new_reference_causes_a_jump() {
    let mut s = StreamTimeline::new(sp(StreamMode::Interpolate), None).unwrap();
    s.observe(
        p(1, 0, Some(r(1, true, YearContext::default()))),
        0,
        100,
        false,
    )
    .unwrap();
    assert!(
        s.observe(p(2, 5_000_000, None), 0, 100, false)
            .unwrap()
            .answers
            .is_empty()
    );
    let bytes = [1, 3, 0, 0, 0, 2, 0, 0, 1, 1, 0x24, 0x20]; //2024Jan1 +2s at RTC1s: +1s jump
    let dated = ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let b = s
        .observe(p(3, 10_000_000, Some(dated)), 0, 100, false)
        .unwrap();
    assert_eq!(b.answers[0].basis.release, ReleaseReason::Discontinuity);
    assert_eq!(b.answers[0].time.unwrap().year(), Some(2024));
    assert_eq!(b.answers[0].time.unwrap().nanoseconds(), 500_000_000);
}
#[test]
fn changed_year_recomputes_explicit_normalization_or_records_missing_context() {
    for (year, expected) in [
        (2024, Some(1_704_067_237_000_000_000i128)),
        (2030, Some(1_893_456_037_000_000_000i128)),
        (1971, None),
    ] {
        let mut raw = r(1, false, YearContext::default());
        raw.value = ReferenceValue::Readable(
            AbsoluteTime::new(1, 0, 0, 0, 0)
                .unwrap()
                .with_year(Some(2023))
                .unwrap(),
            true,
        );
        raw.year_origin = YearOrigin::Caller;
        let raw = raw
            .normalize_to_tai(leap::LeapSecondTable::builtin())
            .unwrap();
        let bytes = if year == 2024 {
            [1, 3, 0, 0, 0, 1, 0, 0, 1, 1, 0x24, 0x20]
        } else if year == 2030 {
            [1, 2, 0, 0, 0, 1, 0, 0, 1, 1, 0x30, 0x20]
        } else {
            [1, 2, 0, 0, 0, 1, 0, 0, 1, 1, 0x71, 0x19]
        };
        let dated = ReferenceInput::format1(
            &bytes,
            12,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap();
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let k = b.observe(p(1, 0, Some(raw)), false).unwrap();
        b.observe(p(2, 10_000_000, Some(dated)), false).unwrap();
        let t = b.finish().unwrap();
        let input = &t.sessions()[0].references[0].input;
        assert_eq!(
            input.normalization.as_ref().map(|n| n.tai_nanoseconds()),
            expected
        );
        assert_eq!(
            t.findings()
                .iter()
                .any(|f| f.code == 21 && f.severity == Severity::Warning),
            year == 2030
        );
        assert_eq!(
            input.year_basis.normalization_issue().is_some(),
            expected.is_none()
        );
        assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), Some(year));
        if expected.is_none() {
            let q = PacketKey { ticks: 1, ..k };
            assert!(t.lookup(q).unwrap().time.is_none());
        }
    }
}
#[test]
fn readable_dmy_leap_label_supplies_year_without_becoming_an_ordinary_time_anchor() {
    // Fig11-14:2024Jun30 23:59:60, decoded as an explicit UTC leap label,
    // not a claim that IERS scheduled one. Its calendar year remains readable.
    let bytes = [1, 3, 0, 0, 0, 0x60, 0x59, 0x23, 0x30, 6, 0x24, 0x20];
    let dated = ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    assert_eq!(dated.value, ReferenceValue::NeedsContext);
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b
        .observe(p(1, 0, Some(r(182, true, YearContext::default()))), false)
        .unwrap();
    b.observe(p(2, 10_000_000, Some(dated.clone())), false)
        .unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), Some(2024));
    assert!(!t.sessions()[0].references[1].admitted);
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        s.observe(
            p(1, 0, Some(r(182, true, YearContext::default()))),
            0,
            100,
            false,
        )
        .unwrap();
        s.observe(p(2, 10_000_000, Some(dated.clone())), 0, 100, false)
            .unwrap();
        let a = s.observe(p(3, 0, None), 0, 100, false).unwrap();
        assert_eq!(a.answers[0].time.unwrap().year(), Some(2024));
        assert_eq!(a.answers[0].basis.preceding.as_ref().unwrap().key.packet, 1);
    }
}
#[test]
fn newly_invalid_day366_does_not_hide_a_previous_eligible_stream_reference() {
    const DAY_TICKS: u64 = 864_000_000_000;
    for committed in [false, true] {
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(sp(mode), None).unwrap();
            s.observe(
                p(1, 0, Some(r(365, false, YearContext::default()))),
                0,
                100,
                false,
            )
            .unwrap();
            s.observe(
                p(2, DAY_TICKS, Some(r(366, true, YearContext::default()))),
                0,
                100,
                false,
            )
            .unwrap();
            s.observe(p(3, DAY_TICKS + 5_000_000, None), 0, 100, false)
                .unwrap();
            // DMY2023Dec31 00:00:01: backward day jump, no year rollover.
            let bytes = [1, 2, 0, 0, 0, 1, 0, 0, 0x31, 0x12, 0x23, 0x20];
            let dated = ReferenceInput::format1(
                &bytes,
                12,
                TimeF1Layout::SourceWithIts17,
                ScalePolicy::default(),
            )
            .unwrap();
            let mut dated = p(4, DAY_TICKS + 10_000_000, Some(dated));
            dated.channel = if committed { 1 } else { 2 };
            let b = s.observe(dated, 0, 100, false).unwrap();
            assert!(b.findings.iter().any(|f| f.code == 4 && f.packet == 2));
            if committed && mode == StreamMode::Interpolate {
                let held = b.answers.iter().find(|a| a.basis.key.packet == 3).unwrap();
                assert_eq!(held.basis.release, ReleaseReason::Discontinuity);
                assert_eq!(held.time.unwrap().year(), Some(2024));
                assert_eq!(held.time.unwrap().nanoseconds(), 500_000_000);
                assert_eq!(held.basis.preceding.as_ref().unwrap().key.packet, 1);
            }
            let b = s
                .observe(p(5, DAY_TICKS + 20_000_000, None), 0, 100, false)
                .unwrap();
            let answers = if mode == StreamMode::Standard {
                b.answers
            } else {
                s.finish(0).unwrap().answers
            };
            let a = answers.iter().find(|a| a.basis.key.packet == 5).unwrap();
            assert_eq!(
                a.time.unwrap().year(),
                Some(if committed { 2023 } else { 2024 })
            );
            assert_eq!(
                a.time.unwrap().day_of_year(),
                if committed { 365 } else { 1 }
            );
            assert_eq!(a.time.unwrap().seconds(), 2);
            assert_eq!(
                a.basis.preceding.as_ref().unwrap().key.packet,
                if committed { 4 } else { 1 }
            );
        }
    }
}
#[test]
fn an_already_dated_recording_year_does_not_outrank_explicit_caller_context() {
    let mut raw = r(
        1,
        false,
        YearContext {
            caller: Some(2023),
            recording: None,
        },
    );
    raw.value = ReferenceValue::Readable(
        AbsoluteTime::new(1, 0, 0, 0, 0)
            .unwrap()
            .with_year(Some(2022))
            .unwrap(),
        true,
    );
    raw.year_origin = YearOrigin::RecordingDate;
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b.observe(p(1, 0, Some(raw)), false).unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), Some(2023));
    assert_eq!(
        t.sessions()[0].references[0]
            .input
            .year_basis
            .candidates()
            .iter()
            .map(|c| c.year)
            .collect::<Vec<_>>(),
        vec![2023, 2022]
    );
}

// Review regressions: Fig11-13 (11-25/PDF33), contract5.3 / COR-110 clauses.
// Partial evidence only; no whole-requirement Requirements marker.
fn review_year_end(year: u16, context: YearContext) -> ReferenceInput {
    // DOY365/366 23:59:59: word0=0x5900,word1=0x2359,word2=0x0365/366.
    let bytes = [
        1,
        u8::from(year == 2024),
        0,
        0,
        0,
        0x59,
        0x59,
        0x23,
        if year == 2024 { 0x66 } else { 0x65 },
        3,
    ];
    ReferenceInput::format1(
        &bytes,
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(context)
    .unwrap()
}
#[test]
fn review_sparse_rollovers_advance_offline_years() {
    for year in [2023, 2024] {
        let ctx = YearContext {
            caller: Some(year),
            recording: None,
        };
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(p(1, 0, Some(review_year_end(year, ctx))), false)
            .unwrap();
        // Dec31 23:59:59 -> Jan2 00:00:00 =86401s, no Jan1 reference.
        let k = b
            .observe(
                p(2, 86401 * 10_000_000, Some(r(2, year == 2023, ctx))),
                false,
            )
            .unwrap();
        let t = b.finish().unwrap();
        let a = t.lookup(k).unwrap().time.unwrap();
        assert_eq!((a.year(), a.day_of_year()), (Some(year + 1), 2));
        assert!(!t.findings().iter().any(|f| matches!(f.code, 7 | 17)));
    }
}
#[test]
fn review_sparse_rollovers_advance_stream_years() {
    for year in [2023, 2024] {
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let ctx = YearContext {
                caller: Some(year),
                recording: None,
            };
            let mut s = StreamTimeline::new(sp(mode), None).unwrap();
            s.observe(p(1, 0, Some(review_year_end(year, ctx))), 0, 100, false)
                .unwrap();
            let b = s
                .observe(
                    p(2, 86401 * 10_000_000, Some(r(2, year == 2023, ctx))),
                    0,
                    100,
                    false,
                )
                .unwrap();
            let a = s
                .observe(p(3, 86401 * 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap();
            assert_eq!((a.year(), a.day_of_year()), (Some(year + 1), 2));
            assert!(!b.findings.iter().any(|f| matches!(f.code, 7 | 17)));
        }
    }
}
#[test]
fn review_ri4_alone_advances_in_both_engines() {
    for year in [2023, 2024] {
        let ctx = YearContext {
            caller: None,
            recording: Some(year),
        };
        let packets = [
            p(1, 0, Some(review_year_end(year, ctx))),
            p(2, 10_000_000, Some(r(1, year == 2023, ctx))),
        ];
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let mut k = None;
        for packet in &packets {
            k = Some(b.observe(packet.clone(), false).unwrap());
        }
        let t = b.finish().unwrap();
        assert_eq!(
            t.lookup(k.unwrap()).unwrap().time.unwrap().year(),
            Some(year + 1)
        );
        let input = &t.sessions()[0].references[1].input;
        assert_eq!(input.year_origin, YearOrigin::RecordingDate);
        assert_eq!(input.year_basis.selected().unwrap().supplied, year);
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(sp(mode), None).unwrap();
            for packet in &packets {
                s.observe(packet.clone(), 0, 100, false).unwrap();
            }
            let a = s.observe(p(3, 10_000_000, None), 0, 100, false).unwrap();
            assert_eq!(a.answers[0].time.unwrap().year(), Some(year + 1));
            assert_eq!(
                a.answers[0]
                    .basis
                    .preceding
                    .as_ref()
                    .unwrap()
                    .input
                    .year_origin,
                YearOrigin::RecordingDate
            );
        }
    }
}
#[test]
fn review_late_sparse_year_end_keeps_previous_year() {
    for year in [2023, 2024] {
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let ctx = YearContext {
                caller: Some(year),
                recording: None,
            };
            let mut policy = sp(mode);
            policy.offline.late_ns = 2 * 86_400_000_000_000; // caller allows a late day, not a reset
            let mut s = StreamTimeline::new(policy, None).unwrap();
            s.observe(p(1, 0, Some(review_year_end(year, ctx))), 0, 100, false)
                .unwrap();
            s.observe(
                p(2, 86401 * 10_000_000, Some(r(2, year == 2023, ctx))),
                0,
                100,
                false,
            )
            .unwrap();
            // Fig11-13 word0=0x5950: Dec31 23:59:59.50 at RTC+.5s.
            let bytes = [
                1,
                u8::from(year == 2024),
                0,
                0,
                0x50,
                0x59,
                0x59,
                0x23,
                if year == 2024 { 0x66 } else { 0x65 },
                3,
            ];
            let late = ReferenceInput::format1(
                &bytes,
                10,
                TimeF1Layout::SourceWithIts17,
                ScalePolicy::default(),
            )
            .unwrap()
            .with_year_context(ctx)
            .unwrap();
            let b = s
                .observe(p(3, 5_000_000, Some(late)), 0, 100, false)
                .unwrap();
            let a = s.observe(p(4, 5_000_000, None), 0, 100, false).unwrap();
            assert_eq!(a.answers[0].time.unwrap().year(), Some(year));
            assert_eq!(a.answers[0].time.unwrap().nanoseconds(), 500_000_000);
            assert!(!b.findings.iter().any(|f| matches!(f.code, 7 | 17 | 18)));
        }
    }
}
#[test]
fn review_full_year_gap_needs_context_without_consuming_stream_state() {
    const DAY: u64 = 864_000_000_000;
    for days in [365, 400] {
        let ctx = YearContext {
            caller: Some(2023),
            recording: None,
        };
        let mut packets = vec![p(1, 0, Some(review_year_end(2023, ctx)))];
        // Each100-day step is < half a48-bit wrap; both engines can unwrap365/400days.
        for (id, ticks) in [
            (2, 100 * DAY),
            (3, 200 * DAY),
            (4, 300 * DAY),
            (5, days * DAY - 10_000_000),
        ] {
            packets.push(p(id, ticks, None));
        }
        let mut bad = r(2, false, ctx);
        bad.source = Some(TimeF1Source::Source(TimeSource::None));
        packets.push(p(6, days * DAY - 5_000_000, Some(bad))); // unusable observation is skipped
        let target = p(7, days * DAY, Some(r(2, false, ctx)));
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for packet in &packets {
            b.observe(packet.clone(), false).unwrap();
        }
        b.observe(target.clone(), false).unwrap();
        let t = b.finish().unwrap();
        assert_eq!(
            t.lookup(t.sessions()[0].references[0].key)
                .unwrap()
                .time
                .unwrap()
                .year(),
            Some(2023)
        );
        let post = t.sessions()[0].references.last().unwrap();
        assert!(post.admitted);
        assert_eq!(t.lookup(post.key).unwrap().time.unwrap().year(), None);
        assert_eq!(post.input.year_origin, YearOrigin::Unknown);
        assert_eq!(post.input.year_basis.context().caller, Some(2023));
        let expected_gap = if days == 365 {
            315_360_000_000_000i128
        } else {
            345_599_995_000_000
        };
        assert_eq!(
            post.input.year_basis.gap_ticks().unwrap().get(),
            expected_gap
        );
        assert_eq!(post.input.year_basis.continuity_segment(), 1);
        assert!(post.input.year_basis.year_issue().is_some());
        let finding = t
            .findings()
            .iter()
            .find(|f| f.code == 20 && f.packet == 7)
            .unwrap();
        assert_eq!(finding.severity, Severity::Information);
        assert_eq!(finding.evidence.validity(), ValueValidity::Kept);
        assert!(
            finding
                .evidence
                .raw_fields()
                .iter()
                .any(|v| v.name == "year.gap_ticks"
                    && v.observed == findings::RawValue::Signed(expected_gap))
        );
        assert!(
            finding
                .evidence
                .raw_fields()
                .iter()
                .any(|v| v.name == "year.session_caller"
                    && v.observed == findings::RawValue::Unsigned(2023))
        );
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(sp(mode), None).unwrap();
            for packet in &packets {
                s.observe(packet.clone(), 0, 100, false).unwrap();
            }
            assert!(matches!(
                s.observe(target.clone(), 2_000_000_000, 100, false),
                Err(TimeError::NeedsContext { .. })
            ));
            if mode == StreamMode::Interpolate {
                assert_eq!(
                    s.usage().packets,
                    4,
                    "gap rejection must precede progress/timeouts"
                );
                assert_eq!(s.finish(2_000_000_000).unwrap().answers.len(), 4);
            } else {
                // The rejected ID is reusable with explicit new-session/year context.
                let dated = p(
                    7,
                    days * DAY,
                    Some(r(
                        2,
                        false,
                        YearContext {
                            caller: Some(2025),
                            recording: None,
                        },
                    )),
                );
                s.observe(dated, 2_000_000_000, 100, true).unwrap();
            }
        }
    }
}
#[test]
fn review_stable_first_uses_observation_order_not_counter_order() {
    let bytes = [1, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x25, 0x20]; //2025Jan1, later observation/earlier RTC
    let late = ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let packets = [
        p(1, 0, Some(r(1, true, YearContext::default()))),
        p(2, 10_000_000, Some(dmy())),
        p(3, 5_000_000, Some(late)),
        p(4, 20_000_000, Some(r(1, true, YearContext::default()))),
        p(5, 30_000_000, Some(r(1, true, YearContext::default()))),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut key = None;
    for packet in &packets {
        key = Some(b.observe(packet.clone(), false).unwrap());
    }
    let t = b.finish().unwrap();
    assert_eq!(
        t.lookup(key.unwrap()).unwrap().time.unwrap().year(),
        Some(2024)
    );
    assert_eq!(
        t.sessions()[0].references[4]
            .input
            .year_basis
            .selected()
            .unwrap()
            .packet,
        Some(2)
    );
    for id in [4, 5] {
        assert!(t.findings().iter().any(|f| f.code == 32 && f.packet == id));
    }
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for packet in &packets {
            let b = s.observe(packet.clone(), 0, 100, false).unwrap();
            if packet.id >= 4 {
                assert!(
                    b.findings
                        .iter()
                        .any(|f| f.code == 32 && f.packet == packet.id)
                );
            }
        }
        let a = s.observe(p(6, 30_000_000, None), 0, 100, false).unwrap();
        assert_eq!(a.answers[0].time.unwrap().year(), Some(2024));
        assert_eq!(
            a.answers[0]
                .basis
                .preceding
                .as_ref()
                .unwrap()
                .input
                .year_basis
                .selected()
                .unwrap()
                .packet,
            Some(2)
        );
    }
    // Caller/RI4 base year also names the first observed calendar, not the
    // earliest-RTC late reference. Counter sorting must preserve that origin.
    let ctx = YearContext {
        caller: Some(2023),
        recording: None,
    };
    let packets = [
        p(1, 10_000_000, Some(r(100, false, ctx))),
        p(2, 5_000_000, Some(r(300, false, ctx))),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let key = b.observe(packets[0].clone(), false).unwrap();
    b.observe(packets[1].clone(), false).unwrap();
    assert_eq!(
        b.finish()
            .unwrap()
            .lookup(key)
            .unwrap()
            .time
            .unwrap()
            .year(),
        Some(2023)
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for packet in &packets {
            s.observe(packet.clone(), 0, 100, false).unwrap();
        }
        assert_eq!(
            s.observe(p(3, 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            Some(2023)
        );
    }
}

// Review2 clause evidence: Fig11-13 word groups, contract5.3 / COR-110.
fn review2_wire(bytes: &[u8]) -> ReferenceInput {
    ReferenceInput::format1(
        bytes,
        bytes.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(YearContext {
        caller: Some(2023),
        recording: None,
    })
    .unwrap()
}
#[test]
fn review2_backward_day_jump_keeps_year_in_both_engines() {
    // Day200 00:00:05 -> day199 23:59:59: reading -6s, RTC+1s => TF-017 residual-7s.
    let first = review2_wire(&[1, 0, 0, 0, 0, 5, 0, 0, 0, 2]);
    let back = review2_wire(&[1, 0, 0, 0, 0, 0x59, 0x59, 0x23, 0x99, 1]);
    let packets = [p(1, 0, Some(first)), p(2, 10_000_000, Some(back))];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut k = None;
    for packet in &packets {
        k = Some(b.observe(packet.clone(), false).unwrap());
    }
    let t = b.finish().unwrap();
    assert_eq!(
        t.lookup(k.unwrap()).unwrap().time.unwrap().year(),
        Some(2023)
    );
    assert!(
        t.findings()
            .iter()
            .any(|f| f.code == 17 && f.delta_ns == Some(-7_000_000_000))
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        s.observe(packets[0].clone(), 0, 100, false).unwrap();
        let b = s.observe(packets[1].clone(), 0, 100, false).unwrap();
        assert!(
            b.findings
                .iter()
                .any(|f| f.code == 17 && f.delta_ns == Some(-7_000_000_000))
        );
        assert_eq!(
            s.observe(p(3, 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            Some(2023)
        );
    }
}
#[test]
fn review2_equal_counter_day_change_is_a_jump_not_a_rollover() {
    // Dec31 23:59:59.99 -> Jan1:10ms cyclic interval, within default10ms
    // resolution tolerance of zero RTC, but positive RTC progress is mandatory.
    let first = review2_wire(&[1, 0, 0, 0, 0x99, 0x59, 0x59, 0x23, 0x65, 3]);
    let next = review2_wire(&[1, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    let packets = [p(1, 0, Some(first)), p(2, 0, Some(next))];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for packet in &packets {
        b.observe(packet.clone(), false).unwrap();
    }
    let t = b.finish().unwrap();
    let input = &t.sessions()[0].references[1].input;
    assert_eq!(input.year_basis.rollovers(), 0);
    assert!(matches!(input.value,ReferenceValue::Readable(t,_) if t.year()==Some(2023)));
    assert!(t.findings().iter().any(|f| f.code == 17));
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for packet in &packets {
            s.observe(packet.clone(), 0, 100, false).unwrap();
        }
        assert_eq!(
            s.observe(p(3, 0, None), 0, 100, false).unwrap().answers[0]
                .time
                .unwrap()
                .year(),
            Some(2023)
        );
    }
}
#[test]
fn review2_same_day_late_reference_does_not_lose_a_year() {
    // Day200 00:00:04.50 arrives after00:00:05, half a second earlier in RTC.
    let packets = [
        p(
            1,
            10_000_000,
            Some(review2_wire(&[1, 0, 0, 0, 0, 5, 0, 0, 0, 2])),
        ),
        p(
            2,
            5_000_000,
            Some(review2_wire(&[1, 0, 0, 0, 0x50, 4, 0, 0, 0, 2])),
        ),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut k = None;
    for packet in &packets {
        k = Some(b.observe(packet.clone(), false).unwrap());
    }
    let t = b.finish().unwrap();
    assert_eq!(
        t.lookup(k.unwrap()).unwrap().time.unwrap().year(),
        Some(2023)
    );
    assert!(!t.findings().iter().any(|f| f.code == 17));
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for packet in &packets {
            assert!(
                !s.observe(packet.clone(), 0, 100, false)
                    .unwrap()
                    .findings
                    .iter()
                    .any(|f| f.code == 17)
            );
        }
        let a = s.observe(p(3, 5_000_000, None), 0, 100, false).unwrap();
        assert_eq!(a.answers[0].time.unwrap().year(), Some(2023));
        assert_eq!(a.answers[0].time.unwrap().seconds(), 4);
        assert_eq!(a.answers[0].time.unwrap().nanoseconds(), 500_000_000);
    }
}
#[test]
fn review2_adjacent_ordinal_days_can_span_a_real_year_boundary() {
    const DAY: u64 = 864_000_000_000;
    // Jan2 2023 23:59:59 -> Jan1 2024 midnight =363days+1s;
    // bridge RTC wraps with <=100day data steps. day1+1<day2 would miss this.
    let mut packets = vec![p(
        1,
        0,
        Some(review2_wire(&[1, 0, 0, 0, 0, 0x59, 0x59, 0x23, 2, 0])),
    )];
    for step in 1..=3 {
        packets.push(p(step + 1, step * 100 * DAY, None));
    }
    packets.push(p(
        5,
        363 * DAY + 10_000_000,
        Some(r(
            1,
            true,
            YearContext {
                caller: Some(2023),
                recording: None,
            },
        )),
    ));
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut k = None;
    for packet in &packets {
        k = Some(b.observe(packet.clone(), false).unwrap());
    }
    assert_eq!(
        b.finish()
            .unwrap()
            .lookup(k.unwrap())
            .unwrap()
            .time
            .unwrap()
            .year(),
        Some(2024)
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for packet in &packets {
            s.observe(packet.clone(), 0, 100, false).unwrap();
        }
        assert_eq!(
            s.observe(p(6, 363 * DAY + 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            Some(2024)
        );
    }
}
#[test]
fn review2_rollover_clock_tolerance_is_explicit_and_inclusive() {
    for (threshold, year) in [(0, 2023), (99, 2023), (100, 2024)] {
        let first = review2_wire(&[1, 0, 0, 0, 0x99, 0x59, 0x59, 0x23, 0x65, 3]);
        let packets = [
            p(1, 0, Some(first)),
            p(
                2,
                100_001,
                Some(review2_wire(&[1, 0, 0, 0, 0, 0, 0, 0, 1, 0])),
            ),
        ];
        // Cyclic calendar10ms versus RTC10ms+100ns: residual-100ns.
        let policy = OfflinePolicy {
            jump_threshold_ns: Some(threshold),
            ..Default::default()
        };
        let mut b = TimelineBuilder::new(policy).unwrap();
        for packet in &packets {
            b.observe(packet.clone(), false).unwrap();
        }
        let t = b.finish().unwrap();
        assert!(
            matches!(t.sessions()[0].references[1].input.value,ReferenceValue::Readable(t,_) if t.year()==Some(year))
        );
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut policy = sp(mode);
            policy.offline.jump_threshold_ns = Some(threshold);
            let mut s = StreamTimeline::new(policy, None).unwrap();
            for packet in &packets {
                s.observe(packet.clone(), 0, 100, false).unwrap();
            }
            assert_eq!(
                s.observe(p(3, 100_001, None), 0, 100, false)
                    .unwrap()
                    .answers[0]
                    .time
                    .unwrap()
                    .year(),
                Some(year)
            );
        }
    }
}
fn review2_gap_builder(post: ReferenceInput) -> (TimelineBuilder, PacketKey) {
    const DAY: u64 = 864_000_000_000;
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let key = b
        .observe(
            p(
                1,
                0,
                Some(review_year_end(
                    2023,
                    YearContext {
                        caller: Some(2023),
                        recording: None,
                    },
                )),
            ),
            false,
        )
        .unwrap();
    for step in 1..=4 {
        b.observe(p(step + 1, step * 100 * DAY, None), false)
            .unwrap();
    }
    b.observe(p(6, 400 * DAY + 10_000_000, Some(post)), false)
        .unwrap();
    (b, key)
}
#[test]
fn review2_gap_recovery_uses_new_dated_evidence_not_the_pre_gap_anchor() {
    let (mut b, pre) = review2_gap_builder(r(
        2,
        false,
        YearContext {
            caller: Some(2023),
            recording: None,
        },
    ));
    let dated = review2_wire(&[1, 2, 0, 0, 0, 1, 0, 0, 2, 1, 0x25, 0x20]); //2025Jan2 00:00:01
    b.observe(p(7, 400 * 864_000_000_000 + 20_000_000, Some(dated)), false)
        .unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(pre).unwrap().time.unwrap().year(), Some(2023));
    let post = &t.sessions()[0].references[1];
    assert_eq!(t.lookup(post.key).unwrap().time.unwrap().year(), Some(2025));
    assert_eq!(post.input.year_basis.continuity_segment(), 1);
    assert!(post.input.year_basis.year_issue().is_none());
    assert_eq!(post.input.year_basis.context().caller, Some(2023));
}
#[test]
fn review2_gap_clears_normalization_of_an_inferred_year() {
    let mut seed = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    seed.observe(
        p(
            1,
            0,
            Some(r(
                2,
                false,
                YearContext {
                    caller: Some(2023),
                    recording: None,
                },
            )),
        ),
        false,
    )
    .unwrap();
    let seed = seed.finish().unwrap();
    let post = seed.sessions()[0].references[0]
        .input
        .clone()
        .normalize_to_tai(leap::LeapSecondTable::builtin())
        .unwrap();
    let (b, pre) = review2_gap_builder(post);
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(pre).unwrap().time.unwrap().year(), Some(2023));
    let post = &t.sessions()[0].references[1];
    assert!(post.input.normalization.is_none());
    assert_eq!(t.lookup(post.key).unwrap().time.unwrap().year(), None);
    assert!(post.input.year_basis.year_issue().is_some());
    assert!(post.input.year_basis.gap_ticks().unwrap().get() >= 365 * 864_000_000_000);
}
#[test]
fn review2_explicit_leap_label_retains_the_second_before_new_year() {
    // A format-allowed explicit2023Dec31 23:59:60 label, not an IERS claim.
    let bytes = [1, 2, 0, 0, 0, 0x60, 0x59, 0x23, 0x31, 0x12, 0x23, 0x20];
    let leap = ReferenceInput::format1(
        &bytes,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let packets = [
        p(1, 0, Some(leap)),
        p(2, 10_000_000, Some(r(1, true, YearContext::default()))),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut k = None;
    for packet in &packets {
        k = Some(b.observe(packet.clone(), false).unwrap());
    }
    assert_eq!(
        b.finish()
            .unwrap()
            .lookup(k.unwrap())
            .unwrap()
            .time
            .unwrap()
            .year(),
        Some(2024)
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        for packet in &packets {
            s.observe(packet.clone(), 0, 100, false).unwrap();
        }
        assert_eq!(
            s.observe(p(3, 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            Some(2024)
        );
    }
}
#[test]
fn review2_unknown_year_backward_jump_keeps_partial_and_reports_residual() {
    let unknown = YearContext::default();
    let packets = [
        p(
            1,
            0,
            Some(
                review2_wire(&[1, 0, 0, 0, 0, 5, 0, 0, 0, 2])
                    .with_year_context(unknown)
                    .unwrap(),
            ),
        ),
        p(
            2,
            10_000_000,
            Some(
                review2_wire(&[1, 0, 0, 0, 0, 0x59, 0x59, 0x23, 0x99, 1])
                    .with_year_context(unknown)
                    .unwrap(),
            ),
        ),
    ];
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut key = None;
    for packet in &packets {
        key = Some(b.observe(packet.clone(), false).unwrap());
    }
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(key.unwrap()).unwrap().time.unwrap().year(), None);
    assert!(
        t.findings()
            .iter()
            .any(|f| f.code == 17 && f.delta_ns == Some(-7_000_000_000))
    );
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(sp(mode), None).unwrap();
        s.observe(packets[0].clone(), 0, 100, false).unwrap();
        let b = s.observe(packets[1].clone(), 0, 100, false).unwrap();
        assert!(
            b.findings
                .iter()
                .any(|f| f.code == 17 && f.delta_ns == Some(-7_000_000_000))
        );
        assert_eq!(
            s.observe(p(3, 10_000_000, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            None
        );
    }
}
#[test]
fn review2_unknown_year_jump_checks_never_bridge_a_year_gap() {
    let first = review_year_end(2023, YearContext::default());
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(p(1, 0, Some(first)), false).unwrap();
    for step in 1..=4 {
        b.observe(p(step + 1, step * 100 * 864_000_000_000, None), false)
            .unwrap();
    }
    let k = b
        .observe(
            p(
                6,
                400 * 864_000_000_000 + 10_000_000,
                Some(r(2, false, YearContext::default())),
            ),
            false,
        )
        .unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(k).unwrap().time.unwrap().year(), None);
    assert!(t.findings().iter().any(|f| f.code == 20 && f.packet == 6));
    assert!(
        !t.findings().iter().any(|f| f.code == 17),
        "no signed year comparison across an unknown cycle count"
    );
}
