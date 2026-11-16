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
