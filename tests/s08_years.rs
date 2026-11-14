//! S08 clause evidence: Fig11-13/14 (11-25/PDF33), LY §11.2.3.2a,
//! Ch9 Table9-4 RI4 and contract5.3; D9 owner policy. No encoder oracle.
use irig106_time::{findings::ValueValidity, stream::*, timeline::*, years::*, *};
fn r(day: u16, ly: bool, ctx: YearContext) -> ReferenceInput {
    // Hand-positioned DOY decimal digits in word2; clock00:00:00.
    let word = (day / 100) << 8 | ((day / 10) % 10) << 4 | day % 10;
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
            .observe(p(2, 10_000_000, Some(r(1, year != 2024, ctx))), false)
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
    b.observe(p(2, 10_000_000, Some(dmy())), false).unwrap();
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
