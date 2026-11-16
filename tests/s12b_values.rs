#![cfg(feature = "serde")]
//! Partial API107/110 and POL103 wire clauses; schema remains a review candidate.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{format1::CalendarReading, scale::*, snapshot::*, *};
use serde_tree::{Node, decode, encode};
#[test]
fn exact_decimal_boundaries_and_json_protocol() {
    for n in [u64::MAX, 9_007_199_254_740_993] {
        let d = Decimal::new(n);
        let node = encode(&d);
        assert_eq!(node.json(), format!("\"{n}\""));
        assert_eq!(decode::<Decimal<u64>>(node).unwrap().get(), n);
    }
    for n in [i128::MIN, i128::MAX, -1, 0] {
        let node = encode(&Decimal::new(n));
        assert_eq!(node.json(), format!("\"{n}\""));
        assert_eq!(decode::<Decimal<i128>>(node).unwrap().get(), n);
    }
    let node = encode(&Decimal::new(u128::MAX));
    assert_eq!(node.json(), format!("\"{}\"", u128::MAX));
    assert_eq!(decode::<Decimal<u128>>(node).unwrap().get(), u128::MAX);
    // Shared i128 serde defaults to a numeric token: never use it directly in reports.
    assert!(matches!(
        encode(&SignedNanos::new(i128::MAX)),
        Node::Number(_)
    ));
    for text in [
        "+1",
        "01",
        "-0",
        " 1",
        "1 ",
        "1e3",
        "1.0",
        "",
        "170141183460469231731687303715884105728",
    ] {
        assert!(
            decode::<Decimal<i128>>(Node::String(text.into())).is_err(),
            "{text}"
        );
    }
    assert!(decode::<Decimal<u64>>(Node::Number("1".into())).is_err());
    assert!(decode::<Decimal<u64>>(Node::String("-1".into())).is_err());
}
#[test]
fn scalar_snapshot_requires_version_units_width_and_exact_wide_shared_values() {
    let s = ScalarSnapshot::new(ScalarValue::SignedNanos(SignedNanos::new(i128::MIN))).unwrap();
    let wire = encode(&s);
    assert_eq!(
        wire.json(),
        format!(
            "{{\"schema\":\"irig106-time/scalar\",\"version\":1,\"kind\":\"signed_nanos\",\"unit\":\"ns\",\"value\":\"{}\"}}",
            i128::MIN
        )
    );
    assert_eq!(decode::<ScalarSnapshot>(wire.clone()).unwrap(), s);
    for (key, bad) in [
        ("schema", Node::String("other".into())),
        ("version", Node::Number("2".into())),
        ("kind", Node::String("future".into())),
        ("unit", Node::String("rtc_ticks".into())),
        ("value", Node::Number("1".into())),
    ] {
        let mut bad_wire = wire.clone();
        *bad_wire.field(key) = bad;
        assert!(decode::<ScalarSnapshot>(bad_wire).is_err(), "{key}");
    }
    let mut rtc = encode(&ScalarSnapshot::new(ScalarValue::Rtc(Rtc::from_raw(0))).unwrap());
    *rtc.field("value") = Node::String((1u64 << 48).to_string());
    assert!(decode::<ScalarSnapshot>(rtc).is_err());
}
