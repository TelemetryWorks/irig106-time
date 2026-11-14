//! S02 hand fixtures, 106-24R1 Figures11-13/14 (11-25/PDF33), not encoders.
use irig106_time::{
    format1::{CalendarReading, Format1Input},
    scale::{ScaleEvidence, TimeScale},
    stream::*,
    timeline::*,
    *,
};
const DOY: [u8; 10] = [1, 0, 0, 0, 0x56, 0x34, 0x12, 1, 0x23, 1]; // day123 01:12:34.560
const DMY: [u8; 12] = [1, 2, 0, 0, 0x56, 0x34, 0x12, 1, 0x15, 3, 0, 0x20]; // 2000 March15=day75
fn input(raw: &[u8], data_length: usize) -> Format1Input<'_> {
    Format1Input {
        payload: raw,
        data_length,
        layout: TimeF1Layout::SourceWithIts17,
        scales: ScalePolicy::default(),
    }
}
fn packet(id: u64) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(100),
        kind: PacketKind::Time,
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference: None,
    }
}
fn stream() -> StreamTimeline {
    StreamTimeline::new(
        StreamPolicy::new(
            1,
            BufferCaps {
                packets: 2,
                bytes: 128,
                references: 2,
            },
        ),
        Some(&[StreamChannel {
            id: 7,
            timein: true,
            external: true,
        }]),
    )
    .unwrap()
}
/// Requirements: L1-BCD-003, L2-BCD-103, L3-BCD-103, L1-CSDW-005, L2-CSDW-105, L3-CSDW-105
#[test]
fn every_zero_bit_is_a_warning_with_exact_operands_and_kept_value() {
    for (raw, masks) in [
        (DOY.as_slice(), [0x8000u16, 0xc080, 0xfc00, 0]),
        (DMY.as_slice(), [0x8000, 0xc080, 0xe000, 0xc000]),
    ] {
        let len = raw.len();
        let baseline = input(raw, len).decode().unwrap().time;
        let mut padded = raw.to_vec();
        padded.extend_from_slice(&[0xff; 8]);
        assert_eq!(
            input(&padded, len).decode().unwrap().reserved_body_bits,
            [0; 4]
        );
        for (word, mask) in masks.into_iter().enumerate() {
            for bit in 0..16 {
                if mask & (1 << bit) == 0 {
                    continue;
                }
                let mut bytes = raw.to_vec();
                bytes.extend_from_slice(&[0xff; 8]);
                bytes[4 + 2 * word + bit / 8] |= 1 << (bit % 8);
                let reading = input(&bytes, len).decode().unwrap();
                assert_eq!(reading.time, baseline);
                let mut expected_masks = [0; 4];
                expected_masks[word] = 1 << bit;
                assert_eq!(reading.reserved_body_bits, expected_masks);
                assert_eq!(reading.consumed, len);
                let mut offline = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
                let key = offline
                    .observe_format1(packet(42), input(&bytes, len), false)
                    .unwrap();
                let timeline = offline.finish().unwrap();
                assert!(timeline.lookup(key).unwrap().time.is_some());
                let batch = stream()
                    .observe_format1(packet(42), input(&bytes, len), 0, 32, false)
                    .unwrap();
                for findings in [timeline.findings(), batch.findings.as_slice()] {
                    let f = findings.iter().find(|f| f.code == 5).unwrap();
                    assert_eq!(f.severity, Severity::Warning);
                    let name = [
                        "format1.word0.reserved",
                        "format1.word1.reserved",
                        "format1.word2.reserved",
                        "format1.word3.reserved",
                    ][word];
                    let field = f
                        .evidence
                        .raw_fields()
                        .iter()
                        .find(|f| f.name == name)
                        .unwrap();
                    assert_eq!(field.observed, findings::RawValue::Unsigned(1 << bit));
                    assert_eq!(field.expected, Some(findings::RawValue::Unsigned(0)));
                }
            }
        }
    }
}
/// Requirements: L1-BCD-004, L2-BCD-104, L3-BCD-104, L1-ERR-002, L2-ERR-102, L3-ERR-102
#[test]
fn every_digit_pattern_has_field_word_digit_and_raw_evidence() {
    let shared = [
        (0, 0, 4, "milliseconds", 1),
        (0, 4, 4, "milliseconds", 2),
        (0, 8, 4, "seconds", 0),
        (0, 12, 3, "seconds", 1),
        (1, 0, 4, "minutes", 0),
        (1, 4, 3, "minutes", 1),
        (1, 8, 4, "hours", 0),
        (1, 12, 2, "hours", 1),
    ];
    for (raw, extra) in [
        (
            DOY.as_slice(),
            vec![
                (2, 0, 4, "day_of_year", 0),
                (2, 4, 4, "day_of_year", 1),
                (2, 8, 2, "day_of_year", 2),
            ],
        ),
        (
            DMY.as_slice(),
            vec![
                (2, 0, 4, "day_of_month", 0),
                (2, 4, 4, "day_of_month", 1),
                (2, 8, 4, "month", 0),
                (2, 12, 1, "month", 1),
                (3, 0, 4, "year", 0),
                (3, 4, 4, "year", 1),
                (3, 8, 4, "year", 2),
                (3, 12, 2, "year", 3),
            ],
        ),
    ] {
        for (word, shift, width, field, digit) in shared.into_iter().chain(extra) {
            for value in 0..1u16 << width {
                let mut bytes = raw.to_vec();
                let offset = 4 + 2 * word;
                let w = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                let changed = (w & !(((1 << width) - 1) << shift)) | (value << shift);
                bytes[offset..offset + 2].copy_from_slice(&changed.to_le_bytes());
                let result = input(&bytes, raw.len()).decode();
                if value > 9 {
                    assert!(
                        matches!(result,Err(TimeError::InvalidBcdDigit{nibble,position,word:w,digit:d,bit_offset,raw_word}) if nibble==value as u8 && position==field && w==word as u8 && d==digit && bit_offset==shift && raw_word==changed),
                        "{field} {word}/{shift}={value}"
                    );
                } else {
                    assert!(
                        !matches!(result, Err(TimeError::InvalidBcdDigit { .. })),
                        "decimal {field}={value}"
                    );
                }
            }
        }
    }
}
/// Requirements: L1-ERR-002, L2-ERR-102, L3-ERR-102, L1-ERR-003, L2-ERR-103, L3-ERR-103
#[test]
fn both_engines_retain_the_original_parse_error_and_packet_context() {
    let mut bytes = DOY;
    bytes[5] = 0x3a; // seconds units=A; complete word0=0x3a56.
    for len in [3, 9, 10] {
        let expected = input(&bytes, len).decode().unwrap_err();
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let key = b
            .observe_format1(packet(99), input(&bytes, len), false)
            .unwrap();
        let t = b.finish().unwrap();
        assert!(t.lookup(key).unwrap().time.is_none());
        let batch = stream()
            .observe_format1(packet(99), input(&bytes, len), 0, 32, false)
            .unwrap();
        for findings in [t.findings(), batch.findings.as_slice()] {
            let f = findings.iter().find(|f| f.code == 4).unwrap();
            assert_eq!(f.evidence.decode_error(), Some(&expected));
            assert_eq!(f.evidence.location().packet, Some(99));
            assert_eq!(f.evidence.location().channel, Some(7));
            assert_eq!(f.evidence.raw_rtc(), Some(Rtc::from_raw(100)));
        }
    }
}
/// Requirements: L1-ABS-001, L2-ABS-101, L3-ABS-101, L1-ERR-003, L2-ERR-103, L3-ERR-103
/// Interpretations: RDG-010
#[test]
fn leap_label_is_distinct_and_arithmetic_refuses_missing_context() {
    // RCC200-16 §3.6/A.2: 23:59:60 is not the following midnight.
    let reading = CalendarReading {
        year: Some(2016),
        day_of_year: 366,
        hours: 23,
        minutes: 59,
        seconds: 60,
        nanoseconds: 0,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    };
    assert!(reading.validate().is_ok());
    assert_eq!(reading.checked_shift(SignedNanos::ZERO).unwrap(), reading);
    for ns in [-1, 1, 1_000_000_000] {
        assert!(matches!(
            reading.checked_shift(SignedNanos::new(ns)),
            Err(TimeError::NeedsContext { .. })
        ));
    }
    let ordinary = CalendarReading {
        seconds: 59,
        ..reading
    };
    assert!(
        ordinary
            .checked_shift(SignedNanos::new(1_000_000_000))
            .is_err()
    );
    assert_eq!(
        ordinary.checked_shift(SignedNanos::new(1)).unwrap().seconds,
        59
    );
    for scale in [TimeScale::Tai, TimeScale::Gps, TimeScale::Unknown] {
        assert!(
            CalendarReading {
                scale: ScaleEvidence::standard(scale),
                ..reading
            }
            .validate()
            .is_err()
        );
    }
}
/// Requirements: L1-ABS-001, L2-ABS-101, L3-ABS-101
#[test]
fn checked_calendar_bounds_never_invent_a_year() {
    let t = AbsoluteTime::new(365, 23, 59, 59, 0).unwrap();
    assert!(matches!(
        t.add_nanos(1_000_000_000),
        Err(TimeError::NeedsContext { .. })
    ));
    assert!(
        AbsoluteTime::new(1, 0, 0, 0, 0)
            .unwrap()
            .sub_nanos(1)
            .is_err()
    );
    for (year, day, ns) in [(0, 1, -1), (9999, 365, 86_400_000_000_000i128)] {
        assert!(
            AbsoluteTime::new(day, 0, 0, 0, 0)
                .unwrap()
                .with_year(Some(year))
                .unwrap()
                .checked_shift(SignedNanos::new(ns))
                .is_err()
        );
    }
    assert!(t.checked_shift(SignedNanos::new(i128::MAX)).is_err());
}

/// Requirements: L1-ERR-002, L2-ERR-102, L3-ERR-102, L1-BCD-003, L2-BCD-103, L3-BCD-103
#[test]
fn malformed_digit_does_not_hide_an_independent_zero_bit_warning() {
    let mut bytes = DOY;
    bytes[5] = 0x3a;
    bytes[7] |= 0x80; // invalid seconds + word1 bit15
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe_format1(packet(88), input(&bytes, 10), false)
        .unwrap();
    let t = b.finish().unwrap();
    let batch = stream()
        .observe_format1(packet(88), input(&bytes, 10), 0, 32, false)
        .unwrap();
    for findings in [t.findings(), batch.findings.as_slice()] {
        assert!(
            findings
                .iter()
                .any(|f| f.code == 4 && f.evidence.decode_error().is_some())
        );
        let warning = findings.iter().find(|f| f.code == 5).unwrap();
        assert_eq!(warning.severity, Severity::Warning);
        assert_eq!(
            warning.evidence.raw_fields()[0].observed,
            findings::RawValue::Unsigned(0x8000)
        );
    }
}
/// Requirements: L1-ERR-003, L2-ERR-103, L3-ERR-103, L1-ABS-001, L2-ABS-101, L3-ABS-101
#[test]
fn gregorian_dates_not_ly_bits_and_field_boundaries_govern_validation() {
    for (year, word, leap) in [
        (1900, 0x1900u16, false),
        (2000, 0x2000, true),
        (2100, 0x2100, false),
    ] {
        for ly in [false, true] {
            let mut bytes = DMY;
            bytes[1] = 2 | u8::from(ly);
            bytes[8] = 0x29;
            bytes[9] = 2;
            bytes[10..12].copy_from_slice(&word.to_le_bytes());
            let r = input(&bytes, 12).decode();
            assert_eq!(r.is_ok(), leap, "{year} LY={ly}");
            if leap {
                let r = r.unwrap();
                assert_eq!(r.time.unwrap().day_of_year, 60);
                assert_eq!(r.csdw.is_leap_year(), ly);
            }
        }
    }
    let base = input(&DOY, 10).decode().unwrap().time.unwrap();
    for bad in [
        CalendarReading {
            day_of_year: 0,
            ..base
        },
        CalendarReading {
            day_of_year: 367,
            ..base
        },
        CalendarReading { hours: 24, ..base },
        CalendarReading {
            minutes: 60,
            ..base
        },
        CalendarReading {
            seconds: 61,
            ..base
        },
        CalendarReading {
            nanoseconds: 1_000_000_000,
            ..base
        },
        CalendarReading {
            year: Some(10000),
            ..base
        },
    ] {
        assert!(bad.validate().is_err());
    }
    let max = CalendarReading {
        year: None,
        day_of_year: 366,
        hours: 23,
        minutes: 59,
        seconds: 59,
        nanoseconds: 999_999_999,
        ..base
    };
    assert!(max.validate().is_ok());
    assert!(max.with_year(1900).is_err());
}
