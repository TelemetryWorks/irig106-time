//! Unit tests for the `csdw` module.
//!
//! # Test Documentation
//!
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `time_source_internal` | Source=0 → Internal | L3-CSDW-004, L3-CSDW-008 |
//! | `time_source_external` | Source=1 → External | L3-CSDW-004, L3-CSDW-008 |
//! | `time_source_gps` | Source=3 → Gps | L3-CSDW-004, L3-CSDW-008 |
//! | `time_source_none` | Source=0xF → None | L3-CSDW-004, L3-CSDW-008 |
//! | `time_source_reserved` | Source=7 → Reserved(7) | L3-CSDW-004, L3-CSDW-008 |
//! | `time_format_irig_b` | Format=0 → IrigB | L3-CSDW-005, L3-CSDW-009 |
//! | `time_format_gps` | Format=5 → Gps | L3-CSDW-005, L3-CSDW-009 |
//! | `time_format_reserved` | Format=9 → Reserved(9) | L3-CSDW-005, L3-CSDW-009 |
//! | `leap_year_set` | Bit 8 = 1 → true | L3-CSDW-006 |
//! | `leap_year_clear` | Bit 8 = 0 → false | L3-CSDW-006 |
//! | `date_format_doy` | Bit 9 = 0 → DayOfYear | L3-CSDW-007 |
//! | `date_format_dmy` | Bit 9 = 1 → DayMonthYear | L3-CSDW-007 |
//! | `from_le_bytes_round_trip` | LE byte parse matches raw | L3-CSDW-003 |
//! | `combined_fields` | Multiple fields decoded from one CSDW | L3-CSDW-001..010 |

use super::*;

#[test]
fn time_source_internal() {
    let csdw = TimeF1Csdw::from_raw(0x00);
    assert_eq!(csdw.time_source(), TimeSource::Internal);
}

#[test]
fn time_source_external() {
    let csdw = TimeF1Csdw::from_raw(0x01);
    assert_eq!(csdw.time_source(), TimeSource::External);
}

#[test]
fn time_source_gps() {
    let csdw = TimeF1Csdw::from_raw(0x03);
    assert_eq!(csdw.time_source(), TimeSource::Gps);
}

#[test]
fn time_source_none() {
    let csdw = TimeF1Csdw::from_raw(0x0F);
    assert_eq!(csdw.time_source(), TimeSource::None);
}

#[test]
fn time_source_reserved() {
    let csdw = TimeF1Csdw::from_raw(0x07);
    assert_eq!(csdw.time_source(), TimeSource::Reserved(7));
}

#[test]
fn time_format_irig_b() {
    let csdw = TimeF1Csdw::from_raw(0x00); // bits [7:4] = 0
    assert_eq!(csdw.time_format(), TimeFormat::IrigB);
}

#[test]
fn time_format_gps() {
    let csdw = TimeF1Csdw::from_raw(0x50); // bits [7:4] = 5
    assert_eq!(csdw.time_format(), TimeFormat::Gps);
}

#[test]
fn time_format_reserved() {
    let csdw = TimeF1Csdw::from_raw(0x90); // bits [7:4] = 9
    assert_eq!(csdw.time_format(), TimeFormat::Reserved(9));
}

#[test]
fn leap_year_set() {
    let csdw = TimeF1Csdw::from_raw(1 << 8);
    assert!(csdw.is_leap_year());
}

#[test]
fn leap_year_clear() {
    let csdw = TimeF1Csdw::from_raw(0);
    assert!(!csdw.is_leap_year());
}

#[test]
fn date_format_doy() {
    let csdw = TimeF1Csdw::from_raw(0);
    assert_eq!(csdw.date_format(), DateFormat::DayOfYear);
}

#[test]
fn date_format_dmy() {
    let csdw = TimeF1Csdw::from_raw(1 << 9);
    assert_eq!(csdw.date_format(), DateFormat::DayMonthYear);
}

#[test]
fn from_le_bytes_round_trip() {
    let raw: u32 = 0x0000_0351; // Source=1(External), Format=5(GPS), Leap=1, DateFmt=1(DMY)
    let bytes = raw.to_le_bytes();
    let csdw = TimeF1Csdw::from_le_bytes(bytes);
    assert_eq!(csdw.as_raw(), raw);
}

#[test]
fn combined_fields() {
    // Build: Source=3(GPS), Format=0(IrigB), LeapYear=1, DateFormat=0(DOY)
    // Bits: [3:0]=0b0011, [7:4]=0b0000, [8]=1, [9]=0
    // = 0b0_1_0000_0011 = 0x103
    let csdw = TimeF1Csdw::from_raw(0x103);
    assert_eq!(csdw.time_source(), TimeSource::Gps);
    assert_eq!(csdw.time_format(), TimeFormat::IrigB);
    assert!(csdw.is_leap_year());
    assert_eq!(csdw.date_format(), DateFormat::DayOfYear);
}
