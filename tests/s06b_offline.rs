//! Literal Figs11-15..17 (106-24R1 printed11-26/27, PDF34/35).
//! Physical coordinates use S04 IERS offsets and RDG-012, never host time.
use irig106_time::{edition::PacketEdition, format2::*, leap::*, ntp::*, scale::*, timeline::*, *};
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
fn network(
    id: u64,
    ch: u16,
    ticks: u64,
    wire: [u32; 3],
    raw: bool,
    table: &LeapSecondTable,
) -> ReferenceInput {
    let [word, seconds, tail] = wire;
    let mut payload = [0; 12];
    payload[..4].copy_from_slice(&word.to_le_bytes());
    payload[4..8].copy_from_slice(&seconds.to_le_bytes());
    payload[8..].copy_from_slice(&tail.to_le_bytes());
    let o = Format2Input {
        payload: &payload,
        data_length: 12,
        capture: ReferenceCapture {
            session: 1,
            packet: id,
            channel: ch,
            rtc: Rtc::from_raw(ticks),
            byte_offset: Some(id * 100),
        },
        declared_edition: Some(PacketEdition::V24R1),
        data_type_version: 10,
        declared_timein: true,
    }
    .decode()
    .unwrap();
    ReferenceInput::format2(o, NtpContext::new(1), table.clone(), raw).unwrap()
}
fn utc(table: &LeapSecondTable) -> ReferenceInput {
    // Literal Figure11-14 DMY: 2017-01-02 00:00:00.00, FMT4 UTC / SRC1.
    ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(table.clone())
    .unwrap()
}
/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-COR-004, L2-COR-104, L3-COR-104
/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
/// Interpretations: RDG-009, RDG-012
#[test]
fn mixed_utc_ptp_and_f1_use_one_physical_scale_without_fake_fields() {
    let t = LeapSecondTable::builtin();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    // 2017-01-02 UTC1483315200; TAI=UTC+37. NTP=UTC+2208988800.
    b.observe(
        packet(
            1,
            4,
            0,
            Some(network(1, 4, 0, [1, 3_692_304_000, 0], false, &t)),
        ),
        false,
    )
    .unwrap();
    b.observe(
        packet(
            2,
            4,
            10_000_000,
            Some(network(
                2,
                4,
                10_000_000,
                [0x21, 1_483_315_238, 0],
                false,
                &t,
            )),
        ),
        false,
    )
    .unwrap();
    let f1 = utc(&t); // same physical start on another channel.
    b.observe(packet(3, 5, 0, Some(f1)), false).unwrap();
    let key = b.observe(packet(4, 4, 15_000_000, None), false).unwrap();
    let timeline = b.finish().unwrap();
    assert!(!timeline.findings().iter().any(|f| f.code == 17));
    let q = timeline.quality(1, 4).unwrap();
    assert_eq!(q.drift_ppm, Some(0.0));
    let answer = timeline.lookup(key).unwrap();
    let r = answer.basis.reference.unwrap();
    assert_eq!(r.channel, 4);
    assert_eq!(r.input.source, None);
    assert_eq!(r.input.format, None);
    assert!(r.input.its.is_none());
    assert_eq!(
        r.input
            .network
            .as_ref()
            .unwrap()
            .reference()
            .observation()
            .capture()
            .rtc,
        Rtc::from_raw(10_000_000)
    );
    let value = answer.time.unwrap();
    assert_eq!(
        (
            value.year(),
            value.day_of_year(),
            value.hours(),
            value.minutes(),
            value.seconds(),
            value.nanoseconds()
        ),
        (Some(2017), 2, 0, 0, 38, 500_000_000)
    );
    assert_eq!(answer.reading.unwrap().scale.effective(), TimeScale::Tai);
    // Same physical coordinate: Format1 UTC and NTP UTC exactly agree.
    let a = &timeline.sessions()[0].references;
    assert_eq!(
        a[0].input.normalization.as_ref().unwrap().tai_nanoseconds(),
        a[2].input.normalization.as_ref().unwrap().tai_nanoseconds()
    );
}
/// Requirements: L1-COR-008, L2-COR-108, L3-COR-108
/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
#[test]
fn raw_network_gap_is_a_policy_finding_not_a_standard_rate_violation() {
    let t = LeapSecondTable::builtin();
    for (first_raw, second_raw) in [(false, false), (true, true), (true, false), (false, true)] {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(
            packet(
                1,
                4,
                0,
                Some(network(1, 4, 0, [1, 3_692_304_000, 0], first_raw, &t)),
            ),
            false,
        )
        .unwrap();
        b.observe(
            packet(
                2,
                4,
                20_000_000,
                Some(network(
                    2,
                    4,
                    20_000_000,
                    [0x11, 1_483_315_239, 0],
                    second_raw,
                    &t,
                )),
            ),
            false,
        )
        .unwrap();
        let timeline = b.finish().unwrap();
        let f = timeline.findings().iter().find(|f| f.code == 16).unwrap();
        assert_eq!(f.delta_ns, Some(2_000_000_000));
        assert_eq!(f.related, Some(1));
        assert_eq!(
            f.evidence
                .raw_fields()
                .iter()
                .find(|f| f.name == "standard_minimum_1hz_applies")
                .unwrap()
                .observed,
            findings::RawValue::Unsigned(u128::from(!(first_raw && second_raw)))
        );
        assert!(
            timeline.sessions()[0].references[1]
                .input
                .network
                .as_ref()
                .unwrap()
                .raw_network_rate()
                == second_raw
        );
    }
}
/// Requirements: L1-COR-006, L2-COR-106, L3-COR-106
/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
#[test]
fn rejected_network_ts_keeps_previous_reference_and_engine_policy_wins() {
    let t = LeapSecondTable::builtin();
    for include in [false, true] {
        let p = OfflinePolicy {
            include_invalid: include,
            strategy: ReferenceStrategy::Preceding,
            ..Default::default()
        };
        let mut b = TimelineBuilder::new(p).unwrap();
        b.observe(
            packet(
                1,
                4,
                0,
                Some(network(1, 4, 0, [1, 3_692_304_000, 0], false, &t)),
            ),
            false,
        )
        .unwrap();
        b.observe(
            packet(
                2,
                4,
                5_000_000,
                Some(network(
                    2,
                    4,
                    5_000_000,
                    [0x20, 1_483_315_300, 0],
                    false,
                    &t,
                )),
            ),
            false,
        )
        .unwrap();
        let k = b.observe(packet(3, 4, 6_000_000, None), false).unwrap();
        let timeline = b.finish().unwrap();
        assert!(timeline.findings().iter().any(|f| f.code == 3));
        assert_eq!(
            timeline
                .lookup(k)
                .unwrap()
                .basis
                .reference
                .unwrap()
                .key
                .packet,
            if include { 2 } else { 1 }
        );
        assert_eq!(timeline.sessions()[0].references[1].admitted, include);
        let a = timeline.lookup(k).unwrap().reading.unwrap();
        assert_eq!(
            (a.hours, a.minutes, a.seconds, a.nanoseconds),
            if include {
                (0, 1, 40, 100_000_000)
            } else {
                (0, 0, 0, 600_000_000)
            }
        );
    }
}
/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
/// Interpretations: RDG-012
#[test]
fn ambiguous_ntp_does_not_establish_an_exact_physical_bracket() {
    let t = LeapSecondTable::builtin();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(
        packet(
            1,
            4,
            0,
            Some(network(1, 4, 0, [1, 3_692_217_600, 0x80000000], false, &t)),
        ),
        false,
    )
    .unwrap();
    let exact = b.observe(packet(2, 4, 0, None), false).unwrap();
    let later = b.observe(packet(3, 4, 1_000_000, None), false).unwrap();
    b.observe(
        packet(
            4,
            4,
            10_000_000,
            Some(network(
                4,
                4,
                10_000_000,
                [1, 3_692_217_601, 0x8000_0000],
                false,
                &t,
            )),
        ),
        false,
    )
    .unwrap();
    let timeline = b.finish().unwrap();
    assert!(timeline.lookup(exact).unwrap().reading.is_some());
    assert!(timeline.lookup(later).unwrap().reading.is_none());
    assert!(timeline.quality(1, 4).unwrap().drift_ppm.is_none());
    assert!(timeline.findings().iter().any(|f| f.code == 30));
    assert!(!timeline.findings().iter().any(|f| f.code == 17));
}
/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-TAI-004, L2-TAI-104, L3-TAI-104
/// Interpretations: RDG-012
#[test]
fn stale_ptp_basis_and_epoch_identity_survive_offline_admission() {
    let t = LeapSecondTable::builtin();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(
        packet(
            1,
            4,
            0,
            Some(network(1, 4, 0, [0x21, 2_000_000_037, 0], false, &t)),
        ),
        false,
    )
    .unwrap();
    let k = b.observe(packet(2, 4, 5_000_000, None), false).unwrap();
    let timeline = b.finish().unwrap();
    let f = timeline.findings().iter().find(|f| f.code == 21).unwrap();
    assert_eq!(f.evidence.location().byte_offset, Some(100));
    let a = timeline.lookup(k).unwrap();
    let net = a.basis.reference.unwrap().input.network.unwrap();
    assert!(
        matches!(net.reference().conversion(),NetworkConversion::Ptp(c) if c.status()==PtpConversionStatus::PossiblyOutOfDate && c.unix_seconds().unwrap().get()==2_000_000_000)
    );
    assert_eq!(a.reading.unwrap().scale.effective(), TimeScale::Tai);
}

/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
/// Interpretations: RDG-009
#[test]
fn gps_uses_inferred_nineteen_seconds_and_unknown_scale_blocks_normalization() {
    let table = LeapSecondTable::builtin();
    // 2017-01-02 GPS00:00:18 = UTC00:00:00 = TAI00:00:37.
    // Fig11-14 FMT5 / SRC1, DMY; 18s => word0=0x1800.
    let mut gps = ReferenceInput::format1(
        &[0x51, 2, 0, 0, 0, 0x18, 0, 0, 2, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(table.clone())
    .unwrap();
    assert_eq!(gps.normalization.as_ref().unwrap().applied_seconds(), 19);
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 4, 0, Some(gps.clone())), false)
        .unwrap();
    b.observe(
        packet(
            2,
            4,
            10_000_000,
            Some(network(
                2,
                4,
                10_000_000,
                [1, 3_692_304_001, 0],
                false,
                &table,
            )),
        ),
        false,
    )
    .unwrap();
    let k = b.observe(packet(3, 4, 5_000_000, None), false).unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.quality(1, 4).unwrap().drift_ppm, Some(0.0));
    assert!(!t.findings().iter().any(|f| f.code == 17));
    let a = t.lookup(k).unwrap();
    assert_eq!(a.reading.unwrap().scale.effective(), TimeScale::Gps);
    assert_eq!(
        (a.time.unwrap().seconds(), a.time.unwrap().nanoseconds()),
        (18, 500_000_000)
    );
    gps.normalization = None;
    gps.scale = ScaleEvidence::unknown();
    assert!(gps.clone().normalize_to_tai(table).is_err());
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 4, 0, Some(gps)), false).unwrap();
    b.observe(
        packet(2, 4, 10_000_000, Some(utc(&LeapSecondTable::builtin()))),
        false,
    )
    .unwrap();
    assert!(
        b.finish()
            .unwrap()
            .quality(1, 4)
            .unwrap()
            .drift_ppm
            .is_none()
    );
}
/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-COR-004, L2-COR-104, L3-COR-104
/// Interpretations: RDG-010
#[test]
fn physical_shift_across_insertion_retains_second_sixty_and_not_a_jump() {
    let table = LeapSecondTable::builtin();
    // Fig11-14: 2016-12-31 23:59:59; TAI=B+35, B=1483228800.
    // UTC2017 midnight is TAI=B+37: two physical seconds later.
    let f = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0x59, 0x59, 0x23, 0x31, 0x12, 0x16, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(table.clone())
    .unwrap();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 4, 0, Some(f)), false).unwrap();
    b.observe(
        packet(
            2,
            4,
            20_000_000,
            Some(network(
                2,
                4,
                20_000_000,
                [0x21, 1_483_228_837, 0],
                false,
                &table,
            )),
        ),
        false,
    )
    .unwrap();
    let k = b.observe(packet(3, 4, 10_000_000, None), false).unwrap();
    let t = b.finish().unwrap();
    assert!(!t.findings().iter().any(|f| f.code == 17));
    assert_eq!(t.quality(1, 4).unwrap().drift_ppm, Some(0.0));
    let a = t.lookup(k).unwrap();
    assert!(a.time.is_none());
    let r = a.reading.unwrap();
    assert_eq!(
        (r.year, r.day_of_year, r.hours, r.minutes, r.seconds),
        (Some(2016), 366, 23, 59, 60)
    );
    assert_eq!(a.basis.reference.unwrap().key.packet, 1);
}
/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
/// Requirements: L1-COR-006, L2-COR-106, L3-COR-106
#[test]
fn malformed_reserved_zero_and_capture_mismatches_are_not_admitted() {
    let table = LeapSecondTable::builtin();
    for (word, sec, tail, code) in [(0x21, 1, 1_000_000_000, 4), (0x31, 1, 0, 6), (1, 0, 0, 29)] {
        let mut b = TimelineBuilder::new(OfflinePolicy {
            include_invalid: true,
            ..Default::default()
        })
        .unwrap();
        b.observe(
            packet(
                1,
                4,
                0,
                Some(network(1, 4, 0, [word, sec, tail], false, &table)),
            ),
            false,
        )
        .unwrap();
        let t = b.finish().unwrap();
        assert!(!t.sessions()[0].references[0].admitted);
        assert!(t.findings().iter().any(|f| f.code == code));
    }
    let r = network(1, 4, 0, [1, 3_692_304_000, 0], false, &table);
    for field in 0..4 {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let mut p = packet(1, 4, 0, Some(r.clone()));
        match field {
            0 => p.id = 2,
            1 => p.channel = 5,
            2 => p.rtc = Rtc::from_raw(1),
            _ => p.declared_timein = false,
        };
        assert!(b.observe(p, false).is_err());
        assert!(b.observe(packet(1, 4, 0, Some(r.clone())), false).is_ok());
        assert!(
            b.observe(
                packet(
                    2,
                    4,
                    1,
                    Some(network(2, 4, 1, [1, 3_692_304_001, 0], false, &table))
                ),
                true
            )
            .is_err()
        );
    }
    let ambiguous = network(1, 4, 0, [1, 3_692_217_600, 0], false, &table);
    assert!(
        ambiguous
            .network
            .as_ref()
            .unwrap()
            .normalization_issue()
            .is_some()
    );
    assert!(ambiguous.normalize_to_tai(table).is_err());
}
/// Requirements: L1-TAI-004, L2-TAI-104, L3-TAI-104
/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
#[test]
fn stale_utc_normalization_labels_the_offset_and_engine_level() {
    let table = LeapSecondTable::builtin();
    let p = OfflinePolicy {
        finding_levels: [(21, Severity::Information)].into(),
        ..Default::default()
    };
    let mut b = TimelineBuilder::new(p).unwrap();
    // NTP4208988800-epoch2208988800=Unix2000000000; last known offset37.
    b.observe(
        packet(
            1,
            4,
            0,
            Some(network(1, 4, 0, [1, 4_208_988_800, 0], false, &table)),
        ),
        false,
    )
    .unwrap();
    let t = b.finish().unwrap();
    let n = t.sessions()[0].references[0]
        .input
        .normalization
        .as_ref()
        .unwrap();
    assert!(n.possibly_out_of_date());
    assert_eq!(n.applied_seconds(), 37);
    assert_eq!(n.tai_nanoseconds(), 2_000_000_037_000_000_000);
    let f = t.findings().iter().find(|f| f.code == 21).unwrap();
    assert_eq!(f.severity, Severity::Information);
    assert_eq!(f.evidence.validity(), findings::ValueValidity::Kept);
    assert_eq!(
        f.evidence
            .raw_fields()
            .iter()
            .find(|f| f.name == "tai_minus_native_seconds")
            .unwrap()
            .observed,
        findings::RawValue::Signed(37)
    );
}

/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
#[test]
fn stream_deferral_rejects_new_inputs_before_mutating_legacy_state() {
    use irig106_time::stream::*;
    let policy = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 64,
            references: 2,
        },
    );
    let mut stream = StreamTimeline::new(
        policy,
        Some(&[StreamChannel {
            id: 4,
            timein: true,
            external: false,
        }]),
    )
    .unwrap();
    let table = LeapSecondTable::builtin();
    let net = network(1, 4, 0, [1, 3_692_304_000, 0], false, &table);
    assert!(matches!(
        stream.observe(packet(1, 4, 0, Some(net)), 0, 12, false),
        Err(TimeError::NeedsContext { .. })
    ));
    // The failed future-adapter input must not consume the packet identity.
    let legacy = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 2, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    assert!(
        stream
            .observe(packet(1, 4, 0, Some(legacy)), 0, 12, false)
            .is_ok()
    );
}

/// Requirements: L1-COR-004, L2-COR-104, L3-COR-104
/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
#[test]
fn normalized_jump_residual_and_drift_use_physical_delta_and_coarser_resolution() {
    let table = LeapSecondTable::builtin();
    for threshold in [None, Some(1_000_000)] {
        let mut b = TimelineBuilder::new(OfflinePolicy {
            jump_threshold_ns: threshold,
            ..Default::default()
        })
        .unwrap();
        b.observe(packet(1, 4, 0, Some(utc(&table))), false)
            .unwrap();
        // RTC1s versus TAI1.005s: residual+5000000ns; +5000ppm.
        // Coarser Format1 resolution10ms suppresses a default jump, explicit1ms does not.
        b.observe(
            packet(
                2,
                4,
                10_000_000,
                Some(network(
                    2,
                    4,
                    10_000_000,
                    [0x21, 1_483_315_238, 5_000_000],
                    false,
                    &table,
                )),
            ),
            false,
        )
        .unwrap();
        let t = b.finish().unwrap();
        let jumps: Vec<_> = t.findings().iter().filter(|f| f.code == 17).collect();
        assert_eq!(jumps.len(), usize::from(threshold.is_some()));
        if let Some(f) = jumps.first() {
            assert_eq!(f.delta_ns, Some(5_000_000));
            assert_eq!(f.related, Some(1));
            assert_eq!(f.severity, Severity::Warning)
        }
        assert!((t.quality(1, 4).unwrap().drift_ppm.unwrap() - 5000.0).abs() < 0.00001);
    }
}

/// Requirements: L1-COR-006, L2-COR-106, L3-COR-106
/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
#[test]
fn undeclared_network_admission_and_boundary_levels_use_engine_policy() {
    let table = LeapSecondTable::builtin();
    // RFC5905 Fig4: 1970 NTP2208988800 = 0x83aa7e80 (LE); TS1.
    let payload = [1, 0, 0, 0, 0x80, 0x7e, 0xaa, 0x83, 0, 0, 0, 0];
    for include in [false, true] {
        let o = Format2Input {
            payload: &payload,
            data_length: 12,
            capture: ReferenceCapture {
                session: 1,
                packet: 1,
                channel: 4,
                rtc: Rtc::from_raw(0),
                byte_offset: Some(100),
            },
            declared_edition: Some(PacketEdition::V15),
            data_type_version: 8,
            declared_timein: false,
        }
        .decode()
        .unwrap();
        let r = ReferenceInput::format2(o, NtpContext::new(1), table.clone(), false).unwrap();
        let mut p = packet(1, 4, 0, Some(r));
        p.declared_timein = false;
        let mut b = TimelineBuilder::new(OfflinePolicy {
            include_undeclared: include,
            finding_levels: [(26, Severity::Information), (8, Severity::Information)].into(),
            ..Default::default()
        })
        .unwrap();
        b.observe(p, false).unwrap();
        let t = b.finish().unwrap();
        assert_eq!(t.sessions()[0].references[0].admitted, include);
        let net = t.sessions()[0].references[0]
            .input
            .network
            .as_ref()
            .unwrap();
        assert_eq!(net.reference().admission().undeclared, include);
        assert_eq!(net.reference().policy().include_undeclared, include);
        for id in [8, 26] {
            let f = t.findings().iter().find(|f| f.code == id).unwrap();
            assert_eq!(f.severity, Severity::Information);
            assert_eq!(f.evidence.location().byte_offset, Some(100));
        }
    }
}

/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101
#[test]
fn review_pre1970_utc_cannot_borrow_the_1972_offset() {
    // Fig11-14: 1969-01-01 00:00:00, FMT4 / DATE1, BCD year0x1969.
    // Unix=-365*86400=-31536000; S04's supported baseline starts in1972.
    let r = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x69, 0x19],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    assert!(matches!(
        r.normalize_to_tai(LeapSecondTable::builtin()),
        Err(TimeError::NeedsContext { .. })
    ));
}

/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-COR-004, L2-COR-104, L3-COR-104
/// Requirements: L1-COR-013, L2-COR-113, L3-COR-113
#[test]
fn review_partial_utc_normalization_cannot_fall_back_across_insertion() {
    let table = LeapSecondTable::builtin();
    // Fig11-14: 2016-12-31 23:59:59 -> 2017-01-01 00:00:00.
    // Native calendar delta1s; physical delta2s after the +1 insertion.
    let before = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0x59, 0x59, 0x23, 0x31, 0x12, 0x16, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let after = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    for normalize_before in [false, true] {
        let (mut a, mut z) = (before.clone(), after.clone());
        if normalize_before {
            a = a.normalize_to_tai(table.clone()).unwrap();
        } else {
            z = z.normalize_to_tai(table.clone()).unwrap();
        }
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(packet(1, 4, 0, Some(a)), false).unwrap();
        b.observe(packet(2, 4, 20_000_000, Some(z)), false).unwrap();
        let t = b.finish().unwrap();
        assert!(!t.findings().iter().any(|f| f.code == 17));
        assert!(t.quality(1, 4).unwrap().drift_ppm.is_none());
        // Drift and jump both consume absolute_delta: neither may claim a
        // physical bracket from this partly qualified pair. Stream is S06b2.
    }
}

/// Requirements: L1-F2COR-002, L2-F2COR-102, L3-F2COR-102
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101
/// Interpretations: RDG-010
#[test]
fn review_negative_step_shifts_skip_a_second_without_inventing_sixty() {
    // S04 caller fixture: hypothetical2028-01-01 B=1830297600, offset37->36.
    // UTC23:59:58 is TAI B+35; +1s reaches midnight at B+36; +2s reaches
    // UTC00:00:01 at B+37. No inserted second60 exists at a negative step.
    let old = LeapSecondTable::builtin();
    let mut info = old.metadata().clone();
    info.id = "s06b-negative-step".into();
    info.source = "synthetic caller data, not an IERS prediction".into();
    info.known_current_through = UnixSeconds::new(1_861_920_000);
    let table = old
        .extended(
            vec![LeapSecondEntry {
                effective_unix: UnixSeconds::new(1_830_297_600),
                tai_utc_offset: TaiUtcOffset::new(36),
            }],
            info,
        )
        .unwrap();
    let r = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0x58, 0x59, 0x23, 0x31, 0x12, 0x27, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .normalize_to_tai(table)
    .unwrap();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 4, 0, Some(r)), false).unwrap();
    let keys: Vec<_> = (0..6)
        .map(|i| {
            b.observe(packet(i + 2, 4, i * 5_000_000, None), false)
                .unwrap()
        })
        .collect();
    let t = b.finish().unwrap();
    for (i, key) in keys.into_iter().enumerate() {
        let answer = t.lookup(key).unwrap();
        assert!(
            answer.reading.is_some(),
            "negative step must retain an ordinary readable answer at sample {i}"
        );
        let r = answer.reading.unwrap();
        assert_ne!(r.seconds, 60);
        assert_eq!(r.nanoseconds, if i % 2 == 0 { 0 } else { 500_000_000 });
        assert_eq!(
            (r.year, r.day_of_year, r.hours, r.minutes, r.seconds),
            if i < 2 {
                (Some(2027), 365, 23, 59, 58)
            } else {
                (Some(2028), 1, 0, 0, (i / 2 - 1) as u8)
            }
        );
    }
}

/// Requirements: L1-COR-008, L2-COR-108, L3-COR-108
/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
#[test]
fn review_raw_rate_qualification_preserves_default_and_overridden_gap_level() {
    let table = LeapSecondTable::builtin();
    for level in [None, Some(Severity::Information)] {
        let mut policy = OfflinePolicy::default();
        if let Some(level) = level {
            policy.finding_levels.insert(16, level);
        }
        let mut b = TimelineBuilder::new(policy).unwrap();
        b.observe(
            packet(
                1,
                4,
                0,
                Some(network(1, 4, 0, [1, 3_692_304_000, 0], true, &table)),
            ),
            false,
        )
        .unwrap();
        b.observe(
            packet(
                2,
                4,
                20_000_000,
                Some(network(
                    2,
                    4,
                    20_000_000,
                    [0x21, 1_483_315_239, 0],
                    true,
                    &table,
                )),
            ),
            false,
        )
        .unwrap();
        let t = b.finish().unwrap();
        let f = t.findings().iter().find(|f| f.code == 16).unwrap();
        assert_eq!(f.severity, level.unwrap_or(Severity::Warning));
        assert_eq!(
            f.evidence
                .raw_fields()
                .iter()
                .find(|f| f.name == "standard_minimum_1hz_applies")
                .unwrap()
                .observed,
            findings::RawValue::Unsigned(0)
        );
    }
}

/// Requirements: L1-F2COR-001, L2-F2COR-101, L3-F2COR-101
/// Requirements: L1-COR-004, L2-COR-104, L3-COR-104
#[test]
fn review_network_precision_is_one_ns_not_format1_ten_ms() {
    let table = LeapSecondTable::builtin();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    // RTC1s versus absolute1s+100ns: +100ns residual is above1ns, below10ms.
    b.observe(
        packet(
            1,
            4,
            0,
            Some(network(1, 4, 0, [1, 3_692_304_000, 0], false, &table)),
        ),
        false,
    )
    .unwrap();
    let key = b
        .observe(
            packet(
                2,
                4,
                10_000_000,
                Some(network(
                    2,
                    4,
                    10_000_000,
                    [0x21, 1_483_315_238, 100],
                    false,
                    &table,
                )),
            ),
            false,
        )
        .unwrap();
    let t = b.finish().unwrap();
    assert_eq!(
        t.lookup(key)
            .unwrap()
            .basis
            .reference
            .unwrap()
            .input
            .precision_ns,
        1
    );
    let f = t.findings().iter().find(|f| f.code == 17).unwrap();
    assert_eq!(f.delta_ns, Some(100));
}
