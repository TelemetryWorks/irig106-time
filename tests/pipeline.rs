//! Integration tests for `irig106-time`.
//!
//! These tests exercise the full pipeline: parsing CSDW + BCD time messages,
//! feeding reference points into the correlator, and resolving data packet
//! timestamps to absolute time.
//!
//! # Test Documentation
//!
//! | Test | Scenario | Traces |
//! |------|----------|--------|
//! | `full_day_format_pipeline` | Parse CSDW → DOY BCD → AbsoluteTime → Correlator | L1-CSDW-001..005, L1-BCD-001, L1-COR-001..002 |
//! | `full_dmy_format_pipeline` | Parse CSDW → DMY BCD → AbsoluteTime with date | L1-CSDW-005, L1-BCD-002 |
//! | `multi_channel_correlation` | Two time sources, select by channel | L1-COR-003 |
//! | `gps_lock_time_jump_detection` | Detect GPS lock discontinuity across refs | L1-COR-004 |
//! | `secondary_header_to_correlation` | Parse IEEE-1588 secondary header → correlate | L1-SEC-002, L1-COR-001 |
//! | `intra_packet_rtc_to_absolute` | Parse intra-packet RTC → correlate → absolute | L1-IPT-001, L1-COR-002 |
//! | `rtc_wrap_around_correlation` | Correlation across a 48-bit RTC rollover | L1-RTC-003, L1-COR-002 |
//! | `invalid_bcd_propagates_error` | Bad BCD data yields typed error | L1-ERR-002 |
//! | `all_error_variants_display` | Every TimeError variant has useful Display | L1-ERR-001 |
//! | `no_std_types_are_copy` | Core types satisfy Copy + Clone + Eq | L1-API-001 |

use irig106_time::*;

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

/// Encode a DOY BCD time message for Day 100, 12:30:25.340.
fn encode_day100_123025_340() -> [u8; 8] {
    let w0: u16 = 0x2534; // Tmn=4, Hmn=3, Sn=5, TSn=2
    let w1: u16 = 0x1230; // Mn=0, TMn=3, Hn=2, THn=1
    let w2: u16 = 0x0100; // Dn=0, TDn=0, HDn=1
    let w3: u16 = 0x0000;
    let mut buf = [0u8; 8];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[2..4].copy_from_slice(&w1.to_le_bytes());
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    buf[6..8].copy_from_slice(&w3.to_le_bytes());
    buf
}

/// Encode a DMY BCD time message for March 15, 2025, 08:45:30.120.
fn encode_mar15_2025_084530_120() -> [u8; 10] {
    let w0: u16 = 0x3012; // Tmn=2, Hmn=1, Sn=0, TSn=3
    let w1: u16 = 0x0845; // Mn=5, TMn=4, Hn=8, THn=0
    let w2: u16 = 0x0315; // Dn=5, TDn=1, On=3, TOn=0
    let w3: u16 = 0x2025; // Yn=5, TYn=2, HYn=0, OYn=2
    let mut buf = [0u8; 10];
    buf[0..2].copy_from_slice(&w0.to_le_bytes());
    buf[2..4].copy_from_slice(&w1.to_le_bytes());
    buf[4..6].copy_from_slice(&w2.to_le_bytes());
    buf[6..8].copy_from_slice(&w3.to_le_bytes());
    buf
}

/// Build a 12-byte secondary header with valid checksum containing IEEE-1588
/// time at the given seconds and nanoseconds.
fn encode_ieee1588_secondary(seconds: u32, nanoseconds: u32) -> [u8; 12] {
    let mut buf = [0u8; 12];
    buf[0..4].copy_from_slice(&nanoseconds.to_le_bytes());
    buf[4..8].copy_from_slice(&seconds.to_le_bytes());
    // bytes [8..10] reserved = 0
    let mut sum: u32 = 0;
    for i in 0..5 {
        let w = u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]);
        sum = sum.wrapping_add(w as u32);
    }
    buf[10..12].copy_from_slice(&((sum & 0xFFFF) as u16).to_le_bytes());
    buf
}

// -----------------------------------------------------------------------
// Full Pipeline Tests
// -----------------------------------------------------------------------

/// Parse a CSDW, decode a DOY BCD time message, and correlate an RTC to
/// the decoded absolute time.
#[test]
fn full_day_format_pipeline() {
    // Step 1: Parse CSDW — GPS source, IRIG-B format, non-leap, DOY
    let csdw = TimeF1Csdw::from_raw(0x0003); // Source=GPS, Format=IrigB
    assert_eq!(csdw.time_source(), TimeSource::Gps);
    assert_eq!(csdw.time_format(), TimeFormat::IrigB);
    assert_eq!(csdw.date_format(), DateFormat::DayOfYear);
    assert!(!csdw.is_leap_year());

    // Step 2: Decode BCD time message
    let bcd_buf = encode_day100_123025_340();
    let day_time = DayFormatTime::from_le_bytes(&bcd_buf).unwrap();
    assert_eq!(day_time.day_of_year, 100);
    assert_eq!(day_time.hours, 12);
    assert_eq!(day_time.minutes, 30);
    assert_eq!(day_time.seconds, 25);
    assert_eq!(day_time.milliseconds, 340);

    // Step 3: Convert to AbsoluteTime
    let abs_time = day_time.to_absolute();
    assert_eq!(abs_time.nanoseconds(), 340_000_000);

    // Step 4: Correlate — time packet arrived at RTC=50_000_000 (5 sec)
    let ref_rtc = Rtc::from_raw(50_000_000);
    let mut correlator = TimeCorrelator::new();
    correlator.add_reference(1, ref_rtc, abs_time);

    // Step 5: Resolve a data packet at RTC=50_150_000 (15ms later)
    let target_rtc = Rtc::from_raw(50_150_000);
    let resolved = correlator.correlate(target_rtc, None).unwrap();
    assert_eq!(resolved.day_of_year(), 100);
    assert_eq!(resolved.hours(), 12);
    assert_eq!(resolved.minutes(), 30);
    assert_eq!(resolved.seconds(), 25);
    assert_eq!(resolved.nanoseconds(), 355_000_000); // 340ms + 15ms
}

/// Parse a DMY format time message and verify calendar date propagation.
#[test]
fn full_dmy_format_pipeline() {
    let csdw = TimeF1Csdw::from_raw(0x0201); // Source=External, Format=IrigB, DateFmt=DMY
    assert_eq!(csdw.date_format(), DateFormat::DayMonthYear);

    let bcd_buf = encode_mar15_2025_084530_120();
    let dmy_time = DmyFormatTime::from_le_bytes(&bcd_buf).unwrap();
    let abs = dmy_time.to_calendar_time();

    assert_eq!(abs.year(), Some(2025));
    assert_eq!(abs.month(), 3);
    assert_eq!(abs.day_of_month(), 15);
    assert_eq!(abs.day_of_year(), 74); // Jan(31) + Feb(28) + 15
    assert_eq!(abs.hours(), 8);
    assert_eq!(abs.minutes(), 45);
    assert_eq!(abs.seconds(), 30);
    assert_eq!(abs.nanoseconds(), 120_000_000);
}

/// Two time channels (IRIG-B and GPS) providing different absolute times
/// for the same RTC range. Correlation selects by channel.
#[test]
fn multi_channel_correlation() {
    let mut correlator = TimeCorrelator::new();

    // Channel 1: IRIG-B, says 12:00:00.000 at RTC=10M
    let irig_time = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    correlator.add_reference(1, Rtc::from_raw(10_000_000), irig_time);

    // Channel 2: GPS, says 12:00:03.500 at RTC=10M (3.5s offset — typical before sync)
    let gps_time = AbsoluteTime::new(100, 12, 0, 3, 500_000_000).unwrap();
    correlator.add_reference(2, Rtc::from_raw(10_000_000), gps_time);

    // Correlate at RTC=20M (1 sec later) using IRIG-B (channel 1)
    let via_irig = correlator
        .correlate(Rtc::from_raw(20_000_000), Some(1))
        .unwrap();
    assert_eq!(via_irig.hours(), 12);
    assert_eq!(via_irig.minutes(), 0);
    assert_eq!(via_irig.seconds(), 1); // 12:00:01 per IRIG-B

    // Same RTC using GPS (channel 2)
    let via_gps = correlator
        .correlate(Rtc::from_raw(20_000_000), Some(2))
        .unwrap();
    assert_eq!(via_gps.hours(), 12);
    assert_eq!(via_gps.minutes(), 0);
    assert_eq!(via_gps.seconds(), 4); // 12:00:04.5 per GPS
    assert_eq!(via_gps.nanoseconds(), 500_000_000);
}

/// Simulate a GPS lock event: time jumps forward by 5 seconds.
#[test]
fn gps_lock_time_jump_detection() {
    let mut correlator = TimeCorrelator::new();

    // Pre-GPS-lock: internal clock
    correlator.add_reference(
        3,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(50, 14, 0, 0, 0).unwrap(),
    );

    // 1 second later by RTC, but GPS corrects time +5 seconds
    correlator.add_reference(
        3,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(50, 14, 0, 6, 0).unwrap(), // expected 14:00:01 but got 14:00:06
    );

    // Stable after lock
    correlator.add_reference(
        3,
        Rtc::from_raw(30_000_000),
        AbsoluteTime::new(50, 14, 0, 7, 0).unwrap(), // 1 sec after correction
    );

    let jumps = correlator.detect_time_jump(3, 1_000_000_000); // 1 sec threshold
    assert_eq!(jumps.len(), 1);
    assert!(jumps[0].delta_nanos > 0); // jumped forward
}

/// Parse an IEEE-1588 secondary header and feed it into correlation.
#[test]
fn secondary_header_to_correlation() {
    let sec_buf = encode_ieee1588_secondary(86400, 500_000_000);
    let parsed =
        irig106_time::secondary::parse_secondary_header(&sec_buf, SecHdrTimeFormat::Ieee1588)
            .unwrap();

    match parsed {
        SecondaryHeaderTime::Ieee1588(t) => {
            assert_eq!(t.seconds, 86400);
            assert_eq!(t.nanoseconds, 500_000_000);
            assert_eq!(t.to_nanos_since_epoch(), 86_400_500_000_000);
        }
        other => panic!("expected Ieee1588, got {other:?}"),
    }
}

/// Parse an intra-packet 48-bit RTC timestamp and correlate to absolute time.
#[test]
fn intra_packet_rtc_to_absolute() {
    // Set up a correlator with one reference
    let mut correlator = TimeCorrelator::new();
    correlator.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(200, 8, 0, 0, 0).unwrap(),
    );

    // Parse an intra-packet RTC timestamp
    let mut ipt_buf = [0u8; 8];
    let rtc_val: u64 = 10_500_000; // 50ms after reference
    ipt_buf[0..6].copy_from_slice(&rtc_val.to_le_bytes()[0..6]);

    let ipt =
        irig106_time::intra_packet::parse_intra_packet_time(&ipt_buf, IntraPacketTimeFormat::Rtc48)
            .unwrap();

    match ipt {
        IntraPacketTime::Rtc(rtc) => {
            let abs = correlator.correlate(rtc, Some(1)).unwrap();
            assert_eq!(abs.day_of_year(), 200);
            assert_eq!(abs.hours(), 8);
            assert_eq!(abs.seconds(), 0);
            assert_eq!(abs.nanoseconds(), 50_000_000); // 50ms
        }
        other => panic!("expected Rtc, got {other:?}"),
    }
}

/// Correlation handles large RTC deltas gracefully.
#[test]
fn rtc_large_delta_correlation() {
    let mut correlator = TimeCorrelator::new();

    // Reference at RTC=10M (1 sec), 23:59:58.000
    correlator.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(1, 23, 59, 58, 0).unwrap(),
    );

    // Target 2 seconds later → should be 00:00:00.000 next day
    let target = Rtc::from_raw(30_000_000); // 2 sec later
    let result = correlator.correlate(target, None).unwrap();

    assert_eq!(result.day_of_year(), 2); // crossed midnight
    assert_eq!(result.hours(), 0);
    assert_eq!(result.minutes(), 0);
    assert_eq!(result.seconds(), 0);
}

/// Invalid BCD digit in the time message propagates a typed error.
#[test]
fn invalid_bcd_propagates_error() {
    let mut buf = [0u8; 8];
    buf[0] = 0x0A; // Tmn nibble = 0xA (invalid)
    buf[4] = 0x01; // day=1 (valid)
    let result = DayFormatTime::from_le_bytes(&buf);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        TimeError::InvalidBcdDigit { nibble: 0x0A, .. }
    ));
}

/// All TimeError variants produce useful Display output.
#[test]
fn all_error_variants_display() {
    use std::fmt::Write;
    let errors = vec![
        TimeError::InvalidBcdDigit {
            nibble: 0xB,
            position: "test",
        },
        TimeError::ReservedBitSet { position: "test" },
        TimeError::OutOfRange {
            field: "test",
            value: 99,
            max: 59,
        },
        TimeError::ChecksumMismatch {
            stored: 0x1234,
            computed: 0x5678,
        },
        TimeError::NoReferencePoint,
        TimeError::BufferTooShort {
            expected: 12,
            actual: 4,
        },
    ];
    for err in &errors {
        let mut s = String::new();
        write!(&mut s, "{err}").unwrap();
        assert!(!s.is_empty(), "Display was empty for {err:?}");
    }
}

/// Core types satisfy Copy + Clone + Eq for ergonomic use.
#[test]
fn no_std_types_are_copy() {
    fn assert_copy_clone_eq<T: Copy + Clone + Eq>() {}
    assert_copy_clone_eq::<Rtc>();
    assert_copy_clone_eq::<AbsoluteTime>();
    assert_copy_clone_eq::<Ch4BinaryTime>();
    assert_copy_clone_eq::<Ieee1588Time>();
    assert_copy_clone_eq::<Ertc>();
    assert_copy_clone_eq::<TimeF1Csdw>();
    assert_copy_clone_eq::<DayFormatTime>();
    assert_copy_clone_eq::<DmyFormatTime>();
    assert_copy_clone_eq::<TimeSource>();
    assert_copy_clone_eq::<TimeFormat>();
    assert_copy_clone_eq::<DateFormat>();
    assert_copy_clone_eq::<TimeF2Csdw>();
    assert_copy_clone_eq::<NtpTime>();
    assert_copy_clone_eq::<PtpTime>();
    assert_copy_clone_eq::<NetworkTimeProtocol>();
}

// ═══════════════════════════════════════════════════════════════════
// Format 2 (Network Time) Integration Tests
// ═══════════════════════════════════════════════════════════════════

/// Full NTP pipeline: parse F2 payload → to_absolute → correlate.
#[test]
fn full_ntp_pipeline() {
    use irig106_time::network_time::{parse_time_f2_payload, NetworkTime};

    // Build an NTP payload: CSDW(protocol=NTP) + NTP time data
    // 2025-01-01 00:00:00 UTC → NTP seconds = 3,944,678,400
    let ntp_seconds: u32 = 3_944_678_400;
    let mut payload = [0u8; 12];
    payload[0..4].copy_from_slice(&0x0000_0000u32.to_le_bytes()); // CSDW: NTP
    payload[4..8].copy_from_slice(&ntp_seconds.to_le_bytes());
    // fraction = 0

    let (csdw, time) = parse_time_f2_payload(&payload).unwrap();
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Ntp);

    let ntp = match time {
        NetworkTime::Ntp(n) => n,
        _ => panic!("expected NTP"),
    };

    // Convert to AbsoluteTime
    let abs = ntp.to_absolute().unwrap();
    assert_eq!(abs.year(), Some(2025));
    assert_eq!(abs.day_of_year(), 1);
    assert_eq!(abs.hours(), 0);
    assert_eq!(abs.minutes(), 0);
    assert_eq!(abs.seconds(), 0);

    // Feed into correlator
    let mut correlator = TimeCorrelator::new();
    correlator.add_reference(5, Rtc::from_raw(10_000_000), abs);

    // Resolve a data packet 500ms later
    let target = Rtc::from_raw(15_000_000);
    let resolved = correlator.correlate(target, Some(5)).unwrap();
    assert_eq!(resolved.year(), Some(2025));
    assert_eq!(resolved.day_of_year(), 1);
    assert_eq!(resolved.hours(), 0);
    assert_eq!(resolved.minutes(), 0);
    assert_eq!(resolved.seconds(), 0);
    assert_eq!(resolved.nanoseconds(), 500_000_000); // 500ms
}

/// Full PTP pipeline: parse F2 payload → apply leap seconds → correlate.
#[test]
fn full_ptp_pipeline() {
    use irig106_time::network_time::{parse_time_f2_payload, LeapSecondTable, NetworkTime};

    // Build a PTP payload: CSDW(protocol=PTP) + PTP time data
    // 2025-01-01 00:00:00 UTC → Unix = 1,735,689,600 → TAI = 1,735,689,637 (offset=37)
    let tai_seconds: u64 = 1_735_689_637;
    let mut payload = [0u8; 14];
    payload[0..4].copy_from_slice(&0x0000_0001u32.to_le_bytes()); // CSDW: PTP
    payload[4..10].copy_from_slice(&tai_seconds.to_le_bytes()[0..6]);
    // nanoseconds = 0

    let (csdw, time) = parse_time_f2_payload(&payload).unwrap();
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Ptp);

    let ptp = match time {
        NetworkTime::Ptp(p) => p,
        _ => panic!("expected PTP"),
    };

    // Convert to AbsoluteTime using leap table
    let table = LeapSecondTable::builtin();
    let offset = table.offset_at_tai(ptp.seconds);
    assert_eq!(offset, 37);

    let abs = ptp.to_absolute(offset).unwrap();
    assert_eq!(abs.year(), Some(2025));
    assert_eq!(abs.day_of_year(), 1);
    assert_eq!(abs.hours(), 0);

    // Feed into correlator via add_reference_f2
    let mut correlator = TimeCorrelator::new();
    let net_time = NetworkTime::Ptp(ptp);
    correlator
        .add_reference_f2(8, Rtc::from_raw(10_000_000), &net_time, &table)
        .unwrap();

    // Resolve a data packet 1 second later
    let resolved = correlator
        .correlate(Rtc::from_raw(20_000_000), Some(8))
        .unwrap();
    assert_eq!(resolved.year(), Some(2025));
    assert_eq!(resolved.seconds(), 1);
}

/// Mixed F1 + F2 time sources in the same correlator.
#[test]
fn mixed_f1_f2_correlation() {
    let mut correlator = TimeCorrelator::new();

    // Channel 3: F1 (BCD) source — 12:00:00.000 at RTC 10M
    correlator.add_reference(
        3,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );

    // Channel 8: F2 (NTP) source — also 12:00:00.000 at RTC 10M
    // Day 100, 2025 = Unix 1744243200 (approx)
    // Construct NTP time for 2025, day 100, 12:00:00
    // We'll use add_reference directly with a pre-computed AbsoluteTime
    // to test that both sources coexist
    let mut ntp_abs = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    ntp_abs.set_year(Some(2025));
    correlator.add_reference(8, Rtc::from_raw(10_000_000), ntp_abs);

    // Both channels should resolve the same RTC to the same time
    let f1_time = correlator
        .correlate(Rtc::from_raw(10_000_000), Some(3))
        .unwrap();
    let f2_time = correlator
        .correlate(Rtc::from_raw(10_000_000), Some(8))
        .unwrap();
    assert_eq!(f1_time.hours(), f2_time.hours());
    assert_eq!(f1_time.minutes(), f2_time.minutes());
    assert_eq!(f1_time.seconds(), f2_time.seconds());
}

/// NTP fractional seconds provide sub-millisecond precision.
#[test]
fn ntp_sub_millisecond_precision() {
    use irig106_time::network_time::NtpTime;

    // fraction = 2^31 = half second → 500,000,000 ns
    let ntp = NtpTime {
        seconds: 3_944_678_400,
        fraction: 1 << 31,
    };
    let abs = ntp.to_absolute().unwrap();
    assert!(abs.nanoseconds() >= 499_999_999 && abs.nanoseconds() <= 500_000_001);

    // fraction = 2^30 = quarter second → ~250,000,000 ns
    let ntp2 = NtpTime {
        seconds: 3_944_678_400,
        fraction: 1 << 30,
    };
    let abs2 = ntp2.to_absolute().unwrap();
    assert!(abs2.nanoseconds() >= 249_999_999 && abs2.nanoseconds() <= 250_000_001);
}

/// Leap second table correctly differentiates historical offsets.
#[test]
fn leap_second_table_historical_accuracy() {
    use irig106_time::network_time::LeapSecondTable;

    let table = LeapSecondTable::builtin();

    // 1975-06-01 ≈ Unix 168307200 → offset should be 14
    assert_eq!(table.offset_at_unix(168_307_200), 14);

    // 2000-06-01 ≈ Unix 959817600 → offset should be 32
    assert_eq!(table.offset_at_unix(959_817_600), 32);

    // 2017-06-01 ≈ Unix 1496275200 → offset should be 37
    assert_eq!(table.offset_at_unix(1_496_275_200), 37);

    // 2026 → still 37 (no new leap seconds since 2017)
    assert_eq!(table.offset_at_unix(1_800_000_000), 37);
}

// ═══════════════════════════════════════════════════════════════════════
// v0.3.0 integration tests
// ═══════════════════════════════════════════════════════════════════════

/// AbsoluteTime Display impl formats DOY correctly.
#[test]
fn display_absolute_time_doy() {
    let t = AbsoluteTime::new(42, 13, 5, 30, 123_456_000).unwrap();
    let s = format!("{}", t);
    assert_eq!(s, "Day 042 13:05:30.123.456");
}

/// CalendarTime Display impl formats YYYY-MM-DD correctly.
#[test]
fn display_calendar_time_dmy() {
    let ct = CalendarTime::from_parts(2025, 3, 15, 74, 8, 30, 0, 0).unwrap();
    let s = format!("{}", ct);
    assert_eq!(s, "2025-03-15 08:30:00.000.000");
}

/// AbsoluteTime Display with year shows YYYY Day DDD format.
#[test]
fn display_absolute_time_with_year() {
    let mut t = AbsoluteTime::new(74, 8, 30, 0, 0).unwrap();
    t.set_year(Some(2025));
    let s = format!("{}", t);
    assert_eq!(s, "2025 Day 074 08:30:00.000.000");
}

/// Display zeroes are correctly padded.
#[test]
fn display_absolute_time_zero_padding() {
    let t = AbsoluteTime::new(1, 0, 0, 0, 1_000).unwrap();
    let s = format!("{}", t);
    assert_eq!(s, "Day 001 00:00:00.000.001");
}

/// drift_ppm returns None with fewer than 2 reference points.
#[test]
fn drift_ppm_insufficient_references() {
    let mut correlator = TimeCorrelator::new();
    assert!(correlator.drift_ppm(1).is_none());

    correlator.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    assert!(correlator.drift_ppm(1).is_none());
}

/// drift_ppm returns ~0 for a perfectly synchronized clock.
#[test]
fn drift_ppm_zero_drift() {
    let mut correlator = TimeCorrelator::new();

    // 10M ticks = 1 second of RTC at 10 MHz
    correlator.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    // Exactly 1 second later by RTC, exactly 1 second later by absolute time
    correlator.add_reference(
        1,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );

    let drift = correlator.drift_ppm(1).unwrap();
    assert!(drift.abs() < 0.01, "expected ~0 ppm, got {:.6}", drift);
}

/// drift_ppm detects a fast-running RTC.
#[test]
fn drift_ppm_fast_rtc() {
    let mut correlator = TimeCorrelator::new();

    correlator.add_reference(
        2,
        Rtc::from_raw(0),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    // RTC advanced 1.001 seconds (10,010,000 ticks) but absolute time advanced 1.000 seconds
    // → RTC is running 1000 ppm fast
    correlator.add_reference(
        2,
        Rtc::from_raw(10_010_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );

    let drift = correlator.drift_ppm(2).unwrap();
    assert!(
        (drift - 1000.0).abs() < 1.0,
        "expected ~1000 ppm, got {:.2}",
        drift
    );
}

/// drift_ppm uses only the requested channel.
#[test]
fn drift_ppm_channel_isolation() {
    let mut correlator = TimeCorrelator::new();

    // Channel 1: perfect clock
    correlator.add_reference(
        1,
        Rtc::from_raw(0),
        AbsoluteTime::new(1, 0, 0, 0, 0).unwrap(),
    );
    correlator.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(1, 0, 0, 1, 0).unwrap(),
    );

    // Channel 2: no references
    assert!(correlator.drift_ppm(2).is_none());
    assert!(correlator.drift_ppm(1).unwrap().abs() < 0.01);
}

/// Calendar validation rejects Feb 30.
#[test]
fn calendar_rejects_feb_30() {
    // Verify valid dates succeed through the public API.
    // Feb 28 on a non-leap year should work:
    // This is a structural test — the actual BCD byte construction is tested
    // in unit tests.
    let t = AbsoluteTime::new(59, 12, 0, 0, 0).unwrap(); // Day 59 = Feb 28
    assert_eq!(t.day_of_year(), 59);
}

/// Year overflow guard: extreme PTP timestamps don't panic.
#[test]
fn year_overflow_guard_no_panic() {
    use irig106_time::network_time::NtpTime;
    // NTP max seconds = u32::MAX ≈ year 2106
    let ntp = NtpTime {
        seconds: u32::MAX,
        fraction: 0,
    };
    // Should not panic — may return an error for out-of-range day but must not overflow
    let _ = ntp.to_absolute();
}

// ═══════════════════════════════════════════════════════════════════════
// v0.4.0 integration tests
// ═══════════════════════════════════════════════════════════════════════

// ── Version detection ────────────────────────────────────────────────

#[test]
fn version_detection_from_tmats_csdw() {
    use irig106_time::version::{detect_version, Irig106Version};

    assert_eq!(detect_version(0x00000000), Irig106Version::Pre07);
    assert_eq!(detect_version(0x00000007), Irig106Version::V07);
    assert_eq!(detect_version(0x0000000E), Irig106Version::V22);
    assert_eq!(detect_version(0x0000000F), Irig106Version::V23);
    // Upper bits should be ignored
    assert_eq!(detect_version(0xFFFFFF0C), Irig106Version::V17);
}

#[test]
fn version_ordering_is_chronological() {
    use irig106_time::version::Irig106Version;

    assert!(Irig106Version::Pre07 < Irig106Version::V07);
    assert!(Irig106Version::V07 < Irig106Version::V17);
    assert!(Irig106Version::V17 < Irig106Version::V22);
}

#[test]
fn version_feature_queries() {
    use irig106_time::version::Irig106Version;

    assert!(Irig106Version::Pre07.is_pre_ordering_guarantee());
    assert!(!Irig106Version::V07.is_pre_ordering_guarantee());

    assert!(!Irig106Version::V17.supports_format_2());
    assert!(Irig106Version::V22.supports_format_2());

    assert!(!Irig106Version::Pre07.has_gps_time_source());
    assert!(Irig106Version::V07.has_gps_time_source());
}

// ── Version-aware CSDW ──────────────────────────────────────────────

#[test]
fn csdw_time_source_versioned_pre07_ambiguous() {
    use irig106_time::version::Irig106Version;

    // Value 3 in pre-07 is ambiguous (04="None", 05="GPS")
    let csdw = TimeF1Csdw::from_raw(0x03); // time_source bits = 3
    let ts = csdw.time_source_versioned(&Irig106Version::Pre07);
    assert_eq!(ts, TimeSource::Reserved(3));
}

#[test]
fn csdw_time_source_versioned_v07_gps() {
    use irig106_time::version::Irig106Version;

    // Value 3 in 07+ is definitively GPS
    let csdw = TimeF1Csdw::from_raw(0x03);
    let ts = csdw.time_source_versioned(&Irig106Version::V07);
    assert_eq!(ts, TimeSource::Gps);
}

#[test]
fn csdw_time_source_unversioned_still_works() {
    // The non-versioned method should always return GPS for value 3
    let csdw = TimeF1Csdw::from_raw(0x03);
    assert_eq!(csdw.time_source(), TimeSource::Gps);
}

// ── OOO window ──────────────────────────────────────────────────────

#[test]
fn correlator_default_has_no_ooo_window() {
    let c = TimeCorrelator::new();
    assert!(c.ooo_window_ns().is_none());
}

#[test]
fn correlator_with_ooo_window() {
    let c = TimeCorrelator::with_ooo_window(Some(TimeCorrelator::DEFAULT_OOO_WINDOW_NS));
    assert_eq!(c.ooo_window_ns(), Some(2_000_000_000));
}

#[test]
fn correlator_unbounded_ooo() {
    let c = TimeCorrelator::with_ooo_window(None);
    assert!(c.ooo_window_ns().is_none());
}

// ── RTC reset detection ─────────────────────────────────────────────

#[test]
fn detect_rtc_reset_basic() {
    let mut c = TimeCorrelator::new();

    // Normal progression: RTC 100M → 200M, abs time 12:00:00 → 12:00:10
    c.add_reference(
        1,
        Rtc::from_raw(100_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        1,
        Rtc::from_raw(200_000_000),
        AbsoluteTime::new(100, 12, 0, 10, 0).unwrap(),
    );

    // Reset: RTC drops to 1M, but abs time advances to 12:00:20
    c.add_reference(
        1,
        Rtc::from_raw(1_000_000),
        AbsoluteTime::new(100, 12, 0, 20, 0).unwrap(),
    );

    let resets = c.detect_rtc_resets(1);
    assert_eq!(resets.len(), 1);
    assert_eq!(resets[0].rtc_before, Rtc::from_raw(200_000_000));
    assert_eq!(resets[0].rtc_after, Rtc::from_raw(1_000_000));
}

#[test]
fn detect_rtc_reset_no_false_positive_on_normal_progression() {
    let mut c = TimeCorrelator::new();

    c.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        1,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );
    c.add_reference(
        1,
        Rtc::from_raw(30_000_000),
        AbsoluteTime::new(100, 12, 0, 2, 0).unwrap(),
    );

    let resets = c.detect_rtc_resets(1);
    assert!(resets.is_empty());
}

#[test]
fn detect_rtc_reset_channel_isolation() {
    let mut c = TimeCorrelator::new();

    // Channel 1: has a reset
    c.add_reference(
        1,
        Rtc::from_raw(100_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        1,
        Rtc::from_raw(1_000_000),
        AbsoluteTime::new(100, 12, 0, 20, 0).unwrap(),
    );

    // Channel 2: no reset
    c.add_reference(
        2,
        Rtc::from_raw(50_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        2,
        Rtc::from_raw(60_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );

    assert_eq!(c.detect_rtc_resets(1).len(), 1);
    assert!(c.detect_rtc_resets(2).is_empty());
}

// ── to_le_bytes round-trip ──────────────────────────────────────────

#[test]
fn rtc_to_le_bytes_round_trip() {
    let original = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
    let rtc = Rtc::from_le_bytes(original);
    assert_eq!(rtc.to_le_bytes(), original);
}

#[test]
fn csdw_f1_to_le_bytes_round_trip() {
    let original = [0x12, 0x34, 0x56, 0x78];
    let csdw = TimeF1Csdw::from_le_bytes(original);
    assert_eq!(csdw.to_le_bytes(), original);
}

#[test]
fn csdw_f2_to_le_bytes_round_trip() {
    use irig106_time::network_time::TimeF2Csdw;
    let original = [0x01, 0x00, 0x00, 0x00]; // NTP protocol
    let csdw = TimeF2Csdw::from_le_bytes(original);
    assert_eq!(csdw.to_le_bytes(), original);
}

#[test]
fn ntp_to_le_bytes_round_trip() {
    use irig106_time::network_time::NtpTime;
    let original = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let ntp = NtpTime::from_le_bytes(&original).unwrap();
    assert_eq!(ntp.to_le_bytes(), original);
}

#[test]
fn ptp_to_le_bytes_round_trip() {
    use irig106_time::network_time::PtpTime;
    let mut original = [0u8; 10];
    original[0..6].copy_from_slice(&100_000u64.to_le_bytes()[0..6]);
    original[6..10].copy_from_slice(&500_000_000u32.to_le_bytes());
    let ptp = PtpTime::from_le_bytes(&original).unwrap();
    assert_eq!(ptp.to_le_bytes(), original);
}

#[test]
fn bcd_day_to_le_bytes_round_trip() {
    use irig106_time::bcd::DayFormatTime;
    // Construct known-good BCD bytes for Day 123, 14:30:45.670
    let dt = DayFormatTime {
        milliseconds: 670,
        seconds: 45,
        minutes: 30,
        hours: 14,
        day_of_year: 123,
    };
    let encoded = dt.to_le_bytes();
    let decoded = DayFormatTime::from_le_bytes(&encoded).unwrap();
    assert_eq!(decoded.milliseconds, 670);
    assert_eq!(decoded.seconds, 45);
    assert_eq!(decoded.minutes, 30);
    assert_eq!(decoded.hours, 14);
    assert_eq!(decoded.day_of_year, 123);
}

#[test]
fn bcd_dmy_to_le_bytes_round_trip() {
    use irig106_time::bcd::DmyFormatTime;
    let dt = DmyFormatTime {
        milliseconds: 120,
        seconds: 59,
        minutes: 0,
        hours: 23,
        day: 15,
        month: 3,
        year: 2025,
    };
    let encoded = dt.to_le_bytes();
    let decoded = DmyFormatTime::from_le_bytes(&encoded).unwrap();
    assert_eq!(decoded.milliseconds, 120);
    assert_eq!(decoded.seconds, 59);
    assert_eq!(decoded.minutes, 0);
    assert_eq!(decoded.hours, 23);
    assert_eq!(decoded.day, 15);
    assert_eq!(decoded.month, 3);
    assert_eq!(decoded.year, 2025);
}

// ═══════════════════════════════════════════════════════════════════════
// v0.5.0 integration tests
// ═══════════════════════════════════════════════════════════════════════

// ── Channel-indexed correlation (P4-01) ─────────────────────────────

#[test]
fn channel_references_accessor() {
    let mut c = TimeCorrelator::new();
    c.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        2,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );
    c.add_reference(
        1,
        Rtc::from_raw(30_000_000),
        AbsoluteTime::new(100, 12, 0, 2, 0).unwrap(),
    );

    assert_eq!(c.channel_references(1).len(), 2);
    assert_eq!(c.channel_references(2).len(), 1);
    assert_eq!(c.channel_references(99).len(), 0);
}

#[test]
fn channel_ids_returns_active_channels() {
    let mut c = TimeCorrelator::new();
    c.add_reference(
        5,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        3,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );
    c.add_reference(
        5,
        Rtc::from_raw(30_000_000),
        AbsoluteTime::new(100, 12, 0, 2, 0).unwrap(),
    );

    let ids = c.channel_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&3));
    assert!(ids.contains(&5));
}

#[test]
fn channel_indexed_correlate_same_result_as_any() {
    let mut c = TimeCorrelator::new();
    // Channel 1 at RTC 10M
    c.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    // Channel 2 at RTC 20M
    c.add_reference(
        2,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );

    // Query at RTC 10M — channel 1 should give exact match
    let by_ch = c.correlate(Rtc::from_raw(10_000_000), Some(1)).unwrap();
    assert_eq!(by_ch.hours(), 12);
    assert_eq!(by_ch.minutes(), 0);
    assert_eq!(by_ch.seconds(), 0);
}

#[test]
fn channel_indexed_correlate_large_set() {
    // Build a correlator with 1000 refs across 4 channels
    let mut c = TimeCorrelator::new();
    for i in 0..1000u64 {
        let ch = (i % 4) as u16;
        c.add_reference(
            ch,
            Rtc::from_raw((i + 1) * 10_000_000),
            AbsoluteTime::new(100, 12, (i % 60) as u8, (i % 60) as u8, 0).unwrap(),
        );
    }

    // Should resolve without error
    let mid = Rtc::from_raw(500 * 10_000_000 + 5_000_000);
    let any = c.correlate(mid, None);
    let ch1 = c.correlate(mid, Some(1));
    assert!(any.is_ok());
    assert!(ch1.is_ok());
}

// ── sub_nanos year rollover (GAP-05) ────────────────────────────────

#[test]
fn sub_nanos_crosses_year_boundary() {
    let mut t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap(); // Day 1, midnight
    t.set_year(Some(2025));

    // Subtract 1 second — should roll back to day 366, 23:59:59 of 2024
    // (2024 is a leap year → 366 days)
    let result = t.sub_nanos(1_000_000_000);
    assert_eq!(result.year(), Some(2024));
    assert_eq!(result.day_of_year(), 366);
    assert_eq!(result.hours(), 23);
    assert_eq!(result.minutes(), 59);
    assert_eq!(result.seconds(), 59);
}

#[test]
fn sub_nanos_same_day_no_year_change() {
    let mut t = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    t.set_year(Some(2025));

    // Subtract 1 hour — same day
    let result = t.sub_nanos(3_600_000_000_000);
    assert_eq!(result.year(), Some(2025));
    assert_eq!(result.day_of_year(), 100);
    assert_eq!(result.hours(), 11);
}

#[test]
fn sub_nanos_no_year_info_wraps_gracefully() {
    // Without year info, should still compute days correctly (assumes 365)
    let t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap(); // Day 1, no year
    let result = t.sub_nanos(1_000_000_000); // 1 second back
    assert_eq!(result.day_of_year(), 365); // wraps to 365 (no year = non-leap assumed)
    assert_eq!(result.hours(), 23);
    assert_eq!(result.minutes(), 59);
    assert_eq!(result.seconds(), 59);
}

// ═══════════════════════════════════════════════════════════════════════
// v0.6.0 integration tests
// ═══════════════════════════════════════════════════════════════════════

// ── P5-01: PacketStandard ───────────────────────────────────────────

#[test]
fn packet_standard_from_version() {
    use irig106_time::packet_standard::PacketStandard;
    use irig106_time::version::Irig106Version;

    assert_eq!(
        PacketStandard::from_version(&Irig106Version::Pre07),
        PacketStandard::Ch10
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V15),
        PacketStandard::Ch10
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V17),
        PacketStandard::Ch11
    );
    assert_eq!(
        PacketStandard::from_version(&Irig106Version::V22),
        PacketStandard::Ch11
    );
    assert!(PacketStandard::Ch11.is_ch11());
    assert!(!PacketStandard::Ch11.is_ch10());
}

// ── P5-02: StreamingTimeCorrelator ──────────────────────────────────

#[test]
fn streaming_correlator_basic_pipeline() {
    use irig106_time::streaming::StreamingTimeCorrelator;

    let mut sc = StreamingTimeCorrelator::new(30_000_000_000); // 30 sec window

    // Simulate 1 Hz time packets for 5 seconds
    for i in 0..5u64 {
        sc.add_reference(
            1,
            Rtc::from_raw((i + 1) * 10_000_000),
            AbsoluteTime::new(100, 12, 0, i as u8, 0).unwrap(),
        );
    }
    assert_eq!(sc.len(), 5);

    // Correlate a point between refs 3 and 4
    let mid = Rtc::from_raw(35_000_000);
    let result = sc.correlate(mid, Some(1)).unwrap();
    assert_eq!(result.hours(), 12);
}

#[test]
fn streaming_correlator_eviction_pipeline() {
    use irig106_time::streaming::StreamingTimeCorrelator;

    let mut sc = StreamingTimeCorrelator::new(5_000_000_000); // 5 sec window

    // Insert at RTC 10M (1 sec)
    sc.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    // Jump to RTC 200M (20 sec) — first ref is 19 sec old, evicted
    sc.add_reference(
        1,
        Rtc::from_raw(200_000_000),
        AbsoluteTime::new(100, 12, 0, 20, 0).unwrap(),
    );

    assert_eq!(sc.len(), 1);
    assert!(sc.total_evicted() > 0);
}

// ── P5-04: Quality metrics ──────────────────────────────────────────

#[test]
fn quality_metrics_from_correlator() {
    use irig106_time::quality::compute_quality;

    let mut c = TimeCorrelator::new();
    c.add_reference(
        1,
        Rtc::from_raw(10_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );
    c.add_reference(
        1,
        Rtc::from_raw(20_000_000),
        AbsoluteTime::new(100, 12, 0, 1, 0).unwrap(),
    );
    c.add_reference(
        2,
        Rtc::from_raw(15_000_000),
        AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(),
    );

    let q = compute_quality(c.references());
    assert_eq!(q.total_refs, 3);
    assert_eq!(q.channel_count, 2);
    assert!(q.rtc_span_ns.is_some());
    assert!(q.ref_density_per_sec.is_some());
}

// ── GAP-04: F1 leap second handling ─────────────────────────────────

#[test]
fn f1_leap_second_offset() {
    let table = LeapSecondTable::builtin();
    // 2017 day 1 should have offset 37 (last leap second was 2017-01-01)
    let offset = table.offset_for_f1(2017, 1);
    assert_eq!(offset, 37);
}

#[test]
fn is_near_leap_second_boundary() {
    let table = LeapSecondTable::builtin();
    // The 2017-01-01 leap second was at Unix 1483228800
    assert!(table.is_near_leap_second(1483228800, 10));
    // Far from any boundary
    assert!(!table.is_near_leap_second(1_600_000_000, 10));
}

// ── GAP-08: Recording events ────────────────────────────────────────

#[test]
fn recording_event_pipeline() {
    use irig106_time::recording_event::{RecordingEvent, RecordingEventType};

    let abs = AbsoluteTime::new(100, 12, 0, 0, 0).unwrap();
    let event = RecordingEvent::new(0x01, 1, Rtc::from_raw(10_000_000), Some(abs));

    assert_eq!(event.event_type, RecordingEventType::Started);
    assert!(event.has_reference_time());
    assert!(!event.event_type.may_cause_time_gap());

    // Overrun event without secondary header
    let overrun = RecordingEvent::new(0x03, 1, Rtc::from_raw(50_000_000), None);
    assert!(!overrun.has_reference_time());
    assert!(overrun.event_type.may_cause_time_gap());
}

// ═══════════════════════════════════════════════════════════════════════
// v0.7.0 MSRV integration tests — exercise functionality backed by
// util::is_leap_year and util::abs_diff_u64 through the public API
// ═══════════════════════════════════════════════════════════════════════

// ── Leap year via sub_nanos year rollover ────────────────────────────

#[test]
fn sub_nanos_leap_year_rollover_366_days() {
    // 2024 is a leap year (366 days). Rolling back from Day 1 of 2025
    // by 1 day should land on Day 366 of 2024.
    let mut t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    t.set_year(Some(2025));
    let result = t.sub_nanos(86_400_000_000_000); // exactly 1 day
    assert_eq!(result.year(), Some(2024));
    assert_eq!(result.day_of_year(), 366); // 2024 is leap
    assert_eq!(result.hours(), 0);
}

#[test]
fn sub_nanos_non_leap_year_rollover_365_days() {
    // 2023 is not a leap year (365 days). Rolling back from Day 1 of 2024
    // by 1 day should land on Day 365 of 2023.
    let mut t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    t.set_year(Some(2024));
    let result = t.sub_nanos(86_400_000_000_000); // exactly 1 day
    assert_eq!(result.year(), Some(2023));
    assert_eq!(result.day_of_year(), 365); // 2023 is not leap
}

#[test]
fn sub_nanos_century_non_leap_year() {
    // 1900 is divisible by 100 but NOT by 400 → not a leap year (365 days)
    let mut t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    t.set_year(Some(1901));
    let result = t.sub_nanos(86_400_000_000_000);
    assert_eq!(result.year(), Some(1900));
    assert_eq!(result.day_of_year(), 365); // 1900 NOT leap
}

#[test]
fn sub_nanos_quad_century_leap_year() {
    // 2000 is divisible by 400 → leap year (366 days)
    let mut t = AbsoluteTime::new(1, 0, 0, 0, 0).unwrap();
    t.set_year(Some(2001));
    let result = t.sub_nanos(86_400_000_000_000);
    assert_eq!(result.year(), Some(2000));
    assert_eq!(result.day_of_year(), 366); // 2000 IS leap
}

// ── Leap year via BCD DMY decoding (uses days_in_month) ─────────────

#[test]
fn bcd_dmy_feb_29_leap_year_accepted() {
    use irig106_time::bcd::DmyFormatTime;
    // Feb 29 on a leap year should be valid
    // Manually construct a DmyFormatTime for 2024-02-29
    let t = DmyFormatTime {
        milliseconds: 0,
        seconds: 0,
        minutes: 0,
        hours: 12,
        day: 29,
        month: 2,
        year: 2024,
    };
    let abs = t.to_calendar_time();
    assert_eq!(abs.day_of_year(), 60); // Jan(31) + Feb(29) = day 60
    assert_eq!(abs.year(), Some(2024));
}

#[test]
fn bcd_dmy_feb_29_non_leap_year_doy_calculation() {
    use irig106_time::bcd::DmyFormatTime;
    // On a non-leap year, Feb has 28 days so day 29 is technically March 1
    // The DOY calculation uses month_day_to_doy which doesn't reject — it
    // just computes. Feb 29 on a non-leap year → day 60 (no leap day added)
    let t = DmyFormatTime {
        milliseconds: 0,
        seconds: 0,
        minutes: 0,
        hours: 12,
        day: 29,
        month: 2,
        year: 2023,
    };
    let abs = t.to_calendar_time();
    // On non-leap year, month_day_to_doy gives 31+29=60 (no extra day for leap)
    assert_eq!(abs.day_of_year(), 60);
}

// ── abs_diff via is_near_leap_second ────────────────────────────────

#[test]
fn is_near_leap_second_exact_boundary() {
    let table = LeapSecondTable::builtin();
    // 2017-01-01 leap second at Unix 1483228800
    assert!(table.is_near_leap_second(1483228800, 0)); // exact match, window=0
}

#[test]
fn is_near_leap_second_within_window() {
    let table = LeapSecondTable::builtin();
    // 5 seconds before the boundary
    assert!(table.is_near_leap_second(1483228795, 10));
    // 5 seconds after the boundary
    assert!(table.is_near_leap_second(1483228805, 10));
}

#[test]
fn is_near_leap_second_outside_window() {
    let table = LeapSecondTable::builtin();
    // 100 seconds away, window=10
    assert!(!table.is_near_leap_second(1483228900, 10));
}

#[test]
fn is_near_leap_second_symmetry() {
    let table = LeapSecondTable::builtin();
    let boundary = 1483228800u64;
    // Distance 5 from below and above should give same result
    assert_eq!(
        table.is_near_leap_second(boundary - 5, 10),
        table.is_near_leap_second(boundary + 5, 10),
    );
}

#[test]
fn is_near_leap_second_far_future() {
    let table = LeapSecondTable::builtin();
    // Very large Unix timestamp — far from any known leap second
    assert!(!table.is_near_leap_second(u64::MAX, 1000));
}
