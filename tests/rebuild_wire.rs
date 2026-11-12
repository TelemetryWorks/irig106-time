//! Independent standard fixtures for the first rebuild slice.
//! These bytes come from 106-24R1 figures, not the crate's encoders.
use irig106_time::bcd::{DayFormatTime, DmyFormatTime};
use irig106_time::intra_packet::IntraPacketTimeFormat;
use irig106_time::secondary::{SecondaryHeaderTime, parse_secondary_header};
use irig106_time::{Ertc, SecHdrTimeFormat, TimeF1Csdw};

/// Requirements: L1-CSDW-002, L2-CSDW-102, L3-CSDW-102
#[test]
fn t6_source_three_is_reserved() {
    // §11.2.3.2 a: 3–14 reserved, not GPS.
    assert_eq!(
        format!("{:?}", TimeF1Csdw::from_raw(3).time_source()),
        "Reserved(3)"
    );
}

/// Requirements: L1-CSDW-002, L2-CSDW-102, L3-CSDW-102
#[test]
fn t6_source_two_is_rmm() {
    assert_eq!(
        format!("{:?}", TimeF1Csdw::from_raw(2).time_source()),
        "InternalFromRmm"
    );
}

/// Requirements: L1-CSDW-003, L2-CSDW-103, L3-CSDW-103
#[test]
fn t7_none_format_is_distinct() {
    // §11.2.3.2 a: FMT 0xF means NONE (payload invalid).
    assert_eq!(
        format!("{:?}", TimeF1Csdw::from_raw(0xf0).time_format()),
        "None"
    );
}

/// Requirements: L1-IPT-005, L2-IPT-105, L3-IPT-105
#[test]
fn t1_secondary_ch4_selected_by_bit_six() {
    // §11.2.1.1 g: presence=7, source=6, format=3–2.
    assert_eq!(
        IntraPacketTimeFormat::from_packet_flags(0xc0),
        IntraPacketTimeFormat::Ch4Binary
    );
}

/// Requirements: L1-IPT-006, L2-IPT-106, L3-IPT-106
#[test]
fn t1_bit_two_alone_does_not_select_secondary() {
    assert_eq!(
        IntraPacketTimeFormat::from_packet_flags(0x04),
        IntraPacketTimeFormat::Rtc48
    );
}

/// Requirements: L1-BCD-005, L2-BCD-105, L3-BCD-105, L1-ERR-005
#[test]
fn t12_day_body_is_six_bytes() {
    // Figure 11-13, contract §7: day187, 13:45:27.350.
    let t = DayFormatTime::from_le_bytes(&[0x35, 0x27, 0x45, 0x13, 0x87, 0x01]).unwrap();
    assert_eq!(
        (
            t.day_of_year(),
            t.hours(),
            t.minutes(),
            t.seconds(),
            t.milliseconds()
        ),
        (187, 13, 45, 27, 350)
    );
    assert_eq!(t.to_le_bytes().len(), 6);
    let body = [0x35, 0x27, 0x45, 0x13, 0x87, 0x01];
    assert_eq!(t.to_le_bytes(), body);
    for n in 0..6 {
        assert!(DayFormatTime::from_le_bytes(&body[..n]).is_err());
    }
    let mut with_filler = body.to_vec();
    with_filler.extend_from_slice(&[0xff; 4]);
    assert_eq!(DayFormatTime::from_le_bytes(&with_filler).unwrap(), t);
}

/// Requirements: L1-BCD-006, L2-BCD-106, L3-BCD-106, L1-ERR-005
#[test]
fn t12_calendar_body_is_eight_bytes() {
    // Figure 11-14, independently packed 2024-02-29 13:45:27.350.
    let t =
        DmyFormatTime::from_le_bytes(&[0x35, 0x27, 0x45, 0x13, 0x29, 0x02, 0x24, 0x20]).unwrap();
    assert_eq!((t.year(), t.month(), t.day()), (2024, 2, 29));
    assert_eq!(t.to_le_bytes().len(), 8);
    let body = [0x35, 0x27, 0x45, 0x13, 0x29, 0x02, 0x24, 0x20];
    assert_eq!(t.to_le_bytes(), body);
    for n in 0..8 {
        assert!(DmyFormatTime::from_le_bytes(&body[..n]).is_err());
    }
    let mut with_filler = body.to_vec();
    with_filler.extend_from_slice(&[0xff; 4]);
    assert_eq!(DmyFormatTime::from_le_bytes(&with_filler).unwrap(), t);
}

/// Requirements: L1-EDN-002, L2-EDN-102, L3-EDN-102
#[test]
fn t9_setup_fifteen_is_unknown() {
    use irig106_time::version::{Irig106Version, detect_version};
    assert_eq!(detect_version(15), Irig106Version::Unknown(15));
    assert_eq!(detect_version(0), Irig106Version::Unknown(0));
}

/// Requirements: L1-ABS-002, L2-ABS-102, L3-ABS-102, L1-SEC-001
#[test]
fn t10_secondary_ch4_has_usec_low_high_words() {
    // Figure 11-4: reserved/usec/low/high. 0x270f+0x1234+1=0x3944.
    // This slice retains the prototype word-sum checksum; RDG-008 is deferred.
    let bytes = [0, 0, 0x0f, 0x27, 0x34, 0x12, 1, 0, 0, 0, 0x44, 0x39];
    let SecondaryHeaderTime::Ch4(t) =
        parse_secondary_header(&bytes, SecHdrTimeFormat::Ch4).unwrap()
    else {
        panic!("wrong dispatch")
    };
    assert_eq!(
        (t.high_order, t.low_order, t.microseconds),
        (1, 0x1234, 9999)
    );
    // Top binary low-word bit is retained (DERIVED reading, §4.7).
    let top = [0, 0, 0, 0, 0, 0x80, 0, 0, 0, 0, 0, 0x80];
    let SecondaryHeaderTime::Ch4(t) = parse_secondary_header(&top, SecHdrTimeFormat::Ch4).unwrap()
    else {
        panic!("wrong dispatch")
    };
    assert_eq!((t.high_order, t.low_order, t.microseconds), (0, 0x8000, 0));
    let stamp = irig106_time::intra_packet::parse_intra_packet_time(
        &bytes[..8],
        IntraPacketTimeFormat::Ch4Binary,
    )
    .unwrap();
    let irig106_time::intra_packet::IntraPacketTime::Ch4(t) = stamp else {
        panic!("wrong stamp dispatch")
    };
    assert_eq!(
        (t.high_order, t.low_order, t.microseconds),
        (1, 0x1234, 9999)
    );
    for n in 0..12 {
        assert!(parse_secondary_header(&bytes[..n], SecHdrTimeFormat::Ch4).is_err());
    }
    let mut reserved = bytes;
    reserved[0] = 1;
    reserved[10] = 0x45;
    assert!(parse_secondary_header(&reserved, SecHdrTimeFormat::Ch4).is_err());
    let too_many_usec = [0, 0, 0x10, 0x27, 0, 0, 0, 0, 0, 0, 0x10, 0x27];
    assert!(parse_secondary_header(&too_many_usec, SecHdrTimeFormat::Ch4).is_err());
}

/// Requirements: L1-ABS-004, L2-ABS-104, L3-ABS-104
#[test]
fn t5_shared_ertc_tick_is_one_nanosecond() {
    assert_eq!(
        Ertc::from_le_bytes(&[1, 0, 0, 0, 0, 0, 0, 0])
            .unwrap()
            .to_nanos(),
        1
    );
}

/// Requirements: L1-IPT-005, L1-IPT-006, L1-IPT-007, L2-IPT-107, L3-IPT-107
#[test]
fn t1_all_flag_combinations_are_checked() {
    use irig106_time::intra_packet::parse_intra_packet_time;
    for flags in 0..=255u8 {
        let invalid = ((flags & 0x40 != 0) && (flags & 0x80 == 0)) || (flags >> 2) & 3 == 3;
        let expected = if invalid {
            format!("InvalidFlags({flags})")
        } else if flags & 0x40 == 0 {
            "Rtc48".into()
        } else {
            match (flags >> 2) & 3 {
                0 => "Ch4Binary",
                1 => "Ieee1588",
                2 => "Ertc64",
                _ => unreachable!(),
            }
            .into()
        };
        let fmt = IntraPacketTimeFormat::from_packet_flags(flags);
        assert_eq!(format!("{fmt:?}"), expected, "flags {flags:#04x}");
        assert_eq!(parse_intra_packet_time(&[0; 8], fmt).is_err(), invalid);
    }
}
