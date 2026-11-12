//! Separate shared setup-record and packet data-type version mappings.
//!
//! 106-24R1 §11.2.7.2 Figure 11-34 and §11.2.1.1 e use different lists.
//! Neither constructor resolves a recording edition or selects an EXT layout.
//! Setup version fields do not exist in 106-04/05. Q-EDITION is deferred.

pub use irig106_types::Irig106Version;

/// Decode bits 7–0 of a setup CSDW known to contain RCCVER (106-07+).
///
/// Reserved/unknown values, including zero and 0x0F, stay Unknown(raw).
/// Do not apply this function to reserved legacy setup words.
/// **Traces:** L3-EDN-102
pub fn detect_version(tmats_csdw: u32) -> Irig106Version {
    Irig106Version::from_setup_record((tmats_csdw & 0xff) as u8)
}

/// Decode a primary header data type version byte, not a setup code.
///
/// 106-24R1 §11.2.1.1 e permits a value at or below the release applied
/// to the data type. A returned V04 is not proof of a 106-04 recording.
/// **Traces:** L3-EDN-102
pub fn detect_data_type_version(version: u8) -> Irig106Version {
    Irig106Version::from_data_type_version(version)
}

/// Whether a caller-declared release defines Format 2 (from 106-17).
/// Unknown codes never enable the capability. No recording edition is inferred.
/// **Traces:** L3-EDN-101
pub fn supports_format_2(version: &Irig106Version) -> bool {
    matches!(
        version,
        Irig106Version::V17 | Irig106Version::V19 | Irig106Version::V22
    )
}

#[cfg(test)]
#[path = "version_tests.rs"]
mod version_tests;
