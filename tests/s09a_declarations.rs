//! S09a: Table9-4 (9-50/9-72), Ch9 definitions9-5/9-6;
//! Ch11 Table11-4 and §11.2.3.2/3; RCC124-22 Ch7,7-3/PDF149.
//! Plain caller facts; no TMATS parser/encoder oracle.
use irig106_time::{declarations::*, findings::*, stream::*, timeline::*, *};
fn channel(ttf: Option<&str>, tfmt: Option<&str>) -> ChannelDeclaration {
    ChannelDeclaration {
        id: 7,
        cdt: Some("TIMEIN".into()),
        ttf: ttf.map(Into::into),
        tfmt: tfmt.map(Into::into),
        tsrc: Some("E".into()),
        shtf: Some("0".into()),
    }
}
fn setup(start: i128, end: Option<i128>, ch: ChannelDeclaration) -> SetupDeclarations {
    SetupDeclarations {
        range: GoverningRange::new(1, SignedRtcTicks::new(start), end.map(SignedRtcTicks::new))
            .unwrap(),
        setup_packet: 100,
        provenance: "caller:setup-one".into(),
        declared_version: Some("106-24R1".into()),
        ri4: Some("2016-12-31".into()),
        recording_year: Some(2016),
        channels: vec![ch],
    }
}
fn key(ticks: i128) -> PacketKey {
    PacketKey {
        session: 1,
        ticks,
        packet: 42,
    }
}
fn facts(data_type: u8, format: Option<u8>) -> PacketFields {
    PacketFields {
        key: key(5),
        channel: 7,
        data_type,
        format,
        source: Some(0),
    }
}
fn compare(
    ttf: Option<&str>,
    tfmt: Option<&str>,
    data_type: u8,
    format: u8,
    defaults: bool,
) -> ComparisonReport {
    Declarations::present(vec![setup(0, None, channel(ttf, tfmt))])
        .unwrap()
        .compare(
            facts(data_type, Some(format)),
            &DeclarationPolicy {
                use_tfmt_default: defaults,
                ..Default::default()
            },
        )
        .unwrap()
}
/// Requirements: L1-DECL-001, L2-DECL-101, L3-DECL-101
#[test]
fn governing_ranges_keep_all_plain_facts_and_exact_counter_boundary() {
    let first = setup(0, Some(10), channel(Some("1"), Some(" B ")));
    let mut next = setup(10, None, channel(Some("2"), Some("2")));
    next.setup_packet = 101;
    next.provenance = "caller:setup-two".into();
    next.ri4 = Some("unparsed original date".into());
    next.recording_year = None;
    next.declared_version = Some("unknown version spelling".into());
    next.channels.push(ChannelDeclaration {
        id: 8,
        ..Default::default()
    });
    let ds = Declarations::present(vec![next.clone(), first.clone()]).unwrap();
    for (ticks, expected) in [(0, &first), (9, &first), (10, &next), (999, &next)] {
        let b = ds.basis(key(ticks), 7).unwrap();
        assert_eq!(b.presence, DeclarationPresence::Present);
        let g = b.governing.unwrap();
        assert_eq!(g.range, expected.range);
        assert_eq!(g.setup_packet, expected.setup_packet);
        assert_eq!(g.provenance, expected.provenance);
        assert_eq!(g.declared_version, expected.declared_version);
        assert_eq!(g.ri4, expected.ri4);
        assert_eq!(g.recording_year, expected.recording_year);
        assert_eq!(b.channel, Some(expected.channels[0].clone()));
    }
    assert_eq!(
        ds.basis(key(-1), 7).unwrap().presence,
        DeclarationPresence::NoGoverningRange
    );
    assert!(ds.basis(key(11), 8).unwrap().channel.is_some());
    assert!(ds.basis(key(11), 9).unwrap().channel.is_none());
    assert_eq!(
        ds.basis(
            PacketKey {
                session: 2,
                ..key(10)
            },
            7
        )
        .unwrap()
        .presence,
        DeclarationPresence::NoGoverningRange
    );
    assert!(GoverningRange::new(0, SignedRtcTicks::new(0), None).is_err());
    assert!(GoverningRange::new(1, SignedRtcTicks::new(1), Some(SignedRtcTicks::new(1))).is_err());
    assert!(
        Declarations::present(vec![first.clone(), setup(9, None, channel(None, None))]).is_err()
    );
    let mut duplicate = first.clone();
    duplicate.channels.push(duplicate.channels[0].clone());
    assert!(Declarations::present(vec![duplicate]).is_err());
    for ticks in [9, 10] {
        let original = PacketObservation {
            rtc: Rtc::from_raw(ticks),
            ..time_packet()
        };
        let prepared = ds
            .prepare(original.clone(), key(i128::from(ticks)))
            .unwrap();
        assert_eq!(prepared.observation.rtc, original.rtc);
        assert_eq!(
            prepared.observation.reference.as_ref().unwrap().value,
            original.reference.unwrap().value
        );
        assert_eq!(
            prepared
                .observation
                .reference
                .as_ref()
                .unwrap()
                .year_basis
                .context()
                .recording,
            if ticks == 9 { Some(2016) } else { None }
        );
    }
    let carried = ds.prepare(time_packet(), key(9)).unwrap().observation;
    let updated = ds.prepare(carried, key(10)).unwrap();
    assert_eq!(
        updated
            .observation
            .reference
            .unwrap()
            .year_basis
            .context()
            .recording,
        None
    );
    assert_eq!(ds.snapshots().unwrap()[0], first);
    assert_eq!(ds.snapshots().unwrap()[1], next);
    two_timein_channels_select_independent_declarations_and_basis();
}
fn two_timein_channels_select_independent_declarations_and_basis() {
    let first = channel(Some("1"), Some("B"));
    let mut second = channel(Some("2"), Some("2"));
    second.id = 8;
    second.tsrc = Some("I".into());
    second.shtf = Some("2".into());
    let mut snapshot = setup(0, None, first.clone());
    snapshot.channels.push(second.clone());
    // G\OD is NOT an API input. This opaque caller provenance mentions a
    // different origination date to ensure no scanning/fallback into RI4.
    snapshot.provenance = "caller: RI4=2016-12-31; G\\OD=2025-01-01".into();
    let ds = Declarations::present(vec![snapshot.clone()]).unwrap();
    for (ch, data_type, fmt, reading) in [
        (&first, 0x11, 0, Reading::Rdg002),
        (&second, 0x12, 2, Reading::Rdg004),
    ] {
        let packet = PacketFields {
            channel: ch.id,
            ..facts(data_type, Some(fmt))
        };
        let r = ds.compare(packet, &Default::default()).unwrap();
        assert_eq!(r.ttf.status, ComparisonStatus::Match);
        assert_eq!(r.tfmt.status, ComparisonStatus::Match);
        assert_eq!(r.ttf.expected, Some(data_type));
        assert_eq!(r.tfmt.expected, Some(fmt));
        assert_eq!(r.tfmt.reading, reading);
        assert_eq!(r.basis.channel.as_ref(), Some(ch));
        let governing = r.basis.governing.unwrap();
        assert_eq!(governing.setup_packet, snapshot.setup_packet);
        assert_eq!(governing.provenance, snapshot.provenance);
        assert_eq!(governing.ri4.as_deref(), Some("2016-12-31"));
        assert_eq!(governing.recording_year, Some(2016));
        // Swap the wire facts: each channel still uses its OWN TTF/TFMT,
        // never the other TIMEIN channel in the same setup.
        let wrong = ds
            .compare(
                PacketFields {
                    data_type: if data_type == 0x11 { 0x12 } else { 0x11 },
                    ..packet
                },
                &Default::default(),
            )
            .unwrap();
        assert_eq!(wrong.ttf.status, ComparisonStatus::Mismatch);
        assert_eq!(wrong.ttf.expected, Some(data_type));
        assert_eq!(wrong.basis.channel.as_ref(), Some(ch));
        let prepared = ds
            .prepare(
                PacketObservation {
                    channel: ch.id,
                    ..time_packet()
                },
                key(5),
            )
            .unwrap();
        assert!(prepared.observation.declared_timein);
        assert_eq!(prepared.observation.declared_external, ch.id == 7);
        assert_eq!(prepared.declarations.channel.as_ref(), Some(ch));
        assert_eq!(
            prepared
                .observation
                .reference
                .unwrap()
                .year_basis
                .context()
                .recording,
            Some(2016)
        );
    }
    snapshot.ri4 = None;
    snapshot.recording_year = None;
    let ds = Declarations::present(vec![snapshot]).unwrap();
    let prepared = ds.prepare(time_packet(), key(5)).unwrap();
    assert_eq!(prepared.declarations.governing.unwrap().ri4, None);
    assert_eq!(
        prepared
            .observation
            .reference
            .unwrap()
            .year_basis
            .context()
            .recording,
        None
    );
}
/// Requirements: L2-RDG-201, L3-RDG-201
/// Interpretations: RDG-001
#[test]
fn every_ttf_value_packet_type_missing_unknown_and_range_context_is_reported() {
    for (ttf, expected) in [("1", 0x11), ("2", 0x12)] {
        for observed in 0..=255 {
            let r = compare(Some(ttf), Some("B"), observed, 0, false);
            assert_eq!(
                r.ttf.status,
                if observed == expected {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert_eq!(r.ttf.expected, Some(expected));
            assert_eq!(r.ttf.observed, Some(observed));
            assert_eq!(r.ttf.reading, Reading::Rdg001);
            assert_eq!(
                r.findings.iter().any(|f| f.field == DeclarationField::Ttf),
                observed != expected
            );
        }
    }
    for (ttf, state) in [
        (None, ComparisonStatus::MissingDeclaration),
        (Some("0"), ComparisonStatus::ReservedDeclaration),
        (Some("3"), ComparisonStatus::UnknownDeclaration),
        (Some("01"), ComparisonStatus::UnknownDeclaration),
    ] {
        let r = compare(ttf, Some("B"), 0x11, 0, false);
        assert_eq!(r.ttf.status, state);
        assert_eq!(r.ttf.raw, ttf.map(Into::into));
        assert!(r.findings.iter().any(|f| f.field == DeclarationField::Ttf));
    }
    let mut ch = channel(Some("1"), Some("B"));
    ch.cdt = Some("PCM".into());
    let ds = Declarations::present(vec![
        setup(0, Some(10), ch),
        setup(10, None, channel(Some("2"), Some("0"))),
    ])
    .unwrap();
    assert_eq!(
        ds.compare(facts(0x11, Some(0)), &Default::default())
            .unwrap()
            .ttf
            .status,
        ComparisonStatus::NotApplicable
    );
    assert!(
        !ds.prepare(time_packet(), key(5))
            .unwrap()
            .observation
            .declared_external
    );
    let r = ds
        .compare(
            PacketFields {
                key: key(10),
                ..facts(0x12, Some(0))
            },
            &Default::default(),
        )
        .unwrap();
    assert_eq!(r.ttf.status, ComparisonStatus::Match);
    assert_eq!(r.basis.governing.unwrap().range.start().get(), 10);
}
fn all_format1_pairs() {
    for (letter, fmt) in [
        ("B", 0),
        ("A", 1),
        ("G", 2),
        ("I", 3),
        ("U", 4),
        ("N", 5),
        ("X", 15),
    ] {
        let m = format1_mapping(letter).unwrap();
        assert_eq!(m.wire, fmt);
        assert_eq!(m.inferred, letter == "I");
        let reverse = format1_declaration(fmt).unwrap();
        assert_eq!(reverse.declared, letter);
        assert_eq!(reverse.inferred, letter == "I");
        assert_eq!(reverse.reading, m.reading);
        assert_eq!(
            m.reading,
            if letter == "I" {
                Reading::Rdg003
            } else {
                Reading::Rdg002
            }
        );
        for observed in 0..16 {
            let r = compare(Some("1"), Some(letter), 0x11, observed, false);
            assert_eq!(
                r.tfmt.status,
                if fmt == observed {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            // §11.2.3.2a: 0..5 are defined, 6..14 reserved, 15 is NONE.
            // Assert the negative cases too: X/NONE must never become TF-006.
            assert_eq!(
                r.findings.iter().any(|f| f.finding.code().number() == 6),
                matches!(observed, 6..=14),
                "FMT={observed:#x}, TFMT={letter}"
            );
            assert_eq!(r.tfmt.inferred, letter == "I");
            assert_eq!(
                r.tfmt.reading,
                if letter == "I" {
                    Reading::Rdg003
                } else {
                    Reading::Rdg002
                }
            );
            if fmt != observed {
                let f = r
                    .findings
                    .iter()
                    .find(|f| f.finding.code().number() == 10 && f.field == DeclarationField::Tfmt)
                    .unwrap();
                assert_eq!(f.finding.severity(), Severity::Warning);
                assert_eq!(f.comparison.inferred, letter == "I");
            }
        }
    }
    for fmt in 6..15 {
        assert_eq!(format1_declaration(fmt), None);
    }
    for text in ["", "a", " B ", "3", "A123", "?"] {
        assert!(format1_mapping(text).is_none());
        assert_eq!(
            compare(Some("1"), Some(text), 0x11, 0, false).tfmt.status,
            ComparisonStatus::UnknownDeclaration
        );
    }
}
/// Requirements: L2-RDG-202, L3-RDG-202
/// Interpretations: RDG-002
#[test]
fn format1_pairs_missing_required_defaults_and_absence_are_distinct() {
    all_format1_pairs();
    for defaults in [false, true] {
        for observed in [0, 1] {
            let ds = Declarations::present(vec![setup(0, None, channel(Some("1"), None))]).unwrap();
            let before = ds.snapshots().unwrap().to_vec();
            let r = ds
                .compare(
                    facts(0x11, Some(observed)),
                    &DeclarationPolicy {
                        use_tfmt_default: defaults,
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_eq!(r.tfmt.raw, None);
            assert_eq!(r.tfmt.published_default, Some("A"));
            assert_eq!(r.tfmt.default_derived, defaults);
            assert_eq!(r.tfmt.effective, defaults.then(|| "A".into()));
            assert_eq!(r.tfmt.expected, defaults.then_some(1));
            assert_eq!(
                r.tfmt.status,
                if !defaults {
                    ComparisonStatus::MissingDeclaration
                } else if observed == 1 {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert!(
                r.findings
                    .iter()
                    .any(|f| f.reason == DeclarationIssue::MissingRequired
                        && f.field == DeclarationField::Tfmt)
            );
            assert_eq!(
                r.findings
                    .iter()
                    .any(|f| f.reason == DeclarationIssue::Mismatch),
                defaults && observed != 1
            );
            assert_eq!(ds.snapshots().unwrap(), before);
            assert_eq!(r.basis.channel.unwrap().tfmt, None);
        }
    }
    for text in ["", "a", "?"] {
        let r = compare(Some("1"), Some(text), 0x11, 0, true);
        assert_eq!(r.tfmt.effective.as_deref(), Some(text));
        assert!(!r.tfmt.default_derived);
        assert_eq!(r.tfmt.status, ComparisonStatus::UnknownDeclaration);
        assert!(
            !r.findings
                .iter()
                .any(|f| f.reason == DeclarationIssue::MissingRequired)
        );
    }
    let r = compare(Some("1"), Some("A"), 0x11, 1, true);
    assert!(!r.tfmt.default_derived);
    assert_eq!(r.tfmt.effective, Some("A".into()));
    assert!(r.findings.is_empty());
    let mut ch = channel(None, None);
    ch.cdt = None;
    let r = Declarations::present(vec![setup(0, None, ch)])
        .unwrap()
        .compare(facts(0x11, Some(0)), &Default::default())
        .unwrap();
    assert_eq!(r.tfmt.status, ComparisonStatus::NotApplicable);
    assert!(r.findings.is_empty());
}
/// Requirements: L2-RDG-203, L3-RDG-203
/// Interpretations: RDG-002, RDG-003
#[test]
fn all_seven_bidirectional_pairs_and_rcc124_chapter7_example_keep_inference() {
    all_format1_pairs();
    // RCC124-22 Ch7 printed7-3/PDF149: R-1\TTF-1:1; R-1\TFMT-1:I;
    // R-1\TSRC-1:I; "Time format is internal" / "Time source is internal".
    // Plain declaration values, not parsed here. CSDW0x30 -> FMT3/SRC0.
    let mut ch = channel(Some("1"), Some("I"));
    ch.tsrc = Some("I".into());
    let ds = Declarations::present(vec![setup(0, None, ch)]).unwrap();
    for (raw, status) in [
        (0x30u32, ComparisonStatus::Match),
        (0x00, ComparisonStatus::Mismatch),
        (0x31, ComparisonStatus::Match),
    ] {
        let r = ds
            .compare(
                PacketFields {
                    format: Some(((raw >> 4) & 15) as u8),
                    source: Some((raw & 15) as u8),
                    ..facts(0x11, Some(3))
                },
                &Default::default(),
            )
            .unwrap();
        assert_eq!(r.tfmt.status, status);
        assert!(r.tfmt.inferred);
        assert_eq!(r.tfmt.reading, Reading::Rdg003);
        assert_eq!(r.packet.source, Some((raw & 15) as u8));
        assert_eq!(r.basis.channel.unwrap().tsrc, Some("I".into()));
    }
}
/// Requirements: L2-RDG-204, L3-RDG-204
/// Interpretations: RDG-004
#[test]
fn numeric_format2_context_never_uses_letter_or_ntp_fallbacks() {
    for n in 0..3 {
        for observed in 0..16 {
            let r = compare(Some("2"), Some(&n.to_string()), 0x12, observed, false);
            assert_eq!(r.tfmt.reading, Reading::Rdg004);
            assert!(!r.tfmt.inferred);
            assert_eq!(
                r.tfmt.status,
                if n == observed {
                    ComparisonStatus::Match
                } else {
                    ComparisonStatus::Mismatch
                }
            );
            assert_eq!(
                r.findings.iter().any(|f| f.finding.code().number() == 6),
                observed >= 3
            );
        }
    }
    for text in ["I", "B", "A", "3", "00", ""] {
        assert_eq!(
            compare(Some("2"), Some(text), 0x12, 0, false).tfmt.status,
            ComparisonStatus::UnknownDeclaration
        );
    }
    assert_eq!(
        compare(Some("1"), Some("0"), 0x12, 0, false).tfmt.status,
        ComparisonStatus::ContextMismatch
    );
    assert_eq!(
        compare(None, Some("0"), 0x12, 0, false).tfmt.status,
        ComparisonStatus::ContextMismatch
    );
    let wrong = compare(Some("1"), Some("0"), 0x12, 0, false);
    assert_eq!(wrong.tfmt.expected, None);
    assert_eq!(wrong.tfmt.observed, Some(0));
    let r = compare(Some("2"), None, 0x12, 0, true);
    assert!(r.tfmt.default_derived);
    assert_eq!(r.tfmt.effective, Some("A".into()));
    assert_eq!(r.tfmt.expected, None);
    assert_eq!(r.tfmt.status, ComparisonStatus::UnknownDeclaration);
    assert!(
        r.findings
            .iter()
            .any(|f| f.reason == DeclarationIssue::MissingRequired)
    );
}
#[test]
fn each_finding_keeps_governing_raw_comparison_and_severity_basis() {
    for (r, id) in [
        (Reading::Rdg001, "RDG-001"),
        (Reading::Rdg002, "RDG-002"),
        (Reading::Rdg003, "RDG-003"),
        (Reading::Rdg004, "RDG-004"),
    ] {
        assert_eq!(r.id(), id);
    }
    let mut p = DeclarationPolicy::default();
    p.levels.insert(10, Severity::Information);
    let ds = Declarations::present(vec![setup(0, None, channel(Some("1"), Some("I")))]).unwrap();
    let r = ds.compare(facts(0x11, Some(0)), &p).unwrap();
    let f = &r.findings[0];
    assert_eq!(f.finding.severity(), Severity::Information);
    assert_eq!(f.finding.evidence().validity(), ValueValidity::Undetermined);
    assert_eq!(f.finding.evidence().location().packet, Some(42));
    assert_eq!(f.finding.evidence().location().channel, Some(7));
    assert_eq!(f.basis.governing.as_ref().unwrap().setup_packet, 100);
    assert_eq!(f.comparison.raw.as_deref(), Some("I"));
    assert_eq!(f.comparison.expected, Some(3));
    assert_eq!(f.comparison.observed, Some(0));
    assert!(f.comparison.inferred);
    assert_eq!(f.comparison.reading.id(), "RDG-003");
    assert_eq!(f.finding.evidence().related()[0].packet, Some(100));
    assert_eq!(f.finding.evidence().related()[0].channel, None);
    assert!(
        f.finding
            .evidence()
            .raw_fields()
            .iter()
            .any(|v| v.name == "packet.rtc_ticks_unwrapped" && v.observed == RawValue::Signed(5))
    );
    p.levels.insert(6, Severity::Information);
    let ds_missing = Declarations::present(vec![setup(0, None, channel(Some("1"), None))]).unwrap();
    let r = ds_missing
        .compare(
            facts(0x11, Some(6)),
            &DeclarationPolicy {
                use_tfmt_default: true,
                ..p.clone()
            },
        )
        .unwrap();
    assert_eq!(r.findings.len(), 3);
    assert!(
        r.findings
            .iter()
            .all(|f| f.finding.severity() == Severity::Information)
    );
    assert!(
        r.findings
            .iter()
            .all(|f| f.finding.evidence().validity() == ValueValidity::Undetermined)
    );

    assert_eq!(
        f.finding.evidence().raw_fields()[0].observed,
        RawValue::Unsigned(0)
    );
    assert_eq!(
        f.finding.evidence().raw_fields()[0].expected,
        Some(RawValue::Unsigned(3))
    );
    assert_eq!(
        ds.compare(facts(0x11, None), &Default::default())
            .unwrap()
            .tfmt
            .status,
        ComparisonStatus::UnavailablePacketField
    );
    assert!(
        ds.compare(facts(0x11, Some(16)), &Default::default())
            .is_err()
    );
    assert!(
        ds.prepare(
            time_packet(),
            PacketKey {
                packet: 99,
                ..key(5)
            }
        )
        .is_err()
    );
    assert!(
        Declarations::absent()
            .basis(
                PacketKey {
                    session: 0,
                    ..key(5)
                },
                7
            )
            .is_err()
    );
}
fn time_packet() -> PacketObservation {
    let b = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0];
    PacketObservation {
        id: 42,
        channel: 7,
        rtc: Rtc::from_raw(5),
        kind: PacketKind::Time,
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference: Some(
            ReferenceInput::format1(
                &b,
                10,
                TimeF1Layout::SourceWithIts17,
                ScalePolicy::default(),
            )
            .unwrap(),
        ),
    }
}
// Partial DECL-102: all legacy answer/report propagation belongs to S10b/S12.
#[test]
fn absent_empty_gap_and_undeclared_policy_work_in_both_engines_without_counter_reset() {
    let absent = Declarations::absent();
    let empty = Declarations::present(vec![setup(
        0,
        None,
        ChannelDeclaration {
            id: 8,
            ..Default::default()
        },
    )])
    .unwrap();
    let mut empty_setup = setup(0, None, channel(None, None));
    empty_setup.channels.clear();
    let present_empty = Declarations::present(vec![empty_setup]).unwrap();
    assert_eq!(
        Declarations::present(vec![])
            .unwrap()
            .basis(key(5), 7)
            .unwrap()
            .presence,
        DeclarationPresence::NoGoverningRange
    );
    assert_eq!(
        absent.basis(key(5), 7).unwrap().presence,
        DeclarationPresence::Absent
    );
    assert_eq!(
        present_empty.basis(key(5), 7).unwrap().presence,
        DeclarationPresence::Present
    );
    assert!(present_empty.basis(key(5), 7).unwrap().channel.is_none());
    for declarations in [absent, empty, present_empty] {
        for include in [false, true] {
            let prepared = declarations.prepare(time_packet(), key(5)).unwrap();
            assert!(!prepared.observation.declared_timein);
            assert!(!prepared.observation.declared_external);
            assert_eq!(prepared.observation.rtc, Rtc::from_raw(5));
            let op = OfflinePolicy {
                include_undeclared: include,
                ..Default::default()
            };
            let mut b = TimelineBuilder::new(op.clone()).unwrap();
            let k = b.observe(prepared.observation.clone(), false).unwrap();
            let t = b.finish().unwrap();
            let a = t.lookup(k).unwrap();
            assert_eq!(a.time.is_some(), include);
            if include {
                assert!(!a.basis.reference.unwrap().declared_timein);
            }
            for mode in [StreamMode::Standard, StreamMode::Interpolate] {
                let mut p = StreamPolicy::new(
                    1,
                    BufferCaps {
                        packets: 2,
                        bytes: 200,
                        references: 2,
                    },
                );
                p.mode = mode;
                p.offline = op.clone();
                p.offline.channel = Some(7);
                let mut s = StreamTimeline::new(p, None).unwrap();
                s.observe(prepared.observation.clone(), 0, 100, false)
                    .unwrap();
                let a = s
                    .observe(
                        PacketObservation {
                            id: 43,
                            kind: PacketKind::Data,
                            reference: None,
                            ..prepared.observation.clone()
                        },
                        0,
                        100,
                        false,
                    )
                    .unwrap()
                    .answers
                    .remove(0);
                assert_eq!(a.time.is_some(), include);
            }
        }
    }
    let ds = Declarations::present(vec![setup(0, None, channel(Some("1"), Some("B")))]).unwrap();
    let before = time_packet();
    let after = ds.prepare(before.clone(), key(5)).unwrap();
    assert!(after.observation.declared_timein);
    assert!(after.observation.declared_external);
    assert_eq!(
        after
            .observation
            .reference
            .as_ref()
            .unwrap()
            .year_basis
            .context()
            .recording,
        Some(2016)
    );
    assert_eq!(
        after.observation.reference.as_ref().unwrap().value,
        before.reference.unwrap().value
    );
}

// Partial applicability/validation clauses only; no additional whole marker.
#[test]
fn non_timein_channels_never_receive_tfmt_defaults_or_reserved_findings() {
    for cdt in [None, Some("PCM"), Some("timein")] {
        let mut ch = channel(Some("1"), None);
        ch.cdt = cdt.map(Into::into);
        let ds = Declarations::present(vec![setup(0, None, ch)]).unwrap();
        for data_type in [0x11, 0x12] {
            for fmt in 0..=15 {
                let r = ds
                    .compare(
                        facts(data_type, Some(fmt)),
                        &DeclarationPolicy {
                            use_tfmt_default: true,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                assert_eq!(r.tfmt.status, ComparisonStatus::NotApplicable);
                assert!(!r.tfmt.default_derived);
                assert_eq!(r.tfmt.effective, None);
                assert_eq!(r.tfmt.published_default, None);
                assert!(!r.findings.iter().any(|f| f.finding.code().number() == 6));
                assert!(r.findings.is_empty());
            }
        }
    }
}
#[test]
fn recording_year_validation_accepts_boundary_and_rejects_out_of_domain() {
    for year in [0, 9999, 10000, u16::MAX] {
        let mut s = setup(0, None, channel(None, None));
        s.recording_year = Some(year);
        assert_eq!(
            Declarations::present(vec![s]).is_ok(),
            year <= 9999,
            "year={year}"
        );
    }
}
