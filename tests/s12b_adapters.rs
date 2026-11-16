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
    assert_eq!(decode::<network_time::Format2Reading>(wire).unwrap(), f2);
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
    assert_eq!(decode::<format1::Format1Reading>(wire).unwrap(), f1);
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
