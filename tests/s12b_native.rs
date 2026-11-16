#![cfg(feature = "serde")]
//! Leaf-load guards, partial API110 clauses.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::{format1::CalendarReading, scale::*, *};
use serde_tree::{Node, decode, encode};
fn calendar() -> CalendarReading {
    CalendarReading {
        year: Some(2016),
        day_of_year: 366,
        hours: 23,
        minutes: 59,
        seconds: 60,
        nanoseconds: 999_999_999,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    }
}
#[test]
fn native_calendar_load_validates_invariants() {
    let cal = calendar();
    assert_eq!(decode::<CalendarReading>(encode(&cal)).unwrap(), cal);
    let wire = encode(&cal);
    for (key, bad) in [
        ("day_of_year", 0),
        ("hours", 24),
        ("seconds", 61),
        ("nanoseconds", 1_000_000_000),
    ] {
        let mut w = wire.clone();
        *w.field(key) = Node::Number(bad.to_string());
        assert!(decode::<CalendarReading>(w).is_err(), "{key}");
    }
}
#[test]
fn native_scale_load_validates_provenance() {
    let scale = ScaleEvidence::assumed(TimeScale::Gps);
    assert_eq!(decode::<ScaleEvidence>(encode(&scale)).unwrap(), scale);
    let mut bad = encode(&scale);
    *bad.field("stated") = Node::String("Utc".into());
    assert!(decode::<ScaleEvidence>(bad).is_err());
}

#[test]
fn native_calendar_time_load_checks_consistent_date() {
    let time = CalendarTime::new(
        AbsoluteTime::new(1, 0, 0, 0, 0)
            .unwrap()
            .with_year(Some(2017))
            .unwrap(),
        1,
        1,
    )
    .unwrap();
    let mut w = encode(&time);
    *w.field("month") = Node::Number("2".into());
    assert!(decode::<CalendarTime>(w).is_err());
}
#[test]
fn native_ptp_load_checks_fraction_range() {
    let mut w = encode(&PtpTime {
        seconds: 1,
        nanoseconds: 0,
    });
    *w.field("nanoseconds") = Node::Number("1000000000".into());
    assert!(decode::<PtpTime>(w).is_err());
}
