//! 106-24R1 §11.2.7.2/Figure 11-34 and §11.2.1.1 e.
use super::*;

/// Requirements: L1-EDN-002, L2-EDN-102, L3-EDN-102
#[test]
fn separate_version_codes_from_standard() {
    let setup = [
        Irig106Version::V07,
        Irig106Version::V09,
        Irig106Version::V11,
        Irig106Version::V13,
        Irig106Version::V15,
        Irig106Version::V17,
        Irig106Version::V19,
        Irig106Version::V22,
    ];
    let header = [
        Irig106Version::V04,
        Irig106Version::V05,
        Irig106Version::V07,
        Irig106Version::V09,
        Irig106Version::V11,
        Irig106Version::V13,
        Irig106Version::V15,
        Irig106Version::V17,
        Irig106Version::V19,
        Irig106Version::V22,
    ];
    for code in 0..=255u8 {
        let expected_setup = if (7..=14).contains(&code) {
            setup[(code - 7) as usize]
        } else {
            Irig106Version::Unknown(code)
        };
        let expected_header = if (1..=10).contains(&code) {
            header[(code - 1) as usize]
        } else {
            Irig106Version::Unknown(code)
        };
        assert_eq!(detect_version(code as u32 | 0xffff_ff00), expected_setup);
        assert_eq!(detect_data_type_version(code), expected_header);
    }
}

/// Requirements: L1-EDN-001
#[test]
fn format_two_requires_known_declared_release() {
    assert!(!supports_format_2(&Irig106Version::V15));
    for v in [
        Irig106Version::V17,
        Irig106Version::V19,
        Irig106Version::V22,
    ] {
        assert!(supports_format_2(&v));
    }
    assert!(!supports_format_2(&Irig106Version::Unknown(0xff)));
}
