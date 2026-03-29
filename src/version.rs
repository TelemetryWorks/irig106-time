//! IRIG 106 standard version detection.
//!
//! The version is encoded in the TMATS CSDW's "IRIG 106 Chapter 10 Version"
//! field (bits \[7:0\]), defined from 106-07 onward. Files recorded before
//! 106-07 have this field set to zero.
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | P2-03       | Version enum and detection |

/// Known IRIG 106 standard versions with time-relevant changes.
///
/// Key inflection points for time handling:
/// - **V04** (2004): Initial Ch10. No version field. No packet ordering constraint.
/// - **V05** (2005): 100 ms buffer + 1 sec write deadline. Secondary header IEEE-1588.
/// - **V07** (2007): Version field added to TMATS CSDW. ERTC secondary header format.
/// - **V09** (2009): Clarifications only — no time format changes.
/// - **V17** (2017): Ch10/Ch11 split. Time fields identical across both chapters.
/// - **V22** (2022): Time Data Format 2 (0x12) — NTP/PTP network time.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Irig106Version {
    /// Pre-07 file (version field = 0). Assume 106-04/05 behavior.
    Pre07,
    /// IRIG 106-07 (version field = 7).
    V07,
    /// IRIG 106-09 (version field = 8).
    V09,
    /// IRIG 106-11 (version field = 9).
    V11,
    /// IRIG 106-13 (version field = 10 / 0x0A).
    V13,
    /// IRIG 106-15 (version field = 11 / 0x0B).
    V15,
    /// IRIG 106-17 (version field = 12 / 0x0C). Ch10/Ch11 split.
    V17,
    /// IRIG 106-19 (version field = 13 / 0x0D).
    V19,
    /// IRIG 106-22 (version field = 14 / 0x0E). Network Time (0x12) added.
    V22,
    /// IRIG 106-23 (version field = 15 / 0x0F).
    V23,
    /// Unrecognized version field value.
    Unknown(u8),
}

impl Irig106Version {
    /// Decode the IRIG 106 version from the TMATS CSDW version field (bits \[7:0\]).
    ///
    /// A value of 0 indicates a pre-07 file (106-04 or 106-05).
    #[inline]
    pub fn from_version_field(val: u8) -> Self {
        match val {
            0 => Irig106Version::Pre07,
            7 => Irig106Version::V07,
            8 => Irig106Version::V09,
            9 => Irig106Version::V11,
            0x0A => Irig106Version::V13,
            0x0B => Irig106Version::V15,
            0x0C => Irig106Version::V17,
            0x0D => Irig106Version::V19,
            0x0E => Irig106Version::V22,
            0x0F => Irig106Version::V23,
            other => Irig106Version::Unknown(other),
        }
    }

    /// Returns `true` if this version predates 106-05's ordering guarantees.
    ///
    /// Pre-05 files have no packet ordering constraint, meaning packets may
    /// arrive 5+ seconds out of order. Post-05 files are guaranteed within
    /// ~1.1 seconds (100 ms buffer + 1 sec write deadline).
    #[inline]
    pub fn is_pre_ordering_guarantee(&self) -> bool {
        matches!(self, Irig106Version::Pre07)
    }

    /// Returns `true` if this version supports Time Data Format 2 (0x12).
    ///
    /// Network Time (NTP/PTP) was introduced in IRIG 106-22.
    #[inline]
    pub fn supports_format_2(&self) -> bool {
        matches!(
            self,
            Irig106Version::V22 | Irig106Version::V23 | Irig106Version::Unknown(_)
        )
    }

    /// Returns `true` if the CSDW time source field (bits \[3:0\]) uses the
    /// 106-05+ mapping where value 3 = GPS.
    ///
    /// In 106-04, value 3 was mapped to "None" (no time source).
    /// Starting with 106-05, value 3 was reassigned to "GPS".
    /// Since pre-07 files report version=0 and we cannot distinguish 04 from 05,
    /// this returns `false` for `Pre07` — callers should treat value 3 as
    /// ambiguous for pre-07 files.
    #[inline]
    pub fn has_gps_time_source(&self) -> bool {
        !matches!(self, Irig106Version::Pre07)
    }
}

/// Detect the IRIG 106 version from a TMATS CSDW (Data Type 0x01).
///
/// Extracts bits \[7:0\] from the 32-bit CSDW and returns the decoded version.
///
/// # Example
///
/// ```
/// use irig106_time::version::{detect_version, Irig106Version};
///
/// // A TMATS CSDW with version field = 0x0E (106-22)
/// let tmats_csdw: u32 = 0x0000_000E;
/// assert_eq!(detect_version(tmats_csdw), Irig106Version::V22);
///
/// // Pre-07 file with version = 0
/// assert_eq!(detect_version(0x0000_0000), Irig106Version::Pre07);
/// ```
pub fn detect_version(tmats_csdw: u32) -> Irig106Version {
    Irig106Version::from_version_field((tmats_csdw & 0xFF) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_version_pre07() {
        assert_eq!(detect_version(0x0000_0000), Irig106Version::Pre07);
    }

    #[test]
    fn detect_version_v07() {
        assert_eq!(detect_version(0x0000_0007), Irig106Version::V07);
    }

    #[test]
    fn detect_version_v22() {
        assert_eq!(detect_version(0x0000_000E), Irig106Version::V22);
    }

    #[test]
    fn detect_version_v23() {
        assert_eq!(detect_version(0x0000_000F), Irig106Version::V23);
    }

    #[test]
    fn detect_version_unknown() {
        assert_eq!(detect_version(0x0000_00FF), Irig106Version::Unknown(0xFF));
    }

    #[test]
    fn detect_version_ignores_upper_bits() {
        // Upper bits are TMATS config fields, not version
        assert_eq!(detect_version(0xFFFF_FF0E), Irig106Version::V22);
    }

    #[test]
    fn pre07_has_no_ordering_guarantee() {
        assert!(Irig106Version::Pre07.is_pre_ordering_guarantee());
        assert!(!Irig106Version::V07.is_pre_ordering_guarantee());
    }

    #[test]
    fn format_2_support() {
        assert!(!Irig106Version::Pre07.supports_format_2());
        assert!(!Irig106Version::V17.supports_format_2());
        assert!(Irig106Version::V22.supports_format_2());
        assert!(Irig106Version::V23.supports_format_2());
    }

    #[test]
    fn gps_time_source_mapping() {
        // Pre07 cannot distinguish 04 from 05, so GPS mapping is ambiguous
        assert!(!Irig106Version::Pre07.has_gps_time_source());
        assert!(Irig106Version::V07.has_gps_time_source());
        assert!(Irig106Version::V22.has_gps_time_source());
    }

    #[test]
    fn version_ordering() {
        assert!(Irig106Version::Pre07 < Irig106Version::V07);
        assert!(Irig106Version::V07 < Irig106Version::V22);
        assert!(Irig106Version::V22 < Irig106Version::V23);
    }
}
