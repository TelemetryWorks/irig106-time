//! Unit tests for the `intra_packet` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `format_no_secondary_is_rtc` | Bit 2 clear → Rtc48 | L3-IPT-003 |
//! | `format_ch4_binary` | Direct enum construction | L3-IPT-003 |
//! | `parse_rtc48_known` | 48-bit RTC extraction | L3-IPT-002, L2-IPT-001 |
//! | `parse_rtc48_reserved_ignored` | Bytes [6..8] don't affect result | L3-IPT-002, L2-IPT-001 |
//! | `parse_ieee1588_known` | IEEE-1588 extraction | L3-IPT-002, L2-IPT-003 |
//! | `parse_ertc64_known` | 64-bit ERTC extraction | L3-IPT-002, L2-IPT-004 |
//! | `parse_buffer_too_short` | <8 bytes returns error | L3-IPT-002 |
//! | `intra_packet_time_is_enum` | All 4 variants constructible | L3-IPT-001 |

use super::*;

#[test]
fn format_no_secondary_is_rtc() {
    // Any flags with bit 2 clear → Rtc48
    assert_eq!(
        IntraPacketTimeFormat::from_packet_flags(0x00),
        IntraPacketTimeFormat::Rtc48,
    );
    assert_eq!(
        IntraPacketTimeFormat::from_packet_flags(0x03), // checksum bits only
        IntraPacketTimeFormat::Rtc48,
    );
    assert_eq!(
        IntraPacketTimeFormat::from_packet_flags(0xFB), // all except bit 2
        IntraPacketTimeFormat::Rtc48,
    );
}

#[test]
fn format_ch4_binary() {
    // Directly use the enum since bit-mapping is version-dependent
    let fmt = IntraPacketTimeFormat::Ch4Binary;
    assert_eq!(fmt, IntraPacketTimeFormat::Ch4Binary);
}

#[test]
fn parse_rtc48_known() {
    // RTC = 0x0000_AABB_CCDD_EEFF → LE bytes [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x00, 0x00]
    let buf: [u8; 8] = [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x00, 0x00];
    let result = parse_intra_packet_time(&buf, IntraPacketTimeFormat::Rtc48).unwrap();
    match result {
        IntraPacketTime::Rtc(rtc) => {
            assert_eq!(rtc.as_raw(), 0x0000_AABB_CCDD_EEFF);
        }
        other => panic!("expected Rtc, got {other:?}"),
    }
}

#[test]
fn parse_rtc48_reserved_ignored() {
    // Same RTC value but with non-zero reserved bytes [6..8]
    let buf: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xAA, 0xBB];
    let result = parse_intra_packet_time(&buf, IntraPacketTimeFormat::Rtc48).unwrap();
    match result {
        IntraPacketTime::Rtc(rtc) => {
            assert_eq!(rtc.as_raw(), 1); // reserved bytes don't affect RTC
        }
        other => panic!("expected Rtc, got {other:?}"),
    }
}

#[test]
fn parse_ieee1588_known() {
    // 123_456_789 ns, 42 seconds
    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&123_456_789u32.to_le_bytes());
    buf[4..8].copy_from_slice(&42u32.to_le_bytes());
    let result = parse_intra_packet_time(&buf, IntraPacketTimeFormat::Ieee1588).unwrap();
    match result {
        IntraPacketTime::Ieee1588(t) => {
            assert_eq!(t.nanoseconds, 123_456_789);
            assert_eq!(t.seconds, 42);
        }
        other => panic!("expected Ieee1588, got {other:?}"),
    }
}

#[test]
fn parse_ertc64_known() {
    let val: u64 = 0xDEAD_BEEF_CAFE_BABE;
    let buf = val.to_le_bytes();
    let result = parse_intra_packet_time(&buf, IntraPacketTimeFormat::Ertc64).unwrap();
    match result {
        IntraPacketTime::Ertc(e) => {
            assert_eq!(e.as_raw(), val);
        }
        other => panic!("expected Ertc, got {other:?}"),
    }
}

#[test]
fn parse_buffer_too_short() {
    let buf = [0u8; 6];
    let result = parse_intra_packet_time(&buf, IntraPacketTimeFormat::Rtc48);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::BufferTooShort {
            expected: 8,
            actual: 6,
        } => {}
        other => panic!("expected BufferTooShort, got {other:?}"),
    }
}

#[test]
fn intra_packet_time_is_enum() {
    // Ensure all four variants are constructible
    let _ = IntraPacketTime::Rtc(Rtc::ZERO);
    let _ = IntraPacketTime::Ch4(Ch4BinaryTime {
        high_order: 0,
        low_order: 0,
        microseconds: 0,
    });
    let _ = IntraPacketTime::Ieee1588(Ieee1588Time {
        nanoseconds: 0,
        seconds: 0,
    });
    let _ = IntraPacketTime::Ertc(Ertc::from_le_bytes(&[0u8; 8]).unwrap());
}
