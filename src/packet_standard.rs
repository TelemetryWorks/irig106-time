//! IRIG 106 packet standard provenance.
//!
//! IRIG 106-17 split Chapter 10 into two chapters: Chapter 10 retains recorder
//! operations, while Chapter 11 defines the recorder packet formats. Individual
//! field layouts still depend on the caller-declared edition; this module
//! provides metadata to track which chapter a packet originated from.
//!
//! # Why This Matters
//!
//! For time handling, Ch10 and Ch11 packets are byte-identical. The distinction
//! matters for:
//! - Compliance reporting (which standard revision governs the file)
//! - TMATS interpretation (Ch11 TMATS has additional attributes)
//! - Tooling that needs to label provenance in exports
//!
//! # Requirement Traceability
//!
//! | Requirement | Description |
//! |-------------|-------------|
//! | P5-01       | Ch11 packet format awareness |

/// Identifies whether a packet follows the Chapter 10 or Chapter 11 format.
///
/// In IRIG 106-17 and later, packet format definitions moved to Chapter 11.
/// This enum tracks the chapter named by a caller-declared release, not
/// the per-field wire layout or an automatically inferred recording edition.
///
/// # Determining the Standard
///
/// The packet standard can be inferred from the IRIG 106 version:
/// - Known releases before 106-17: Chapter 10
/// - Known releases from 106-17: Chapter 11
/// - Unknown codes: Unknown; no modern capability is inferred
///
/// Use [`from_version`](PacketStandard::from_version) to derive this
/// automatically from an [`Irig106Version`](crate::version::Irig106Version).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketStandard {
    /// Packet format defined by IRIG 106 Chapter 10 (pre-17 standards).
    Ch10,
    /// Packet format defined by IRIG 106 Chapter 11 (106-17 and later).
    Ch11,
    /// Version code does not establish packet provenance.
    Unknown,
}

impl PacketStandard {
    /// Derive the packet standard from an IRIG 106 version.
    ///
    /// 106-17 (version field 0x0C) and later use Chapter 11 packet formats.
    /// Earlier versions use Chapter 10.
    ///
    /// # Example
    ///
    /// ```
    /// use irig106_time::packet_standard::PacketStandard;
    /// use irig106_time::version::Irig106Version;
    ///
    /// assert_eq!(PacketStandard::from_version(&Irig106Version::V04), PacketStandard::Ch10);
    /// assert_eq!(PacketStandard::from_version(&Irig106Version::V07), PacketStandard::Ch10);
    /// assert_eq!(PacketStandard::from_version(&Irig106Version::V17), PacketStandard::Ch11);
    /// assert_eq!(PacketStandard::from_version(&Irig106Version::V22), PacketStandard::Ch11);
    /// ```
    pub fn from_version(version: &crate::version::Irig106Version) -> Self {
        use crate::version::Irig106Version;
        match version {
            Irig106Version::V04
            | Irig106Version::V05
            | Irig106Version::V07
            | Irig106Version::V09
            | Irig106Version::V11
            | Irig106Version::V13
            | Irig106Version::V15 => PacketStandard::Ch10,
            Irig106Version::V17 | Irig106Version::V19 | Irig106Version::V22 => PacketStandard::Ch11,
            Irig106Version::Unknown(_) => PacketStandard::Unknown,
        }
    }

    /// Returns `true` if this is a Chapter 11 packet format.
    #[inline]
    pub fn is_ch11(self) -> bool {
        matches!(self, PacketStandard::Ch11)
    }

    /// Returns `true` if this is a Chapter 10 packet format.
    #[inline]
    pub fn is_ch10(self) -> bool {
        matches!(self, PacketStandard::Ch10)
    }
}

impl core::fmt::Display for PacketStandard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PacketStandard::Ch10 => write!(f, "Chapter 10"),
            PacketStandard::Ch11 => write!(f, "Chapter 11"),
            PacketStandard::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
#[path = "packet_standard_tests.rs"]
mod packet_standard_tests;
