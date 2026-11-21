use super::*;
use crate::version::Irig106Version;
use alloc::format;

/// Requirements: L1-EDN-002
#[test]
fn pre17_is_ch10() {
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V04),
        PacketStandard::Ch10
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V07),
        PacketStandard::Ch10
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V09),
        PacketStandard::Ch10
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V15),
        PacketStandard::Ch10
    );
}

/// Requirements: L1-EDN-002
#[test]
fn v17_and_later_is_ch11() {
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V17),
        PacketStandard::Ch11
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V19),
        PacketStandard::Ch11
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V22),
        PacketStandard::Ch11
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V22),
        PacketStandard::Ch11
    );
}

/// Requirements: L1-EDN-002
#[test]
fn unknown_has_unknown_provenance() {
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::Unknown(0xFF)),
        PacketStandard::Unknown
    );
}

/// Requirements: L1-EDN-002
#[test]
fn is_ch10_and_is_ch11() {
    assert!(PacketStandard::Ch10.is_ch10());
    assert!(!PacketStandard::Ch10.is_ch11());
    assert!(PacketStandard::Ch11.is_ch11());
    assert!(!PacketStandard::Ch11.is_ch10());
}

/// Requirements: L1-EDN-002
#[test]
fn display_formatting() {
    assert_eq!(format!("{}", PacketStandard::Ch10), "Chapter 10");
    assert_eq!(format!("{}", PacketStandard::Ch11), "Chapter 11");
}
