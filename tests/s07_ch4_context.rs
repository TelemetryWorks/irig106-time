//! S07 clause evidence, not whole-requirement markers: ABS-102 / WORD-101.
//! Sources: 106-24R1 Ch4 §4.7, Fig4-4 printed4-10/11 PDF14/15;
//! Ch11 Fig11-4 printed11-10/PDF18; T-10 prior page checks; Q-CH4 owner2026-11-12.
use irig106_time::{AbsoluteTime, Ch4BinaryTime, TimeError, ch4::*, edition::PacketEdition};

fn origin(year: u16, day: u16, h: u8, m: u8, s: u8, ns: u32) -> AbsoluteTime {
    AbsoluteTime::new(day, h, m, s, ns)
        .unwrap()
        .with_year(Some(year))
        .unwrap()
}
fn words(high: u16, low: u16, usec: u16) -> Ch4Words {
    Ch4Words {
        high,
        low,
        microseconds: usec,
    }
}
fn bcd_context() -> BcdContext {
    BcdContext {
        year: Some(2024),
        full_day: Some(131),
        interpretation: Some(BcdDayInterpretation::LowThreePackedBcdBits),
        edition: Some(PacketEdition::V24R1),
    }
}

#[test]
fn binary_counts_are_full_width_elapsed_values_without_an_origin() {
    // high*655.36s + low*.01s + usec*.000001s; no binary day field.
    for (high, low, usec, ns) in [
        (0, 0, 0, 0),
        (0, 0x8000, 0, 327_680_000_000),
        (1, 0x1234, 9999, 701_969_999_000),
        (0xffff, 0xffff, 9999, 42_949_672_959_999_000),
    ] {
        let w = words(high, low, usec);
        let r = w.binary(None).unwrap();
        assert_eq!(r.raw(), w);
        assert_eq!(
            r.value(),
            Ch4Value::BinaryElapsed(irig106_time::NanosDuration::new(ns))
        );
        assert_eq!(r.context(), Ch4Context::Binary { origin: None });
        assert_eq!(r.calendar(), None);
        assert!(r.missing().origin);
        assert_eq!(r.bcd_wording(), None);
    }
}

#[test]
fn shared_secondary_and_stamp_layouts_feed_the_context_api_without_reordering() {
    // Figure11-4 literal reserved/usec/low/high. Reserved retained by caller;
    // existing validate_ch4_reserved reports it separately as TF-005.
    let bytes = [0x55, 0xaa, 0x0f, 0x27, 0x34, 0x12, 0x01, 0x00];
    let a = Ch4BinaryTime::from_secondary_bytes(&bytes).unwrap();
    let b = Ch4BinaryTime::from_intra_packet_bytes(&bytes).unwrap();
    assert_eq!(Ch4Words::from(a), words(1, 0x1234, 9999));
    assert_eq!(
        Ch4Words::from(b).binary(None).unwrap(),
        Ch4Words::from(a).binary(None).unwrap()
    );
    assert!(irig106_time::secondary::validate_ch4_reserved(&bytes).is_err());
}

#[test]
fn explicit_binary_origin_rolls_over_year_without_discarding_subseconds() {
    let o = origin(2023, 365, 23, 59, 59, 900_000_000);
    let r = words(0, 20, 3).binary(Some(o)).unwrap(); // .200003s
    assert_eq!(r.calendar(), Some(origin(2024, 1, 0, 0, 0, 100_003_000)));
    assert_eq!(r.context(), Ch4Context::Binary { origin: Some(o) });
    assert_eq!(r.missing(), MissingCh4Context::default());
    let leap = words(0, 100, 0)
        .binary(Some(origin(2024, 366, 23, 59, 59, 0)))
        .unwrap();
    assert_eq!(leap.calendar(), Some(origin(2025, 1, 0, 0, 0, 0)));
    assert!(
        words(0, 100, 0)
            .binary(Some(origin(9999, 365, 23, 59, 59, 0)))
            .is_err()
    );
}

#[test]
fn an_undated_binary_origin_is_still_partial() {
    let o = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    let r = words(0, 1, 0).binary(Some(o)).unwrap();
    assert_eq!(r.calendar(), None);
    assert!(!r.missing().origin);
    assert!(r.missing().year);
    assert_eq!(r.context(), Ch4Context::Binary { origin: Some(o) });
}

#[test]
fn bcd_fields_require_all_three_context_items_and_keep_raw_partial_values() {
    // Fig4-4 high=1<<13 |2<<11 |3<<7 |4<<4 |5 =0x31c5:
    // day fragment1,23h45m. Low=5<<12|6<<8|7<<4|8=0x5678 =>56.78s.
    // Binary usec1234 adds.001234s =>56.781234s.
    for mask in 0..8 {
        let mut c = bcd_context();
        if mask & 1 == 0 {
            c.year = None;
        }
        if mask & 2 == 0 {
            c.full_day = None;
        }
        if mask & 4 == 0 {
            c.interpretation = None;
        }
        let w = words(0x31c5, 0x5678, 1234);
        let r = w.bcd(c).unwrap();
        assert_eq!(r.raw(), w);
        assert_eq!(r.context(), Ch4Context::Bcd(c));
        assert_eq!(
            r.value(),
            Ch4Value::BcdTimeOfDay {
                day_fragment: 1,
                hours: 23,
                minutes: 45,
                seconds: 56,
                nanoseconds: 781_234_000
            }
        );
        assert_eq!(
            r.missing(),
            MissingCh4Context {
                origin: false,
                year: mask & 1 == 0,
                full_day: mask & 2 == 0,
                interpretation: mask & 4 == 0
            }
        );
        assert_eq!(
            r.calendar(),
            if mask == 7 {
                Some(origin(2024, 131, 23, 45, 56, 781_234_000))
            } else {
                None
            }
        );
        assert_eq!(r.bcd_wording(), Some(BcdDayWording::Bits));
        assert!(r.bcd_wording_unresolved());
    }
}

#[test]
fn bcd_fragment_comparison_uses_only_explicit_interpretation_and_never_supplies_a_day() {
    let w = words(0x31c5, 0x5678, 1234);
    // 131's packed BCD ends in1, low3bits1 (not binary131&7=3).
    assert!(w.bcd(bcd_context()).unwrap().calendar().is_some());
    let mut c = bcd_context();
    c.full_day = Some(132);
    assert!(matches!(w.bcd(c), Err(TimeError::InvalidTimelineInput(_))));
    c.interpretation = None;
    assert!(w.bcd(c).unwrap().calendar().is_none()); // no silent interpretation
    // Day131 and139 collide in three packed BCD bits; caller full day wins.
    c = bcd_context();
    c.full_day = Some(139);
    assert_eq!(w.bcd(c).unwrap().calendar().unwrap().day_of_year(), 139);
    c.full_day = None;
    assert!(w.bcd(c).unwrap().missing().full_day);
}

#[test]
fn bcd_historical_wording_stays_unresolved_even_with_complete_context() {
    use PacketEdition::*;
    for e in [
        V04, V05, V07, V09, V11, V13, V15, V17, V19, V20, V22, V23, V24, V24R1,
    ] {
        let mut c = bcd_context();
        c.edition = Some(e);
        let r = words(0x31c5, 0x5678, 0).bcd(c).unwrap();
        assert!(r.calendar().is_some());
        assert!(r.bcd_wording_unresolved());
        let older = matches!(e, V04 | V05 | V07 | V09 | V11 | V13 | V15);
        assert_eq!(
            r.bcd_wording(),
            Some(if older {
                BcdDayWording::Characters
            } else {
                BcdDayWording::Bits
            })
        );
    }
    let mut c = bcd_context();
    c.edition = None;
    let r = words(0x31c5, 0x5678, 0).bcd(c).unwrap();
    assert_eq!(r.bcd_wording(), Some(BcdDayWording::EditionNotDeclared));
    assert!(r.bcd_wording_unresolved());
}

#[test]
fn bcd_digits_and_total_ranges_are_independently_validated() {
    // Each changed bit group is independently derived from Fig4-4.
    for (high, low) in [
        (0x0500, 0),
        (0x000a, 0),
        (0, 0x0a00),
        (0, 0x00a0),
        (0, 0x000a),
    ] {
        assert!(matches!(
            words(high, low, 0).bcd(BcdContext::default()),
            Err(TimeError::InvalidBcdDigit { .. })
        ));
    }
    // Decimal digits remain valid:24h,60m/70m,60s/70s are range errors.
    for (high, low) in [
        (0x1200, 0),
        (0x0060, 0),
        (0x0070, 0),
        (0, 0x6000),
        (0, 0x7000),
    ] {
        assert!(matches!(
            words(high, low, 0).bcd(BcdContext::default()),
            Err(TimeError::OutOfRange { .. })
        ));
    }
    for day in [0, 367] {
        let mut c = bcd_context();
        c.full_day = Some(day);
        assert!(words(0, 0, 0).bcd(c).is_err());
    }
    let mut c = bcd_context();
    c.year = Some(2023);
    c.full_day = Some(366);
    assert!(words(0xc000, 0, 0).bcd(c).is_err());
    c.year = Some(10000);
    assert!(words(0, 0, 0).bcd(c).is_err());
}

#[test]
fn reserved_bcd_zero_is_separate_from_usable_time_and_binary_top_bit() {
    let c = bcd_context();
    let a = words(0x31c5, 0x5678, 9999).bcd(c).unwrap();
    let b = words(0x31c5, 0xd678, 9999).bcd(c).unwrap();
    assert_eq!(a.reserved_bits(), 0);
    assert_eq!(b.reserved_bits(), 0x8000);
    assert_eq!(a.calendar(), b.calendar());
    assert_eq!(b.raw().low, 0xd678);
    assert_eq!(a.calendar().unwrap().nanoseconds(), 789_999_000);
    assert_eq!(words(0, 0x8000, 0).binary(None).unwrap().reserved_bits(), 0);
    for u in [10000, 0xffff] {
        assert!(matches!(
            words(0, 0, u).binary(None),
            Err(TimeError::OutOfRange {
                field: "ch4_microseconds",
                ..
            })
        ));
        assert!(matches!(
            words(0, 0, u).bcd(BcdContext::default()),
            Err(TimeError::OutOfRange {
                field: "ch4_microseconds",
                ..
            })
        ));
    }
}
