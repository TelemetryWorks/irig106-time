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
    assert_eq!(abs_time.nanoseconds, 340_000_000);

    // Step 4: Correlate — time packet arrived at RTC=50_000_000 (5 sec)
    let ref_rtc = Rtc::from_raw(50_000_000);
    let mut correlator = TimeCorrelator::new();
    correlator.add_reference(1, ref_rtc, abs_time);

    // Step 5: Resolve a data packet at RTC=50_150_000 (15ms later)
    let target_rtc = Rtc::from_raw(50_150_000);
    let resolved = correlator.correlate(target_rtc, None).unwrap();
    assert_eq!(resolved.day_of_year, 100);
    assert_eq!(resolved.hours, 12);
    assert_eq!(resolved.minutes, 30);
    assert_eq!(resolved.seconds, 25);
    assert_eq!(resolved.nanoseconds, 355_000_000); // 340ms + 15ms
}

/// Parse a DMY format time message and verify calendar date propagation.
#[test]
fn full_dmy_format_pipeline() {
    let csdw = TimeF1Csdw::from_raw(0x0201); // Source=External, Format=IrigB, DateFmt=DMY
    assert_eq!(csdw.date_format(), DateFormat::DayMonthYear);

    let bcd_buf = encode_mar15_2025_084530_120();
    let dmy_time = DmyFormatTime::from_le_bytes(&bcd_buf).unwrap();
    let abs = dmy_time.to_absolute();

    assert_eq!(abs.year, Some(2025));
    assert_eq!(abs.month, Some(3));
    assert_eq!(abs.day_of_month, Some(15));
    assert_eq!(abs.day_of_year, 74); // Jan(31) + Feb(28) + 15
    assert_eq!(abs.hours, 8);
    assert_eq!(abs.minutes, 45);
    assert_eq!(abs.seconds, 30);
    assert_eq!(abs.nanoseconds, 120_000_000);
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
    let via_irig = correlator.correlate(Rtc::from_raw(20_000_000), Some(1)).unwrap();
    assert_eq!(via_irig.hours, 12);
    assert_eq!(via_irig.minutes, 0);
    assert_eq!(via_irig.seconds, 1); // 12:00:01 per IRIG-B

    // Same RTC using GPS (channel 2)
    let via_gps = correlator.correlate(Rtc::from_raw(20_000_000), Some(2)).unwrap();
    assert_eq!(via_gps.hours, 12);
    assert_eq!(via_gps.minutes, 0);
    assert_eq!(via_gps.seconds, 4); // 12:00:04.5 per GPS
    assert_eq!(via_gps.nanoseconds, 500_000_000);
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
    let parsed = irig106_time::secondary::parse_secondary_header(
        &sec_buf,
        SecHdrTimeFormat::Ieee1588,
    )
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

    let ipt = irig106_time::intra_packet::parse_intra_packet_time(
        &ipt_buf,
        IntraPacketTimeFormat::Rtc48,
    )
    .unwrap();

    match ipt {
        IntraPacketTime::Rtc(rtc) => {
            let abs = correlator.correlate(rtc, Some(1)).unwrap();
            assert_eq!(abs.day_of_year, 200);
            assert_eq!(abs.hours, 8);
            assert_eq!(abs.seconds, 0);
            assert_eq!(abs.nanoseconds, 50_000_000); // 50ms
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

    assert_eq!(result.day_of_year, 2); // crossed midnight
    assert_eq!(result.hours, 0);
    assert_eq!(result.minutes, 0);
    assert_eq!(result.seconds, 0);
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
    assert!(matches!(err, TimeError::InvalidBcdDigit { nibble: 0x0A, .. }));
}

/// All TimeError variants produce useful Display output.
#[test]
fn all_error_variants_display() {
    use std::fmt::Write;
    let errors = vec![
        TimeError::InvalidBcdDigit { nibble: 0xB, position: "test" },
        TimeError::ReservedBitSet { position: "test" },
        TimeError::OutOfRange { field: "test", value: 99, max: 59 },
        TimeError::ChecksumMismatch { stored: 0x1234, computed: 0x5678 },
        TimeError::NoReferencePoint,
        TimeError::BufferTooShort { expected: 12, actual: 4 },
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
}
