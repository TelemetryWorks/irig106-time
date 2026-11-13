//! E2 hand-derived stream fixtures; no encoder oracle or host clock.
use irig106_time::scale::{ScaleEvidence, ScalePolicy, TimeScale};
use irig106_time::stream::*;
use irig106_time::timeline::*;
use irig106_time::{AbsoluteTime, Rtc, TimeF1Layout, TimeF1Source, TimeFormat, TimeSource};
fn policy(mode: StreamMode) -> StreamPolicy {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 8,
            bytes: 800,
            references: 4,
        },
    );
    p.mode = mode;
    p.offline.channel = Some(1);
    p
}
fn reference(ns: u64) -> ReferenceInput {
    ReferenceInput {
        value: ReferenceValue::Readable(
            AbsoluteTime::new(100, 12, 0, 0, 0).unwrap().add_nanos(ns),
            true,
        ),
        scale: ScaleEvidence::assumed(TimeScale::Utc),
        source: TimeF1Source::Source(TimeSource::External),
        format: TimeFormat::IrigB,
        its: None,
        precision_ns: 10_000_000,
        wire: None,
        raw_csdw: None,
        year_origin: YearOrigin::Unknown,
    }
}
fn packet(id: u64, ticks: u64, r: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: if r.is_some() { 1 } else { 9 },
        rtc: Rtc::from_raw(ticks),
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
fn stream(mode: StreamMode) -> StreamTimeline {
    StreamTimeline::new(policy(mode), None).unwrap()
}
fn held(p: StreamPolicy) -> StreamTimeline {
    let mut s = StreamTimeline::new(p, None).unwrap();
    s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
        .unwrap();
    assert!(
        s.observe(packet(2, 5_000_000, None), 0, 100, false)
            .unwrap()
            .answers
            .is_empty()
    );
    s
}
fn finding(batch: &StreamBatch, code: u8, level: Severity) {
    assert!(
        batch
            .findings
            .iter()
            .any(|f| f.code == code && f.severity == level),
        "missing TF-{code:03}"
    );
}
/// Requirements: L1-STRM-001, L2-STRM-201, L3-STRM-201, L2-POL-201, L3-POL-201, L1-POL-001, L1-ANS-001, L2-ANS-101, L3-ANS-101
#[test]
fn handbook_standard_and_contract_basis() {
    // RCC 123-20 5.6 printed 5-44/PDF78 gives numbers, not packet bytes.
    // Envelope hand-derived from Ch11 11.2.1.1, Figures 11-12/13.
    let handbook = [
        0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0x40, 0x42, 0x0f, 0, 0, 0,
        0xa9, 0x3e, 1, 0x30, 0, 0, 0, 0x25, 0x30, 0x12, 0, 1, 0, 0,
    ];
    // Contract 7.2, all 36 bytes verbatim.
    let contract = [
        0x25, 0xeb, 1, 0, 0x24, 0, 0, 0, 10, 0, 0, 0, 6, 0, 0, 0x11, 0, 0xe1, 0xf5, 5, 0, 0, 0x4f,
        0xe3, 1, 0x30, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x87, 1, 0, 0,
    ];
    for (bytes, target, expected) in [
        (
            handbook,
            1_150_000,
            AbsoluteTime::new(100, 12, 30, 25, 15_000_000).unwrap(),
        ),
        (
            contract,
            104_200_000,
            AbsoluteTime::new(187, 13, 45, 27, 770_000_000).unwrap(),
        ),
    ] {
        let mut s = stream(StreamMode::Standard);
        let r = ReferenceInput::format1(
            &bytes[24..],
            10,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap();
        let ticks = Rtc::from_le_bytes(bytes[16..22].try_into().unwrap()).as_raw();
        s.observe(packet(1, ticks, Some(r)), 0, 36, false).unwrap();
        let a = s
            .observe(packet(2, target, None), 1, 36, false)
            .unwrap()
            .answers
            .remove(0);
        assert_eq!(a.time, Some(expected));
        assert_eq!(a.basis.policy.mode, StreamMode::Standard);
        assert_eq!(a.basis.release, ReleaseReason::Standard);
        assert_eq!(
            a.basis.preceding.unwrap().input.its,
            Some(irig106_time::IrigTimeSource::ExternalIrig)
        );
        assert!(a.basis.following.is_none());
    }
    assert_eq!(
        StreamPolicy::new(1, policy(StreamMode::Standard).caps).mode,
        StreamMode::Standard
    );
}
/// Requirements: L1-STRM-001, L2-STRM-202, L3-STRM-202, L1-ANS-001, L2-ANS-101, L3-ANS-101
#[test]
fn interpolation_drift_endpoints_and_floor_proposal() {
    let mut s = held(policy(StreamMode::Interpolate));
    // 10 MHz RTC: 1 s, absolute interval 1.000010 s => midpoint .500005 s.
    let b = s
        .observe(
            packet(3, 10_000_000, Some(reference(1_000_010_000))),
            1_000_000_000,
            100,
            false,
        )
        .unwrap();
    let a = &b.answers[0];
    assert_eq!(
        a.time,
        Some(AbsoluteTime::new(100, 12, 0, 0, 500_005_000).unwrap())
    );
    assert_eq!(a.basis.release, ReleaseReason::Interpolated);
    assert_eq!(a.basis.drift_correction_ns, Some(5_000));
    assert_eq!(a.basis.rounding, Some("floor (L3 proposal)"));
    assert_eq!(a.basis.precision_ns, Some(10_000_000));
    assert_eq!(a.basis.preceding.as_ref().unwrap().key.packet, 1);
    assert_eq!(a.basis.following.as_ref().unwrap().key.packet, 3);
    // Separate synthetic arithmetic oracle: 302/3=100+2/3 ns, floor=100.
    let mut f = stream(StreamMode::Interpolate);
    f.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
        .unwrap();
    f.observe(packet(2, 1, None), 0, 100, false).unwrap();
    let b = f
        .observe(packet(3, 3, Some(reference(302))), 1, 100, false)
        .unwrap();
    assert_eq!(
        b.answers[0].time,
        Some(AbsoluteTime::new(100, 12, 0, 0, 100).unwrap())
    );
    // Exact known endpoint needs no look-ahead.
    let a = s
        .observe(packet(4, 10_000_000, None), 1_000_000_000, 100, false)
        .unwrap()
        .answers
        .remove(0);
    assert_eq!(
        a.time,
        Some(AbsoluteTime::new(100, 12, 0, 1, 10_000).unwrap())
    );
}
/// Requirements: L1-STRM-001, L2-STRM-203, L3-STRM-203, L1-COR-008, L2-COR-108, L3-COR-108
#[test]
fn hold_gap_and_explicit_progress_boundaries() {
    for spacing in [
        1_000_000_000u64,
        1_000_000_100,
        1_050_000_000,
        1_100_000_000,
    ] {
        let mut p = policy(StreamMode::Interpolate);
        p.raw_network_rate = true;
        let mut s = held(p);
        let b = s
            .observe(
                packet(3, spacing / 100, Some(reference(spacing))),
                1_100_000_000,
                100,
                false,
            )
            .unwrap();
        assert_eq!(b.answers[0].basis.release, ReleaseReason::Interpolated);
        assert!(b.answers[0].basis.policy.raw_network_rate);
        assert_eq!(
            b.findings.iter().any(|f| f.code == 16),
            spacing > 1_000_000_000
        );
    }
    let mut s = held(policy(StreamMode::Interpolate));
    assert!(s.advance(1_100_000_000).unwrap().answers.is_empty());
    let b = s.advance(1_100_000_001).unwrap();
    assert_eq!(b.answers[0].basis.release, ReleaseReason::Timeout);
    assert_eq!(b.answers[0].basis.deadline_ns, Some(1_100_000_000));
    assert!(s.advance(1).is_err());
    let mut p = policy(StreamMode::Interpolate);
    p.hold_tolerance_ns = 0;
    p.offline.gap_tolerance_ns = 100_000_000;
    let mut s = held(p);
    let b = s
        .observe(
            packet(3, 10_500_000, Some(reference(1_050_000_000))),
            1_000_000_001,
            100,
            false,
        )
        .unwrap();
    assert_eq!(b.answers[0].basis.release, ReleaseReason::Timeout);
    assert!(!b.findings.iter().any(|f| f.code == 16));
}
/// Requirements: L1-STRM-001, L2-STRM-203, L3-STRM-203, L2-STRM-205, L3-STRM-205, L2-STRM-206, L3-STRM-206, L1-POL-002, L2-POL-102, L3-POL-102
#[test]
fn every_resource_and_lifecycle_fallback_is_labelled() {
    for reason in [
        ReleaseReason::PacketCap,
        ReleaseReason::ByteCap,
        ReleaseReason::ReferenceCap,
        ReleaseReason::Eof,
        ReleaseReason::ModeChange,
        ReleaseReason::SessionBoundary,
    ] {
        let mut p = policy(StreamMode::Interpolate);
        if reason == ReleaseReason::PacketCap {
            p.caps.packets = 1;
        }
        if reason == ReleaseReason::ByteCap {
            p.caps.bytes = 100;
        }
        if reason == ReleaseReason::ReferenceCap {
            p.caps.references = 2;
        }
        let mut s = held(p.clone());
        let b = match reason {
            ReleaseReason::Eof => s.finish(1).unwrap(),
            ReleaseReason::ModeChange => {
                p.id = 2;
                p.mode = StreamMode::Standard;
                s.set_policy(p, 1).unwrap()
            }
            ReleaseReason::SessionBoundary => {
                s.observe(packet(3, 6_000_000, None), 1, 100, true).unwrap()
            }
            ReleaseReason::ReferenceCap => {
                // Out-of-order future neighbours fill the mandatory reference cap.
                s.observe(
                    packet(3, 9_000_000, Some(reference(900_000_000))),
                    1,
                    100,
                    false,
                )
                .unwrap();
                // Start another pending query after last reference.
                s.observe(packet(4, 9_500_000, None), 1, 100, false)
                    .unwrap();
                s.observe(
                    packet(5, 10_000_000, Some(reference(1_000_000_000))),
                    2,
                    100,
                    false,
                )
                .unwrap()
            }
            _ => s
                .observe(packet(3, 6_000_000, None), 1, 100, false)
                .unwrap(),
        };
        let a = b
            .answers
            .iter()
            .find(|a| a.basis.release == reason)
            .expect("fallback reason missing");
        assert!(a.time.is_some());
        assert!(a.basis.extrapolated);
        assert_eq!(a.basis.policy.id, 1);
        assert_eq!(a.basis.policy.mode, StreamMode::Interpolate);
        assert!(a.basis.following.is_none());
        assert_eq!(a.basis.drift_correction_ns, None);
        let u = s.usage();
        assert!(u.packets <= s.policy().caps.packets);
        assert!(u.bytes <= s.policy().caps.bytes);
        assert!(u.references <= s.policy().caps.references);
    }
}
/// Requirements: L1-STRM-001, L2-STRM-204, L3-STRM-204, L1-ANS-005, L2-ANS-105, L3-ANS-105, L1-COR-012, L2-COR-112, L3-COR-112
/// Interpretations: RDG-011
#[test]
fn early_data_none_and_no_time_findings() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = stream(mode);
        let b = s.observe(packet(1, 0, None), 0, 100, false).unwrap();
        assert!(b.answers[0].time.is_none());
        finding(&b, 2, Severity::Warning);
        let end = s.finish(1).unwrap();
        finding(&end, 1, Severity::Warning);
        let mut s = stream(mode);
        let mut r = reference(0);
        r.source = TimeF1Source::Source(TimeSource::None);
        let b = s.observe(packet(1, 0, Some(r)), 0, 100, false).unwrap();
        finding(&b, 3, Severity::Warning);
        let b = s.observe(packet(2, 1, None), 1, 100, false).unwrap();
        assert!(b.answers[0].time.is_none());
        assert!(!b.findings.iter().any(|f| f.code == 2));
        assert!(!s.finish(2).unwrap().findings.iter().any(|f| f.code == 1));
    }
}
/// Requirements: L1-COR-003, L2-COR-103, L3-COR-103, L1-COR-007, L2-COR-107, L3-COR-107, L1-STRM-001, L2-STRM-206, L3-STRM-206
#[test]
fn commitment_inventory_loss_and_no_revision() {
    let inventory = [
        StreamChannel {
            id: 7,
            timein: true,
            external: false,
        },
        StreamChannel {
            id: 3,
            timein: true,
            external: true,
        },
        StreamChannel {
            id: 2,
            timein: true,
            external: true,
        },
    ];
    let mut p = policy(StreamMode::Standard);
    p.offline.channel = None;
    let s = StreamTimeline::new(p.clone(), Some(&inventory)).unwrap();
    assert_eq!(
        s.channel(),
        Some((2, StreamChannelChoice::DeclaredExternal))
    );
    let ordinary = [
        StreamChannel {
            id: 7,
            timein: true,
            external: false,
        },
        StreamChannel {
            id: 1,
            timein: true,
            external: false,
        },
    ];
    assert_eq!(
        StreamTimeline::new(p.clone(), Some(&ordinary))
            .unwrap()
            .channel(),
        Some((1, StreamChannelChoice::LowestId))
    );
    let mut s = StreamTimeline::new(p, None).unwrap();
    let early = s
        .observe(packet(1, 0, None), 0, 100, false)
        .unwrap()
        .answers
        .remove(0);
    s.choose_channel(1, 2, 1).unwrap();
    s.observe(packet(2, 0, Some(reference(0))), 1, 100, false)
        .unwrap();
    let old = s
        .observe(packet(3, 5_000_000, None), 2, 100, false)
        .unwrap()
        .answers
        .remove(0);
    let mut r = reference(1_000_000_000);
    r.source = TimeF1Source::Source(TimeSource::Internal);
    let b = s
        .observe(packet(4, 10_000_000, Some(r)), 3, 100, false)
        .unwrap();
    finding(&b, 11, Severity::Information);
    assert_eq!(s.channel(), Some((1, StreamChannelChoice::Caller)));
    assert_eq!(
        old.time,
        Some(AbsoluteTime::new(100, 12, 0, 0, 500_000_000).unwrap())
    );
    assert!(early.time.is_none());
    for id in 5..15 {
        let mut p = packet(id, id * 10_000_000, Some(reference(id * 1_000_000_000)));
        p.channel = 7;
        s.observe(p, id, 100, false).unwrap();
    }
    assert_eq!(s.channel(), Some((1, StreamChannelChoice::Caller)));
    assert!(s.choose_channel(2, 3, 15).is_err());
}
/// Requirements: L1-STRM-001, L2-STRM-202, L3-STRM-202, L2-STRM-205, L3-STRM-205, L1-COR-004, L2-COR-104, L3-COR-104, L1-COR-005, L2-COR-105, L3-COR-105
#[test]
fn discontinuities_and_scale_never_bridge() {
    for (reason, ticks) in [
        (ReleaseReason::Discontinuity, 0),
        (ReleaseReason::Discontinuity, 10_000_000),
        (ReleaseReason::IncompatibleScale, 10_000_000),
        (ReleaseReason::UnusableReference, 10_000_000),
    ] {
        let mut p = policy(StreamMode::Interpolate);
        p.offline.stop_on_loss = reason == ReleaseReason::UnusableReference;
        let mut s = held(p);
        let mut r = reference(1_000_000_000);
        if reason == ReleaseReason::Discontinuity {
            r = reference(2_000_000_000);
        }
        if reason == ReleaseReason::IncompatibleScale {
            r.scale = ScaleEvidence::standard(TimeScale::Gps);
        }
        if reason == ReleaseReason::UnusableReference {
            r.source = TimeF1Source::Source(TimeSource::Internal);
        }
        let b = s.observe(packet(3, ticks, Some(r)), 1, 100, false).unwrap();
        assert_eq!(b.answers[0].basis.release, reason);
        assert!(b.answers[0].basis.following.is_none());
        if reason == ReleaseReason::Discontinuity {
            finding(&b, 17, Severity::Warning);
        }
    }
    let mut s = held(policy(StreamMode::Interpolate));
    let b = s
        .observe(packet(3, (1u64 << 48) - 20_000_000, None), 1, 100, false)
        .unwrap();
    finding(&b, 18, Severity::Information);
    finding(&b, 19, Severity::Warning);
    assert_eq!(b.answers[0].basis.release, ReleaseReason::SessionBoundary);
    assert!(b.answers.last().unwrap().time.is_none());
}
/// Requirements: L1-FND-003, L2-FND-103, L3-FND-103, L1-COR-006, L2-COR-106, L3-COR-106
#[test]
fn admission_findings_and_raw_sync_counter() {
    for (code, mut r) in [
        (3, reference(0)),
        (4, reference(0)),
        (6, reference(0)),
        (8, reference(0)),
    ] {
        if code == 3 {
            r.value = ReferenceValue::Invalid;
        }
        if code == 4 {
            r.value = ReferenceValue::Malformed;
        }
        if code == 6 {
            r.source = TimeF1Source::Source(TimeSource::Reserved(3));
        }
        let mut p = packet(1, 123, Some(r));
        if code == 8 {
            p.declared_timein = false;
        }
        let mut s = stream(StreamMode::Standard);
        let b = s.observe(p, 0, 100, false).unwrap();
        finding(
            &b,
            code,
            if code == 4 {
                Severity::Error
            } else {
                Severity::Warning
            },
        );
    }
    let mut p = policy(StreamMode::Interpolate);
    p.offline.include_invalid = true;
    let mut s = StreamTimeline::new(p, None).unwrap();
    let mut r = reference(0);
    r.value = ReferenceValue::Readable(AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(), false);
    s.observe(packet(1, 0, Some(r)), 0, 100, false).unwrap();
    let b = s
        .observe(packet(2, 5_000_000, None), 1, 100, false)
        .unwrap();
    assert_eq!(b.answers[0].basis.release, ReleaseReason::UnusableReference);
    assert!(b.answers[0].basis.extrapolated);
    for flags in 0..=255 {
        let mut s = stream(StreamMode::Standard);
        let mut p = packet(1, 123, None);
        p.flags = flags;
        let b = s.observe(p, 0, 100, false).unwrap();
        let f = b.findings.iter().find(|f| f.code == 24);
        assert_eq!(f.is_some(), flags & 0x20 != 0);
        if let Some(f) = f {
            assert_eq!(f.raw_rtc, Some(Rtc::from_raw(123)));
            assert_eq!(f.severity, Severity::Warning);
        }
    }
}
/// Requirements: L1-STRM-001, L2-STRM-101, L3-STRM-101, L2-STRM-205, L3-STRM-205, L1-POL-001, L2-POL-101, L3-POL-101, L1-POL-002, L2-POL-102, L3-POL-102
#[test]
fn bounded_burst_configuration_and_frozen_policy() {
    let mut p = policy(StreamMode::Interpolate);
    p.caps.packets = 2;
    p.caps.bytes = 200;
    p.caps.references = 2;
    let mut s = StreamTimeline::new(p.clone(), None).unwrap();
    s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
        .unwrap();
    for id in 2..1002 {
        s.observe(packet(id, id * 100, None), 0, 100, false)
            .unwrap();
        let u = s.usage();
        assert!(u.packets <= 2 && u.bytes <= 200 && u.references <= 2);
    }
    let b = s.finish(1).unwrap();
    assert!(!b.answers.is_empty());
    assert!(s.observe(packet(1002, 100, None), 2, 100, false).is_err());
    for caps in [
        BufferCaps {
            packets: 0,
            bytes: 1,
            references: 2,
        },
        BufferCaps {
            packets: 1,
            bytes: 0,
            references: 2,
        },
        BufferCaps {
            packets: 1,
            bytes: 1,
            references: 1,
        },
    ] {
        p.caps = caps;
        assert!(StreamTimeline::new(p.clone(), None).is_err());
    }
}

/// Requirements: L1-STRM-001, L2-STRM-201, L3-STRM-201, L2-STRM-202, L3-STRM-202, L2-STRM-203, L3-STRM-203, L2-STRM-205, L3-STRM-205, L2-STRM-206, L3-STRM-206
#[test]
fn late_endpoints_policy_transitions_and_oversized_data() {
    let mut s = held(policy(StreamMode::Interpolate));
    let b = s
        .observe(
            packet(3, 5_000_000, Some(reference(500_000_000))),
            1,
            100,
            false,
        )
        .unwrap();
    assert_eq!(
        b.answers[0].time,
        Some(AbsoluteTime::new(100, 12, 0, 0, 500_000_000).unwrap())
    );
    let mut s = held(policy(StreamMode::Interpolate));
    let b = s
        .observe(
            packet(3, 10_000_000, Some(reference(1_000_000_000))),
            1_100_000_001,
            100,
            false,
        )
        .unwrap();
    assert_eq!(b.answers[0].basis.release, ReleaseReason::Timeout);
    let mut s = stream(StreamMode::Standard);
    s.observe(
        packet(1, 10_000_000, Some(reference(1_000_000_000))),
        0,
        100,
        false,
    )
    .unwrap();
    assert!(
        s.observe(packet(2, 5_000_000, None), 1, 100, false)
            .unwrap()
            .answers[0]
            .time
            .is_none()
    );
    let mut s = held(policy(StreamMode::Interpolate));
    let old = s.set_policy(policy(StreamMode::Standard), 1);
    assert!(old.is_err());
    let mut p = policy(StreamMode::Standard);
    p.id = 2;
    let old = s.set_policy(p.clone(), 1).unwrap().answers.remove(0);
    p.id = 3;
    p.mode = StreamMode::Interpolate;
    s.set_policy(p, 2).unwrap();
    assert_eq!(old.basis.policy.id, 1);
    assert_eq!(old.basis.release, ReleaseReason::ModeChange);
    let b = s
        .observe(packet(3, 6_000_000, None), 3, 801, false)
        .unwrap();
    assert_eq!(b.answers[0].basis.release, ReleaseReason::ByteCap);
    assert_eq!(s.usage().packets, 0);
}
/// Requirements: L1-COR-001, L2-COR-101, L3-COR-101, L1-TAI-007, L2-TAI-107, L3-TAI-107, L1-COR-007, L2-COR-107, L3-COR-107
/// Interpretations: RDG-009, RDG-010
#[test]
fn reserved_leap_and_stop_on_loss_evidence() {
    // 106-24R1 Figure11-13 printed11-25/PDF33: word0 bit15 reserved;
    // UTC second60 retains its label (RDG-010), never silently normalizes.
    for (bytes, code, level) in [
        (
            [1, 0x30, 0, 0, 0x35, 0xa7, 0x45, 0x13, 0x87, 1],
            5,
            Severity::Warning,
        ),
        (
            [0x41, 0, 0, 0, 0, 0x60, 0x59, 0x23, 0x65, 3],
            22,
            Severity::Information,
        ),
    ] {
        let r = ReferenceInput::format1(
            &bytes,
            10,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap();
        let mut s = stream(StreamMode::Standard);
        let b = s.observe(packet(1, 0, Some(r)), 0, 100, false).unwrap();
        finding(&b, code, level);
    }
    let mut p = policy(StreamMode::Standard);
    p.offline.stop_on_loss = true;
    let mut s = StreamTimeline::new(p, None).unwrap();
    s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
        .unwrap();
    let mut r = reference(1_000_000_000);
    r.source = TimeF1Source::Source(TimeSource::Internal);
    let b = s
        .observe(packet(2, 10_000_000, Some(r)), 1, 100, false)
        .unwrap();
    finding(&b, 11, Severity::Information);
    assert!(
        s.observe(packet(3, 11_000_000, None), 2, 100, false)
            .unwrap()
            .answers[0]
            .time
            .is_none()
    );
    assert_eq!(s.channel(), Some((1, StreamChannelChoice::Caller)));
}

/// Requirements: L1-STRM-001, L2-STRM-201, L3-STRM-201, L2-STRM-202, L3-STRM-202, L2-STRM-204, L3-STRM-204, L2-STRM-205, L3-STRM-205, L1-COR-006, L2-COR-106, L3-COR-106
#[test]
fn review_unusable_reference_keeps_eligible_predecessor() {
    // Review probe: valid at0, data+.2s, source None at+.5s, data+.6s.
    // None is one unusable reference, not a discontinuity in valid clock time.
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        for invalid in 0..6 {
            let mut s = stream(mode);
            s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
                .unwrap();
            let first = s
                .observe(packet(2, 2_000_000, None), 0, 100, false)
                .unwrap();
            let mut r = reference(500_000_000);
            match invalid {
                0 => r.source = TimeF1Source::Source(TimeSource::None),
                1 => r.value = ReferenceValue::Invalid,
                2 => r.value = ReferenceValue::Malformed,
                4 => {
                    r.value = ReferenceValue::Readable(
                        AbsoluteTime::new(100, 12, 0, 0, 500_000_000).unwrap(),
                        false,
                    )
                }
                5 => r.format = TimeFormat::None,
                _ => {}
            }
            let mut p = packet(3, 5_000_000, Some(r));
            if invalid == 3 {
                p.reference = None;
            }
            let bad = s.observe(p, 0, 100, false).unwrap();
            finding(
                &bad,
                if invalid == 2 || invalid == 3 { 4 } else { 3 },
                if invalid == 2 || invalid == 3 {
                    Severity::Error
                } else {
                    Severity::Warning
                },
            );
            assert!(
                bad.answers.is_empty(),
                "unusable reference must not drain pending answers"
            );
            assert_eq!(s.usage().references, 1);
            let later = s
                .observe(packet(4, 6_000_000, None), 0, 100, false)
                .unwrap();
            let answers = if mode == StreamMode::Standard {
                assert_eq!(
                    first.answers[0].time,
                    Some(AbsoluteTime::new(100, 12, 0, 0, 200_000_000).unwrap())
                );
                assert_eq!(
                    later.answers[0].time,
                    Some(AbsoluteTime::new(100, 12, 0, 0, 600_000_000).unwrap())
                );
                later.answers
            } else {
                assert!(first.answers.is_empty() && later.answers.is_empty());
                let b = s
                    .observe(
                        packet(5, 10_000_000, Some(reference(1_000_010_000))),
                        900_000_000,
                        100,
                        false,
                    )
                    .unwrap();
                assert_eq!(b.answers.len(), 2);
                assert_eq!(
                    b.answers[0].time,
                    Some(AbsoluteTime::new(100, 12, 0, 0, 200_002_000).unwrap())
                );
                assert_eq!(
                    b.answers[1].time,
                    Some(AbsoluteTime::new(100, 12, 0, 0, 600_006_000).unwrap())
                );
                assert!(
                    b.answers
                        .iter()
                        .all(|a| a.basis.release == ReleaseReason::Interpolated)
                );
                b.answers
            };
            for a in answers {
                let r = a.basis.preceding.unwrap();
                assert_eq!((r.key.packet, r.channel), (1, 1));
            }
        }
    }
}
/// Requirements: L1-COR-003, L2-COR-103, L3-COR-103, L1-STRM-001, L2-STRM-206, L3-STRM-206
#[test]
fn review_inventory_external_precedes_lower_id() {
    let mut p = policy(StreamMode::Standard);
    p.offline.channel = None;
    let candidates = [
        StreamChannel {
            id: 1,
            timein: true,
            external: false,
        },
        StreamChannel {
            id: 7,
            timein: true,
            external: true,
        },
    ];
    assert_eq!(
        StreamTimeline::new(p, Some(&candidates)).unwrap().channel(),
        Some((7, StreamChannelChoice::DeclaredExternal))
    );
}
/// Requirements: L1-COR-003, L2-COR-103, L3-COR-103, L1-STRM-001, L2-STRM-206, L3-STRM-206
#[test]
fn review_inventory_lowest_id_excludes_non_timein() {
    let mut p = policy(StreamMode::Standard);
    p.offline.channel = None;
    let candidates = [
        StreamChannel {
            id: 0,
            timein: false,
            external: false,
        },
        StreamChannel {
            id: 7,
            timein: true,
            external: false,
        },
    ];
    assert_eq!(
        StreamTimeline::new(p, Some(&candidates)).unwrap().channel(),
        Some((7, StreamChannelChoice::LowestId))
    );
}
/// Requirements: L1-COR-003, L2-COR-103, L3-COR-103, L1-ANS-001, L2-ANS-101, L3-ANS-101, L1-STRM-001, L2-STRM-201, L3-STRM-201, L2-STRM-202, L3-STRM-202
#[test]
fn review_noncommitted_references_cannot_supply_answers() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = stream(mode);
        s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
            .unwrap();
        let mut other = packet(2, 1_000_000, Some(reference(10_000_000_000)));
        other.channel = 2;
        s.observe(other, 0, 100, false).unwrap();
        let b = s
            .observe(packet(3, 6_000_000, None), 0, 100, false)
            .unwrap();
        let a = if mode == StreamMode::Standard {
            assert_eq!(
                b.answers[0].time,
                Some(AbsoluteTime::new(100, 12, 0, 0, 600_000_000).unwrap())
            );
            b.answers.into_iter().next().unwrap()
        } else {
            assert!(b.answers.is_empty());
            let b = s
                .observe(
                    packet(4, 10_000_000, Some(reference(1_000_010_000))),
                    1,
                    100,
                    false,
                )
                .unwrap();
            assert_eq!(
                b.answers[0].time,
                Some(AbsoluteTime::new(100, 12, 0, 0, 600_006_000).unwrap())
            );
            assert_eq!(b.answers[0].basis.following.as_ref().unwrap().channel, 1);
            b.answers.into_iter().next().unwrap()
        };
        assert_eq!(a.basis.preceding.as_ref().unwrap().channel, 1);
        assert_eq!(a.basis.preceding.unwrap().key.packet, 1);
    }
}
/// Requirements: L1-STRM-001, L2-STRM-101, L3-STRM-101, L1-POL-002, L2-POL-102, L3-POL-102
#[test]
fn review_nonincreasing_packet_ids_rejected() {
    let mut s = stream(StreamMode::Standard);
    s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
        .unwrap();
    s.observe(packet(3, 2_000_000, None), 0, 100, false)
        .unwrap();
    for id in [3, 2] {
        let err = s
            .observe(packet(id, 4_000_000, None), 0, 100, false)
            .unwrap_err();
        assert_eq!(
            err,
            irig106_time::TimeError::InvalidTimelineInput(
                "byte charge, identity order or non-time reference"
            )
        );
    }
    let b = s
        .observe(packet(4, 6_000_000, None), 0, 100, false)
        .unwrap();
    assert_eq!(
        b.answers[0].time,
        Some(AbsoluteTime::new(100, 12, 0, 0, 600_000_000).unwrap())
    );
}

/// Requirements: L1-STRM-001, L2-STRM-202, L3-STRM-202, L2-STRM-205, L3-STRM-205, L1-COR-006, L2-COR-106, L3-COR-106
#[test]
fn review_admitted_invalid_does_not_barrier_valid_interpolation() {
    let mut p = policy(StreamMode::Interpolate);
    p.offline.include_invalid = true;
    let mut s = StreamTimeline::new(p, None).unwrap();
    s.observe(packet(1, 0, Some(reference(0))), 0, 100, false)
        .unwrap();
    assert!(
        s.observe(packet(2, 2_000_000, None), 0, 100, false)
            .unwrap()
            .answers
            .is_empty()
    );
    let mut r = reference(500_000_000);
    r.value = ReferenceValue::Readable(
        AbsoluteTime::new(100, 12, 0, 0, 500_000_000).unwrap(),
        false,
    );
    let b = s
        .observe(packet(3, 5_000_000, Some(r)), 0, 100, false)
        .unwrap();
    finding(&b, 3, Severity::Warning);
    assert!(b.answers.is_empty());
    let b = s
        .observe(
            packet(4, 10_000_000, Some(reference(1_000_010_000))),
            1,
            100,
            false,
        )
        .unwrap();
    assert_eq!(
        b.answers[0].time,
        Some(AbsoluteTime::new(100, 12, 0, 0, 200_002_000).unwrap())
    );
    assert_eq!(b.answers[0].basis.preceding.as_ref().unwrap().key.packet, 1);
    assert_eq!(b.answers[0].basis.following.as_ref().unwrap().key.packet, 4);
    assert_eq!(b.answers[0].basis.release, ReleaseReason::Interpolated);
}
