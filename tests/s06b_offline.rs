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
    word: u32,
    seconds: u32,
    tail: u32,
    raw: bool,
    table: &LeapSecondTable,
) -> ReferenceInput {
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
            Some(network(1, 4, 0, 1, 3_692_304_000, 0, false, &t)),
        ),
        false,
    )
    .unwrap();
    b.observe(
        packet(
            2,
            4,
            10_000_000,
            Some(network(2, 4, 10_000_000, 0x21, 1_483_315_238, 0, false, &t)),
        ),
        false,
    )
    .unwrap();
    let f1 = utc(&t); // same UTC start; RTC2s => deliberate -2s jump.
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
    assert_eq!(answer.time.unwrap().nanoseconds(), 500_000_000);
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
    for raw in [false, true] {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(
            packet(
                1,
                4,
                0,
                Some(network(1, 4, 0, 1, 3_692_304_000, 0, raw, &t)),
            ),
            false,
        )
        .unwrap();
        b.observe(
            packet(
                2,
                4,
                20_000_000,
                Some(network(2, 4, 20_000_000, 0x11, 1_483_315_239, 0, raw, &t)),
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
            findings::RawValue::Unsigned(u128::from(!raw))
        );
        assert!(
            timeline.sessions()[0].references[1]
                .input
                .network
                .as_ref()
                .unwrap()
                .raw_network_rate()
                == raw
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
                Some(network(1, 4, 0, 1, 3_692_304_000, 0, false, &t)),
            ),
            false,
        )
        .unwrap();
        b.observe(
            packet(
                2,
                4,
                5_000_000,
                Some(network(2, 4, 5_000_000, 0x20, 1_483_315_300, 0, false, &t)),
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
            Some(network(1, 4, 0, 1, 3_692_217_600, 0x80000000, false, &t)),
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
                0x21,
                1_483_228_838,
                500_000_000,
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
            Some(network(1, 4, 0, 0x21, 2_000_000_037, 0, false, &t)),
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
