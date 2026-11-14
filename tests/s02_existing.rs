//! S02 executed-before-fix probes, separate from missing-API compilation evidence.
use irig106_time::{AbsoluteTime, DayFormatTime, TimeError};
use irig106_time::{ScalePolicy, TimeF1Layout, format1::decode_time_f1};
/// Requirements: L1-ABS-001, L2-ABS-101, L3-ABS-101, L1-ERR-003, L2-ERR-103, L3-ERR-103
#[test]
fn year_attachment_validates_gregorian_day() {
    // Gregorian centuries: 1900/2100 common, 2000 leap; no LY inference.
    for (year, leap) in [(1900, false), (2000, true), (2100, false)] {
        assert_eq!(
            AbsoluteTime::new(366, 0, 0, 0, 0)
                .unwrap()
                .with_year(Some(year))
                .is_ok(),
            leap
        );
    }
}
/// Requirements: L1-ABS-001, L2-ABS-101, L3-ABS-101
#[test]
fn nominal_add_advances_the_known_year() {
    // Exactly 1 nominal second from the year's last second to next Jan1.
    for (year, last) in [(1900, 365), (2000, 366), (2100, 365)] {
        let t = AbsoluteTime::new(last, 23, 59, 59, 0)
            .unwrap()
            .with_year(Some(year))
            .unwrap();
        let next = t.add_nanos(1_000_000_000);
        assert_eq!(
            (next.year(), next.day_of_year(), next.seconds()),
            (Some(year + 1), 1, 0)
        );
    }
}
/// Requirements: L1-BCD-004, L2-BCD-104, L3-BCD-104, L1-ERR-002, L2-ERR-102, L3-ERR-102
#[test]
fn canonical_digit_error_names_the_field() {
    // Figure11-13 word0 bits11..8: units of seconds = A; day1.
    let raw = [1, 0, 0, 0, 0, 0x0a, 0, 0, 1, 0];
    assert!(matches!(
        decode_time_f1(
            &raw,
            10,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default()
        ),
        Err(TimeError::InvalidBcdDigit {
            nibble: 10,
            position: "seconds",
            ..
        })
    ));
}
/// Requirements: L1-BCD-004, L2-BCD-104, L3-BCD-104, L1-ERR-003, L2-ERR-103, L3-ERR-103
#[test]
fn valid_decimal_tens_six_is_a_range_error_without_scale_context() {
    // Figure11-13: tens of minutes=6 is a decimal digit, but minute60 is invalid.
    let raw = [0, 0, 0x60, 0, 1, 0];
    assert!(matches!(
        DayFormatTime::from_le_bytes(&raw),
        Err(TimeError::OutOfRange {
            field: "minutes",
            value: 60,
            ..
        })
    ));
}
/// Requirements: L1-CSDW-005, L2-CSDW-105, L3-CSDW-105
#[test]
fn date_bit_alone_selects_the_bounded_body() {
    // Figures11-13/14, printed11-25/PDF33: word2=0x0101 is DOY101
    // or Jan1; appended year word=0x2024, filler must not count as data.
    let mut raw = [1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0x24, 0x20, 0xff, 0xff];
    let r = decode_time_f1(
        &raw,
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    assert_eq!(
        (
            r.consumed,
            r.time.unwrap().year,
            r.time.unwrap().day_of_year
        ),
        (10, None, 101)
    );
    raw[1] = 2;
    assert!(
        decode_time_f1(
            &raw,
            10,
            TimeF1Layout::SourceWithIts17,
            ScalePolicy::default()
        )
        .is_err()
    );
    let r = decode_time_f1(
        &raw,
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    assert_eq!(
        (
            r.consumed,
            r.time.unwrap().year,
            r.time.unwrap().day_of_year
        ),
        (12, Some(2024), 1)
    );
}
