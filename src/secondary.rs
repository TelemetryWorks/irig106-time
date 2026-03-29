//! Secondary header time format parsing.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-SEC-001..004 | Secondary header enum, checksum, parsing |

use crate::absolute::{Ch4BinaryTime, Ertc, Ieee1588Time};
use crate::error::{Result, TimeError};

/// Time format discriminant derived from Packet Flag bits \[3:2\].
///
/// **Traces:** L3-SEC-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecHdrTimeFormat {
    /// IRIG 106 Chapter 4 Binary Weighted Time.
    Ch4,
    /// IEEE-1588 Precision Time Protocol.
    Ieee1588,
    /// Extended Relative Time Counter (64-bit).
    Ertc,
    /// Reserved/unknown format.
    Reserved(u8),
}

impl SecHdrTimeFormat {
    /// Decode from Packet Flag bits \[3:2\].
    pub fn from_packet_flags(flags: u8) -> Self {
        match (flags >> 2) & 0x03 {
            0 => SecHdrTimeFormat::Ch4,
            1 => SecHdrTimeFormat::Ieee1588,
            2 => SecHdrTimeFormat::Ertc,
            other => SecHdrTimeFormat::Reserved(other),
        }
    }
}

/// Parsed secondary header time value.
///
/// **Traces:** L3-SEC-003
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryHeaderTime {
    /// Chapter 4 Binary Weighted Time.
    Ch4(Ch4BinaryTime),
    /// IEEE-1588 time.
    Ieee1588(Ieee1588Time),
    /// 64-bit Extended RTC.
    Ertc(Ertc),
}

/// Validate the 16-bit secondary header checksum.
///
/// The checksum is the 16-bit sum of the first 5 little-endian u16 words
/// (bytes [0..10]), compared against the u16 at bytes [10..12].
///
/// **Traces:** L3-SEC-002 ← L2-SEC-005 ← L1-SEC-004
pub fn validate_secondary_checksum(buf: &[u8]) -> Result<()> {
    if buf.len() < 12 {
        return Err(TimeError::BufferTooShort {
            expected: 12,
            actual: buf.len(),
        });
    }
    let mut sum: u32 = 0;
    for i in 0..5 {
        let word = u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]);
        sum = sum.wrapping_add(word as u32);
    }
    let computed = (sum & 0xFFFF) as u16;
    let stored = u16::from_le_bytes([buf[10], buf[11]]);
    if computed != stored {
        Err(TimeError::ChecksumMismatch { stored, computed })
    } else {
        Ok(())
    }
}

/// Parse a 12-byte secondary header into a typed time value.
///
/// Validates the checksum first, then dispatches based on the time format.
///
/// **Traces:** L3-SEC-004 ← L2-SEC-001..L2-SEC-005
pub fn parse_secondary_header(
    buf: &[u8],
    fmt: SecHdrTimeFormat,
) -> Result<SecondaryHeaderTime> {
    if buf.len() < 12 {
        return Err(TimeError::BufferTooShort {
            expected: 12,
            actual: buf.len(),
        });
    }
    validate_secondary_checksum(buf)?;

    match fmt {
        SecHdrTimeFormat::Ch4 => {
            let ch4 = Ch4BinaryTime::from_secondary_bytes(&buf[0..8])?;
            Ok(SecondaryHeaderTime::Ch4(ch4))
        }
        SecHdrTimeFormat::Ieee1588 => {
            let t = Ieee1588Time::from_le_bytes(&buf[0..8])?;
            Ok(SecondaryHeaderTime::Ieee1588(t))
        }
        SecHdrTimeFormat::Ertc => {
            let e = Ertc::from_le_bytes(&buf[0..8])?;
            Ok(SecondaryHeaderTime::Ertc(e))
        }
        SecHdrTimeFormat::Reserved(v) => Err(TimeError::OutOfRange {
            field: "secondary_header_time_format",
            value: v as u32,
            max: 2,
        }),
    }
}

#[cfg(test)]
#[path = "secondary_tests.rs"]
mod secondary_tests;
