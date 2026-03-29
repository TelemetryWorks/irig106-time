# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.4.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.4.0) - 2026-03-28

### Added

- **`version` module** (P2-03) — `Irig106Version` enum with variants `Pre07` through `V23` plus `Unknown(u8)`. `detect_version(tmats_csdw)` extracts the version from bits \[7:0\] of the TMATS CSDW. Helper methods: `is_pre_ordering_guarantee()`, `supports_format_2()`, `has_gps_time_source()`.
- **Version-aware CSDW parsing** (P2-04, P2-01) — `TimeF1Csdw::time_source_versioned(version)` disambiguates time source value 3, which was "None" in 106-04 but "GPS" from 106-05 onward. Pre-07 files return `Reserved(3)` to signal ambiguity.
- **Configurable out-of-order window** (P2-02) — `TimeCorrelator::with_ooo_window(ooo_window_ns)` constructor. Pre-105 files may need unbounded OOO tolerance; post-105 defaults to 2 seconds via `DEFAULT_OOO_WINDOW_NS`. Accessor: `ooo_window_ns()`.
- **RTC reset detection** (GAP-07) — `TimeCorrelator::detect_rtc_resets(channel_id)` identifies counter resets (as opposed to 48-bit wraps) by checking whether RTC went backward while absolute time continued forward. Returns `Vec<RtcReset>` with before/after RTC and absolute time values.
- **`to_le_bytes()` encoding** (GAP-11) — Wire-format encoding for all parseable types, enabling packet construction for `irig106-write`:
  - `Rtc::to_le_bytes()` → `[u8; 6]`
  - `TimeF1Csdw::to_le_bytes()` → `[u8; 4]`
  - `TimeF2Csdw::to_le_bytes()` → `[u8; 4]`
  - `NtpTime::to_le_bytes()` → `[u8; 8]`
  - `PtpTime::to_le_bytes()` → `[u8; 10]`
  - `DayFormatTime::to_le_bytes()` → `[u8; 8]`
  - `DmyFormatTime::to_le_bytes()` → `[u8; 10]`
- **22 new integration tests** in `tests/pipeline.rs`: version detection, version-aware CSDW, OOO window, RTC reset detection (basic, no false positive, channel isolation), and `to_le_bytes` round-trip for all 7 types.
- **4 new property tests** in `tests/properties.rs`: encode round-trip for RTC, CSDW, NTP, and PTP.

### Changed

- **GitHub Actions** — Updated `actions/checkout` from `v4` to `v6` across the CI workflow.
- **Rust formatting** — Ran `cargo fmt` across the repository to normalize code style.
- **Crate docs updated** — `lib.rs` feature list now includes version detection, RTC reset detection, and encoding.
- **Re-exports** — `RtcReset`, `detect_version`, and `Irig106Version` added to crate root.
- **`Cargo.toml`** — Version bumped to 0.4.0.

## [v0.3.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.3.0) - 2026-03-28

### Added

- **GitHub Actions CI/CD** (P1-03) — Test on stable + MSRV 1.87, clippy, rustfmt, rustdoc, and CLI build/lint.
- **`impl Display for AbsoluteTime`** (GAP-01) — Formats as `YYYY-MM-DD HH:MM:SS.mmm.uuu` when year/month/day are available, or `Day DDD HH:MM:SS.mmm.uuu` otherwise.
- **`TimeCorrelator::drift_ppm(channel_id)`** (GAP-10) — Estimates RTC clock drift in parts-per-million against absolute time references. Returns average drift across consecutive same-channel reference pairs.
- **Calendar validation in DMY BCD decoding** (GAP-09) — `DmyFormatTime::from_le_bytes` now rejects invalid day-for-month combinations (e.g., Feb 30, Jun 31) via a `days_in_month()` helper that accounts for leap years.
- **CLI `Proto` column** (P1-09/GAP-12) — `channels` command table now shows NTP/PTP protocol identity. Summary view appends `Proto: NTP` or `Proto: PTP` for Format 2 channels. `TimeChannelInfo` carries a `network_protocol` field.
- **`proptest` property-based tests** (P1-08) — `proptest = "1"` added to `[dev-dependencies]`. `tests/properties.rs` rewritten with `proptest!` macros covering RTC masking, round-trip, elapsed bounds, absolute time add/sub/monotonicity/Display, IEEE-1588 consistency, CSDW stability, NTP fraction bounds, and PTP monotonicity.
- **10 new integration tests** in `tests/pipeline.rs`: Display formatting (DOY, DMY, zero-padding), drift_ppm (zero drift, fast RTC, channel isolation, insufficient refs), calendar validation, and year overflow guard.

### Changed

- **`#[deny(missing_docs)]`** (P1-04) — Added to crate root. All `pub mod` declarations in `lib.rs` now carry `///` doc comments.
- **Crate docs updated** (GAP-13) — `lib.rs` module-level documentation now includes network time (NTP/PTP) and drift estimation in the feature list. Requirement traceability paths updated to lowercase filenames.
- **`unix_seconds_to_ymd_pub`** (GAP-15) — Visibility reduced from `pub` to `pub(crate)`.
- **Year overflow guard** (GAP-16) — `unix_seconds_to_ymd` now uses `saturating_add` and breaks at `u16::MAX` to prevent panic on malformed timestamps far in the future.
- **`Cargo.toml`** — Version bumped to 0.3.0. `rust-version = "1.87"` MSRV declared.
- **CLI (irig106-time-cli)** — Version bumped to 0.3.0.
- Total test count: **163** (126 unit + 24 integration + 13 property).

### Fixed

- Escaped bit-range bracket notation (`[3:2]`, `[3:0]`, etc.) in rustdoc comments across `absolute.rs`, `csdw.rs`, `secondary.rs`, `intra_packet.rs`, and `network_time.rs` to resolve 9 intra-doc link warnings.
- Resolved `prop_assert!` macro conflict with nested `format!` in property tests.
- Removed unused imports and variables in `properties.rs` and `pipeline.rs`.

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
- CLI `process_time_f2_packet` refactored to use `add_reference_f2()` instead of manually duplicating NTP/PTP→AbsoluteTime conversion logic.
- F2 L1 requirements merged from standalone addendum into canonical `L1_Requirements.md` (§3.10–3.14).
- F2 L2/L3 requirements merged from standalone addendum into canonical `L2_Requirements.md` (§3.10–3.14) and `L3_Requirements.md` (§3.12–3.13). Removed `L2L3_REQUIREMENTS_F2_ADDENDUM.md`.
- `usage.md` updated with 3 new Format 2 sections (§13 Processing Network Time Packets, §14 Working with the Leap Second Table, §15 Correlating with F1 + F2 Sources), updated imports and version references throughout.

### Fixed

- Resolved clippy `manual_is_multiple_of` in `network_time.rs` leap year check.
- Resolved clippy `manual_range_contains` in `network_time_tests.rs` and `pipeline.rs` NTP fractional precision assertions.
- Resolved clippy `unused_imports` in `pipeline.rs` F2 integration tests.
- Resolved clippy `unused_must_use` in `time_benchmarks.rs` for `ntp_to_absolute` and `ptp_to_absolute` benchmarks.
- Resolved clippy `needless_borrow`, `for_kv_map`, `redundant_closure`, `single_match`, and `manual_is_multiple_of` in CLI.

### Known Issues

- CLI `channels` command displays NTP sources as "External / UTC" and PTP sources as "External / GPS" — the specific network protocol identity is not shown. Tracked as GAP-12, planned for v0.3.0 (P1-09).

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
