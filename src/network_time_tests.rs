//! Unit tests for the `network_time` module (Time Data Format 2, 0x12).
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `f2_csdw_ntp` | CSDW protocol=0 → NTP | L1-F2CSDW-002 |
//! | `f2_csdw_ptp` | CSDW protocol=1 → PTP | L1-F2CSDW-002 |
//! | `f2_csdw_reserved` | CSDW protocol=5 → Reserved | L1-F2CSDW-002 |
//! | `f2_csdw_reserved_bits_clean` | Reserved bits zero → Ok | L1-F2CSDW-003 |
//! | `f2_csdw_reserved_bits_dirty` | Reserved bits set → Err | L1-F2CSDW-003 |
//! | `ntp_from_le_bytes` | Parse NTP from buffer | L1-NTP-001 |
//! | `ntp_fraction_to_nanos` | Fractional → nanoseconds | L1-NTP-003 |
//! | `ntp_to_unix_seconds` | NTP → Unix epoch conversion | L1-NTP-004 |
//! | `ntp_before_unix_epoch` | Pre-1970 NTP → None | L1-NTP-004 |
//! | `ntp_to_absolute` | NTP → AbsoluteTime | L1-NTP-001..004 |
//! | `ntp_buffer_too_short` | <8 bytes rejected | L1-NTP-001 |
//! | `ptp_from_le_bytes` | Parse PTP from buffer | L1-PTP-001 |
//! | `ptp_nanos_overflow` | nanos >= 1B rejected | L1-PTP-003 |
//! | `ptp_to_utc_seconds` | TAI → UTC with offset | L1-PTP-004 |
//! | `ptp_to_absolute` | PTP → AbsoluteTime | L1-PTP-001..004 |
//! | `ptp_buffer_too_short` | <10 bytes rejected | L1-PTP-001 |
//! | `parse_f2_ntp_payload` | Full payload parse (NTP) | L1-F2CSDW-001, L1-NTP-001 |
//! | `parse_f2_ptp_payload` | Full payload parse (PTP) | L1-F2CSDW-001, L1-PTP-001 |
//! | `leap_second_table_builtin` | Built-in table has entries | L1-TAI-002 |
//! | `leap_second_table_lookup_2024` | Offset at 2024 = 37 | L1-TAI-001 |
//! | `leap_second_table_lookup_1980` | Offset at 1980 = 19 | L1-TAI-001 |
//! | `leap_second_table_custom` | Custom table add + lookup | L1-TAI-003 |

use super::*;

// ── F2 CSDW ──────────────────────────────────────────────────────────

#[test]
fn f2_csdw_ntp() {
    let csdw = TimeF2Csdw::from_raw(0x0000_0000);
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Ntp);
}

#[test]
fn f2_csdw_ptp() {
    let csdw = TimeF2Csdw::from_raw(0x0000_0001);
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Ptp);
}

#[test]
fn f2_csdw_reserved() {
    let csdw = TimeF2Csdw::from_raw(0x0000_0005);
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Reserved(5));
}

#[test]
fn f2_csdw_reserved_bits_clean() {
    let csdw = TimeF2Csdw::from_raw(0x0000_0001); // Only protocol bits set
    assert!(csdw.validate_reserved().is_ok());
}

#[test]
fn f2_csdw_reserved_bits_dirty() {
    let csdw = TimeF2Csdw::from_raw(0x0000_0010); // Bit 4 set (reserved)
    assert!(csdw.validate_reserved().is_err());
}

// ── NTP Time ─────────────────────────────────────────────────────────

#[test]
fn ntp_from_le_bytes() {
    // 1_000_000 seconds, 0 fraction
    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&1_000_000u32.to_le_bytes());
    let ntp = NtpTime::from_le_bytes(&buf).unwrap();
    assert_eq!(ntp.seconds, 1_000_000);
    assert_eq!(ntp.fraction, 0);
}

#[test]
fn ntp_fraction_to_nanos() {
    // Half a second: fraction = 2^31 = 2_147_483_648
    let ntp = NtpTime {
        seconds: 0,
        fraction: 2_147_483_648,
    };
    let nanos = ntp.fraction_as_nanos();
    // Should be ~500_000_000 (500 ms)
    assert!((499_999_999..=500_000_001).contains(&nanos),
        "expected ~500_000_000, got {nanos}");
}

#[test]
fn ntp_to_unix_seconds() {
    // NTP seconds for Unix epoch: 2_208_988_800
    let ntp = NtpTime {
        seconds: NTP_UNIX_EPOCH_OFFSET as u32 + 1000,
        fraction: 0,
    };
    assert_eq!(ntp.to_unix_seconds(), Some(1000));
}

#[test]
fn ntp_before_unix_epoch() {
    let ntp = NtpTime {
        seconds: 100, // Well before 1970
        fraction: 0,
    };
    assert_eq!(ntp.to_unix_seconds(), None);
}

#[test]
fn ntp_to_absolute() {
    // 2025-01-01 00:00:00 UTC
    // Unix timestamp: 1_735_689_600
    // NTP timestamp: 1_735_689_600 + 2_208_988_800 = 3_944_678_400
    let ntp = NtpTime {
        seconds: 3_944_678_400,
        fraction: 0,
    };
    let abs = ntp.to_absolute().unwrap();
    assert_eq!(abs.year, Some(2025));
    assert_eq!(abs.day_of_year, 1);
    assert_eq!(abs.hours, 0);
    assert_eq!(abs.minutes, 0);
    assert_eq!(abs.seconds, 0);
}

#[test]
fn ntp_buffer_too_short() {
    let buf = [0u8; 6];
    assert!(NtpTime::from_le_bytes(&buf).is_err());
}

// ── PTP Time ─────────────────────────────────────────────────────────

#[test]
fn ptp_from_le_bytes() {
    // 1000 seconds, 500_000_000 nanoseconds
    let mut buf = [0u8; 10];
    buf[0..6].copy_from_slice(&1000u64.to_le_bytes()[0..6]);
    buf[6..10].copy_from_slice(&500_000_000u32.to_le_bytes());
    let ptp = PtpTime::from_le_bytes(&buf).unwrap();
    assert_eq!(ptp.seconds, 1000);
    assert_eq!(ptp.nanoseconds, 500_000_000);
}

#[test]
fn ptp_nanos_overflow() {
    let mut buf = [0u8; 10];
    buf[6..10].copy_from_slice(&1_000_000_000u32.to_le_bytes());
    assert!(PtpTime::from_le_bytes(&buf).is_err());
}

#[test]
fn ptp_to_utc_seconds() {
    let ptp = PtpTime {
        seconds: 1_000_000_037, // TAI
        nanoseconds: 0,
    };
    // TAI-UTC offset = 37 → UTC = TAI - 37
    let utc = ptp.to_utc_seconds(37);
    assert_eq!(utc, 1_000_000_000);
}

#[test]
fn ptp_to_absolute() {
    // 2025-01-01 00:00:00 UTC
    // Unix: 1_735_689_600
    // TAI: 1_735_689_600 + 37 = 1_735_689_637
    let ptp = PtpTime {
        seconds: 1_735_689_637,
        nanoseconds: 0,
    };
    let abs = ptp.to_absolute(37).unwrap();
    assert_eq!(abs.year, Some(2025));
    assert_eq!(abs.day_of_year, 1);
    assert_eq!(abs.hours, 0);
}

#[test]
fn ptp_buffer_too_short() {
    let buf = [0u8; 8];
    assert!(PtpTime::from_le_bytes(&buf).is_err());
}

// ── Full payload parse ───────────────────────────────────────────────

#[test]
fn parse_f2_ntp_payload() {
    let mut payload = [0u8; 12]; // 4 CSDW + 8 NTP
    // CSDW: protocol = NTP (0)
    payload[0..4].copy_from_slice(&0x0000_0000u32.to_le_bytes());
    // NTP: 3_944_678_400 seconds (2025-01-01), 0 fraction
    payload[4..8].copy_from_slice(&3_944_678_400u32.to_le_bytes());

    let (csdw, time) = parse_time_f2_payload(&payload).unwrap();
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Ntp);
    match time {
        NetworkTime::Ntp(ntp) => assert_eq!(ntp.seconds, 3_944_678_400),
        _ => panic!("expected NTP"),
    }
}

#[test]
fn parse_f2_ptp_payload() {
    let mut payload = [0u8; 14]; // 4 CSDW + 10 PTP
    // CSDW: protocol = PTP (1)
    payload[0..4].copy_from_slice(&0x0000_0001u32.to_le_bytes());
    // PTP: 1000 seconds, 500M nanoseconds
    payload[4..10].copy_from_slice(&1000u64.to_le_bytes()[0..6]);
    payload[10..14].copy_from_slice(&500_000_000u32.to_le_bytes());

    let (csdw, time) = parse_time_f2_payload(&payload).unwrap();
    assert_eq!(csdw.time_protocol(), NetworkTimeProtocol::Ptp);
    match time {
        NetworkTime::Ptp(ptp) => {
            assert_eq!(ptp.seconds, 1000);
            assert_eq!(ptp.nanoseconds, 500_000_000);
        }
        _ => panic!("expected PTP"),
    }
}

// ── Leap second table ────────────────────────────────────────────────

#[test]
fn leap_second_table_builtin() {
    let table = LeapSecondTable::builtin();
    assert!(table.len() >= 27); // At least from 1972 to 2017
}

#[test]
fn leap_second_table_lookup_2024() {
    let table = LeapSecondTable::builtin();
    // 2024-06-15 00:00:00 UTC = approx 1_718_409_600
    let offset = table.offset_at_unix(1_718_409_600);
    assert_eq!(offset, 37);
}

#[test]
fn leap_second_table_lookup_1980() {
    let table = LeapSecondTable::builtin();
    // 1980-06-01 = approx 328_665_600
    let offset = table.offset_at_unix(328_665_600);
    assert_eq!(offset, 19);
}

#[test]
fn leap_second_table_custom() {
    let mut table = LeapSecondTable::empty();
    table.add(LeapSecondEntry {
        effective_unix: 2_000_000_000,
        tai_utc_offset: 38,
    });
    assert_eq!(table.len(), 1);
    assert_eq!(table.offset_at_unix(2_500_000_000), 38);
}
