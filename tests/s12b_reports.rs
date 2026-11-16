#![cfg(feature = "serde")]
//! Q5 checked replay clauses; schema review remains an explicit owner step.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{snapshot::*, *};
use serde_tree::{Node, decode, encode};
#[path = "support/replay_fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn report_and_answer_snapshots_replay_full_native_views() {
    let report = ReportSnapshot::new(plan()).unwrap();
    let mut wire = encode(&report);
    assert!(matches!(
        wire.field("outputs").field("sessions"),
        Node::Seq(_)
    ));
    assert_eq!(report.sessions().unwrap()[0].references.len(), 1);
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

#[test]
fn all_adapter_envelopes_roundtrip_raw_partial_and_absolute_context() {
    use irig106_time::{ch4::*, extracted::*, scale::*, secondary_packet::*};
    let mut p = plan();
    let ctx = context(1);
    // Figure11-5: IEEE nanoseconds first, seconds next; zero prefix has checksum0.
    p.requests.push(ReportRequest::Own {
        counter: d(0),
        origin: origin(),
        data_type: 0x21,
        rtc: d(10),
        flags: 0x84,
        raw: vec![0; 12],
        applicability: TimeApplicability::FirstDataBit,
        context: ctx.clone(),
    });
    let mut stamp = vec![0; 8];
    stamp[0] = 10;
    p.requests.push(ReportRequest::Stamp {
        counter: Some(d(0)),
        origin: origin(),
        query_id: d(2),
        flags: 0,
        raw: stamp,
        order: rtc::CounterOrder::ForwardWithinOneWrap,
        context: ctx.clone(),
    });
    p.requests.push(ReportRequest::Word {
        origin: origin(),
        word: TimeWord::Binary(Ch4Words {
            high: 0,
            low: 0x8000,
            microseconds: 0,
        }),
        context: ctx.clone(),
        bcd: BcdContext::default(),
        bcd_scale: TimeScale::Unknown,
        matching: None,
    });
    // RFC5905 Fig4:1970 coordinate and fraction0x80000000=0.5s.
    p.requests.push(ReportRequest::Word {
        origin: origin(),
        word: TimeWord::Network(NetworkWord::Ntp(NtpTime {
            seconds: 2208988800,
            fraction: 0x80000000,
        })),
        context: ctx,
        bcd: BcdContext::default(),
        bcd_scale: TimeScale::Unknown,
        matching: None,
    });
    let r = ReportSnapshot::new(p).unwrap();
    assert_eq!(r.answers().len(), 5);
    assert!(
        r.answers()[1]
            .view()
            .value()
            .findings
            .iter()
            .any(|f| f.code().number() == 14)
    );
    assert!(r.answers()[2].view().value().counter.is_some());
    assert!(r.answers()[3].view().value().chapter4.is_some());
    assert!(r.answers()[3].view().value().calendar.is_none());
    assert!(r.answers()[4].view().value().network.is_some());
    let wire = encode(&r);
    let loaded = decode::<ReportSnapshot>(wire.clone()).unwrap();
    assert_eq!(encode(&loaded), wire);
}
#[test]
fn mixed_network_zero_and_stale_context_reports_keep_findings_and_scales() {
    use irig106_time::edition::PacketEdition;
    let mut p = plan();
    let ctx = context(1);
    if let ClockRecipe::Offline {
        packets, quality, ..
    } = &mut p.clock
    {
        // NTP seconds/fraction0 is the unknown sentinel (TF029); PTP u32-max is stale.
        packets.insert(
            1,
            packet(
                3,
                1,
                ReferenceSpec::Format2 {
                    payload: vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    data_length: d(12),
                    session: d(1),
                    byte_offset: None,
                    edition: Some(PacketEdition::V24),
                    data_type_version: 0x10,
                    context: ctx.clone(),
                    raw_network_rate: true,
                },
            ),
        );
        // Keep packet IDs in observation order. PTP minimum is seconds32+nanos32.
        packets[2].packet.id = d(4);
        packets.push(packet(
            5,
            20,
            ReferenceSpec::Format2 {
                payload: vec![0x11, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0],
                data_length: d(12),
                session: d(1),
                byte_offset: None,
                edition: Some(PacketEdition::V24),
                data_type_version: 0x10,
                context: ctx,
                raw_network_rate: true,
            },
        ));
        quality.push(QualityQuery {
            session: d(1),
            channel: 7,
        });
    }
    let r = ReportSnapshot::new(p).unwrap();
    assert!(r.findings().iter().any(|f| f.code == 29));
    assert!(r.findings().iter().any(|f| f.code == 21));
    assert_eq!(
        decode::<ReportSnapshot>(encode(&r)).unwrap().findings(),
        r.findings()
    );
}
#[test]
fn interpolate_progress_policy_drain_and_prior_qualification_are_replayable() {
    use irig106_time::stream::*;
    let mut policy = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 5,
            bytes: 100,
            references: 3,
        },
    );
    policy.mode = StreamMode::Interpolate;
    policy.offline.jump_threshold_ns = Some(20_000);
    let mut next = policy.clone();
    next.id = 2;
    next.mode = StreamMode::Standard;
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
                    bytes: d(10),
                },
                StreamStep::Observe {
                    record: packet(2, 5_000_000, ReferenceSpec::None),
                    progress: d(1),
                    bytes: d(10),
                },
                StreamStep::Observe {
                    record: packet(3, 10_000_000, calendar(1, 10_000)),
                    progress: d(2),
                    bytes: d(10),
                },
                StreamStep::Observe {
                    record: packet(4, 15_000_000, ReferenceSpec::None),
                    progress: d(3),
                    bytes: d(10),
                },
                StreamStep::Policy {
                    policy: PolicySnapshot::stream(next).unwrap(),
                    progress: d(4),
                },
                StreamStep::Advance { progress: d(5) },
                StreamStep::Finish { progress: d(6) },
            ],
        },
        requests: vec![],
    };
    let r = ReportSnapshot::new(p).unwrap();
    assert_eq!(r.answers().len(), 2);
    assert_eq!(
        r.answers()[0].view().value().calendar.unwrap().nanoseconds,
        500_005_000
    );
    let irig106_time::answer::AnswerBasis::Counter(c) = r.answers()[1].view().basis() else {
        panic!()
    };
    let irig106_time::answer::ClockBasis::Stream(b) = c.clock() else {
        panic!()
    };
    assert_eq!(b.policy.id, 1);
    assert_eq!(b.release, ReleaseReason::ModeChange);
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
    let mut p = plan();
    if let ClockRecipe::Offline {
        policy, packets, ..
    } = &mut p.clock
    {
        *policy = context(2);
        packets[0].packet.prepare.push(Preparation {
            policy: context(1),
            key: ReplayKey {
                session: d(1),
                ticks: d(0),
                packet: d(1),
            },
        });
    }
    let r = ReportSnapshot::new(p).unwrap();
    let irig106_time::answer::AnswerBasis::Counter(c) = r.answers()[0].view().basis() else {
        panic!()
    };
    let irig106_time::answer::ClockBasis::Offline(b) = c.clock() else {
        panic!()
    };
    assert_eq!(
        b.reference
            .as_ref()
            .unwrap()
            .input
            .qualification
            .as_ref()
            .unwrap()
            .provided_policy()
            .unwrap()
            .id(),
        1
    );
    let wire = encode(&r).json();
    assert!(wire.contains("provided_policy"));
    assert!(wire.contains(r#""id":"1""#));
    assert_eq!(
        encode(&decode::<ReportSnapshot>(encode(&r)).unwrap()),
        encode(&r)
    );
}
