# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.2.0) - 2026-03-28

### Added

- **`network_time` module** — Time Data Format 2 (0x12, Network Time) support, introduced in IRIG 106-17 (CR93).
  - `TimeF2Csdw` — Format 2 channel-specific data word parser with `NetworkTimeProtocol` enum (NTP/PTP).
  - `NtpTime` — NTP timestamp (32-bit seconds since 1900-01-01 UTC + 32-bit fractional seconds). Includes `fraction_as_nanos()` (2⁻³² → ns conversion), `to_unix_seconds()`, `to_nanos_since_ntp_epoch()`, and `to_absolute()`.
  - `PtpTime` — PTP/IEEE 1588 timestamp (48-bit seconds since 1970-01-01 TAI + 32-bit nanoseconds). Includes `to_utc_seconds(tai_offset)`, `to_nanos_since_tai_epoch()`, and `to_absolute(tai_offset)`.
  - `NetworkTime` enum — Discriminated union of `Ntp(NtpTime)` and `Ptp(PtpTime)`.
  - `parse_time_f2_payload()` — Full payload dispatcher: CSDW → NTP or PTP time data.
  - `LeapSecondTable` — Built-in table with all 28 TAI-UTC leap seconds from 1972-01-01 (offset 10) through 2017-01-01 (offset 37). Binary search lookup via `offset_at_unix()` and `offset_at_tai()`. Runtime extensible via `add()`.
  - `NTP_UNIX_EPOCH_OFFSET` constant (2,208,988,800 seconds).
  - `DEFAULT_TAI_UTC_OFFSET` constant (37 seconds, since 2017).
- **`TimeCorrelator::add_reference_f2()`** — Accept Format 2 (NTP/PTP) time packets as correlation reference points, with automatic leap-second offset application for PTP sources.
- **22 new unit tests** in `network_time_tests.rs` covering F2 CSDW parsing, NTP parse/convert/epoch math, PTP parse/TAI-UTC conversion, leap second table lookup, and full payload dispatch.
- **5 new integration tests** in `tests/pipeline.rs`: full NTP pipeline, full PTP pipeline, mixed F1+F2 correlation, NTP sub-millisecond precision, and leap second table historical accuracy.
- **2 new fuzz targets**: `fuzz_ntp` and `fuzz_ptp` covering NTP and PTP parsing entry points.
- **7 new benchmarks**: `ntp_from_le_bytes` (1.1 ns), `ntp_to_absolute` (115 ns), `ptp_from_le_bytes` (1.6 ns), `ptp_to_absolute` (119 ns), `leap_table_lookup` (6.6 ns), `f2_ntp_payload_parse` (2.7 ns).
- **L1 Requirements updated** — 16 new L1 requirements for Format 2 (F2CSDW, NTP, PTP, F2COR, TAI) merged into `L1_Requirements.md`, traced to Ch11 §11.2.3.3. Total: 53 L1 requirements.
- **L2/L3 Requirements updated** — Format 2 functional decomposition and design specifications merged into `L2_Requirements.md` (§3.10–3.14) and `L3_Requirements.md` (§3.12–3.13) with extended traceability matrices.
- **CLI (irig106-time-cli)** — `ch10time` now recognizes and processes Type 0x12 Network Time packets in all commands (summary, channels, jumps, timeline, csv, correlate). Uses built-in leap second table for PTP→UTC conversion.

### Changed

- Re-exports at crate root now include all Format 2 types: `TimeF2Csdw`, `NetworkTimeProtocol`, `NtpTime`, `PtpTime`, `NetworkTime`, `LeapSecondTable`, `LeapSecondEntry`, `NTP_UNIX_EPOCH_OFFSET`, `DEFAULT_TAI_UTC_OFFSET`.
- Total test count: **151** (126 unit + 15 integration + 10 property).
- Total fuzz targets: **10** (8 original + 2 new).
- Total benchmarks: **30** (23 original + 7 new).

## [v0.1.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.1.0) - 2026-03-26

### Added

- **`rtc` module** — 48-bit Relative Time Counter newtype (`Rtc`) with `from_le_bytes`, `from_raw`, `elapsed_ticks`, `elapsed_nanos`, `to_nanos`, and wrap-around safe arithmetic.
- **`absolute` module** — `AbsoluteTime` with nanosecond precision, optional DMY calendar date, `add_nanos`/`sub_nanos` with carry logic. `Ch4BinaryTime` (Chapter 4 Binary Weighted Time), `Ieee1588Time`, and `Ertc` (64-bit Extended RTC).
- **`csdw` module** — `TimeF1Csdw` parser for Time Data Format 1 (0x11) channel-specific data word. `TimeSource`, `TimeFormat`, and `DateFormat` enums with bitfield extraction.
- **`bcd` module** — `DayFormatTime` (8-byte DOY BCD) and `DmyFormatTime` (10-byte DMY BCD) decoders with full BCD nibble validation, reserved-bit checking, and range validation.
- **`secondary` module** — Secondary header checksum validation (`validate_secondary_checksum`) and time parsing for Ch4, IEEE-1588, and ERTC formats via `parse_secondary_header`.
- **`intra_packet` module** — `IntraPacketTime` enum with 4 variants (Rtc, Ch4, Ieee1588, Ertc) and `parse_intra_packet_time` format dispatcher.
- **`correlation` module** — `TimeCorrelator` with sorted-insert reference points, nearest-point correlation with optional channel filtering, and `detect_time_jump` for GPS lock discontinuity detection.
- **`error` module** — `TimeError` non-exhaustive enum with 6 variants: `InvalidBcdDigit`, `ReservedBitSet`, `OutOfRange`, `ChecksumMismatch`, `NoReferencePoint`, `BufferTooShort`. `Display` impl and feature-gated `std::error::Error`.
- **`#![no_std]` support** — Crate compiles with only `core` and `alloc`. Optional `std` feature for `Error` impl.
- **Zero required dependencies** in `[dependencies]`.
- **Zero `unsafe` blocks** in the entire crate.
- **`#[inline]` annotations** on all hot-path functions.
- **104 unit tests** across 8 modules (tests in separate files alongside source).
- **10 integration tests** in `tests/pipeline.rs` covering full CSDW-to-correlation flows, multi-channel scenarios, and GPS lock jump detection.
- **10 property-based tests** in `tests/properties.rs` (10,000 iterations each, zero external dependencies).
- **23 benchmarks** in `benches/time_benchmarks.rs` (zero-dependency, `std::time::Instant`).
- **8 fuzz targets** in `fuzz/fuzz_targets/` covering all parsing entry points.
- **Documentation**: L1/L2/L3 requirements (37/78/65), architecture doc, security doc, test index, project structure, shared types migration plan, roadmap, and rationale for separate repo.
- **13-slide technical presentation** (PPTX).

