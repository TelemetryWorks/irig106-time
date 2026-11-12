//! CLI Format 1 consumer regressions from 106-24R1 Figures 11-13/14.
use super::*;

fn consume_bcd(body: &[u8], calendar: bool) -> Ch10TimeScanner {
    // Exact consumer buffer: header + CSDW + body, without trailer filler.
    let mut packet = vec![0; HEADER_SIZE];
    packet.extend_from_slice(&(if calendar { 0x200u32 } else { 0 }).to_le_bytes());
    packet.extend_from_slice(body);
    let hdr = PktHeader {
        channel_id: 1,
        packet_length: packet.len() as u32,
        _data_length: (4 + body.len()) as u32,
        packet_flags: 0,
        data_type: DATA_TYPE_TIME_F1,
        rtc: Rtc::from_raw(1),
    };
    let mut scanner = Ch10TimeScanner::new();
    scanner.process_time_packet(&packet, &hdr);
    scanner
}

/// Requirements: L1-BCD-005, L2-BCD-105, L3-BCD-105
#[test]
fn t12_cli_accepts_six_byte_day_body_at_end() {
    let body = [0x35, 0x27, 0x45, 0x13, 0x87, 0x01];
    let scanner = consume_bcd(&body, false);
    assert!(scanner.errors.is_empty(), "{:?}", scanner.errors);
    assert_eq!(scanner.correlator.len(), 1);
    let time = scanner.time_channels[&1].first_time.unwrap();
    assert_eq!(
        (
            time.day_of_year(),
            time.hours(),
            time.minutes(),
            time.seconds()
        ),
        (187, 13, 45, 27)
    );
    for n in 0..body.len() {
        let short = consume_bcd(&body[..n], false);
        assert_eq!(short.errors.len(), 1);
        assert_eq!(short.correlator.len(), 0);
    }
}

/// Requirements: L1-BCD-006, L2-BCD-106, L3-BCD-106
#[test]
fn t12_cli_accepts_eight_byte_calendar_body_at_end() {
    let body = [0x35, 0x27, 0x45, 0x13, 0x29, 0x02, 0x24, 0x20];
    let scanner = consume_bcd(&body, true);
    assert!(scanner.errors.is_empty(), "{:?}", scanner.errors);
    assert_eq!(scanner.correlator.len(), 1);
    let time = scanner.time_channels[&1].first_time.unwrap();
    assert_eq!(
        (time.year(), time.day_of_year(), time.hours()),
        (Some(2024), 60, 13)
    );
    for n in 0..body.len() {
        let short = consume_bcd(&body[..n], true);
        assert_eq!(short.errors.len(), 1);
        assert_eq!(short.correlator.len(), 0);
    }
}
