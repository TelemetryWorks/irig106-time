# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.7.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.7.0) - 2026-03-29

### ⚠️ BREAKING CHANGES

- **`AbsoluteTime` restructured with `u64` internal representation** (P4-04) — Fields are now accessed via methods instead of direct field access. The internal representation is a single `u64` (nanoseconds since start of day 1), making `add_nanos`/`sub_nanos` single arithmetic operations on the common path.

  **Migration guide:**
  - Field reads: `t.hours` → `t.hours()`, `t.day_of_year` → `t.day_of_year()`, `t.nanoseconds` → `t.nanoseconds()`, etc.
  - Optional fields: `t.year` → `t.year()`, `t.month` → `t.month()`, `t.day_of_month` → `t.day_of_month()`
  - Field mutations: `t.year = Some(2025)` → `t.set_year(Some(2025))`, `t.month = Some(3)` → `t.set_month(Some(3))`, `t.day_of_month = Some(15)` → `t.set_day_of_month(Some(15))`
  - Struct literals: `AbsoluteTime { day_of_year: 100, hours: 12, ... }` → `AbsoluteTime::new(100, 12, ...)`

### Added

- **`AbsoluteTime::as_total_ns()`** — Exposes the raw internal nanosecond count for efficient comparison and serialization.
- **`AbsoluteTime::set_year()`, `set_month()`, `set_day_of_month()`** — Setter methods for optional calendar metadata fields.
- **UDP framing documentation** (P5-03) — New `docs/udp_framing.md` documenting that UDP Transfer Format 1 and 2 headers carry no time fields, and how to use `StreamingTimeCorrelator` for live UDP streams.
- **WASM build verification** (P6-06) — CI now verifies `wasm32-unknown-unknown` compilation with `--no-default-features` and `--no-default-features --features serde`.
- **`util` module** (P6-08) — Crate-internal `is_leap_year` and `abs_diff_u64` helpers that replace `u16::is_multiple_of` (Rust 1.87) and `u64::abs_diff` (Rust 1.60) respectively. Each carries a targeted `#[allow(clippy::...)]` and full MSRV dependency documentation. 18 unit tests covering leap year rules (common, century, quad-century, edge cases, IRIG 106 era years) and abs_diff properties (symmetry, extremes, leap second timestamps).
- **11 new integration tests** in `tests/pipeline.rs`: leap year rollover through `sub_nanos` (leap, non-leap, century, quad-century), BCD DMY Feb 29 on leap/non-leap years, `is_near_leap_second` via `abs_diff_u64` (exact boundary, within window, outside, symmetry, far future).

### Changed

- **`AbsoluteTime` internal representation** (P4-04) — Single `u64` replaces 5 numeric fields. `add_nanos` is now a single `u64` addition (was 4-level carry chain). `sub_nanos` year rollover is cleaner arithmetic. `total_nanos_of_day()` is a single modulo operation.
- **Custom serde for `AbsoluteTime`** — Serializes to the same expanded JSON shape as v0.6 (`day_of_year`, `hours`, `minutes`, `seconds`, `nanoseconds`, `year`, `month`, `day_of_month`) for backward compatibility. Deserializes and recomposes the `u64` internally.
- **GitHub Actions CI** — Added `wasm32-unknown-unknown` build job. Separated stable (full `cargo test`) from MSRV 1.56 (`cargo check` only — dev-dependencies require newer Rust). Tests 4 feature combos on stable + WASM + MSRV check.
- **`Cargo.toml`** — Version bumped to 0.7.0. MSRV lowered from 1.87 to 1.56.
- **CLI (irig106-time-cli)** — Version bumped to 0.7.0. All `AbsoluteTime` field accesses migrated to methods.
- **MSRV lowered from 1.87 → 1.56** (P6-08) — Replaced `u16::is_multiple_of` (Rust 1.87) with `util::is_leap_year` and `u64::abs_diff` (Rust 1.60) with `util::abs_diff_u64`. The MSRV is now the Edition 2021 floor. Each helper carries a targeted clippy `#[allow]` and full documentation of the API it replaces, the Rust version it avoids, and when it can be upgraded.
- **API audit** — `AbsoluteTime` intentionally omits `Ord`/`Hash` because derived `PartialEq` compares all fields including optional year metadata; callers needing within-year ordering should use `as_total_ns()`. To be finalized at 1.0 API freeze.
- **Documentation** — All code examples in `usage.md` updated for method-based access. Version refs `0.6`→`0.7` across all docs.
- All consumer modules migrated: `bcd.rs`, `chrono_interop.rs`, `network_time.rs`, `correlation.rs`, `quality.rs`, `streaming.rs`, and all test files.
- Total test count: **269** (184 unit + 68 integration + 17 property).

## [v0.6.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.6.0) - 2026-03-29

### Added

- **`streaming` module** (P5-02) — `StreamingTimeCorrelator` with sliding-window eviction for live UDP stream processing. Per-channel `BTreeMap` index with O(log n) lookup. Configurable `max_age_ns` window with automatic eviction of stale references. Methods: `add_reference`, `add_reference_f2`, `correlate`, `evict`, `len`, `is_empty`, `total_evicted`, `max_age_ns`, `latest_rtc`, `channel_ids`, `channel_len`. `StreamingRef` struct for reference points.
- **`quality` module** (P5-04) — `TimeQuality` struct and `compute_quality(refs)` function. Metrics: `total_refs`, `channel_count`, `refs_per_channel`, `max_rtc_gap_ns`, `min_rtc_gap_ns`, `ref_density_per_sec`, `drift_ppm_per_channel`, `rtc_span_ns`.
- **`packet_standard` module** (P5-01) — `PacketStandard::Ch10` / `Ch11` enum for IRIG 106-17 chapter provenance. `from_version()`, `is_ch10()`, `is_ch11()`, `Display`.
- **`recording_event` module** (GAP-08) — `RecordingEventType` enum (Started, Stopped, Overrun, IndexPoint, Reserved) and `RecordingEvent` struct with `has_reference_time()` and `may_cause_time_gap()`. Parses Data Type 0x02 event packets for time context.
- **`chrono_interop` module** (GAP-06) — Feature-gated `From<AbsoluteTime> for chrono::NaiveDateTime` and reverse. Enable with `features = ["chrono"]`.
- **F1 leap second handling** (GAP-04) — `LeapSecondTable::offset_for_f1(year, doy)` for Format 1 time sources (IRIG-B, GPS). `is_near_leap_second(unix_seconds, window_secs)` for flagging packets near leap second boundaries.
- **34 new unit tests** across 5 new test files: `packet_standard_tests.rs` (5), `streaming_tests.rs` (10), `quality_tests.rs` (6), `recording_event_tests.rs` (9), `chrono_interop.rs` (4).
- **7 new integration tests** in `tests/pipeline.rs`: PacketStandard from version, streaming correlator basic pipeline, streaming eviction, quality metrics, F1 leap second offset, near leap second boundary, recording event pipeline.

### Changed

- **`Cargo.toml`** — Version bumped to 0.6.0. Added `chrono` optional dependency and feature gate.
- **CLI (irig106-time-cli)** — Version bumped to 0.6.0.
- **GitHub Actions CI** — Added `--features serde` and `--features chrono` individual test runs to catch cross-feature coupling. Pipeline now tests 4 feature combinations: `--all-features`, `--no-default-features`, `--features serde`, `--features chrono`.
- **Crate docs updated** — `lib.rs` feature list now includes streaming correlation, quality metrics, packet standard, recording events, and chrono interop.
- **Re-exports** — `PacketStandard`, `StreamingTimeCorrelator`, `StreamingRef`, `TimeQuality`, `compute_quality`, `RecordingEvent`, `RecordingEventType` added to crate root.
- **P5-05 (async API)** permanently deferred — async runtime choice belongs to the application layer, not a `#![no_std]` parsing library.
- **Rust formatting** — Ran `cargo fmt` across all new modules.
- Test matrix totals: **244** checks under default features (166 unit + 57 integration + 17 property + 4 doc) and **249** with `--all-features` (170 unit + 57 integration + 17 property + 5 doc).

### Fixed

- **`RtcReset.index` now reports global index** — v0.5.0 regression where `detect_rtc_resets` returned per-channel indices instead of global `references()` indices. Fixed via `global_index_of()` helper that matches on all fields.
- **`TimeJump.index` correct for duplicate RTCs** — v0.5.0 regression where `.position()` matched only on `rtc + channel_id`, always returning the first match for duplicate RTC values. Same `global_index_of()` fix.
- **README/lib.rs serde claim** — Now correctly says "except `TimeError`" to match usage.md and CHANGELOG.
- **Clippy fixes** — Removed unused `Rtc` import in `quality.rs`. Added `alloc::format` and `alloc::vec` imports in `no_std` test modules. Replaced manual abs diff with `abs_diff()` in `is_near_leap_second`.
- **chrono doc test** — Replaced `ignore` fence with real ```` ```rust ```` doc test that compiles under the `chrono` feature gate.

## [v0.5.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.5.0) - 2026-03-29

### Added

- **Channel-indexed correlation** (P4-01) — Per-channel `BTreeMap<u16, Vec<ReferencePoint>>` index alongside the flat RTC-sorted vec. `nearest_for_channel` is now O(log n) binary search instead of O(n) linear scan. Methods upgraded: `correlate(…, Some(ch))`, `detect_time_jump`, `drift_ppm`, `detect_rtc_resets`.
- **Per-channel accessors** (P4-02) — `TimeCorrelator::channel_references(channel_id)` returns a sorted slice for a single channel. `channel_ids()` returns all active channel IDs.
- **BCD lookup table** (P4-03) — `extract_bcd_digit` now validates nibbles via a 16-entry const `BCD_LUT` instead of a branch, eliminating conditional logic in the hot path.
- **Criterion benchmarks** (P4-06) — `benches/correlation_bench.rs` with statistical benchmarks for `correlate_any`, `correlate_by_channel`, `detect_time_jump`, `drift_ppm`, and `add_reference` at 10/100/1000/3600 reference point scales.
- **`serde` feature gate** (GAP-02) — Optional `Serialize`/`Deserialize` derives on all public data types (except `TimeError` which contains `&'static str` fields). Enable with `features = ["serde"]`.
- **7 new integration tests** in `tests/pipeline.rs`: channel_references accessor, channel_ids, channel-indexed correlate consistency, large-set correlate, sub_nanos year boundary crossing, same-day no-year-change, and no-year-info graceful wrap.

### Changed

- **`Cargo.toml`** — Version bumped to 0.5.0. Added `serde` optional dependency and feature gate. Added `criterion` to dev-dependencies with `correlation_bench` bench target.
- **CLI (irig106-time-cli)** — Version bumped to 0.5.0.
- **Test structure** — Extracted `version.rs` inline tests to `src/version_tests.rs` to match the crate's test-per-module convention.
- **Crate docs updated** — `lib.rs` feature list now includes channel-indexed O(log n) lookup and serde support.
- Total test count: **203** (136 unit + 50 integration + 17 property).

### Fixed

- **`AbsoluteTime::sub_nanos` year rollover** (GAP-05) — Subtracting past day 1 now correctly decrements the year (accounting for leap years) instead of wrapping to day 366. When no year info is available, assumes 365 days.

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
- **19 new integration tests** in `tests/pipeline.rs`: version detection, version-aware CSDW, OOO window, RTC reset detection (basic, no false positive, channel isolation), and `to_le_bytes` round-trip for all 7 types.
- **4 new property tests** in `tests/properties.rs`: encode round-trip for RTC, CSDW, NTP, and PTP.

### Changed

- **GitHub Actions** — Updated `actions/checkout` from `v4` to `v6` across the CI workflow.
- **Rust formatting** — Ran `cargo fmt` across the repository to normalize code style.
- **Crate docs updated** — `lib.rs` feature list now includes version detection, RTC reset detection, and encoding.
- **Re-exports** — `RtcReset`, `detect_version`, and `Irig106Version` added to crate root.
- **`Cargo.toml`** — Version bumped to 0.4.0.
- **CLI (irig106-time-cli)** — Version bumped to 0.4.0.
- Total test count: **196** (136 unit + 43 integration + 17 property).

### Fixed

- **`detect_rtc_resets` sorted by absolute time** — References are stored sorted by RTC for lookup efficiency. After a reset, the low RTC value was inserted before pre-reset values, masking the discontinuity. Now sorts filtered channel references by absolute time to restore temporal order.
- **Removed unnecessary `as u16` casts** in `DayFormatTime::to_le_bytes()` and `DmyFormatTime::to_le_bytes()` — fields already typed `u16` (`milliseconds`, `day_of_year`, `year`) no longer carry redundant casts.

### Known Issues

- **Legacy support is specification-based only.** Version-aware CSDW parsing and OOO tolerance are implemented per the IRIG 106-04/05 specifications and validated with synthetic tests, but have not been tested against real Ch10 files from legacy recorders (Ampex DCRsi, L-3 MARS, Acra KAM-500). Tracked as P1-07 and P2-05.
- **Ch4 Binary Weighted Time bit layout is unverified against multi-vendor samples.** The current decode assumes a single bit-field interpretation. Real-world validation against multiple recorder vendors is needed. Tracked as GAP-03.

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
