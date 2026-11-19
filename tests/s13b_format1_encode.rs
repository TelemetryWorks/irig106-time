//! Independent106-24R1 §11.2.3.2a/b, Figures11-12..14 (11-23/25, PDF31/33).
use TimeF1Layout::{Legacy04, Source05, SourceWithIts17};
use irig106_time::{
    TimeError, TimeF1Csdw,
    bcd::{DayFormatTime, DmyFormatTime},
    csdw::TimeF1Layout,
    format1::{CalendarReading, encode_format1},
    scale::{ScaleEvidence, TimeScale},
};
fn time(year: Option<u16>, day: u16) -> CalendarReading {
    CalendarReading {
        year,
        day_of_year: day,
        hours: 12,
        minutes: 30,
        seconds: 25,
        nanoseconds: 340_000_000,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    }
}

#[test]
fn dmy_always_zero_year_bits_are_not_emitted() {
    // Fig11-14: year3999 =>0x3999; bits15..14 MUST be zero.
    let t = DmyFormatTime::new(3999, 12, 31, 23, 59, 59, 990).unwrap();
    assert_eq!(
        t.to_le_bytes().unwrap(),
        [0x99, 0x59, 0x59, 0x23, 0x31, 0x12, 0x99, 0x39]
    );
    // Existing writer must not set the reserved high-year position for4000.
    let t = DmyFormatTime::new(4000, 1, 1, 0, 0, 0, 0).unwrap();
    assert!(matches!(
        t.to_le_bytes(),
        Err(TimeError::OutOfRange {
            field: "bcd_year",
            ..
        })
    ));
}
#[test]
fn literal_bcd_slots_do_not_depend_on_a_decoder() {
    // §11.2.3.2b/Fig11-13: .340s=>Tmn4/Hmn3;25s,12h30m,DOY100.
    assert_eq!(
        DayFormatTime::new(100, 12, 30, 25, 340)
            .unwrap()
            .to_le_bytes()
            .unwrap(),
        [0x34, 0x25, 0x30, 0x12, 0, 1]
    );
    // Fig11-14: Apr10=>0x0410; year2025=>0x2025.
    assert_eq!(
        DmyFormatTime::new(2025, 4, 10, 12, 30, 25, 340)
            .unwrap()
            .to_le_bytes()
            .unwrap(),
        [0x34, 0x25, 0x30, 0x12, 0x10, 4, 0x25, 0x20]
    );
    assert_eq!(TimeF1Csdw::from_raw(0x241).as_raw(), 0x241);
}

/// Requirements: L1-ENC-001, L2-ENC-101, L3-ENC-101
/// Interpretations: RDG-010
#[test]
fn complete_format1_encoder_contract() {
    dmy_always_zero_year_bits_are_not_emitted();
    literal_bcd_slots_do_not_depend_on_a_decoder();
    // §11.2.3.2a/Fig11-12: DATE1/FMT4/SRC1; b/Fig11-14 body.
    // Contract7 numeric example, transcribed through Fig11-13 digit slots.
    let example = CalendarReading {
        year: None,
        day_of_year: 187,
        hours: 13,
        minutes: 45,
        seconds: 27,
        nanoseconds: 350_000_000,
        ..time(None, 187)
    };
    assert_eq!(
        encode_format1(TimeF1Csdw::from_raw(0x3001), SourceWithIts17, Some(example))
            .unwrap()
            .as_bytes(),
        [1, 0x30, 0, 0, 0x35, 0x27, 0x45, 0x13, 0x87, 1]
    );
    let t = time(Some(2025), 100);
    let out = encode_format1(TimeF1Csdw::from_raw(0x241), SourceWithIts17, Some(t)).unwrap();
    assert_eq!(
        out.as_bytes(),
        [0x41, 2, 0, 0, 0x34, 0x25, 0x30, 0x12, 0x10, 4, 0x25, 0x20]
    );
    // Same numeric fields in the DOY form (Fig11-13); no implicit date/year.
    assert_eq!(
        encode_format1(TimeF1Csdw::from_raw(0x41), SourceWithIts17, Some(t))
            .unwrap()
            .as_bytes(),
        [0x41, 0, 0, 0, 0x34, 0x25, 0x30, 0x12, 0, 1]
    );
    for bit in 10..32 {
        assert_eq!(
            TimeF1Csdw::from_raw(0x241 | (1 << bit))
                .to_le_bytes(SourceWithIts17)
                .unwrap(),
            [0x41, 2, 0, 0]
        );
    }
    // Applicable ITS3 at15..12; LY1 at8; IRIG-B FMT0 and SRC1.
    assert_eq!(
        TimeF1Csdw::from_raw(0xffff_3d01)
            .to_le_bytes(SourceWithIts17)
            .unwrap(),
        [1, 0x31, 0, 0]
    );
    assert_eq!(
        TimeF1Csdw::from_raw(0xffff_3d01)
            .to_le_bytes(Source05)
            .unwrap(),
        [1, 1, 0, 0]
    );
    assert_eq!(
        TimeF1Csdw::from_raw(0xffff_fc4f)
            .to_le_bytes(Legacy04)
            .unwrap(),
        [0x41, 0, 0, 0]
    );
    for fmt in 0..16u32 {
        let w = TimeF1Csdw::from_raw((fmt << 4) | 1);
        assert_eq!(
            w.to_le_bytes(SourceWithIts17).is_ok(),
            matches!(fmt, 0..=5 | 15)
        );
        assert_eq!(w.to_le_bytes(Legacy04).is_ok(), fmt <= 5);
    }
    for src in 3..15 {
        assert!(
            TimeF1Csdw::from_raw(0x40 | src)
                .to_le_bytes(Source05)
                .is_err()
        );
    }
    assert_eq!(
        TimeF1Csdw::from_raw(0x4f).to_le_bytes(Source05).unwrap(),
        [0x4f, 0, 0, 0]
    );
    for its in 0..16u32 {
        assert_eq!(
            TimeF1Csdw::from_raw((its << 12) | 1)
                .to_le_bytes(SourceWithIts17)
                .is_ok(),
            its < 8
        );
    }
    for (raw, byte) in [(0x40, 0x40), (0x41, 0x41), (0x42, 0x42), (0x4f, 0x4f)] {
        assert_eq!(
            TimeF1Csdw::from_raw(raw).to_le_bytes(Source05).unwrap(),
            [byte, 0, 0, 0]
        );
    }
    for (fmt, byte) in [(0, 1), (1, 0x11), (2, 0x21)] {
        assert_eq!(
            TimeF1Csdw::from_raw(0x3001 | (fmt << 4))
                .to_le_bytes(SourceWithIts17)
                .unwrap(),
            [byte, 0x30, 0, 0]
        );
    }
    for its in 0..8u32 {
        assert_eq!(
            TimeF1Csdw::from_raw((its << 12) | 1)
                .to_le_bytes(SourceWithIts17)
                .unwrap(),
            [1, (its << 4) as u8, 0, 0]
        );
    }
    // NONE is explicit payload-invalid, not a zero-time value. Canonical invalid
    // body is zero-filled under its DATE shape; supplying a time is rejected.
    assert_eq!(
        encode_format1(TimeF1Csdw::from_raw(0xf0), Source05, None)
            .unwrap()
            .as_bytes(),
        [0xf0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_format1(TimeF1Csdw::from_raw(0x2ff), Source05, None)
            .unwrap()
            .as_bytes(),
        [0xff, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert!(encode_format1(TimeF1Csdw::from_raw(0xf0), Source05, Some(t)).is_err());
    assert!(encode_format1(TimeF1Csdw::from_raw(0x41), Source05, None).is_err());
    for ms in [1, 9, 11, 999] {
        assert!(matches!(
            DayFormatTime::new(1, 0, 0, 0, ms).unwrap().to_le_bytes(),
            Err(TimeError::UnrepresentablePrecision { quantum: 10, .. })
        ));
        assert!(matches!(
            DmyFormatTime::new(2025, 1, 1, 0, 0, 0, ms)
                .unwrap()
                .to_le_bytes(),
            Err(TimeError::UnrepresentablePrecision { quantum: 10, .. })
        ));
    }
    let mut x = t;
    x.nanoseconds = 1;
    assert!(matches!(
        encode_format1(TimeF1Csdw::from_raw(0x241), Source05, Some(x)),
        Err(TimeError::UnrepresentablePrecision {
            quantum: 10_000_000,
            ..
        })
    ));
    for (year, day, word) in [
        (2025, 366, 0x241),
        (2025, 0, 0x241),
        (4000, 1, 0x341),
        (10_000, 1, 0x341),
    ] {
        assert!(
            encode_format1(
                TimeF1Csdw::from_raw(word),
                Source05,
                Some(time(Some(year), day))
            )
            .is_err()
        );
    }
    assert!(encode_format1(TimeF1Csdw::from_raw(0x341), Source05, Some(t)).is_err()); // LY/year conflict
    assert!(encode_format1(TimeF1Csdw::from_raw(0x241), Source05, Some(time(None, 100))).is_err());
    assert!(encode_format1(TimeF1Csdw::from_raw(0x41), Source05, Some(time(None, 366))).is_err());
    // Fig11-13/14 seconds60 fits TSn6, Sn0 only on an RDG-010 eligible date.
    let mut leap = time(Some(2016), 366);
    leap.hours = 23;
    leap.minutes = 59;
    leap.seconds = 60;
    leap.nanoseconds = 0;
    assert_eq!(
        encode_format1(TimeF1Csdw::from_raw(0x141), SourceWithIts17, Some(leap))
            .unwrap()
            .as_bytes(),
        [0x41, 1, 0, 0, 0, 0x60, 0x59, 0x23, 0x66, 3]
    );
    assert_eq!(
        encode_format1(TimeF1Csdw::from_raw(0x341), SourceWithIts17, Some(leap))
            .unwrap()
            .as_bytes(),
        [0x41, 3, 0, 0, 0, 0x60, 0x59, 0x23, 0x31, 0x12, 0x16, 0x20]
    );
    let mut partial = leap;
    partial.year = None;
    partial.day_of_year = 182;
    assert!(encode_format1(TimeF1Csdw::from_raw(0x41), Source05, Some(partial)).is_err());
    assert!(encode_format1(TimeF1Csdw::from_raw(0x141), Source05, Some(partial)).is_ok());
    partial.day_of_year = 181;
    assert!(encode_format1(TimeF1Csdw::from_raw(0x141), Source05, Some(partial)).is_err());
    assert!(encode_format1(TimeF1Csdw::from_raw(0x41), Source05, Some(partial)).is_ok());
    leap.day_of_year = 365;
    assert!(encode_format1(TimeF1Csdw::from_raw(0x141), Source05, Some(leap)).is_err());
    assert_eq!(
        encode_format1(
            TimeF1Csdw::from_raw(0x341),
            Source05,
            Some(time(Some(2024), 60))
        )
        .unwrap()
        .as_bytes(),
        [0x41, 3, 0, 0, 0x34, 0x25, 0x30, 0x12, 0x29, 2, 0x24, 0x20]
    );
    assert!(encode_format1(TimeF1Csdw::from_raw(0x251), Source05, Some(t)).is_err());
    let mut gps = t;
    gps.scale = ScaleEvidence::standard(TimeScale::Gps);
    assert!(encode_format1(TimeF1Csdw::from_raw(0x241), Source05, Some(gps)).is_err());
    assert_eq!(
        &encode_format1(TimeF1Csdw::from_raw(0x251), Source05, Some(gps))
            .unwrap()
            .as_bytes()[..4],
        [0x51, 2, 0, 0]
    );
    // Maximum field digits exercise every always-zero body position.
    let body = DmyFormatTime::new(3999, 12, 31, 23, 59, 59, 990)
        .unwrap()
        .to_le_bytes()
        .unwrap();
    for (i, mask) in [(0, 0x8000), (1, 0xc080), (2, 0xe000), (3, 0xc000)] {
        assert_eq!(u16::from_le_bytes([body[2 * i], body[2 * i + 1]]) & mask, 0);
    }
    let body = DayFormatTime::new(366, 23, 59, 59, 990)
        .unwrap()
        .to_le_bytes()
        .unwrap();
    for (i, mask) in [(0, 0x8000), (1, 0xc080), (2, 0xfc00)] {
        assert_eq!(u16::from_le_bytes([body[2 * i], body[2 * i + 1]]) & mask, 0);
    }
}

#[test]
fn existing_ieee_word_does_not_repair_invalid_packet_flags() {
    use irig106_time::{
        Ieee1588Time,
        intra_packet::{IntraPacketTimeFormat, parse_intra_packet_time},
        own_time::encode_ieee_time,
    };
    // §11.2.1.2/Fig11-5, printed11-10/PDF18: nanos first, seconds second.
    let raw = encode_ieee_time(Ieee1588Time {
        seconds: 0x05060708,
        nanoseconds: 0x01020304,
    })
    .unwrap();
    assert_eq!(raw, [4, 3, 2, 1, 8, 7, 6, 5]);
    for flags in [0x40, 0x4c, 0xcc] {
        assert!(
            parse_intra_packet_time(&raw, IntraPacketTimeFormat::from_packet_flags(flags)).is_err()
        );
    }
    assert!(
        encode_ieee_time(Ieee1588Time {
            seconds: 0,
            nanoseconds: 1_000_000_000
        })
        .is_err()
    );
}
