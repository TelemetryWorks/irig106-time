//! E1 source fixtures. Numeric baselines precede the new timeline API.
use irig106_time::format1::decode_time_f1;
use irig106_time::scale::ScalePolicy;
use irig106_time::{AbsoluteTime, Rtc, TimeCorrelator, TimeF1Layout};

// RCC 123-20 §5.6 printed 5-44/PDF78 supplies the numeric example, not
// complete packet bytes. These literal wire bytes are hand-derived from
// 106-24R1 §11.2.1.1 and Figures 11-12/13; no crate encoder is an oracle.
const HANDBOOK: [u8; 36] = [
    0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0x40, 0x42, 0x0f, 0, 0, 0, 0xa9,
    0x3e, 1, 0x30, 0, 0, 0, 0x25, 0x30, 0x12, 0, 1, 0, 0,
];
// Contract §7.2, byte for byte, including Data Length=10 and trailer filler.
const CONTRACT: [u8; 36] = [
    0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0, 0xe1, 0xf5, 5, 0, 0, 0x4f,
    0xe3, 1, 0x30, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x87, 1, 0, 0,
];
fn reading(bytes: &[u8; 36]) -> (Rtc, AbsoluteTime) {
    let sum = bytes[..22].chunks_exact(2).fold(0u16, |s, w| {
        s.wrapping_add(u16::from_le_bytes([w[0], w[1]]))
    });
    assert_eq!(sum, u16::from_le_bytes([bytes[22], bytes[23]]));
    let r = decode_time_f1(
        &bytes[24..],
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    (
        Rtc::from_le_bytes(bytes[16..22].try_into().unwrap()),
        r.time.unwrap().to_absolute().unwrap().value,
    )
}

/// Requirements: L1-COR-002, L2-COR-102, L3-COR-102
#[test]
fn handbook_example_literal_wire_baseline() {
    let (rtc, time) = reading(&HANDBOOK);
    assert_eq!(rtc.as_raw(), 1_000_000);
    let mut c = TimeCorrelator::new();
    c.add_reference(1, rtc, time);
    assert_eq!(
        c.correlate(Rtc::from_raw(1_150_000), Some(1)).unwrap(),
        AbsoluteTime::new(100, 12, 30, 25, 15_000_000).unwrap()
    );
}

/// Requirements: L1-COR-002, L2-COR-102, L3-COR-102, L1-ANS-001, L2-ANS-101, L3-ANS-101
#[test]
fn contract_example_literal_wire_baseline() {
    let (rtc, time) = reading(&CONTRACT);
    let mut c = TimeCorrelator::new();
    c.add_reference(1, rtc, time);
    assert_eq!(
        c.correlate(Rtc::from_raw(104_200_000), Some(1)).unwrap(),
        AbsoluteTime::new(187, 13, 45, 27, 770_000_000).unwrap()
    );
}

/// Requirements: L1-COR-009, L2-COR-109, L3-COR-109
#[test]
fn accepted_late_default_is_1100_ms() {
    // Ch10 §10.6.1 b/c: 100 ms generation + 1000 ms commit, ADR-0010.
    assert_eq!(TimeCorrelator::DEFAULT_OOO_WINDOW_NS, 1_100_000_000);
}

/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
#[test]
fn quality_drift_uses_absolute_over_rtc_sign() {
    // Independent arithmetic: RTC=1 s, absolute=1.0001 s -> +100 ppm.
    let mut c = TimeCorrelator::new();
    let t = AbsoluteTime::new(100, 0, 0, 0, 0).unwrap();
    c.add_reference(1, Rtc::from_raw(0), t);
    c.add_reference(1, Rtc::from_raw(10_000_000), t.add_nanos(1_000_100_000));
    let q = irig106_time::quality::compute_quality(c.references());
    assert!((q.drift_ppm_per_channel[0].1 - 100.0).abs() < 0.001);
    assert!((c.drift_ppm(1).unwrap() - 100.0).abs() < 0.001);
}

use irig106_time::scale::{ScaleEvidence, ScaleOrigin, TimeScale};
use irig106_time::timeline::*;
use irig106_time::{TimeF1Source, TimeFormat, TimeSource};

fn packet(id: u64, ch: u16, ticks: u64, reference: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: ch,
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
fn reference(second: u8) -> ReferenceInput {
    ReferenceInput {
        value: ReferenceValue::Readable(AbsoluteTime::new(100, 12, 0, second, 0).unwrap(), true),
        scale: ScaleEvidence::assumed(TimeScale::Utc),
        source: TimeF1Source::Source(TimeSource::External),
        format: TimeFormat::IrigB,
        its: None,
        precision_ns: 10_000_000,
        raw_csdw: None,
        year_origin: YearOrigin::Unknown,
    }
}
fn source_packet(id: u64, bytes: &[u8; 36]) -> PacketObservation {
    let (rtc, _) = reading(bytes);
    let r = ReferenceInput::format1(
        &bytes[24..],
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let mut p = packet(id, 1, rtc.as_raw(), Some(r));
    p.declared_external = true;
    p
}

/// Requirements: L1-COR-002, L2-COR-102, L3-COR-102, L1-ANS-001, L2-ANS-101, L3-ANS-101
#[test]
fn both_literal_examples_carry_reproducible_basis() {
    for (bytes, target, expected) in [
        (
            &HANDBOOK,
            1_150_000,
            AbsoluteTime::new(100, 12, 30, 25, 15_000_000).unwrap(),
        ),
        (
            &CONTRACT,
            104_200_000,
            AbsoluteTime::new(187, 13, 45, 27, 770_000_000).unwrap(),
        ),
    ] {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let first = b.observe(source_packet(1, bytes), false).unwrap();
        let key = b.observe(packet(2, 3, target, None), false).unwrap();
        // Contract 7.1's next reference, one second later; r1 is 0.42 s away,
        // the following reference 0.58 s away. Literal Chapter 11 body words.
        if target == 104_200_000 {
            let next = [
                0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0x80, 0x77, 0x8e, 6,
                0, 0, 0x68, 0x7a, 1, 0x30, 0, 0, 0x35, 0x28, 0x45, 0x13, 0x87, 1, 0, 0,
            ];
            b.observe(source_packet(3, &next), false).unwrap();
        }
        let timeline = b.finish();
        let a = timeline.lookup(key).unwrap();
        assert_eq!(a.time, Some(expected));
        assert_eq!(a.basis.key.session, 1);
        assert_eq!(a.basis.channel, Some((1, ChannelChoice::DeclaredExternal)));
        let r = a.basis.reference.as_ref().unwrap();
        assert_eq!(r.key, first);
        assert_eq!(r.input.raw_csdw, Some(0x3001));
        assert_eq!(r.input.format, TimeFormat::IrigB);
        assert_eq!(r.input.source, TimeF1Source::Source(TimeSource::External));
        assert!(r.input.its.is_some());
        assert_eq!(r.input.precision_ns, 10_000_000);
        assert_eq!(r.input.year_origin, YearOrigin::Unknown);
        assert_eq!(r.input.scale.effective(), TimeScale::Utc);
        assert_eq!(r.input.scale.origin(), ScaleOrigin::Policy);
        assert_eq!(
            a.basis.distance_ns,
            Some(if target == 104_200_000 {
                420_000_000
            } else {
                15_000_000
            })
        );
        assert_eq!(a.basis.policy, OfflinePolicy::default());
        assert!(!a.basis.drift_corrected);
        assert_eq!(
            a.basis.position,
            if target == 104_200_000 {
                ReferencePosition::Between
            } else {
                ReferencePosition::After
            }
        );
    }
}

/// Requirements: L1-COR-005, L2-COR-105, L3-COR-105, L1-COR-009, L2-COR-109, L3-COR-109
#[test]
fn sessions_high_water_wrap_and_exact_late_boundary() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 1, 20_000_000, Some(reference(2))), false)
        .unwrap();
    assert_eq!(
        b.observe(packet(2, 3, 9_000_000, None), false)
            .unwrap()
            .session,
        1
    );
    // A late packet must not lower high water: now 1100 ms + one tick back.
    let reset = b.observe(packet(3, 3, 8_999_999, None), false).unwrap();
    assert_eq!(reset.session, 2);
    let t = b.finish();
    assert_eq!(t.sessions().len(), 2);
    assert!(t.lookup(reset).unwrap().time.is_none());
    for code in [18, 19] {
        assert!(t.findings().iter().any(|f| f.code == code));
    }
    assert_eq!(
        t.findings().iter().find(|f| f.code == 18).unwrap().severity,
        Severity::Information
    );
    assert_eq!(
        t.findings().iter().find(|f| f.code == 19).unwrap().severity,
        Severity::Warning
    );
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 1, (1u64 << 48) - 10, Some(reference(0))), false)
        .unwrap();
    let wrapped = b.observe(packet(2, 3, 5, None), false).unwrap();
    assert_eq!(wrapped.session, 1);
    assert_eq!(
        b.finish().lookup(wrapped).unwrap().basis.distance_ns,
        Some(1500)
    );
    let mut b = TimelineBuilder::new(OfflinePolicy {
        infer_sessions: false,
        ..Default::default()
    })
    .unwrap();
    b.observe(packet(1, 3, 20_000_000, None), false).unwrap();
    assert_eq!(b.observe(packet(2, 3, 0, None), false).unwrap().session, 1);
    assert_eq!(b.observe(packet(3, 3, 1, None), true).unwrap().session, 2);
}

/// Requirements: L1-COR-003, L2-COR-103, L3-COR-103, L1-COR-006, L2-COR-106, L3-COR-106
#[test]
fn channel_rank_admission_none_and_caller_no_fallback() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 2, 0, Some(reference(0))), false)
        .unwrap();
    let mut external = packet(2, 7, 1, Some(reference(0)));
    external.declared_external = true;
    b.observe(external, false).unwrap();
    for (id, source, state, declared) in [
        (
            3,
            TimeSource::None,
            ReferenceValue::Readable(AbsoluteTime::new(100, 0, 0, 0, 0).unwrap(), true),
            true,
        ),
        (4, TimeSource::Internal, ReferenceValue::Malformed, true),
        (
            5,
            TimeSource::Internal,
            ReferenceValue::Readable(AbsoluteTime::new(100, 0, 0, 0, 0).unwrap(), true),
            false,
        ),
    ] {
        let mut r = reference(0);
        r.source = TimeF1Source::Source(source);
        r.value = state;
        let mut p = packet(id, 8, id, Some(r));
        p.declared_timein = declared;
        b.observe(p, false).unwrap();
    }
    let key = b.observe(packet(6, 3, 10, None), false).unwrap();
    let t = b.finish();
    assert_eq!(
        t.lookup(key).unwrap().basis.channel,
        Some((7, ChannelChoice::DeclaredExternal))
    );
    assert_eq!(t.quality(1, 8).unwrap().rejected, 3);
    for (code, level) in [
        (3, Severity::Warning),
        (4, Severity::Error),
        (8, Severity::Warning),
    ] {
        assert!(
            t.findings()
                .iter()
                .any(|f| f.code == code && f.severity == level)
        );
    }
    let mut b = TimelineBuilder::new(OfflinePolicy {
        channel: Some(99),
        ..Default::default()
    })
    .unwrap();
    let key = b
        .observe(packet(1, 1, 0, Some(reference(0))), false)
        .unwrap();
    let t = b.finish();
    assert!(t.lookup(key).unwrap().time.is_none());
    assert_eq!(
        t.lookup(key).unwrap().basis.channel,
        Some((99, ChannelChoice::Caller))
    );
    let mut b = TimelineBuilder::new(OfflinePolicy {
        include_invalid: true,
        include_undeclared: true,
        ..Default::default()
    })
    .unwrap();
    let mut r = reference(0);
    if let ReferenceValue::Readable(t, _) = r.value {
        r.value = ReferenceValue::Readable(t, false);
    }
    let mut p = packet(1, 1, 0, Some(r));
    p.declared_timein = false;
    let key = b.observe(p, false).unwrap();
    let t = b.finish();
    let a = t.lookup(key).unwrap();
    assert!(a.time.is_some());
    assert_eq!(
        a.basis.reference.unwrap().reasons,
        vec![Rejection::Undeclared, Rejection::Invalid]
    );
}

/// Requirements: L1-COR-002, L2-COR-102, L3-COR-102, L1-ANS-005, L2-ANS-105, L3-ANS-105, L1-POL-001, L2-POL-101, L3-POL-101
#[test]
fn nearest_preceding_ties_counter_only_and_policy_snapshots() {
    for (strategy, tie, next) in [
        (ReferenceStrategy::Nearest, true, false),
        (ReferenceStrategy::Nearest, false, true),
        (ReferenceStrategy::Preceding, false, false),
    ] {
        let policy = OfflinePolicy {
            strategy,
            preceding_on_tie: tie,
            ..Default::default()
        };
        let mut b = TimelineBuilder::new(policy.clone()).unwrap();
        let before = b.observe(packet(1, 3, 0, None), false).unwrap();
        b.observe(packet(2, 1, 10_000_000, Some(reference(1))), false)
            .unwrap();
        let key = b.observe(packet(3, 3, 15_000_000, None), false).unwrap();
        b.observe(packet(4, 1, 20_000_000, Some(reference(10))), false)
            .unwrap();
        let t = b.finish();
        assert!(t.lookup(before).unwrap().time.is_none());
        let a = t.lookup(key).unwrap();
        assert_eq!(
            a.basis.reference.unwrap().key.packet,
            if next { 4 } else { 2 }
        );
        assert_eq!(a.basis.policy, policy);
    }
    let mut b = TimelineBuilder::new(OfflinePolicy {
        back_extrapolate: true,
        ..Default::default()
    })
    .unwrap();
    let key = b.observe(packet(1, 3, 0, None), false).unwrap();
    b.observe(packet(2, 1, 10_000_000, Some(reference(1))), false)
        .unwrap();
    let a = b.finish().lookup(key).unwrap();
    assert!(a.time.is_some());
    assert_eq!(a.basis.position, ReferencePosition::Before);
}

/// Requirements: L1-COR-004, L2-COR-104, L3-COR-104, L1-COR-007, L2-COR-107, L3-COR-107, L1-COR-008, L2-COR-108, L3-COR-108, L1-COR-013, L2-COR-113, L3-COR-113
#[test]
fn quality_gap_jump_source_loss_and_duplicate_reasons() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 1, 0, Some(reference(0))), false)
        .unwrap();
    let mut r = reference(0);
    r.value = ReferenceValue::Readable(AbsoluteTime::new(100, 12, 0, 1, 100_000).unwrap(), true);
    b.observe(packet(2, 1, 10_000_000, Some(r)), false).unwrap();
    // Keep channels separate: unrelated intervening reference cannot shrink gap.
    b.observe(packet(3, 2, 15_000_000, Some(reference(1))), false)
        .unwrap();
    let mut r = reference(4);
    r.source = TimeF1Source::Source(TimeSource::Internal);
    let key = b.observe(packet(4, 1, 20_000_001, Some(r)), false).unwrap();
    let t = b.finish();
    let q = t.quality(1, 1).unwrap();
    assert_eq!(q.valid, 3);
    assert_eq!(q.min_spacing_ns, Some(1_000_000_000));
    assert_eq!(q.max_spacing_ns, Some(1_000_000_100));
    assert!(q.drift_ppm.unwrap() > 0.);
    for (code, level) in [
        (11, Severity::Information),
        (16, Severity::Warning),
        (17, Severity::Warning),
    ] {
        assert!(
            t.findings()
                .iter()
                .any(|f| f.code == code && f.severity == level)
        );
    }
    assert!(t.lookup(key).unwrap().time.is_some());
    let mut b = TimelineBuilder::new(OfflinePolicy {
        stop_on_loss: true,
        ..Default::default()
    })
    .unwrap();
    b.observe(packet(1, 1, 0, Some(reference(0))), false)
        .unwrap();
    let mut r = reference(1);
    r.source = TimeF1Source::Source(TimeSource::Internal);
    let key = b.observe(packet(2, 1, 10_000_000, Some(r)), false).unwrap();
    assert!(b.finish().lookup(key).unwrap().time.is_none());
    for duplicate in [false, true] {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(packet(1, 1, 0, Some(reference(0))), false)
            .unwrap();
        if duplicate {
            b.observe(packet(2, 1, 0, Some(reference(1))), false)
                .unwrap();
        }
        assert_eq!(
            b.finish().quality(1, 1).unwrap().unavailable,
            Some(if duplicate {
                "duplicate RTC"
            } else {
                "fewer than two valid references"
            })
        );
    }
}

/// Requirements: L1-COR-012, L2-COR-112, L3-COR-112, L1-POL-002, L2-POL-102, L3-POL-102, L1-ANS-001, L2-ANS-101, L3-ANS-101
#[test]
fn first_time_presence_scale_boundaries_and_invalid_queries() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut p = packet(1, 0, 0, None);
    p.kind = PacketKind::StaticSetup;
    b.observe(p, false).unwrap();
    let mut r = reference(0);
    r.source = TimeF1Source::Source(TimeSource::None);
    b.observe(packet(2, 1, 1, Some(r)), false).unwrap();
    let key = b.observe(packet(3, 3, 2, None), false).unwrap();
    let t = b.finish();
    assert!(t.lookup(key).unwrap().time.is_none());
    assert!(!t.findings().iter().any(|f| f.code == 1 || f.code == 2));
    assert!(t.lookup(PacketKey { session: 99, ..key }).is_err());
    assert!(
        t.lookup(PacketKey {
            ticks: i128::MAX,
            ..key
        })
        .is_err()
    );
    let p = OfflinePolicy {
        version: 2,
        ..Default::default()
    };
    assert!(TimelineBuilder::new(p).is_err());
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut r = reference(0);
    r.scale = ScaleEvidence::standard(TimeScale::Gps);
    r.format = TimeFormat::Gps;
    let key = b.observe(packet(1, 1, 0, Some(r)), false).unwrap();
    let t = b.finish();
    let a = t.lookup(key).unwrap();
    assert_eq!(
        a.basis.reference.unwrap().input.scale.effective(),
        TimeScale::Gps
    );
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let mut r = reference(0);
    r.value = ReferenceValue::Readable(
        AbsoluteTime::new(365, 23, 59, 59, 0)
            .unwrap()
            .with_year(Some(2023))
            .unwrap(),
        true,
    );
    r.year_origin = YearOrigin::Caller;
    b.observe(packet(1, 1, 0, Some(r)), false).unwrap();
    let key = b.observe(packet(2, 3, 20_000_000, None), false).unwrap();
    let a = b.finish().lookup(key).unwrap();
    assert_eq!(
        a.time.unwrap(),
        AbsoluteTime::new(1, 0, 0, 1, 0)
            .unwrap()
            .with_year(Some(2024))
            .unwrap()
    );
}

/// Requirements: L1-COR-003, L2-COR-103, L3-COR-103
#[test]
fn valid_count_id_and_caller_preference_are_ordered() {
    for (equal, preference, expected) in [(false, false, 2), (true, false, 1), (false, true, 1)] {
        let p = OfflinePolicy {
            include_undeclared: true,
            channel_preference: if preference { vec![99, 1] } else { vec![] },
            ..Default::default()
        };
        let mut b = TimelineBuilder::new(p).unwrap();
        b.observe(packet(1, 2, 0, Some(reference(0))), false)
            .unwrap();
        b.observe(packet(2, 2, 10_000_000, Some(reference(1))), false)
            .unwrap();
        b.observe(packet(3, 1, 0, Some(reference(0))), false)
            .unwrap();
        if equal {
            b.observe(packet(4, 1, 10_000_000, Some(reference(1))), false)
                .unwrap();
        }
        let key = b.observe(packet(5, 3, 15_000_000, None), false).unwrap();
        let mut undeclared = packet(6, 9, 15_000_000, Some(reference(1)));
        undeclared.declared_timein = false;
        undeclared.declared_external = true;
        b.observe(undeclared, false).unwrap();
        assert_eq!(
            b.finish().lookup(key).unwrap().basis.channel.unwrap().0,
            expected
        );
    }
}

/// Requirements: L1-COR-006, L2-COR-106, L3-COR-106
#[test]
fn none_format_never_supplies_a_readable_body() {
    let mut b = TimelineBuilder::new(OfflinePolicy {
        include_invalid: true,
        ..Default::default()
    })
    .unwrap();
    let mut r = reference(0);
    r.format = TimeFormat::None;
    let key = b.observe(packet(1, 1, 0, Some(r)), false).unwrap();
    let t = b.finish();
    assert!(t.lookup(key).unwrap().time.is_none());
    assert!(!t.sessions()[0].references[0].admitted);
}
