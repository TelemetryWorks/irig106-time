#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::csdw::TimeF1Csdw;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 4 {
        let csdw = TimeF1Csdw::from_le_bytes([data[0], data[1], data[2], data[3]]);

        // All field extractors must not panic
        let _src = csdw.time_source();
        let _fmt = csdw.time_format();
        let _leap = csdw.is_leap_year();
        let _dfmt = csdw.date_format();

        // Round-trip: raw → construct → raw
        assert_eq!(csdw.as_raw(), TimeF1Csdw::from_raw(csdw.as_raw()).as_raw());
    }

    // Any u32 must produce a valid CSDW (no panics)
    if data.len() >= 4 {
        let raw = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let csdw = TimeF1Csdw::from_raw(raw);
        let _ = csdw.time_source();
        let _ = csdw.time_format();
    }
});
