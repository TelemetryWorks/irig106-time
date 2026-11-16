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
