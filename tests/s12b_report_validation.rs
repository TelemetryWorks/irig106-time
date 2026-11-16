#![cfg(feature = "serde")]
//! Full-stack corruption/resource audit; schema review is still an owner step.
#[path = "support/replay_fixtures.rs"]
mod fixtures;
#[path = "support/serde_tree.rs"]
mod serde_tree;
use fixtures::*;
use irig106_time::{snapshot::*, *};
use serde_tree::{Node, decode, encode};

fn path<'a>(mut v: &'a mut Node, keys: &[&str]) -> &'a mut Node {
    for key in keys {
        v = match v {
            Node::Seq(xs) => &mut xs[key.parse::<usize>().unwrap()],
            _ => v.field(key),
        }
    }
    v
}
#[test]
fn every_projection_field_is_checked_against_replay_not_just_calendar() {
    let wire = encode(&ReportSnapshot::new(plan()).unwrap());
    for (keys, bad) in [
        (
            vec!["answers", "0", "value", "value", "counter_duration"],
            Node::String("999".into()),
        ),
        (
            vec![
                "answers",
                "0",
                "value",
                "basis",
                "Counter",
                "clock",
                "Offline",
                "distance_ns",
            ],
            Node::String("999".into()),
        ),
        (
            vec![
                "answers",
                "0",
                "value",
                "basis",
                "Counter",
                "clock",
                "Offline",
                "session_origin",
                "session",
            ],
            Node::String("2".into()),
        ),
        (
            vec![
                "answers",
                "0",
                "value",
                "basis",
                "Counter",
                "clock",
                "Offline",
                "reference",
                "key",
                "packet",
            ],
            Node::String("999".into()),
        ),
        (
            vec![
                "answers",
                "0",
                "value",
                "basis",
                "Counter",
                "clock",
                "Offline",
                "reference",
                "input",
                "scale",
                "stated",
            ],
            Node::String("Tai".into()),
        ),
        (
            vec![
                "answers", "0", "value", "basis", "Counter", "clock", "Offline", "policy",
                "late_ns",
            ],
            Node::String("1".into()),
        ),
    ] {
        let mut w = wire.clone();
        *path(w.field("outputs"), &keys) = bad;
        assert!(decode::<ReportSnapshot>(w).is_err(), "{keys:?}");
    }
    let mut w = wire.clone();
    *path(
        w.field("outputs"),
        &[
            "answers", "0", "value", "basis", "Counter", "clock", "Offline", "key", "packet",
        ],
    ) = Node::Number("2".into());
    assert!(decode::<ReportSnapshot>(w).is_err());
    let mut w = wire.clone();
    let Node::Map(fields) = w.field("outputs") else {
        panic!()
    };
    fields.push(("extra".into(), Node::Null));
    assert!(decode::<ReportSnapshot>(w).is_err());
    let mut w = wire;
    let Node::Map(fields) = w.field("outputs") else {
        panic!()
    };
    fields.push(fields[0].clone());
    assert!(decode::<ReportSnapshot>(w).is_err());
}
#[test]
fn resource_caps_links_and_selected_answer_bounds_reject() {
    for key in ["steps", "answers", "raw_bytes"] {
        let wire = encode(&ReportSnapshot::new(plan()).unwrap());
        let mut w = wire;
        *path(w.field("plan"), &["limits", key]) = Node::String("0".into());
        assert!(decode::<ReportSnapshot>(w).is_err(), "{key}");
    }
    let r = ReportSnapshot::new(plan()).unwrap();
    assert!(AnswerSnapshot::new(r.clone(), 1).is_err());
    let mut w = encode(&AnswerSnapshot::new(r, 0).unwrap());
    *w.field("value").field("index") = Node::String("1".into());
    assert!(decode::<AnswerSnapshot>(w).is_err());
    let mut p = plan();
    p.requests.push(ReportRequest::Own {
        counter: d(99),
        origin: origin(),
        data_type: 0x21,
        rtc: d(10),
        flags: 0,
        raw: vec![],
        applicability: secondary_packet::TimeApplicability::FirstDataBit,
        context: context(1),
    });
    assert!(ReportSnapshot::new(p).is_err());
    let mut p = plan();
    p.requests.push(ReportRequest::Stamp {
        counter: Some(d(99)),
        origin: origin(),
        query_id: d(2),
        flags: 0xc4,
        raw: vec![0; 8],
        order: rtc::CounterOrder::Nearest,
        context: context(1),
    });
    assert!(ReportSnapshot::new(p).is_err());
    let mut p = plan();
    if let ReportRequest::Counter { key } = &mut p.requests[0] {
        key.session = d(0);
    }
    assert!(ReportSnapshot::new(p).is_err());
    let mut p = plan();
    if let ClockRecipe::Offline { packets, .. } = &mut p.clock {
        packets[0].packet.rtc = d(1u64 << 48);
    }
    assert!(ReportSnapshot::new(p).is_err());
}
#[test]
fn report_tree_depth_and_unknown_input_fields_are_rejected() {
    let wire = encode(&ReportSnapshot::new(plan()).unwrap());
    let mut w = wire.clone();
    let mut nested = Node::Null;
    for _ in 0..130 {
        nested = Node::Seq(vec![nested]);
    }
    *w.field("outputs") = nested;
    assert!(
        decode::<ReportSnapshot>(w)
            .unwrap_err()
            .to_string()
            .contains("nesting depth")
    );
    for keys in [
        vec![],
        vec!["plan"],
        vec!["plan", "limits"],
        vec!["plan", "clock", "Offline"],
        vec!["plan", "clock", "Offline", "packets", "0"],
        vec!["plan", "clock", "Offline", "packets", "0", "packet"],
        vec![
            "plan", "clock", "Offline", "packets", "0", "packet", "body", "Format1",
        ],
    ] {
        let mut w = wire.clone();
        let Node::Map(fields) = path(&mut w, &keys) else {
            panic!()
        };
        fields.push(("extra".into(), Node::Null));
        assert!(decode::<ReportSnapshot>(w).is_err(), "{keys:?}");
    }
    let mut w = wire;
    *w.field("units").field("counter") = Node::String("ns".into());
    assert!(decode::<ReportSnapshot>(w).is_err());
}
#[test]
fn leap_ambiguity_keeps_exact_ntp_fraction_and_finding_evidence() {
    use irig106_time::edition::PacketEdition;
    let mut p = plan();
    if let ClockRecipe::Offline { packets, .. } = &mut p.clock {
        let mut payload = vec![1, 0, 0, 0];
        payload.extend_from_slice(&3_692_217_600u32.to_le_bytes());
        payload.extend_from_slice(&0x80000000u32.to_le_bytes());
        packets.insert(
            1,
            packet(
                3,
                1,
                ReferenceSpec::Format2 {
                    payload,
                    data_length: d(12),
                    session: d(1),
                    byte_offset: Some(d(42)),
                    edition: Some(PacketEdition::V24),
                    data_type_version: 0x10,
                    context: context(1),
                    raw_network_rate: false,
                },
            ),
        );
        packets[2].packet.id = d(4);
    }
    let r = ReportSnapshot::new(p).unwrap();
    let f = r.findings().iter().find(|f| f.code == 30).unwrap();
    assert_eq!(f.evidence.ntp().unwrap().raw.fraction, 0x80000000);
    let loaded = decode::<ReportSnapshot>(encode(&r)).unwrap();
    assert_eq!(loaded.findings(), r.findings());
}

#[test]
fn timeout_eof_and_cap_release_reasons_survive_replay() {
    use irig106_time::{answer::*, stream::*};
    for expected in [
        ReleaseReason::Eof,
        ReleaseReason::Timeout,
        ReleaseReason::PacketCap,
    ] {
        let mut p = StreamPolicy::new(
            1,
            BufferCaps {
                packets: 1,
                bytes: 100,
                references: 3,
            },
        );
        p.mode = StreamMode::Interpolate;
        let mut steps = vec![
            StreamStep::Observe {
                record: packet(1, 0, calendar(0, 0)),
                progress: d(0),
                bytes: d(10),
            },
            StreamStep::Observe {
                record: packet(2, 2_000_000, ReferenceSpec::None),
                progress: d(1),
                bytes: d(10),
            },
        ];
        let end = match expected {
            ReleaseReason::Timeout => {
                steps.push(StreamStep::Advance {
                    progress: d(1_100_000_002),
                });
                1_100_000_003
            }
            ReleaseReason::PacketCap => {
                steps.push(StreamStep::Observe {
                    record: packet(3, 3_000_000, ReferenceSpec::None),
                    progress: d(2),
                    bytes: d(10),
                });
                3
            }
            _ => 2,
        };
        steps.push(StreamStep::Finish { progress: d(end) });
        let p = ReplayPlan {
            limits: limits(),
            clock: ClockRecipe::Stream {
                policy: PolicySnapshot::stream(p).unwrap(),
                inventory: Some(vec![StreamChannel {
                    id: 7,
                    timein: true,
                    external: true,
                }]),
                steps,
            },
            requests: vec![],
        };
        let r = ReportSnapshot::new(p).unwrap();
        let AnswerBasis::Counter(c) = r.answers()[0].view().basis() else {
            panic!()
        };
        let ClockBasis::Stream(b) = c.clock() else {
            panic!()
        };
        assert_eq!(b.release, expected);
        assert_eq!(
            b.progress_ns,
            if expected == ReleaseReason::Timeout {
                1_100_000_002
            } else {
                2
            }
        );
        assert_eq!(b.deadline_ns, Some(1_100_000_001));
        assert_eq!(
            r.answers()[0].view().value().calendar.unwrap().nanoseconds,
            200_000_000
        );
        assert_eq!(
            encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
            encode(&r)
        );
    }
}
#[test]
fn whole_report_preserves_unsigned_extrema_and_signed_counter_duration() {
    let mut p = plan();
    if let ClockRecipe::Offline { packets, .. } = &mut p.clock {
        packets[0].packet.id = d(u64::MAX - 1);
        packets[1].packet.id = d(u64::MAX);
    }
    if let ReportRequest::Counter { key } = &mut p.requests[0] {
        key.packet = d(u64::MAX);
    }
    let r = ReportSnapshot::new(p).unwrap();
    assert!(encode(&r).json().contains(r#""18446744073709551615""#));
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
    let p = ReplayPlan {
        limits: limits(),
        clock: ClockRecipe::Offline {
            policy: PolicySnapshot::offline(timeline::OfflinePolicy::default()).unwrap(),
            packets: vec![packet(1, 5, ReferenceSpec::None)],
            quality: vec![],
        },
        requests: vec![ReportRequest::Counter {
            key: ReplayKey {
                session: d(1),
                ticks: d(4),
                packet: d(2),
            },
        }],
    };
    let r = ReportSnapshot::new(p).unwrap();
    assert_eq!(
        r.answers()[0].view().value().counter_duration,
        Some(SignedNanos::new(-100))
    );
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
}
#[test]
fn positive_resource_ceilings_and_explicit_finish_are_enforced() {
    let mut p = plan();
    p.limits.steps = d(1);
    assert!(ReportSnapshot::new(p).is_err());
    let mut p = plan();
    p.limits.raw_bytes = d(1);
    assert!(ReportSnapshot::new(p).is_err());
    let mut p = plan();
    p.limits.answers = d(1);
    p.requests.push(p.requests[0].clone());
    assert!(ReportSnapshot::new(p).is_err());
    use irig106_time::stream::*;
    let p = ReplayPlan {
        limits: limits(),
        clock: ClockRecipe::Stream {
            policy: PolicySnapshot::stream(StreamPolicy::new(
                1,
                BufferCaps {
                    packets: 1,
                    bytes: 100,
                    references: 2,
                },
            ))
            .unwrap(),
            inventory: None,
            steps: vec![],
        },
        requests: vec![],
    };
    assert!(ReportSnapshot::new(p).is_err());
}

#[test]
fn policy_dependent_answer_headers_name_the_defaults_version() {
    let r = ReportSnapshot::new(plan()).unwrap();
    let mut output = encode(&r.answers()[0]);
    assert_eq!(
        output.field("units").json(),
        r#"{"bytes":"byte","calendar":"year/day/h/min/s/ns","counter":"rtc_ticks","density":"reference/s","drift":"ppm","duration":"ns","identity":"count","ntp_fraction":"2^-32 s","tai":"tai_ns","unix":"unix_s"}"#
    );
    let Node::Map(fields) = &output else { panic!() };
    assert!(
        fields
            .iter()
            .any(|(k, v)| k == "defaults_version" && *v == Node::Number("1".into()))
    );
    let mut answer = encode(&AnswerSnapshot::new(r, 0).unwrap());
    assert_eq!(answer.field("defaults_version"), &Node::Number("1".into()));
    *answer.field("defaults_version") = Node::Number("2".into());
    assert!(decode::<AnswerSnapshot>(answer).is_err());
}

#[test]
fn generic_scaled_shared_quantities_are_exact_too() {
    let value = scale::Scaled {
        value: u64::MAX,
        scale: scale::ScaleEvidence::unknown(),
    };
    let mut w = encode(&value);
    assert_eq!(w.field("value"), &Node::String(u64::MAX.to_string()));
    let value = scale::Scaled {
        value: SignedNanos::new(i128::MIN),
        scale: scale::ScaleEvidence::unknown(),
    };
    let mut w = encode(&value);
    assert_eq!(w.field("value"), &Node::String(i128::MIN.to_string()));
}

#[test]
fn explicit_stream_session_boundaries_are_retained() {
    use irig106_time::{answer::*, stream::*};
    let mut later = calendar(0, 0);
    if let ReferenceSpec::Calendar { value, .. } = &mut later {
        let mut c = value.value();
        c.year = Some(2018);
        *value = CalendarSnapshot::new(c).unwrap();
    }
    let mut boundary = packet(3, 0, later);
    boundary.boundary = true;
    let steps = vec![
        StreamStep::Observe {
            record: packet(1, 0, calendar(0, 0)),
            progress: d(0),
            bytes: d(10),
        },
        StreamStep::Observe {
            record: packet(2, 10, ReferenceSpec::None),
            progress: d(1),
            bytes: d(10),
        },
        StreamStep::Observe {
            record: boundary,
            progress: d(2),
            bytes: d(10),
        },
        StreamStep::Observe {
            record: packet(4, 10, ReferenceSpec::None),
            progress: d(3),
            bytes: d(10),
        },
        StreamStep::Finish { progress: d(4) },
    ];
    let r = ReportSnapshot::new(ReplayPlan {
        limits: limits(),
        clock: ClockRecipe::Stream {
            policy: PolicySnapshot::stream(StreamPolicy::new(
                1,
                BufferCaps {
                    packets: 2,
                    bytes: 100,
                    references: 3,
                },
            ))
            .unwrap(),
            inventory: Some(vec![StreamChannel {
                id: 7,
                timein: true,
                external: true,
            }]),
            steps,
        },
        requests: vec![],
    })
    .unwrap();
    let AnswerBasis::Counter(c) = r.answers()[1].view().basis() else {
        panic!()
    };
    let ClockBasis::Stream(b) = c.clock() else {
        panic!()
    };
    assert_eq!(b.key.session, 2);
    assert_eq!(
        r.answers()[0].view().value().calendar.unwrap().year,
        Some(2017)
    );
    assert_eq!(
        r.answers()[1].view().value().calendar.unwrap().year,
        Some(2018)
    );
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
}

#[test]
fn qualified_edition_and_provided_year_evidence_survive_replay() {
    use irig106_time::edition::*;
    let mut p = plan();
    if let ClockRecipe::Offline { packets, .. } = &mut p.clock {
        packets[0].packet.body = ReferenceSpec::QualifiedFormat1 {
            payload: vec![0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
            data_length: d(12),
            evidence: EditionEvidence {
                override_edition: Some(PacketEdition::V04),
                tmats_edition: Some("24".into()),
                use_tmats: true,
                setup: SetupVersionEvidence::Unavailable,
                data_type_version: Some(0x0A),
            },
            scales: scale::ScalePolicy::default(),
        };
    }
    let mut capped = p.clone();
    capped.limits.raw_bytes = d(1);
    assert!(ReportSnapshot::new(capped).is_err());
    let r = ReportSnapshot::new(p.clone()).unwrap();
    assert!(!r.findings().iter().any(|f| f.code == 31));
    let refi = &r.sessions().unwrap()[0].references[0].input;
    assert!(refi.qualified.is_some());
    assert!(matches!(
        refi.source,
        Some(TimeF1Source::ExternalTime { present: true, .. })
    ));
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
    if let ClockRecipe::Offline { packets, .. } = &mut p.clock {
        if let ReferenceSpec::QualifiedFormat1 { evidence, .. } = &mut packets[0].packet.body {
            evidence.override_edition = None;
            evidence.tmats_edition = Some("04".into());
        }
    }
    let unresolved = ReportSnapshot::new(p).unwrap();
    assert!(unresolved.findings().iter().any(|f| f.code == 31));
    assert!(matches!(
        unresolved.sessions().unwrap()[0].references[0].input.source,
        Some(TimeF1Source::Unresolved(1))
    ));
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&unresolved)).unwrap()),
        encode(&unresolved)
    );
    let mut p = plan();
    if let ClockRecipe::Offline { packets, .. } = &mut p.clock {
        let mut body = calendar(0, 0);
        if let ReferenceSpec::Calendar {
            value, normalize, ..
        } = &mut body
        {
            let mut c = value.value();
            c.year = None;
            *value = CalendarSnapshot::new(c).unwrap();
            *normalize = None;
        }
        packets[0].packet.body = body;
        packets[0].packet.year_context = Some(years::YearContext {
            caller: Some(2023),
            recording: Some(2022),
        });
    }
    let r = ReportSnapshot::new(p).unwrap();
    assert_eq!(
        r.answers()[0].view().value().calendar.unwrap().year,
        Some(2023)
    );
    assert!(r.findings().iter().any(|f| f.code == 32));
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
}

#[test]
fn decoded_header_and_stamp_subseconds_reject_invalid_loads() {
    let c = Ch4BinaryTime {
        high_order: 0,
        low_order: 0,
        microseconds: 0,
    };
    let mut w = encode(&IntraPacketTime::Ch4(c));
    assert!(decode::<IntraPacketTime>(w.clone()).is_ok());
    *w.field("Ch4").field("microseconds") = Node::Number("10000".into());
    assert!(decode::<IntraPacketTime>(w).is_err());
    let mut w = encode(&SecondaryHeaderTime::Ch4(c));
    *w.field("Ch4").field("microseconds") = Node::Number("10000".into());
    assert!(decode::<SecondaryHeaderTime>(w).is_err());
    let ieee = Ieee1588Time {
        seconds: 0,
        nanoseconds: 0,
    };
    let mut w = encode(&IntraPacketTime::Ieee1588(ieee));
    assert!(decode::<IntraPacketTime>(w.clone()).is_ok());
    *w.field("Ieee1588").field("nanoseconds") = Node::Number("1000000000".into());
    assert!(decode::<IntraPacketTime>(w).is_err());
    let mut w = encode(&SecondaryHeaderTime::Ieee1588(ieee));
    *w.field("Ieee1588").field("nanoseconds") = Node::Number("1000000000".into());
    assert!(decode::<SecondaryHeaderTime>(w).is_err());
    let mut w = encode(&years::YearContext {
        caller: Some(2023),
        recording: None,
    });
    *w.field("caller") = Node::Number("10000".into());
    assert!(decode::<years::YearContext>(w).is_err());
}

#[test]
fn mismatched_word_era_context_is_retained_and_zero_still_wins() {
    use irig106_time::{ch4::*, extracted::*, ntp::*, scale::*};
    let mut ntp = NtpContext::new(2);
    ntp.declared = Some(EraDeclaration {
        era: 1,
        id: "other session".into(),
    });
    let mut p = plan();
    for seconds in [63104, 0] {
        p.requests.push(ReportRequest::Word {
            origin: origin(),
            word: TimeWord::Network(NetworkWord::Ntp(NtpTime {
                seconds,
                fraction: 0,
            })),
            context: context(1),
            bcd: BcdContext::default(),
            bcd_scale: TimeScale::Unknown,
            matching: None,
            ntp_context: Some(ntp.clone()),
        });
    }
    let r = ReportSnapshot::new(p).unwrap();
    assert!(r.answers()[1].view().value().calendar.is_none());
    assert!(matches!(
        r.answers()[1].view().value().issue,
        Some(TimeError::NeedsContext {
            context: "NTP word session context"
        })
    ));
    assert!(
        r.answers()[2]
            .view()
            .value()
            .findings
            .iter()
            .any(|f| f.code().number() == 29)
    );
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
}
