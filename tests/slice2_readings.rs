//! New decoded-reading contracts; hand-derived values cited on each case.
use irig106_time::csdw::TimeF1Layout;
use irig106_time::format1::decode_time_f1;
use irig106_time::network_time::{NetworkTimeFormat, TimeStatus, decode_time_f2};
use irig106_time::scale::{ScaleOrigin, ScalePolicy, TimeScale, TimeScaleTagged};
use irig106_time::secondary::inspect_secondary_checksum;
use irig106_time::version::{Format2Availability, Irig106Version, format2_availability};

/// Requirements: L1-F2CSDW-004, L2-F2CSDW-104, L3-F2CSDW-104, L1-F2CSDW-005, L2-F2CSDW-105, L3-F2CSDW-105
#[test]
fn t2_each_ntf_and_status_remain_independent() {
    // Fig11-15, printed11-27/PDF35. All256 low bytes, zero body words.
    for ntf in 0..16u8 {
        for ts in 0..16u8 {
            let mut bytes = [0; 12];
            bytes[0] = (ntf << 4) | ts;
            let r = decode_time_f2(&bytes, 12).unwrap();
            assert_eq!(
                r.csdw.time_format(),
                match ntf {
                    0 => NetworkTimeFormat::NtpV3,
                    1 => NetworkTimeFormat::Ptp2002,
                    2 => NetworkTimeFormat::Ptp2008,
                    v => NetworkTimeFormat::Reserved(v),
                }
            );
            assert_eq!(
                r.csdw.time_status(),
                match ts {
                    0 => TimeStatus::NotValid,
                    1 => TimeStatus::Valid,
                    v => TimeStatus::Reserved(v),
                }
            );
            assert_eq!(r.time.is_some(), ntf < 3);
            assert_eq!(r.is_usable(), ntf < 3 && ts == 1);
            assert_eq!(
                irig106_time::network_time::parse_time_f2_payload(&bytes).is_ok(),
                ntf < 3 && ts == 1
            );
        }
    }
    for bit in 8..32 {
        let mut bytes = [0; 12];
        bytes[..4].copy_from_slice(&((1u32 << bit) | 0x21).to_le_bytes());
        let r = decode_time_f2(&bytes, 12).unwrap();
        assert_eq!(r.csdw.reserved_bits(), 1u32 << bit);
        assert!(r.csdw.validate_reserved().is_err());
        assert!(r.is_usable()); // TF-005 warning never discards the decoded value.
    }
}

/// Requirements: L1-ERR-005, L2-ERR-105, L3-ERR-105, L1-PTP-003, L2-PTP-103, L3-PTP-103
#[test]
fn f2_data_length_bounds_and_nanosecond_boundaries() {
    // Fig11-17, 11-27/PDF35; §11.2.1.1d excludes trailer bytes.
    for nanos in [0u32, 999_999_999, 1_000_000_000, u32::MAX] {
        let mut bytes = [0xff; 16];
        bytes[..12].fill(0);
        bytes[0] = 0x21;
        bytes[8..12].copy_from_slice(&nanos.to_le_bytes());
        assert_eq!(decode_time_f2(&bytes, 12).is_ok(), nanos < 1_000_000_000);
        for n in 0..12 {
            assert!(decode_time_f2(&bytes, n).is_err());
        }
        assert!(decode_time_f2(&bytes, 17).is_err());
    }
    for (fraction, nanos) in [
        (0u32, 0),
        (0x80000000, 500_000_000),
        (u32::MAX, 999_999_999),
    ] {
        let mut bytes = [0xff; 16];
        bytes[..4].copy_from_slice(&1u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[8..12].copy_from_slice(&fraction.to_le_bytes());
        let r = decode_time_f2(&bytes, 12).unwrap();
        assert_eq!(r.consumed, 12);
        let irig106_time::NetworkTime::Ntp(t) = r.time.unwrap().value else {
            panic!("not NTP")
        };
        assert_eq!(
            (t.seconds, t.fraction, t.fraction_as_nanos()),
            (u32::MAX, fraction, nanos)
        );
    }
}

/// Requirements: L1-EDN-001, L2-EDN-101, L3-EDN-101
#[test]
fn t4_explicit_availability_never_infers_unknown_edition() {
    // 106-17 Table11-4, printed11-6/PDF14: 0x12, current DT version0x08;
    // Figures11-14..16, 11-24/PDF32. Not an inference from a header code.
    assert_eq!(
        format2_availability(&Irig106Version::V15),
        Format2Availability::Unavailable
    );
    for v in [
        Irig106Version::V17,
        Irig106Version::V19,
        Irig106Version::V22,
    ] {
        assert_eq!(format2_availability(&v), Format2Availability::Available);
    }
    assert_eq!(
        format2_availability(&Irig106Version::Unknown(15)),
        Format2Availability::Unknown
    );
}

/// Requirements: L1-TAI-005, L2-TAI-105, L3-TAI-105
/// Interpretations: RDG-009
#[test]
fn t17_f2_stated_scales_do_not_depend_on_counter_value() {
    // §11.2.3.3, printed11-26/PDF34: NTP UTC, PTP TAI.
    for (ntf, scale) in [
        (0, TimeScale::Utc),
        (1, TimeScale::Tai),
        (2, TimeScale::Tai),
    ] {
        let mut bytes = [0; 12];
        bytes[0] = (ntf << 4) | 1;
        let r = decode_time_f2(&bytes, 12).unwrap();
        let time = r.time.unwrap();
        assert_eq!(time.scale.effective(), scale);
        assert_eq!(time.scale.origin(), ScaleOrigin::Standard);
        assert_eq!(time.value.scale(), time.scale);
    }
}

/// Requirements: L1-TAI-005, L2-TAI-105, L3-TAI-105, L1-TAI-006, L2-TAI-106, L3-TAI-106
/// Interpretations: RDG-009
#[test]
fn t17_all_fmt_scales_and_policy_provenance() {
    // Fig11-13/FMT, printed11-25/PDF33; IRIG assumption RCC200-16 §1 p1-1/PDF13.
    for fmt in 0..16u8 {
        let mut bytes = [0; 10];
        bytes[0] = fmt << 4;
        bytes[8] = 1;
        let r = decode_time_f1(
            &bytes,
            10,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default(),
        )
        .unwrap();
        if fmt >= 6 {
            assert!(r.time.is_none());
            continue;
        }
        let t = r.time.unwrap();
        let expected = match fmt {
            0..=2 => TimeScale::Utc,
            3 => TimeScale::Unknown,
            4 => TimeScale::Utc,
            5 => TimeScale::Gps,
            _ => unreachable!(),
        };
        assert_eq!(t.scale.effective(), expected);
        assert_eq!(
            t.scale.origin(),
            match fmt {
                0..=2 => ScaleOrigin::Policy,
                3 => ScaleOrigin::Unspecified,
                _ => ScaleOrigin::Standard,
            }
        );
        if fmt == 5 {
            assert!(t.to_absolute().is_err());
        }
    }
    let mut bytes = [0; 10];
    bytes[0] = 0x30;
    bytes[8] = 1;
    let r = decode_time_f1(
        &bytes,
        10,
        TimeF1Layout::Source05,
        ScalePolicy {
            irig: TimeScale::Unknown,
            rtc: TimeScale::Gps,
        },
    )
    .unwrap();
    let t = r.time.unwrap();
    assert_eq!(t.scale.stated(), TimeScale::Unknown);
    assert_eq!(t.scale.effective(), TimeScale::Gps);
    assert_eq!(t.scale.origin(), ScaleOrigin::Policy);
    for fmt in [0, 1, 2, 3, 4, 5] {
        let mut bytes = [0; 14];
        bytes[0] = fmt << 4;
        bytes[8] = 1;
        bytes[10..].fill(0xff);
        let r = decode_time_f1(
            &bytes,
            10,
            TimeF1Layout::Source05,
            ScalePolicy {
                irig: TimeScale::Tai,
                rtc: TimeScale::Utc,
            },
        )
        .unwrap();
        let t = r.time.unwrap();
        assert_eq!(r.consumed, 10);
        assert_eq!(
            t.scale.effective(),
            match fmt {
                0..=2 => TimeScale::Tai,
                3 | 4 => TimeScale::Utc,
                5 => TimeScale::Gps,
                _ => unreachable!(),
            }
        );
        for n in 0..10 {
            assert!(
                decode_time_f1(&bytes, n, TimeF1Layout::Source05, ScalePolicy::default()).is_err()
            );
        }
        assert!(
            decode_time_f1(&bytes, 15, TimeF1Layout::Source05, ScalePolicy::default()).is_err()
        );
    }
    let bytes = [0x30, 0, 0, 0, 0, 0x60, 0x59, 0x23, 0x81, 1];
    assert!(
        decode_time_f1(
            &bytes,
            10,
            TimeF1Layout::Source05,
            ScalePolicy {
                irig: TimeScale::Utc,
                rtc: TimeScale::Utc
            }
        )
        .is_err()
    );
}

/// Requirements: L1-TAI-007, L2-TAI-107, L3-TAI-107
/// Interpretations: RDG-010, RDG-009
#[test]
fn t16_leap_second_retained_only_at_permitted_utc_dates() {
    // Fig11-13, 11-25/PDF33: TSn=6 encoded in three bits.
    // RCC200-16 A.2, A-1/PDF56 permits June30/Dec31. This reading is RDG-010.
    for day in [180u16, 181, 182, 183, 364, 365, 366] {
        for fmt in 0..6u8 {
            let bytes = [
                fmt << 4,
                0,
                0,
                0,
                0,
                0x60,
                0x59,
                0x23,
                (((day / 10 % 10) << 4) | (day % 10)) as u8,
                (day / 100) as u8,
            ];
            let r = decode_time_f1(
                &bytes,
                10,
                TimeF1Layout::SourceWithIts17,
                ScalePolicy::default(),
            );
            let valid = matches!(day, 181 | 182 | 365 | 366) && matches!(fmt, 0..=2 | 4);
            assert_eq!(r.is_ok(), valid, "day{day} fmt{fmt}");
            if let Ok(r) = r {
                let t = r.time.unwrap();
                assert_eq!(t.seconds, 60);
                assert!(t.is_leap_second());
                assert!(t.to_absolute().is_err());
            }
        }
    }
}

/// Requirements: L1-TAI-007, L2-TAI-107, L3-TAI-107
/// Interpretations: RDG-010
#[test]
fn t16_calendar_known_year_and_wrong_time_boundaries() {
    // Fig11-14, 11-25/PDF33: explicit dates, independently packed.
    for (month, day, valid) in [
        (0x06, 0x30, true),
        (0x12, 0x31, true),
        (0x06, 0x29, false),
        (0x12, 0x30, false),
    ] {
        let bytes = [0x40, 2, 0, 0, 0, 0x60, 0x59, 0x23, day, month, 0x24, 0x20];
        assert_eq!(
            decode_time_f1(&bytes, 12, TimeF1Layout::Source05, ScalePolicy::default()).is_ok(),
            valid
        );
    }
    let mut bytes = [0x40, 0, 0, 0, 0, 0x60, 0x59, 0x23, 0x81, 1];
    for (offset, value) in [(7, 0x22), (6, 0x58), (5, 0x61)] {
        let old = bytes[offset];
        bytes[offset] = value;
        assert!(
            decode_time_f1(&bytes, 10, TimeF1Layout::Source05, ScalePolicy::default()).is_err()
        );
        bytes[offset] = old;
    }
    let bytes = [0x40, 0, 0, 0, 0x99, 0x60, 0x59, 0x23, 0x82, 1];
    let t = decode_time_f1(&bytes, 10, TimeF1Layout::Source05, ScalePolicy::default())
        .unwrap()
        .time
        .unwrap();
    assert_eq!((t.seconds, t.nanoseconds), (60, 990_000_000));
    assert!(t.with_year(2024).is_ok());
    assert!(t.with_year(2023).is_err());
    let bytes = [0x40, 0, 0, 0, 0, 0x60, 0x59, 0x23, 0x65, 3];
    let t = decode_time_f1(&bytes, 10, TimeF1Layout::Source05, ScalePolicy::default())
        .unwrap()
        .time
        .unwrap();
    assert!(t.with_year(2023).is_ok());
    assert!(t.with_year(2024).is_err());
    let bytes = [0x40, 2, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x29, 2, 0x23, 0x20]; // invalid 2023-Feb29
    assert!(decode_time_f1(&bytes, 12, TimeF1Layout::Source05, ScalePolicy::default()).is_err());
}

/// Requirements: L1-SEC-004, L2-SEC-104, L3-SEC-104, L1-ERR-004, L2-ERR-104, L3-ERR-104
/// Interpretations: RDG-008
#[test]
fn rdg008_reports_byte_word_both_and_neither() {
    // 106-24R1 §11.2.1.2c, printed11-10/PDF18; RCC123-20 B-31/PDF137.
    for (bytes, byte_ok, word_ok) in [
        ([1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0], true, false),
        ([1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2], false, true),
        ([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0], true, true),
        ([1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0], false, false),
    ] {
        let r = inspect_secondary_checksum(&bytes).unwrap();
        assert_eq!((r.byte_valid(), r.word_valid()), (byte_ok, word_ok));
        assert_eq!(r.stored, u16::from_le_bytes([bytes[10], bytes[11]]));
    }
    let mut bytes = [0xff; 16];
    bytes[10] = 0xf6;
    bytes[11] = 9; // ten*255=2550=0x09f6
    let r = inspect_secondary_checksum(&bytes).unwrap();
    assert_eq!((r.byte_sum, r.word_sum), (2550, 65531));
    assert!(r.byte_valid());
    assert!(!r.word_valid());
    for n in 0..12 {
        assert!(inspect_secondary_checksum(&bytes[..n]).is_err());
    }
    bytes[12..].fill(0);
    assert_eq!(inspect_secondary_checksum(&bytes).unwrap(), r);
}
