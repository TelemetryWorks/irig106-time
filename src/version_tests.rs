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
