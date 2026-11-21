//! Main-first regression for the withdrawn prototype day split (T-10, Q-CH4).
use irig106_time::{Ch4BinaryTime, TimeError};

// Partial ABS-102 / WORD-101 clause evidence; no whole-requirement marker.
#[test]
fn explicit_ch4_partial_never_invents_day_one_or_a_day_from_binary_bits() {
    // Figure11-4, 11-10/PDF18; full count0 and0x00020000 are elapsed counts,
    // not day0 repaired to1 and day1 respectively. Neither supplies an origin.
    for (high_order, low_order) in [(0, 0), (2, 0), (0xffff, 0xffff)] {
        let t = Ch4BinaryTime {
            high_order,
            low_order,
            microseconds: 0,
        };
        let reading = irig106_time::ch4::Ch4Words::from(t).binary(None).unwrap();
        assert!(reading.calendar().is_none());
        assert!(matches!(
            reading.value(),
            irig106_time::ch4::Ch4Value::BinaryElapsed(_)
        ));
    }
}

// Partial ABS-102 validation-order evidence; no whole-requirement marker.
#[test]
fn explicit_ch4_partial_rejects_invalid_microseconds_before_missing_context() {
    // Ch4 §4.7, printed4-11/PDF15: maximum9999 decimal;10000 is invalid
    // even without a calendar origin (Fig11-4 usec word,11-10/PDF18).
    let t = Ch4BinaryTime {
        high_order: 0,
        low_order: 0,
        microseconds: 10_000,
    };
    assert!(matches!(
        irig106_time::ch4::Ch4Words::from(t).binary(None),
        Err(TimeError::OutOfRange {
            field: "ch4_microseconds",
            value: 10_000,
            max: 9999,
        })
    ));
}
