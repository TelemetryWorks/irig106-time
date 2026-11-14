//! Catalog/evidence fixtures: architecture §8, owner 2026-11-13; RFC5905 §6
//! p14 zero sentinel; RFC1305 E.8 pp78–79 duplicate insertion coordinate.
use irig106_time::{NtpTime, Rtc, UnixSeconds, findings::*, timeline::Severity};
fn location() -> FindingLocation {
    FindingLocation {
        session: Some(2),
        packet: Some(77),
        channel: Some(4),
        byte_offset: None,
    }
}
/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
#[test]
fn every_catalog_level_and_pending_identity_is_explicit() {
    // Independent table copied from accepted architecture rows, not code logic.
    use Severity::{Error as E, Information as I, Warning as W};
    let levels = [
        W, W, W, E, W, W, W, W, I, W, I, E, W, W, E, W, W, I, W, I, W, I, I, W, W, W, I, E, W, W, W,
    ];
    for (i, level) in levels.into_iter().enumerate() {
        let id = FindingCode::from_number((i + 1) as u8).unwrap();
        assert_eq!(id.default_severity(), Some(level));
        assert_eq!(id.resolve_level(None).unwrap(), level);
        assert!(!id.title().is_empty());
    }
    for (id, title) in [
        (32, "Year evidence is contradictory"),
    ] {
        let code = FindingCode::from_number(id).unwrap();
        assert_eq!(code.title(), title);
        assert_eq!(code.default_severity(), None);
        for level in [None, Some(W), Some(E)] {
            assert!(code.resolve_level(level).is_err());
        }
    }
    for id in [0, 33, 255] {
        assert!(FindingCode::from_number(id).is_err());
    }
}
/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
#[test]
fn raw_evidence_offsets_relations_and_validity_survive_override() {
    let mut evidence = FindingEvidence::new(location(), ValueValidity::Unusable);
    evidence = evidence.with_raw_rtc(Rtc::MAX);
    evidence = evidence.with_raw_field(RawField {
        name: "csdw",
        observed: RawValue::Unsigned(u32::MAX.into()),
        expected: Some(RawValue::Unsigned(0)),
    });
    evidence = evidence.with_related(FindingLocation {
        byte_offset: Some(4096),
        ..location()
    });
    let f = Finding::new(
        FindingCode::from_number(4).unwrap(),
        Some(Severity::Information),
        evidence.clone(),
    )
    .unwrap();
    assert_eq!(f.severity(), Severity::Information);
    assert_eq!(f.evidence(), &evidence);
    assert_eq!(f.evidence().validity(), ValueValidity::Unusable);
    assert_eq!(f.evidence().location().byte_offset, None); // Packet ID is NOT an offset.
    assert_eq!(f.evidence().related()[0].byte_offset, Some(4096));
}
/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
/// Interpretations: RDG-012
#[test]
fn ntp_zero_and_ambiguity_evidence_are_distinct() {
    let raw = NtpTime {
        seconds: 0,
        fraction: 0,
    };
    let zero = Finding::ntp_zero(location(), raw, None).unwrap();
    assert_eq!(zero.code().number(), 29);
    assert_eq!(zero.severity(), Severity::Warning);
    assert_eq!(zero.evidence().validity(), ValueValidity::Unusable);
    assert_eq!(
        zero.evidence().ntp(),
        Some(NtpEvidence {
            raw,
            mapped_unix: None
        })
    );
    assert!(
        Finding::ntp_zero(
            location(),
            NtpTime {
                seconds: 0,
                fraction: 1
            },
            None
        )
        .is_err()
    );
    assert!(
        Finding::ntp_zero(
            location(),
            NtpTime {
                seconds: 1,
                fraction: 0
            },
            None
        )
        .is_err()
    );
    // RFC1305 E.8 duplicate-coordinate evidence is supplied by the later S05
    // converter, not inferred by this plumbing. Fraction=0x80000000 = 1/2 s.
    let raw = NtpTime {
        seconds: 3_692_217_600,
        fraction: 0x8000_0000,
    };
    let mapped = UnixSeconds::new(1_483_228_800); // 2017-01-01, seconds minus2208988800.
    let f = Finding::ntp_ambiguity(location(), raw, mapped, Some(Severity::Information)).unwrap();
    assert_eq!(f.code().number(), 30);
    assert_eq!(f.severity(), Severity::Information);
    assert_eq!(f.evidence().validity(), ValueValidity::Kept);
    assert_eq!(
        f.evidence().ntp(),
        Some(NtpEvidence {
            raw,
            mapped_unix: Some(mapped)
        })
    );
    assert_eq!(f.evidence().location(), location());
}

/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
/// Interpretations: RDG-012
#[test]
fn sentinel_evidence_cannot_keep_an_absolute_value() {
    let raw = NtpTime {
        seconds: 0,
        fraction: 0,
    };
    assert!(Finding::ntp_ambiguity(location(), raw, UnixSeconds::new(0), None).is_err());
}

/// Requirements: L1-FND-001, L2-FND-101, L3-FND-101
/// Interpretations: RDG-012
#[test]
fn each_ntp_evidence_invariant_is_independent() {
    let zero = NtpTime {
        seconds: 0,
        fraction: 0,
    };
    let raw = NtpTime {
        seconds: 3_692_217_600,
        fraction: 0x8000_0000,
    };
    let mapped = Some(UnixSeconds::new(1_483_228_800));
    // Exactly ONE invariant differs from the corresponding valid fixture.
    for (id, validity, raw, mapped_unix) in [
        (29, ValueValidity::Unusable, zero, mapped), // mapped value only
        (29, ValueValidity::Kept, zero, None),       // validity only
        (30, ValueValidity::Kept, raw, None),        // missing mapping only
        (30, ValueValidity::Unusable, raw, mapped),  // validity only
    ] {
        let mut evidence = FindingEvidence::new(location(), validity);
        evidence = evidence.with_ntp(NtpEvidence { raw, mapped_unix });
        assert!(
            Finding::new(FindingCode::from_number(id).unwrap(), None, evidence).is_err(),
            "TF-{id} {validity:?} mapped={mapped_unix:?}"
        );
    }
}
