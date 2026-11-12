//! Hand-derived fixtures, ADR-0016. No fixture uses the implementation encoder.
use irig106_time::network_time::{NetworkTimeProtocol, NtpTime, PtpTime, TimeF2Csdw};
use irig106_time::secondary::validate_secondary_checksum;

/// Requirements: L1-F2CSDW-004, L2-F2CSDW-104, L3-F2CSDW-104
#[test]
fn t2_ntf_is_high_nibble_not_valid_status() {
    // 106-24R1 Fig 11-15, printed 11-27/PDF35: TS=1, NTF=0.
    assert_eq!(
        TimeF2Csdw::from_raw(0x01).time_protocol(),
        NetworkTimeProtocol::Ntp
    );
}

/// Requirements: L1-F2CSDW-004, L2-F2CSDW-104, L3-F2CSDW-104
#[test]
fn t2_reserved_ntf_does_not_become_ptp_from_ts() {
    // Fig 11-15: NTF=8 reserved independently of TS=1.
    assert_eq!(
        TimeF2Csdw::from_raw(0x81).time_protocol(),
        NetworkTimeProtocol::Reserved(8)
    );
}

/// Requirements: L1-F2CSDW-006, L2-F2CSDW-106, L3-F2CSDW-106
#[test]
fn t2_defined_ntf_bits_are_not_reserved() {
    // Fig 11-15: reserved starts at bit8, not bit4.
    for word in [0x11, 0x21] {
        assert!(TimeF2Csdw::from_raw(word).validate_reserved().is_ok());
    }
}

/// Requirements: L1-PTP-005, L2-PTP-105, L3-PTP-105
#[test]
fn t3_ptp_has_two_u32_words() {
    // 106-24R1 Fig 11-17, 11-27/PDF35; 106-17 Fig 11-16, 11-24/PDF32.
    let bytes = [4, 3, 2, 1, 0, 0x65, 0xcd, 0x1d]; // seconds=0x01020304, ns=500000000.
    let t = PtpTime::from_le_bytes(&bytes).unwrap();
    assert_eq!(
        (u64::from(t.seconds), t.nanoseconds),
        (0x01020304, 500_000_000)
    );
    assert_eq!(&t.to_le_bytes()[..], &bytes);
    let maximum = PtpTime::from_le_bytes(&[255, 255, 255, 255, 0, 0, 0, 0]).unwrap();
    assert_eq!(u64::from(maximum.seconds), u32::MAX as u64);
}

/// Requirements: L1-PTP-005, L2-PTP-105, L3-PTP-105, L1-ERR-005, L2-ERR-105, L3-ERR-105
#[test]
fn t3_ptp_never_reads_following_filler() {
    // Same two words, then nonzero bytes outside the eight-byte body.
    let bytes = [4, 3, 2, 1, 0, 0x65, 0xcd, 0x1d, 0xff, 0xff];
    let t = PtpTime::from_le_bytes(&bytes).unwrap();
    assert_eq!(
        (u64::from(t.seconds), t.nanoseconds),
        (0x01020304, 500_000_000)
    );
    for n in 0..8 {
        assert!(PtpTime::from_le_bytes(&bytes[..n]).is_err());
    }
}

/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104
#[test]
fn q_ntp_absolute_conversion_requires_context() {
    // Fig 11-16 is a raw 32-bit counter, not an era declaration.
    let t = NtpTime {
        seconds: 2_208_989_800,
        fraction: 0x80000000,
    };
    assert!(format!("{:?}", t.to_absolute()).contains("NeedsContext"));
    assert!(format!("{:?}", t.to_nanos_since_ntp_epoch()).contains("NeedsContext"));
    assert_eq!(t.raw_fixed_point_nanos(), 2_208_989_800_500_000_000);
}

/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104
#[test]
fn q_ntp_unix_conversion_requires_context() {
    let t = NtpTime {
        seconds: 2_208_989_800,
        fraction: 0,
    };
    assert!(format!("{:?}", t.to_unix_seconds()).contains("NeedsContext"));
}

/// Requirements: L1-SEC-004, L2-SEC-104, L3-SEC-104
/// Interpretations: RDG-008
#[test]
fn rdg008_byte_only_checksum_is_valid() {
    // §11.2.1.2c, 11-10/PDF18; RCC123-20 B-31/PDF137.
    // Bytes 1+2=3; LE word sum 0x0201. Stored=3 matches bytes only.
    assert!(validate_secondary_checksum(&[1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0]).is_ok());
}

/// Requirements: L1-SEC-004, L2-SEC-104, L3-SEC-104
/// Interpretations: RDG-008
#[test]
fn rdg008_word_only_checksum_is_not_trusted() {
    assert!(validate_secondary_checksum(&[1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2]).is_err());
}
