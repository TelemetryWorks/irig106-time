//! S06b2 clause evidence: 106-24R1 Figs11-14..17, printed11-25..27/PDF33..35.
//! Partial clause checks deliberately have no whole-requirement marker.
use irig106_time::{format2::*, leap::*, ntp::*, scale::*, stream::*, timeline::*, *};
fn policy(mode: StreamMode) -> StreamPolicy {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 3,
            bytes: 300,
            references: 3,
        },
    );
    assert_eq!(p.mode, StreamMode::Standard);
    assert_eq!(p.hold_tolerance_ns, 100_000_000);
    assert_eq!(p.offline.gap_tolerance_ns, 0);
    assert!(!p.raw_network_rate);
    p.mode = mode;
    p.offline.channel = Some(1);
    p.offline.jump_threshold_ns = Some(20_000);
    p
}
fn packet(id: u64, ch: u16, ticks: u64, r: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: ch,
        rtc: Rtc::from_raw(ticks),
        kind: if r.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference: r,
    }
}
fn net(at: (usize, u64, u16, u64), wire: [u32; 3], raw: bool) -> ReferenceInput {
    contextual_net(
        at,
        wire,
        raw,
        NtpContext::new(at.0),
        true,
        edition::PacketEdition::V24R1,
    )
}
fn contextual_net(
    at: (usize, u64, u16, u64),
    wire: [u32; 3],
    raw: bool,
    context: NtpContext,
    declared: bool,
    edition: edition::PacketEdition,
) -> ReferenceInput {
    let (session, id, ch, ticks) = at;
    let mut bytes = [0; 12];
    for (dst, word) in bytes.chunks_exact_mut(4).zip(wire) {
        dst.copy_from_slice(&word.to_le_bytes());
    }
    let o = Format2Input {
        payload: &bytes,
        data_length: 12,
        capture: ReferenceCapture {
            session,
            packet: id,
            channel: ch,
            rtc: Rtc::from_raw(ticks),
            byte_offset: Some(id * 100),
        },
        declared_edition: Some(edition),
        data_type_version: 10,
        declared_timein: declared,
    }
    .decode()
    .unwrap();
    ReferenceInput::format2(o, context, LeapSecondTable::builtin(), raw).unwrap()
}
fn rate_wire(ntf: u32, ns: u64) -> [u32; 3] {
    let (sec, nanos) = ((ns / 1_000_000_000) as u32, ns % 1_000_000_000);
    if ntf == 0 {
        [
            1,
            3_692_304_000 + sec,
            (nanos * (1u64 << 32)).div_ceil(1_000_000_000) as u32,
        ]
    } else {
        [(ntf << 4) | 1, 1_483_315_237 + sec, nanos as u32]
    }
}
fn f1(source: u8) -> ReferenceInput {
    f1_at(source, 0)
}
fn f1_at(source: u8, ms: u8) -> ReferenceInput {
    // DMY2017-01-02 midnight UTC, FMT4; SRC supplied literally in low nibble.
    ReferenceInput::format1(
        &[0x40 | source, 2, 0, 0, ms, 0, 0, 0, 2, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(LeapSecondTable::builtin())
    .unwrap()
}
fn push(
    s: &mut StreamTimeline,
    id: u64,
    ch: u16,
    ticks: u64,
    r: Option<ReferenceInput>,
) -> StreamBatch {
    s.observe(packet(id, ch, ticks, r), 0, 100, false).unwrap()
}
// Supply consistent capture/header identities; expected values stay literal.
fn network(
    s: &mut StreamTimeline,
    at: (usize, u64, u16, u64),
    wire: [u32; 3],
    raw: bool,
) -> StreamBatch {
    push(s, at.1, at.2, at.3, Some(net(at, wire, raw)))
}
fn code(b: &StreamBatch, id: u8) -> &TimelineFinding {
    b.findings.iter().find(|f| f.code == id).unwrap()
}
fn raw_field(f: &TimelineFinding, name: &str) -> findings::RawValue {
    f.evidence
        .raw_fields()
        .iter()
        .find(|field| field.name == name)
        .unwrap()
        .observed
}
fn ordinary(a: &StreamAnswer, sec: u8, ns: u32, scale: TimeScale) {
    let t = a.time.unwrap();
    assert_eq!(
        (
            t.year(),
            t.day_of_year(),
            t.hours(),
            t.minutes(),
            t.seconds(),
            t.nanoseconds()
        ),
        (Some(2017), 2, 0, 0, sec, ns)
    );
    assert_eq!(
        a.basis.preceding.as_ref().unwrap().input.scale.effective(),
        scale
    );
}
// Clause evidence: STRM-201; F2COR-101; ANS-101. No complete-statement claim.
#[test]
fn standard_native_scales_and_interpolation_use_physical_coordinates() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        network(&mut s, (1, 1, 1, 0), [1, 3_692_304_000, 0], false);
        let first = push(&mut s, 2, 9, 5_000_000, None);
        let next = network(
            &mut s,
            (1, 3, 1, 10_000_000),
            [0x21, 1_483_315_238, 10_000],
            false,
        );
        let answer = if mode == StreamMode::Standard {
            assert!(next.answers.is_empty());
            first.answers[0].clone()
        } else {
            assert!(first.answers.is_empty());
            next.answers[0].clone()
        };
        ordinary(
            &answer,
            0,
            if mode == StreamMode::Standard {
                500_000_000
            } else {
                500_005_000
            },
            TimeScale::Utc,
        );
        assert_eq!(
            answer.basis.drift_correction_ns,
            if mode == StreamMode::Standard {
                None
            } else {
                Some(5000)
            }
        );
        assert_eq!(
            answer
                .basis
                .preceding
                .as_ref()
                .unwrap()
                .input
                .network
                .as_ref()
                .unwrap()
                .reference()
                .observation()
                .capture()
                .channel,
            1
        );
        if mode == StreamMode::Standard {
            assert_eq!(answer.basis.release, ReleaseReason::Standard);
        }
        if mode == StreamMode::Interpolate {
            assert_eq!(answer.basis.following.as_ref().unwrap().key.packet, 3);
            assert_eq!(answer.basis.release, ReleaseReason::Interpolated);
        }
        let last = push(&mut s, 4, 9, 15_000_000, None);
        let last = if mode == StreamMode::Standard {
            last
        } else {
            s.finish(0).unwrap()
        };
        ordinary(&last.answers[0], 38, 500_010_000, TimeScale::Tai);
    }
}
// Clause evidence: COR-103/107 and STRM-101/206 Option A; offline ranking unclaimed.
#[test]
fn commitment_ignores_counts_other_channels_and_unreported_network_source() {
    let inventory = [
        StreamChannel {
            id: 0,
            timein: false,
            external: true,
        },
        StreamChannel {
            id: 1,
            timein: true,
            external: false,
        },
        StreamChannel {
            id: 2,
            timein: true,
            external: false,
        },
        StreamChannel {
            id: 7,
            timein: true,
            external: true,
        },
        StreamChannel {
            id: 8,
            timein: true,
            external: true,
        },
    ];
    for (fixed, inventory, expected) in [
        (
            Some(1),
            Some(&inventory[..]),
            Some((1, StreamChannelChoice::Caller)),
        ),
        (
            None,
            Some(&inventory[..]),
            Some((7, StreamChannelChoice::DeclaredExternal)),
        ),
        (
            None,
            Some(&inventory[..3]),
            Some((1, StreamChannelChoice::LowestId)),
        ),
        (None, None, None),
    ] {
        let mut p = policy(StreamMode::Standard);
        p.offline.channel = fixed;
        let mut s = StreamTimeline::new(p, inventory).unwrap();
        assert_eq!(s.channel(), expected);
        // Channel1's bad absolute epoch and repeated arrivals cannot displace channel7.
        network(&mut s, (1, 1, 1, 0), [0x21, 1_483_315_247, 0], false);
        network(&mut s, (1, 2, 7, 0), [1, 3_692_304_000, 0], false);
        let b = push(&mut s, 3, 9, 2_000_000, None);
        let frozen = b.answers[0].clone();
        if let Some((ch, _)) = expected {
            assert_eq!(frozen.basis.preceding.as_ref().unwrap().channel, ch);
            if ch == 7 {
                ordinary(&frozen, 0, 200_000_000, TimeScale::Utc)
            } else {
                ordinary(&frozen, 47, 200_000_000, TimeScale::Tai)
            }
        } else {
            assert!(frozen.time.is_none());
            s.choose_channel(7, 2, 0).unwrap();
        }
        network(
            &mut s,
            (1, 4, 1, 3_000_000),
            [0x21, 1_483_315_248, 0],
            false,
        );
        assert_eq!(s.channel().unwrap().0, expected.map_or(7, |x| x.0));
        assert_eq!(b.answers[0], frozen);
        if expected.is_none() {
            network(&mut s, (1, 5, 7, 4_000_000), [1, 3_692_304_000, 0], false);
            let after = push(&mut s, 6, 9, 6_000_000, None);
            ordinary(&after.answers[0], 0, 200_000_000, TimeScale::Utc);
            assert_eq!(
                after.answers[0].basis.preceding.as_ref().unwrap().channel,
                7
            );
            assert!(b.answers[0].time.is_none());
        }
    }
    let mut s = StreamTimeline::new(policy(StreamMode::Standard), None).unwrap();
    push(&mut s, 1, 1, 0, Some(f1(1)));
    network(&mut s, (1, 2, 1, 1_000_000), [1, 3_692_304_000, 0], false);
    let b = push(&mut s, 3, 1, 2_000_000, Some(f1(0)));
    code(&b, 11);
    assert_eq!(s.channel(), Some((1, StreamChannelChoice::Caller)));
}
/// Requirements: L1-STRM-001, L2-STRM-204, L3-STRM-204
#[test]
fn no_time_before_usable_preceding_source_and_no_retrospective_answers() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        let early = push(&mut s, 1, 9, 0, None);
        code(&early, 2);
        assert!(early.answers[0].time.is_none());
        network(&mut s, (1, 2, 1, 0), [0x20, 1_483_315_237, 0], false);
        assert!(
            push(&mut s, 3, 9, 1_000_000, None).answers[0]
                .time
                .is_none()
        );
        push(&mut s, 4, 1, 2_000_000, Some(f1(15)));
        assert!(
            push(&mut s, 5, 9, 3_000_000, None).answers[0]
                .time
                .is_none()
        );
        network(&mut s, (1, 6, 1, 4_000_000), [1, 3_692_304_000, 0], false);
        let data = push(&mut s, 7, 9, 6_000_000, None);
        for (id, ticks, wire, id_expected) in [
            (8, 7_000_000, [1, 0, 0], 29),
            (9, 8_000_000, [0x31, 1, 0], 6),
            (10, 9_000_000, [0x21, 1, 1_000_000_000], 4),
            (11, 10_000_000, [0x20, 1_483_315_238, 0], 3),
        ] {
            let b = network(&mut s, (1, id, 1, ticks), wire, false);
            code(&b, id_expected);
            assert!(b.answers.is_empty());
        }
        let none_source = push(&mut s, 12, 1, 11_000_000, Some(f1(15)));
        code(&none_source, 3);
        assert!(none_source.answers.is_empty());
        let mut missing = packet(13, 1, 12_000_000, None);
        missing.kind = PacketKind::Time;
        let missing = s.observe(missing, 0, 100, false).unwrap();
        code(&missing, 4);
        assert!(missing.answers.is_empty());
        let none_fmt = ReferenceInput::format1(
            &[0xf1, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20],
            12,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap();
        let none_fmt = push(&mut s, 14, 1, 13_000_000, Some(none_fmt));
        code(&none_fmt, 3);
        assert!(none_fmt.answers.is_empty());
        let mut far = NtpContext::new(1);
        far.declared = Some(EraDeclaration {
            era: 100,
            id: "outside-calendar".into(),
        });
        let unsupported = contextual_net(
            (1, 15, 1, 14_000_000),
            [1, 63_104, 0],
            false,
            far,
            true,
            edition::PacketEdition::V24R1,
        );
        assert!(matches!(unsupported.value, ReferenceValue::NeedsContext));
        let b = push(&mut s, 15, 1, 14_000_000, Some(unsupported));
        assert!(b.answers.is_empty());
        let last = push(&mut s, 16, 9, 15_000_000, None);
        if mode == StreamMode::Standard {
            ordinary(&last.answers[0], 1, 100_000_000, TimeScale::Utc);
        }
        let after = if mode == StreamMode::Standard {
            data
        } else {
            s.finish(0).unwrap()
        };
        ordinary(&after.answers[0], 0, 200_000_000, TimeScale::Utc);
        assert_eq!(
            after.answers[0]
                .basis
                .preceding
                .as_ref()
                .unwrap()
                .key
                .packet,
            6
        );
        assert!(early.answers[0].time.is_none());
        let mut empty = StreamTimeline::new(policy(mode), None).unwrap();
        let b = push(&mut empty, 1, 9, 0, None);
        assert!(b.answers[0].time.is_none());
        code(&empty.finish(0).unwrap(), 1);
        let mut late = StreamTimeline::new(policy(mode), None).unwrap();
        network(
            &mut late,
            (1, 1, 1, 10_000_000),
            [1, 3_692_304_001, 0],
            false,
        );
        let old = push(&mut late, 2, 9, 5_000_000, None);
        assert!(old.answers[0].time.is_none());
        network(&mut late, (1, 3, 1, 0), [1, 3_692_304_000, 0], false);
        let available = push(&mut late, 4, 9, 5_000_000, None);
        ordinary(&available.answers[0], 0, 500_000_000, TimeScale::Utc);
        assert!(old.answers[0].time.is_none());
    }
}
/// Requirements: L1-COR-008, L2-COR-108, L3-COR-108
#[test]
fn reference_gap_tolerance_and_raw_frame_exception_keep_policy_findings() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        for raw_policy in [false, true] {
            for first_ntf in 0..3 {
                for last_ntf in 0..3 {
                    for (a_raw, b_raw) in
                        [(false, false), (true, true), (false, true), (true, false)]
                    {
                        for tolerance in [0, 100_000_000] {
                            for elapsed in [
                                1_000_000_000u64,
                                1_000_000_100,
                                1_050_000_000,
                                1_100_000_000,
                            ] {
                                let mut p = policy(mode);
                                p.raw_network_rate = raw_policy;
                                p.offline.gap_tolerance_ns = tolerance;
                                p.offline.finding_levels.insert(16, Severity::Information);
                                let mut s = StreamTimeline::new(p, None).unwrap();
                                network(&mut s, (1, 1, 1, 0), rate_wire(first_ntf, 0), a_raw);
                                let b = network(
                                    &mut s,
                                    (1, 2, 1, elapsed / 100),
                                    rate_wire(last_ntf, elapsed),
                                    b_raw,
                                );
                                let gap = b.findings.iter().find(|f| f.code == 16);
                                assert_eq!(gap.is_some(), elapsed > 1_000_000_000 + tolerance);
                                if let Some(f) = gap {
                                    assert_eq!(f.severity, Severity::Information);
                                    assert_eq!(f.delta_ns, Some(elapsed.into()));
                                    assert_eq!(f.related, Some(1));
                                    assert_eq!(
                                        raw_field(f, "standard_minimum_1hz_applies"),
                                        findings::RawValue::Unsigned(u128::from(
                                            !(raw_policy || (a_raw && b_raw))
                                        ))
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // A stream-wide raw hint cannot exempt a Format1 endpoint.
    let mut p = policy(StreamMode::Standard);
    p.raw_network_rate = true;
    let mut mixed = StreamTimeline::new(p, None).unwrap();
    push(&mut mixed, 1, 1, 0, Some(f1(1)));
    let b = network(
        &mut mixed,
        (1, 2, 1, 20_000_000),
        [0x21, 1_483_315_239, 0],
        false,
    );
    let f = code(&b, 16);
    assert_eq!(
        raw_field(f, "standard_minimum_1hz_applies"),
        findings::RawValue::Unsigned(1)
    );
    // A network channel not committed for answers contributes no chosen-channel gap.
    let mut s = StreamTimeline::new(policy(StreamMode::Standard), None).unwrap();
    network(&mut s, (1, 1, 7, 0), [1, 3_692_304_000, 0], false);
    let b = network(&mut s, (1, 2, 7, 20_000_000), [1, 3_692_304_002, 0], false);
    assert!(!b.findings.iter().any(|f| f.code == 16));
}
// Clause evidence: STRM-202/205: compatibility and jump barriers in mixed inputs.
#[test]
fn incompatible_and_jumping_endpoints_drain_without_crossing_the_barrier() {
    for jump in [false, true] {
        let mut s = StreamTimeline::new(policy(StreamMode::Interpolate), None).unwrap();
        network(&mut s, (1, 1, 1, 0), [1, 3_692_304_000, 0], false);
        assert!(push(&mut s, 2, 9, 5_000_000, None).answers.is_empty());
        let r = if jump {
            net((1, 3, 1, 10_000_000), [0x21, 1_483_315_300, 0], false)
        } else {
            // FMT3 RTC has no stated scale (RDG-009), unlike FMT4 UTC.
            ReferenceInput::format1(
                &[0x31, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20],
                12,
                TimeF1Layout::SourceWithIts17,
                ScalePolicy::default(),
            )
            .unwrap()
        };
        let b = push(&mut s, 3, 1, 10_000_000, Some(r));
        assert_eq!(
            b.answers[0].basis.release,
            if jump {
                ReleaseReason::Discontinuity
            } else {
                ReleaseReason::IncompatibleScale
            }
        );
        ordinary(&b.answers[0], 0, 500_000_000, TimeScale::Utc);
        assert!(b.answers[0].basis.following.is_none());
        assert!(b.answers[0].basis.extrapolated);
        if jump {
            assert_eq!(code(&b, 17).delta_ns, Some(62_000_000_000));
        }
        let saved = b.answers[0].clone();
        let late = push(&mut s, 4, 9, 6_000_000, None);
        assert!(late.answers[0].time.is_none());
        assert_eq!(late.answers[0].basis.release, ReleaseReason::NoReference);
        let _ = s.finish(0).unwrap();
        assert_eq!(b.answers[0], saved);
    }
}
// Clause evidence: STRM-203/205/206: network fallbacks retain old basis and caps.
#[test]
fn every_network_resource_and_lifecycle_fallback_keeps_its_reason() {
    for reason in [
        ReleaseReason::Timeout,
        ReleaseReason::Eof,
        ReleaseReason::PacketCap,
        ReleaseReason::ByteCap,
        ReleaseReason::ReferenceCap,
        ReleaseReason::ModeChange,
        ReleaseReason::SessionBoundary,
    ] {
        let mut p = policy(StreamMode::Interpolate);
        p.caps = BufferCaps {
            packets: 1,
            bytes: 100,
            references: 2,
        };
        if reason == ReleaseReason::ByteCap {
            p.caps.packets = 2;
        }
        let mut s = StreamTimeline::new(p.clone(), None).unwrap();
        network(&mut s, (1, 1, 1, 0), [1, 3_692_304_000, 0], false);
        if reason == ReleaseReason::ReferenceCap {
            network(
                &mut s,
                (1, 2, 1, 2_000_000),
                [0x21, 1_483_315_237, 200_000_000],
                false,
            );
        }
        assert!(push(&mut s, 3, 9, 5_000_000, None).answers.is_empty());
        let b = match reason {
            ReleaseReason::Timeout => {
                assert!(s.advance(1_100_000_000).unwrap().answers.is_empty());
                s.advance(1_100_000_001).unwrap()
            }
            ReleaseReason::Eof => s.finish(0).unwrap(),
            ReleaseReason::PacketCap => push(&mut s, 4, 9, 6_000_000, None),
            ReleaseReason::ByteCap => s
                .observe(packet(4, 9, 6_000_000, None), 0, 101, false)
                .unwrap(),
            ReleaseReason::ReferenceCap => network(
                &mut s,
                (1, 4, 1, 3_000_000),
                [0x21, 1_483_315_237, 300_000_000],
                false,
            ),
            ReleaseReason::ModeChange => {
                let mut new = p.clone();
                new.id = 2;
                new.mode = StreamMode::Standard;
                s.set_policy(new, 0).unwrap()
            }
            _ => s
                .observe(packet(4, 9, 6_000_000, None), 0, 100, true)
                .unwrap(),
        };
        assert_eq!(b.answers[0].basis.release, reason);
        assert_eq!(b.answers[0].basis.policy.id, 1);
        assert!(b.answers[0].basis.extrapolated);
        assert!(b.answers[0].basis.following.is_none());
        let expected = if reason == ReleaseReason::ReferenceCap {
            TimeScale::Tai
        } else {
            TimeScale::Utc
        };
        ordinary(
            &b.answers[0],
            if expected == TimeScale::Tai { 37 } else { 0 },
            500_000_000,
            expected,
        );
        assert!(s.usage().packets <= 1 && s.usage().bytes <= 100 && s.usage().references <= 2);
    }
}

// Clause evidence: COR-107, STRM-204/205. Last reported source survives NTF metadata absence.
#[test]
fn loss_of_lock_never_switches_channel_and_only_opt_in_stops_time() {
    for stop in [false, true] {
        let mut p = policy(StreamMode::Interpolate);
        p.offline.stop_on_loss = stop;
        let mut s = StreamTimeline::new(p, None).unwrap();
        push(&mut s, 1, 1, 0, Some(f1(1)));
        network(
            &mut s,
            (1, 2, 1, 1_000_000),
            [1, 3_692_304_000, 429_496_730],
            false,
        );
        assert!(push(&mut s, 3, 9, 1_500_000, None).answers.is_empty());
        let b = push(&mut s, 4, 1, 2_000_000, Some(f1_at(0, 0x20)));
        code(&b, 11);
        ordinary(&b.answers[0], 0, 150_000_000, TimeScale::Utc);
        assert_eq!(
            b.answers[0].basis.release,
            if stop {
                ReleaseReason::UnusableReference
            } else {
                ReleaseReason::Interpolated
            }
        );
        assert_eq!(s.channel(), Some((1, StreamChannelChoice::Caller)));
        assert_eq!(b.answers[0].basis.preceding.as_ref().unwrap().key.packet, 2);
    }
}
// Clause evidence: STRM-202 and ANS-101 native qualified leap calendar; not all answer adapters.
/// Interpretations: RDG-010
#[test]
fn normalized_stream_answers_keep_a_derived_utc_second_sixty() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let r = ReferenceInput::format1(
            &[0x41, 3, 0, 0, 0, 0x59, 0x59, 0x23, 0x31, 0x12, 0x16, 0x20],
            12,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap()
        .normalize_to_tai(LeapSecondTable::builtin())
        .unwrap();
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        push(&mut s, 1, 1, 0, Some(r));
        let data = push(&mut s, 2, 9, 10_000_000, None);
        // UTC2016-12-31 23:59:59 is TAI B+35; midnight is B+37, B=1483228800.
        let b = network(
            &mut s,
            (1, 3, 1, 20_000_000),
            [0x21, 1_483_228_837, 0],
            false,
        );
        assert!(!b.findings.iter().any(|f| f.code == 17));
        let a = if mode == StreamMode::Standard {
            &data.answers[0]
        } else {
            &b.answers[0]
        };
        assert!(a.time.is_none());
        let c = a.reading.unwrap();
        assert_eq!(
            (c.year, c.day_of_year, c.hours, c.minutes, c.seconds),
            (Some(2016), 366, 23, 59, 60)
        );
        assert!(a.basis.unresolved.is_none());
        assert_eq!(c.scale.effective(), TimeScale::Utc);
    }
    // EOF fallback must remain value-kept/extrapolated even when legacy time
    // cannot represent the qualified leap calendar.
    let r = ReferenceInput::format1(
        &[0x41, 3, 0, 0, 0, 0x59, 0x59, 0x23, 0x31, 0x12, 0x16, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(LeapSecondTable::builtin())
    .unwrap();
    let mut s = StreamTimeline::new(policy(StreamMode::Interpolate), None).unwrap();
    push(&mut s, 1, 1, 0, Some(r));
    push(&mut s, 2, 9, 10_000_000, None);
    let end = s.finish(0).unwrap();
    let a = &end.answers[0];
    assert!(a.time.is_none());
    assert_eq!(a.reading.unwrap().seconds, 60);
    assert!(a.basis.extrapolated);
    assert_eq!(a.basis.release, ReleaseReason::Eof);
}
// Clause evidence: F2COR-102, STRM-202/205: ambiguity is not a unique bracket.
/// Interpretations: RDG-012
#[test]
fn ambiguity_keeps_capture_value_but_blocks_displacement_and_interpolation() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        let mut p = policy(mode);
        p.offline.finding_levels.insert(30, Severity::Information);
        let mut s = StreamTimeline::new(p, None).unwrap();
        let b = network(&mut s, (1, 1, 1, 0), [1, 3_692_217_600, 0x80000000], false);
        let f = code(&b, 30);
        assert_eq!(f.severity, Severity::Information);
        assert_eq!(f.evidence.validity(), findings::ValueValidity::Kept);
        let exact = push(&mut s, 2, 9, 0, None);
        assert_eq!(exact.answers[0].basis.release, ReleaseReason::Standard);
        assert_eq!(exact.answers[0].reading.unwrap().nanoseconds, 500_000_000);
        let data = push(&mut s, 3, 9, 1_000_000, None);
        let b = network(
            &mut s,
            (1, 4, 1, 10_000_000),
            [1, 3_692_217_601, 0x80000000],
            false,
        );
        let a = if mode == StreamMode::Standard {
            &data.answers[0]
        } else {
            assert_eq!(b.answers[0].basis.release, ReleaseReason::IncompatibleScale);
            &b.answers[0]
        };
        assert!(a.reading.is_none());
        assert!(a.basis.unresolved.is_some());
        assert!(!b.findings.iter().any(|f| f.code == 17));
        assert!(
            matches!(exact.answers[0].basis.preceding.as_ref().unwrap().input.network.as_ref().unwrap().reference().conversion(),NetworkConversion::Ntp(c) if c.basis().leap==NtpLeapStatus::Ambiguous)
        );
    }
}
// Clause evidence: STRM-101/205 and F2COR-101: validate before draining/consuming identities.
#[test]
fn capture_errors_are_atomic_and_new_sessions_keep_actual_finding_identity() {
    for wrong_session in [false, true] {
        let mut s = StreamTimeline::new(policy(StreamMode::Interpolate), None).unwrap();
        network(&mut s, (1, 1, 1, 0), [1, 3_692_304_000, 0], false);
        push(&mut s, 2, 9, 5_000_000, None);
        let r = net(
            (if wrong_session { 2 } else { 1 }, 3, 1, 10_000_000),
            [0x21, 1_483_315_238, 10_000],
            false,
        );
        let bad = packet(3, if wrong_session { 1 } else { 7 }, 10_000_000, Some(r));
        assert!(s.observe(bad, 2_000_000_000, 100, false).is_err());
        assert_eq!(s.usage().packets, 1);
        let b = network(
            &mut s,
            (1, 3, 1, 10_000_000),
            [0x21, 1_483_315_238, 10_000],
            false,
        );
        ordinary(&b.answers[0], 0, 500_005_000, TimeScale::Utc);
        assert_eq!(b.answers[0].basis.progress_ns, 0);
    }
    let mut p = policy(StreamMode::Interpolate);
    p.offline.finding_levels.insert(21, Severity::Information);
    let mut s = StreamTimeline::new(p, None).unwrap();
    network(&mut s, (1, 1, 1, 50_000_000), [1, 3_692_304_000, 0], false);
    push(&mut s, 2, 9, 55_000_000, None);
    let mut new = packet(
        3,
        1,
        123,
        Some(net((2, 3, 1, 123), [0x21, 2_000_000_037, 0], false)),
    );
    new.flags = 0x20;
    let b = s.observe(new, 0, 100, true).unwrap();
    assert_eq!(b.answers[0].basis.key.session, 1);
    assert!(!b.findings.iter().any(|f| f.code == 17));
    assert_eq!(b.answers[0].basis.release, ReleaseReason::SessionBoundary);
    for id in [21, 24] {
        let f = code(&b, id);
        assert_eq!(f.session, 2);
        assert_eq!(f.evidence.location().session, Some(2));
        assert_eq!(f.evidence.location().byte_offset, Some(300));
        assert_eq!(f.raw_rtc, Some(Rtc::from_raw(123)));
        assert_eq!(f.evidence.raw_rtc(), Some(Rtc::from_raw(123)));
        assert_eq!(f.evidence.location().packet, Some(3));
        assert_eq!(f.evidence.location().channel, Some(1));
        assert_eq!(
            f.severity,
            if id == 21 {
                Severity::Information
            } else {
                Severity::Warning
            }
        );
    }
    let b = push(&mut s, 4, 1, 1_000_000, Some(f1(3)));
    let f = code(&b, 6);
    assert_eq!(f.session, 2);
    assert_eq!(f.evidence.location().session, Some(2));
    push(&mut s, 5, 9, 20_000_000, None);
    let reset = network(&mut s, (3, 6, 1, 0), [0x21, 1_483_315_237, 0], false);
    for id in [18, 19] {
        let f = code(&reset, id);
        assert_eq!(f.session, 3);
        assert_eq!(f.delta_ns, Some(-2_000_000_000));
    }
    assert!(!reset.findings.iter().any(|f| f.code == 17));
    assert_eq!(reset.answers[0].basis.key.session, 2);
}
// Clause evidence: POL-102 effective point, COR-108 exception provenance; not whole policy/schema.
#[test]
fn raw_rate_policy_is_bound_to_observed_endpoints_not_retroactive() {
    let mut s = StreamTimeline::new(policy(StreamMode::Interpolate), None).unwrap();
    network(&mut s, (1, 1, 1, 0), [1, 3_692_304_000, 0], false);
    push(&mut s, 2, 9, 5_000_000, None);
    let mut p = s.policy().clone();
    p.id = 2;
    p.raw_network_rate = true;
    p.mode = StreamMode::Standard;
    let old = s.set_policy(p, 0).unwrap();
    assert_eq!(old.answers[0].basis.release, ReleaseReason::ModeChange);
    assert!(!old.answers[0].basis.policy.raw_network_rate);
    assert_eq!(
        old.answers[0]
            .basis
            .preceding
            .as_ref()
            .unwrap()
            .input
            .network
            .as_ref()
            .unwrap()
            .rate_policy(),
        Some((1, false))
    );
    for (id, ticks, seconds, standard_applies) in [
        (3, 20_000_000, 1_483_315_239, 1),
        (4, 40_000_000, 1_483_315_241, 0),
    ] {
        let b = network(&mut s, (1, id, 1, ticks), [0x21, seconds, 0], false);
        let f = code(&b, 16);
        for (name, expected) in [
            (
                "preceding_raw_network_rate_policy_id",
                if id == 3 { 1 } else { 2 },
            ),
            ("following_raw_network_rate_policy_id", 2),
            ("preceding_raw_network_rate_policy", u128::from(id != 3)),
            ("following_raw_network_rate_policy", 1),
            ("preceding_ntf", if id == 3 { 0 } else { 2 }),
            ("following_ntf", 2),
        ] {
            assert_eq!(raw_field(f, name), findings::RawValue::Unsigned(expected));
        }
        assert_eq!(f.severity, Severity::Warning);
        assert_eq!(
            raw_field(f, "standard_minimum_1hz_applies"),
            findings::RawValue::Unsigned(standard_applies)
        );
    }
    let b = push(&mut s, 5, 9, 41_000_000, None);
    let r = b.answers[0].basis.preceding.as_ref().unwrap();
    assert_eq!(
        r.input.network.as_ref().unwrap().rate_policy(),
        Some((2, true))
    );
    assert!(!old.answers[0].basis.policy.raw_network_rate);
}

// Clause evidence: F2COR-102/STRM-202 native GPS is never relabelled UTC.
/// Interpretations: RDG-009
#[test]
fn normalized_gps_to_ptp_bracket_keeps_the_native_gps_scale() {
    // GPS2017-01-02 00:00:18 +19 = TAI00:00:37; RDG-009.
    let gps = ReferenceInput::format1(
        &[0x51, 2, 0, 0, 0, 0x18, 0, 0, 2, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(LeapSecondTable::builtin())
    .unwrap();
    let mut s = StreamTimeline::new(policy(StreamMode::Interpolate), None).unwrap();
    push(&mut s, 1, 1, 0, Some(gps));
    push(&mut s, 2, 9, 5_000_000, None);
    let b = network(
        &mut s,
        (1, 3, 1, 10_000_000),
        [0x21, 1_483_315_238, 10_000],
        false,
    );
    ordinary(&b.answers[0], 18, 500_005_000, TimeScale::Gps);
    assert_eq!(
        b.answers[0].reading.unwrap().scale,
        ScaleEvidence::standard(TimeScale::Gps)
    );
    assert_eq!(
        b.answers[0]
            .basis
            .preceding
            .as_ref()
            .unwrap()
            .input
            .normalization
            .as_ref()
            .unwrap()
            .applied_seconds(),
        19
    );
}

// Clause evidence: COR-106 / F2COR-101 actual engine admission/levels, not full declaration-range work.
#[test]
fn engine_admission_overrides_and_invalid_labels_survive_stream_answers() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        for status in [0, 1, 2] {
            for declared in [false, true] {
                for mask in 0..4 {
                    let mut p = policy(mode);
                    p.offline.include_invalid = mask & 1 != 0;
                    p.offline.include_undeclared = mask & 2 != 0;
                    for id in [3, 5, 6, 8, 26] {
                        p.offline.finding_levels.insert(id, Severity::Information);
                    }
                    let eligible = (status == 1 || p.offline.include_invalid)
                        && (declared || p.offline.include_undeclared);
                    let r = contextual_net(
                        (1, 1, 1, 0),
                        [0x80000020 | status, 1_483_315_237, 0],
                        false,
                        NtpContext::new(1),
                        declared,
                        if mask == 0 {
                            edition::PacketEdition::V15
                        } else {
                            edition::PacketEdition::V24R1
                        },
                    );
                    let mut packet = packet(1, 1, 0, Some(r));
                    packet.declared_timein = declared;
                    let mut s = StreamTimeline::new(p.clone(), None).unwrap();
                    let b = s.observe(packet, 0, 100, false).unwrap();
                    if mask == 0 {
                        assert_eq!(code(&b, 26).severity, Severity::Information);
                    }
                    let reserved = code(&b, 5);
                    assert_eq!(reserved.severity, Severity::Information);
                    assert_eq!(
                        raw_field(reserved, "reserved_csdw_bits"),
                        findings::RawValue::Unsigned(0x80000000)
                    );
                    if status != 1 {
                        assert_eq!(
                            code(&b, if status == 0 { 3 } else { 6 }).severity,
                            Severity::Information
                        );
                    }
                    if !declared {
                        assert_eq!(code(&b, 8).severity, Severity::Information);
                    }
                    let answer = push(&mut s, 2, 9, 1_000_000, None);
                    let answer = if eligible && status == 1 && mode == StreamMode::Interpolate {
                        assert!(answer.answers.is_empty());
                        s.finish(0).unwrap()
                    } else {
                        answer
                    };
                    let a = &answer.answers[0];
                    if eligible && status != 1 && mode == StreamMode::Interpolate {
                        assert_eq!(a.basis.release, ReleaseReason::UnusableReference);
                    }
                    assert_eq!(a.time.is_some(), eligible);
                    if eligible {
                        ordinary(a, 37, 100_000_000, TimeScale::Tai);
                        let n = a
                            .basis
                            .preceding
                            .as_ref()
                            .unwrap()
                            .input
                            .network
                            .as_ref()
                            .unwrap();
                        assert_eq!(n.reference().admission().invalid, status != 1);
                        assert_eq!(n.reference().admission().undeclared, !declared);
                        assert_eq!(
                            n.reference().policy().include_invalid,
                            p.offline.include_invalid
                        );
                        assert_eq!(
                            n.reference().policy().include_undeclared,
                            p.offline.include_undeclared
                        );
                    }
                }
            }
        }
    }
}

// Clause evidence: STRM-205 late references must not reopen a closed clock segment.
#[test]
fn late_prejump_reference_cannot_replace_the_later_segment() {
    for prior_anchor in [false, true] {
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(policy(mode), None).unwrap();
            if prior_anchor {
                network(&mut s, (1, 1, 1, 0), [1, 3_692_304_000, 0], false);
            }
            // At RTC1s the clock has jumped: UTC10s = TAI47s. The residual is +9s.
            network(
                &mut s,
                (1, 2, 1, 10_000_000),
                [0x21, 1_483_315_247, 0],
                false,
            );
            let old = push(&mut s, 3, 9, 15_000_000, None);
            let frozen = old.answers.clone();
            // Late by1s, inside1100ms bound: old-domain UTC0.5s at RTC0.5s.
            let late = network(
                &mut s,
                (1, 4, 1, 5_000_000),
                [1, 3_692_304_000, 0x80000000],
                false,
            );
            if prior_anchor {
                assert!(late.answers.is_empty());
                assert!(!late.findings.iter().any(|f| f.code == 17));
            }
            let old_domain = push(&mut s, 5, 9, 7_500_000, None);
            assert!(old_domain.answers[0].reading.is_none());
            assert_eq!(
                old_domain.answers[0].basis.release,
                ReleaseReason::NoReference
            );
            let data = push(&mut s, 6, 9, 16_000_000, None);
            let data = if mode == StreamMode::Interpolate {
                s.finish(0).unwrap()
            } else {
                data
            };
            let a = data
                .answers
                .iter()
                .find(|a| a.basis.key.packet == 6)
                .unwrap();
            assert_eq!(a.basis.preceding.as_ref().unwrap().key.packet, 2);
            ordinary(a, 47, 600_000_000, TimeScale::Tai);
            if !prior_anchor {
                assert_eq!(code(&late, 17).related, Some(2));
            }
            assert_eq!(old.answers, frozen);
        }
    }
}
