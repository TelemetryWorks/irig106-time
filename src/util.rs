//! Crate-internal utility functions.
//!
//! # MSRV Constraints
//!
//! This module consolidates functionality that was previously implemented
//! using APIs with high minimum Rust version requirements. By providing
//! crate-internal alternatives, we maintain a lower MSRV.
//!
//! | Function | Replaces | MSRV Impact |
//! |----------|----------|-------------|
//! | [`is_leap_year`] | `u16::is_multiple_of` (Rust 1.87) | Enables MSRV 1.60 |
//!
//! The crate's MSRV is determined by the highest-versioned stable API used:
//!
//! | API | Stabilized In | Used By |
//! |-----|---------------|---------|
//! | `u64::abs_diff` | Rust 1.60 | `LeapSecondTable::is_near_leap_second` |
//! | `u16::saturating_sub` | Rust 1.0 | `AbsoluteTime::sub_nanos`, BCD helpers |
//! | `u64::saturating_add` | Rust 1.0 | `network_time::unix_seconds_to_ymd` |
//! | Edition 2021 | Rust 1.56 | `Cargo.toml` |
//!
//! **Current MSRV: 1.60** (constrained by `u64::abs_diff`)

/// Determine whether a year is a leap year per the Gregorian calendar.
///
/// A year is a leap year if:
/// - It is divisible by 4, AND
/// - It is NOT divisible by 100, UNLESS
/// - It is also divisible by 400
///
/// This function uses modulo arithmetic (`%`) instead of the standard library's
/// `u16::is_multiple_of()` method, which was stabilized in Rust 1.87. This
/// allows the crate to maintain an MSRV of 1.60.
///
/// # MSRV Note
///
/// When the crate's MSRV is raised to 1.87 or higher, this function's
/// internals can be updated to use `is_multiple_of()` without changing
/// callers. The `#[allow(clippy::manual_is_multiple_of)]` annotation
/// suppresses the lint that would otherwise suggest that migration.
///
/// # Examples
///
/// ```
/// # // This function is pub(crate), so we test it via the module's own tests.
/// # // These examples document the expected behavior.
/// // Leap years:
/// // 2024: divisible by 4, not by 100 → leap
/// // 2000: divisible by 400 → leap
/// //
/// // Non-leap years:
/// // 2023: not divisible by 4 → not leap
/// // 1900: divisible by 100 but not 400 → not leap
/// ```
#[allow(clippy::manual_is_multiple_of)]
#[inline]
pub(crate) fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic leap year rules ───────────────────────────────────────

    #[test]
    fn common_leap_years() {
        // Divisible by 4 but not by 100
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2016));
        assert!(is_leap_year(2004));
        assert!(is_leap_year(1996));
    }

    #[test]
    fn common_non_leap_years() {
        // Not divisible by 4
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2022));
        assert!(!is_leap_year(2021));
        assert!(!is_leap_year(2019));
        assert!(!is_leap_year(2025));
    }

    #[test]
    fn century_years_not_leap() {
        // Divisible by 100 but not by 400
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(1800));
        assert!(!is_leap_year(1700));
        assert!(!is_leap_year(2100));
        assert!(!is_leap_year(2200));
        assert!(!is_leap_year(2300));
    }

    #[test]
    fn quad_century_years_are_leap() {
        // Divisible by 400
        assert!(is_leap_year(2000));
        assert!(is_leap_year(1600));
        assert!(is_leap_year(2400));
    }

    // ── Edge cases ──────────────────────────────────────────────────

    #[test]
    fn year_zero() {
        // Year 0 is divisible by 400
        assert!(is_leap_year(0));
    }

    #[test]
    fn year_one() {
        assert!(!is_leap_year(1));
    }

    #[test]
    fn max_u16_year() {
        // u16::MAX = 65535, not divisible by 4
        assert!(!is_leap_year(u16::MAX));
    }

    // ── Telemetry-relevant years ────────────────────────────────────

    #[test]
    fn irig106_era_years() {
        // Years commonly seen in IRIG 106 flight test data
        assert!(is_leap_year(2004)); // 106-04 era
        assert!(!is_leap_year(2005)); // 106-05 era
        assert!(!is_leap_year(2007)); // 106-07 era
        assert!(!is_leap_year(2017)); // 106-17 (Ch10/Ch11 split)
        assert!(!is_leap_year(2022)); // 106-22 (Network Time)
        assert!(!is_leap_year(2023)); // 106-23
        assert!(is_leap_year(2024)); // Current recordings
    }
}
