//! Main-first regression for the withdrawn prototype day split (T-10, Q-CH4).
use irig106_time::{Ch4BinaryTime, Ch4BinaryTimeExt, TimeError};

// Partial ABS-102 / WORD-101 clause evidence; no whole-requirement marker.
#[test]
fn legacy_ch4_conversion_never_invents_day_one_or_a_day_from_binary_bits() {
    // Figure11-4, 11-10/PDF18; full count0 and0x00020000 are elapsed counts,
    // not day0 repaired to1 and day1 respectively. Neither supplies an origin.
    for (high_order, low_order) in [(0, 0), (2, 0), (0xffff, 0xffff)] {
        let t = Ch4BinaryTime {
            high_order,
            low_order,
            microseconds: 0,
        };
        assert!(matches!(
            t.to_absolute(),
            Err(TimeError::NeedsContext { .. })
        ));
    }
}
