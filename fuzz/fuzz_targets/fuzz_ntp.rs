#![no_main]
use irig106_time::network_time::NtpTime;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    match NtpTime::from_le_bytes(data) {
        Ok(ntp) => {
            // fraction_as_nanos must not panic and must be < 1B
            let nanos = ntp.fraction_as_nanos();
            assert!(nanos < 1_000_000_000);

            // to_unix_seconds must not panic
            let _ = ntp.to_unix_seconds();

            // to_nanos_since_ntp_epoch must not panic
            let _ = ntp.to_nanos_since_ntp_epoch();

            // to_absolute must not panic (may return Err for pre-Unix-epoch)
            let _ = ntp.to_absolute();
        }
        Err(_) => {}
    }
});
