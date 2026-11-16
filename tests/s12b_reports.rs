#![cfg(feature = "serde")]
//! Q5 checked replay clauses; schema review remains an explicit owner step.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{snapshot::*, timeline::*, *};
use serde_tree::{Node, decode, encode};
fn d<T: ExactInteger>(v: T) -> Decimal<T> {
    Decimal::new(v)
}
fn packet(id: u64, ticks: u64, body: ReferenceSpec) -> RecordedPacket {
    RecordedPacket {
        packet: ReplayPacket {
            id: d(id),
            rtc: d(ticks),
            channel: 7,
            kind: if matches!(body, ReferenceSpec::None) {
                PacketKind::Data
            } else {
                PacketKind::Time
            },
            declared_timein: true,
            declared_external: true,
            flags: 0,
            body,
        },
        boundary: false,
    }
}
fn limits() -> ReplayLimits {
    ReplayLimits {
        steps: d(100),
        answers: d(100),
        raw_bytes: d(10000),
    }
}
fn plan() -> ReplayPlan {
    ReplayPlan {
        limits: limits(),
        clock: ClockRecipe::Offline {
            policy: PolicySnapshot::offline(OfflinePolicy::default()).unwrap(),
            packets: vec![
                packet(
                    1,
                    0,
                    ReferenceSpec::Format1 {
                        payload: vec![0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
                        data_length: d(12),
                        layout: TimeF1Layout::SourceWithIts17,
                        scales: scale::ScalePolicy::default(),
                    },
                ),
                packet(2, 10, ReferenceSpec::None),
            ],
            quality: vec![],
        },
        requests: vec![ReportRequest::Counter {
            key: ReplayKey {
                session: d(1),
                ticks: d(10),
                packet: d(2),
            },
        }],
    }
}
#[test]
fn report_and_answer_snapshots_replay_full_native_views() {
    let report = ReportSnapshot::new(plan()).unwrap();
    let wire = encode(&report);
    let loaded = decode::<ReportSnapshot>(wire.clone()).unwrap();
    assert_eq!(encode(&loaded), wire);
    assert_eq!(
        loaded.answers()[0].view().value().calendar.unwrap().year,
        Some(2017)
    );
    let answer = AnswerSnapshot::new(report, 0).unwrap();
    let loaded = decode::<AnswerSnapshot>(encode(&answer)).unwrap();
    assert_eq!(loaded.view().value(), answer.view().value());
}
#[test]
fn report_load_rejects_projection_and_header_tampering() {
    let report = ReportSnapshot::new(plan()).unwrap();
    let wire = encode(&report);
    for (key, bad) in [
        ("version", Node::Number("2".into())),
        ("defaults_version", Node::Number("2".into())),
        ("schema", Node::String("other".into())),
    ] {
        let mut w = wire.clone();
        *w.field(key) = bad;
        assert!(decode::<ReportSnapshot>(w).is_err());
    }
    let mut w = wire;
    *w.field("outputs") = Node::Null;
    assert!(decode::<ReportSnapshot>(w).is_err());
}
