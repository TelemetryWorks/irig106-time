#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::absolute::Ieee1588Time;

fuzz_target!(|data: &[u8]| {
    match Ieee1588Time::from_le_bytes(data) {
        Ok(t) => {
            assert!(t.nanoseconds < 1_000_000_000);
            let epoch_ns = t.to_nanos_since_epoch();
            assert_eq!(
                epoch_ns,
                (t.seconds as u64) * 1_000_000_000 + (t.nanoseconds as u64)
            );
        }
        Err(_) => {}
    }
});
