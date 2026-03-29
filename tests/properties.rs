//! Property-based tests for `irig106-time` using `proptest`.
//!
//! These tests verify invariants that must hold for ALL valid inputs,
//! not just hand-picked examples.
//!
//! # Run
//! ```sh
//! cargo test --test properties
//! ```

use irig106_time::*;
use proptest::prelude::*;

// ── RTC properties ───────────────────────────────────────────────────

proptest! {
    #[test]
    fn rtc_from_raw_always_48bit(value: u64) {
        let rtc = Rtc::from_raw(value);
        prop_assert!(rtc.as_raw() <= 0x0000_FFFF_FFFF_FFFF);
    }

    #[test]
    fn rtc_round_trip_le_bytes(
        b0: u8, b1: u8, b2: u8, b3: u8, b4: u8, b5: u8
    ) {
        let bytes = [b0, b1, b2, b3, b4, b5];
        let rtc = Rtc::from_le_bytes(bytes);
        let raw = rtc.as_raw();
        let reconstructed = Rtc::from_raw(raw);
        prop_assert_eq!(rtc, reconstructed);
    }

    #[test]
    fn rtc_elapsed_ticks_bounded(a: u64, b: u64) {
        let r1 = Rtc::from_raw(a);
        let r2 = Rtc::from_raw(b);
        let elapsed = r1.elapsed_ticks(r2);
        prop_assert!(elapsed <= 0x0000_FFFF_FFFF_FFFF);
    }

    #[test]
    fn rtc_elapsed_self_is_zero(value: u64) {
        let rtc = Rtc::from_raw(value);
        prop_assert_eq!(rtc.elapsed_ticks(rtc), 0);
    }

    #[test]
    fn rtc_nanos_is_ticks_times_100(value in 0u64..=0x0000_FFFF_FFFF_FFFFu64) {
        let rtc = Rtc::from_raw(value);
        prop_assert_eq!(rtc.to_nanos(), rtc.as_raw() * 100);
    }
}

// ── AbsoluteTime properties ──────────────────────────────────────────

proptest! {
    #[test]
    fn absolute_time_add_sub_round_trip(
        doy in 1u16..=366,
        h in 0u8..=23,
        m in 0u8..=59,
        s in 0u8..=59,
        ns in 0u32..1_000_000_000,
        delta in 0u64..1_000_000_000,  // < 1 second to avoid day wrap
    ) {
        let t = AbsoluteTime::new(doy, h, m, s, ns).unwrap();
        let added = t.add_nanos(delta);
        let restored = added.sub_nanos(delta);
        prop_assert_eq!(t.day_of_year, restored.day_of_year);
        prop_assert_eq!(t.hours, restored.hours);
        prop_assert_eq!(t.minutes, restored.minutes);
        prop_assert_eq!(t.seconds, restored.seconds);
        prop_assert_eq!(t.nanoseconds, restored.nanoseconds);
    }

    #[test]
    fn absolute_time_add_monotonic(
        doy in 1u16..=300,
        h in 0u8..=22,
        m in 0u8..=58,
        s in 0u8..=58,
        ns in 0u32..999_000_000,
        delta in 1u64..3_600_000_000_000,  // up to 1 hour
    ) {
        let t = AbsoluteTime::new(doy, h, m, s, ns).unwrap();
        let t2 = t.add_nanos(delta);
        let ns1 = (t.day_of_year as u64 - 1) * 86_400_000_000_000 + t.total_nanos_of_day();
        let ns2 = (t2.day_of_year as u64 - 1) * 86_400_000_000_000 + t2.total_nanos_of_day();
        prop_assert!(ns2 > ns1, "time must advance: {} > {}", ns2, ns1);
    }

    #[test]
    fn absolute_time_display_contains_time_fields(
        doy in 1u16..=366,
        h in 0u8..=23,
        m in 0u8..=59,
        s in 0u8..=59,
    ) {
        let t = AbsoluteTime::new(doy, h, m, s, 0).unwrap();
        let display = format!("{}", t);
        // Should contain HH:MM:SS
        let expected = format!("{:02}:{:02}:{:02}", h, m, s);
        prop_assert!(display.contains(&expected));
    }
}

// ── IEEE-1588 properties ─────────────────────────────────────────────

proptest! {
    #[test]
    fn ieee1588_nanos_consistent(secs: u32, ns in 0u32..1_000_000_000) {
        let t = Ieee1588Time { seconds: secs, nanoseconds: ns };
        let total = t.to_nanos_since_epoch();
        prop_assert_eq!(total, (secs as u64) * 1_000_000_000 + (ns as u64));
    }
}

// ── CSDW properties ──────────────────────────────────────────────────

proptest! {
    #[test]
    fn csdw_field_extraction_stable(raw: u32) {
        let csdw = TimeF1Csdw::from_raw(raw);
        // Same raw value should always produce the same fields
        let csdw2 = TimeF1Csdw::from_raw(raw);
        prop_assert_eq!(csdw.as_raw(), csdw2.as_raw());
    }
}

// ── BCD properties ───────────────────────────────────────────────────

proptest! {
    #[test]
    fn bcd_parsed_fields_always_in_range(
        b0: u8, b1: u8, b2: u8, b3: u8, b4: u8, b5: u8, b6: u8, b7: u8,
    ) {
        // Any random bytes that successfully parse must produce in-range fields
        let buf = [b0, b1, b2, b3, b4, b5, b6, b7];
        if let Ok(t) = irig106_time::bcd::DayFormatTime::from_le_bytes(&buf) {
            prop_assert!(t.hours <= 23);
            prop_assert!(t.minutes <= 59);
            prop_assert!(t.seconds <= 59);
            prop_assert!((1..=366).contains(&t.day_of_year));
        }
    }

    #[test]
    fn ntp_fraction_nanos_bounded(fraction: u32) {
        let ntp = irig106_time::network_time::NtpTime {
            seconds: 0,
            fraction,
        };
        let nanos = ntp.fraction_as_nanos();
        prop_assert!(nanos < 1_000_000_000, "nanos {} must be < 1B", nanos);
    }

    #[test]
    fn ptp_nanos_since_epoch_monotonic(
        s1 in 0u64..1_000_000_000,
        s2 in 0u64..1_000_000_000,
    ) {
        let (lo, hi) = if s1 <= s2 { (s1, s2) } else { (s2, s1) };
        let t1 = irig106_time::network_time::PtpTime { seconds: lo, nanoseconds: 0 };
        let t2 = irig106_time::network_time::PtpTime { seconds: hi, nanoseconds: 0 };
        prop_assert!(t2.to_nanos_since_tai_epoch() >= t1.to_nanos_since_tai_epoch());
    }
}

// ── Encode round-trip properties (v0.4.0) ────────────────────────────

proptest! {
    #[test]
    fn rtc_encode_round_trip(b0: u8, b1: u8, b2: u8, b3: u8, b4: u8, b5: u8) {
        let bytes = [b0, b1, b2, b3, b4, b5];
        let rtc = irig106_time::Rtc::from_le_bytes(bytes);
        prop_assert_eq!(rtc.to_le_bytes(), bytes);
    }

    #[test]
    fn csdw_f1_encode_round_trip(raw: u32) {
        let csdw = irig106_time::TimeF1Csdw::from_raw(raw);
        let bytes = csdw.to_le_bytes();
        let csdw2 = irig106_time::TimeF1Csdw::from_le_bytes(bytes);
        prop_assert_eq!(csdw.as_raw(), csdw2.as_raw());
    }

    #[test]
    fn ntp_encode_round_trip(seconds: u32, fraction: u32) {
        let ntp = irig106_time::network_time::NtpTime { seconds, fraction };
        let bytes = ntp.to_le_bytes();
        let ntp2 = irig106_time::network_time::NtpTime::from_le_bytes(&bytes).unwrap();
        prop_assert_eq!(ntp.seconds, ntp2.seconds);
        prop_assert_eq!(ntp.fraction, ntp2.fraction);
    }

    #[test]
    fn ptp_encode_round_trip(
        seconds in 0u64..=0x0000_FFFF_FFFF_FFFFu64,
        nanoseconds in 0u32..1_000_000_000u32,
    ) {
        let ptp = irig106_time::network_time::PtpTime { seconds, nanoseconds };
        let bytes = ptp.to_le_bytes();
        let ptp2 = irig106_time::network_time::PtpTime::from_le_bytes(&bytes).unwrap();
        prop_assert_eq!(ptp.seconds, ptp2.seconds);
        prop_assert_eq!(ptp.nanoseconds, ptp2.nanoseconds);
    }
}
