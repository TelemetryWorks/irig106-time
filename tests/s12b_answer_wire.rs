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
                packet: 9_007_199_254_740_993,
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
    let answer = b.finish().unwrap().answer(key).unwrap();
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
