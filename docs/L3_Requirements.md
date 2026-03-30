# L3 Requirements — irig106-time

**Document:** L3_Requirements.md
**Crate:** irig106-time
**Version:** 0.2.0
**Parent:** L2_Requirements.md
**Date:** 2026-03-27

---

## 1. Purpose

Level 3 (L3) requirements define the concrete design, data layout, and algorithmic detail
needed to implement each L2 requirement. Every L3 traces to one or more L2 requirements
and forward to specific source files and tests.

---

## 2. Module Map

| Module | Source File | Unit Test File | Description |
|--------|-----------|----------------|-------------|
| `error` | `src/error.rs` | `src/error_tests.rs` | Error types |
| `rtc` | `src/rtc.rs` | `src/rtc_tests.rs` | 48-bit RTC type |
| `absolute` | `src/absolute.rs` | `src/absolute_tests.rs` | Absolute time type |
| `csdw` | `src/csdw.rs` | `src/csdw_tests.rs` | Time F1 CSDW parsing |
| `bcd` | `src/bcd.rs` | `src/bcd_tests.rs` | BCD day/DMY decoding |
| `secondary` | `src/secondary.rs` | `src/secondary_tests.rs` | Secondary header time |
| `intra_packet` | `src/intra_packet.rs` | `src/intra_packet_tests.rs` | Intra-packet timestamps |
| `correlation` | `src/correlation.rs` | `src/correlation_tests.rs` | RTC↔absolute correlation |
| `network_time` | `src/network_time.rs` | `src/network_time_tests.rs` | Format 2 NTP/PTP/LeapSecondTable |

---

## 3. L3 Specifications

### 3.1 Error Types (`src/error.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-ERR-001 | Define `TimeError` as a `#[non_exhaustive]` enum with variants matching L2-ERR-001. Each variant shall carry enough context for a meaningful error message. | L2-ERR-001 |
| L3-ERR-002 | Implement `core::fmt::Display` for `TimeError` with human-readable messages. | L2-ERR-003 |
| L3-ERR-003 | Under `#[cfg(feature = "std")]`, implement `std::error::Error` for `TimeError`. | L2-ERR-003 |
| L3-ERR-004 | Define `type Result<T> = core::result::Result<T, TimeError>;` as the crate-wide result alias. | L2-ERR-002 |

### 3.2 RTC (`src/rtc.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-RTC-001 | `pub struct Rtc(u64);` — invariant: `self.0 <= 0x0000_FFFF_FFFF_FFFF`. | L2-RTC-001 |
| L3-RTC-002 | `const MASK_48: u64 = 0x0000_FFFF_FFFF_FFFF;` | L2-RTC-001, L2-RTC-003 |
| L3-RTC-003 | `const NANOS_PER_TICK: u64 = 100;` (10 MHz → 100 ns/tick). | L2-RTC-006, L2-RTC-007 |
| L3-RTC-004 | `pub const ZERO: Rtc = Rtc(0);` | L2-RTC-010 |
| L3-RTC-005 | `pub const MAX: Rtc = Rtc(MASK_48);` | L2-RTC-011 |
| L3-RTC-006 | `from_le_bytes`: construct `u64` via `u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], 0, 0])`. | L2-RTC-002 |
| L3-RTC-007 | `from_raw`: `Rtc(value & MASK_48)`. | L2-RTC-003 |
| L3-RTC-008 | `as_raw`: return `self.0`. | L2-RTC-004 |
| L3-RTC-009 | `elapsed_ticks`: `(later.0.wrapping_sub(self.0)) & MASK_48`. This handles 48-bit wrap. | L2-RTC-005 |
| L3-RTC-010 | `elapsed_nanos`: `self.elapsed_ticks(later) * NANOS_PER_TICK`. | L2-RTC-006 |
| L3-RTC-011 | `to_nanos`: `self.0 * NANOS_PER_TICK`. | L2-RTC-007 |
| L3-RTC-012 | Implement `Ord` and `PartialOrd` by comparing `self.0`. | L2-RTC-008 |
| L3-RTC-013 | Derive `Debug, Clone, Copy, PartialEq, Eq, Hash`. | L2-API-001 |

### 3.3 Absolute Time (`src/absolute.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-ABS-001 | `pub struct AbsoluteTime { total_ns: u64, year: Option<u16> }` — DOY-based timestamp with optional year metadata. Internal `u64` representation (nanoseconds since start of day 1). Year is metadata, not a calendar invariant. Cannot hold month or day-of-month. | L2-ABS-001, L2-ABS-002 |
| L3-ABS-002 | Constructor `AbsoluteTime::new(day_of_year, hours, minutes, seconds, nanoseconds)` shall validate ranges and return `Result`. `with_year(self, year: Option<u16>) -> Result<Self>` attaches year metadata (0–9999) as a consuming builder. | L2-ABS-001 |
| L3-ABS-003 | `CalendarTime` wraps `AbsoluteTime` with validated `month: u8` (1–12) and `day_of_month: u8` (1–N, leap-aware). Constructor `CalendarTime::new(time, month, day)` validates: year is present, day ≤ `days_in_month(year, month)`, and DOY matches `month_day_to_doy(year, month, day)`. Implements `Deref<Target=AbsoluteTime>`. | L2-ABS-002 |
| L3-ABS-004 | `AbsoluteTime::add_nanos(&self, nanos: u64) -> AbsoluteTime` shall carry into seconds, minutes, hours, days. Single `u64` add on the common path. | L2-ABS-001 |
| L3-ABS-005 | `AbsoluteTime::total_nanos_of_day(&self) -> u64` returns nanoseconds since midnight. | L2-ABS-001 |
| L3-ABS-006 | `CalendarTime::from_parts(year, month, day, doy, h, m, s, ns) -> Result<Self>` convenience constructor combining `new` + `with_year` + `CalendarTime::new`. `DmyFormatTime::to_calendar_time() -> Result<CalendarTime>` produces calendar time from BCD DMY wire format. `From<NaiveDateTime> for CalendarTime` (chrono feature). | L2-ABS-002 |

### 3.4 Chapter 4 Binary Weighted Time

| ID | Specification | Traces |
|----|--------------|--------|
| L3-CH4-001 | `pub struct Ch4BinaryTime { pub high_order: u16, pub low_order: u16, pub microseconds: u16 }` | L2-ABS-003 |
| L3-CH4-002 | The combined `high_order:low_order` 32-bit value represents seconds since midnight of the day-of-year. Bits [16:0] of `low_order` and bits [16:0] of `high_order` form a 32-bit seconds-of-day. Upper bits of the combined field encode the day-of-year. | L2-ABS-004 |
| L3-CH4-003 | Specifically: combined = `((high_order as u32) << 16) | (low_order as u32)`. Bits [16:0] = time in 10 ms increments. Bits [25:17] = day of year (1-based). | L2-ABS-004 |
| L3-CH4-004 | `to_absolute`: Extract day, compute hours/minutes/seconds from 10ms count, add microseconds. | L2-ABS-004 |
| L3-CH4-005 | `from_le_bytes([u8; 6])`: parse from buffer (`[unused_2B][high_2B][low_2B][usec_2B]` for secondary header, or appropriate layout for intra-packet). | L2-SEC-002 |

### 3.5 IEEE-1588 Time

| ID | Specification | Traces |
|----|--------------|--------|
| L3-1588-001 | `pub struct Ieee1588Time { pub nanoseconds: u32, pub seconds: u32 }` | L2-ABS-005 |
| L3-1588-002 | `from_le_bytes([u8; 8])`: `nanoseconds = u32::from_le_bytes(buf[0..4])`, `seconds = u32::from_le_bytes(buf[4..8])`. | L2-ABS-005 |
| L3-1588-003 | `to_nanos_since_epoch`: `(self.seconds as u64) * 1_000_000_000 + (self.nanoseconds as u64)`. | L2-ABS-006 |
| L3-1588-004 | Validate `nanoseconds < 1_000_000_000`; return `Err(OutOfRange)` otherwise. | L2-ABS-006 |

### 3.6 ERTC

| ID | Specification | Traces |
|----|--------------|--------|
| L3-ERTC-001 | `pub struct Ertc(u64);` — 64-bit extended RTC, 100 ns resolution. | L2-ABS-007 |
| L3-ERTC-002 | `from_le_bytes([u8; 8])`: `Ertc(u64::from_le_bytes(buf))`. | L2-ABS-007 |
| L3-ERTC-003 | `to_nanos`: `(self.0 as u128) * 100`. Returns `u128` to avoid overflow. | L2-ABS-008 |

### 3.7 CSDW (`src/csdw.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-CSDW-001 | `pub struct TimeF1Csdw(u32);` | L2-CSDW-001, L2-CSDW-008 |
| L3-CSDW-002 | `from_raw(raw: u32) -> Self`: store raw value. | L2-CSDW-008 |
| L3-CSDW-003 | `from_le_bytes(buf: [u8; 4]) -> Self`: `Self(u32::from_le_bytes(buf))`. | L2-CSDW-001 |
| L3-CSDW-004 | `time_source`: `(self.0 & 0x0F) as u8` → match to `TimeSource` enum. | L2-CSDW-002 |
| L3-CSDW-005 | `time_format`: `((self.0 >> 4) & 0x0F) as u8` → match to `TimeFormat` enum. | L2-CSDW-004 |
| L3-CSDW-006 | `is_leap_year`: `(self.0 >> 8) & 1 == 1`. | L2-CSDW-006 |
| L3-CSDW-007 | `date_format`: `(self.0 >> 9) & 1` → `0 = DayOfYear`, `1 = DayMonthYear`. | L2-CSDW-007 |
| L3-CSDW-008 | Define `TimeSource` enum: `Internal = 0`, `External = 1`, `InternalRtc = 2`, `None = 0xF`, plus catch-all `Reserved(u8)`. | L2-CSDW-003 |
| L3-CSDW-009 | Define `TimeFormat` enum: `IrigB = 0`, `IrigA = 1`, `IrigG = 2`, `Rtc = 3`, `Utc = 4`, `Gps = 5`, plus catch-all `Reserved(u8)`. | L2-CSDW-005 |
| L3-CSDW-010 | Define `DateFormat` enum: `DayOfYear = 0`, `DayMonthYear = 1`. | L2-CSDW-007 |

### 3.8 BCD Decoding (`src/bcd.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-BCD-001 | Helper `fn decode_bcd_nibble(word: u16, bit_offset: u8, width: u8) -> Result<u8>`: extract `width` bits at `bit_offset`, validate each 4-bit nibble ≤ 9. | L2-BCD-006, L2-BCD-009 |
| L3-BCD-002 | Helper `fn check_reserved_zero(word: u16, bit_offset: u8, width: u8, name: &'static str) -> Result<()>`: verify bits are zero. | L2-BCD-005 |
| L3-BCD-003 | `DayFormatTime` byte layout (8 bytes = 4× u16 LE): Word 0 bits [3:0] = Tmn (tens ms), [7:4] = Hmn (hundreds ms), [11:8] = Sn (units sec), [14:12] = TSn (tens sec), [15] = reserved. Word 1 bits [3:0] = Mn (units min), [6:4] = TMn (tens min), [7] = reserved, [11:8] = Hn (units hr), [13:12] = THn (tens hr), [15:14] = reserved. Word 2 bits [3:0] = Dn (units day), [7:4] = TDn (tens day), [9:8] = HDn (hundreds day), [15:10] = reserved. Word 3 is unused/reserved. | L2-BCD-001, L2-BCD-002 |
| L3-BCD-004 | `DmyFormatTime` byte layout (10 bytes = 5× u16 LE): Words 0–2 identical to DayFormatTime except Word 2 bits [3:0] = Dn (units day), [7:4] = TDn (tens day), [11:8] = On (units month), [12] = TOn (tens month), [15:13] = reserved. Word 3 bits [3:0] = Yn (units year), [7:4] = TYn (tens year), [11:8] = HYn (hundreds year), [13:12] = OYn (thousands year), [15:14] = reserved. Word 4 is unused/reserved. | L2-BCD-003, L2-BCD-004 |
| L3-BCD-005 | Milliseconds = `Hmn * 100 + Tmn * 10`. No units digit; resolution is 10 ms per spec. | L2-BCD-010 |
| L3-BCD-006 | Validate ranges after decode: hours ≤ 23, minutes ≤ 59, seconds ≤ 59, day_of_year ∈ [1, 366], month ∈ [1, 12], day ∈ [1, 31]. Return `Err(OutOfRange)` on violation. | L2-BCD-002, L2-BCD-004 |
| L3-BCD-007 | `to_absolute()`: construct `AbsoluteTime` with `nanoseconds = milliseconds * 1_000_000`. | L2-BCD-007, L2-BCD-008 |

### 3.9 Secondary Header (`src/secondary.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-SEC-001 | `pub enum SecHdrTimeFormat { Ch4 = 0, Ieee1588 = 1, Ertc = 2, Reserved(u8) }` derived from Packet Flag bits [3:2]. | L2-SEC-001 |
| L3-SEC-002 | `validate_secondary_checksum(buf: &[u8; 12]) -> Result<()>`: sum 5 LE u16 words from bytes [0..10], compare with u16 at bytes [10..12]. | L2-SEC-005 |
| L3-SEC-003 | `pub enum SecondaryHeaderTime { Ch4(Ch4BinaryTime), Ieee1588(Ieee1588Time), Ertc(Ertc) }` | L2-SEC-001..L2-SEC-004 |
| L3-SEC-004 | `parse_secondary_header(buf: &[u8; 12], fmt: SecHdrTimeFormat) -> Result<SecondaryHeaderTime>`: validate checksum first, then dispatch. | L2-SEC-001..L2-SEC-005 |

### 3.10 Intra-Packet Timestamps (`src/intra_packet.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-IPT-001 | `pub enum IntraPacketTime { Rtc(Rtc), Ch4(Ch4BinaryTime), Ieee1588(Ieee1588Time), Ertc(Ertc) }` | L2-IPT-005 |
| L3-IPT-002 | `parse_intra_packet_time(buf: &[u8; 8], fmt: IntraPacketTimeFormat) -> Result<IntraPacketTime>` dispatches based on format. | L2-IPT-001..L2-IPT-004 |
| L3-IPT-003 | `IntraPacketTimeFormat` enum: `Rtc48`, `Ch4Binary`, `Ieee1588`, `Ertc64`. | L2-IPT-001..L2-IPT-004 |

### 3.11 Correlation Engine (`src/correlation.rs`)

| ID | Specification | Traces |
|----|--------------|--------|
| L3-COR-001 | `pub struct ReferencePoint { pub channel_id: u16, pub rtc: Rtc, pub time: AbsoluteTime }` | L2-COR-001 |
| L3-COR-002 | `pub struct TimeCorrelator { references: Vec<ReferencePoint> }` (requires `alloc`). Sorted by RTC on insert. | L2-COR-001 |
| L3-COR-003 | `add_reference`: push and maintain sort order by `rtc`. Use `Vec::binary_search` for O(log n) insert. | L2-COR-002 |
| L3-COR-004 | `correlate`: binary search for closest RTC, compute delta ticks, convert to nanos, add to reference time via `AbsoluteTime::add_nanos`. | L2-COR-003, L2-COR-004 |
| L3-COR-005 | When filtering by `channel_id`, use a linear scan of sorted references (optimizable later). | L2-COR-005 |
| L3-COR-006 | `TimeJump { index: usize, channel_id: u16, expected_nanos: u64, actual_nanos: u64, delta_nanos: i64 }` | L2-COR-006 |
| L3-COR-007 | `detect_time_jump`: iterate consecutive same-channel pairs, compute expected absolute time from RTC delta, compare with actual, flag if |delta| > threshold. | L2-COR-006 |

### 3.12 Network Time (`src/network_time.rs`)

*Added in v0.2.0.*

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

### 3.13 Correlator Format 2 Extension (`src/correlation.rs`)

*Added in v0.2.0.*

| ID | Specification | Traces |
|----|--------------|--------|
| L3-F2-014 | `TimeCorrelator::add_reference_f2(channel_id, rtc, network_time, leap_table)` dispatches NTP/PTP → `AbsoluteTime` → `add_reference`. | L2-F2COR-001..003 |

---

## 4. Shared Types (Candidates for `irig106-types`)

The following types are used across multiple crates in the ecosystem and should
eventually be migrated to `irig106-types`. See `shared_types_for_irig106_types.md`.

- `Rtc` — used by `irig106-core`, `irig106-time`, `irig106-decode`, `irig106-write`
- `Ch4BinaryTime` — used by `irig106-time`, `irig106-decode`
- `Ieee1588Time` — used by `irig106-time`, `irig106-decode`
- `Ertc` — used by `irig106-time`, `irig106-decode`
- `TimeSource`, `TimeFormat`, `DateFormat` enums — used by `irig106-time`, `irig106-decode`

---

## 5. Full Traceability Matrix (L2 → L3)

| L2 ID | L3 IDs | Source File |
|-------|--------|-------------|
| L2-RTC-001 | L3-RTC-001, L3-RTC-002 | rtc.rs |
| L2-RTC-002 | L3-RTC-006 | rtc.rs |
| L2-RTC-003 | L3-RTC-002, L3-RTC-007 | rtc.rs |
| L2-RTC-004 | L3-RTC-008 | rtc.rs |
| L2-RTC-005 | L3-RTC-009 | rtc.rs |
| L2-RTC-006 | L3-RTC-003, L3-RTC-010 | rtc.rs |
| L2-RTC-007 | L3-RTC-003, L3-RTC-011 | rtc.rs |
| L2-RTC-008 | L3-RTC-012 | rtc.rs |
| L2-RTC-009 | L3-RTC-005 | rtc.rs |
| L2-RTC-010 | L3-RTC-004 | rtc.rs |
| L2-RTC-011 | L3-RTC-005 | rtc.rs |
| L2-ABS-001..002 | L3-ABS-001..L3-ABS-005 | absolute.rs |
| L2-ABS-003..004 | L3-CH4-001..L3-CH4-005 | absolute.rs |
| L2-ABS-005..006 | L3-1588-001..L3-1588-004 | absolute.rs |
| L2-ABS-007..008 | L3-ERTC-001..L3-ERTC-003 | absolute.rs |
| L2-CSDW-001..008 | L3-CSDW-001..L3-CSDW-010 | csdw.rs |
| L2-BCD-001..010 | L3-BCD-001..L3-BCD-007 | bcd.rs |
| L2-SEC-001..005 | L3-SEC-001..L3-SEC-004 | secondary.rs |
| L2-IPT-001..005 | L3-IPT-001..L3-IPT-003 | intra_packet.rs |
| L2-COR-001..007 | L3-COR-001..L3-COR-007 | correlation.rs |
| L2-ERR-001..003 | L3-ERR-001..L3-ERR-004 | error.rs |
| L2-API-001..004 | L3-RTC-013, L3-ABS-001, L3-CSDW-001 | (all modules) |
| L2-F2CSDW-001..004 | L3-F2-001, L3-F2-002 | network_time.rs |
| L2-NTP-001..005 | L3-F2-003..L3-F2-005, L3-F2-013 | network_time.rs |
| L2-PTP-001..005 | L3-F2-006..L3-F2-007, L3-F2-013 | network_time.rs |
| L2-F2COR-001..003 | L3-F2-014 | correlation.rs |
| L2-TAI-001..004 | L3-F2-010..L3-F2-012 | network_time.rs |
