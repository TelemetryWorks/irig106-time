//! Intra-packet time stamp parsing for IRIG 106 Chapter 10 data messages.
//!
//! Each data message within a packet may carry an 8-byte intra-packet time
//! stamp whose format is determined by the Packet Flag bits.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-IPT-001  | `IntraPacketTime` enum |
//! | L3-IPT-002  | `parse_intra_packet_time` dispatcher |
//! | L3-IPT-003  | `IntraPacketTimeFormat` enum |

use crate::absolute::{Ch4BinaryTime, Ertc, Ieee1588Time};
use crate::error::{Result, TimeError};
use crate::rtc::Rtc;

/// Discriminant for intra-packet time stamp format.
///
/// **Traces:** L3-IPT-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntraPacketTimeFormat {
    /// 48-bit RTC (6 bytes used, 2 bytes reserved).
    Rtc48,
    /// IRIG 106 Chapter 4 Binary Weighted Time.
    Ch4Binary,
    /// IEEE-1588 Precision Time Protocol.
    Ieee1588,
    /// 64-bit Extended RTC.
    Ertc64,
}

impl IntraPacketTimeFormat {
    /// Derive the intra-packet time format from Packet Flag bits.
    ///
    /// - If bit 2 (secondary header present) is clear, the format is always RTC.
    /// - If bit 2 is set, bits \[3:2\] select the format:
    ///   0 = Ch4, 1 = IEEE-1588, 2 = ERTC.
    pub fn from_packet_flags(flags: u8) -> Self {
        let has_secondary = (flags & 0x04) != 0;
        if !has_secondary {
            return IntraPacketTimeFormat::Rtc48;
        }
        match (flags >> 2) & 0x03 {
            0 => IntraPacketTimeFormat::Ch4Binary,
            1 => IntraPacketTimeFormat::Ieee1588,
            2 => IntraPacketTimeFormat::Ertc64,
            _ => IntraPacketTimeFormat::Rtc48, // fallback
        }
    }
}

/// Parsed intra-packet time stamp.
///
/// **Traces:** L3-IPT-001 ← L2-IPT-005
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntraPacketTime {
    /// 48-bit Relative Time Counter value.
    Rtc(Rtc),
    /// Chapter 4 Binary Weighted Time.
    Ch4(Ch4BinaryTime),
    /// IEEE-1588 time.
    Ieee1588(Ieee1588Time),
    /// 64-bit Extended RTC.
    Ertc(Ertc),
}

/// Parse an 8-byte intra-packet time stamp buffer.
///
/// **Traces:** L3-IPT-002 ← L2-IPT-001..L2-IPT-004
pub fn parse_intra_packet_time(buf: &[u8], fmt: IntraPacketTimeFormat) -> Result<IntraPacketTime> {
    if buf.len() < 8 {
        return Err(TimeError::BufferTooShort {
            expected: 8,
            actual: buf.len(),
        });
    }

    match fmt {
        IntraPacketTimeFormat::Rtc48 => {
            // Bytes [0..6] = 48-bit RTC, bytes [6..8] = reserved (unused)
            let rtc = Rtc::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]]);
            Ok(IntraPacketTime::Rtc(rtc))
        }
        IntraPacketTimeFormat::Ch4Binary => {
            let ch4 = Ch4BinaryTime::from_intra_packet_bytes(buf)?;
            Ok(IntraPacketTime::Ch4(ch4))
        }
        IntraPacketTimeFormat::Ieee1588 => {
            let t = Ieee1588Time::from_le_bytes(buf)?;
            Ok(IntraPacketTime::Ieee1588(t))
        }
        IntraPacketTimeFormat::Ertc64 => {
            let e = Ertc::from_le_bytes(buf)?;
            Ok(IntraPacketTime::Ertc(e))
        }
    }
}

#[cfg(test)]
#[path = "intra_packet_tests.rs"]
mod intra_packet_tests;
