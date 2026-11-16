//! S10b: Ch11 §11.2.1.1g /1.3b, printed11-9/11 (PDF17/19);
//! Ch4 §4.7, Ch9 Table9-11. Caller supplies extraction, never a body walker.
use irig106_time::{
    ch4::*, extracted::*, findings::*, own_time::*, rtc::CounterOrder, scale::*,
    secondary_packet::SecondaryPolicy, timeline::*, *,
};
fn origin(ticks: i128) -> ExtractionOrigin {
    ExtractionOrigin {
        key: PacketKey {
            session: 1,
            packet: 2,
            ticks,
        },
        channel: 7,
        byte_offset: Some(100),
        field: "synthetic extracted message time".into(),
    }
}
fn reference() -> ReferenceInput {
    ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(leap::LeapSecondTable::builtin())
    .unwrap()
}
fn obs(id: u64, rtc: u64, r: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(rtc),
        kind: if r.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference: r,
    }
}
fn timeline(start: u64) -> OfflineTimeline {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(obs(1, start, Some(reference())), false).unwrap();
    b.finish().unwrap()
}
fn epoch() -> EpochContext {
    EpochContext {
        origin: format1::CalendarReading {
            year: Some(2017),
            day_of_year: 1,
            hours: 0,
            minutes: 0,
            seconds: 0,
            nanoseconds: 0,
            scale: ScaleEvidence::standard(TimeScale::Utc),
        },
        counting: EpochCounting::Continuous,
        source: "explicit caller origin".into(),
    }
}
fn ctx() -> OwnContext {
    OwnContext {
        ieee: Some(epoch()),
        ch4: Some(epoch()),
        ..Default::default()
    }
}
fn stamp(flags: u8, b: &[u8], ticks: i128) -> StampInput<'_> {
    StampInput {
        origin: origin(ticks),
        query_id: 2,
        flags,
        bytes: b,
        order: CounterOrder::Nearest,
    }
}
fn absolute(flags: u8, b: &[u8], c: &OwnContext, p: &SecondaryPolicy) -> StampAnswer {
    inspect_stamp(stamp(flags, b, 0), p)
        .unwrap()
        .resolve(c, |_| panic!("absolute must not query"))
        .unwrap()
}
fn word_answer(w: TimeWord, c: &WordContext, p: &SecondaryPolicy) -> WordAnswer {
    answer_word(origin(0), w, c, p).unwrap()
}
fn finding(fs: &[Finding], id: u8) -> &Finding {
    fs.iter().find(|f| f.code().number() == id).unwrap()
}
fn net(w: NetworkWord, c: &WordContext, p: &SecondaryPolicy) -> WordAnswer {
    word_answer(TimeWord::Network(w), c, p)
}
fn codes(fs: &[Finding]) -> Vec<u8> {
    let mut a: Vec<_> = fs.iter().map(|f| f.code().number()).collect();
    a.sort();
    a
}
/// Requirements: L1-IPT-007, L2-IPT-107, L3-IPT-107
#[test]
fn all_flags_preserve_simultaneous_findings_and_value_validity() {
    // Independent truth table: groups00,40,80,c0 and format00/01/10/11.
    // 0=RTC,1=Ch4,2=IEEE,3=ERTC,4=unreadable; bits encode expected TF15/6/28.
    const TABLE: [(u8, u8); 16] = [
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 2),
        (4, 1),
        (4, 1),
        (4, 1),
        (4, 7),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 6),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 6),
    ];
    let t = timeline(0);
    let b = [0; 8];
    for flags in 0..=255u8 {
        let (kind, mask) = TABLE[usize::from(flags >> 6) * 4 + usize::from((flags >> 2) & 3)];
        let mut p = SecondaryPolicy::default();
        for c in [4, 5, 6, 15, 24, 28] {
            p.levels.insert(c, Severity::Information);
        }
        let r = inspect_stamp(stamp(flags, &b, 0), &p).unwrap();
        let mut want = Vec::new();
        for (bit, id) in [(1, 15), (2, 6), (4, 28)] {
            if mask & bit != 0 {
                want.push(id);
            }
        }
        if flags & 32 != 0 {
            want.push(24);
        }
        want.sort();
        assert_eq!(codes(r.findings()), want, "{flags:02x}");
        assert_eq!(r.time().is_some(), kind != 4);
        for f in r.findings() {
            assert_eq!(f.severity(), Severity::Information);
            assert_eq!(f.evidence().location().packet, Some(2));
            assert_eq!(f.evidence().location().channel, Some(7));
            assert_eq!(
                f.evidence().validity(),
                if matches!(f.code().number(), 15 | 28) {
                    ValueValidity::Unusable
                } else {
                    ValueValidity::Kept
                }
            );
            assert_eq!(f.evidence().location().byte_offset, None);
        }
        let defaults = inspect_stamp(stamp(flags, &b, 0), &Default::default()).unwrap();
        for f in defaults.findings() {
            assert_eq!(
                f.severity(),
                if matches!(f.code().number(), 15 | 28) {
                    Severity::Error
                } else {
                    Severity::Warning
                }
            );
        }
        // Default ERROR028 describes the unreadable header, not the RTC stamp.
        assert_eq!(
            defaults
                .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::offline))
                .unwrap()
                .point()
                .is_some(),
            kind != 4
        );
        let a = r
            .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::offline))
            .unwrap();
        assert_eq!(a.point().is_some(), kind != 4, "{flags:02x}");
    }
    for (flags, expected) in [
        (0x0c, vec![6]),
        (0x8c, vec![6, 28]),
        (0x4c, vec![6, 15, 28]),
        (0xcc, vec![6, 28]),
        (0x40, vec![15]),
    ] {
        assert_eq!(
            codes(
                inspect_stamp(stamp(flags, &b, 0), &Default::default())
                    .unwrap()
                    .findings()
            ),
            expected
        );
    }
}
/// Requirements: L1-ANS-003, L2-ANS-103, L3-ANS-103
#[test]
fn stamp_forms_query_their_clock_and_keep_partial_and_failure_evidence() {
    let t = timeline(0);
    for (flags, b, ns) in [
        (0, [1, 0, 0, 0, 0, 0, 0, 0], 100i128),
        (0xc8, 199u64.to_le_bytes(), 199),
        (0xc0, [0, 0, 0, 0, 100, 0, 0, 0], 1_000_000_000),
        (0xc4, [1, 0, 0, 0, 1, 0, 0, 0], 1_000_000_001),
    ] {
        let r = inspect_stamp(stamp(flags, &b, 0), &Default::default()).unwrap();
        let a = r
            .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::offline))
            .unwrap();
        assert_eq!(
            a.point().unwrap().tai_ns().get(),
            1_483_228_837_000_000_000 + ns
        );
        assert_eq!(a.origin().field, origin(0).field);
    }
    let b = [0; 8];
    for flags in [0xc0, 0xc4] {
        let a = inspect_stamp(stamp(flags, &b, 0), &Default::default())
            .unwrap()
            .resolve(&OwnContext::default(), |_| {
                panic!("absolute stamp must not query counters")
            })
            .unwrap();
        assert!(a.point().is_none());
        assert!(a.issue().is_some());
        assert!(a.inspection().time().is_some());
    }
    let only_ieee = OwnContext {
        ieee: Some(epoch()),
        ..Default::default()
    };
    assert!(
        absolute(0xc0, &b, &only_ieee, &Default::default())
            .point()
            .is_none()
    );
    for n in 0..=8 {
        let a = inspect_stamp(stamp(0x4c, &b[..n], 0), &Default::default()).unwrap();
        assert_eq!(
            codes(a.findings()),
            if n < 8 {
                vec![4, 6, 15, 28]
            } else {
                vec![6, 15, 28]
            }
        );
        assert_eq!(a.origin().key.packet, 2);
        assert!(a.time().is_none());
    }
    let mut no = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    no.observe(obs(1, 0, None), false).unwrap();
    let no = no.finish().unwrap();
    for flags in [0, 0xc8] {
        let a = inspect_stamp(stamp(flags, &b, 0), &Default::default())
            .unwrap()
            .resolve(&ctx(), |k| no.lookup(k).map(ResolvedCounter::offline))
            .unwrap();
        assert!(a.point().is_none());
        assert!(a.counter().is_some());
        assert!(a.issue().is_some());
        assert_eq!(a.counter_offset().unwrap().get(), 0);
    }
    let t = timeline((1 << 48) - 1);
    let a = inspect_stamp(stamp(0, &b, (1i128 << 48) - 1), &Default::default()).unwrap();
    assert_eq!(a.query().unwrap().ticks, 1i128 << 48);
    let a = a
        .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::offline))
        .unwrap();
    assert_eq!(a.point().unwrap().tai_ns().get(), 1_483_228_837_000_000_100);
}
/// Requirements: L1-ANS-004, L2-ANS-104, L3-ANS-104
#[test]
fn word_context_is_explicit_and_network_components_never_default_to_zero() {
    let o = origin(0);
    let w = TimeWord::Binary(Ch4Words {
        high: 0,
        low: 100,
        microseconds: 0,
    });
    let mut c = WordContext::new(1);
    let a = word_answer(w, &c, &Default::default());
    assert!(a.calendar().is_none());
    assert!(a.issue().is_some());
    c.own.ieee = Some(epoch()); // Never borrow the unrelated IEEE epoch for binary.
    assert!(word_answer(w, &c, &Default::default()).calendar().is_none());
    c.own = ctx();
    let a = word_answer(w, &c, &Default::default());
    assert_eq!(a.calendar().unwrap().seconds, 1);
    assert_eq!(a.origin(), &o);
    // Fig4-4: high day-fragment1, all time digits0; explicit full day/year.
    let bcd = TimeWord::Bcd(Ch4Words {
        high: 0x2000,
        low: 0,
        microseconds: 0,
    });
    assert!(
        word_answer(bcd, &c, &Default::default())
            .calendar()
            .is_none()
    );
    c.bcd = BcdContext {
        year: Some(2024),
        full_day: Some(1),
        interpretation: Some(BcdDayInterpretation::LowThreePackedBcdBits),
        edition: Some(edition::PacketEdition::V24R1),
    };
    c.bcd_scale = TimeScale::Utc;
    let b = word_answer(bcd, &c, &Default::default());
    assert_eq!(b.calendar().unwrap().year, Some(2024));
    assert!(b.chapter4().unwrap().bcd_wording_unresolved());
    c.bcd.year = None;
    assert!(
        word_answer(bcd, &c, &Default::default())
            .calendar()
            .is_none()
    );
    let w = TimeWord::Network(NetworkWord::NtpSeconds(2_208_988_800));
    let a = word_answer(w, &c, &Default::default());
    assert!(a.calendar().is_none());
    c.matching = Some(ComponentBinding {
        target: o.clone(),
        other: ExtractionOrigin {
            field: "fraction from same measurement".into(),
            ..o.clone()
        },
        word: NetworkWord::NtpFraction(0x80000000),
    });
    let a = word_answer(w, &c, &Default::default());
    assert_eq!(a.calendar().unwrap().year, Some(1970));
    assert_eq!(a.calendar().unwrap().nanoseconds, 500_000_000);
}
/// Requirements: L1-WORD-002, L2-WORD-102, L3-WORD-102
/// Interpretations: RDG-012
#[test]
fn all_six_network_kinds_use_matching_components_and_qualified_conversions() {
    use format2::NetworkConversion;
    use ntp::*;
    let o = origin(0);
    let mut c = WordContext::new(1);
    let p = PtpTime {
        seconds: 1_483_228_837,
        nanoseconds: 500_000_000,
    };
    let n = NtpTime {
        seconds: 2_208_988_800,
        fraction: 0x80000000,
    };
    for (word, other, ptp) in [
        (NetworkWord::Ptp(p), None, true),
        (
            NetworkWord::PtpSeconds(p.seconds),
            Some(NetworkWord::PtpNanoseconds(p.nanoseconds)),
            true,
        ),
        (
            NetworkWord::PtpNanoseconds(p.nanoseconds),
            Some(NetworkWord::PtpSeconds(p.seconds)),
            true,
        ),
        (NetworkWord::Ntp(n), None, false),
        (
            NetworkWord::NtpSeconds(n.seconds),
            Some(NetworkWord::NtpFraction(n.fraction)),
            false,
        ),
        (
            NetworkWord::NtpFraction(n.fraction),
            Some(NetworkWord::NtpSeconds(n.seconds)),
            false,
        ),
    ] {
        c.matching = None;
        let a = net(word, &c, &Default::default());
        assert_eq!(a.calendar().is_none(), other.is_some());
        c.matching = other.map(|word| ComponentBinding {
            target: o.clone(),
            other: ExtractionOrigin {
                field: "matching other component".into(),
                ..o.clone()
            },
            word,
        });
        let a = net(word, &c, &Default::default());
        assert_eq!(a.word(), TimeWord::Network(word));
        assert_eq!(a.calendar().unwrap().nanoseconds, 500_000_000);
        match a.network().unwrap() {
            NetworkConversion::Ptp(v) => {
                assert!(ptp);
                assert_eq!(v.raw(), p);
                assert_eq!(v.utc().unwrap().seconds, 0);
                assert_eq!(a.calendar().unwrap().seconds, 37);
                assert_eq!(
                    a.calendar().unwrap().scale,
                    ScaleEvidence::standard(TimeScale::Tai)
                );
            }
            NetworkConversion::Ntp(v) => {
                assert!(!ptp);
                assert_eq!(v.raw(), n);
                assert_eq!(v.unix_seconds().unwrap().get(), 0);
                assert_eq!(v.basis().era_basis, Some(EraBasis::AssumedZero));
            }
            _ => panic!(),
        }
        assert_eq!(a.context().matching, c.matching);
    }
    c.matching = None;
    for (era, sec, year, day) in [(0, 2_272_060_800, 1972, 1), (1, 63_104, 2036, 39)] {
        c.ntp.declared = Some(EraDeclaration {
            era,
            id: "caller era".into(),
        });
        let a = net(
            NetworkWord::Ntp(NtpTime {
                seconds: sec,
                fraction: 0,
            }),
            &c,
            &Default::default(),
        );
        assert_eq!(
            (
                a.calendar().unwrap().year,
                a.calendar().unwrap().day_of_year
            ),
            (Some(year), day)
        );
    }
    c.ntp.declared = None;
    c.ntp
        .pivots
        .push(NtpPivot::dated(1, "dated reference", 1, epoch().origin).unwrap());
    let a = net(
        NetworkWord::Ntp(NtpTime {
            seconds: 3_692_304_000,
            fraction: 0,
        }),
        &c,
        &Default::default(),
    );
    let Some(NetworkConversion::Ntp(v)) = a.network() else {
        panic!()
    };
    assert_eq!(v.basis().era_basis, Some(EraBasis::PivotDerived));
    assert_eq!(v.unix_seconds().unwrap().get(), 1_483_315_200);
    let mut policy = SecondaryPolicy::default();
    for id in [21, 29, 30] {
        policy.levels.insert(id, Severity::Information);
    }
    for (seconds, fraction, id) in [(0, 0, 29), (3_692_217_600, 0x80000000, 30)] {
        let a = net(NetworkWord::Ntp(NtpTime { seconds, fraction }), &c, &policy);
        let f = finding(a.findings(), id);
        assert_eq!(f.severity(), Severity::Information);
        assert_eq!(
            f.evidence().validity(),
            if id == 29 {
                ValueValidity::Unusable
            } else {
                ValueValidity::Kept
            }
        );
        assert_eq!(a.calendar().is_none(), id == 29);
    }
    // 2028-01-01 TAI seconds1830297600, last known offset37; stale UTC kept.
    let raw = PtpTime {
        seconds: 1_830_297_600,
        nanoseconds: 1,
    };
    let w = TimeWord::Network(NetworkWord::Ptp(raw));
    let a = word_answer(w, &c, &policy);
    let Some(NetworkConversion::Ptp(v)) = a.network() else {
        panic!()
    };
    assert_eq!(v.status(), leap::PtpConversionStatus::PossiblyOutOfDate);
    assert_eq!(v.basis().offset.unwrap().get(), 37);
    assert!(v.utc().is_some());
    assert_eq!(finding(a.findings(), 21).severity(), Severity::Information);
    c.own.table = leap::LeapSecondTable::empty();
    let a = word_answer(w, &c, &policy);
    let Some(NetworkConversion::Ptp(v)) = a.network() else {
        panic!()
    };
    assert_eq!(v.raw(), raw);
    assert!(v.utc().is_none());
    assert!(a.calendar().is_some());
    assert!(a.issue().is_some());
}
#[test]
fn reserved_fields_malformed_words_and_wrong_binding_do_not_invent_times() {
    let t = timeline(0);
    let b = [1, 0, 0, 0, 0, 0, 0x55, 0xaa];
    let mut tail = b.to_vec();
    tail.extend([0x11, 0x22, 0x33]);
    let mut input = stamp(0, &tail, 0);
    input.origin.channel = 11;
    let r = inspect_stamp(input, &Default::default()).unwrap();
    assert_eq!(r.raw(), b);
    assert_eq!(r.origin().channel, 11);
    let f = finding(r.findings(), 5);
    assert_eq!(f.evidence().validity(), ValueValidity::Kept);
    assert_eq!(f.evidence().location().byte_offset, Some(106));
    assert_eq!(r.raw(), b);
    let a = r
        .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::offline))
        .unwrap();
    assert_eq!(a.counter_offset().unwrap().get(), 100);
    for b in [[1, 0, 0, 0, 100, 0, 0, 0], [0, 0, 0x10, 0x27, 0, 0, 0, 0]] {
        let a = inspect_stamp(stamp(0xc0, &b, 0), &Default::default())
            .unwrap()
            .resolve(&ctx(), |_| panic!())
            .unwrap();
        if b[0] == 1 {
            assert!(a.point().is_some());
            assert!(codes(a.inspection().findings()).contains(&5));
        } else {
            assert!(a.point().is_none());
            assert!(codes(a.inspection().findings()).contains(&4));
        }
    }
    let o = origin(0);
    let mut c = WordContext::new(1);
    for word in [
        TimeWord::Network(NetworkWord::PtpNanoseconds(1_000_000_000)),
        TimeWord::Network(NetworkWord::Ptp(PtpTime {
            seconds: 1,
            nanoseconds: 1_000_000_000,
        })),
        TimeWord::Binary(Ch4Words {
            high: 0,
            low: 0,
            microseconds: 10000,
        }),
    ] {
        let a = word_answer(word, &c, &Default::default());
        assert!(codes(a.findings()).contains(&4));
        assert_eq!(a.word(), word);
        assert!(a.calendar().is_none());
    }
    c.matching = Some(ComponentBinding {
        target: o.clone(),
        other: o.clone(),
        word: NetworkWord::NtpFraction(0x80000000),
    });
    for mismatch in 0..3 {
        let mut wrong = c.clone();
        match mismatch {
            0 => wrong.matching.as_mut().unwrap().target.key.packet += 1,
            1 => wrong.matching.as_mut().unwrap().other.key.session += 1,
            _ => wrong.matching.as_mut().unwrap().word = NetworkWord::PtpSeconds(1),
        }
        let a = net(
            NetworkWord::NtpSeconds(2_208_988_800),
            &wrong,
            &Default::default(),
        );
        assert!(a.calendar().is_none());
        assert!(a.issue().is_some());
    }
    let r = inspect_stamp(stamp(0, &[0; 8], 0), &Default::default()).unwrap();
    assert!(
        r.resolve(&ctx(), |k| {
            let mut a = t.lookup(k)?;
            a.basis.key.session += 1;
            Ok(ResolvedCounter::offline(a))
        })
        .is_err()
    );
}
#[test]
fn stream_stamp_queries_keep_interpolation_and_the_original_release_basis() {
    use stream::*;
    for (mode, ticks, span, mapped) in [
        (StreamMode::Standard, 5_000_000, 1_000_010_000, 500_000_000),
        (
            StreamMode::Interpolate,
            5_000_000,
            1_000_010_000,
            500_005_000,
        ),
        (
            StreamMode::Interpolate,
            9_090_909,
            1_000_000_011,
            909_090_909,
        ),
    ] {
        let mut p = StreamPolicy::new(
            17,
            BufferCaps {
                packets: 2,
                bytes: 200,
                references: 3,
            },
        );
        p.mode = mode;
        p.offline.channel = Some(7);
        p.offline.jump_threshold_ns = Some(20_000_000);
        let mut s = StreamTimeline::new(p, None).unwrap();
        s.observe(obs(1, 0, Some(reference())), 0, 24, false)
            .unwrap();
        let mut q = s.observe(obs(2, ticks, None), 1, 24, false).unwrap();
        q.answers.extend(
            s.observe(obs(3, ticks, None), 1, 24, false)
                .unwrap()
                .answers,
        );
        let queries = if mode == StreamMode::Standard {
            q.answers
        } else {
            let mut r = reference();
            let ReferenceValue::Readable(t, _) = r.value else {
                panic!()
            };
            r.value = ReferenceValue::Readable(t.add_nanos(span).unwrap(), true);
            r = r
                .normalize_to_tai(leap::LeapSecondTable::builtin())
                .unwrap();
            s.observe(obs(4, 10_000_000, Some(r)), 2, 24, false)
                .unwrap()
                .answers
        };
        for ((flags, b, remainder), query) in [
            (0x20, ticks.to_le_bytes(), 0),
            (
                0xc8,
                (ticks * 100 + 99).to_le_bytes(),
                if ticks == 9_090_909 { 100 } else { 99 },
            ),
        ]
        .into_iter()
        .zip(queries)
        {
            // Two queries from physical packet2 retain independent core IDs.
            let mut input = stamp(flags, &b, 10_000_000);
            input.query_id = query.basis.key.packet;
            let r = inspect_stamp(input, &Default::default()).unwrap();
            assert_eq!(r.origin().key.packet, 2);
            assert_eq!(r.query_id(), query.basis.key.packet);
            assert_eq!(r.query().unwrap(), query.basis.key);
            assert!(!codes(r.findings()).contains(&25));
            if flags == 0x20 {
                assert_eq!(
                    r.findings()[0].evidence().raw_rtc(),
                    Some(Rtc::from_raw(10_000_000))
                );
            }
            let a = r
                .resolve(&ctx(), |_| Ok(ResolvedCounter::stream(query.clone())))
                .unwrap();
            assert_eq!(a.counter().unwrap().as_counter().stream().unwrap(), &query);
            assert_eq!(
                a.counter_offset().unwrap().get(),
                i128::from(ticks * 100) + if flags == 0xc8 { 99 } else { 0 }
            );
            // Third case: floor(909090900*1.000000011)=909090909;
            // floor(909090999*1.000000011)=909091009: the99ns remainder
            // changes the released physical coordinate by100ns, not99.

            assert_eq!(
                a.point().unwrap().tai_ns().get(),
                1_483_228_837_000_000_000 + mapped + remainder
            );
        }
    }
}
/// Interpretations: RDG-010
#[test]
fn degraded_conversions_keep_values_levels_and_context_without_epoch_guesses() {
    let mut c = ctx();
    let mut e = epoch();
    e.origin.year = Some(2016);
    e.origin.day_of_year = 366;
    e.origin.hours = 23;
    e.origin.minutes = 59;
    e.origin.seconds = 59;
    c.ieee = Some(e.clone());
    c.ch4 = Some(e);
    let mut p = SecondaryPolicy::default();
    for id in [5, 21, 22] {
        p.levels.insert(id, Severity::Information);
    }
    let b = [0, 0, 0, 0, 1, 0, 0, 0];
    let a = absolute(0xc4, &b, &c, &p);
    assert_eq!(a.calendar().unwrap().seconds, 60);
    assert!(codes(a.inspection().findings()).contains(&22));
    let mut w = WordContext::new(1);
    w.own = c;
    let raw = TimeWord::Binary(Ch4Words {
        high: 0,
        low: 100,
        microseconds: 0,
    });
    let a = word_answer(raw, &w, &p);
    assert_eq!(a.calendar().unwrap().seconds, 60);
    assert!(codes(a.findings()).contains(&22));
    w.own.ieee.as_mut().unwrap().origin = epoch().origin;
    w.own.ieee.as_mut().unwrap().origin.year = Some(2028);
    let a = absolute(0xc4, &[0; 8], &w.own, &p);
    assert!(a.point().is_some());
    assert_eq!(
        finding(a.inspection().findings(), 21).severity(),
        Severity::Information
    );
    w.bcd = BcdContext {
        year: Some(2028),
        full_day: Some(1),
        interpretation: Some(BcdDayInterpretation::LowThreePackedBcdBits),
        edition: None,
    };
    w.bcd_scale = TimeScale::Utc;
    let raw = TimeWord::Bcd(Ch4Words {
        high: 0x2000,
        low: 0x8000,
        microseconds: 0,
    });
    let a = word_answer(raw, &w, &p);
    assert_eq!(a.calendar().unwrap().year, Some(2028));
    for id in [5, 21] {
        let f = finding(a.findings(), id);
        assert_eq!(f.severity(), Severity::Information);
        assert_eq!(f.evidence().validity(), ValueValidity::Kept);
    }
    assert!(!codes(net(NetworkWord::PtpNanoseconds(999_999_999), &w, &p).findings()).contains(&4));
    let mut bad = origin(0);
    bad.key.session = 0;
    assert!(matches!(
        inspect_stamp(
            StampInput {
                origin: bad,
                query_id: 2,
                flags: 0,
                bytes: &[0; 8],
                order: CounterOrder::Nearest
            },
            &p
        ),
        Err(TimeError::InvalidTimelineInput(_))
    ));
    let r = inspect_stamp(
        StampInput {
            origin: origin(i128::MAX),
            query_id: 2,
            flags: 0,
            bytes: &[1, 0, 0, 0, 0, 0, 0, 0],
            order: CounterOrder::ForwardWithinOneWrap,
        },
        &p,
    )
    .unwrap();
    let a = r
        .resolve(&w.own, |_| panic!("overflow must not query"))
        .unwrap();
    assert!(a.point().is_none());
    assert!(matches!(
        a.issue(),
        Some(TimeError::QuantityOverflow { .. })
    ));
}
// Clause-only: L1-ANS-004 -> L2-ANS-104 -> L3-ANS-104, per-session era context.
// RFC5905 Figure4: era1 offset63104 maps2036-02-08 (day39), but session2's
// declaration cannot date a word extracted from session1.
/// Interpretations: RDG-012
#[test]
fn ntp_word_cannot_borrow_another_sessions_era_but_zero_remains_a_sentinel() {
    use format2::NetworkConversion;
    use ntp::{EraDeclaration, NtpConversionStatus};
    let mut context = WordContext::new(2);
    context.ntp.declared = Some(EraDeclaration {
        era: 1,
        id: "era declared for session2 only".into(),
    });
    let raw = NtpTime {
        seconds: 63_104,
        fraction: 0,
    };
    let a = net(NetworkWord::Ntp(raw), &context, &Default::default());
    assert_eq!(a.origin().key.session, 1);
    assert!(a.calendar().is_none());
    assert_eq!(
        a.issue(),
        Some(&TimeError::NeedsContext {
            context: "NTP word session context"
        })
    );
    assert!(a.network().is_none());
    assert_eq!(a.word(), TimeWord::Network(NetworkWord::Ntp(raw)));

    // Both zero means unknown/unsynchronized before consulting era/session
    // evidence (RFC5905 section6; RDG-012), never a fabricated era1 instant.
    let raw = NtpTime {
        seconds: 0,
        fraction: 0,
    };
    let a = net(NetworkWord::Ntp(raw), &context, &Default::default());
    assert!(a.calendar().is_none());
    assert_eq!(
        a.issue(),
        Some(&TimeError::NeedsContext {
            context: "NTP era/zero/calendar context"
        })
    );
    let Some(NetworkConversion::Ntp(n)) = a.network() else {
        panic!("sentinel must reach the NTP converter")
    };
    assert_eq!(n.raw(), raw);
    assert_eq!(n.status(), NtpConversionStatus::UnknownUnsynchronized);
    assert_eq!(n.basis().era, None);
    assert_eq!(n.basis().context.session, 2);
    let f = finding(a.findings(), 29);
    assert_eq!(f.severity(), Severity::Warning);
    assert_eq!(f.evidence().validity(), ValueValidity::Unusable);
}
