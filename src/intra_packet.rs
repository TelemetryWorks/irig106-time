//! Intra-packet time stamp parsing for IRIG 106 Chapter 10 data messages.
//!
//! Each data message within a packet may carry an 8-byte intra-packet time
//! stamp whose format is determined by the Packet Flag bits.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-IPT-106  | `IntraPacketTime` enum |
//! | L3-IPT-106  | `parse_intra_packet_time` dispatcher |
//! | L3-IPT-105  | `IntraPacketTimeFormat` enum |

use crate::absolute::{Ch4BinaryTime, Ertc, Ieee1588Time};
use crate::error::{Result, TimeError};
use crate::rtc::Rtc;

/// Discriminant for intra-packet time stamp format.
///
/// **Traces:** L3-IPT-105
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntraPacketTimeFormat {
    /// 48-bit RTC (6 bytes used, 2 bytes reserved).
    Rtc48,
    /// IRIG 106 Chapter 4 Binary Weighted Time.
    Ch4Binary,
    /// IEEE-1588 Precision Time Protocol.
    Ieee1588,
    /// 64-bit Extended RTC.
    Ertc64,
    /// Invalid flag combination, preserving the byte for diagnostics.
    InvalidFlags(u8),
}

impl IntraPacketTimeFormat {
    /// Decode §11.2.1.1 g: bit 7 is secondary presence, bit 6 selects
    /// the stamp source, and bits 3–2 select the secondary format.
    /// L1-IPT-007 strictness is retained: reserved format 11 is invalid
    /// even for RTC-derived stamps, pending owner question Q-FLAGS.
    pub fn from_packet_flags(flags: u8) -> Self {
        let format = (flags >> 2) & 3;
        if format == 3 || flags & 0x40 != 0 && flags & 0x80 == 0 {
            return Self::InvalidFlags(flags);
        }
        if flags & 0x40 == 0 {
            return Self::Rtc48;
        }
        match format {
            0 => Self::Ch4Binary,
            1 => Self::Ieee1588,
            2 => Self::Ertc64,
            _ => Self::InvalidFlags(flags),
        }
    }
}

/// Parsed intra-packet time stamp.
///
/// **Traces:** L3-IPT-106 ← L2-IPT-106
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// **Traces:** L3-IPT-106 ← L2-IPT-106, L2-IPT-107
pub fn parse_intra_packet_time(buf: &[u8], fmt: IntraPacketTimeFormat) -> Result<IntraPacketTime> {
    if buf.len() < 8 {
        return Err(TimeError::BufferTooShort {
            expected: 8,
            actual: buf.len(),
        });
    }

    match fmt {
        IntraPacketTimeFormat::InvalidFlags(flags) => Err(TimeError::InvalidPacketFlags { flags }),
        IntraPacketTimeFormat::Rtc48 => {
            // §11.2.1.3 b: 48-bit RTC plus 16 high-order zero bits.
            if buf[6] != 0 || buf[7] != 0 {
                return Err(TimeError::ReservedBitSet {
                    position: "RTC stamp padding",
                });
            }
            let rtc = Rtc::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]]);
            Ok(IntraPacketTime::Rtc(rtc))
        }
        IntraPacketTimeFormat::Ch4Binary => {
            crate::secondary::validate_ch4_time(buf)?;
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
