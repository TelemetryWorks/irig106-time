#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::rtc::Rtc;

fuzz_target!(|data: &[u8]| {
    // from_le_bytes requires exactly 6 bytes
    if data.len() >= 6 {
        let rtc = Rtc::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5]]);

        // Invariant: raw value fits in 48 bits
        assert!(rtc.as_raw() <= 0x0000_FFFF_FFFF_FFFF);

        // to_nanos must not overflow (48-bit * 100 fits in u64)
        let nanos = rtc.to_nanos();
        assert_eq!(nanos, rtc.as_raw() * 100);
    }

    // from_raw with arbitrary u64
    if data.len() >= 8 {
        let val = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]);
        let rtc = Rtc::from_raw(val);
        assert!(rtc.as_raw() <= 0x0000_FFFF_FFFF_FFFF);

        // elapsed_ticks with self must be 0
        assert_eq!(rtc.elapsed_ticks(rtc), 0);
    }

    // elapsed_ticks between two RTCs
    if data.len() >= 12 {
        let a = Rtc::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5]]);
        let b = Rtc::from_le_bytes([data[6], data[7], data[8], data[9], data[10], data[11]]);

        // Must not panic
        let _ticks = a.elapsed_ticks(b);
        let _nanos = a.elapsed_nanos(b);

        // elapsed_ticks result fits in 48 bits
        assert!(a.elapsed_ticks(b) <= 0x0000_FFFF_FFFF_FFFF);
    }
});
