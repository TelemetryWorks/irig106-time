//! Existing correct controls, not claims of main-first conversion failures.
use irig106_time::{NtpTime, TimeError};
/// Requirements: L1-NTP-001, L2-NTP-101, L3-NTP-101
#[test]
fn figure11_16_little_endian_words_and_bounds_are_reused() {
    // 106-24R1 Fig11-16, 11-27/PDF35: two32-bit LE words; no network order.
    let bytes = [4, 3, 2, 1, 0, 0, 0, 0x80, 0xff, 0xff];
    let t = NtpTime::from_le_bytes(&bytes).unwrap();
    assert_eq!((t.seconds, t.fraction), (0x01020304, 0x80000000));
    for n in 0..8 {
        assert!(NtpTime::from_le_bytes(&bytes[..n]).is_err());
    }
}
/// Requirements: L1-NTP-003, L2-NTP-103, L3-NTP-103
#[test]
fn accepted_floor_fraction_boundaries_are_already_correct() {
    for (fraction, ns) in [
        (0, 0),
        (1, 0),
        (4, 0),
        (5, 1),
        (0x7fffffff, 499999999),
        (0x80000000, 500000000),
        (0xffffffff, 999999999),
    ] {
        let t = NtpTime {
            seconds: 1,
            fraction,
        };
        assert_eq!(t.fraction_as_nanos(), ns);
        let remainder = u64::from(fraction) * 1_000_000_000 - u64::from(ns) * (1u64 << 32);
        assert!(remainder < (1u64 << 32));
    }
}
/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104
#[test]
fn contextless_compatibility_helpers_cannot_discard_era_basis() {
    let t = NtpTime {
        seconds: 2208988800,
        fraction: 0,
    };
    assert!(matches!(
        t.to_absolute(),
        Err(TimeError::NeedsContext { .. })
    ));
    assert!(matches!(
        t.to_unix_seconds(),
        Err(TimeError::NeedsContext { .. })
    ));
}
