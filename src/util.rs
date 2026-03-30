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
//! | [`is_leap_year`] | `u16::is_multiple_of` (Rust 1.87) | Avoids MSRV creep to 1.87 |
//! | [`abs_diff_u64`] | `u64::abs_diff` (Rust 1.60) | Avoids clippy `manual_abs_diff` lint |
//! | [`days_in_month`] | (shared helper) | Used by `CalendarTime` and `bcd.rs` |
//! | [`month_day_to_doy`] | (shared helper) | Used by `CalendarTime` and `bcd.rs` |
//!
//! The crate's MSRV is determined by the highest-versioned stable API or
//! Cargo feature used:
//!
//! | API / Feature | Stabilized In | Used By | Status |
//! |---------------|---------------|---------|--------|
//! | `dep:` namespaced features | Rust 1.60 | `Cargo.toml` `[features]` | **MSRV constraint** |
//! | `u64::abs_diff` | Rust 1.60 | `LeapSecondTable::is_near_leap_second` | Replaced by `abs_diff_u64` |
//! | `u16::is_multiple_of` | Rust 1.87 | Leap year checks (3 files) | Replaced by `is_leap_year` |
//! | `u16::saturating_sub` | Rust 1.0 | `AbsoluteTime::sub_nanos`, BCD helpers | No concern |
//! | `u64::saturating_add` | Rust 1.0 | `network_time::unix_seconds_to_ymd` | No concern |
//! | Edition 2021 | Rust 1.56 | `Cargo.toml` | Below MSRV floor |
//!
//! **Current MSRV: 1.60** (constrained by `dep:` namespaced features in `Cargo.toml`)

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

/// Compute the absolute difference between two `u64` values.
///
/// Returns `|a - b|` without risk of underflow. This replaces the standard
/// library's `u64::abs_diff()` method, which was stabilized in Rust 1.60.
/// Using this helper allows the crate to avoid the `clippy::manual_abs_diff`
/// lint without raising the MSRV beyond 1.60.
///
/// # MSRV Note
///
/// When the crate's MSRV is raised to 1.60 or higher, this function's
/// internals can be updated to use `u64::abs_diff()` without changing
/// callers. The `#[allow(clippy::manual_abs_diff)]` annotation suppresses
/// the lint that would otherwise suggest that migration.
///
/// # Examples
///
/// ```
/// # // This function is pub(crate), so we test it via the module's own tests.
/// # // These examples document the expected behavior.
/// // abs_diff_u64(10, 3) == 7
/// // abs_diff_u64(3, 10) == 7
/// // abs_diff_u64(5, 5) == 0
/// // abs_diff_u64(0, u64::MAX) == u64::MAX
/// ```
#[allow(clippy::manual_abs_diff)]
#[inline]
pub(crate) fn abs_diff_u64(a: u64, b: u64) -> u64 {
    if a >= b {
        a - b
    } else {
        b - a
    }
}

/// Returns the number of days in a given month (1-indexed), accounting for
/// leap years.
///
/// Used by `CalendarTime::new` for day-of-month validation and by `bcd.rs`
/// for DMY parsing.
#[inline]
pub(crate) fn days_in_month(year: u16, month: u8) -> u8 {
    const DAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let m = (month as usize).saturating_sub(1).min(11);
    if m == 1 && is_leap_year(year) {
        29
    } else {
        DAYS[m]
    }
}

/// Convert a calendar date (year, month 1–12, day 1–31) to day-of-year (1–366).
///
/// Used by `CalendarTime::new` for DOY consistency validation and by `bcd.rs`
/// for DMY-to-DOY conversion.
#[inline]
pub(crate) fn month_day_to_doy(year: u16, month: u8, day: u8) -> u16 {
    let days_before: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let m = (month as usize).saturating_sub(1).min(11);
    let mut doy = days_before[m] + day as u16;
    if is_leap_year(year) && month > 2 {
        doy += 1;
    }
    doy
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

    // ═══════════════════════════════════════════════════════════════════
    // abs_diff_u64 tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn abs_diff_a_greater() {
        assert_eq!(abs_diff_u64(10, 3), 7);
        assert_eq!(abs_diff_u64(1_000_000, 1), 999_999);
    }

    #[test]
    fn abs_diff_b_greater() {
        assert_eq!(abs_diff_u64(3, 10), 7);
        assert_eq!(abs_diff_u64(1, 1_000_000), 999_999);
    }

    #[test]
    fn abs_diff_equal() {
        assert_eq!(abs_diff_u64(0, 0), 0);
        assert_eq!(abs_diff_u64(42, 42), 0);
        assert_eq!(abs_diff_u64(u64::MAX, u64::MAX), 0);
    }

    #[test]
    fn abs_diff_extremes() {
        assert_eq!(abs_diff_u64(0, u64::MAX), u64::MAX);
        assert_eq!(abs_diff_u64(u64::MAX, 0), u64::MAX);
    }

    #[test]
    fn abs_diff_commutative() {
        // abs_diff(a, b) == abs_diff(b, a) for all inputs
        let pairs: [(u64, u64); 5] = [
            (0, 1),
            (100, 200),
            (1_483_228_800, 1_600_000_000), // Unix timestamps near leap seconds
            (u64::MAX - 1, u64::MAX),
            (0, u64::MAX),
        ];
        for (a, b) in pairs {
            assert_eq!(abs_diff_u64(a, b), abs_diff_u64(b, a));
        }
    }

    #[test]
    fn abs_diff_leap_second_timestamps() {
        // Real-world use case: distance from a leap second boundary
        let leap_2017 = 1_483_228_800u64; // 2017-01-01 leap second
        assert_eq!(abs_diff_u64(leap_2017, leap_2017 + 10), 10);
        assert_eq!(abs_diff_u64(leap_2017, leap_2017 - 10), 10);
        assert_eq!(abs_diff_u64(leap_2017 + 5, leap_2017 - 5), 10);
    }

    // ═══════════════════════════════════════════════════════════════════
    // days_in_month tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn days_in_month_standard() {
        assert_eq!(days_in_month(2025, 1), 31);
        assert_eq!(days_in_month(2025, 2), 28);
        assert_eq!(days_in_month(2025, 3), 31);
        assert_eq!(days_in_month(2025, 4), 30);
        assert_eq!(days_in_month(2025, 5), 31);
        assert_eq!(days_in_month(2025, 6), 30);
        assert_eq!(days_in_month(2025, 7), 31);
        assert_eq!(days_in_month(2025, 8), 31);
        assert_eq!(days_in_month(2025, 9), 30);
        assert_eq!(days_in_month(2025, 10), 31);
        assert_eq!(days_in_month(2025, 11), 30);
        assert_eq!(days_in_month(2025, 12), 31);
    }

    #[test]
    fn days_in_month_leap_feb() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2000, 2), 29);
    }

    #[test]
    fn days_in_month_century_non_leap_feb() {
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2100, 2), 28);
    }

    // ═══════════════════════════════════════════════════════════════════
    // month_day_to_doy tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn month_day_to_doy_jan_1() {
        assert_eq!(month_day_to_doy(2025, 1, 1), 1);
    }

    #[test]
    fn month_day_to_doy_dec_31_non_leap() {
        assert_eq!(month_day_to_doy(2025, 12, 31), 365);
    }

    #[test]
    fn month_day_to_doy_dec_31_leap() {
        assert_eq!(month_day_to_doy(2024, 12, 31), 366);
    }

    #[test]
    fn month_day_to_doy_mar_1_leap() {
        // Jan(31) + Feb(29) + 1 = 61
        assert_eq!(month_day_to_doy(2024, 3, 1), 61);
    }

    #[test]
    fn month_day_to_doy_mar_1_non_leap() {
        // Jan(31) + Feb(28) + 1 = 60
        assert_eq!(month_day_to_doy(2025, 3, 1), 60);
    }
}
