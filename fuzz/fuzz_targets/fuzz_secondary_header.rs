#![no_main]
use libfuzzer_sys::fuzz_target;
use irig106_time::secondary::{parse_secondary_header, validate_secondary_checksum, SecHdrTimeFormat};

fuzz_target!(|data: &[u8]| {
    // Checksum validation must never panic
    if data.len() >= 12 {
        let _ = validate_secondary_checksum(data);
    }

    // Full parse with each format variant must never panic
    if data.len() >= 12 {
        let formats = [
            SecHdrTimeFormat::Ch4,
            SecHdrTimeFormat::Ieee1588,
            SecHdrTimeFormat::Ertc,
            SecHdrTimeFormat::Reserved(3),
        ];
        for fmt in &formats {
            let _ = parse_secondary_header(data, *fmt);
        }
    }

    // Short buffers must return BufferTooShort, never panic
    let _ = validate_secondary_checksum(data);
    let _ = parse_secondary_header(data, SecHdrTimeFormat::Ieee1588);
});
