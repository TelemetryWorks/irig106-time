#![cfg(feature = "serde")]
//! Partial POL103/API107/API110: whole report replay remains a separate part.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{policy::*, snapshot::*, stream::*, timeline::*};
use serde_tree::{Node, decode, encode};

#[test]
fn complete_policy_roundtrip_preserves_nondefaults_and_exact_fields() {
    let shared = SharedPolicy::new(
        9_007_199_254_740_993,
        SharedSettings {
            caller_year: Some(2016),
            use_tfmt_default: true,
            ..Default::default()
        },
    )
    .unwrap();
    let caps = BufferCaps {
        packets: 23,
        bytes: 9_007_199_254_740_993,
        references: 3,
    };
    let mut core = StreamPolicy::new(shared.id(), caps);
    core.mode = StreamMode::Interpolate;
    core.raw_network_rate = true;
    core.offline.jump_threshold_ns = Some(17);
    core.offline
        .finding_levels
        .insert(30, irig106_time::findings::Severity::Information);
    let core = shared.stream(core).unwrap();
    let snapshot = PolicySnapshot::stream(core.clone()).unwrap();
    let wire = encode(&snapshot);
    assert!(wire.json().contains("\"9007199254740993\""));
    assert!(!wire.json().contains(":9007199254740993"));
    assert_eq!(
        decode::<PolicySnapshot>(wire).unwrap().value(),
        &PolicyValue::Stream(core)
    );
    let offline = PolicySnapshot::offline(OfflinePolicy::default()).unwrap();
    assert_eq!(decode::<PolicySnapshot>(encode(&offline)).unwrap(), offline);
}

#[test]
fn policy_load_rejects_tampered_headers_units_and_rules() {
    let p = PolicySnapshot::stream(StreamPolicy::new(
        1,
        BufferCaps {
            packets: 1,
            bytes: 4,
            references: 2,
        },
    ))
    .unwrap();
    let wire = encode(&p);
    for (key, bad) in [
        ("version", Node::Number("2".into())),
        ("defaults_version", Node::Number("2".into())),
        ("schema", Node::String("other".into())),
    ] {
        let mut w = wire.clone();
        *w.field(key) = bad;
        assert!(decode::<PolicySnapshot>(w).is_err(), "{key}");
    }
    let mut w = wire.clone();
    *w.field("units").field("duration") = Node::String("ms".into());
    assert!(decode::<PolicySnapshot>(w).is_err());
    for (key, bad) in [
        ("id", Node::String("0".into())),
        ("hold_interval_ns", Node::String(u64::MAX.to_string())),
    ] {
        let mut w = wire.clone();
        *w.field("value").field("Stream").field(key) = bad;
        assert!(decode::<PolicySnapshot>(w).is_err(), "{key}");
    }
    let mut w = wire.clone();
    *w.field("value")
        .field("Stream")
        .field("caps")
        .field("references") = Node::String("1".into());
    assert!(decode::<PolicySnapshot>(w).is_err());
    let mut w = wire;
    if let Node::Map(fields) = &mut w {
        fields.push(("extra".into(), Node::Null));
    }
    assert!(decode::<PolicySnapshot>(w).is_err());
}

fn full_policy() -> StreamPolicy {
    use irig106_time::{
        declarations::*, ntp::*, own_time::*, scale::*, secondary_packet::TimeApplicability, *,
    };
    let date = format1::CalendarReading {
        year: Some(2016),
        day_of_year: 366,
        hours: 23,
        minutes: 59,
        seconds: 60,
        nanoseconds: 17,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    };
    let mut context = NtpContext::new(2);
    context.declared = Some(EraDeclaration {
        era: 0,
        id: "caller era".into(),
    });
    context.pivots.push(
        NtpPivot::approximate(2, "before1970", SignedUnixSeconds::new(-1), 999_999_999).unwrap(),
    );
    context
        .pivots
        .push(NtpPivot::dated(2, "leap", u64::MAX, date).unwrap());
    let conversion = convert_ntp(
        NtpTime {
            seconds: 2_208_988_800,
            fraction: 0x80000000,
        },
        &context,
        None,
    );
    context
        .pivots
        .push(conversion.as_pivot("resolved").unwrap());
    // Independently derived (rather than declared) is retained, too.
    let mut derived = NtpContext::new(2);
    derived
        .pivots
        .push(NtpPivot::approximate(2, "1970", SignedUnixSeconds::new(0), 0).unwrap());
    context.pivots.push(
        convert_ntp(
            NtpTime {
                seconds: 2_208_988_801,
                fraction: 1,
            },
            &derived,
            None,
        )
        .as_pivot("derived")
        .unwrap(),
    );
    let declarations = Declarations::present(vec![SetupDeclarations {
        range: GoverningRange::new(
            2,
            SignedRtcTicks::new(i128::MIN),
            Some(SignedRtcTicks::new(i128::MAX)),
        )
        .unwrap(),
        setup_packet: u64::MAX,
        provenance: "raw caller setup".into(),
        declared_version: Some("unknown retained".into()),
        ri4: Some("2016 source spelling".into()),
        recording_year: Some(2016),
        channels: vec![ChannelDeclaration {
            id: 7,
            cdt: Some("TIMEIN".into()),
            ttf: Some("IRIG".into()),
            tfmt: None,
            tsrc: Some("E".into()),
            shtf: Some("unknown retained".into()),
        }],
    }])
    .unwrap();
    let shared = SharedPolicy::new(
        u64::MAX,
        SharedSettings {
            caller_year: Some(2017),
            year_order: YearOrder::new(vec![YearSource::Caller, YearSource::Recording]).unwrap(),
            scales: ScalePolicy {
                irig: TimeScale::Tai,
                rtc: TimeScale::Gps,
            },
            own: OwnContext {
                ieee: Some(EpochContext {
                    origin: date,
                    counting: EpochCounting::Continuous,
                    source: "caller epoch".into(),
                }),
                ch4: Some(EpochContext {
                    origin: format1::CalendarReading {
                        year: Some(2010),
                        day_of_year: 1,
                        hours: 0,
                        minutes: 0,
                        seconds: 0,
                        nanoseconds: 0,
                        scale: ScaleEvidence::standard(TimeScale::Tai),
                    },
                    counting: EpochCounting::Continuous,
                    source: "explicit Chapter4 origin".into(),
                }),
                counter_applicability: TimeApplicability::CitedException {
                    data_type: 0x21,
                    citation: "explicit caller citation".into(),
                },
                ..Default::default()
            },
            own_tolerance: Some(NanosDuration::new(u64::MAX)),
            declarations: Some(declarations),
            use_tfmt_default: true,
            ntp: [(2, context)].into(),
        },
    )
    .unwrap();
    let mut p = StreamPolicy::new(
        u64::MAX,
        BufferCaps {
            packets: usize::MAX,
            bytes: usize::MAX,
            references: usize::MAX,
        },
    );
    p.mode = StreamMode::Interpolate;
    p.hold_interval_ns = 17;
    p.hold_tolerance_ns = 31;
    p.raw_network_rate = true;
    p.offline.gap_interval_ns = 23;
    p.offline.gap_tolerance_ns = 19;
    p.offline.late_ns = 43;
    p.offline.infer_sessions = false;
    p.offline.channel = Some(7);
    p.offline.include_invalid = true;
    p.offline.include_undeclared = true;
    p.offline.preceding_on_tie = false;
    p.offline.stop_on_loss = true;
    p.offline.jump_threshold_ns = Some(u64::MAX);
    p.offline
        .finding_levels
        .insert(21, findings::Severity::Information);
    shared.stream(p).unwrap()
}
#[test]
fn policy_full_graph_preserves_context_sources_tables_and_all_options() {
    let p = full_policy();
    let snap = PolicySnapshot::stream(p.clone()).unwrap();
    let wire = encode(&snap);
    assert_eq!(decode::<PolicySnapshot>(wire.clone()).unwrap(), snap);
    let json = wire.json();
    for exact in [
        "\"18446744073709551615\"",
        "\"-170141183460469231731687303715884105728\"",
        "\"170141183460469231731687303715884105727\"",
        "\"-1\"",
        "\"CallerApproximateDate\"",
        "\"DatedReference\"",
        "\"ResolvedNtp\"",
        "\"PivotDerived\"",
    ] {
        assert!(json.contains(exact), "{exact}");
    }
    println!("POLICY_JSON {json}");
    // Offline-only lookup choices are independent of stream restrictions.
    let mut offline = p.offline;
    offline.strategy = ReferenceStrategy::Preceding;
    offline.channel_preference = vec![9, 7];
    offline.back_extrapolate = true;
    let s = PolicySnapshot::offline(offline).unwrap();
    assert_eq!(decode::<PolicySnapshot>(encode(&s)).unwrap(), s);
    // Three declaration states remain distinct, including null versus absent.
    for d in [
        None,
        Some(irig106_time::declarations::Declarations::absent()),
        Some(irig106_time::declarations::Declarations::present(vec![]).unwrap()),
    ] {
        let s = SharedPolicy::new(
            1,
            SharedSettings {
                declarations: d,
                ..Default::default()
            },
        )
        .unwrap();
        let snapshot =
            PolicySnapshot::offline(s.offline(OfflinePolicy::default()).unwrap()).unwrap();
        assert_eq!(
            decode::<PolicySnapshot>(encode(&snapshot)).unwrap(),
            snapshot
        );
    }
}

fn path<'a>(mut node: &'a mut Node, keys: &[&str]) -> &'a mut Node {
    for key in keys {
        node = match node {
            Node::Seq(v) => &mut v[key.parse::<usize>().unwrap()],
            _ => node.field(key),
        };
    }
    node
}
#[test]
fn policy_graph_rejects_invalid_links_ranges_provenance_and_duplicates() {
    let wire = encode(&PolicySnapshot::stream(full_policy()).unwrap());
    let prefix = ["value", "Stream", "offline", "shared"];
    let mut cases: Vec<(Vec<&str>, Node)> = vec![
        (vec!["id"], Node::String("1".into())), // shared/stream identity mismatch
        (vec!["caller_year"], Node::Number("10000".into())),
        (
            vec!["year_order"],
            Node::Seq(vec![
                Node::String("Caller".into()),
                Node::String("Caller".into()),
            ]),
        ),
        (vec!["ieee", "counting"], Node::String("Civil".into())), // second60 origin
        (vec!["ieee", "source"], Node::String(" ".into())),
        (
            vec!["counter_applicability", "CitedException", "citation"],
            Node::String("".into()),
        ),
        (vec!["ntp", "0", "key"], Node::String("3".into())),
        (vec!["ntp", "0", "session"], Node::String("0".into())),
        (vec!["ntp", "0", "declared", "id"], Node::String("".into())),
        (
            vec!["ntp", "0", "pivots", "0", "session"],
            Node::String("3".into()),
        ),
        (
            vec!["declarations", "Present", "0", "session"],
            Node::String("0".into()),
        ),
        (
            vec!["declarations", "Present", "0", "end"],
            Node::String(i128::MIN.to_string()),
        ),
        (
            vec!["declarations", "Present", "0", "recording_year"],
            Node::Number("10000".into()),
        ),
        (
            vec!["ntp", "0", "pivots", "0", "fraction", "Nanoseconds"],
            Node::Number("1000000000".into()),
        ),
        (
            vec!["ntp", "0", "pivots", "0", "scale", "effective"],
            Node::String("Tai".into()),
        ),
        (
            vec!["ntp", "0", "pivots", "1", "seconds"],
            Node::String("0".into()),
        ),
        (
            vec!["ntp", "0", "pivots", "2", "origin", "ResolvedNtp", "kind"],
            Node::String("AssumedZero".into()),
        ),
        (
            vec!["ntp", "0", "pivots", "2", "origin", "ResolvedNtp", "era"],
            Node::String("1".into()),
        ),
        (
            vec!["ntp", "0", "pivots", "2", "seconds"],
            Node::String(i128::MAX.to_string()),
        ),
        (vec!["ntp", "0", "pivots", "2", "calendar"], Node::Null),
    ];
    // Duplicate lists must not silently last-write-wins into a map.
    for keys in [
        vec!["ntp"],
        vec!["declarations", "Present"],
        vec!["declarations", "Present", "0", "channels"],
    ] {
        let mut w = wire.clone();
        let target = path(path(&mut w, &prefix), &keys);
        let Node::Seq(list) = target else { panic!() };
        list.push(list[0].clone());
        assert!(decode::<PolicySnapshot>(w).is_err(), "duplicate {keys:?}");
    }
    for (keys, bad) in cases.drain(..) {
        let mut w = wire.clone();
        *path(path(&mut w, &prefix), &keys) = bad;
        assert!(decode::<PolicySnapshot>(w).is_err(), "{keys:?}");
    }
    for key in ["shared", "channel", "jump_threshold_ns"] {
        let mut w = wire.clone();
        let Node::Map(fields) = path(&mut w, &["value", "Stream", "offline"]) else {
            panic!()
        };
        fields.retain(|(k, _)| k != key);
        assert!(decode::<PolicySnapshot>(w).is_err(), "missing {key}");
    }
    let mut w = wire;
    let Node::Seq(levels) = path(&mut w, &["value", "Stream", "offline", "finding_levels"]) else {
        panic!()
    };
    levels.push(levels[0].clone());
    assert!(decode::<PolicySnapshot>(w).is_err());
}

#[test]
fn policy_schema_pins_units_and_unknown_field_guards_at_every_new_layer() {
    let wire = encode(&PolicySnapshot::stream(full_policy()).unwrap());
    let mut units = wire.clone();
    assert_eq!(
        units.field("units").json(),
        r#"{"buffer_bytes":"byte","counter":"rtc_ticks","duration":"ns","identity":"count","ntp_fraction":"2^-32 s","unix":"unix_s"}"#
    );
    for keys in [
        vec![],
        vec!["value", "Stream"],
        vec!["value", "Stream", "caps"],
        vec!["value", "Stream", "offline"],
        vec!["value", "Stream", "offline", "shared"],
        vec!["value", "Stream", "offline", "shared", "scales"],
        vec!["value", "Stream", "offline", "shared", "ieee"],
        vec![
            "value",
            "Stream",
            "offline",
            "shared",
            "counter_applicability",
            "CitedException",
        ],
        vec![
            "value",
            "Stream",
            "offline",
            "shared",
            "declarations",
            "Present",
            "0",
        ],
        vec![
            "value",
            "Stream",
            "offline",
            "shared",
            "declarations",
            "Present",
            "0",
            "channels",
            "0",
        ],
        vec!["value", "Stream", "offline", "shared", "ntp", "0"],
        vec![
            "value", "Stream", "offline", "shared", "ntp", "0", "declared",
        ],
        vec![
            "value", "Stream", "offline", "shared", "ntp", "0", "pivots", "0",
        ],
        vec![
            "value",
            "Stream",
            "offline",
            "shared",
            "ntp",
            "0",
            "pivots",
            "2",
            "origin",
            "ResolvedNtp",
        ],
    ] {
        let mut w = wire.clone();
        let Node::Map(fields) = path(&mut w, &keys) else {
            panic!()
        };
        fields.push(("extra".into(), Node::Bool(true)));
        assert!(decode::<PolicySnapshot>(w).is_err(), "unknown at {keys:?}");
    }
    // Required option presence, not silent adoption of a future default.
    for keys in [
        vec!["value", "Stream", "offline", "shared"],
        vec!["value", "Stream", "offline", "shared", "ntp", "0"],
        vec![
            "value",
            "Stream",
            "offline",
            "shared",
            "declarations",
            "Present",
            "0",
        ],
    ] {
        let mut w = wire.clone();
        let Node::Map(fields) = path(&mut w, &keys) else {
            panic!()
        };
        let key = if keys.last() == Some(&"shared") {
            "own_tolerance"
        } else if keys.contains(&"ntp") {
            "declared"
        } else {
            "recording_year"
        };
        fields.retain(|(k, _)| k != key);
        assert!(decode::<PolicySnapshot>(w).is_err(), "missing {key}");
    }
}

#[test]
fn resolved_pivot_cannot_reload_zero_sentinel_as_independent_date() {
    use irig106_time::{NtpTime, ntp::*};
    let mut context = NtpContext::new(1);
    context.declared = Some(EraDeclaration {
        era: 0,
        id: "caller".into(),
    });
    // RFC5905 §6: both words zero is unknown. One fractional unit is not zero,
    // although floor(1 * 1e9 / 2^32) is0ns, so the civil coordinate is identical.
    let pivot = convert_ntp(
        NtpTime {
            seconds: 0,
            fraction: 1,
        },
        &context,
        None,
    )
    .as_pivot("source")
    .unwrap();
    let mut wire = encode(&pivot);
    assert_eq!(decode::<NtpPivot>(wire.clone()).unwrap(), pivot);
    *wire.field("fraction").field("NtpUnits") = Node::Number("0".into());
    assert!(decode::<NtpPivot>(wire).is_err());
}

#[test]
fn resolved_ntp_pivot_rejects_non_utc_effective_scale_on_load() {
    use irig106_time::{ntp::*, scale::*};
    // RDG-012 / RFC5905 §6: NTP follows UTC. Keep the resolved coordinate and
    // calendar unchanged; replace the scale with another internally valid
    // ScaleEvidence, so rejection cannot come from contradictory scale fields.
    let policy = full_policy();
    let pivot = &policy.offline.shared.as_ref().unwrap().settings().ntp[&2].pivots[2];
    assert!(matches!(pivot.origin(), PivotOrigin::ResolvedNtp { .. }));
    let wire = encode(pivot);
    assert_eq!(decode::<NtpPivot>(wire.clone()).unwrap(), *pivot);
    for scale in [TimeScale::Tai, TimeScale::Gps, TimeScale::Unknown] {
        let mut bad = wire.clone();
        *bad.field("scale") = encode(&ScaleEvidence::standard(scale));
        assert!(decode::<NtpPivot>(bad).is_err(), "{scale:?}");
    }
}

#[test]
fn caller_approximate_pivot_rejects_ntp_fraction_units_on_load() {
    use irig106_time::{SignedUnixSeconds, ntp::*};
    // Both numeric values are zero; only the unit discriminant changes.
    // Approximate caller dates carry nanoseconds, not raw 2^-32-second units
    // (RDG-012). All other provenance/coordinate fields remain valid.
    let pivot = NtpPivot::approximate(1, "caller date", SignedUnixSeconds::new(0), 0).unwrap();
    let mut wire = encode(&pivot);
    assert_eq!(decode::<NtpPivot>(wire.clone()).unwrap(), pivot);
    *wire.field("fraction") = encode(&PivotFraction::NtpUnits(0));
    assert!(decode::<NtpPivot>(wire).is_err());
}
