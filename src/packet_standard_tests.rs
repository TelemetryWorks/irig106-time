use super::*;
use alloc::format;
use crate::version::Irig106Version;

#[test]
fn pre17_is_ch10() {
    assert_eq!(PacketStandard::from_version(&Irig106Version::Pre07), PacketStandard::Ch10);
    assert_eq!(PacketStandard::from_version(&Irig106Version::V07), PacketStandard::Ch10);
    assert_eq!(PacketStandard::from_version(&Irig106Version::V09), PacketStandard::Ch10);
    assert_eq!(PacketStandard::from_version(&Irig106Version::V15), PacketStandard::Ch10);
}

#[test]
fn v17_and_later_is_ch11() {
    assert_eq!(PacketStandard::from_version(&Irig106Version::V17), PacketStandard::Ch11);
    assert_eq!(PacketStandard::from_version(&Irig106Version::V19), PacketStandard::Ch11);
    assert_eq!(PacketStandard::from_version(&Irig106Version::V22), PacketStandard::Ch11);
    assert_eq!(PacketStandard::from_version(&Irig106Version::V23), PacketStandard::Ch11);
}

#[test]
fn unknown_defaults_to_ch11() {
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::Unknown(0xFF)),
        PacketStandard::Ch11
    );
}

#[test]
fn is_ch10_and_is_ch11() {
    assert!(PacketStandard::Ch10.is_ch10());
    assert!(!PacketStandard::Ch10.is_ch11());
    assert!(PacketStandard::Ch11.is_ch11());
    assert!(!PacketStandard::Ch11.is_ch10());
}

#[test]
fn display_formatting() {
    assert_eq!(format!("{}", PacketStandard::Ch10), "Chapter 10");
    assert_eq!(format!("{}", PacketStandard::Ch11), "Chapter 11");
}
