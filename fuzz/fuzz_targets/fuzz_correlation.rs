#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::{AbsoluteTime, Rtc, TimeCorrelator};

fuzz_target!(|data: &[u8]| {
    // Need at least 16 bytes to construct a reference point + target
    if data.len() < 16 {
        return;
    }

    let mut correlator = TimeCorrelator::new();

    // Construct reference points from chunks of the fuzz input
    let mut offset = 0;
    while offset + 10 <= data.len() && correlator.len() < 100 {
        let channel_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let rtc = Rtc::from_le_bytes([
            data[offset + 2], data[offset + 3], data[offset + 4],
            data[offset + 5], data[offset + 6], data[offset + 7],
        ]);
        let hour = data[offset + 8] % 24;
        let minute = data[offset + 9] % 60;

        // Use safe constructor — it may fail, that's fine
        if let Ok(time) = AbsoluteTime::new(1, hour, minute, 0, 0) {
            correlator.add_reference(channel_id, rtc, time);
        }
        offset += 10;
    }

    // Correlate with remaining bytes as target RTC
    if data.len() >= offset + 6 {
        let target = Rtc::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2],
            data[offset + 3], data[offset + 4], data[offset + 5],
        ]);

        // Must not panic regardless of state
        let _ = correlator.correlate(target, None);
        let _ = correlator.correlate(target, Some(0));
        let _ = correlator.correlate(target, Some(1));
    }

    // Jump detection must never panic
    let _ = correlator.detect_time_jump(0, 1_000_000);
    let _ = correlator.detect_time_jump(1, 0); // zero threshold
});
