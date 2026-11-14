//! S03: literal layouts, not encoder round trips. Sources: 106-04 Fig10-15
//! (10-30/PDF288, definitions10-31/289), 106-05 Fig10-16a (10-37/PDF41),
//! 106-24R1 §11.2.1.1 e (11-6/PDF14), Fig11-12 (11-24/PDF32).
use irig106_time::{csdw::ItsQualification, format1::*, stream::*, timeline::*, version::*, *};
fn declared(e: PacketEdition) -> EditionEvidence {
    EditionEvidence {
        override_edition: Some(e),
        ..EditionEvidence::default()
    }
}
fn tmats(raw: &str, adopt: bool, header: Option<u8>) -> EditionEvidence {
    EditionEvidence {
        tmats_edition: Some(raw.into()),
        use_tmats: adopt,
        data_type_version: header,
        ..EditionEvidence::default()
    }
}
/// Requirements: L1-EDN-002, L2-EDN-201, L3-EDN-201
#[test]
fn resolver_follows_every_accepted_evidence_row() {
    for e in [
        EditionEvidence::default(),
        EditionEvidence {
            data_type_version: Some(1),
            ..EditionEvidence::default()
        },
        tmats("04", false, Some(1)),
        tmats("broken", true, None),
    ] {
        let r = resolve_edition(e.clone());
        assert_eq!(r.edition(), None);
        assert!(!r.contradiction());
        assert_eq!(r.evidence(), &e);
    }
    for (text, edition) in [
        ("04", PacketEdition::V04),
        ("05", PacketEdition::V05),
        ("24", PacketEdition::V24),
    ] {
        let r = resolve_edition(tmats(text, true, Some(1)));
        assert_eq!(r.edition(), Some(edition));
        assert_eq!(r.basis(), EditionBasis::DeclaredAssumedFromTmats);
        assert!(!r.contradiction());
    }
    let r = resolve_edition(tmats("04", true, Some(10)));
    assert_eq!(r.edition(), None);
    assert!(r.contradiction());
    assert_eq!(r.evidence().data_type_version, Some(10));
    for edition in [PacketEdition::V04, PacketEdition::V05, PacketEdition::V24R1] {
        let mut e = tmats("05", true, Some(10));
        e.override_edition = Some(edition);
        let r = resolve_edition(e.clone());
        assert_eq!(r.edition(), Some(edition));
        assert_eq!(r.basis(), EditionBasis::Override);
        assert!(!r.contradiction());
        assert_eq!(r.evidence(), &e);
    }
    // Lower permitted header, unknown header, or differing unadopted TMATS alone
    // never establish a conflict. No host clock or fallback supplies an edition.
    assert!(!resolve_edition(tmats("24", true, Some(1))).contradiction());
    assert_eq!(
        resolve_edition(tmats("04", true, Some(255))).edition(),
        Some(PacketEdition::V04)
    );
}
/// Requirements: L1-EDN-002, L2-EDN-201, L3-EDN-201, L1-EDN-003, L2-EDN-103, L3-EDN-103
#[test]
fn setup_and_header_domains_do_not_choose_a_layout() {
    for setup in [
        SetupVersionEvidence::Unavailable,
        SetupVersionEvidence::Reserved(0x07070707),
        SetupVersionEvidence::Rccver(7),
        SetupVersionEvidence::Rccver(12),
        SetupVersionEvidence::Rccver(15),
    ] {
        let e = EditionEvidence {
            setup,
            data_type_version: Some(1),
            ..EditionEvidence::default()
        };
        let r = resolve_edition(e.clone());
        assert_eq!(r.edition(), None);
        assert_eq!(r.evidence(), &e);
    }
    assert_eq!(SetupVersionEvidence::Reserved(7).decoded(), None);
    assert_eq!(
        SetupVersionEvidence::Rccver(7).decoded(),
        Some(Irig106Version::V07)
    );
    assert_eq!(detect_data_type_version(7), Irig106Version::V15);
    assert_eq!(
        SetupVersionEvidence::Rccver(15).decoded(),
        Some(Irig106Version::Unknown(15))
    );
}
/// Requirements: L1-EDN-003, L2-EDN-103, L3-EDN-103, L1-RDG-002, L2-RDG-207, L3-RDG-207
/// Interpretations: RDG-007
#[test]
fn independent_edition_rule_boundaries_and_its_evidence() {
    use PacketEdition::*;
    let rows = [
        (V04, TimeF1Layout::Legacy04, 10, 1, false),
        (V05, TimeF1Layout::Source05, 10, 2, false),
        (V07, TimeF1Layout::Source05, 10, 3, false),
        (V09, TimeF1Layout::Source05, 10, 4, false),
        (V11, TimeF1Layout::Source05, 10, 5, false),
        (V13, TimeF1Layout::Source05, 10, 6, false),
        (V15, TimeF1Layout::Source05, 10, 7, false),
        (V17, TimeF1Layout::SourceWithIts17, 11, 8, true),
        (V19, TimeF1Layout::SourceWithIts17, 11, 9, true),
        (V20, TimeF1Layout::SourceWithIts17, 11, 9, true),
        (V22, TimeF1Layout::SourceWithIts17, 11, 10, true),
        (V23, TimeF1Layout::SourceWithIts17, 11, 10, true),
        (V24, TimeF1Layout::SourceWithIts17, 11, 10, true),
        (V24R1, TimeF1Layout::SourceWithIts17, 11, 10, true),
    ];
    for (e, layout, chapter, max, format2) in rows {
        let rule = e.rules();
        assert_eq!(
            (
                rule.layout,
                rule.chapter,
                rule.max_data_type_version,
                rule.format2
            ),
            (layout, chapter, max, format2)
        );
        assert!(!rule.source.is_empty());
        for raw in 0..16u32 {
            for fmt in [0, 1, 2, 3, 4, 5, 15] {
                let q = TimeF1Csdw::from_raw(1 | (fmt << 4) | (raw << 12))
                    .qualify(resolve_edition(declared(e)));
                let expected = if e.rules().layout != TimeF1Layout::SourceWithIts17 {
                    ItsQualification::NotReported
                } else if fmt <= 2 {
                    ItsQualification::Reported(IrigTimeSource::from_raw(raw as u8))
                } else {
                    ItsQualification::NotApplicable
                };
                assert_eq!(q.its(), expected, "{e:?} fmt{fmt} ITS{raw}");
            }
        }
    }
    let q = TimeF1Csdw::from_raw(3).qualify(resolve_edition(EditionEvidence::default()));
    assert_eq!(q.its(), ItsQualification::FreewheelingOrNotReported);
    let q = TimeF1Csdw::from_raw(0x3003).qualify(resolve_edition(EditionEvidence::default()));
    assert_eq!(q.its(), ItsQualification::Unresolved(3));
}
/// Requirements: L1-CSDW-001, L2-CSDW-101, L3-CSDW-101, L1-CSDW-002, L2-CSDW-102, L3-CSDW-102, L1-CSDW-003, L2-CSDW-103, L3-CSDW-103
#[test]
fn every_source_and_format_nibble_is_edition_qualified() {
    for raw in 0..16u32 {
        let word = TimeF1Csdw::from_raw(raw | (raw << 4));
        let old = word.qualify(resolve_edition(declared(PacketEdition::V04)));
        assert_eq!(
            old.source(),
            Some(TimeF1Source::ExternalTime {
                present: raw & 1 != 0,
                reserved_bits: (raw & 14) as u8
            })
        );
        assert_eq!(
            old.format(),
            Some(if raw >= 6 {
                TimeFormat::Reserved(raw as u8)
            } else {
                TimeFormat::from_raw(raw as u8)
            })
        );
        let modern = word.qualify(resolve_edition(declared(PacketEdition::V05)));
        assert_eq!(
            modern.source(),
            Some(TimeF1Source::Source(TimeSource::from_raw(raw as u8)))
        );
        assert_eq!(modern.format(), Some(TimeFormat::from_raw(raw as u8)));
        let unknown = word.qualify(resolve_edition(EditionEvidence::default()));
        assert_eq!(unknown.source(), None);
        assert_eq!(unknown.raw().as_raw(), raw | (raw << 4));
        assert_eq!(
            unknown.format(),
            if raw == 15 {
                None
            } else {
                Some(TimeFormat::from_raw(raw as u8))
            }
        );
    }
    let q = TimeF1Csdw::from_raw(3).qualify(resolve_edition(EditionEvidence {
        data_type_version: Some(1),
        ..EditionEvidence::default()
    }));
    assert_eq!(q.source(), None);
    assert_eq!(q.raw().as_raw(), 3);
}
/// Requirements: L1-CSDW-007, L2-CSDW-107, L3-CSDW-107
#[test]
fn each_reserved_bit_has_selected_or_competing_raw_intersections() {
    for bit in 0..32 {
        let raw = 1u32 << bit;
        let word = TimeF1Csdw::from_raw(raw);
        let alternatives = [raw & 0xfffffc0e, raw & 0xfffffc00, raw & 0xffff0c00];
        let q = word.qualify(resolve_edition(EditionEvidence::default()));
        assert_eq!(q.reserved_alternatives(), alternatives);
        assert_eq!(q.reserved_bits(), raw & 0xffff0c00);
        for (e, expected) in [PacketEdition::V04, PacketEdition::V05, PacketEdition::V17]
            .into_iter()
            .zip(alternatives)
        {
            let q = word.qualify(resolve_edition(declared(e)));
            assert_eq!(q.reserved_bits(), expected);
        }
    }
}
fn observation(id: u64, reading: QualifiedFormat1Reading) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(100),
        kind: PacketKind::Time,
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference: Some(ReferenceInput::qualified_format1(reading).unwrap()),
    }
}
/// Requirements: L1-EDN-002, L2-EDN-201, L3-EDN-201, L1-CSDW-002, L2-CSDW-102, L3-CSDW-102
#[test]
fn contradiction_keeps_body_and_evidence_through_both_engines() {
    // Fig11-13 day1 / midnight; source nibble3 must never become GPS or None.
    let bytes = [3, 0, 0, 0, 0, 0, 0, 0, 1, 0];
    for e in [
        EditionEvidence::default(),
        tmats("04", true, Some(10)),
        declared(PacketEdition::V04),
        declared(PacketEdition::V17),
    ] {
        let reading =
            decode_qualified_time_f1(&bytes, 10, e.clone(), ScalePolicy::default()).unwrap();
        assert_eq!(reading.time.unwrap().day_of_year, 1);
        let conflict = reading.csdw.resolution().contradiction();
        assert_eq!(conflict, e.use_tmats);
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let key = b.observe(observation(1, reading.clone()), false).unwrap();
        let t = b.finish().unwrap();
        let answer = t.lookup(key).unwrap();
        assert!(answer.time.is_some());
        let kept = answer.basis.reference.unwrap().input.qualified.unwrap();
        assert_eq!(kept.csdw.resolution().evidence(), &e);
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
                id: 7,
                timein: true,
                external: true,
            }]),
        )
        .unwrap();
        let batch = stream
            .observe(observation(1, reading), 0, 32, false)
            .unwrap();
        for findings in [t.findings(), batch.findings.as_slice()] {
            assert_eq!(findings.iter().any(|f| f.code == 31), conflict);
            assert_eq!(
                findings.iter().any(|f| f.code == 5),
                e.override_edition == Some(PacketEdition::V04)
            );
            assert_eq!(
                findings.iter().any(|f| f.code == 6),
                e.override_edition == Some(PacketEdition::V17)
            );
            if conflict {
                let f = findings.iter().find(|f| f.code == 31).unwrap();
                assert_eq!(f.severity, Severity::Warning);
                assert!(
                    f.evidence
                        .raw_fields()
                        .iter()
                        .any(|f| f.name == "data_type_version"
                            && f.observed == findings::RawValue::Unsigned(10))
                );
            }
        }
    }
}
/// Requirements: L1-CSDW-001, L2-CSDW-101, L3-CSDW-101, L1-CSDW-003, L2-CSDW-103, L3-CSDW-103
#[test]
fn qualified_data_length_and_unresolved_fmt15_never_invent_none() {
    let bytes = [0xf3, 0, 0, 0, 0, 0, 0, 0, 1, 0, 255, 255];
    for len in 0..10 {
        assert!(
            decode_qualified_time_f1(
                &bytes,
                len,
                EditionEvidence::default(),
                ScalePolicy::default()
            )
            .is_err()
        );
    }
    let r = decode_qualified_time_f1(
        &bytes,
        10,
        EditionEvidence::default(),
        ScalePolicy::default(),
    )
    .unwrap();
    assert!(r.time.is_none());
    assert_eq!(r.csdw.format(), None);
    assert_eq!(r.consumed, 10);
    for (e, fmt) in [
        (PacketEdition::V04, TimeFormat::Reserved(15)),
        (PacketEdition::V05, TimeFormat::None),
    ] {
        let r = decode_qualified_time_f1(&bytes, 10, declared(e), ScalePolicy::default()).unwrap();
        assert_eq!(r.csdw.format(), Some(fmt));
        assert!(r.time.is_none());
    }
}
