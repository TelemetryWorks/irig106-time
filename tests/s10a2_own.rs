//! Ch11 Fig11-5/6, 11-10 (PDF18); §11.2.1.2a; contract6.6.
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
    offline_with(ticks, reference())
}
fn offline_with(ticks: u64, reference: ReferenceInput) -> OfflineAnswer {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(observation(1, 0, Some(reference)), false)
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
fn own<'a>(
    c: &'a OfflineAnswer,
    flags: u8,
    bytes: &[u8],
    ctx: &OwnContext,
    policy: &OwnPolicy,
) -> OwnAnswer<'a> {
    answer_own(
        CounterAnswer::Offline(c),
        packet(flags, bytes, c.basis.key),
        ctx,
        policy,
    )
    .unwrap()
}
fn ids(a: &OwnAnswer<'_>) -> Vec<u8> {
    a.findings().iter().map(|f| f.code().number()).collect()
}
/// Requirements: L1-SEC-002, L2-SEC-102, L3-SEC-102
#[test]
fn explicit_ieee_epoch_retains_native_scale_and_signed_comparison() {
    // one second elapsed from2017 UTC; ns0x12345678=305419896, secs1.
    let b = header([0x78, 0x56, 0x34, 0x12, 1, 0, 0, 0]);
    let c = offline(10_000_000);
    let mut ctx = OwnContext {
        ieee: Some(epoch(TimeScale::Utc)),
        ..OwnContext::default()
    };
    let a = own(&c, 0x84, &b, &ctx, &OwnPolicy::default());
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
    assert_eq!(a.header().applicability(), &TimeApplicability::FirstDataBit);
    let mut exception = packet(0x84, &b, c.basis.key);
    exception.applicability = TimeApplicability::CitedException {
        data_type: 0x19,
        citation: "explicit caller sensor citation".into(),
    };
    let expected = exception.applicability.clone();
    let e = answer_own(
        CounterAnswer::Offline(&c),
        exception,
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert_eq!(e.header().applicability(), &expected);
    assert!(e.own().is_some());
    let mut corrupt = b;
    corrupt[10] ^= 1;
    assert!(
        own(&c, 0x84, &corrupt, &ctx, &OwnPolicy::default())
            .own()
            .is_none()
    );
    assert_eq!(f.evidence().related()[0].packet, Some(1));
    assert_eq!(f.evidence().related()[0].session, Some(c.basis.key.session));
    assert_eq!(f.evidence().raw_rtc(), Some(Rtc::from_raw(10_000_000)));
    assert_eq!(f.evidence().validity(), ValueValidity::Kept);
    ctx.ieee = Some(epoch(TimeScale::Tai));
    let a = own(&c, 0x84, &b, &ctx, &OwnPolicy::default());
    assert_eq!(a.delta_ns().unwrap().get(), -36_694_580_104);
}
/// Requirements: L1-ABS-003, L2-ABS-103, L3-ABS-103
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
    let literal = [0x15, 0xcd, 0x5b, 7, 4, 3, 2, 1];
    let parsed = Ieee1588Time::from_le_bytes(&literal).unwrap();
    assert_eq!(parsed, t);
    let f2 = PtpTime::from_le_bytes(&[4, 3, 2, 1, 0x15, 0xcd, 0x5b, 7]).unwrap();
    assert_eq!((f2.seconds, f2.nanoseconds), (t.seconds, t.nanoseconds));
    assert_eq!(
        Ieee1588Time::from_le_bytes(&[0xff, 0xc9, 0x9a, 0x3b, 4, 3, 2, 1])
            .unwrap()
            .nanoseconds,
        999_999_999
    );
    let c = offline(0);
    let mut ctx = OwnContext::default();
    let mut e = epoch(TimeScale::Tai);
    e.origin.year = Some(1970);
    ctx.ieee = Some(e);
    let b = header(literal);
    let a = own(&c, 0x84, &b, &ctx, &OwnPolicy::default());
    assert_eq!(a.own().unwrap().tai_ns().get(), 16_909_060_123_456_789);
    assert_eq!(
        a.own().unwrap().native().unwrap().scale,
        ScaleEvidence::assumed(TimeScale::Tai)
    );
    let bad = header([0, 0xca, 0x9a, 0x3b, 4, 3, 2, 1]);
    assert!(
        own(&c, 0x84, &bad, &ctx, &OwnPolicy::default())
            .header()
            .time()
            .is_none()
    );
}
/// Requirements: L1-ANS-002, L2-ANS-102, L3-ANS-102
#[test]
fn partial_context_checksum_failure_and_applicability_never_fake_zero() {
    let c = offline(0);
    let b = header([0; 8]);
    let mut ctx = OwnContext::default();
    for flags in [0x80, 0x84] {
        let a = own(&c, flags, &b, &ctx, &OwnPolicy::default());
        assert!(a.header().time().is_some());
        assert!(a.own().is_none());
        assert!(a.issue().is_some());
        assert_eq!(a.delta_ns(), None);
    }
    ctx.ch4 = Some(epoch(TimeScale::Utc));
    ctx.ieee = Some(epoch(TimeScale::Utc));
    for flags in [0x80, 0x84, 0x88] {
        let a = own(&c, flags, &b, &ctx, &OwnPolicy::default());
        assert_eq!(a.delta_ns().unwrap().get(), 0);
        assert!(!ids(&a).contains(&14));
    }
    for (ns, warn) in [(10_000_000u32, false), (10_000_001, true)] {
        let mut w = [0; 8];
        w[..4].copy_from_slice(&ns.to_le_bytes());
        let bytes = header(w);
        let a = own(&c, 0x84, &bytes, &ctx, &OwnPolicy::default());
        assert_eq!(ids(&a).contains(&14), warn);
    }
    let mut unknown = c.clone();
    unknown.reading.as_mut().unwrap().scale = ScaleEvidence::unknown();
    assert_eq!(
        own(&unknown, 0x84, &b, &ctx, &OwnPolicy::default()).delta_ns(),
        None
    );
    let mut absent = c.clone();
    absent.reading = None;
    absent.time = None;
    let a = own(&absent, 0x88, &b, &ctx, &OwnPolicy::default());
    assert!(a.own().is_none());
    assert_eq!(a.delta_ns(), None);
    ctx.ieee = Some(epoch(TimeScale::Utc));
    let mut bad = b;
    bad[10] = 1;
    let a = own(&c, 0x84, &bad, &ctx, &OwnPolicy::default());
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
/// Requirements: L1-SEC-003, L2-SEC-103, L3-SEC-103
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
        let a = own(&c, 0x88, &b, &ctx, &OwnPolicy::default());
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
    // Full64-bit raw value remains a relative counter, not a Unix epoch.
    for raw in [u64::MAX, 0x0102030405060708] {
        let mut counter = c.clone();
        let ticks = (raw / 100) & ((1 << 48) - 1);
        counter.basis.key.ticks = i128::from(ticks);
        let b = header(raw.to_le_bytes());
        let a = own(&counter, 0x88, &b, &ctx, &OwnPolicy::default());
        assert_eq!(a.delta_ns().unwrap().get(), i128::from(raw % 100));
        let mut corrupt = b;
        corrupt[11] ^= 1;
        assert!(
            own(&counter, 0x88, &corrupt, &ctx, &OwnPolicy::default())
                .own()
                .is_none()
        );
    }
    let mut half = c.clone();
    half.basis.key.ticks = 1i128 << 47;
    let zero = header([0; 8]);
    let a = own(&half, 0x88, &zero, &ctx, &OwnPolicy::default());
    assert!(a.own().is_none());
    assert_eq!(a.delta_ns(), None);
    assert_eq!(a.issue(), Some(&TimeError::AmbiguousCounterDelta));
    assert!(ids(&a).contains(&25));
    let mut no = c.clone();
    no.reading = None;
    no.time = None;
    let b = header(100u64.to_le_bytes());
    let a = own(&no, 0x88, &b, &ctx, &OwnPolicy::default());
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
    let ctx = OwnContext {
        ieee: Some(epoch(TimeScale::Utc)),
        ..OwnContext::default()
    };
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
        let a = own(&c, 0x84, &b, &ctx, &policy);
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
    assert!(ids(&own(&c, 0x84, &b, &ctx, &policy)).contains(&14));
}
#[test]
fn ch4_binary_origin_and_raw_precision_remain_explicit() {
    // Fig11-4/T10: [reserved,usec,low,high], low0x8000 =327.68s,
    // usec9999 adds0.009999s. No17-bit day split.
    let b = header([0, 0, 0x0f, 0x27, 0, 0x80, 0, 0]);
    let c = offline(3_276_800_000);
    let ctx = OwnContext {
        ch4: Some(epoch(TimeScale::Utc)),
        ..OwnContext::default()
    };
    let a = own(&c, 0x80, &b, &ctx, &OwnPolicy::default());
    assert_eq!(
        a.own().unwrap().tai_ns().get(),
        1_483_228_837_000_000_000 + 327_689_999_000
    );
    assert_eq!(a.delta_ns().unwrap().get(), 9_999_000);
    assert!(!ids(&a).contains(&14));
    let mut w = [0, 0, 0x10, 0x27, 0, 0x80, 0, 0];
    let bad = header(w);
    let a = own(&c, 0x80, &bad, &ctx, &OwnPolicy::default());
    assert!(a.own().is_none());
    assert!(ids(&a).contains(&4));
    w[2..4].copy_from_slice(&[0x0f, 0x27]);
    w[0] = 1;
    let b = header(w);
    let a = own(&c, 0x80, &b, &ctx, &OwnPolicy::default());
    assert_eq!(a.delta_ns().unwrap().get(), 9_999_000);
    assert!(ids(&a).contains(&5));
}
#[test]
fn explicit_scale_and_table_currency_never_change_raw_words() {
    let c = offline(0);
    let b = header([0; 8]);
    let mut ctx = OwnContext {
        ieee: Some(epoch(TimeScale::Gps)),
        ..OwnContext::default()
    };
    let a = own(&c, 0x84, &b, &ctx, &OwnPolicy::default());
    assert_eq!(a.delta_ns().unwrap().get(), -18_000_000_000);
    assert_eq!(
        a.own().unwrap().native().unwrap().scale.effective(),
        TimeScale::Gps
    );
    ctx.ieee = Some(epoch(TimeScale::Tai));
    ctx.ieee.as_mut().unwrap().origin.year = Some(2028);
    let mut policy = OwnPolicy::default();
    policy.levels.insert(21, Severity::Information);
    let a = own(&c, 0x84, &b, &ctx, &policy);
    let utc = a.own().unwrap().utc_conversion();
    assert_eq!(utc.status(), PtpConversionStatus::PossiblyOutOfDate);
    assert_eq!(utc.basis().offset.unwrap().get(), 37);
    assert!(utc.reading().is_some());
    let f = a
        .findings()
        .iter()
        .find(|f| f.code().number() == 21)
        .unwrap();
    assert_eq!(f.severity(), Severity::Information);
    assert_eq!(f.evidence().validity(), ValueValidity::Kept);
    ctx.table = LeapSecondTable::empty();
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert!(a.own().is_some());
    assert!(a.own().unwrap().native().is_some());
    assert!(a.own().unwrap().utc_conversion().reading().is_none());
    assert_eq!(a.delta_ns(), None);
    assert!(!ids(&a).contains(&21));
    ctx.ieee.as_mut().unwrap().origin.year = Some(1969);
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert!(a.own().unwrap().tai_ns().get() < 0);
    assert_eq!(a.own().unwrap().native().unwrap().year, Some(1969));
    ctx.table = LeapSecondTable::builtin();
    ctx.ieee = Some(epoch(TimeScale::Utc));
    ctx.ieee.as_mut().unwrap().origin.year = Some(2200);
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert_eq!(a.own().unwrap().native().unwrap().year, Some(2200));
    assert!(ids(&a).contains(&21));
}
/// Interpretations: RDG-010
#[test]
fn leap_endpoints_and_civil_ambiguity_use_explicit_counting_convention() {
    let c = offline(0);
    let b = header([0, 0, 0, 0, 1, 0, 0, 0]);
    let mut ctx = OwnContext::default();
    let mut e = epoch(TimeScale::Utc);
    e.origin.year = Some(2016);
    e.origin.day_of_year = 366;
    e.origin.hours = 23;
    e.origin.minutes = 59;
    e.origin.seconds = 59;
    ctx.ieee = Some(e);
    let mut policy = OwnPolicy::default();
    policy.levels.insert(22, Severity::Warning);
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert_eq!(a.own().unwrap().native().unwrap().seconds, 60);
    assert_eq!(a.delta_ns().unwrap().get(), -1_000_000_000);
    assert_eq!(
        a.findings()
            .iter()
            .find(|f| f.code().number() == 22)
            .unwrap()
            .severity(),
        Severity::Warning
    );
    // Explicit second60 at the origin: one more SI second reaches midnight.
    ctx.ieee.as_mut().unwrap().origin.seconds = 60;
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert_eq!(a.delta_ns().unwrap().get(), 0);
    assert_eq!(a.own().unwrap().native().unwrap().day_of_year, 1);
    assert!(!ids(&a).contains(&22));
    // Civil seconds cannot choose which occurrence of the insertion coordinate.
    ctx.ieee.as_mut().unwrap().origin.seconds = 59;
    ctx.ieee.as_mut().unwrap().counting = EpochCounting::Civil;
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert!(a.own().is_none());
    assert_eq!(a.delta_ns(), None);
    assert!(a.issue().is_some());
    assert!(!ids(&a).contains(&30));
    ctx.ieee.as_mut().unwrap().origin.day_of_year = 182; // eligible RDG010 date but no2016 June insertion.
    ctx.ieee.as_mut().unwrap().origin.seconds = 60;
    ctx.ieee.as_mut().unwrap().counting = EpochCounting::Continuous;
    let a = own(&c, 0x84, &b, &ctx, &policy);
    assert!(a.own().is_none());
}
#[test]
fn all_missing_context_and_scope_guards_leave_counter_untouched() {
    let c = offline(0);
    let b = header([0; 8]);
    let mut ctx = OwnContext::default();
    for which in 0..4 {
        let mut e = epoch(TimeScale::Utc);
        match which {
            0 => e.source.clear(),
            1 => e.origin.year = None,
            2 => e.origin.scale = ScaleEvidence::unknown(),
            _ => {
                ctx.table = LeapSecondTable::empty();
            }
        }
        ctx.ieee = Some(e);
        let a = own(&c, 0x84, &b, &ctx, &OwnPolicy::default());
        assert!(a.own().is_none());
        assert_eq!(a.delta_ns(), None);
        assert_eq!(a.counter().offline().unwrap(), &c);
    }
    ctx = OwnContext::default();
    ctx.ieee = Some(epoch(TimeScale::Utc));
    let mut unknown = c.clone();
    unknown.reading.as_mut().unwrap().scale = ScaleEvidence::unknown();
    let a = answer_own(
        CounterAnswer::Offline(&unknown),
        packet(0x84, &b, c.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert!(a.own().is_some());
    assert_eq!(a.delta_ns(), None);
    assert!(!ids(&a).contains(&14));
    let mut p = packet(0x84, &b, c.basis.key);
    p.rtc = Rtc::from_raw(1);
    assert!(answer_own(CounterAnswer::Offline(&c), p, &ctx, &OwnPolicy::default()).is_err());
    let mut missing_reference = c.clone();
    missing_reference.basis.reference = None;
    let a = answer_own(
        CounterAnswer::Offline(&missing_reference),
        packet(0x88, &b, c.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert!(a.own().is_none());
    let mut cross = c.clone();
    cross.basis.reference.as_mut().unwrap().key.session += 1;
    let a = answer_own(
        CounterAnswer::Offline(&cross),
        packet(0x88, &b, c.basis.key),
        &ctx,
        &OwnPolicy::default(),
    )
    .unwrap();
    assert!(a.own().is_none());
}
fn stream_answer_at(
    mode: irig106_time::stream::StreamMode,
    ticks: u64,
) -> irig106_time::stream::StreamAnswer {
    use irig106_time::stream::*;
    let mut p = StreamPolicy::new(
        9,
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
    s.observe(observation(1, 0, Some(reference())), 0, 24, false)
        .unwrap();
    let data = s
        .observe(observation(2, ticks, None), 1, 24, false)
        .unwrap();
    if mode == StreamMode::Standard {
        return data.answers.into_iter().next().unwrap();
    }
    let mut next = reference();
    let ReferenceValue::Readable(t, _) = next.value else {
        panic!()
    };
    next.value = ReferenceValue::Readable(t.add_nanos(1_000_010_000).unwrap(), true);
    next = next.normalize_to_tai(LeapSecondTable::builtin()).unwrap();
    s.observe(observation(3, 10_000_000, Some(next)), 2, 24, false)
        .unwrap()
        .answers
        .into_iter()
        .next()
        .unwrap()
}
#[test]
fn released_stream_modes_preserve_basis_and_ertc_interpolation_phase() {
    use irig106_time::stream::StreamMode;
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let c = stream_answer_at(mode, 5_000_000);
        let raw = 500_000_099u64;
        let b = header(raw.to_le_bytes());
        let ctx = OwnContext::default();
        let a = answer_own(
            CounterAnswer::Stream(&c),
            packet(0x88, &b, c.basis.key),
            &ctx,
            &OwnPolicy::default(),
        )
        .unwrap();
        assert_eq!(a.delta_ns().unwrap().get(), 99); // floor(99*1.00001)=99.
        let base = if mode == StreamMode::Standard {
            500_000_000
        } else {
            500_005_000
        };
        assert_eq!(
            a.own().unwrap().tai_ns().get(),
            1_483_228_837_000_000_000 + base + 99
        );
        assert!(core::ptr::eq(a.counter().stream().unwrap(), &c));
        assert_eq!(a.counter().stream().unwrap().basis, c.basis);
        assert_eq!(a.tolerance().unwrap().get(), 10_000_000);
        assert!(!ids(&a).contains(&14));
        // A10ms ERTC/RTC discrepancy inherits the same released drift slope.
        let b = header(510_000_000u64.to_le_bytes());
        let a = answer_own(
            CounterAnswer::Stream(&c),
            packet(0x88, &b, c.basis.key),
            &ctx,
            &OwnPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            a.delta_ns().unwrap().get(),
            if mode == StreamMode::Standard {
                10_000_000
            } else {
                10_000_100
            }
        );
        assert_eq!(ids(&a).contains(&14), mode == StreamMode::Interpolate);
        assert!(ids(&a).contains(&25));
        let mut bad = b;
        bad[10] ^= 1;
        let a = answer_own(
            CounterAnswer::Stream(&c),
            packet(0x88, &bad, c.basis.key),
            &ctx,
            &OwnPolicy::default(),
        )
        .unwrap();
        assert!(a.own().is_none());
        assert_eq!(a.counter().stream().unwrap().basis, c.basis);
    }
    // Difference of floors at a non-integral bracket phase: base100ns maps
    // to100ns; base+99999 maps100100ns, so the released-point delta is100000.
    let c = stream_answer_at(StreamMode::Interpolate, 1);
    let b = header(100099u64.to_le_bytes());
    let a = answer_own(
        CounterAnswer::Stream(&c),
        packet(0x88, &b, c.basis.key),
        &OwnContext::default(),
        &OwnPolicy::default(),
    )
    .unwrap();
    assert_eq!(a.delta_ns().unwrap().get(), 100000);
}

#[test]
fn format2_counter_context_remains_independent_and_ambiguous_ntp_is_not_repaired() {
    use irig106_time::{edition::PacketEdition, format2::*, ntp::NtpContext};
    // Fig11-17: TAI1483228837 =586846a5, UTC2017-01-01 plus37s.
    // RFC5905/RDG012 insertion window NTP3692217600 =dc12c500.
    for (payload, ambiguous) in [
        ([0x11, 0, 0, 0, 0xa5, 0x46, 0x68, 0x58, 0, 0, 0, 0], false),
        ([1, 0, 0, 0, 0, 0xc5, 0x12, 0xdc, 0, 0, 0, 0x80], true),
    ] {
        let o = Format2Input {
            payload: &payload,
            data_length: 12,
            capture: ReferenceCapture {
                session: 1,
                packet: 1,
                channel: 7,
                rtc: Rtc::from_raw(0),
                byte_offset: Some(24),
            },
            declared_edition: Some(PacketEdition::V24R1),
            data_type_version: 10,
            declared_timein: true,
        }
        .decode()
        .unwrap();
        let input =
            ReferenceInput::format2(o, NtpContext::new(1), LeapSecondTable::builtin(), false)
                .unwrap();
        let c = offline_with(0, input);
        let b = header([0; 8]);
        let ctx = OwnContext {
            ieee: Some(epoch(TimeScale::Utc)),
            ..Default::default()
        };
        let a = own(&c, 0x84, &b, &ctx, &OwnPolicy::default());
        assert!(
            a.counter()
                .offline()
                .unwrap()
                .basis
                .reference
                .as_ref()
                .unwrap()
                .input
                .network
                .is_some()
        );
        assert!(a.own().is_some());
        assert_eq!(
            a.delta_ns().map(|n| n.get()),
            if ambiguous { None } else { Some(0) }
        );
        assert!(!ids(&a).contains(&14));
    }
}
