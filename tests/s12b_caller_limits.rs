#![cfg(feature = "serde")]
//! Owner 2026-11-17 Option A: our Q5 JSON protocol, not IRIG 106 behaviour.
#[path = "support/serde_tree.rs"]
mod serde_tree;
#[path = "support/replay_fixtures.rs"]
mod fixtures;
use fixtures::*;
use irig106_time::snapshot::*;
use serde::de::IntoDeserializer;
use serde_tree::{decode, encode, Node};

#[test]
fn plain_load_rejects_document_claiming_huge_limits() {
    let mut wire = encode(&ReportSnapshot::new(plan()).unwrap());
    *wire.field("plan").field("limits").field("steps") = Node::String(usize::MAX.to_string());
    assert!(decode::<ReportSnapshot>(wire).is_err());
}

#[test]
fn explicit_caller_limits_apply_to_report_and_nested_answer() {
    let report = ReportSnapshot::new(plan()).unwrap();
    let wire = encode(&report);
    let caller = ReplayLimits::default();
    assert!(ReportSnapshot::load(wire.clone().into_deserializer(), &caller).is_ok());
    let answer = encode(&AnswerSnapshot::new(report, 0).unwrap());
    assert!(AnswerSnapshot::load(answer.clone().into_deserializer(), &caller).is_ok());
    let mut small = caller;
    small.steps = d(1);
    assert!(ReportSnapshot::load(wire.into_deserializer(), &small).is_err());
    assert!(AnswerSnapshot::load(answer.into_deserializer(), &small).is_err());
}

#[test]
fn actual_sizes_and_quality_work_are_checked_before_replay() {
    let wire = encode(&ReportSnapshot::new(plan()).unwrap());
    // Lower the *document's* claims, then enlarge content and poison the first
    // RTC: rejection must be resource preflight, not a later decode/replay error.
    for (limit, actual) in [("packets", "packets"), ("steps", "packets"),
        ("answers", "requests"), ("raw_bytes", "raw"),
        ("quality_queries", "quality"), ("quality_work", "work")] {
        let mut w = wire.clone();
        *w.field("plan").field("limits").field(limit) = Node::String("1".into());
        let plan = w.field("plan");
        match actual {
            "requests" => {
                let Node::Seq(v) = plan.field("requests") else { panic!() };
                v.push(v[0].clone());
            }
            "quality" | "work" => {
                *plan.field("clock").field("Offline").field("quality") = Node::Seq(vec![Node::Map(vec![
                    ("session".into(), Node::String("1".into())),
                    ("channel".into(), Node::Number("7".into()))])]);
                if actual == "quality" {
                    let Node::Seq(v) = plan.field("clock").field("Offline").field("quality") else { panic!() };
                    v.push(v[0].clone());
                }
            }
            _ => {}
        }
        let Node::Seq(packets) = plan.field("clock").field("Offline").field("packets") else { panic!() };
        *packets[0].field("packet").field("rtc") = Node::String(u64::MAX.to_string());
        let err = ReportSnapshot::load(w.into_deserializer(), &ReplayLimits::default()).unwrap_err();
        assert!(err.to_string().contains("replay resource limits"), "{limit}: {err}");
    }
}
