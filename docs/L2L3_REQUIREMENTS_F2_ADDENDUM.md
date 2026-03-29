# L2/L3 Requirements Addendum — Time Data Format 2 (Network Time)

**Document:** L2L3_REQUIREMENTS_F2_ADDENDUM.md
**Crate:** irig106-time
**Target Version:** 0.2.0
**Parent:** L1_REQUIREMENTS_F2_ADDENDUM.md
**Date:** 2026-03-27

---

## 1. L2 Requirements (Functional Decomposition)

### 1.1 Format 2 CSDW

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-F2CSDW-001 | `TimeF2Csdw` shall be parsed from a `u32` value read in little-endian order. | L1-F2CSDW-001 |
| L2-F2CSDW-002 | `TimeF2Csdw::time_protocol(&self) -> NetworkTimeProtocol` shall return an enum decoded from bits [3:0]. | L1-F2CSDW-002 |
| L2-F2CSDW-003 | `TimeF2Csdw::validate_reserved(&self) -> Result<()>` shall verify bits [31:4] are zero. | L1-F2CSDW-003 |
| L2-F2CSDW-004 | The `NetworkTimeProtocol` enum shall include variants `Ntp`, `Ptp`, and `Reserved(u8)`. | L1-F2CSDW-002 |

### 1.2 NTP Time

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-NTP-001 | `NtpTime::from_le_bytes(&[u8]) -> Result<Self>` shall parse 8 bytes into seconds and fractional seconds. | L1-NTP-001 |
| L2-NTP-002 | `NtpTime::fraction_as_nanos(&self) -> u32` shall convert the fractional field to nanoseconds via `(fraction * 10^9) >> 32`. | L1-NTP-003 |
| L2-NTP-003 | `NtpTime::to_unix_seconds(&self) -> Option<u64>` shall subtract `NTP_UNIX_EPOCH_OFFSET` (2,208,988,800), returning `None` if underflow. | L1-NTP-004 |
| L2-NTP-004 | `NtpTime::to_absolute(&self) -> Result<AbsoluteTime>` shall convert NTP time to year/doy/time-of-day. | L1-NTP-001..004 |
| L2-NTP-005 | `NtpTime::to_nanos_since_ntp_epoch(&self) -> u64` shall return total nanoseconds since the NTP epoch. | L1-NTP-002 |

### 1.3 PTP Time

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-PTP-001 | `PtpTime::from_le_bytes(&[u8]) -> Result<Self>` shall parse 10 bytes: 6-byte (48-bit) seconds and 4-byte nanoseconds. | L1-PTP-001 |
| L2-PTP-002 | `PtpTime::from_le_bytes` shall return `Err(OutOfRange)` if nanoseconds >= 1,000,000,000. | L1-PTP-003 |
| L2-PTP-003 | `PtpTime::to_utc_seconds(&self, tai_utc_offset: i32) -> u64` shall subtract the TAI-UTC offset. | L1-PTP-004 |
| L2-PTP-004 | `PtpTime::to_absolute(&self, tai_utc_offset: i32) -> Result<AbsoluteTime>` shall convert to year/doy/time-of-day. | L1-PTP-001..004 |
| L2-PTP-005 | `PtpTime::to_nanos_since_tai_epoch(&self) -> u128` shall return total nanoseconds since the TAI epoch. | L1-PTP-002 |

### 1.4 Correlation Integration

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-F2COR-001 | `TimeCorrelator::add_reference_f2(&mut self, channel_id, rtc, network_time, leap_table)` shall accept Format 2 time data as a reference point. | L1-F2COR-001 |
| L2-F2COR-002 | For NTP sources, the correlator shall convert NTP time to `AbsoluteTime` before inserting. | L1-F2COR-001 |
| L2-F2COR-003 | For PTP sources, the correlator shall apply the leap-second offset from the provided table before inserting. | L1-F2COR-001, L1-F2COR-002 |

### 1.5 Leap Second Table

| ID | Requirement | Traces |
|----|-------------|--------|
| L2-TAI-001 | `LeapSecondTable::builtin()` shall return a table with all leap seconds from 1972 through the crate release date. | L1-TAI-002 |
| L2-TAI-002 | `LeapSecondTable::offset_at_unix(unix_seconds) -> i32` shall return the TAI-UTC offset effective at the given UTC time. | L1-TAI-001, L1-TAI-003 |
| L2-TAI-003 | `LeapSecondTable::add(&mut self, entry)` shall allow runtime insertion of new entries. | L1-TAI-003 |
| L2-TAI-004 | `LeapSecondTable::offset_at_tai(tai_seconds) -> i32` shall approximate the offset for a TAI timestamp. | L1-TAI-001 |

---

## 2. L3 Specifications (Design Detail)

### 2.1 Module: `src/network_time.rs`

| ID | Specification | Traces |
|----|--------------|--------|
| L3-F2-001 | `pub struct TimeF2Csdw(u32);` — same newtype pattern as `TimeF1Csdw`. | L2-F2CSDW-001 |
| L3-F2-002 | `pub enum NetworkTimeProtocol { Ntp, Ptp, Reserved(u8) }` from bits [3:0]. | L2-F2CSDW-004 |
| L3-F2-003 | `pub struct NtpTime { pub seconds: u32, pub fraction: u32 }` — LE bytes [0..4] = seconds, [4..8] = fraction. | L2-NTP-001 |
| L3-F2-004 | NTP fraction → nanos: `((fraction as u64 * 1_000_000_000) >> 32) as u32`. | L2-NTP-002 |
| L3-F2-005 | `const NTP_UNIX_EPOCH_OFFSET: u64 = 2_208_988_800;` (70 years including 17 leap years). | L2-NTP-003 |
| L3-F2-006 | `pub struct PtpTime { pub seconds: u64, pub nanoseconds: u32 }` — LE bytes [0..6] = 48-bit seconds, [6..10] = nanos. 48-bit masking via zero-extend. | L2-PTP-001 |
| L3-F2-007 | `const DEFAULT_TAI_UTC_OFFSET: i32 = 37;` (since 2017-01-01). | L2-PTP-003 |
| L3-F2-008 | `pub enum NetworkTime { Ntp(NtpTime), Ptp(PtpTime) }` — parsed payload discriminated union. | L2-F2CSDW-004 |
| L3-F2-009 | `pub fn parse_time_f2_payload(payload: &[u8]) -> Result<(TimeF2Csdw, NetworkTime)>` — dispatch on CSDW protocol field. | L2-F2CSDW-001..004 |
| L3-F2-010 | `pub struct LeapSecondEntry { pub effective_unix: u64, pub tai_utc_offset: i32 }` | L2-TAI-001 |
| L3-F2-011 | `pub struct LeapSecondTable { entries: Vec<LeapSecondEntry> }` — sorted by `effective_unix`, binary search lookup. | L2-TAI-001..004 |
| L3-F2-012 | Built-in table: 28 entries from 1972-01-01 (offset 10) through 2017-01-01 (offset 37). | L2-TAI-001 |
| L3-F2-013 | `unix_seconds_to_ymd(unix_secs) -> (year, doy, hour, minute, second)` — internal helper, walks years from 1970. | L2-NTP-004, L2-PTP-004 |

### 2.2 Correlator Extension: `src/correlation.rs`

| ID | Specification | Traces |
|----|--------------|--------|
| L3-F2-014 | `TimeCorrelator::add_reference_f2(channel_id, rtc, network_time, leap_table)` dispatches NTP/PTP → `AbsoluteTime` → `add_reference`. | L2-F2COR-001..003 |

---

## 3. Traceability Matrix

| L1 ID | L2 IDs | L3 IDs |
|-------|--------|--------|
| L1-F2CSDW-001 | L2-F2CSDW-001 | L3-F2-001 |
| L1-F2CSDW-002 | L2-F2CSDW-002, L2-F2CSDW-004 | L3-F2-002 |
| L1-F2CSDW-003 | L2-F2CSDW-003 | L3-F2-001 |
| L1-NTP-001 | L2-NTP-001 | L3-F2-003 |
| L1-NTP-002 | L2-NTP-005 | L3-F2-005 |
| L1-NTP-003 | L2-NTP-002 | L3-F2-004 |
| L1-NTP-004 | L2-NTP-003, L2-NTP-004 | L3-F2-005, L3-F2-013 |
| L1-PTP-001 | L2-PTP-001 | L3-F2-006 |
| L1-PTP-002 | L2-PTP-005 | L3-F2-006 |
| L1-PTP-003 | L2-PTP-002 | L3-F2-006 |
| L1-PTP-004 | L2-PTP-003, L2-PTP-004 | L3-F2-007, L3-F2-013 |
| L1-F2COR-001 | L2-F2COR-001, L2-F2COR-002 | L3-F2-014 |
| L1-F2COR-002 | L2-F2COR-003 | L3-F2-014 |
| L1-TAI-001 | L2-TAI-001, L2-TAI-002 | L3-F2-010, L3-F2-011 |
| L1-TAI-002 | L2-TAI-001 | L3-F2-012 |
| L1-TAI-003 | L2-TAI-002, L2-TAI-003 | L3-F2-011 |
