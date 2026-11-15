//! S09b hand-derived Table9-4 (9-72/PDF78), §11.2.3.2a (11-24/PDF32),
//! §11.2.1.1g (11-8/PDF16), Fig11-4 (11-10/PDF18); RDG005/006.
use irig106_time::{csdw::QualifiedCsdw, declarations::*, edition::*, findings::*, *};
fn key(id: u64, ticks: i128) -> timeline::PacketKey {
    timeline::PacketKey {
        session: 1,
        packet: id,
        ticks,
    }
}
fn ch(id: u16, tsrc: Option<&str>, shtf: Option<&str>) -> ChannelDeclaration {
    ChannelDeclaration {
        id,
        cdt: Some("TIMEIN".into()),
        ttf: Some("1".into()),
        tfmt: Some("B".into()),
        tsrc: tsrc.map(Into::into),
        shtf: shtf.map(Into::into),
    }
}
fn setup(start: i128, end: Option<i128>, channels: Vec<ChannelDeclaration>) -> SetupDeclarations {
    SetupDeclarations {
        range: GoverningRange::new(1, SignedRtcTicks::new(start), end.map(SignedRtcTicks::new))
            .unwrap(),
        setup_packet: 100 + start as u64,
        provenance: format!("setup:{start}"),
        declared_version: Some("106-24R1".into()),
        ri4: None,
        recording_year: None,
        channels,
    }
}
fn ds(tsrc: Option<&str>, shtf: Option<&str>) -> Declarations {
    Declarations::present(vec![setup(0, None, vec![ch(7, tsrc, shtf)])]).unwrap()
}
fn word(raw: u32, edition: Option<PacketEdition>) -> QualifiedCsdw {
    TimeF1Csdw::from_raw(raw).qualify(resolve_edition(EditionEvidence {
        override_edition: edition,
        ..Default::default()
    }))
}
fn fields(id: u64, ticks: i128, channel: u16, raw: u32) -> PacketFields {
    PacketFields {
        key: key(id, ticks),
        channel,
        data_type: 0x11,
        format: Some(((raw >> 4) & 15) as u8),
        source: Some((raw & 15) as u8),
    }
}
fn codes(r: &DetailedComparison) -> Vec<u8> {
    r.findings
        .iter()
        .map(|f| f.finding.code().number())
        .collect()
}
fn source_suite() {
    for (letter, expected) in [("I", 0), ("E", 1), ("R", 2), ("X", 15)] {
        let m = source_mapping(letter).unwrap();
        assert_eq!(
            (m.wire, m.reading.id(), m.inferred),
            (expected, "RDG-005", false)
        );
        let d = ds(Some(letter), None);
        let before = d.snapshots().unwrap().to_vec();
        for observed in 0..=15 {
            let q = word(observed.into(), Some(PacketEdition::V24R1));
            let p = fields(1, 0, 7, observed.into());
            let r = d
                .compare_qualified(p, Some(&q), None, &Default::default())
                .unwrap();
            assert_eq!(r.tsrc.expected, Some(expected));
            assert_eq!(r.tsrc.observed, Some(observed));
            assert_eq!(r.tsrc.raw.as_deref(), Some(letter));
            assert!(!r.tsrc.default_derived);
            assert_eq!(
                r.tsrc.status,
                if observed == expected {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert_eq!(codes(&r).contains(&11), observed != expected);
            assert_eq!(codes(&r).contains(&6), matches!(observed, 3..=14));
            for f in r
                .findings
                .iter()
                .filter(|f| f.finding.code().number() == 11)
            {
                assert_eq!(f.finding.severity(), Severity::Information);
                assert_eq!(f.field, DeclarationField::Tsrc);
                assert_eq!(f.comparison.reading.id(), "RDG-005");
                assert_eq!(f.finding.evidence().validity(), ValueValidity::Undetermined);
            }
            assert_eq!(r.basic.packet, p);
            assert_eq!(r.csdw.as_ref(), Some(&q));
        }
        assert_eq!(d.snapshots().unwrap(), before);
    }
    for letter in ["G", "e", " E ", ""] {
        assert!(source_mapping(letter).is_none());
    }
    for (value, status) in [
        (None, ComparisonStatus::MissingDeclaration),
        (Some("?"), ComparisonStatus::UnknownDeclaration),
    ] {
        let d = ds(value, None);
        let q = word(1, Some(PacketEdition::V05));
        let r = d
            .compare_qualified(fields(1, 0, 7, 1), Some(&q), None, &Default::default())
            .unwrap();
        assert_eq!(r.tsrc.status, status);
        assert_eq!(r.tsrc.expected, None);
        assert!(!codes(&r).contains(&11));
    }
    // Loss AND regain, including a regain that again matches static TSRC=E.
    let d = ds(Some("E"), None);
    let mut a = DeclarationAudit::new(&d);
    for (id, src, changed, mismatched) in [
        (1, 1, false, false),
        (2, 0, true, true),
        (3, 1, true, false),
    ] {
        let q = word(src, Some(PacketEdition::V24R1));
        let r = a
            .observe(
                fields(id, id as i128, 7, src),
                Some(&q),
                None,
                &Default::default(),
            )
            .unwrap();
        let changes: Vec<_> = r
            .findings
            .iter()
            .filter(|f| f.reason == DeclarationIssue::SourceChanged)
            .collect();
        assert_eq!(changes.len(), usize::from(changed));
        assert_eq!(
            r.findings.iter().any(
                |f| f.field == DeclarationField::Tsrc && f.reason == DeclarationIssue::Mismatch
            ),
            mismatched
        );
        for f in changes {
            assert_eq!(f.finding.severity(), Severity::Information);
            assert!(
                f.finding
                    .evidence()
                    .related()
                    .iter()
                    .any(|l| l.packet == Some(id - 1))
            );
            assert!(
                f.finding
                    .evidence()
                    .raw_fields()
                    .iter()
                    .any(|v| v.name == "packet.previous_source"
                        && v.observed == RawValue::Unsigned(if id == 2 { 1 } else { 0 }))
            );
        }
    }
    assert!(a.finish(&Default::default()).unwrap().is_empty());
}
/// Requirements: L2-RDG-205, L3-RDG-205
/// Interpretations: RDG-005
#[test]
fn all_source_pairs_reserved_missing_and_loss_regain_are_informational() {
    source_suite();
}

fn secondary_suite() {
    for (text, expected) in [("0", 0), ("1", 1), ("2", 2)] {
        assert_eq!(secondary_mapping(text).unwrap().wire, expected);
        let mut c = ch(8, None, Some(text));
        c.cdt = Some("PCM".into());
        let d =
            Declarations::present(vec![setup(0, None, vec![ch(7, None, Some("2")), c])]).unwrap();
        for flags in 0..=255 {
            let r = d
                .compare_qualified(fields(1, 0, 8, 0), None, Some(flags), &Default::default())
                .unwrap();
            let fmt = (flags >> 2) & 3;
            assert_eq!(r.flags, Some(flags));
            assert_eq!(r.shtf.reading.id(), "RDG-006");
            assert_eq!(
                r.shtf.status,
                if flags & 0x80 == 0 {
                    ComparisonStatus::NotApplicable
                } else if fmt == expected {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert_eq!(
                codes(&r).contains(&10),
                flags & 0x80 != 0 && fmt != expected
            );
            assert_eq!(codes(&r).contains(&6), flags & 0x80 != 0 && fmt == 3);
            assert_eq!(r.secondary_wording.is_some(), flags & 0x80 != 0 && fmt == 0);
            assert_eq!(r.basic.basis.channel.unwrap().shtf.as_deref(), Some(text));
        }
        let r = d
            .compare_qualified(fields(1, 0, 7, 0), None, Some(0x88), &Default::default())
            .unwrap();
        assert_eq!(r.shtf.status, ComparisonStatus::Match);
        assert_eq!(r.shtf.expected, Some(2));
    }
    for value in [None, Some("?"), Some("3")] {
        let d = ds(None, value);
        let r = d
            .compare_qualified(fields(1, 0, 7, 0), None, Some(0x80), &Default::default())
            .unwrap();
        assert_eq!(
            r.shtf.status,
            if value.is_none() {
                ComparisonStatus::MissingDeclaration
            } else {
                ComparisonStatus::UnknownDeclaration
            }
        );
        assert!(
            r.findings
                .iter()
                .any(|f| f.field == DeclarationField::Shtf && f.finding.code().number() == 10)
        );
        assert!(!r.shtf.default_derived);
        assert_eq!(r.shtf.published_default, None);
        assert_eq!(r.shtf.expected, None);
    }
    for value in ["00", "bcd", "A", ""] {
        assert!(secondary_mapping(value).is_none());
    }
    // Fig11-4 reserved/usec/low/high: BCD-looking bytes still mean binary
    // usec0x0123=291, low0x1234=4660, NOT decimal123 /1234.
    let d = ds(None, Some("0"));
    let r = d
        .compare_qualified(fields(1, 0, 7, 0), None, Some(0x80), &Default::default())
        .unwrap();
    assert!(r.secondary_wording.unwrap().contains("binary weighted"));
    let bytes = [0, 0, 0x23, 1, 0x34, 0x12, 0, 0, 0, 0, 0x6a, 0]; // byte sum0x6a
    let t = secondary::parse_secondary_header(&bytes, secondary::SecHdrTimeFormat::Ch4).unwrap();
    let secondary::SecondaryHeaderTime::Ch4(t) = t else {
        panic!("binary Ch4 required")
    };
    assert_eq!((t.high_order, t.low_order, t.microseconds), (0, 4660, 291));
}
/// Requirements: L2-RDG-206, L3-RDG-206
/// Interpretations: RDG-006
#[test]
fn all_shtf_pairs_flags_channels_and_binary_dispatch_keep_wording_discrepancy() {
    secondary_suite();
}

#[test]
fn edition_qualification_withholds_ext_and_unresolved_without_source_guess() {
    for edition in [
        None,
        Some(PacketEdition::V04),
        Some(PacketEdition::V05),
        Some(PacketEdition::V24R1),
    ] {
        for src in [0, 1, 3, 15] {
            let q = word(src, edition);
            let d = ds(Some("E"), None);
            let r = d
                .compare_qualified(fields(1, 0, 7, src), Some(&q), None, &Default::default())
                .unwrap();
            if matches!(edition, None | Some(PacketEdition::V04)) {
                assert_eq!(r.tsrc.status, ComparisonStatus::UnavailablePacketField);
                assert_eq!(r.tsrc.observed, None);
                assert!(!codes(&r).contains(&11));
                assert!(!codes(&r).contains(&6));
            } else {
                assert_eq!(r.tsrc.observed, Some(src as u8));
            }
            assert_eq!(r.csdw.as_ref().unwrap().raw().as_raw(), src);
            assert_eq!(r.csdw.unwrap().resolution(), q.resolution());
        }
    }
    let d = ds(Some("E"), None);
    let q = word(1, Some(PacketEdition::V24R1));
    assert!(
        d.compare_qualified(fields(1, 0, 7, 0), Some(&q), None, &Default::default())
            .is_err()
    );
    let q = word(1, None);
    let mut a = DeclarationAudit::new(&d);
    a.observe(fields(1, 0, 7, 1), Some(&q), None, &Default::default())
        .unwrap();
    let q = word(0, Some(PacketEdition::V24R1));
    let r = a
        .observe(fields(2, 1, 7, 0), Some(&q), None, &Default::default())
        .unwrap();
    assert!(
        !r.findings
            .iter()
            .any(|f| f.reason == DeclarationIssue::SourceChanged)
    );
}

fn basic_suite() {
    // Whole FND104 includes S09a obligations, not merely the new clauses.
    let letters = [
        ("B", 0),
        ("A", 1),
        ("G", 2),
        ("I", 3),
        ("U", 4),
        ("N", 5),
        ("X", 15),
    ];
    for (letter, expected) in letters {
        let mut c = ch(7, None, None);
        c.tfmt = Some(letter.into());
        let d = Declarations::present(vec![setup(0, None, vec![c])]).unwrap();
        for observed in 0..=15 {
            let q = word(u32::from(observed) << 4, Some(PacketEdition::V24R1));
            let r = d
                .compare_qualified(
                    fields(1, 0, 7, q.raw().as_raw()),
                    Some(&q),
                    None,
                    &Default::default(),
                )
                .unwrap();
            assert_eq!(
                r.basic.tfmt.status,
                if observed == expected {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert_eq!(r.basic.tfmt.inferred, letter == "I");
            assert_eq!(format1_declaration(expected).unwrap().declared, letter);
            assert_eq!(
                r.findings
                    .iter()
                    .any(|f| f.field == DeclarationField::Tfmt && f.finding.code().number() == 10),
                observed != expected
            );
            for f in r
                .findings
                .iter()
                .filter(|f| f.field == DeclarationField::Tfmt && f.finding.code().number() == 10)
            {
                assert_eq!(f.finding.severity(), Severity::Warning);
                assert_eq!(f.comparison.inferred, letter == "I");
            }
        }
    }
    for (ttf, expected) in [("1", 0x11), ("2", 0x12)] {
        let mut c = ch(7, None, None);
        c.ttf = Some(ttf.into());
        c.tfmt = Some(if ttf == "1" { "B" } else { "0" }.into());
        let d = Declarations::present(vec![setup(0, None, vec![c])]).unwrap();
        for observed in 0..=255 {
            let mut p = fields(1, 0, 7, 0);
            p.data_type = observed;
            let r = d
                .compare_qualified(p, None, None, &Default::default())
                .unwrap();
            assert_eq!(r.basic.ttf.expected, Some(expected));
            assert_eq!(
                r.basic.ttf.status,
                if observed == expected {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
        }
    }
    for n in 0..=2 {
        let mut c = ch(7, None, None);
        c.ttf = Some("2".into());
        c.tfmt = Some(n.to_string());
        let d = Declarations::present(vec![setup(0, None, vec![c])]).unwrap();
        for observed in 0..=15 {
            let p = PacketFields {
                data_type: 0x12,
                format: Some(observed),
                ..fields(1, 0, 7, 0)
            };
            let r = d
                .compare_qualified(p, None, None, &Default::default())
                .unwrap();
            assert_eq!(
                r.basic.tfmt.status,
                if observed == n {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert_eq!(r.basic.tfmt.reading.id(), "RDG-004");
        }
    }
    for explicit in [None, Some("A"), Some("?"), Some(" B ")] {
        for opt_in in [false, true] {
            let mut c = ch(7, None, None);
            c.tfmt = explicit.map(Into::into);
            let d = Declarations::present(vec![setup(0, None, vec![c])]).unwrap();
            let before = d.snapshots().unwrap().to_vec();
            for observed in [0u8, 1] {
                let q = word(u32::from(observed) << 4, Some(PacketEdition::V24R1));
                let r = d
                    .compare_qualified(
                        fields(1, 0, 7, q.raw().as_raw()),
                        Some(&q),
                        None,
                        &DeclarationPolicy {
                            use_tfmt_default: opt_in,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                assert_eq!(r.basic.tfmt.raw, explicit.map(Into::into));
                assert_eq!(r.basic.tfmt.default_derived, explicit.is_none() && opt_in);
                assert_eq!(
                    r.findings.iter().any(|f| f.field == DeclarationField::Tfmt
                        && f.reason == DeclarationIssue::MissingRequired),
                    explicit.is_none()
                );
                if explicit.is_none() {
                    assert_eq!(r.basic.tfmt.effective, opt_in.then(|| "A".into()));
                    assert_eq!(r.basic.tfmt.published_default, Some("A"));
                    assert_eq!(r.basic.tfmt.expected, opt_in.then_some(1));
                }
                if explicit == Some("A") {
                    assert_eq!(r.basic.tfmt.effective.as_deref(), Some("A"));
                }
                if explicit.is_some_and(|v| v != "A") {
                    assert_eq!(r.basic.tfmt.status, ComparisonStatus::UnknownDeclaration);
                }
            }
            assert_eq!(d.snapshots().unwrap(), before);
        }
    }
}
/// Requirements: L1-FND-004, L2-FND-104, L3-FND-104
/// Interpretations: RDG-001, RDG-002, RDG-003, RDG-004, RDG-005, RDG-006
#[test]
fn combined_registered_comparisons_cover_the_whole_finding_contract() {
    basic_suite();
    source_suite();
    secondary_suite();
    let d = Declarations::present(vec![
        setup(0, Some(10), vec![ch(7, Some("E"), Some("0"))]),
        setup(10, None, vec![ch(7, Some("I"), Some("1"))]),
    ])
    .unwrap();
    for (tick, src, flags, start) in [(9, 0, 0x84, 0), (10, 1, 0x80, 10)] {
        let q = word(src, Some(PacketEdition::V24R1));
        let r = d
            .compare_qualified(
                fields(1, tick, 7, src),
                Some(&q),
                Some(flags),
                &Default::default(),
            )
            .unwrap();
        for f in r
            .findings
            .iter()
            .filter(|f| matches!(f.field, DeclarationField::Tsrc | DeclarationField::Shtf))
        {
            assert_eq!(
                f.basis.governing.as_ref().unwrap().range.start().get(),
                start
            );
            assert_eq!(
                f.comparison.observed,
                Some(if f.field == DeclarationField::Tsrc {
                    src as u8
                } else {
                    (flags >> 2) & 3
                })
            );
            assert!(
                f.finding
                    .evidence()
                    .related()
                    .iter()
                    .any(|l| l.packet == Some(100 + start as u64))
            );
        }
    }
}

#[test]
fn audit_counts_time_packets_per_range_not_valid_references_and_keeps_severities() {
    let d = Declarations::present(vec![
        setup(0, Some(10), vec![ch(7, None, None), ch(8, None, None)]),
        setup(10, None, vec![ch(7, None, None)]),
    ])
    .unwrap();
    let mut a = DeclarationAudit::new(&d);
    let r = a
        .observe(fields(1, 1, 9, 0), None, None, &Default::default())
        .unwrap();
    assert_eq!(r.activity[0].finding.code().number(), 8);
    assert_eq!(r.activity[0].finding.severity(), Severity::Warning);
    // No readable body required: Format2 packet on7 counts as a time packet.
    a.observe(
        PacketFields {
            data_type: 0x12,
            ..fields(2, 2, 7, 0)
        },
        None,
        None,
        &Default::default(),
    )
    .unwrap();
    // A data packet on declared8 does NOT fill its time-packet obligation.
    a.observe(
        PacketFields {
            data_type: 0x19,
            ..fields(3, 3, 8, 0)
        },
        None,
        None,
        &Default::default(),
    )
    .unwrap();
    let missing = a.finish(&Default::default()).unwrap();
    assert_eq!(missing.len(), 2);
    for (f, ch, start) in [(&missing[0], 8, 0), (&missing[1], 7, 10)] {
        assert_eq!(f.finding.code().number(), 9);
        assert_eq!(f.finding.severity(), Severity::Information);
        assert_eq!(f.basis.channel.as_ref().unwrap().id, ch);
        assert_eq!(
            f.basis.governing.as_ref().unwrap().range.start().get(),
            start
        );
        assert_eq!(f.finding.evidence().location().packet, None);
        assert_eq!(f.finding.evidence().validity(), ValueValidity::Undetermined);
    }
    let d = ds(Some("E"), Some("1"));
    let mut p = DeclarationPolicy::default();
    for id in [6, 8, 9, 10, 11] {
        p.levels.insert(id, Severity::Error);
    }
    let q = word(3, Some(PacketEdition::V24R1));
    let mut a = DeclarationAudit::new(&d);
    let r = a
        .observe(fields(1, 0, 7, 3), Some(&q), Some(0x80), &p)
        .unwrap();
    assert!(codes(&r).contains(&6));
    assert!(codes(&r).contains(&10));
    assert!(codes(&r).contains(&11));
    assert!(
        r.findings
            .iter()
            .all(|f| f.finding.severity() == Severity::Error
                && f.finding.evidence().validity() == ValueValidity::Undetermined)
    );
    let r = a.observe(fields(2, 1, 8, 0), None, None, &p).unwrap();
    assert_eq!(r.activity[0].finding.severity(), Severity::Error);
    let missing = DeclarationAudit::new(&d).finish(&p).unwrap();
    assert_eq!(missing[0].finding.severity(), Severity::Error);
}

// Partial context/activity guards; no broader requirement promotion.
#[test]
fn only_qualified_defined_sources_enter_session_channel_history() {
    let mut second=setup(0,None,vec![ch(7,Some("E"),None)]);
    second.range=GoverningRange::new(2,SignedRtcTicks::new(0),None).unwrap();
    let d=Declarations::present(vec![setup(0,None,vec![ch(7,Some("E"),None),ch(8,Some("E"),None)]),second]).unwrap();
    let mut a=DeclarationAudit::new(&d);
    let q=word(1,Some(PacketEdition::V24R1));
    a.observe(fields(1,0,7,1),Some(&q),None,&Default::default()).unwrap();
    for (id,q) in [(2,word(0,None)),(3,word(0,Some(PacketEdition::V04))),(4,word(3,Some(PacketEdition::V24R1)))] {
        let r=a.observe(fields(id,id as i128,7,q.raw().as_raw()),Some(&q),None,&Default::default()).unwrap();
        assert!(!r.findings.iter().any(|f|f.reason==DeclarationIssue::SourceChanged));
    }
    let q=word(0,Some(PacketEdition::V24R1));
    let r=a.observe(fields(5,5,8,0),Some(&q),None,&Default::default()).unwrap();
    assert!(!r.findings.iter().any(|f|f.reason==DeclarationIssue::SourceChanged));
    let r=a.observe(PacketFields{data_type:0x12,..fields(6,6,7,0)},None,None,&Default::default()).unwrap();
    assert_eq!(r.tsrc.status,ComparisonStatus::NotApplicable);
    let mut p=DeclarationPolicy::default();p.levels.insert(11,Severity::Error);
    let r=a.observe(fields(7,7,7,0),Some(&q),None,&p).unwrap();
    let change=r.findings.iter().find(|f|f.reason==DeclarationIssue::SourceChanged).unwrap();
    assert_eq!(change.finding.severity(),Severity::Error);
    assert!(change.finding.evidence().related().iter().any(|l|l.packet==Some(1)));
    assert!(change.finding.evidence().raw_fields().iter().any(|v|v.name=="packet.previous_source"&&v.observed==RawValue::Unsigned(1)));
    let q=word(1,Some(PacketEdition::V24R1));
    let r=a.observe(PacketFields{key:timeline::PacketKey{session:2,..key(8,0)},..fields(8,0,7,1)},Some(&q),None,&Default::default()).unwrap();
    assert!(!r.findings.iter().any(|f|f.reason==DeclarationIssue::SourceChanged));
    assert!(a.finish(&Default::default()).unwrap().is_empty());
    // Session1 seeing7 does not discharge session2's same-counter declaration.
    let mut a=DeclarationAudit::new(&d);
    a.observe(fields(1,0,7,1),Some(&q),None,&Default::default()).unwrap();
    let missing=a.finish(&Default::default()).unwrap();
    assert!(missing.iter().any(|f|f.finding.evidence().location().session==Some(2)&&f.basis.channel.as_ref().unwrap().id==7));
}

#[test]
fn absent_non_timein_missing_fields_and_qualified_overrides_do_not_guess() {
    for d in [Declarations::absent(),Declarations::present(vec![setup(0,None,vec![ChannelDeclaration{cdt:Some("PCM".into()),..ch(7,Some("E"),None)}])]).unwrap()] {
        let q=word(0,Some(PacketEdition::V24R1));
        let p=DeclarationPolicy{use_tfmt_default:true,..Default::default()};
        let r=d.compare_qualified(fields(1,0,7,0),Some(&q),None,&p).unwrap();
        assert_eq!(r.tsrc.status,ComparisonStatus::NotApplicable);assert!(!codes(&r).contains(&11));
        assert!(!r.basic.tfmt.default_derived);assert_eq!(r.basic.tfmt.published_default,None);
        let mut a=DeclarationAudit::new(&d);
        let r=a.observe(PacketFields{data_type:0x19,..fields(1,0,7,0)},None,None,&p).unwrap();
        assert!(r.activity.is_empty());assert!(a.finish(&p).unwrap().is_empty());
    }
    let mut c=ch(7,None,None);c.ttf=None;
    let d=Declarations::present(vec![setup(0,None,vec![c])]).unwrap();
    let r=d.compare_qualified(fields(1,0,7,0),None,None,&Default::default()).unwrap();
    assert_eq!(r.basic.ttf.status,ComparisonStatus::MissingDeclaration);
    assert!(codes(&r).contains(&10));
    let q=TimeF1Csdw::from_raw(0).qualify(resolve_edition(EditionEvidence {
        override_edition:Some(PacketEdition::V24R1),tmats_edition:Some("04".into()),use_tmats:true,data_type_version:Some(1),..Default::default()
    }));
    let d=ds(Some("E"),None);let r=d.compare_qualified(fields(1,0,7,0),Some(&q),None,&Default::default()).unwrap();
    assert_eq!(r.tsrc.status,ComparisonStatus::Mismatch);assert!(codes(&r).contains(&11));
    assert_eq!(r.csdw.unwrap().resolution().basis(),EditionBasis::Override);
}
