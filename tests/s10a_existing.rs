//! Existing raw baselines: 106-24R1 Figures11-5/6 (11-10/PDF18).
use irig106_time::secondary::*;
#[test]
fn ieee_distinct_words_and_bounds_are_not_a_format2_body() {
    let b = [0x15, 0xcd, 0x5b, 7, 4, 3, 2, 1, 0, 0, 0x4e, 1];
    let SecondaryHeaderTime::Ieee1588(t) =
        parse_secondary_header(&b, SecHdrTimeFormat::Ieee1588).unwrap()
    else {
        panic!("IEEE expected")
    };
    assert_eq!((t.seconds, t.nanoseconds), (0x01020304, 123456789));
    // ns=10^9 ->00 ca 9a 3b, seconds0, byte sum019f.
    let bad = [0, 0xca, 0x9a, 0x3b, 0, 0, 0, 0, 0, 0, 0x9f, 1];
    assert!(parse_secondary_header(&bad, SecHdrTimeFormat::Ieee1588).is_err());
}
