//! Secondary header time format parsing.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-SEC-101..105 | Secondary time parsing and separate reserved checks; checksum rebuild deferred |

use crate::absolute::{Ch4BinaryTime, Ertc, Ieee1588Time};
use crate::error::{Result, TimeError};

pub use irig106_types::SecHdrTimeFormat;

/// Parsed secondary header time value.
///
/// **Traces:** L3-SEC-101, L3-SEC-102, L3-SEC-103
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// Partial L3-SEC-104: prototype word sum only; byte-sum rebuild is deferred
/// under RDG-008. This function does not yet meet the full requirement.
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
/// Reserved words do not suppress the decoded value (TF-005). Report
/// [`validate_ch4_reserved`] and [`validate_secondary_reserved`] separately.
/// Invalid Chapter 4 microseconds still prevent decoding (TF-004).
///
/// **Traces:** L3-SEC-101, L3-SEC-102, L3-SEC-103
pub fn parse_secondary_header(buf: &[u8], fmt: SecHdrTimeFormat) -> Result<SecondaryHeaderTime> {
    if buf.len() < 12 {
        return Err(TimeError::BufferTooShort {
            expected: 12,
            actual: buf.len(),
        });
    }
    validate_secondary_checksum(buf)?;

    match fmt {
        SecHdrTimeFormat::Ch4 => {
            validate_ch4_microseconds(&buf[0..8])?;
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

// Figure 11-4 / Chapter 4 §4.7. Shared readers preserve raw bits; the time
// boundary rejects invalid usec, reporting reserved words separately as warnings.
pub(crate) fn validate_ch4_microseconds(buf: &[u8]) -> Result<()> {
    if buf.len() < 8 {
        return Err(TimeError::BufferTooShort {
            expected: 8,
            actual: buf.len(),
        });
    }
    let usec = u16::from_le_bytes([buf[2], buf[3]]);
    if usec > 9999 {
        return Err(TimeError::OutOfRange {
            field: "ch4_microseconds",
            value: usec as u32,
            max: 9999,
        });
    }
    Ok(())
}

/// Check the internal reserved word of an eight-byte Chapter 4 time.
///
/// A `ReservedBitSet` result is a separate TF-005 warning; secondary and
/// intra-packet parsers still use the decoded time. It is distinct from the
/// secondary header reserved word checked by [`validate_secondary_reserved`].
///
/// **Traces:** L3-SEC-105 ← L2-SEC-105 ← L1-SEC-005
pub fn validate_ch4_reserved(buf: &[u8]) -> Result<()> {
    if buf.len() < 8 {
        return Err(TimeError::BufferTooShort {
            expected: 8,
            actual: buf.len(),
        });
    }
    if buf[0] != 0 || buf[1] != 0 {
        return Err(TimeError::ReservedBitSet {
            position: "Chapter 4 reserved word",
        });
    }
    Ok(())
}

/// Check bytes 8–9 reserved in every twelve-byte secondary header.
///
/// A `ReservedBitSet` result is a separate TF-005 warning, not a reason to
/// discard the parsed time. Check Chapter 4's internal word independently
/// with [`validate_ch4_reserved`] to preserve both findings.
///
/// **Traces:** L3-SEC-105 ← L2-SEC-105 ← L1-SEC-005
pub fn validate_secondary_reserved(buf: &[u8]) -> Result<()> {
    if buf.len() < 12 {
        return Err(TimeError::BufferTooShort {
            expected: 12,
            actual: buf.len(),
        });
    }
    if buf[8] != 0 || buf[9] != 0 {
        return Err(TimeError::ReservedBitSet {
            position: "secondary header reserved word",
        });
    }
    Ok(())
}
