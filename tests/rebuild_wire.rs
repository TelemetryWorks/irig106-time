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
    // Figure 11-4: reserved/usec/low/high. RDG-008 byte sum:
    // 0x0f+0x27+0x34+0x12+1 = 0x7d; word sum would be 0x3944.
    let bytes = [0, 0, 0x0f, 0x27, 0x34, 0x12, 1, 0, 0, 0, 0x7d, 0];
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
    let top = [0, 0, 0, 0, 0, 0x80, 0, 0, 0, 0, 0x80, 0];
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
    reserved[10] = 0x7e;
    let SecondaryHeaderTime::Ch4(t) =
        parse_secondary_header(&reserved, SecHdrTimeFormat::Ch4).unwrap()
    else {
        panic!("wrong reserved-word dispatch")
    };
    assert_eq!(
        (t.high_order, t.low_order, t.microseconds),
        (1, 0x1234, 9999)
    );
    let stamp = irig106_time::intra_packet::parse_intra_packet_time(
        &reserved[..8],
        IntraPacketTimeFormat::Ch4Binary,
    )
    .unwrap();
    assert_eq!(stamp, irig106_time::intra_packet::IntraPacketTime::Ch4(t));
    let too_many_usec = [0, 0, 0x10, 0x27, 0, 0, 0, 0, 0, 0, 0x37, 0];
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
    // 106-24R1 §11.2.1.1 g, printed 11-8/11-9 (PDF 16/17):
    // bit 6 selects source, bit 7 presence, bits 3-2 secondary format.
    // §11.2.1.3 b, printed 11-11 (PDF 19): RTC is 48 bits + 16 zero bits.
    // Hand-derived truth table: every source=0 row is RTC; source=1
    // needs presence=1 and format 00/01/10. Other bits do not select time.
    let mut counts = [0; 3]; // RTC, secondary, invalid
    for flags in 0..=255u8 {
        let source = flags & 0x40 != 0;
        let present = flags & 0x80 != 0;
        let format = (flags >> 2) & 3;
        let expected = match (source, present, format) {
            (false, _, _) => {
                counts[0] += 1;
                IntraPacketTimeFormat::Rtc48
            }
            (true, true, 0) => {
                counts[1] += 1;
                IntraPacketTimeFormat::Ch4Binary
            }
            (true, true, 1) => {
                counts[1] += 1;
                IntraPacketTimeFormat::Ieee1588
            }
            (true, true, 2) => {
                counts[1] += 1;
                IntraPacketTimeFormat::Ertc64
            }
            _ => {
                counts[2] += 1;
                IntraPacketTimeFormat::InvalidFlags(flags)
            }
        };
        let fmt = IntraPacketTimeFormat::from_packet_flags(flags);
        assert_eq!(fmt, expected, "flags {flags:#04x}");
        assert_eq!(
            parse_intra_packet_time(&[0; 8], fmt).is_err(),
            matches!(expected, IntraPacketTimeFormat::InvalidFlags(_)),
            "flags {flags:#04x}"
        );
    }
    assert_eq!(counts, [128, 48, 80]);
}

/// Requirements: L1-SEC-001, L2-SEC-101, L3-SEC-101
#[test]
fn t10_intra_ch4_reserved_word_keeps_value() {
    use irig106_time::intra_packet::{IntraPacketTime, parse_intra_packet_time};
    // Figure 11-4 reserved/usec/low/high; reserved is a TF-005 warning.
    let bytes = [1, 0, 0x0f, 0x27, 0x34, 0x12, 1, 0];
    let IntraPacketTime::Ch4(t) =
        parse_intra_packet_time(&bytes, IntraPacketTimeFormat::Ch4Binary).unwrap()
    else {
        panic!("wrong dispatch")
    };
    assert_eq!(
        (t.high_order, t.low_order, t.microseconds),
        (1, 0x1234, 9999)
    );
}

/// Requirements: L1-RTC-002, L2-RTC-102, L3-RTC-102
#[test]
fn rtc_padding_warning_is_separate_from_decoding() {
    use irig106_time::intra_packet::{
        IntraPacketTime, parse_intra_packet_time, validate_rtc48_reserved,
    };
    use irig106_time::{Rtc, TimeError};
    let mut bytes = [1, 2, 3, 4, 5, 6, 0, 0];
    assert_eq!(validate_rtc48_reserved(&bytes), Ok(()));
    for bit in 0..16 {
        bytes[6..8].copy_from_slice(&(1u16 << bit).to_le_bytes());
        assert_eq!(
            parse_intra_packet_time(&bytes, IntraPacketTimeFormat::Rtc48).unwrap(),
            IntraPacketTime::Rtc(Rtc::from_raw(0x0605_0403_0201))
        );
        assert_eq!(
            validate_rtc48_reserved(&bytes),
            Err(TimeError::ReservedBitSet {
                position: "RTC stamp padding",
            })
        );
    }
    for n in 0..8 {
        assert!(
            matches!(validate_rtc48_reserved(&bytes[..n]), Err(TimeError::BufferTooShort { expected: 8, actual }) if actual == n)
        );
    }
}

/// Requirements: L1-SEC-001, L1-SEC-005, L2-SEC-101, L3-SEC-101, L2-SEC-105, L3-SEC-105
#[test]
fn ch4_reserved_words_report_independent_warnings_and_keep_time() {
    use irig106_time::TimeError;
    use irig106_time::intra_packet::{IntraPacketTime, parse_intra_packet_time};
    use irig106_time::secondary::{validate_ch4_reserved, validate_secondary_reserved};
    // RDG-008 byte sum: 0x0f+0x27+0x80+1 = 0xb7 (word alternative 0xa710).
    let mut bytes = [0, 0, 0x0f, 0x27, 0, 0x80, 1, 0, 0, 0, 0xb7, 0];
    assert_eq!(validate_ch4_reserved(&bytes), Ok(()));
    assert_eq!(validate_secondary_reserved(&bytes), Ok(()));
    for bit in 0..16 {
        let reserved = 1u16 << bit;
        for offset in [0, 8] {
            bytes[0..2].fill(0);
            bytes[8..10].fill(0);
            bytes[offset..offset + 2].copy_from_slice(&reserved.to_le_bytes());
            bytes[10..12]
                .copy_from_slice(&(0xb7u16 + (reserved & 255) + (reserved >> 8)).to_le_bytes());
            let SecondaryHeaderTime::Ch4(t) =
                parse_secondary_header(&bytes, SecHdrTimeFormat::Ch4).unwrap()
            else {
                panic!("wrong dispatch")
            };
            assert_eq!(
                (t.high_order, t.low_order, t.microseconds),
                (1, 0x8000, 9999)
            );
            assert_eq!(
                parse_intra_packet_time(&bytes[..8], IntraPacketTimeFormat::Ch4Binary).unwrap(),
                IntraPacketTime::Ch4(t)
            );
            assert_eq!(validate_ch4_reserved(&bytes).is_err(), offset == 0);
            assert_eq!(validate_secondary_reserved(&bytes).is_err(), offset == 8);
        }
    }
    // Both findings remain independently observable without suppressing time.
    bytes[0..2].copy_from_slice(&1u16.to_le_bytes());
    bytes[8..10].copy_from_slice(&2u16.to_le_bytes());
    bytes[10..12].copy_from_slice(&0xbau16.to_le_bytes());
    assert!(parse_secondary_header(&bytes, SecHdrTimeFormat::Ch4).is_ok());
    assert_eq!(
        validate_ch4_reserved(&bytes),
        Err(TimeError::ReservedBitSet {
            position: "Chapter 4 reserved word"
        })
    );
    assert_eq!(
        validate_secondary_reserved(&bytes),
        Err(TimeError::ReservedBitSet {
            position: "secondary header reserved word"
        })
    );
    for n in 0..8 {
        assert!(
            matches!(validate_ch4_reserved(&bytes[..n]), Err(TimeError::BufferTooShort { expected: 8, actual }) if actual == n)
        );
    }
    for n in 0..12 {
        assert!(
            matches!(validate_secondary_reserved(&bytes[..n]), Err(TimeError::BufferTooShort { expected: 12, actual }) if actual == n)
        );
    }
}

/// Requirements: L1-ABS-002, L2-ABS-102, L3-ABS-102
#[test]
fn ch4_invalid_microseconds_still_reject_both_paths() {
    use irig106_time::TimeError;
    use irig106_time::intra_packet::parse_intra_packet_time;
    // RDG-008: 0x10+0x27+reserved = 0x37+reserved, byte reading.
    for reserved in [0, 1] {
        let bytes = [
            reserved,
            0,
            0x10,
            0x27,
            0,
            0,
            0,
            0,
            0,
            0,
            0x37 + reserved,
            0,
        ];
        let expected = Err(TimeError::OutOfRange {
            field: "ch4_microseconds",
            value: 10000,
            max: 9999,
        });
        assert_eq!(
            parse_secondary_header(&bytes, SecHdrTimeFormat::Ch4),
            expected
        );
        assert_eq!(
            parse_intra_packet_time(&bytes[..8], IntraPacketTimeFormat::Ch4Binary),
            Err(TimeError::OutOfRange {
                field: "ch4_microseconds",
                value: 10000,
                max: 9999
            })
        );
    }
}

/// Requirements: L1-IPT-007, L2-IPT-107, L3-IPT-107
#[test]
fn qflags_literal_rtc_reserved_format_keeps_value() {
    use irig106_time::intra_packet::{IntraPacketTime, parse_intra_packet_time};
    // §11.2.1.3 b, printed 11-11: six LE counter bytes, two zero bytes.
    // 0x060504030201 is hand-authored, not generated by an encoder.
    let bytes = [1, 2, 3, 4, 5, 6, 0, 0];
    for flags in [0x0c, 0x8c] {
        let fmt = IntraPacketTimeFormat::from_packet_flags(flags);
        let value = parse_intra_packet_time(&bytes, fmt).unwrap();
        assert_eq!(
            value,
            IntraPacketTime::Rtc(irig106_time::Rtc::from_raw(0x0605_0403_0201)),
            "flags {flags:#04x}"
        );
    }
}

/// Requirements: L1-IPT-007, L2-IPT-107, L3-IPT-107
#[test]
fn qflags_literal_secondary_errors_are_distinct() {
    use irig106_time::intra_packet::parse_intra_packet_time;
    // §11.2.1.1 g, printed 11-8/11-9: 11 is undefined secondary time.
    // Q-FLAGS-ERROR numeric finding ID is still open; use a distinct error.
    let error = parse_intra_packet_time(&[0; 8], IntraPacketTimeFormat::from_packet_flags(0xcc))
        .unwrap_err();
    // Debug comparison deliberately uses only the baseline public API,
    // so this tests-first commit compiles and fails behaviourally on main.
    assert_eq!(
        format!("{error:?}"),
        "UndefinedSecondaryTimeFormat { flags: 204 }"
    );
    for flags in [0x4c, 0x40] {
        assert_eq!(
            parse_intra_packet_time(&[0; 8], IntraPacketTimeFormat::from_packet_flags(flags)),
            Err(irig106_time::TimeError::InvalidPacketFlags { flags })
        );
    }
}
