#![cfg(feature = "serde")]
//! Owner 2026-11-17 Option A: our Q5 JSON protocol, not IRIG 106 behaviour.
#[path = "support/replay_fixtures.rs"]
mod fixtures;
#[path = "support/serde_tree.rs"]
mod serde_tree;
use fixtures::*;
use irig106_time::snapshot::*;
use serde::de::IntoDeserializer;
use serde_tree::{Node, decode, encode};

#[test]
fn plain_load_rejects_document_claiming_huge_limits() {
    let wire = encode(&ReportSnapshot::new(plan()).unwrap());
    for field in [
        "packets",
        "steps",
        "answers",
        "raw_bytes",
        "quality_queries",
        "quality_work",
    ] {
        let mut w = wire.clone();
        *w.field("plan").field("limits").field(field) = Node::String(usize::MAX.to_string());
        assert!(decode::<ReportSnapshot>(w).is_err(), "{field}");
    }
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
    for (limit, actual) in [
        ("packets", "packets"),
        ("steps", "packets"),
        ("answers", "requests"),
        ("raw_bytes", "raw"),
        ("quality_queries", "quality"),
        ("quality_work", "work"),
    ] {
        let mut w = wire.clone();
        *w.field("plan").field("limits").field(limit) = Node::String("1".into());
        let plan = w.field("plan");
        match actual {
            "requests" => {
                let Node::Seq(v) = plan.field("requests") else {
                    panic!()
                };
                v.push(v[0].clone());
            }
            "quality" | "work" => {
                *plan.field("clock").field("Offline").field("quality") =
                    Node::Seq(vec![Node::Map(vec![
                        ("session".into(), Node::String("1".into())),
                        ("channel".into(), Node::Number("7".into())),
                    ])]);
                if actual == "quality" {
                    let Node::Seq(v) = plan.field("clock").field("Offline").field("quality") else {
                        panic!()
                    };
                    v.push(v[0].clone());
                }
            }
            _ => {}
        }
        let Node::Seq(packets) = plan.field("clock").field("Offline").field("packets") else {
            panic!()
        };
        *packets[0].field("packet").field("rtc") = Node::String(u64::MAX.to_string());
        // Claims fit the caller exactly; actual content exceeds that authority.
        let caller: ReplayLimits = decode(w.field("plan").field("limits").clone()).unwrap();
        let err = ReportSnapshot::load(w.into_deserializer(), &caller).unwrap_err();
        assert!(
            err.to_string().contains("replay resource limits"),
            "{limit}: {err}"
        );
    }
}

#[test]
fn larger_explicit_caller_budget_is_not_replaced_by_default_or_nested_document() {
    let report = ReportSnapshot::new(plan()).unwrap();
    let mut report_wire = encode(&report);
    let mut answer_wire = encode(&AnswerSnapshot::new(report, 0).unwrap());
    *report_wire.field("plan").field("limits").field("steps") = Node::String("9000".into());
    *answer_wire
        .field("value")
        .field("report")
        .field("plan")
        .field("limits")
        .field("steps") = Node::String("9000".into());
    assert!(decode::<ReportSnapshot>(report_wire.clone()).is_err());
    assert!(decode::<AnswerSnapshot>(answer_wire.clone()).is_err());
    let caller = ReplayLimits {
        steps: d(9000),
        ..ReplayLimits::default()
    };
    assert!(ReportSnapshot::load(report_wire.into_deserializer(), &caller).is_ok());
    assert!(AnswerSnapshot::load(answer_wire.into_deserializer(), &caller).is_ok());
}

#[test]
fn cached_answer_and_quality_counts_are_also_preflighted() {
    let report = ReportSnapshot::new(plan()).unwrap();
    for field in ["answers", "quality"] {
        let mut w = encode(&report);
        let caller = ReplayLimits {
            answers: d(100),
            ..ReplayLimits::default()
        };
        let n = if field == "answers" { 101 } else { 33 };
        *w.field("outputs").field(field) = Node::Seq(vec![Node::Null; n]);
        let err = ReportSnapshot::load(w.into_deserializer(), &caller).unwrap_err();
        assert!(
            err.to_string().contains("replay resource limits"),
            "{field}: {err}"
        );
    }
}

#[test]
fn stream_packet_and_emitted_answer_counts_are_preflighted() {
    use irig106_time::stream::*;
    let policy = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 4,
            bytes: 100,
            references: 2,
        },
    );
    let p = ReplayPlan {
        limits: limits(),
        clock: ClockRecipe::Stream {
            policy: PolicySnapshot::stream(policy).unwrap(),
            inventory: Some(vec![StreamChannel {
                id: 7,
                timein: true,
                external: true,
            }]),
            steps: vec![
                StreamStep::Observe {
                    record: packet(1, 0, calendar(0, 0)),
                    progress: d(0),
                    bytes: d(1),
                },
                StreamStep::Observe {
                    record: packet(2, 1, ReferenceSpec::None),
                    progress: d(1),
                    bytes: d(1),
                },
                StreamStep::Observe {
                    record: packet(3, 2, ReferenceSpec::None),
                    progress: d(2),
                    bytes: d(1),
                },
                StreamStep::Finish { progress: d(3) },
            ],
        },
        requests: vec![],
    };
    let r = ReportSnapshot::new(p).unwrap();
    let wire = encode(&r);
    assert!(decode::<ReportSnapshot>(wire.clone()).is_ok());
    for field in ["packets", "answers"] {
        let mut w = wire.clone();
        *w.field("plan").field("limits").field(field) = Node::String("1".into());
        // No output-count shortcut: prove recipe counts are checked too.
        *w.field("outputs").field("answers") = Node::Seq(vec![]);
        let err =
            ReportSnapshot::load(w.into_deserializer(), &ReplayLimits::default()).unwrap_err();
        assert!(
            err.to_string().contains("replay resource limits"),
            "{field}: {err}"
        );
    }
}
