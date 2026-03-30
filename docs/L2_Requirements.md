# L2 Requirements — irig106-time

**Document:** L2_Requirements.md
**Crate:** irig106-time
**Version:** 0.2.0
**Parent:** L1_Requirements.md
**Date:** 2026-03-27

---

## 1. Purpose

Level 2 (L2) requirements decompose each L1 requirement into testable functional behaviors.
Every L2 requirement traces upward to exactly one L1 requirement and downward to one or
more L3 design specifications and tests.

---

## 2. Conventions

L2 IDs use the pattern `L2-<DOMAIN>-<NNN>` and include a `Traces` column linking to the
parent L1 requirement.

---

## 3. L2 Requirements

### 3.1 Relative Time Counter (RTC)

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-RTC-001 | `Rtc` shall be a newtype wrapping a `u64` whose lower 48 bits hold the counter value. Upper 16 bits shall always be zero. | L1-RTC-001 |
| L2-RTC-002 | `Rtc::from_le_bytes([u8; 6])` shall construct an RTC from 6 little-endian bytes by zero-extending to `u64`. | L1-RTC-002 |
| L2-RTC-003 | `Rtc::from_raw(u64)` shall mask the input to 48 bits (`& 0x0000_FFFF_FFFF_FFFF`). | L1-RTC-001 |
| L2-RTC-004 | `Rtc::as_raw(&self) -> u64` shall return the raw 48-bit value. | L1-RTC-001 |
| L2-RTC-005 | `Rtc::elapsed_ticks(&self, later: &Rtc) -> u64` shall return `later.raw - self.raw` when `later >= self`, correctly handling 48-bit wrap-around. | L1-RTC-003 |
| L2-RTC-006 | `Rtc::elapsed_nanos(&self, later: &Rtc) -> u64` shall return `elapsed_ticks * 100`. | L1-RTC-003, L1-RTC-004 |
| L2-RTC-007 | `Rtc::to_nanos(&self) -> u64` shall return `raw * 100` (ticks since counter epoch). | L1-RTC-004 |
| L2-RTC-008 | `Rtc` shall implement `Ord` using the raw 48-bit value for comparison. | L1-RTC-001 |
| L2-RTC-009 | The maximum representable RTC value shall be `0x0000_FFFF_FFFF_FFFF` (2^48 − 1), representing approximately 325.4 days at 100 ns resolution. | L1-RTC-001 |
| L2-RTC-010 | `Rtc::ZERO` shall be a const representing tick count zero. | L1-RTC-001 |
| L2-RTC-011 | `Rtc::MAX` shall be a const representing the maximum 48-bit value. | L1-RTC-001 |

### 3.2 Absolute Time Representations

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-ABS-001 | `AbsoluteTime` shall represent a point in time with nanosecond precision using day-of-year (1–366), hours (0–23), minutes (0–59), seconds (0–59), and nanoseconds (0–999_999_999). It may optionally carry a year annotation (0–9999) as metadata; year does not imply calendar validation. | L1-ABS-001 |
| L2-ABS-002 | `CalendarTime` shall represent a calendar-validated point in time with year (0–9999), month (1–12), day-of-month (1–N, leap-aware), and all `AbsoluteTime` fields. Construction shall validate that day-of-month does not exceed the number of days in the given month/year, and that day-of-year is consistent with the calendar date. `CalendarTime` wraps `AbsoluteTime` and provides transparent access to its fields. | L1-ABS-001 |
| L2-ABS-003 | `Ch4BinaryTime` shall represent the Chapter 4 Binary Weighted Time with fields: `high_order: u16`, `low_order: u16`, `microseconds: u16`. | L1-ABS-002 |
| L2-ABS-004 | `Ch4BinaryTime::to_absolute(&self) -> AbsoluteTime` shall decode the BWT fields into day, hour, minute, second, and microsecond. | L1-ABS-002 |
| L2-ABS-005 | `Ieee1588Time` shall represent IEEE-1588 time with fields: `nanoseconds: u32`, `seconds: u32`. | L1-ABS-003 |
| L2-ABS-006 | `Ieee1588Time::to_nanos_since_epoch(&self) -> u64` shall return `seconds * 1_000_000_000 + nanoseconds`. | L1-ABS-003 |
| L2-ABS-007 | `Ertc` shall represent the 64-bit Extended RTC as a newtype wrapping `u64` with 100 ns resolution. | L1-ABS-004 |
| L2-ABS-008 | `Ertc::to_nanos(&self) -> u128` shall return `raw * 100`. | L1-ABS-004 |

### 3.3 Time Data Format 1 CSDW

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-CSDW-001 | `TimeF1Csdw` shall be parsed from a `u32` value read in little-endian order. | L1-CSDW-001 |
| L2-CSDW-002 | `TimeF1Csdw::time_source(&self) -> TimeSource` shall return an enum decoded from bits [3:0]. | L1-CSDW-002 |
| L2-CSDW-003 | The `TimeSource` enum shall include variants: `Internal`, `External`, `InternalRtc`, `Gps` (and `None`/`Reserved` as applicable per Table 10-11). | L1-CSDW-002 |
| L2-CSDW-004 | `TimeF1Csdw::time_format(&self) -> TimeFormat` shall return an enum decoded from bits [7:4]. | L1-CSDW-003 |
| L2-CSDW-005 | The `TimeFormat` enum shall include variants: `IrigB`, `IrigA`, `IrigG`, `Rtc`, `Utc`, `Gps`, and `Reserved`. | L1-CSDW-003 |
| L2-CSDW-006 | `TimeF1Csdw::is_leap_year(&self) -> bool` shall return bit 8. | L1-CSDW-004 |
| L2-CSDW-007 | `TimeF1Csdw::date_format(&self) -> DateFormat` shall decode bit 9: `DayOfYear` or `DayMonthYear`. | L1-CSDW-005 |
| L2-CSDW-008 | `TimeF1Csdw::from_raw(u32)` shall store the raw 32-bit value and extract fields via bitmask/shift. | L1-CSDW-001 |

### 3.4 BCD Time Message Decoding

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-BCD-001 | `DayFormatTime::from_le_bytes([u8; 8])` shall decode 4 little-endian `u16` words into BCD digit fields per Figure 5-13. | L1-BCD-001 |
| L2-BCD-002 | `DayFormatTime` shall expose fields: `milliseconds` (0–999), `seconds` (0–59), `minutes` (0–59), `hours` (0–23), `day_of_year` (1–366). | L1-BCD-001 |
| L2-BCD-003 | `DmyFormatTime::from_le_bytes([u8; 10])` shall decode 5 little-endian `u16` words into BCD digit fields per Figure 5-14. | L1-BCD-002 |
| L2-BCD-004 | `DmyFormatTime` shall expose fields: `milliseconds` (0–999), `seconds` (0–59), `minutes` (0–59), `hours` (0–23), `day` (1–31), `month` (1–12), `year` (0–9999). | L1-BCD-002 |
| L2-BCD-005 | Both `DayFormatTime` and `DmyFormatTime` shall return `Err(TimeError::ReservedBitSet)` if any reserved bit is non-zero. | L1-BCD-003 |
| L2-BCD-006 | Both formats shall return `Err(TimeError::InvalidBcdDigit)` if any 4-bit nibble exceeds 9. | L1-BCD-004 |
| L2-BCD-007 | `DayFormatTime::to_absolute(&self) -> AbsoluteTime` shall convert BCD fields to `AbsoluteTime`. | L1-BCD-001 |
| L2-BCD-008 | `DmyFormatTime::to_absolute(&self) -> AbsoluteTime` shall convert BCD fields to `AbsoluteTime` with full date. | L1-BCD-002 |
| L2-BCD-009 | The BCD decoding formula for a two-digit field shall be `tens * 10 + units`. | L1-BCD-001, L1-BCD-002 |
| L2-BCD-010 | Millisecond decoding shall use the formula `hundreds * 100 + tens * 10` (ms resolution from the standard's Hmn and Tmn fields). | L1-BCD-001 |

### 3.5 Secondary Header Time

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-SEC-001 | `SecondaryHeaderTime::from_le_bytes(buf: &[u8; 12], time_format: SecHdrTimeFormat)` shall dispatch to the correct parser based on the format discriminant. | L1-SEC-001, L1-SEC-002, L1-SEC-003 |
| L2-SEC-002 | For `SecHdrTimeFormat::Ch4`, bytes [2..8] shall be parsed as `Ch4BinaryTime` (high, low, microseconds). | L1-SEC-001 |
| L2-SEC-003 | For `SecHdrTimeFormat::Ieee1588`, bytes [0..8] shall be parsed as `Ieee1588Time` (nanoseconds, seconds). | L1-SEC-002 |
| L2-SEC-004 | For `SecHdrTimeFormat::Ertc`, bytes [0..8] shall be parsed as a 64-bit `Ertc`. | L1-SEC-003 |
| L2-SEC-005 | `validate_secondary_checksum(buf: &[u8; 12]) -> bool` shall compute a 16-bit sum over the first 10 bytes (5 words) and compare against the stored checksum at bytes [10..12]. | L1-SEC-004 |

### 3.6 Intra-Packet Time Stamps

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-IPT-001 | `IntraPacketTime::from_rtc_bytes([u8; 8])` shall extract a 48-bit RTC from bytes [0..6] and verify bytes [6..8] are zero (reserved). | L1-IPT-001 |
| L2-IPT-002 | `IntraPacketTime::from_ch4_bytes([u8; 8])` shall extract `Ch4BinaryTime` from the buffer. | L1-IPT-002 |
| L2-IPT-003 | `IntraPacketTime::from_ieee1588_bytes([u8; 8])` shall extract `Ieee1588Time` (nanoseconds, seconds). | L1-IPT-003 |
| L2-IPT-004 | `IntraPacketTime::from_ertc_bytes([u8; 8])` shall extract a 64-bit `Ertc`. | L1-IPT-004 |
| L2-IPT-005 | `IntraPacketTime` shall be an enum with variants `Rtc(Rtc)`, `Ch4(Ch4BinaryTime)`, `Ieee1588(Ieee1588Time)`, `Ertc(Ertc)`. | L1-IPT-001..L1-IPT-004 |

### 3.7 RTC-to-Absolute-Time Correlation

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-COR-001 | `TimeCorrelator` shall hold a set of reference points, each pairing an `Rtc` value with an `AbsoluteTime` and a `channel_id`. | L1-COR-001 |
| L2-COR-002 | `TimeCorrelator::add_reference(&mut self, channel_id: u16, rtc: Rtc, time: AbsoluteTime)` shall insert a reference point. | L1-COR-001 |
| L2-COR-003 | `TimeCorrelator::correlate(&self, rtc: Rtc, channel_id: Option<u16>) -> Result<AbsoluteTime, TimeError>` shall find the nearest reference point (by RTC) and interpolate. | L1-COR-002 |
| L2-COR-004 | Interpolation shall compute: `ref_time + ((target_rtc - ref_rtc) * 100 ns)` using the nearest reference point. | L1-COR-002 |
| L2-COR-005 | When `channel_id` is `Some(id)`, correlation shall use only reference points from that channel. | L1-COR-003 |
| L2-COR-006 | `TimeCorrelator::detect_time_jump(&self, channel_id: u16, threshold_ns: u64) -> Vec<TimeJump>` shall identify discontinuities where consecutive reference points on the same channel differ by more than the threshold in absolute time vs. expected RTC-based progression. | L1-COR-004 |
| L2-COR-007 | `TimeCorrelator` shall return `Err(TimeError::NoReferencePoint)` when correlation is attempted with no reference points (or none for the requested channel). | L1-COR-001, L1-ERR-001 |

### 3.8 Error Handling

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-ERR-001 | `TimeError` shall be an enum with variants: `InvalidBcdDigit { nibble: u8, position: &'static str }`, `ReservedBitSet { position: &'static str }`, `OutOfRange { field: &'static str, value: u32, max: u32 }`, `ChecksumMismatch { stored: u16, computed: u16 }`, `NoReferencePoint`, `BufferTooShort { expected: usize, actual: usize }`. | L1-ERR-001..L1-ERR-004 |
| L2-ERR-002 | All parsing functions shall return `Result<T, TimeError>`. | L1-ERR-001 |
| L2-ERR-003 | `TimeError` shall implement `core::fmt::Display` and, when `std` is enabled, `std::error::Error`. | L1-ERR-001, L1-API-001 |

### 3.9 API and Type Safety

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-API-001 | `Rtc`, `Ertc`, `Ch4BinaryTime`, `Ieee1588Time`, `AbsoluteTime`, `TimeF1Csdw`, `DayFormatTime`, `DmyFormatTime` shall derive or implement `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`. | L1-API-001 |
| L2-API-002 | The crate root shall declare `#![no_std]` and use `#[cfg(feature = "std")] extern crate std;` for optional std support. | L1-API-002 |
| L2-API-003 | The crate shall have zero required dependencies in `[dependencies]` of `Cargo.toml`. Only `[dev-dependencies]` may contain external crates. | L1-API-003 |
| L2-API-004 | `Rtc` shall not implement `From<u64>` implicitly; construction shall go through `Rtc::from_raw()` to enforce 48-bit masking. | L1-API-004 |

### 3.10 Time Data Format 2 CSDW

*Added in v0.2.0.*

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-F2CSDW-001 | `TimeF2Csdw` shall be parsed from a `u32` value read in little-endian order. | L1-F2CSDW-001 |
| L2-F2CSDW-002 | `TimeF2Csdw::time_protocol(&self) -> NetworkTimeProtocol` shall return an enum decoded from bits [3:0]. | L1-F2CSDW-002 |
| L2-F2CSDW-003 | `TimeF2Csdw::validate_reserved(&self) -> Result<()>` shall verify bits [31:4] are zero. | L1-F2CSDW-003 |
| L2-F2CSDW-004 | The `NetworkTimeProtocol` enum shall include variants `Ntp`, `Ptp`, and `Reserved(u8)`. | L1-F2CSDW-002 |

### 3.11 NTP Time

*Added in v0.2.0.*

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-NTP-001 | `NtpTime::from_le_bytes(&[u8]) -> Result<Self>` shall parse 8 bytes into seconds and fractional seconds. | L1-NTP-001 |
| L2-NTP-002 | `NtpTime::fraction_as_nanos(&self) -> u32` shall convert the fractional field to nanoseconds via `(fraction * 10^9) >> 32`. | L1-NTP-003 |
| L2-NTP-003 | `NtpTime::to_unix_seconds(&self) -> Option<u64>` shall subtract `NTP_UNIX_EPOCH_OFFSET` (2,208,988,800), returning `None` if underflow. | L1-NTP-004 |
| L2-NTP-004 | `NtpTime::to_absolute(&self) -> Result<AbsoluteTime>` shall convert NTP time to year/doy/time-of-day. | L1-NTP-001..004 |
| L2-NTP-005 | `NtpTime::to_nanos_since_ntp_epoch(&self) -> u64` shall return total nanoseconds since the NTP epoch. | L1-NTP-002 |

### 3.12 PTP Time

*Added in v0.2.0.*

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-PTP-001 | `PtpTime::from_le_bytes(&[u8]) -> Result<Self>` shall parse 10 bytes: 6-byte (48-bit) seconds and 4-byte nanoseconds. | L1-PTP-001 |
| L2-PTP-002 | `PtpTime::from_le_bytes` shall return `Err(OutOfRange)` if nanoseconds >= 1,000,000,000. | L1-PTP-003 |
| L2-PTP-003 | `PtpTime::to_utc_seconds(&self, tai_utc_offset: i32) -> u64` shall subtract the TAI-UTC offset. | L1-PTP-004 |
| L2-PTP-004 | `PtpTime::to_absolute(&self, tai_utc_offset: i32) -> Result<AbsoluteTime>` shall convert to year/doy/time-of-day. | L1-PTP-001..004 |
| L2-PTP-005 | `PtpTime::to_nanos_since_tai_epoch(&self) -> u128` shall return total nanoseconds since the TAI epoch. | L1-PTP-002 |

### 3.13 Format 2 Correlation Integration

*Added in v0.2.0.*

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-F2COR-001 | `TimeCorrelator::add_reference_f2(&mut self, channel_id, rtc, network_time, leap_table)` shall accept Format 2 time data as a reference point. | L1-F2COR-001 |
| L2-F2COR-002 | For NTP sources, the correlator shall convert NTP time to `AbsoluteTime` before inserting. | L1-F2COR-001 |
| L2-F2COR-003 | For PTP sources, the correlator shall apply the leap-second offset from the provided table before inserting. | L1-F2COR-001, L1-F2COR-002 |

### 3.14 Leap Second Table

*Added in v0.2.0.*

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-TAI-001 | `LeapSecondTable::builtin()` shall return a table with all leap seconds from 1972 through the crate release date. | L1-TAI-002 |
| L2-TAI-002 | `LeapSecondTable::offset_at_unix(unix_seconds) -> i32` shall return the TAI-UTC offset effective at the given UTC time. | L1-TAI-001, L1-TAI-003 |
| L2-TAI-003 | `LeapSecondTable::add(&mut self, entry)` shall allow runtime insertion of new entries. | L1-TAI-003 |
| L2-TAI-004 | `LeapSecondTable::offset_at_tai(tai_seconds) -> i32` shall approximate the offset for a TAI timestamp. | L1-TAI-001 |

---

## 4. Traceability Matrix

| L1 ID | L2 IDs |
|-------|--------|
| L1-RTC-001 | L2-RTC-001, L2-RTC-003, L2-RTC-004, L2-RTC-008, L2-RTC-009, L2-RTC-010, L2-RTC-011 |
| L1-RTC-002 | L2-RTC-002 |
| L1-RTC-003 | L2-RTC-005, L2-RTC-006 |
| L1-RTC-004 | L2-RTC-006, L2-RTC-007 |
| L1-ABS-001 | L2-ABS-001, L2-ABS-002 |
| L1-ABS-002 | L2-ABS-003, L2-ABS-004 |
| L1-ABS-003 | L2-ABS-005, L2-ABS-006 |
| L1-ABS-004 | L2-ABS-007, L2-ABS-008 |
| L1-CSDW-001 | L2-CSDW-001, L2-CSDW-008 |
| L1-CSDW-002 | L2-CSDW-002, L2-CSDW-003 |
| L1-CSDW-003 | L2-CSDW-004, L2-CSDW-005 |
| L1-CSDW-004 | L2-CSDW-006 |
| L1-CSDW-005 | L2-CSDW-007 |
| L1-BCD-001 | L2-BCD-001, L2-BCD-002, L2-BCD-007, L2-BCD-009, L2-BCD-010 |
| L1-BCD-002 | L2-BCD-003, L2-BCD-004, L2-BCD-008, L2-BCD-009 |
| L1-BCD-003 | L2-BCD-005 |
| L1-BCD-004 | L2-BCD-006 |
| L1-SEC-001 | L2-SEC-001, L2-SEC-002 |
| L1-SEC-002 | L2-SEC-001, L2-SEC-003 |
| L1-SEC-003 | L2-SEC-001, L2-SEC-004 |
| L1-SEC-004 | L2-SEC-005 |
| L1-IPT-001 | L2-IPT-001, L2-IPT-005 |
| L1-IPT-002 | L2-IPT-002, L2-IPT-005 |
| L1-IPT-003 | L2-IPT-003, L2-IPT-005 |
| L1-IPT-004 | L2-IPT-004, L2-IPT-005 |
| L1-COR-001 | L2-COR-001, L2-COR-002, L2-COR-007 |
| L1-COR-002 | L2-COR-003, L2-COR-004 |
| L1-COR-003 | L2-COR-005 |
| L1-COR-004 | L2-COR-006 |
| L1-ERR-001 | L2-ERR-001, L2-ERR-002, L2-ERR-003, L2-COR-007 |
| L1-ERR-002 | L2-ERR-001 |
| L1-ERR-003 | L2-ERR-001 |
| L1-ERR-004 | L2-ERR-001, L2-SEC-005 |
| L1-API-001 | L2-API-001, L2-ERR-003 |
| L1-API-002 | L2-API-002 |
| L1-API-003 | L2-API-003 |
| L1-API-004 | L2-API-004 |
| L1-F2CSDW-001 | L2-F2CSDW-001 |
| L1-F2CSDW-002 | L2-F2CSDW-002, L2-F2CSDW-004 |
| L1-F2CSDW-003 | L2-F2CSDW-003 |
| L1-NTP-001 | L2-NTP-001 |
| L1-NTP-002 | L2-NTP-005 |
| L1-NTP-003 | L2-NTP-002 |
| L1-NTP-004 | L2-NTP-003, L2-NTP-004 |
| L1-PTP-001 | L2-PTP-001 |
| L1-PTP-002 | L2-PTP-005 |
| L1-PTP-003 | L2-PTP-002 |
| L1-PTP-004 | L2-PTP-003, L2-PTP-004 |
| L1-F2COR-001 | L2-F2COR-001, L2-F2COR-002 |
| L1-F2COR-002 | L2-F2COR-003 |
| L1-TAI-001 | L2-TAI-001, L2-TAI-002 |
| L1-TAI-002 | L2-TAI-001 |
| L1-TAI-003 | L2-TAI-002, L2-TAI-003 |
