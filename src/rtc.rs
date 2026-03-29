//! 48-bit Relative Time Counter (RTC) for IRIG 106 Chapter 10.
//!
//! The RTC is a free-running 10 MHz counter providing 100 ns resolution.
//! It occupies 6 bytes (48 bits) in the primary packet header at bytes [16..22].
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-RTC-001  | Newtype `Rtc(u64)` with 48-bit invariant |
//! | L3-RTC-002  | `MASK_48` constant |
//! | L3-RTC-003  | `NANOS_PER_TICK` constant |
//! | L3-RTC-004  | `Rtc::ZERO` |
//! | L3-RTC-005  | `Rtc::MAX` |
//! | L3-RTC-006  | `from_le_bytes` |
//! | L3-RTC-007  | `from_raw` with masking |
//! | L3-RTC-008  | `as_raw` |
//! | L3-RTC-009  | `elapsed_ticks` with 48-bit wrap handling |
//! | L3-RTC-010  | `elapsed_nanos` |
//! | L3-RTC-011  | `to_nanos` |
//! | L3-RTC-012  | `Ord`/`PartialOrd` implementation |
//! | L3-RTC-013  | Standard derives |

use core::cmp::Ordering;

/// Bitmask for the lower 48 bits.
///
/// **Traces:** L3-RTC-002
const MASK_48: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Nanoseconds per RTC tick (10 MHz clock → 100 ns/tick).
///
/// **Traces:** L3-RTC-003
const NANOS_PER_TICK: u64 = 100;

/// A 48-bit Relative Time Counter value.
///
/// The RTC is the fundamental time reference in IRIG 106 Chapter 10 packet
/// headers. It runs at 10 MHz (100 ns per tick) and is represented as a 48-bit
/// unsigned integer stored little-endian in the 6-byte header field.
///
/// The inner value is guaranteed to fit within 48 bits (≤ `0x0000_FFFF_FFFF_FFFF`).
///
/// **Traces:** L3-RTC-001 ← L2-RTC-001 ← L1-RTC-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rtc(u64);

impl Rtc {
    /// RTC value of zero.
    ///
    /// **Traces:** L3-RTC-004 ← L2-RTC-010
    pub const ZERO: Rtc = Rtc(0);

    /// Maximum 48-bit RTC value (2^48 − 1).
    ///
    /// Represents approximately 325.4 days of elapsed time at 100 ns resolution.
    ///
    /// **Traces:** L3-RTC-005 ← L2-RTC-011, L2-RTC-009
    pub const MAX: Rtc = Rtc(MASK_48);

    /// Construct an RTC from 6 little-endian bytes.
    ///
    /// This corresponds to bytes [16..22] of an IRIG 106 Chapter 10 primary
    /// packet header.
    ///
    /// **Traces:** L3-RTC-006 ← L2-RTC-002 ← L1-RTC-002
    #[inline]
    pub fn from_le_bytes(bytes: [u8; 6]) -> Self {
        let raw = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
        ]);
        Rtc(raw)
    }

    /// Construct an RTC from a raw `u64`, masking to 48 bits.
    ///
    /// **Traces:** L3-RTC-007 ← L2-RTC-003 ← L1-RTC-001
    #[inline]
    pub fn from_raw(value: u64) -> Self {
        Rtc(value & MASK_48)
    }

    /// Return the raw 48-bit tick count.
    ///
    /// **Traces:** L3-RTC-008 ← L2-RTC-004 ← L1-RTC-001
    #[inline]
    pub fn as_raw(self) -> u64 {
        self.0
    }

    /// Compute elapsed ticks from `self` to `later`, handling 48-bit wrap-around.
    ///
    /// **Traces:** L3-RTC-009 ← L2-RTC-005 ← L1-RTC-003
    #[inline]
    pub fn elapsed_ticks(self, later: Rtc) -> u64 {
        later.0.wrapping_sub(self.0) & MASK_48
    }

    /// Compute elapsed nanoseconds from `self` to `later`.
    ///
    /// **Traces:** L3-RTC-010 ← L2-RTC-006 ← L1-RTC-003, L1-RTC-004
    #[inline]
    pub fn elapsed_nanos(self, later: Rtc) -> u64 {
        self.elapsed_ticks(later) * NANOS_PER_TICK
    }

    /// Convert the raw tick count to nanoseconds since the counter epoch.
    ///
    /// **Traces:** L3-RTC-011 ← L2-RTC-007 ← L1-RTC-004
    #[inline]
    pub fn to_nanos(self) -> u64 {
        self.0 * NANOS_PER_TICK
    }

    /// Encode the 48-bit RTC value as 6 little-endian bytes.
    ///
    /// This is the inverse of `from_le_bytes`.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 6] {
        let bytes = self.0.to_le_bytes();
        [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
    }
}

/// **Traces:** L3-RTC-012 ← L2-RTC-008
impl PartialOrd for Rtc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// **Traces:** L3-RTC-012 ← L2-RTC-008
impl Ord for Rtc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[cfg(test)]
#[path = "rtc_tests.rs"]
mod rtc_tests;
