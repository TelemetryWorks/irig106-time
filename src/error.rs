//! Error types for the `irig106-time` crate.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-ERR-001  | `TimeError` enum definition |
//! | L3-ERR-002  | `Display` implementation |
//! | L3-ERR-003  | `std::error::Error` (feature-gated) |
//! | L3-ERR-004  | `Result<T>` type alias |

use core::fmt;

/// Crate-wide result alias.
///
/// **Traces:** L3-ERR-004 ← L2-ERR-002 ← L1-ERR-001
pub type Result<T> = core::result::Result<T, TimeError>;

/// Errors produced by time parsing and correlation operations.
///
/// **Traces:** L3-ERR-001 ← L2-ERR-001 ← L1-ERR-001..L1-ERR-004
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimeError {
    /// A BCD nibble contained a value greater than 9.
    InvalidBcdDigit {
        /// The invalid nibble value (10–15).
        nibble: u8,
        /// Human-readable description of where the nibble was found.
        position: &'static str,
    },

    /// A reserved bit field was non-zero.
    ReservedBitSet {
        /// Human-readable description of the reserved field.
        position: &'static str,
    },

    /// A decoded field value exceeded its valid range.
    OutOfRange {
        /// Name of the field (e.g., "hours", "minutes").
        field: &'static str,
        /// The actual decoded value.
        value: u32,
        /// The maximum allowed value (inclusive).
        max: u32,
    },

    /// A secondary header checksum did not match.
    ChecksumMismatch {
        /// Checksum stored in the header.
        stored: u16,
        /// Checksum computed from the header bytes.
        computed: u16,
    },

    /// No reference point available for RTC-to-absolute-time correlation.
    NoReferencePoint,

    /// Input buffer was shorter than required.
    BufferTooShort {
        /// Number of bytes required.
        expected: usize,
        /// Number of bytes actually available.
        actual: usize,
    },
}

/// **Traces:** L3-ERR-002 ← L2-ERR-003 ← L1-ERR-001
impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::InvalidBcdDigit { nibble, position } => {
                write!(f, "invalid BCD digit {nibble} at {position}")
            }
            TimeError::ReservedBitSet { position } => {
                write!(f, "reserved bit set at {position}")
            }
            TimeError::OutOfRange { field, value, max } => {
                write!(f, "{field} value {value} exceeds maximum {max}")
            }
            TimeError::ChecksumMismatch { stored, computed } => {
                write!(
                    f,
                    "secondary header checksum mismatch: stored=0x{stored:04X}, computed=0x{computed:04X}"
                )
            }
            TimeError::NoReferencePoint => {
                write!(f, "no time reference point available for correlation")
            }
            TimeError::BufferTooShort { expected, actual } => {
                write!(f, "buffer too short: need {expected} bytes, got {actual}")
            }
        }
    }
}

/// **Traces:** L3-ERR-003 ← L2-ERR-003 ← L1-ERR-001
#[cfg(feature = "std")]
impl std::error::Error for TimeError {}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
