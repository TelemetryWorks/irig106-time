//! Time Data Format 1 (0x11) Channel-Specific Data Word.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | L3-CSDW-001..010 | CSDW parsing and enum definitions |

/// Time source applied to the recorder.
///
/// **Traces:** L3-CSDW-008 ← L2-CSDW-003 ← L1-CSDW-002
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSource {
    /// Internal time source (0x0).
    Internal,
    /// External time source (0x1).
    External,
    /// Internal RTC (0x2).
    InternalRtc,
    /// GPS time (0x3).
    Gps,
    /// No time source (0xF).
    None,
    /// Reserved or unrecognized value.
    Reserved(u8),
}

impl TimeSource {
    fn from_raw(val: u8) -> Self {
        match val {
            0 => TimeSource::Internal,
            1 => TimeSource::External,
            2 => TimeSource::InternalRtc,
            3 => TimeSource::Gps,
            0xF => TimeSource::None,
            other => TimeSource::Reserved(other),
        }
    }

    /// Decode with version-specific mapping.
    ///
    /// In IRIG 106-04, value 3 was "None" (no time source). Starting with
    /// 106-05, value 3 was reassigned to "GPS". For pre-07 files where we
    /// cannot distinguish 04 from 05, value 3 is mapped to `Reserved(3)`
    /// to signal ambiguity.
    fn from_raw_versioned(val: u8, version: &crate::version::Irig106Version) -> Self {
        if val == 3 && !version.has_gps_time_source() {
            // Pre-07: ambiguous — could be "None" (04) or "GPS" (05)
            TimeSource::Reserved(3)
        } else {
            Self::from_raw(val)
        }
    }
}

/// Format of the external time source.
///
/// **Traces:** L3-CSDW-009 ← L2-CSDW-005 ← L1-CSDW-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    /// IRIG-B (0x0).
    IrigB,
    /// IRIG-A (0x1).
    IrigA,
    /// IRIG-G (0x2).
    IrigG,
    /// Internal RTC (0x3).
    Rtc,
    /// UTC from GPS (0x4).
    Utc,
    /// GPS time (0x5).
    Gps,
    /// Reserved or unrecognized value.
    Reserved(u8),
}

impl TimeFormat {
    fn from_raw(val: u8) -> Self {
        match val {
            0 => TimeFormat::IrigB,
            1 => TimeFormat::IrigA,
            2 => TimeFormat::IrigG,
            3 => TimeFormat::Rtc,
            4 => TimeFormat::Utc,
            5 => TimeFormat::Gps,
            other => TimeFormat::Reserved(other),
        }
    }
}

/// Date representation format in the time message.
///
/// **Traces:** L3-CSDW-010 ← L2-CSDW-007 ← L1-CSDW-005
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    /// Day-of-year format (bit 9 = 0).
    DayOfYear,
    /// Day-month-year format (bit 9 = 1).
    DayMonthYear,
}

/// Parsed Time Data Format 1 (0x11) Channel-Specific Data Word.
///
/// **Traces:** L3-CSDW-001 ← L2-CSDW-001, L2-CSDW-008 ← L1-CSDW-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeF1Csdw(u32);

impl TimeF1Csdw {
    /// Construct from a raw 32-bit value.
    ///
    /// **Traces:** L3-CSDW-002
    #[inline]
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Construct from 4 little-endian bytes.
    ///
    /// **Traces:** L3-CSDW-003
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

    /// Time source (bits \[3:0\]).
    ///
    /// **Traces:** L3-CSDW-004 ← L2-CSDW-002
    #[inline]
    pub fn time_source(self) -> TimeSource {
        TimeSource::from_raw((self.0 & 0x0F) as u8)
    }

    /// Time format (bits \[7:4\]).
    ///
    /// **Traces:** L3-CSDW-005 ← L2-CSDW-004
    #[inline]
    pub fn time_format(self) -> TimeFormat {
        TimeFormat::from_raw(((self.0 >> 4) & 0x0F) as u8)
    }

    /// Leap year indicator (bit 8).
    ///
    /// **Traces:** L3-CSDW-006 ← L2-CSDW-006
    #[inline]
    pub fn is_leap_year(self) -> bool {
        (self.0 >> 8) & 1 == 1
    }

    /// Date format (bit 9).
    ///
    /// **Traces:** L3-CSDW-007 ← L2-CSDW-007
    #[inline]
    pub fn date_format(self) -> DateFormat {
        if (self.0 >> 9) & 1 == 0 {
            DateFormat::DayOfYear
        } else {
            DateFormat::DayMonthYear
        }
    }

    /// Version-aware time source (bits \[3:0\]).
    ///
    /// Uses the IRIG 106 version to disambiguate time source value 3,
    /// which was "None" in 106-04 but "GPS" from 106-05 onward.
    /// For pre-07 files, value 3 is returned as `Reserved(3)` since the
    /// exact standard version is unknown.
    ///
    /// **Traces:** P2-01, P2-04
    #[inline]
    pub fn time_source_versioned(self, version: &crate::version::Irig106Version) -> TimeSource {
        TimeSource::from_raw_versioned((self.0 & 0x0F) as u8, version)
    }
}

#[cfg(test)]
#[path = "csdw_tests.rs"]
mod csdw_tests;
