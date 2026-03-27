# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.1.0) - 2026-03-26

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
- **Companion CLI crate**: `irig106-time-cli` with the `ch10time` binary for file summaries, channel inventory, jump detection, timelines, CSV export, and one-off RTC correlation.
- **Documentation**: L1/L2/L3 requirements, architecture doc, security doc, usage guide, test index, project structure, roadmap, changelog, shared types migration plan, and rationale for separate repo.
