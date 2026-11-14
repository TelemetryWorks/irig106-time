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
        declared_edition: Some(edition::PacketEdition::V24R1),
        data_type_version: 10,
        declared_timein: true,
    }
    .decode()
    .unwrap();
    ReferenceInput::format2(o, NtpContext::new(session), LeapSecondTable::builtin(), raw).unwrap()
}
fn f1(source: u8) -> ReferenceInput {
    // DMY2017-01-02 midnight UTC, FMT4; SRC supplied literally in low nibble.
    ReferenceInput::format1(
        &[0x40 | source, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20],
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
fn code(b: &StreamBatch, id: u8) -> &TimelineFinding {
    b.findings.iter().find(|f| f.code == id).unwrap()
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
        push(
            &mut s,
            1,
            1,
            0,
            Some(net((1, 1, 1, 0), [1, 3_692_304_000, 0], false)),
        );
        let first = push(&mut s, 2, 9, 5_000_000, None);
        let next = push(
            &mut s,
            3,
            1,
            10_000_000,
            Some(net(
                (1, 3, 1, 10_000_000),
                [0x21, 1_483_315_238, 10_000],
                false,
            )),
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
            id: 7,
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
            Some(&inventory[..2]),
            Some((1, StreamChannelChoice::LowestId)),
        ),
        (None, None, None),
    ] {
        let mut p = policy(StreamMode::Standard);
        p.offline.channel = fixed;
        let mut s = StreamTimeline::new(p, inventory).unwrap();
        assert_eq!(s.channel(), expected);
        // Channel1's bad absolute epoch and repeated arrivals cannot displace channel7.
        push(
            &mut s,
            1,
            1,
            0,
            Some(net((1, 1, 1, 0), [0x21, 1_500_000_000, 0], false)),
        );
        push(
            &mut s,
            2,
            7,
            0,
            Some(net((1, 2, 7, 0), [1, 3_692_304_000, 0], false)),
        );
        let b = push(&mut s, 3, 9, 2_000_000, None);
        let frozen = b.answers[0].clone();
        if let Some((ch, _)) = expected {
            assert_eq!(frozen.basis.preceding.as_ref().unwrap().channel, ch);
            if ch == 7 {
                ordinary(&frozen, 0, 200_000_000, TimeScale::Utc)
            }
        } else {
            assert!(frozen.time.is_none());
            s.choose_channel(7, 2, 0).unwrap();
        }
        push(
            &mut s,
            4,
            1,
            3_000_000,
            Some(net((1, 4, 1, 3_000_000), [0x21, 1_500_000_001, 0], false)),
        );
        assert_eq!(s.channel().unwrap().0, expected.map_or(7, |x| x.0));
        assert_eq!(b.answers[0], frozen);
    }
    let mut s = StreamTimeline::new(policy(StreamMode::Standard), None).unwrap();
    push(&mut s, 1, 1, 0, Some(f1(1)));
    push(
        &mut s,
        2,
        1,
        1_000_000,
        Some(net((1, 2, 1, 1_000_000), [1, 3_692_304_000, 0], false)),
    );
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
        push(
            &mut s,
            2,
            1,
            0,
            Some(net((1, 2, 1, 0), [0x20, 1_483_315_237, 0], false)),
        );
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
        push(
            &mut s,
            6,
            1,
            4_000_000,
            Some(net((1, 6, 1, 4_000_000), [1, 3_692_304_000, 0], false)),
        );
        let data = push(&mut s, 7, 9, 6_000_000, None);
        for (id, ticks, wire, id_expected) in [
            (8, 7_000_000, [1, 0, 0], 29),
            (9, 8_000_000, [0x31, 1, 0], 6),
            (10, 9_000_000, [0x21, 1, 1_000_000_000], 4),
            (11, 10_000_000, [0x20, 1_483_315_238, 0], 3),
        ] {
            let b = push(
                &mut s,
                id,
                1,
                ticks,
                Some(net((1, id, 1, ticks), wire, false)),
            );
            code(&b, id_expected);
            assert!(b.answers.is_empty());
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
    }
}
/// Requirements: L1-COR-008, L2-COR-108, L3-COR-108
#[test]
fn reference_gap_tolerance_and_raw_frame_exception_keep_policy_findings() {
    for mode in [StreamMode::Standard, StreamMode::Interpolate] {
        for raw_policy in [false, true] {
            for (a_raw, b_raw) in [(false, false), (true, true), (false, true), (true, false)] {
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
                        push(
                            &mut s,
                            1,
                            1,
                            0,
                            Some(net((1, 1, 1, 0), [0x21, 1_483_315_237, 0], a_raw)),
                        );
                        let b = push(
                            &mut s,
                            2,
                            1,
                            elapsed / 100,
                            Some(net(
                                (1, 2, 1, elapsed / 100),
                                [
                                    0x21,
                                    1_483_315_237 + (elapsed / 1_000_000_000) as u32,
                                    (elapsed % 1_000_000_000) as u32,
                                ],
                                b_raw,
                            )),
                        );
                        let gap = b.findings.iter().find(|f| f.code == 16);
                        assert_eq!(gap.is_some(), elapsed > 1_000_000_000 + tolerance);
                        if let Some(f) = gap {
                            assert_eq!(f.severity, Severity::Information);
                            assert_eq!(f.delta_ns, Some(elapsed.into()));
                            assert_eq!(f.related, Some(1));
                            assert_eq!(
                                f.evidence
                                    .raw_fields()
                                    .iter()
                                    .find(|f| f.name == "standard_minimum_1hz_applies")
                                    .unwrap()
                                    .observed,
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
    // A network channel not committed for answers contributes no chosen-channel gap.
    let mut s = StreamTimeline::new(policy(StreamMode::Standard), None).unwrap();
    push(
        &mut s,
        1,
        7,
        0,
        Some(net((1, 1, 7, 0), [1, 3_692_304_000, 0], false)),
    );
    let b = push(
        &mut s,
        2,
        7,
        20_000_000,
        Some(net((1, 2, 7, 20_000_000), [1, 3_692_304_002, 0], false)),
    );
    assert!(!b.findings.iter().any(|f| f.code == 16));
}
// Clause evidence: STRM-202/205: compatibility and jump barriers in mixed inputs.
#[test]
fn incompatible_and_jumping_endpoints_drain_without_crossing_the_barrier() {
    for jump in [false, true] {
        let mut s = StreamTimeline::new(policy(StreamMode::Interpolate), None).unwrap();
        push(
            &mut s,
            1,
            1,
            0,
            Some(net((1, 1, 1, 0), [1, 3_692_304_000, 0], false)),
        );
        assert!(push(&mut s, 2, 9, 5_000_000, None).answers.is_empty());
        let mut r = if jump {
            net((1, 3, 1, 10_000_000), [0x21, 1_483_315_300, 0], false)
        } else {
            f1(1)
        };
        if !jump {
            r.normalization = None;
            r.scale = ScaleEvidence::unknown();
        }
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
            assert_eq!(code(&b, 17).delta_ns, Some(63_000_000_000));
        }
        let saved = b.answers[0].clone();
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
        let mut s = StreamTimeline::new(p.clone(), None).unwrap();
        push(
            &mut s,
            1,
            1,
            0,
            Some(net((1, 1, 1, 0), [1, 3_692_304_000, 0], false)),
        );
        if reason == ReleaseReason::ReferenceCap {
            push(
                &mut s,
                2,
                1,
                2_000_000,
                Some(net(
                    (1, 2, 1, 2_000_000),
                    [0x21, 1_483_315_237, 200_000_000],
                    false,
                )),
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
            ReleaseReason::ReferenceCap => push(
                &mut s,
                4,
                1,
                3_000_000,
                Some(net(
                    (1, 4, 1, 3_000_000),
                    [0x21, 1_483_315_237, 300_000_000],
                    false,
                )),
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
