//! Unit tests for the `secondary` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `sec_hdr_time_format_ch4` | Packet flags bits [3:2]=0 → Ch4 | L3-SEC-001 |
//! | `sec_hdr_time_format_ieee1588` | Packet flags bits [3:2]=1 → Ieee1588 | L3-SEC-001 |
//! | `sec_hdr_time_format_ertc` | Packet flags bits [3:2]=2 → Ertc | L3-SEC-001 |
//! | `sec_hdr_time_format_reserved` | Packet flags bits [3:2]=3 → Reserved | L3-SEC-001 |
//! | `checksum_valid` | Correct checksum passes | L3-SEC-002 |
//! | `checksum_invalid` | Wrong checksum returns ChecksumMismatch | L3-SEC-002 |
//! | `checksum_buffer_too_short` | <12 bytes returns BufferTooShort | L3-SEC-002 |
//! | `parse_ieee1588_valid` | Full IEEE-1588 secondary header parse | L3-SEC-004 |
//! | `parse_ertc_valid` | Full ERTC secondary header parse | L3-SEC-004 |
//! | `parse_reserved_rejected` | Reserved format returns error | L3-SEC-004 |

use super::*;

/// Build a 12-byte secondary header buffer with a valid checksum.
/// Fills bytes [0..10] with `data`, computes checksum into [10..12].
fn make_sec_hdr(data: &[u8; 10]) -> [u8; 12] {
    let mut buf = [0u8; 12];
    buf[0..10].copy_from_slice(data);
    let mut sum: u32 = 0;
    for i in 0..5 {
        let word = u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]);
        sum = sum.wrapping_add(word as u32);
    }
    let checksum = (sum & 0xFFFF) as u16;
    buf[10..12].copy_from_slice(&checksum.to_le_bytes());
    buf
}

#[test]
fn sec_hdr_time_format_ch4() {
    // bits [3:2] = 0b00 → flags & 0x0C = 0x00
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0x00),
        SecHdrTimeFormat::Ch4
    );
}

#[test]
fn sec_hdr_time_format_ieee1588() {
    // bits [3:2] = 0b01 → flags & 0x0C = 0x04
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0x04),
        SecHdrTimeFormat::Ieee1588
    );
}

#[test]
fn sec_hdr_time_format_ertc() {
    // bits [3:2] = 0b10 → flags & 0x0C = 0x08
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0x08),
        SecHdrTimeFormat::Ertc
    );
}

#[test]
fn sec_hdr_time_format_reserved() {
    // bits [3:2] = 0b11 → flags & 0x0C = 0x0C
    assert_eq!(
        SecHdrTimeFormat::from_packet_flags(0x0C),
        SecHdrTimeFormat::Reserved(3)
    );
}

#[test]
fn checksum_valid() {
    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
    let buf = make_sec_hdr(&data);
    assert!(validate_secondary_checksum(&buf).is_ok());
}

#[test]
fn checksum_invalid() {
    let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A];
    let mut buf = make_sec_hdr(&data);
    buf[10] = 0xFF; // corrupt checksum
    buf[11] = 0xFF;
    let result = validate_secondary_checksum(&buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::ChecksumMismatch { .. } => {}
        other => panic!("expected ChecksumMismatch, got {other:?}"),
    }
}

#[test]
fn checksum_buffer_too_short() {
    let buf = [0u8; 10];
    let result = validate_secondary_checksum(&buf);
    assert!(result.is_err());
    match result.unwrap_err() {
        TimeError::BufferTooShort {
            expected: 12,
            actual: 10,
        } => {}
        other => panic!("expected BufferTooShort, got {other:?}"),
    }
}

#[test]
fn parse_ieee1588_valid() {
    // Encode 500_000_000 ns, 1000 seconds in LE
    let mut data = [0u8; 10];
    let ns: u32 = 500_000_000;
    let secs: u32 = 1000;
    data[0..4].copy_from_slice(&ns.to_le_bytes());
    data[4..8].copy_from_slice(&secs.to_le_bytes());
    // bytes [8..10] are reserved (zero is fine)
    let buf = make_sec_hdr(&data);
    let result = parse_secondary_header(&buf, SecHdrTimeFormat::Ieee1588).unwrap();
    match result {
        SecondaryHeaderTime::Ieee1588(t) => {
            assert_eq!(t.nanoseconds, 500_000_000);
            assert_eq!(t.seconds, 1000);
        }
        other => panic!("expected Ieee1588, got {other:?}"),
    }
}

#[test]
fn parse_ertc_valid() {
    let mut data = [0u8; 10];
    let val: u64 = 12345678;
    data[0..8].copy_from_slice(&val.to_le_bytes());
    let buf = make_sec_hdr(&data);
    let result = parse_secondary_header(&buf, SecHdrTimeFormat::Ertc).unwrap();
    match result {
        SecondaryHeaderTime::Ertc(e) => {
            assert_eq!(e.as_raw(), 12345678);
        }
        other => panic!("expected Ertc, got {other:?}"),
    }
}

#[test]
fn parse_reserved_rejected() {
    let data = [0u8; 10];
    let buf = make_sec_hdr(&data);
    let result = parse_secondary_header(&buf, SecHdrTimeFormat::Reserved(3));
    assert!(result.is_err());
}
