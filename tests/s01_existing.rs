//! S01 existing-API regressions. Ch11 §11.2.1.1 i: 48 bits at 10 MHz.
use irig106_time::{Rtc, stream::*, timeline::*};
fn packet(id: u64, rtc: Rtc) -> PacketObservation {
    PacketObservation {
        id,
        channel: 1,
        rtc,
        kind: PacketKind::Data,
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference: None,
    }
}
/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
#[test]
fn approved_ntp_levels_are_configurable_but_pending_levels_are_not() {
    let mut p = OfflinePolicy::default();
    for id in [29, 30] {
        p.finding_levels.insert(id, Severity::Information);
        assert!(p.validate().is_ok(), "approved TF-{id:03}");
    }
    for id in [0, 31, 32, 33, 255] {
        let mut bad = p.clone();
        bad.finding_levels.insert(id, Severity::Error);
        assert!(bad.validate().is_err(), "pending/unknown TF-{id:03}");
    }
}
/// Requirements: L1-RTC-003, L2-RTC-103, L3-RTC-103
#[test]
fn offline_half_range_requires_ordering_context() {
    // 2^48 modular ticks: 2^47 is equally far forward and backward.
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, Rtc::ZERO), false).unwrap();
    assert!(b.observe(packet(2, Rtc::from_raw(1 << 47)), false).is_err());
    // Rejected input must not consume its identity; caller can name a boundary.
    let key = b.observe(packet(2, Rtc::from_raw(1 << 47)), true).unwrap();
    assert_eq!(key.session, 2);
}
/// Requirements: L1-RTC-003, L2-RTC-103, L3-RTC-103
#[test]
fn stream_half_range_requires_ordering_context() {
    let mut s = StreamTimeline::new(
        StreamPolicy::new(
            1,
            BufferCaps {
                packets: 2,
                bytes: 128,
                references: 2,
            },
        ),
        None,
    )
    .unwrap();
    s.observe(packet(1, Rtc::ZERO), 0, 24, false).unwrap();
    assert!(
        s.observe(packet(2, Rtc::from_raw(1 << 47)), 0, 24, false)
            .is_err()
    );
    let b = s
        .observe(packet(2, Rtc::from_raw(1 << 47)), 0, 24, true)
        .unwrap();
    assert_eq!(b.answers[0].basis.key.session, 2);
}
/// Requirements: L1-RTC-001, L2-RTC-101, L3-RTC-101, L1-API-004, L2-API-104, L3-API-104
#[test]
fn shared_values_keep_width_and_unit_contracts() {
    let r: irig106_types::Rtc = Rtc::from_le_bytes([0xff; 6]);
    assert_eq!(r.as_raw(), (1 << 48) - 1);
    assert_eq!(Rtc::from_raw(1 << 48), Rtc::ZERO); // Explicitly named masking API.
    assert_eq!(Rtc::from_raw(1).to_nanos(), 100);
    assert_eq!(irig106_time::Ertc::from_raw(1).to_nanos(), 1);
}

/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
#[test]
fn both_engines_keep_structured_malformed_evidence_under_level_override() {
    let mut p = OfflinePolicy::default();
    p.finding_levels.insert(4, Severity::Information);
    let mut input = packet(77, Rtc::MAX);
    input.kind = PacketKind::Time; // No body: malformed, even at Information.
    let mut offline = TimelineBuilder::new(p.clone()).unwrap();
    let key = offline.observe(input.clone(), false).unwrap();
    let timeline = offline.finish().unwrap();
    assert!(timeline.lookup(key).unwrap().time.is_none());
    let mut policy = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 128,
            references: 2,
        },
    );
    policy.offline = p;
    let mut stream = StreamTimeline::new(policy, None).unwrap();
    let batch = stream.observe(input, 0, 24, false).unwrap();
    for findings in [timeline.findings(), batch.findings.as_slice()] {
        let f = findings.iter().find(|f| f.code == 4).unwrap();
        assert_eq!(f.severity, Severity::Information);
        assert_eq!(
            f.evidence.validity(),
            irig106_time::findings::ValueValidity::Unusable
        );
        assert_eq!(f.evidence.raw_rtc(), Some(Rtc::MAX));
        assert_eq!(f.evidence.location().packet, Some(77));
        assert_eq!(f.evidence.location().channel, Some(1));
        assert_eq!(f.evidence.location().byte_offset, None);
    }
}

/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
#[test]
fn both_engines_keep_related_identity_and_signed_residual() {
    use irig106_time::findings::RawValue;
    use irig106_time::{
        AbsoluteTime, TimeF1Source, TimeFormat, TimeSource,
        scale::{ScaleEvidence, TimeScale},
    };
    // Ch11 §11.2.1.1 i: 10000000 ticks = 1 s. Absolute advances 0.5/1.5 s,
    // so jump residual = -0.5/+0.5 s, independently of the crate's arithmetic.
    for (second, expected) in [(10, -500_000_000), (11, 500_000_000)] {
        let reference = |id, ticks, second, ns| {
            let mut p = packet(id, Rtc::from_raw(ticks));
            p.kind = PacketKind::Time;
            p.reference = Some(ReferenceInput {
                value: ReferenceValue::Readable(
                    AbsoluteTime::new(100, 0, 0, second, ns).unwrap(),
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
            });
            p
        };
        let first = reference(41, 1_000_000, 10, 0);
        let next = reference(99, 11_000_000, second, 500_000_000);
        let mut offline = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        offline.observe(first.clone(), false).unwrap();
        offline.observe(next.clone(), false).unwrap();
        let timeline = offline.finish().unwrap();
        let mut stream = StreamTimeline::new(
            StreamPolicy::new(
                1,
                BufferCaps {
                    packets: 2,
                    bytes: 128,
                    references: 2,
                },
            ),
            Some(&[StreamChannel {
                id: 1,
                timein: true,
                external: true,
            }]),
        )
        .unwrap();
        stream.observe(first, 0, 24, false).unwrap();
        let batch = stream.observe(next, 1, 24, false).unwrap();
        for findings in [timeline.findings(), batch.findings.as_slice()] {
            let f = findings.iter().find(|f| f.code == 17).unwrap();
            assert_eq!(f.related, Some(41));
            assert_eq!(f.delta_ns, Some(expected));
            assert_eq!(f.evidence.related()[0].packet, Some(41));
            assert_eq!(f.evidence.raw_fields()[0].name, "delta_ns");
            assert_eq!(
                f.evidence.raw_fields()[0].observed,
                RawValue::Signed(expected)
            );
        }
    }
}
