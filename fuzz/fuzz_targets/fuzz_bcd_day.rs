//! Fuzz target: BCD Day-of-Year format parsing.
//!
//! Feeds arbitrary bytes into `DayFormatTime::from_le_bytes` to verify:
//! - No panics on any input
//! - All errors are typed `TimeError` variants
//! - Valid parses produce in-range field values
//!
//! # Run
//! ```sh
//! cargo +nightly fuzz run fuzz_bcd_day
//! ```

#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::bcd::DayFormatTime;

fuzz_target!(|data: &[u8]| {
    // Must not panic regardless of input.
    match DayFormatTime::from_le_bytes(data) {
        Ok(t) => {
            // If parsing succeeded, all fields must be in valid ranges.
            assert!(t.day_of_year >= 1 && t.day_of_year <= 366);
            assert!(t.hours <= 23);
            assert!(t.minutes <= 59);
            assert!(t.seconds <= 59);
            assert!(t.milliseconds <= 990);
            // Millisecond resolution is 10ms (Hmn*100 + Tmn*10)
            assert!(t.milliseconds % 10 == 0);

            // to_absolute must also not panic
            let abs = t.to_absolute();
            assert_eq!(abs.day_of_year, t.day_of_year);
            assert!(abs.nanoseconds <= 999_999_999);
        }
        Err(_) => {
            // All error paths are fine — the point is no panics.
        }
    }
});
