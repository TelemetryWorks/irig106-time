#![cfg(feature = "serde")]
//! Partial API107/110 and POL103 wire clauses; schema remains a review candidate.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{snapshot::*, *};
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
    println!("SNAPSHOT_JSON {}", wire.json());
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

#[test]
fn every_scalar_kind_keeps_units_range_sign_and_shared_type() {
    for value in [
        ScalarValue::Rtc(Rtc::from_raw((1 << 48) - 1)),
        ScalarValue::Ertc(Ertc::from_le_bytes(&u64::MAX.to_le_bytes()).unwrap()),
        ScalarValue::Nanos(NanosDuration::new(u64::MAX)),
        ScalarValue::SignedNanos(SignedNanos::new(i128::MIN)),
        ScalarValue::SignedTicks(SignedRtcTicks::new(i128::MAX)),
        ScalarValue::Unix(UnixSeconds::new(u64::MAX)),
        ScalarValue::SignedUnix(SignedUnixSeconds::new(-1)),
        ScalarValue::Tai(TaiSeconds::new(u64::MAX)),
        ScalarValue::U64(u64::MAX),
        ScalarValue::I64(i64::MIN),
        ScalarValue::U128(u128::MAX),
        ScalarValue::I128(i128::MAX),
    ] {
        let s = ScalarSnapshot::new(value).unwrap();
        let wire = encode(&s);
        let Node::Map(fields) = &wire else { panic!() };
        assert!(matches!(
            fields.iter().find(|(k, _)| k == "value").unwrap().1,
            Node::String(_)
        ));
        assert_eq!(decode::<ScalarSnapshot>(wire).unwrap().value(), value);
    }
    for n in [i64::MIN, i64::MAX] {
        assert_eq!(
            decode::<Decimal<i64>>(encode(&Decimal::new(n)))
                .unwrap()
                .get(),
            n
        );
    }
    for n in [
        "18446744073709551616",
        "-9223372036854775809",
        "340282366920938463463374607431768211456",
    ] {
        assert!(decode::<Decimal<u64>>(Node::String(n.into())).is_err());
    }
}
#[test]
fn calendar_and_leap_snapshots_validate_units_versions_content_and_immutability() {
    use irig106_time::{format1::CalendarReading, scale::*};
    let cal = CalendarReading {
        year: None,
        day_of_year: 1,
        hours: 0,
        minutes: 0,
        seconds: 0,
        nanoseconds: 0,
        scale: ScaleEvidence::assumed(TimeScale::Gps),
    };
    assert!(
        CalendarSnapshot::new(CalendarReading {
            day_of_year: 0,
            ..cal
        })
        .is_err()
    );
    let snap = CalendarSnapshot::new(cal).unwrap();
    let wire = encode(&snap);
    assert_eq!(decode::<CalendarSnapshot>(wire.clone()).unwrap(), snap);
    let mut bad = wire.clone();
    *bad.field("version") = Node::Number("0".into());
    assert!(decode::<CalendarSnapshot>(bad).is_err());
    let mut duplicate_units = wire.clone();
    let Node::Map(fields) = duplicate_units.field("units") else {
        panic!()
    };
    fields.push(("nanoseconds".into(), Node::String("ns".into())));
    assert!(decode::<CalendarSnapshot>(duplicate_units).is_err());
    let mut bad = wire.clone();
    *bad.field("units").field("nanoseconds") = Node::String("us".into());
    assert!(decode::<CalendarSnapshot>(bad).is_err());
    let mut bad = wire;
    *bad.field("value").field("day_of_year") = Node::Number("0".into());
    assert!(decode::<CalendarSnapshot>(bad).is_err());
    let table = LeapSecondTable::builtin();
    let snapshot = LeapTableSnapshot::new(table.clone());
    let wire = encode(&snapshot);
    println!("SNAPSHOT_JSON {}", wire.json());
    let Node::Map(fields) = &wire else { panic!() };
    assert_eq!(fields.len(), 4);
    let mut payload = wire.clone();
    assert!(matches!(
        payload
            .field("value")
            .field("metadata")
            .field("supported_start"),
        Node::String(_)
    ));
    assert_eq!(
        decode::<LeapTableSnapshot>(wire.clone()).unwrap().value(),
        &table
    );
    let mut bad = wire.clone();
    *bad.field("units").field("effective_unix") = Node::String("tai_s".into());
    assert!(decode::<LeapTableSnapshot>(bad).is_err());
    let mut bad = wire.clone();
    *bad.field("value")
        .field("metadata")
        .field("known_current_through") = Node::String("1".into());
    assert!(decode::<LeapTableSnapshot>(bad).is_err());
    let mut bad = wire.clone();
    *bad.field("version") = Node::Number("2".into());
    assert!(decode::<LeapTableSnapshot>(bad).is_err());
    let mut bad = wire.clone();
    let Node::Seq(entries) = bad.field("value").field("entries") else {
        panic!()
    };
    entries.push(entries[0].clone());
    assert!(decode::<LeapTableSnapshot>(bad).is_err());
    let mut bad = wire.clone();
    let Node::Map(fields) = bad.field("value").field("metadata") else {
        panic!()
    };
    fields.push(("unknown_source".into(), Node::String("future".into())));
    assert!(decode::<LeapTableSnapshot>(bad).is_err());
    assert_eq!(encode(&snapshot), wire);
}
#[test]
fn snapshot_maps_are_strict_and_old_shared_source_names_do_not_load() {
    let wire = encode(&ScalarSnapshot::new(ScalarValue::U64(1)).unwrap());
    let Node::Map(fields) = wire.clone() else {
        panic!()
    };
    assert!(
        decode::<ScalarSnapshot>(Node::Seq(fields.into_iter().map(|(_, v)| v).collect())).is_err()
    );
    let mut missing = wire.clone();
    let Node::Map(fields) = &mut missing else {
        panic!()
    };
    fields.retain(|(k, _)| k != "version");
    assert!(decode::<ScalarSnapshot>(missing).is_err());
    let mut duplicate = wire.clone();
    let Node::Map(fields) = &mut duplicate else {
        panic!()
    };
    fields.push(("version".into(), Node::Number("1".into())));
    assert!(decode::<ScalarSnapshot>(duplicate).is_err());
    let mut unknown = wire;
    let Node::Map(fields) = &mut unknown else {
        panic!()
    };
    fields.push(("future".into(), Node::Bool(true)));
    assert!(decode::<ScalarSnapshot>(unknown).is_err());
    assert!(decode::<ScalarSnapshot>(Node::Seq(vec![])).is_err());
    for name in ["Gps", "InternalRtc"] {
        assert!(decode::<TimeSource>(Node::String(name.into())).is_err());
    }
    assert_eq!(
        decode::<TimeSource>(Node::String("InternalFromRmm".into())).unwrap(),
        TimeSource::InternalFromRmm
    );
    assert_eq!(
        decode::<TimeSource>(Node::Map(vec![(
            "Reserved".into(),
            Node::Number("3".into())
        )]))
        .unwrap(),
        TimeSource::Reserved(3)
    );
    assert_eq!(
        decode::<TimeFormat>(Node::String("Gps".into())).unwrap(),
        TimeFormat::Gps
    );
    assert_eq!(
        decode::<IrigTimeSource>(Node::String("ExternalGps".into())).unwrap(),
        IrigTimeSource::ExternalGps
    );
}
