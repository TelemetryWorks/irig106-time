//! Property-based tests for `irig106-time`.
//!
//! These tests verify invariants that must hold for ALL valid inputs,
//! not just hand-picked examples. Uses `proptest` for generation.
//!
//! # Security Relevance
//!
//! Property tests complement fuzzing: fuzzing finds crashes, property tests
//! find logical errors (wrong values that don't crash but corrupt analysis).
//!
//! # Run
//! ```sh
//! cargo test --test properties
//! ```
//!
//! # Test Documentation
//!
//! | Test | Property | Security Relevance |
//! |------|----------|-------------------|
//! | `rtc_from_raw_always_48bit` | Masking invariant | Prevents 48-bit overflow in arithmetic |
//! | `rtc_round_trip_le_bytes` | Encode/decode identity | Data integrity |
//! | `rtc_elapsed_ticks_bounded` | Result fits 48 bits | No u64 overflow in time math |
//! | `rtc_elapsed_self_is_zero` | Self-elapsed = 0 | Correctness foundation |
//! | `rtc_nanos_is_ticks_times_100` | Conversion formula | Time accuracy |
//! | `absolute_time_add_sub_round_trip` | add then sub = identity | Correlation correctness |
//! | `absolute_time_add_monotonic` | Adding nanos advances time | No time reversal |
//! | `ieee1588_nanos_consistent` | Total = s*1B + ns | Epoch conversion correctness |
//! | `csdw_field_extraction_stable` | Same raw → same fields | No state-dependent parsing |
//! | `bcd_valid_parse_fields_in_range` | Parsed fields within spec | No out-of-range corruption |

// NOTE: This file requires `proptest` in dev-dependencies.
// Add to Cargo.toml: [dev-dependencies] proptest = "1"

#[cfg(test)]
mod tests {
    use irig106_time::*;

    // ── If proptest is available, use it. Otherwise, manual iteration. ──
    // Uncomment the proptest import when the dependency is added:
    // use proptest::prelude::*;

    // For now, we use a deterministic pseudo-random approach that needs
    // no external crate. Replace with proptest macros when available.

    /// Simple deterministic PRNG for property testing without dependencies.
    struct Xorshift(u64);
    impl Xorshift {
        fn new(seed: u64) -> Self { Self(seed) }
        fn next_u64(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn next_u8(&mut self) -> u8 { (self.next_u64() & 0xFF) as u8 }
        fn next_u16(&mut self) -> u16 { (self.next_u64() & 0xFFFF) as u16 }
        fn next_u32(&mut self) -> u32 { (self.next_u64() & 0xFFFF_FFFF) as u32 }
    }

    const ITERATIONS: usize = 10_000;

    #[test]
    fn rtc_from_raw_always_48bit() {
        let mut rng = Xorshift::new(0xDEAD_BEEF);
        for _ in 0..ITERATIONS {
            let val = rng.next_u64();
            let rtc = Rtc::from_raw(val);
            assert!(
                rtc.as_raw() <= 0x0000_FFFF_FFFF_FFFF,
                "from_raw({val:#018X}) produced {:#018X}", rtc.as_raw()
            );
        }
    }

    #[test]
    fn rtc_round_trip_le_bytes() {
        let mut rng = Xorshift::new(0xCAFE_BABE);
        for _ in 0..ITERATIONS {
            let bytes: [u8; 6] = [
                rng.next_u8(), rng.next_u8(), rng.next_u8(),
                rng.next_u8(), rng.next_u8(), rng.next_u8(),
            ];
            let rtc = Rtc::from_le_bytes(bytes);
            // Reconstruct from raw and verify
            let expected = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], 0, 0,
            ]);
            assert_eq!(rtc.as_raw(), expected);
        }
    }

    #[test]
    fn rtc_elapsed_ticks_bounded() {
        let mut rng = Xorshift::new(0x1234_5678);
        for _ in 0..ITERATIONS {
            let a = Rtc::from_raw(rng.next_u64());
            let b = Rtc::from_raw(rng.next_u64());
            let ticks = a.elapsed_ticks(b);
            assert!(
                ticks <= 0x0000_FFFF_FFFF_FFFF,
                "elapsed_ticks({:#X}, {:#X}) = {:#X}", a.as_raw(), b.as_raw(), ticks
            );
        }
    }

    #[test]
    fn rtc_elapsed_self_is_zero() {
        let mut rng = Xorshift::new(0xAAAA_BBBB);
        for _ in 0..ITERATIONS {
            let rtc = Rtc::from_raw(rng.next_u64());
            assert_eq!(rtc.elapsed_ticks(rtc), 0);
            assert_eq!(rtc.elapsed_nanos(rtc), 0);
        }
    }

    #[test]
    fn rtc_nanos_is_ticks_times_100() {
        let mut rng = Xorshift::new(0x9999_0000);
        for _ in 0..ITERATIONS {
            let a = Rtc::from_raw(rng.next_u64());
            let b = Rtc::from_raw(rng.next_u64());
            let ticks = a.elapsed_ticks(b);
            let nanos = a.elapsed_nanos(b);
            assert_eq!(nanos, ticks * 100);
        }
    }

    #[test]
    fn absolute_time_add_sub_round_trip() {
        let mut rng = Xorshift::new(0xFEED_FACE);
        for _ in 0..ITERATIONS {
            let doy = (rng.next_u16() % 366) + 1;
            let h = rng.next_u8() % 24;
            let m = rng.next_u8() % 60;
            let s = rng.next_u8() % 60;
            let ns = rng.next_u32() % 1_000_000_000;

            let t = AbsoluteTime::new(doy, h, m, s, ns).unwrap();

            // Add then subtract a small amount (< 1 day to avoid day wrap complexity)
            let delta = rng.next_u64() % 1_000_000_000; // < 1 second
            let added = t.add_nanos(delta);
            let restored = added.sub_nanos(delta);

            assert_eq!(t.day_of_year, restored.day_of_year, "day mismatch for delta={delta}");
            assert_eq!(t.hours, restored.hours, "hours mismatch for delta={delta}");
            assert_eq!(t.minutes, restored.minutes, "minutes mismatch for delta={delta}");
            assert_eq!(t.seconds, restored.seconds, "seconds mismatch for delta={delta}");
            assert_eq!(t.nanoseconds, restored.nanoseconds, "nanos mismatch for delta={delta}");
        }
    }

    #[test]
    fn absolute_time_add_monotonic() {
        let mut rng = Xorshift::new(0x0BAD_F00D);
        for _ in 0..ITERATIONS {
            let doy = (rng.next_u16() % 300) + 1; // avoid day-wrap edge
            let h = rng.next_u8() % 23;
            let m = rng.next_u8() % 59;
            let s = rng.next_u8() % 59;
            let ns = rng.next_u32() % 999_000_000;

            let t = AbsoluteTime::new(doy, h, m, s, ns).unwrap();
            let delta = (rng.next_u64() % 3_600_000_000_000) + 1; // up to 1 hour
            let t2 = t.add_nanos(delta);

            let ns1 = (t.day_of_year as u64 - 1) * 86_400_000_000_000 + t.total_nanos_of_day();
            let ns2 = (t2.day_of_year as u64 - 1) * 86_400_000_000_000 + t2.total_nanos_of_day();
            assert!(
                ns2 > ns1,
                "add_nanos({delta}) did not advance time: {ns1} → {ns2}"
            );
        }
    }

    #[test]
    fn ieee1588_nanos_consistent() {
        let mut rng = Xorshift::new(0xBEEF_CAFE);
        for _ in 0..ITERATIONS {
            let secs = rng.next_u32();
            let nanos = rng.next_u32() % 1_000_000_000;
            let t = Ieee1588Time { nanoseconds: nanos, seconds: secs };
            assert_eq!(
                t.to_nanos_since_epoch(),
                (secs as u64) * 1_000_000_000 + (nanos as u64)
            );
        }
    }

    #[test]
    fn csdw_field_extraction_stable() {
        let mut rng = Xorshift::new(0x1111_2222);
        for _ in 0..ITERATIONS {
            let raw = rng.next_u32();
            let csdw = TimeF1Csdw::from_raw(raw);

            // Extract twice — must be identical (no hidden state)
            let src1 = csdw.time_source();
            let src2 = csdw.time_source();
            assert_eq!(src1, src2);

            let fmt1 = csdw.time_format();
            let fmt2 = csdw.time_format();
            assert_eq!(fmt1, fmt2);

            let leap1 = csdw.is_leap_year();
            let leap2 = csdw.is_leap_year();
            assert_eq!(leap1, leap2);
        }
    }

    #[test]
    fn bcd_valid_parse_fields_in_range() {
        // Generate valid BCD bytes and verify parsed fields stay in range.
        let mut rng = Xorshift::new(0x3333_4444);
        for _ in 0..ITERATIONS {
            // Generate valid BCD digits
            let hmn = rng.next_u8() % 10; // hundreds of ms: 0-9
            let tmn = rng.next_u8() % 10; // tens of ms: 0-9
            let sn = rng.next_u8() % 10;  // units of seconds
            let tsn = rng.next_u8() % 6;  // tens of seconds: 0-5
            let mn = rng.next_u8() % 10;  // units of minutes
            let tmn_m = rng.next_u8() % 6;// tens of minutes: 0-5
            let hn = rng.next_u8() % 10;  // units of hours
            let thn = rng.next_u8() % 3;  // tens of hours: 0-2
            let dn = (rng.next_u8() % 9) + 1; // units of day: 1-9
            let tdn = rng.next_u8() % 10; // tens of day: 0-9
            let hdn = rng.next_u8() % 4;  // hundreds of day: 0-3

            // Validate decoded values would be in range
            let hours = thn * 10 + hn;
            let day = (hdn as u16) * 100 + (tdn as u16) * 10 + (dn as u16);
            if hours > 23 || day == 0 || day > 366 {
                continue; // skip invalid combos
            }

            let w0: u16 = (tmn as u16)
                | ((hmn as u16) << 4)
                | ((sn as u16) << 8)
                | ((tsn as u16) << 12);
            let w1: u16 = (mn as u16)
                | ((tmn_m as u16) << 4)
                | ((hn as u16) << 8)
                | ((thn as u16) << 12);
            let w2: u16 = (dn as u16)
                | ((tdn as u16) << 4)
                | ((hdn as u16) << 8);

            let mut buf = [0u8; 8];
            buf[0..2].copy_from_slice(&w0.to_le_bytes());
            buf[2..4].copy_from_slice(&w1.to_le_bytes());
            buf[4..6].copy_from_slice(&w2.to_le_bytes());

            match irig106_time::bcd::DayFormatTime::from_le_bytes(&buf) {
                Ok(t) => {
                    assert!(t.hours <= 23);
                    assert!(t.minutes <= 59);
                    assert!(t.seconds <= 59);
                    assert!(t.day_of_year >= 1 && t.day_of_year <= 366);
                    assert!(t.milliseconds <= 990);
                }
                Err(_) => {
                    // Some combos still fail (e.g. day=0 from dn=0,tdn=0,hdn=0)
                }
            }
        }
    }
}
