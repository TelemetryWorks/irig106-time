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
/// Use qualify with resolved caller evidence; TMATS comparisons remain separate.
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
    /// No packet layout established; retain raw nibble without a SRC/EXT label.
    Unresolved(u8),
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
    /// For not-reported/unresolved provenance use QualifiedCsdw::its.
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

/// Applicability is separate from a numeric ITS value (RDG-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItsQualification {
    /// Applicable 106-17+ A/B/G, including Reserved(raw).
    Reported(IrigTimeSource),
    /// Pre-106-17 field was reserved; zero is not freewheeling.
    NotReported,
    /// Resolved non-A/B/G format has no applicable ITS meaning.
    NotApplicable,
    /// Unknown edition and zero cannot distinguish old reserved from freewheel.
    FreewheelingOrNotReported,
    /// Unknown applicability; raw ITS nibble retained.
    Unresolved(u8),
}
/// Raw CSDW plus immutable edition basis; unknown layout keeps alternatives.
/// **Traces:** L3-CSDW-101 ← L2-CSDW-101 ← L1-CSDW-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedCsdw {
    raw: TimeF1Csdw,
    resolution: crate::edition::EditionResolution,
}
impl TimeF1Csdw {
    /// Qualify fields only after resolving layout evidence.
    /// **Traces:** L3-CSDW-101 ← L2-CSDW-101 ← L1-CSDW-001
    pub fn qualify(self, resolution: crate::edition::EditionResolution) -> QualifiedCsdw {
        QualifiedCsdw {
            raw: self,
            resolution,
        }
    }
}
impl QualifiedCsdw {
    /// Original word, never repaired.
    pub fn raw(&self) -> TimeF1Csdw {
        self.raw
    }
    /// Original edition evidence and policy basis.
    pub fn resolution(&self) -> &crate::edition::EditionResolution {
        &self.resolution
    }
    /// Selected layout only, never a best guess.
    pub fn layout(&self) -> Option<TimeF1Layout> {
        self.resolution.edition().map(|e| e.rules().layout)
    }
    /// Unknown layout assigns neither SRC nor EXT; nibble remains in raw.
    /// **Traces:** L3-CSDW-102 ← L2-CSDW-102 ← L1-CSDW-002
    pub fn source(&self) -> Option<TimeF1Source> {
        self.layout().map(|l| self.raw.time_source_for_layout(l))
    }
    /// Candidate EXT and SRC readings, never an adopted source by themselves.
    pub fn source_alternatives(&self) -> [TimeF1Source; 2] {
        [
            self.raw.time_source_for_layout(TimeF1Layout::Legacy04),
            self.raw.time_source_for_layout(TimeF1Layout::Source05),
        ]
    }
    /// Candidate 106-04 and 106-05+ FMT readings; 15 retains both meanings.
    pub fn format_alternatives(&self) -> [TimeFormat; 2] {
        [
            self.raw.time_format_for_layout(TimeF1Layout::Legacy04),
            self.raw.time_format_for_layout(TimeF1Layout::Source05),
        ]
    }
    /// FMT0..14 have common meaning, except 15 which is reserved in04/None05+.
    /// **Traces:** L3-CSDW-103 ← L2-CSDW-103 ← L1-CSDW-003
    pub fn format(&self) -> Option<TimeFormat> {
        self.layout()
            .map(|l| self.raw.time_format_for_layout(l))
            .or_else(|| {
                let raw = ((self.raw.as_raw() >> 4) & 15) as u8;
                (raw != 15).then(|| TimeFormat::from_raw(raw))
            })
    }
    /// Applicable current ITS, not-reported legacy ITS, or explicit ambiguity.
    /// **Traces:** L3-CSDW-106 ← L2-CSDW-106 ← L1-CSDW-006
    /// **Traces:** L3-RDG-207 ← L2-RDG-207 ← L1-RDG-002
    pub fn its(&self) -> ItsQualification {
        let raw = ((self.raw.as_raw() >> 12) & 15) as u8;
        match self.layout() {
            Some(TimeF1Layout::Legacy04 | TimeF1Layout::Source05) => ItsQualification::NotReported,
            Some(TimeF1Layout::SourceWithIts17) => self
                .raw
                .irig_time_source_for_layout(TimeF1Layout::SourceWithIts17)
                .map_or(ItsQualification::NotApplicable, ItsQualification::Reported),
            None if ((self.raw.as_raw() >> 4) & 15) > 2 => ItsQualification::NotApplicable,
            None if raw == 0 => ItsQualification::FreewheelingOrNotReported,
            None => ItsQualification::Unresolved(raw),
        }
    }
    /// Raw intersections ordered as Legacy04, Source05, SourceWithIts17.
    pub fn reserved_alternatives(&self) -> [u32; 3] {
        let raw = self.raw.as_raw();
        [raw & 0xfffffc0e, raw & 0xfffffc00, raw & 0xffff0c00]
    }
    /// Selected intersection or only bits reserved under EVERY unresolved layout.
    /// Alternative-only intersections stay labelled, not automatic TF-005.
    /// **Traces:** L3-CSDW-107 ← L2-CSDW-107 ← L1-CSDW-007
    pub fn reserved_bits(&self) -> u32 {
        let a = self.reserved_alternatives();
        match self.layout() {
            Some(TimeF1Layout::Legacy04) => a[0],
            Some(TimeF1Layout::Source05) => a[1],
            _ => a[2],
        }
    }
}
