#![cfg(feature = "serde")]
//! Q5 adapter clauses; whole API107/110 are not claimed by these leaf tests.
#[path = "support/serde_tree.rs"]
mod serde_tree;
use irig106_time::*;
use serde_tree::{Node, decode, encode};
#[test]
fn ptp_seconds_and_consumed_lengths_are_exact_strings() {
    let p = PtpTime {
        seconds: u32::MAX,
        nanoseconds: 17,
    };
    let mut wire = encode(&p);
    assert_eq!(wire.field("seconds"), &Node::String(u32::MAX.to_string()));
    assert_eq!(decode::<PtpTime>(wire).unwrap(), p);
}
#[test]
fn format2_consumed_is_a_decimal_string() {
    let f2 = network_time::Format2Reading {
        csdw: TimeF2Csdw::from_raw(0x11),
        time: None,
        consumed: usize::MAX,
    };
    let mut wire = encode(&f2);
    assert_eq!(
        wire.field("consumed"),
        &Node::String(usize::MAX.to_string())
    );
}
#[test]
fn format1_consumed_is_a_decimal_string() {
    let f1 = format1::Format1Reading {
        csdw: TimeF1Csdw::from_raw(0),
        time: None,
        reserved_body_bits: [0; 4],
        consumed: usize::MAX,
    };
    let mut wire = encode(&f1);
    assert_eq!(
        wire.field("consumed"),
        &Node::String(usize::MAX.to_string())
    );
}
#[test]
fn native_rtc_ertc_adapter_variants_never_emit_wide_numbers() {
    for raw in [0, (1u64 << 48) - 1] {
        let t = IntraPacketTime::Rtc(Rtc::from_raw(raw));
        let wire = encode(&t);
        assert_eq!(wire.json(), format!(r#"{{"Rtc":"{raw}"}}"#));
        assert_eq!(decode::<IntraPacketTime>(wire).unwrap(), t);
    }
}
#[test]
fn native_ertc_variants_are_decimal_strings() {
    for raw in [0, u64::MAX] {
        let e = Ertc::from_le_bytes(&raw.to_le_bytes()).unwrap();
        let t = IntraPacketTime::Ertc(e);
        let wire = encode(&t);
        assert_eq!(wire.json(), format!(r#"{{"Ertc":"{raw}"}}"#));
        assert_eq!(decode::<IntraPacketTime>(wire).unwrap(), t);
        let h = SecondaryHeaderTime::Ertc(e);
        let wire = encode(&h);
        assert_eq!(wire.json(), format!(r#"{{"Ertc":"{raw}"}}"#));
        assert_eq!(decode::<SecondaryHeaderTime>(wire).unwrap(), h);
    }
}

#[test]
fn adapter_loaders_reject_numeric_noncanonical_overflow_and_counter_width() {
    let mut p = encode(&PtpTime {
        seconds: 17,
        nanoseconds: 0,
    });
    for bad in [
        Node::Number("17".into()),
        Node::String("01".into()),
        Node::String("4294967296".into()),
    ] {
        *p.field("seconds") = bad;
        assert!(decode::<PtpTime>(p.clone()).is_err());
    }
    let mut r = encode(&IntraPacketTime::Rtc(Rtc::from_raw(0)));
    for bad in [
        Node::Number("0".into()),
        Node::String((1u64 << 48).to_string()),
        Node::String("-1".into()),
    ] {
        *r.field("Rtc") = bad;
        assert!(decode::<IntraPacketTime>(r.clone()).is_err());
    }
    let mut e = encode(&SecondaryHeaderTime::Ertc(
        Ertc::from_le_bytes(&[0; 8]).unwrap(),
    ));
    for bad in [
        Node::Number("0".into()),
        Node::String("18446744073709551616".into()),
        Node::String("-1".into()),
    ] {
        *e.field("Ertc") = bad;
        assert!(decode::<SecondaryHeaderTime>(e.clone()).is_err());
    }
}
