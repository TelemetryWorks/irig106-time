#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::network_time::{PtpTime, LeapSecondTable};

fuzz_target!(|data: &[u8]| {
    match PtpTime::from_le_bytes(data) {
        Ok(ptp) => {
            assert!(ptp.nanoseconds < 1_000_000_000);
            let _ = ptp.to_nanos_since_tai_epoch();
            let _ = ptp.to_utc_seconds(37);
            let _ = ptp.to_absolute(37);

            // Test with builtin leap table
            let table = LeapSecondTable::builtin();
            let offset = table.offset_at_tai(ptp.seconds);
            let _ = ptp.to_absolute(offset);
        }
        Err(_) => {}
    }
});
