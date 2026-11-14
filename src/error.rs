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
    /// Raw counter construction would discard high bits.
    /// **Traces:** L3-RTC-101 ← L2-RTC-101 ← L1-RTC-001
    CounterWidth {
        /// Unmasked caller input.
        value: u64,
    },
    /// Two raw counters are exactly half a wrap apart; ordering is required.
    /// **Traces:** L3-RTC-103 ← L2-RTC-103 ← L1-RTC-003
    AmbiguousCounterDelta,
    /// Counter subtraction crossed sessions or used an unnamed (zero) session.
    CrossSessionCounterDelta,
    /// A checked quantity conversion or subtraction cannot fit its destination.
    /// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
    QuantityOverflow {
        /// Units and operation, with no truncated result returned.
        quantity: &'static str,
    },
    /// A BCD nibble contained a value greater than 9.
    /// **Traces:** L3-ERR-102 ← L2-ERR-102 ← L1-ERR-002
    InvalidBcdDigit {
        /// The invalid nibble value (10–15).
        nibble: u8,
        /// Human-readable description of where the nibble was found.
        position: &'static str,
        /// Zero-based body word index (Figures 11-13/14).
        word: u8,
        /// Decimal digit index: units=0, tens=1, hundreds=2, thousands=3.
        digit: u8,
        /// Least significant bit of the digit within that word.
        bit_offset: u8,
        /// Entire original little-endian word, including any reserved bits.
        raw_word: u16,
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

    /// Packet flags cannot select a valid intra-packet time (§11.2.1.1 g).
    InvalidPacketFlags {
        /// Original flag byte, retained for diagnostics.
        flags: u8,
    },

    /// Reserved packet secondary time format 11 (TF-006 warning).
    ///
    /// Separate validation finding: an RTC stamp value remains usable.
    /// **Traces:** L3-IPT-107 ← L2-IPT-107 ← L1-IPT-007
    ReservedPacketTimeFormat {
        /// Original flags, including the reserved format value.
        flags: u8,
    },

    /// A secondary-derived time has no defined wire format.
    ///
    /// No time is returned for the affected secondary time; an independent
    /// RTC stamp remains usable. Maps to TF-028 (ERROR), accepted by owner
    /// 2026-11-13; TF-006 remains the separate reserved-value warning.
    /// **Traces:** L3-IPT-107 ← L2-IPT-107 ← L1-IPT-007
    UndefinedSecondaryTimeFormat {
        /// Original flags, retained beside any other flag findings.
        flags: u8,
    },

    /// Caller supplied an invalid offline timeline identity or policy.
    /// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
    InvalidTimelineInput(&'static str),

    /// Conversion cannot proceed without explicit, reviewed context.
    NeedsContext {
        /// Missing or unresolved conversion context.
        context: &'static str,
    },

    /// Format 2 status does not mark time valid.
    InvalidTimeStatus {
        /// Raw TS nibble.
        status: u8,
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

    /// An `irig106-types` error with no more specific `TimeError` equivalent.
    ///
    /// `BufferTooShort` and `OutOfRange` from `irig106-types` are mapped to
    /// the matching `TimeError` variants; this carries anything else.
    Types(irig106_types::TypesError),
}

/// Lift errors from `irig106-types` constructors (e.g.,
/// [`Ieee1588Time::from_le_bytes`](irig106_types::Ieee1588Time::from_le_bytes))
/// into [`TimeError`] so they compose with `?`.
impl From<irig106_types::TypesError> for TimeError {
    fn from(e: irig106_types::TypesError) -> Self {
        match e {
            irig106_types::TypesError::BufferTooShort { expected, actual } => {
                TimeError::BufferTooShort { expected, actual }
            }
            irig106_types::TypesError::OutOfRange { field, value, max } => {
                TimeError::OutOfRange { field, value, max }
            }
            other => TimeError::Types(other),
        }
    }
}

/// **Traces:** L3-ERR-002 ← L2-ERR-003 ← L1-ERR-001
impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::CounterWidth { value } => write!(f, "RTC value {value} exceeds 48 bits"),
            TimeError::AmbiguousCounterDelta => {
                write!(f, "half-range RTC difference requires ordering context")
            }
            TimeError::CrossSessionCounterDelta => {
                write!(f, "RTC difference requires the same nonzero session")
            }
            TimeError::QuantityOverflow { quantity } => {
                write!(f, "quantity out of range: {quantity}")
            }
            TimeError::InvalidBcdDigit {
                nibble,
                position,
                word,
                digit,
                raw_word,
                ..
            } => {
                write!(
                    f,
                    "invalid BCD digit {nibble} at {position} (word {word}, digit {digit}, raw {raw_word:#06x})"
                )
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
            TimeError::InvalidPacketFlags { flags } => {
                write!(f, "invalid intra-packet time flags {flags:#04x}")
            }
            TimeError::ReservedPacketTimeFormat { flags } => {
                write!(
                    f,
                    "reserved packet secondary time format (TF-006 warning): flags {flags:#04x}"
                )
            }
            TimeError::UndefinedSecondaryTimeFormat { flags } => {
                write!(
                    f,
                    "undefined packet secondary time format: flags {flags:#04x}"
                )
            }
            TimeError::InvalidTimelineInput(context) => {
                write!(f, "invalid timeline input: {context}")
            }
            TimeError::NeedsContext { context } => write!(f, "time needs context: {context}"),
            TimeError::InvalidTimeStatus { status } => {
                write!(f, "network time status is not valid: {status:#x}")
            }
            TimeError::NoReferencePoint => {
                write!(f, "no time reference point available for correlation")
            }
            TimeError::BufferTooShort { expected, actual } => {
                write!(f, "buffer too short: need {expected} bytes, got {actual}")
            }
            TimeError::Types(e) => fmt::Display::fmt(e, f),
        }
    }
}

/// **Traces:** L3-ERR-003 ← L2-ERR-003 ← L1-ERR-001
#[cfg(feature = "std")]
impl std::error::Error for TimeError {}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;
