//! Decoder-independent golden vectors: 106-24R1 §11.2.3.3 a/b,
//! Figures11-15..17, printed11-27 / PDF35. §11.2.1 f: little-endian words.
use irig106_time::{NtpTime, PtpTime, TimeF2Csdw, TaiSeconds, TimeError};

#[test]
fn reserved_csdw_positions_are_cleared_without_changing_status() {
    // Fig11-15: NTF2 at7..4, TS1 at3..0 =>0x21;31..8 reserved.
    for bit in 8..32 {
        assert_eq!(TimeF2Csdw::from_raw((1 << bit) | 0x21).to_le_bytes().unwrap(), [0x21,0,0,0]);
    }
    assert_eq!(TimeF2Csdw::from_raw(0xffff_ff20).to_le_bytes().unwrap(), [0x20,0,0,0]);
}

#[test]
fn network_bodies_match_independent_word_order_and_scale_units() {
    // Fig11-16: unsigned seconds0x01020304, fraction0x89abcdef (2^-32s).
    assert_eq!(NtpTime {seconds:0x01020304,fraction:0x89abcdef}.to_le_bytes(),
        [4,3,2,1,0xef,0xcd,0xab,0x89]);
    // Fig11-17: seconds0x01020304, nanos500000000=0x1dcd6500.
    assert_eq!(PtpTime {seconds:0x01020304,nanoseconds:500_000_000}.to_le_bytes().unwrap(),
        [4,3,2,1,0,0x65,0xcd,0x1d]);
}

/// Requirements: L1-ENC-002, L2-ENC-102, L3-ENC-102
#[test]
fn complete_format2_encoder_contract() {
    reserved_csdw_positions_are_cleared_without_changing_status();
    network_bodies_match_independent_word_order_and_scale_units();
    // §11.2.3.3a / Fig11-15: all six defined NTF/TS combinations.
    for (word, bytes) in [(0x00,[0,0,0,0]),(0x01,[1,0,0,0]),
        (0x10,[0x10,0,0,0]),(0x11,[0x11,0,0,0]),
        (0x20,[0x20,0,0,0]),(0x21,[0x21,0,0,0])] {
        assert_eq!(TimeF2Csdw::from_raw(word).to_le_bytes().unwrap(),bytes);
    }
    // Reserved field VALUES cannot become named formats/valid status. The
    // truth table is independent of decoder getters and encoder expressions.
    for ntf in 0..16u32 {
        for ts in 0..16u32 {
            let result=TimeF2Csdw::from_raw(0xffff_ff00 | (ntf << 4) | ts).to_le_bytes();
            let defined=matches!((ntf,ts),(0,0)|(0,1)|(1,0)|(1,1)|(2,0)|(2,1));
            assert_eq!(result.is_ok(),defined,"NTF{ntf} TS{ts}");
            if ntf>2 { assert!(matches!(result,Err(TimeError::OutOfRange {field:"network_time_format",..}))); }
            else if ts>1 { assert!(matches!(result,Err(TimeError::OutOfRange {field:"network_time_status",..}))); }
        }
    }
    // Fig11-16: full fraction range, no decimal-ns rounding or era conversion.
    for (fraction,bytes) in [(0,[0,0,0,0]),(0x8000_0000,[0,0,0,0x80]),
        (u32::MAX,[255,255,255,255])] {
        assert_eq!(NtpTime {seconds:u32::MAX,fraction}.to_le_bytes(),
            [255,255,255,255,bytes[0],bytes[1],bytes[2],bytes[3]]);
    }
    // Zero sentinel remains raw zero, never fabricated as a valid epoch/time.
    assert_eq!(NtpTime {seconds:0,fraction:0}.to_le_bytes(),[0;8]);
    // Fig11-17: nanoseconds are decimal1e-9s; max999999999=0x3b9ac9ff.
    for (seconds,nanos,bytes) in [(0,0,[0;8]),(u32::MAX,999_999_999,
        [255,255,255,255,255,0xc9,0x9a,0x3b])] {
        let time=PtpTime::from_tai_seconds(TaiSeconds::new(seconds.into()),nanos).unwrap();
        assert_eq!(time.to_le_bytes().unwrap(),bytes);
    }
    for nanos in [1_000_000_000,u32::MAX] {
        assert!(matches!(PtpTime {seconds:0,nanoseconds:nanos}.to_le_bytes(),
            Err(TimeError::OutOfRange {field:"ptp_nanoseconds",..})));
        assert!(PtpTime::from_tai_seconds(TaiSeconds::new(0),nanos).is_err());
    }
    for seconds in [1u64<<32,u64::MAX] {
        assert!(matches!(PtpTime::from_tai_seconds(TaiSeconds::new(seconds),0),
            Err(TimeError::QuantityOverflow {..})));
    }
    // Canonicalization does not mutate the original diagnostic word.
    let raw=TimeF2Csdw::from_raw(0xaabb_cc10);
    assert_eq!(raw.to_le_bytes().unwrap(),[0x10,0,0,0]);
    assert_eq!(raw.as_raw(),0xaabb_cc10);
}
