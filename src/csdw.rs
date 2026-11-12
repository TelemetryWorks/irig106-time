//! Time Data Format 1 (0x11) Channel-Specific Data Word.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-CSDW-101..107 | CSDW parsing and enum definitions |

use crate::error::{Result, TimeError};
pub use irig106_types::{DateFormat, IrigTimeSource, TimeFormat, TimeSource};

/// Caller-declared Format 1 layout, never inferred from header version hints.
///
/// 106-04 Figure 10-15; 106-05 Figure 10-16a; 106-24R1 Figure 11-12.
/// Automatic edition resolution and TMATS comparisons are deferred (Q-EDITION).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeF1Layout {
    /// 106-04: EXT bit 0, bits 3–1 reserved; FMT 6–15 reserved.
    Legacy04,
    /// 106-05 through 106-15: SRC, no ITS; FMT 15 means None.
    Source05,
    /// Explicit 106-17+ layout: SRC plus applicable ITS bits 15–12.
    SourceWithIts17,
}

/// Edition-qualified source evidence; EXT is not a modern SRC enum.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeF1Source {
    /// 106-04 EXT, retaining nonzero reserved bits for diagnostics.
    ExternalTime {
        /// Whether an external time source is present (bit 0).
        present: bool,
        /// Raw reserved bits 3–1, without shifting.
        reserved_bits: u8,
    },
    /// 106-05+ SRC, including reserved numeric values.
    Source(TimeSource),
}

/// Parsed Time Data Format 1 (0x11) Channel-Specific Data Word.
///
/// **Traces:** L3-CSDW-101 ← L2-CSDW-101 ← L1-CSDW-001
/// **Traces:** L3-CSDW-102 ← L2-CSDW-102 ← L1-CSDW-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeF1Csdw(u32);

impl TimeF1Csdw {
    /// Construct from a raw 32-bit value.
    ///
    /// **Traces:** L3-CSDW-101
    #[inline]
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Construct from 4 little-endian bytes.
    ///
    /// **Traces:** L3-CSDW-101
    #[inline]
    pub fn from_le_bytes(buf: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(buf))
    }

    /// Raw 32-bit value.
    #[inline]
    pub fn as_raw(self) -> u32 {
        self.0
    }

    /// Encode as 4 little-endian bytes.
    ///
    /// This is the inverse of `from_le_bytes`.
    ///
    /// **Traces:** GAP-11
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Modern (106-05+) SRC, bits \[3:0\] (§11.2.3.2 a).
    /// For legacy packets use time_source_for_layout with an explicit declaration.
    ///
    /// **Traces:** L3-CSDW-102 ← L2-CSDW-102
    #[inline]
    pub fn time_source(self) -> TimeSource {
        TimeSource::from_raw((self.0 & 0x0F) as u8)
    }

    /// Modern (106-05+) FMT, bits \[7:4\] (§11.2.3.2 a).
    /// For legacy packets use time_format_for_layout.
    ///
    /// **Traces:** L3-CSDW-103 ← L2-CSDW-103
    #[inline]
    pub fn time_format(self) -> TimeFormat {
        TimeFormat::from_raw(((self.0 >> 4) & 0x0F) as u8)
    }

    /// Leap year indicator (bit 8).
    ///
    /// **Traces:** L3-CSDW-104 ← L2-CSDW-104
    #[inline]
    pub fn is_leap_year(self) -> bool {
        (self.0 >> 8) & 1 == 1
    }

    /// Date format (bit 9).
    ///
    /// **Traces:** L3-CSDW-105 ← L2-CSDW-105
    #[inline]
    pub fn date_format(self) -> DateFormat {
        if (self.0 >> 9) & 1 == 0 {
            DateFormat::DayOfYear
        } else {
            DateFormat::DayMonthYear
        }
    }

    /// Source for an explicitly supplied layout (106-04 §10.6.3.3;
    /// 106-24R1 §11.2.3.2 a). No edition is guessed from the raw bits.
    ///
    /// **Traces:** L3-CSDW-102
    pub fn time_source_for_layout(self, layout: TimeF1Layout) -> TimeF1Source {
        match layout {
            TimeF1Layout::Legacy04 => TimeF1Source::ExternalTime {
                present: self.0 & 1 != 0,
                reserved_bits: (self.0 & 0x0e) as u8,
            },
            TimeF1Layout::Source05 | TimeF1Layout::SourceWithIts17 => {
                TimeF1Source::Source(self.time_source())
            }
        }
    }

    /// FMT qualified by caller layout: 106-04 reserves 6–15; from 106-05
    /// FMT 15 is None (invalid payload). See the cited figures on TimeF1Layout.
    ///
    /// **Traces:** L3-CSDW-103
    pub fn time_format_for_layout(self, layout: TimeF1Layout) -> TimeFormat {
        let raw = ((self.0 >> 4) & 15) as u8;
        if layout == TimeF1Layout::Legacy04 && raw >= 6 {
            TimeFormat::Reserved(raw)
        } else {
            TimeFormat::from_raw(raw)
        }
    }

    /// Whether FMT defines a valid payload format (§11.2.3.2 a).
    /// This does not validate a body or admit a correlation reference.
    ///
    /// **Traces:** L3-CSDW-103
    pub fn payload_is_valid(self, layout: TimeF1Layout) -> bool {
        !matches!(
            self.time_format_for_layout(layout),
            TimeFormat::None | TimeFormat::Reserved(_)
        )
    }

    /// ITS only for a caller-declared 106-17+ layout and IRIG-A/B/G FMT.
    /// Raw reserved ITS values remain Reserved; absent/inapplicable means None.
    /// This exposes explicit layout facts, not an automatic RDG-007 resolver.
    ///
    /// **Traces:** L3-CSDW-106
    pub fn irig_time_source_for_layout(self, layout: TimeF1Layout) -> Option<IrigTimeSource> {
        if layout == TimeF1Layout::SourceWithIts17
            && matches!(
                self.time_format(),
                TimeFormat::IrigA | TimeFormat::IrigB | TimeFormat::IrigG
            )
        {
            Some(IrigTimeSource::from_raw(((self.0 >> 12) & 15) as u8))
        } else {
            None
        }
    }

    /// Check reserved CSDW bits for the explicitly declared layout.
    /// 106-04 reserves 31–10 and 3–1; 106-05–15 reserves 31–10;
    /// 106-17+ reserves 31–16 and 11–10 (Figure 11-12).
    /// Reserved field values remain visible through their typed accessors.
    ///
    /// **Traces:** L3-CSDW-107
    pub fn validate_reserved(self, layout: TimeF1Layout) -> Result<()> {
        let mask = match layout {
            TimeF1Layout::Legacy04 => 0xffff_fc0e,
            TimeF1Layout::Source05 => 0xffff_fc00,
            TimeF1Layout::SourceWithIts17 => 0xffff_0c00,
        };
        if self.0 & mask != 0 {
            Err(TimeError::ReservedBitSet {
                position: "Format 1 CSDW",
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "csdw_tests.rs"]
mod csdw_tests;
