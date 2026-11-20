//! CLI Format 1 consumer regressions from 106-24R1 Figures 11-13/14.
use super::*;

fn consume_bcd(body: &[u8], calendar: bool) -> Ch10TimeScanner {
    // Exact data-only consumer buffer: CSDW + body, without trailer filler.
    let mut packet = Vec::new();
    packet.extend_from_slice(&(if calendar { 0x200u32 } else { 0 }).to_le_bytes());
    packet.extend_from_slice(body);
    let hdr = PktHeader {
        channel_id: 1,
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

// CLI-104 CSV row-retention clause only; qualified joining remains incomplete.
#[test]
fn csv_row_sink_does_not_retain_packet_rows() {
    // §11.2.1.1: EB25 + 0018 + 0900 = F43D, non-time packet, Data Length0.
    let packet = [
        0x25, 0xEB, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0x3D, 0xF4,
    ];
    let bytes = packet.repeat(64);
    let mut scanner = Ch10TimeScanner::new();
    let mut count = 0;
    let mut emit = |_: &ResolvedPacket| {
        count += 1;
        Ok(())
    };
    scanner
        .scan_to(&bytes[..], true, None, Some(&mut emit))
        .map_err(|e| e.message)
        .unwrap();
    assert_eq!(count, 64);
    assert!(scanner.resolved.is_empty());
}

// Partial CLI-104/107: sample bounds must never cap factual counts or severity.
#[test]
fn finding_samples_are_bounded_and_levels_remain_exact() {
    let mut s = Ch10TimeScanner::new();
    for i in 0..30 {
        s.note(Severity::Warning, format!("sample {i}"));
    }
    for i in 0..7 {
        s.note(Severity::Information, format!("info {i}"));
    }
    s.note(Severity::Error, "late error");
    assert_eq!(s.errors.len(), 20);
    assert_eq!(s.highest, Some(Severity::Error));
    let mut out = Vec::new();
    s.report_findings(&mut out).map_err(|e| e.message).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("Findings: error=1, warning=30, info=7 (total=38)"),
        "{text}"
    );
    assert!(text.contains("18 more findings"), "{text}");
    assert!(text.contains("Warning: sample 19"));
    assert!(!text.contains("sample 20"));
    assert!(!text.contains("late error"));
}
