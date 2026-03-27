#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::intra_packet::{parse_intra_packet_time, IntraPacketTimeFormat};

fuzz_target!(|data: &[u8]| {
    let formats = [
        IntraPacketTimeFormat::Rtc48,
        IntraPacketTimeFormat::Ch4Binary,
        IntraPacketTimeFormat::Ieee1588,
        IntraPacketTimeFormat::Ertc64,
    ];
    for fmt in &formats {
        let _ = parse_intra_packet_time(data, *fmt);
    }
});
