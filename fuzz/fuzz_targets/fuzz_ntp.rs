#![no_main]
use irig106_time::network_time::NtpTime;
use irig106_time::{ntp::{NtpContext,convert_ntp},LeapSecondTable};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    match NtpTime::from_le_bytes(data) {
        Ok(ntp) => {
            // fraction_as_nanos must not panic and must be < 1B
            let nanos = ntp.fraction_as_nanos();
            assert!(nanos < 1_000_000_000);

            let table=LeapSecondTable::builtin();
            let converted=convert_ntp(ntp,&NtpContext::new(1),Some(&table));
            assert_eq!(converted.raw(),ntp);
            let _=converted.utc(); // Range/context outcomes are values, not fuzz crashes.

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
