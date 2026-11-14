//! 106-24R1 §11.2.3.3, printed 11-26/27 (PDF34/35), Figs11-15..17.
//! Literal little-endian words; no packet encoder supplies expected values.
use irig106_time::{findings::*, network_time::*, *};

/// Requirements: L1-F2CSDW-001, L2-F2CSDW-101, L3-F2CSDW-101
/// Requirements: L1-ERR-003, L2-ERR-105, L3-ERR-105
#[test]
fn existing_bounded_wire_control() {
    // NTF=1, TS=1; seconds 0x01020304, nanos=0x075bcd15=123456789.
    let bytes = [0x11, 0, 0, 0, 4, 3, 2, 1, 0x15, 0xcd, 0x5b, 7, 0xff];
    let r = decode_time_f2(&bytes, 12).unwrap();
    assert_eq!(r.consumed, 12);
    assert_eq!(
        r.time.unwrap().value,
        NetworkTime::Ptp(PtpTime {
            seconds: 0x01020304,
            nanoseconds: 123456789
        })
    );
    for n in 0..12 {
        assert!(decode_time_f2(&bytes, n).is_err());
    }
}

use irig106_time::{edition::PacketEdition, format2::*, leap::*, ntp::*, scale::*};
fn capture() -> ReferenceCapture {
    ReferenceCapture {
        session: 1,
        packet: 7,
        channel: 4,
        rtc: Rtc::new(123),
        byte_offset: Some(100),
    }
}
fn bytes(word: u32, seconds: u32, tail: u32) -> [u8; 12] {
    let mut out = [0; 12];
    out[..4].copy_from_slice(&word.to_le_bytes());
    out[4..8].copy_from_slice(&seconds.to_le_bytes());
    out[8..].copy_from_slice(&tail.to_le_bytes());
    out
}
fn decode(b: &[u8], declared: bool, edition: Option<PacketEdition>) -> Format2Observation {
    Format2Input {
        payload: b,
        data_length: 12,
        capture: capture(),
        declared_edition: edition,
        data_type_version: 8,
        declared_timein: declared,
    }
    .decode()
    .unwrap()
}
fn converted(o: &Format2Observation, p: AdmissionPolicy, t: &LeapSecondTable) -> Format2Reference {
    o.convert(&NtpContext::new(1), t, p).unwrap()
}
fn ids(r: &Format2Reference) -> Vec<u8> {
    r.findings().iter().map(|f| f.code().number()).collect()
}
fn field(f: &Finding, name: &str) -> RawValue {
    f.evidence()
        .raw_fields()
        .iter()
        .find(|x| x.name == name)
        .unwrap()
        .observed
}

/// Requirements: L1-F2CSDW-001, L2-F2CSDW-101, L3-F2CSDW-101
/// Requirements: L1-F2CSDW-004, L2-F2CSDW-104, L3-F2CSDW-104
/// Requirements: L1-F2CSDW-005, L2-F2CSDW-105, L3-F2CSDW-105
#[test]
fn all_ntf_ts_keep_raw_and_independent_admission() {
    for ntf in 0..16 {
        for ts in 0..16 {
            // Figures11-15..17: NTF high nibble, TS low nibble; PTP epoch0
            // stays TAI even though no post-1972 offset can map it to UTC.
            let b = bytes((ntf << 4) | ts, 1, 0);
            let o = decode(&b, true, Some(PacketEdition::V24R1));
            assert_eq!(o.csdw().as_raw(), (ntf << 4) | ts);
            assert_eq!(o.raw_body(), &b[4..12]);
            let r = converted(&o, AdmissionPolicy::default(), &LeapSecondTable::builtin());
            assert_eq!(r.is_eligible(), ntf < 3 && ts == 1, "{ntf}/{ts}");
            let expected = match (ntf >= 3, ts) {
                (false, 0) => vec![3],
                (false, 1) => vec![],
                (false, _) => vec![6],
                (true, 0) => vec![3, 6],
                (true, 1) => vec![6],
                (true, _) => vec![6, 6],
            };
            assert_eq!(ids(&r), expected, "{ntf}/{ts}");
            let mut p = AdmissionPolicy::default();
            p.include_invalid = true;
            let included = converted(&o, p, &LeapSecondTable::builtin());
            assert_eq!(included.is_eligible(), ntf < 3);
            assert_eq!(included.admission().invalid, ts != 1 && ntf < 3);
            assert_eq!(included.observation().csdw(), o.csdw());
            if ntf >= 3 {
                assert_eq!(r.conversion(), &NetworkConversion::UnsupportedFormat);
            }
        }
    }
}

/// Requirements: L1-F2CSDW-006, L2-F2CSDW-106, L3-F2CSDW-106
/// Requirements: L1-F2CSDW-004, L2-F2CSDW-104, L3-F2CSDW-104
/// Requirements: L1-F2CSDW-005, L2-F2CSDW-105, L3-F2CSDW-105
#[test]
fn zero_masks_report_exact_operands_without_discarding_time() {
    for bit in 8..32 {
        let b = bytes((1 << bit) | 0x21, 1, 999_999_999);
        let o = decode(&b, true, None);
        let r = converted(&o, AdmissionPolicy::default(), &LeapSecondTable::builtin());
        assert!(r.is_eligible());
        assert_eq!(ids(&r), vec![5]);
        let f = &r.findings()[0];
        assert_eq!(field(f, "reserved_csdw_bits"), RawValue::Unsigned(1 << bit));
        assert_eq!(f.evidence().raw_rtc(), Some(capture().rtc));
        assert_eq!(f.evidence().location().byte_offset, Some(100));
        assert_eq!(f.evidence().location().channel, Some(4));
        assert_eq!(f.evidence().validity(), ValueValidity::Kept);
        assert_eq!(f.severity(), Severity::Warning);
    }
    let b = bytes(0x1234_5632, 9, 0); // reserved TS=2, NTF=3, nonzero zero-mask.
    let r = converted(
        &decode(&b, true, None),
        AdmissionPolicy::default(),
        &LeapSecondTable::empty(),
    );
    assert_eq!(ids(&r), vec![6, 6, 5]);
    assert_eq!(field(&r.findings()[0], "ts"), RawValue::Unsigned(2));
    assert_eq!(field(&r.findings()[1], "ntf"), RawValue::Unsigned(3));
}

/// Requirements: L1-PTP-003, L2-PTP-103, L3-PTP-103
/// Requirements: L1-ERR-003, L2-ERR-105, L3-ERR-105
/// Requirements: L1-F2CSDW-006, L2-F2CSDW-106, L3-F2CSDW-106
#[test]
fn malformed_body_and_bounded_length_retain_independent_findings() {
    let b = bytes(0x8000_0021, 0x01020304, 1_000_000_000);
    let o = decode(&b, true, None);
    assert_eq!(o.raw_body(), &b[4..12]);
    assert!(matches!(
        o.decode_error(),
        Some(TimeError::OutOfRange { .. })
    ));
    let mut p = AdmissionPolicy::default();
    p.include_invalid = true;
    let r = converted(&o, p, &LeapSecondTable::empty());
    assert!(!r.is_eligible());
    assert_eq!(ids(&r), vec![5, 4]);
    assert_eq!(r.findings()[1].evidence().decode_error(), o.decode_error());
    assert_eq!(r.findings()[1].severity(), Severity::Error);
    let mut adjacent = b.to_vec();
    adjacent.extend_from_slice(&[0xff; 8]);
    for n in 0..12 {
        assert!(
            Format2Input {
                payload: &adjacent,
                data_length: n,
                capture: capture(),
                declared_edition: None,
                data_type_version: 8,
                declared_timein: true
            }
            .decode()
            .is_err()
        );
    }
    assert!(
        Format2Input {
            payload: &b,
            data_length: 13,
            capture: capture(),
            declared_edition: None,
            data_type_version: 8,
            declared_timein: true
        }
        .decode()
        .is_err()
    );
    assert_eq!(decode(&adjacent, true, None), o);
}

/// Requirements: L1-EDN-001, L2-EDN-101, L3-EDN-101
#[test]
fn declared_edition_diagnostic_does_not_infer_from_packet_version() {
    let b = bytes(1, 2_208_988_800, 0); // RFC5905 Fig4 p14: 1970 => Unix0.
    for e in [
        PacketEdition::V04,
        PacketEdition::V05,
        PacketEdition::V07,
        PacketEdition::V09,
        PacketEdition::V11,
        PacketEdition::V13,
        PacketEdition::V15,
    ] {
        let r = converted(
            &decode(&b, true, Some(e)),
            AdmissionPolicy::default(),
            &LeapSecondTable::empty(),
        );
        assert!(r.is_eligible());
        assert_eq!(ids(&r), vec![26]);
        assert_eq!(
            field(&r.findings()[0], "data_type"),
            RawValue::Unsigned(0x12)
        );
        assert_eq!(
            field(&r.findings()[0], "data_type_version"),
            RawValue::Unsigned(8)
        );
        assert_eq!(r.observation().declared_edition(), Some(e));
    }
    for e in [
        Some(PacketEdition::V17),
        Some(PacketEdition::V19),
        Some(PacketEdition::V20),
        Some(PacketEdition::V22),
        Some(PacketEdition::V23),
        Some(PacketEdition::V24),
        Some(PacketEdition::V24R1),
        None,
    ] {
        let r = converted(
            &decode(&b, true, e),
            AdmissionPolicy::default(),
            &LeapSecondTable::empty(),
        );
        assert!(ids(&r).is_empty());
        assert!(r.is_eligible());
        assert_eq!(r.observation().declared_edition(), e);
    }
}

/// Requirements: L1-PTP-002, L2-PTP-102, L3-PTP-102
/// Requirements: L1-TAI-005, L2-TAI-105, L3-TAI-105
/// Requirements: L1-TAI-006, L2-TAI-106, L3-TAI-106
/// Interpretations: RDG-009, RDG-012
#[test]
fn native_scales_and_d7_conversion_basis_are_preserved() {
    // 2017-01-02 Unix1483315200; raw TAI adds37 =>1483315237.
    let b = bytes(0x21, 1_483_315_237, 123);
    let o = decode(&b, true, None);
    assert_eq!(o.scale(), ScaleEvidence::standard(TimeScale::Tai));
    let table = LeapSecondTable::builtin();
    let r = converted(&o, AdmissionPolicy::default(), &table);
    match r.conversion() {
        NetworkConversion::Ptp(c) => {
            assert_eq!(c.raw().seconds, 1_483_315_237);
            assert_eq!(c.unix_seconds().unwrap().get(), 1_483_315_200);
            assert_eq!(
                c.utc().unwrap().scale,
                ScaleEvidence::standard(TimeScale::Utc)
            );
            assert_eq!(c.basis().offset.unwrap().get(), 37);
        }
        _ => panic!("PTP"),
    }
    let r = converted(&o, AdmissionPolicy::default(), &LeapSecondTable::empty());
    assert!(r.is_eligible()); // Native TAI is usable; UTC projection lacks context.
    assert!(
        matches!(r.conversion(), NetworkConversion::Ptp(c) if c.status()==PtpConversionStatus::MissingOffset)
    );
    let stale = bytes(0x11, 2_000_000_037, 0);
    let r = converted(
        &decode(&stale, true, None),
        AdmissionPolicy::default(),
        &table,
    );
    assert_eq!(ids(&r), vec![21]);
    assert!(
        matches!(r.conversion(),NetworkConversion::Ptp(c) if c.unix_seconds().unwrap().get()==2_000_000_000 && c.status()==PtpConversionStatus::PossiblyOutOfDate)
    );
    let reserved = decode(&bytes(0x31, 1, 0), true, None);
    assert_eq!(reserved.scale(), ScaleEvidence::unknown());
}

/// Requirements: L1-F2CSDW-005, L2-F2CSDW-105, L3-F2CSDW-105
/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104
/// Interpretations: RDG-012
#[test]
fn ntp_admission_keeps_sentinel_ambiguity_era_and_context() {
    let table = LeapSecondTable::builtin();
    let zero = converted(
        &decode(&bytes(1, 0, 0), true, None),
        AdmissionPolicy::default(),
        &table,
    );
    assert!(!zero.is_eligible());
    assert_eq!(ids(&zero), vec![29]);
    // 2017 midnight Unix1483228800 + epoch2208988800 = NTP3692217600.
    let o = decode(&bytes(1, 3_692_217_600, 0x80000000), true, None);
    assert_eq!(o.scale(), ScaleEvidence::standard(TimeScale::Utc));
    let r = converted(&o, AdmissionPolicy::default(), &table);
    assert!(r.is_eligible());
    assert_eq!(ids(&r), vec![30]);
    assert_eq!(r.findings()[0].evidence().raw_rtc(), Some(capture().rtc));
    match r.conversion() {
        NetworkConversion::Ntp(c) => {
            assert_eq!(c.unix_seconds().unwrap().get(), 1_483_228_800);
            assert_eq!(c.basis().era_kind, Some(EraBasis::AssumedZero));
            assert_eq!(c.basis().leap, NtpLeapStatus::Ambiguous);
        }
        _ => panic!("NTP"),
    }
    assert!(
        o.convert(&NtpContext::new(2), &table, AdmissionPolicy::default())
            .is_err()
    );
}

/// Requirements: L1-F2CSDW-005, L2-F2CSDW-105, L3-F2CSDW-105
/// Requirements: L1-COR-004, L2-COR-106, L3-COR-106
#[test]
fn admission_policy_and_severity_do_not_rewrite_facts() {
    let o = decode(&bytes(0x10, 1, 0), false, None);
    let r = converted(&o, AdmissionPolicy::default(), &LeapSecondTable::empty());
    assert!(!r.is_eligible());
    assert_eq!(ids(&r), vec![3, 8]);
    let mut p = AdmissionPolicy::default();
    p.include_invalid = true;
    p.include_undeclared = true;
    p.finding_levels.insert(3, Severity::Information);
    let r = converted(&o, p.clone(), &LeapSecondTable::empty());
    assert!(r.is_eligible());
    assert!(r.admission().invalid && r.admission().undeclared);
    assert_eq!(r.findings()[0].severity(), Severity::Information);
    assert_eq!(r.observation().csdw().time_status(), TimeStatus::NotValid);
    assert_eq!(r.policy(), &p);
    let mut bad = p;
    bad.finding_levels.insert(32, Severity::Warning);
    assert!(
        o.convert(&NtpContext::new(1), &LeapSecondTable::empty(), bad)
            .is_err()
    );
}
