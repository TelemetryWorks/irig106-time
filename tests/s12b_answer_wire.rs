#![cfg(feature = "serde")]
//! Partial answer projection/protocol proof, not whole API/ANS coverage.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{answer::*, snapshot::*, timeline::*, *};
use serde_tree::encode;
#[test]
fn answer_output_has_version_units_full_basis_and_exact_counter() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let key = b
        .observe(
            PacketObservation {
                id: 9_007_199_254_740_993,
                flags: 0,
                rtc: Rtc::from_raw(7),
                channel: 2,
                declared_timein: false,
                declared_external: false,
                kind: PacketKind::Data,
                reference: None,
            },
            false,
        )
        .unwrap();
    let answer = b.finish().unwrap().lookup(key).unwrap();
    let output = AnswerOutput::new(AnswerView::offline(&answer, &[]).unwrap()).unwrap();
    let wire = encode(&output).json();
    assert!(wire.contains(r#""schema":"irig106-time/answer-output""#));
    assert!(wire.contains(r#""packet":"9007199254740993""#));
    assert!(!wire.contains(":9007199254740993"));
    assert_eq!(
        output.view().value().counter_duration,
        Some(SignedNanos::new(0))
    );
}

#[test]
fn qualified_reference_graph_serializes_context_findings_and_wide_quantities() {
    use irig106_time::{policy::*, scale::*};
    let shared = SharedPolicy::new(u64::MAX, SharedSettings::default()).unwrap();
    let mut b = TimelineBuilder::new(shared.offline(OfflinePolicy::default()).unwrap()).unwrap();
    let reference = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    b.observe(
        PacketObservation {
            id: 1,
            channel: 7,
            rtc: Rtc::from_raw(0),
            kind: PacketKind::Time,
            declared_timein: true,
            declared_external: true,
            flags: 0,
            reference: Some(reference),
        },
        false,
    )
    .unwrap();
    let key = b
        .observe(
            PacketObservation {
                id: 2,
                channel: 7,
                rtc: Rtc::from_raw(10),
                kind: PacketKind::Data,
                declared_timein: true,
                declared_external: true,
                flags: 0,
                reference: None,
            },
            false,
        )
        .unwrap();
    let answer = b.finish().unwrap().lookup(key).unwrap();
    let output = AnswerOutput::new(AnswerView::offline(&answer, &[]).unwrap()).unwrap();
    let mut wire = encode(&output);
    let basis = wire
        .field("value")
        .field("basis")
        .field("Counter")
        .field("clock")
        .field("Offline");
    assert_eq!(
        basis.field("distance_ns"),
        &serde_tree::Node::String("1000".into())
    );
    assert!(wire.json().contains(r#""id":"18446744073709551615""#));
    assert!(wire.json().contains("qualification"));
    assert_eq!(output.view().value().calendar.unwrap().year, Some(2017));
}

fn at<'a>(mut node: &'a serde_tree::Node, path: &[&str]) -> &'a serde_tree::Node {
    for part in path {
        node = match node {
            serde_tree::Node::Map(fields) => &fields.iter().find(|(k, _)| k == part).unwrap().1,
            serde_tree::Node::Seq(values) => &values[part.parse::<usize>().unwrap()],
            _ => panic!("not a container at {part}"),
        };
    }
    node
}
fn fields(node: &serde_tree::Node, expected: &[&str]) {
    let serde_tree::Node::Map(entries) = node else {
        panic!("expected object")
    };
    let mut actual: Vec<_> = entries.iter().map(|(name, _)| name.as_str()).collect();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}
fn word_output() -> AnswerOutput {
    use irig106_time::{extracted::*, ntp::*, secondary_packet::*};
    let mut context = WordContext::new(1);
    context.ntp.declared = Some(EraDeclaration {
        era: -1,
        id: "caller negative era".into(),
    });
    // RFC5905 §6/Fig4: era width2^32s; 1970's era0 word2,208,988,800
    // in era-1 maps to Unix seconds -4,294,967,296. No leap offset is added.
    let raw = NtpTime {
        seconds: 2_208_988_800,
        fraction: 0x80000000,
    };
    let converted = convert_ntp(raw, &context.ntp, None);
    assert_eq!(converted.unix_seconds().unwrap().get(), -4_294_967_296);
    context
        .ntp
        .pivots
        .push(converted.as_pivot("resolved negative era").unwrap());
    let answer = answer_word(
        ExtractionOrigin {
            key: PacketKey {
                session: 1,
                ticks: 0,
                packet: 1,
            },
            channel: 7,
            byte_offset: Some(42),
            field: "NTP word".into(),
        },
        TimeWord::Network(NetworkWord::Ntp(raw)),
        &context,
        &SecondaryPolicy::default(),
    )
    .unwrap();
    AnswerOutput::new(AnswerView::word(&answer).unwrap()).unwrap()
}
#[test]
fn negative_i64_eras_in_answer_output_are_decimal_strings() {
    let output = word_output();
    let wire = encode(&output);
    for path in [
        vec![
            "value", "basis", "Word", "context", "ntp", "declared", "era",
        ],
        vec![
            "value",
            "basis",
            "Word",
            "context",
            "ntp",
            "pivots",
            "0",
            "origin",
            "ResolvedNtp",
            "era",
        ],
        vec![
            "value", "value", "network", "Ntp", "basis", "context", "declared", "era",
        ],
    ] {
        assert_eq!(
            at(&wire, &path),
            &serde_tree::Node::String("-1".into()),
            "{path:?}"
        );
    }
}
fn answer_value_fields(node: &serde_tree::Node, value: &AnswerValue) {
    // Exhaustive public patterns force review when the native structs grow,
    // even if someone forgets to add the new field to serialize_struct!.
    let AnswerValue {
        calendar: _,
        counter_duration: _,
        counter: _,
        point: _,
        own_minus_counter: _,
        tolerance: _,
        issue: _,
        findings: _,
        network: _,
        chapter4: _,
    } = value;
    fields(
        node,
        &[
            "calendar",
            "counter_duration",
            "counter",
            "point",
            "own_minus_counter",
            "tolerance",
            "issue",
            "findings",
            "network",
            "chapter4",
        ],
    );
}
fn offline_fields(node: &serde_tree::Node, basis: &OfflineBasis) {
    let OfflineBasis {
        session_origin: _,
        key: _,
        channel: _,
        reference: _,
        distance_ns: _,
        position: _,
        unresolved: _,
        policy: _,
        drift_corrected: _,
    } = basis;
    fields(
        node,
        &[
            "session_origin",
            "key",
            "channel",
            "reference",
            "distance_ns",
            "position",
            "unresolved",
            "policy",
            "drift_corrected",
        ],
    );
}
#[test]
fn offline_and_word_outputs_pin_complete_emitted_field_sets() {
    let mut builder = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let key = builder
        .observe(
            PacketObservation {
                id: 1,
                channel: 7,
                rtc: Rtc::from_raw(0),
                flags: 0,
                kind: PacketKind::Data,
                declared_timein: false,
                declared_external: false,
                reference: None,
            },
            false,
        )
        .unwrap();
    let answer = builder.finish().unwrap().lookup(key).unwrap();
    let output = AnswerOutput::new(AnswerView::offline(&answer, &[]).unwrap()).unwrap();
    let wire = encode(&output);
    fields(
        &wire,
        &["schema", "version", "defaults_version", "units", "value"],
    );
    fields(at(&wire, &["value"]), &["basis", "value"]);
    fields(at(&wire, &["value", "basis"]), &["Counter"]);
    fields(
        at(&wire, &["value", "basis", "Counter"]),
        &["clock", "findings"],
    );
    fields(
        at(&wire, &["value", "basis", "Counter", "clock"]),
        &["Offline"],
    );
    let basis = at(&wire, &["value", "basis", "Counter", "clock", "Offline"]);
    offline_fields(basis, &answer.basis);
    fields(
        at(basis, &["session_origin"]),
        &["session", "ticks", "packet"],
    );
    fields(at(basis, &["key"]), &["session", "ticks", "packet"]);
    let value = at(&wire, &["value", "value"]);
    answer_value_fields(value, output.view().value());
    fields(
        at(value, &["counter", "Offline"]),
        &["time", "reading", "counter_ns", "basis"],
    );
    offline_fields(at(value, &["counter", "Offline", "basis"]), &answer.basis);

    let output = word_output();
    let wire = encode(&output);
    fields(
        &wire,
        &["schema", "version", "defaults_version", "units", "value"],
    );
    fields(at(&wire, &["value"]), &["basis", "value"]);
    fields(at(&wire, &["value", "basis"]), &["Word"]);
    let basis = at(&wire, &["value", "basis", "Word"]);
    fields(basis, &["origin", "word", "context", "policy"]);
    fields(
        at(basis, &["origin"]),
        &["key", "channel", "byte_offset", "field"],
    );
    fields(
        at(basis, &["context"]),
        &["own", "bcd", "bcd_scale", "ntp", "matching"],
    );
    fields(
        at(basis, &["context", "ntp"]),
        &["session", "declared", "pivots"],
    );
    fields(at(basis, &["context", "ntp", "declared"]), &["era", "id"]);
    answer_value_fields(at(&wire, &["value", "value"]), output.view().value());
}
