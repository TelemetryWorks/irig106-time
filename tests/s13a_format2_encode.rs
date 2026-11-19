//! Decoder-independent golden vectors: 106-24R1 §11.2.3.3 a/b,
//! Figures11-15..17, printed11-27 / PDF35. §11.2.1 f: little-endian words.
use irig106_time::{NtpTime, PtpTime, TimeF2Csdw};

#[test]
fn reserved_csdw_positions_are_cleared_without_changing_status() {
    // Fig11-15: NTF2 at7..4, TS1 at3..0 =>0x21;31..8 reserved.
    for bit in 8..32 {
        assert_eq!(TimeF2Csdw::from_raw((1 << bit) | 0x21).to_le_bytes(), [0x21,0,0,0]);
    }
    assert_eq!(TimeF2Csdw::from_raw(0xffff_ff20).to_le_bytes(), [0x20,0,0,0]);
}

#[test]
fn network_bodies_match_independent_word_order_and_scale_units() {
    // Fig11-16: unsigned seconds0x01020304, fraction0x89abcdef (2^-32s).
    assert_eq!(NtpTime {seconds:0x01020304,fraction:0x89abcdef}.to_le_bytes(),
        [4,3,2,1,0xef,0xcd,0xab,0x89]);
    // Fig11-17: seconds0x01020304, nanos500000000=0x1dcd6500.
    assert_eq!(PtpTime {seconds:0x01020304,nanoseconds:500_000_000}.to_le_bytes(),
        [4,3,2,1,0,0x65,0xcd,0x1d]);
}
