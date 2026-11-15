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
        flags,
        bytes: b,
        order: CounterOrder::Nearest,
    }
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
            if f.code().number() == 28 {
                assert_eq!(f.evidence().validity(), ValueValidity::Unusable);
            }
        }
        let a = r
            .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::Offline))
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
            .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::Offline))
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
    for n in 0..8 {
        let a = inspect_stamp(stamp(0x4c, &b[..n], 0), &Default::default()).unwrap();
        assert_eq!(codes(a.findings()), vec![4, 6, 15, 28]);
        assert!(a.time().is_none());
    }
    let t = timeline((1 << 48) - 1);
    let a = inspect_stamp(stamp(0, &b, (1i128 << 48) - 1), &Default::default()).unwrap();
    assert_eq!(a.query().unwrap().ticks, 1i128 << 48);
    let a = a
        .resolve(&ctx(), |k| t.lookup(k).map(ResolvedCounter::Offline))
        .unwrap();
    assert_eq!(a.point().unwrap().tai_ns().get(), 1_483_228_837_000_000_100);
}
#[test]
fn word_context_is_explicit_and_network_components_never_default_to_zero() {
    let o = origin(0);
    let w = TimeWord::Binary(Ch4Words {
        high: 0,
        low: 100,
        microseconds: 0,
    });
    let mut c = WordContext::new(1);
    let a = answer_word(o.clone(), w, &c, &Default::default()).unwrap();
    assert!(a.calendar().is_none());
    assert!(a.issue().is_some());
    c.own = ctx();
    let a = answer_word(o.clone(), w, &c, &Default::default()).unwrap();
    assert_eq!(a.calendar().unwrap().seconds, 1);
    assert_eq!(a.origin(), &o);
    let w = TimeWord::Network(NetworkWord::NtpSeconds(2_208_988_800));
    let a = answer_word(o.clone(), w, &c, &Default::default()).unwrap();
    assert!(a.calendar().is_none());
    c.matching = Some(ComponentBinding {
        target: o.clone(),
        other: ExtractionOrigin {
            field: "fraction from same measurement".into(),
            ..o.clone()
        },
        word: NetworkWord::NtpFraction(0x80000000),
    });
    let a = answer_word(o, w, &c, &Default::default()).unwrap();
    assert_eq!(a.calendar().unwrap().year, Some(1970));
    assert_eq!(a.calendar().unwrap().nanoseconds, 500_000_000);
}
