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

// Partial CLI-104: bounded online state, not complete qualified report coverage.
#[test]
fn online_reference_jump_and_gap_state_is_bounded_and_exact() {
    let mut scanner = Ch10TimeScanner::new();
    scanner.jump_threshold = Some(1_000_000_000);
    for i in 0..5000 {
        // Fig11-13 DOY wire fields, 00:00:00 / 00:00:05, same RTC.
        let body = [0, 0, 0, 0, 0, if i % 2 == 0 { 0 } else { 5 }, 0, 0, 1, 0];
        scanner.process_time_packet(
            &body,
            &PktHeader {
                channel_id: 1,
                data_type: 0x11,
                rtc: Rtc::from_raw(0),
            },
        );
    }
    assert_eq!(scanner.correlator.len(), 1024);
    assert_eq!(scanner.correlator.total_evicted(), 3976);
    assert_eq!(scanner.references_seen, 5000);
    assert_eq!(scanner.jumps_seen, 4999);
    assert_eq!(scanner.jumps.len(), 20);
    assert_eq!(scanner.finding_counts, [0, 4999, 0]);
    assert_eq!(scanner.errors.len(), 20);
    assert_eq!(scanner.time_channels.len(), 1);
    let info = &scanner.time_channels[&1];
    assert_eq!(info.packet_count, 5000);
    assert_eq!(info.min_gap_ns, Some(0));
    assert_eq!(info.max_gap_ns, Some(0));
    assert_eq!(info.last_time.unwrap().seconds(), 5);
    assert_eq!(scanner.jumps[0].delta_nanos, 5_000_000_000);
    assert_eq!(scanner.jumps[1].delta_nanos, -5_000_000_000);
}

#[test]
fn online_gap_extrema_use_previous_reference_and_jump_threshold_is_strict() {
    let mut scanner = Ch10TimeScanner::new();
    scanner.jump_threshold = Some(1_000_000_000);
    for (ticks, seconds) in [(0, 0), (10_000_000, 2), (30_000_000, 3), (60_000_000, 8)] {
        let body = [0, 0, 0, 0, 0, seconds, 0, 0, 1, 0];
        scanner.process_time_packet(
            &body,
            &PktHeader {
                channel_id: 1,
                data_type: 0x11,
                rtc: Rtc::from_raw(ticks),
            },
        );
    }
    let info = &scanner.time_channels[&1];
    assert_eq!(info.min_gap_ns, Some(1_000_000_000));
    assert_eq!(info.max_gap_ns, Some(3_000_000_000));
    assert_eq!(scanner.jumps_seen, 1); // +/-1 s equality is not >1s; +2s is.
    assert_eq!(scanner.jumps[0].delta_nanos, 2_000_000_000);
    assert_eq!(scanner.jumps[0].index, 3);
}

#[test]
fn target_brackets_match_full_library_for_interleaved_channels() {
    for target in [0, 5, 50, 55, 1000, 1205, 1300] {
        let rtc = Rtc::from_raw(target);
        let mut query = target_refs::TargetReferences::new(rtc);
        let mut full = TimeCorrelator::new();
        for n in (1..=120).rev() {
            // Late observations; two different channel clocks.
            for ch in [1, 2] {
                let r = Rtc::from_raw(n * 10);
                let t = AbsoluteTime::new(100, 12 + ch as u8, 0, 0, (n * n * 1000) as u32).unwrap();
                query.observe(ch, r, t);
                full.add_reference(ch, r, t);
            }
        }
        for ch in [None, Some(1), Some(2), Some(3)] {
            assert_eq!(
                query.correlate(ch),
                full.correlate(rtc, ch),
                "target {target} channel {ch:?}"
            );
        }
    }
}

#[test]
fn late_ptp_still_qualifies_query_before_live_eviction() {
    let mut scanner = Ch10TimeScanner::new();
    scanner.query_refs = Some(target_refs::TargetReferences::new(Rtc::from_raw(
        10_000_000,
    )));
    scanner.jump_threshold = Some(1_000_000_000);
    for seconds in [0u32, 40, 1] {
        // Fig11-15/17: NTF1, TS1; TAI 2017-01-02 = UTC 1483315200 +37.
        let mut body = [0; 12];
        body[0] = 0x11;
        body[4..8].copy_from_slice(&(1_483_315_237 + seconds).to_le_bytes());
        scanner.process_time_f2_packet(
            &body,
            &PktHeader {
                channel_id: 1,
                data_type: 0x12,
                rtc: Rtc::from_raw(u64::from(seconds) * 10_000_000),
            },
        );
        scanner.total_packets += 1;
    }
    assert_eq!(scanner.references_seen, 3);
    assert_eq!(scanner.jumps_seen, 0);
    assert_eq!(scanner.finding_counts, [0, 1, 0]);
    assert_eq!(scanner.time_channels[&1].max_gap_ns, Some(40_000_000_000));
    assert!(matches!(
        scanner
            .correlator
            .correlate(Rtc::from_raw(10_000_000), None),
        Err(TimeError::QueryBeforeRetainedWindow)
    ));
    let time = scanner.resolve(Rtc::from_raw(10_000_000), None).unwrap();
    assert_eq!(
        (time.year(), time.day_of_year(), time.seconds()),
        (Some(2017), 2, 1)
    );
}

#[test]
fn target_brackets_preserve_main_duplicate_rtc_boundary_answers() {
    for duplicates in [1, 2, 3, 9, 32] {
        for target in [0, 10, 15, 20, 25] {
            let rtc = Rtc::from_raw(target);
            let mut query = target_refs::TargetReferences::new(rtc);
            let mut full = TimeCorrelator::new();
            for ticks in [10, 20] {
                for second in 0..duplicates {
                    for ch in [1, 2] {
                        let time = AbsoluteTime::new(100, 12 + ch as u8, 0, second, 0).unwrap();
                        query.observe(ch, Rtc::from_raw(ticks), time);
                        full.add_reference(ch, Rtc::from_raw(ticks), time);
                    }
                }
            }
            for ch in [None, Some(1), Some(2)] {
                assert_eq!(
                    query.correlate(ch),
                    full.correlate(rtc, ch),
                    "target {target} channel {ch:?}"
                );
            }
        }
    }
}

#[test]
fn late_arrival_preserves_pair_baseline_but_updates_inventory_and_query() {
    for network in [false, true] {
        let mut scanner = Ch10TimeScanner::new();
        scanner.jump_threshold = Some(1_000_000_000);
        scanner.query_refs = Some(target_refs::TargetReferences::new(Rtc::from_raw(
            80_000_000,
        )));
        for seconds in (0..=20).chain(core::iter::once(8u32)) {
            let clock = seconds + if scanner.total_packets == 21 { 300 } else { 0 };
            let hdr = PktHeader {
                channel_id: 1,
                data_type: if network { 0x12 } else { 0x11 },
                rtc: Rtc::from_raw(u64::from(seconds) * 10_000_000),
            };
            if network {
                // Fig11-15/17: NTF1/TS1, 2017-01-02 UTC epoch +37 TAI.
                let mut body = [0; 12];
                body[0] = 0x11;
                body[4..8].copy_from_slice(&(1_483_315_237 + clock).to_le_bytes());
                scanner.process_time_f2_packet(&body, &hdr);
            } else {
                // Fig11-13: day1, 12:00:SS, or late 12:05:08.
                let bcd = |n: u32| (((n / 10) << 4) | (n % 10)) as u8;
                let body = [0, 0, 0, 0, 0, bcd(clock % 60), bcd(clock / 60), 0x12, 1, 0];
                scanner.process_time_packet(&body, &hdr);
            }
            scanner.total_packets += 1;
        }
        let info = &scanner.time_channels[&1];
        assert_eq!(info.packet_count, 22);
        assert_eq!(info.last_rtc, Some(Rtc::from_raw(80_000_000)));
        assert_eq!(info.last_time.unwrap().minutes(), 5);
        let (baseline_rtc, baseline_time) = info.pair_baseline.unwrap();
        assert_eq!(baseline_rtc, Rtc::from_raw(200_000_000));
        assert_eq!((baseline_time.minutes(), baseline_time.seconds()), (0, 20));
        assert_eq!(
            (info.min_gap_ns, info.max_gap_ns),
            (Some(1_000_000_000), Some(1_000_000_000))
        );
        assert_eq!(scanner.finding_counts, [0, 1, 0]);
        assert_eq!(scanner.jumps_seen, 0);
        assert_eq!(
            scanner
                .correlate(Rtc::from_raw(80_000_000), Some(1))
                .unwrap()
                .minutes(),
            5
        );
    }
}
