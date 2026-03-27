//! Unit tests for the `error` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `display_invalid_bcd_digit` | `Display` for `InvalidBcdDigit` | L3-ERR-002 |
//! | `display_reserved_bit_set` | `Display` for `ReservedBitSet` | L3-ERR-002 |
//! | `display_out_of_range` | `Display` for `OutOfRange` | L3-ERR-002 |
//! | `display_checksum_mismatch` | `Display` for `ChecksumMismatch` | L3-ERR-002 |
//! | `display_no_reference_point` | `Display` for `NoReferencePoint` | L3-ERR-002 |
//! | `display_buffer_too_short` | `Display` for `BufferTooShort` | L3-ERR-002 |
//! | `error_is_clone_eq` | Derive traits on `TimeError` | L3-ERR-001 |

use super::*;
use alloc::format;

extern crate alloc;

#[test]
fn display_invalid_bcd_digit() {
    let err = TimeError::InvalidBcdDigit {
        nibble: 0xA,
        position: "tens of seconds",
    };
    let msg = format!("{err}");
    assert!(msg.contains("10"));
    assert!(msg.contains("tens of seconds"));
}

#[test]
fn display_reserved_bit_set() {
    let err = TimeError::ReservedBitSet {
        position: "word 0 bit 15",
    };
    let msg = format!("{err}");
    assert!(msg.contains("reserved bit set"));
    assert!(msg.contains("word 0 bit 15"));
}

#[test]
fn display_out_of_range() {
    let err = TimeError::OutOfRange {
        field: "hours",
        value: 25,
        max: 23,
    };
    let msg = format!("{err}");
    assert!(msg.contains("hours"));
    assert!(msg.contains("25"));
    assert!(msg.contains("23"));
}

#[test]
fn display_checksum_mismatch() {
    let err = TimeError::ChecksumMismatch {
        stored: 0x1234,
        computed: 0x5678,
    };
    let msg = format!("{err}");
    assert!(msg.contains("1234"));
    assert!(msg.contains("5678"));
}

#[test]
fn display_no_reference_point() {
    let err = TimeError::NoReferencePoint;
    let msg = format!("{err}");
    assert!(msg.contains("no time reference point"));
}

#[test]
fn display_buffer_too_short() {
    let err = TimeError::BufferTooShort {
        expected: 12,
        actual: 8,
    };
    let msg = format!("{err}");
    assert!(msg.contains("12"));
    assert!(msg.contains("8"));
}

#[test]
fn error_is_clone_eq() {
    let err = TimeError::NoReferencePoint;
    let cloned = err.clone();
    assert_eq!(err, cloned);
}
