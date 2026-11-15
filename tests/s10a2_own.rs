//! Ch11 Fig11-5/6, 11-10/11 (PDF18/19); §11.2.1.2a; contract6.6.
//! Literal word order and independently added byte checksums. Partial clauses
//! are recorded in verification-s10a2, without whole-requirement markers.
use irig106_time::{
    findings::*, format1::*, leap::*, own_time::*, scale::*, secondary_packet::*, timeline::*, *,
};
fn cal(scale: TimeScale) -> CalendarReading {
    CalendarReading {
        year: Some(2017),
        day_of_year: 1,
        hours: 0,
        minutes: 0,
        seconds: 0,
        nanoseconds: 0,
        scale: ScaleEvidence::standard(scale),
    }
}
fn observation(id: u64, ticks: u64, reference: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(ticks),
        kind: if reference.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference,
    }
}
fn reference() -> ReferenceInput {
    // Fig11-14: DMY2017-01-01 UTC, FMT4/SRC1, DATE1/TS1.
    ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(LeapSecondTable::builtin())
    .unwrap()
}
fn offline(ticks: u64) -> OfflineAnswer {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(observation(1, 0, Some(reference())), false)
        .unwrap();
    let k = b.observe(observation(2, ticks, None), false).unwrap();
    b.finish().unwrap().lookup(k).unwrap()
}
fn packet<'a>(flags: u8, bytes: &'a [u8], key: PacketKey) -> SecondaryPacket<'a> {
    SecondaryPacket {
        key,
        channel: 7,
        data_type: 0x19,
        rtc: Rtc::from_raw((key.ticks & ((1i128 << 48) - 1)) as u64),
        flags,
        bytes,
        byte_offset: Some(24),
        applicability: TimeApplicability::FirstDataBit,
    }
}
fn header(words: [u8; 8]) -> [u8; 12] {
    let mut b = [0; 12];
    b[..8].copy_from_slice(&words);
    let sum: u16 = words.iter().map(|v| u16::from(*v)).sum();
    b[10..].copy_from_slice(&sum.to_le_bytes());
    b
}
fn epoch(scale: TimeScale) -> EpochContext {
    EpochContext {
        origin: cal(scale),
        counting: EpochCounting::Continuous,
        source: "explicit synthetic epoch, not an RCC epoch claim".into(),
    }
}
fn ids(a: &OwnAnswer<'_>) -> Vec<u8> {
    a.findings().iter().map(|f| f.code().number()).collect()
}
#[test]
fn explicit_ieee_epoch_retains_native_scale_and_signed_comparison() {
    // one second elapsed from2017 UTC; ns0x12345678=305419896, secs1.
    let b = header([0x78, 0x56, 0x34, 0x12, 1, 0, 0, 0]);
    let c = offline(10_000_000);
    let mut ctx = OwnContext::default();
    ctx.ieee = Some(epoch(TimeScale::Utc));
    let a = answer_own(
        CounterAnswer::Offline(&c),
        packet(0x84, &b, c.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert_eq!(a.own().unwrap().tai_ns().get(), 1_483_228_838_305_419_896);
    assert_eq!(
        a.own().unwrap().native().unwrap().scale,
        ScaleEvidence::assumed(TimeScale::Utc)
    );
    assert_eq!(a.delta_ns().unwrap().get(), 305_419_896);
    assert_eq!(a.tolerance().unwrap().get(), 10_000_000);
    assert!(ids(&a).contains(&14));
    assert!(core::ptr::eq(a.counter().offline().unwrap(), &c));
    let f = a
        .findings()
        .iter()
        .find(|f| f.code().number() == 14)
        .unwrap();
    assert_eq!(f.evidence().location().packet, Some(2));
    assert!(
        f.evidence().raw_fields().iter().any(
            |f| f.name == "own_minus_counter_ns" && f.observed == RawValue::Signed(305_419_896)
        )
    );
    ctx.ieee = Some(epoch(TimeScale::Tai));
    let a = answer_own(
        CounterAnswer::Offline(&c),
        packet(0x84, &b, c.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert_eq!(a.delta_ns().unwrap().get(), -36_694_580_104);
}
#[test]
fn ieee_eight_byte_encoder_is_not_format2_order() {
    let t = Ieee1588Time {
        seconds: 0x01020304,
        nanoseconds: 123456789,
    };
    assert_eq!(
        encode_ieee_time(t).unwrap(),
        [0x15, 0xcd, 0x5b, 7, 4, 3, 2, 1]
    );
    assert!(
        encode_ieee_time(Ieee1588Time {
            nanoseconds: 1_000_000_000,
            ..t
        })
        .is_err()
    );
    assert!(
        encode_ieee_time(Ieee1588Time {
            nanoseconds: 999_999_999,
            ..t
        })
        .is_ok()
    );
}
#[test]
fn partial_context_checksum_failure_and_applicability_never_fake_zero() {
    let c = offline(0);
    let b = header([0; 8]);
    let mut ctx = OwnContext::default();
    for flags in [0x80, 0x84] {
        let a = answer_own(
            CounterAnswer::Offline(&c),
            packet(flags, &b, c.basis.key),
            &ctx,
            &OwnPolicy::default(),
        )
        .unwrap();
        assert!(a.header().time().is_some());
        assert!(a.own().is_none());
        assert!(a.issue().is_some());
        assert_eq!(a.delta_ns(), None);
    }
    ctx.ieee = Some(epoch(TimeScale::Utc));
    let mut bad = b;
    bad[10] = 1;
    let a = answer_own(
        CounterAnswer::Offline(&c),
        packet(0x84, &bad, c.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert!(a.own().is_none());
    assert_eq!(a.delta_ns(), None);
    assert!(ids(&a).contains(&12));
    assert!(a.counter().offline().unwrap().time.is_some());
    let mut p = packet(0x84, &b, c.basis.key);
    p.applicability = TimeApplicability::CitedException {
        data_type: 0x19,
        citation: "caller instrument exception".into(),
    };
    let a = answer_own(CounterAnswer::Offline(&c), p, &ctx, &OwnPolicy::default()).unwrap();
    assert!(a.own().is_some());
    assert_eq!(a.delta_ns(), None);
    assert!(!ids(&a).contains(&14));
}
#[test]
fn ertc_is_relative_to_this_session_with_one_ns_ticks_and_signed_phase() {
    let c = offline(1);
    let ctx = OwnContext::default();
    for (raw, want) in [
        (0u64, -100),
        (1, -99),
        (99, -1),
        (100, 0),
        (101, 1),
        (199, 99),
        (200, 100),
    ] {
        let b = header(raw.to_le_bytes());
        let a = answer_own(
            CounterAnswer::Offline(&c),
            packet(0x88, &b, c.basis.key),
            &ctx,
            &OwnPolicy::default(),
        )
        .unwrap();
        assert_eq!(a.delta_ns().unwrap().get(), want);
        assert_eq!(
            a.own().unwrap().tai_ns().get(),
            1_483_228_837_000_000_000 + i128::from(raw)
        );
        assert_eq!(
            a.header().time(),
            Some(SecondaryHeaderTime::Ertc(
                Ertc::from_le_bytes(&raw.to_le_bytes()).unwrap()
            ))
        );
    }
    let mut no = c.clone();
    no.reading = None;
    no.time = None;
    let b = header(100u64.to_le_bytes());
    let a = answer_own(
        CounterAnswer::Offline(&no),
        packet(0x88, &b, no.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert!(a.own().is_none());
    assert_eq!(a.delta_ns(), None);
    assert!(a.header().time().is_some());
    let mut p = packet(0x88, &b, c.basis.key);
    p.key.session += 1;
    assert!(answer_own(CounterAnswer::Offline(&c), p, &ctx, &OwnPolicy::default()).is_err());
}
#[test]
fn tolerance_is_strict_and_level_override_changes_no_value() {
    let c = offline(0);
    let mut ctx = OwnContext::default();
    ctx.ieee = Some(epoch(TimeScale::Utc));
    let mut policy = OwnPolicy::default();
    policy.levels.insert(14, Severity::Information);
    for (ns, above) in [
        (0u32, false),
        (9_999_999, false),
        (10_000_000, false),
        (10_000_001, true),
    ] {
        let mut w = [0; 8];
        w[..4].copy_from_slice(&ns.to_le_bytes());
        let b = header(w);
        let a = answer_own(
            CounterAnswer::Offline(&c),
            packet(0x84, &b, c.basis.key),
            &ctx,
            &policy,
        )
        .unwrap();
        assert_eq!(a.delta_ns().unwrap().get(), i128::from(ns));
        assert_eq!(ids(&a).contains(&14), above);
        if above {
            assert_eq!(
                a.findings()
                    .iter()
                    .find(|f| f.code().number() == 14)
                    .unwrap()
                    .severity(),
                Severity::Information
            );
        }
    }
    policy.tolerance = Some(NanosDuration::new(0));
    let b = header([1, 0, 0, 0, 0, 0, 0, 0]);
    assert!(
        ids(&answer_own(
            CounterAnswer::Offline(&c),
            packet(0x84, &b, c.basis.key),
            &ctx,
            &policy
        )
        .unwrap())
        .contains(&14)
    );
}
