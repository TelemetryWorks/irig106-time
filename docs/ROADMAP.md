# Roadmap — irig106-time

**Document:** ROADMAP.md
**Crate:** irig106-time
**Date:** 2026-03-26

---

## 1. IRIG 106 Version Support Matrix

The standard has been revised roughly every two years since 2004. Real-world
Ch10 files span the full range. The version is encoded in the TMATS CSDW's
"IRIG 106 Chapter 10 Version" field (only defined from 106-07 onward; files
before 106-07 have this field as zero).

### 1.1 Version History and Time-Relevant Changes

| Version | Year | Time-Relevant Changes | Impact on `irig106-time` |
|---------|------|----------------------|--------------------------|
| **106-04** | 2004 | Initial Ch10. Time Source CSDW field defined. No IRIG version field in header. No packet ordering constraint. | Must handle version=0 (pre-07). Unbounded out-of-order packets in correlation. |
| **106-05** | 2005 | Time Source field slightly changed. 100 ms buffer limit + 1 sec write deadline added. Secondary header IEEE-1588 time added. | CSDW time source mapping may differ. Out-of-order limited to ~1 sec post-05. |
| **106-07** | 2007 | IRIG 106 Version field added to TMATS CSDW. Multiple TMATS packets allowed. XML TMATS introduced. ERTC (64-bit) secondary header format added. | Version-aware parsing enabled. ERTC support needed. |
| **106-09** | 2009 | Clarifications only. No time format changes. | No impact. |
| **106-11** | 2011 | Minor clarifications. | No impact. |
| **106-13** | 2013 | Minor clarifications. | No impact. |
| **106-15** | 2015 | CAN Bus data type added (0x78). No time format changes. | No impact on time module. |
| **106-17** | 2017 | **Chapter 10/11 split.** Recorder packet formats moved to Chapter 11. Chapter 10 retains recorder operations. New data types. No time format changes. | Crate works with both Ch10 and Ch11 packet formats (time fields identical). |
| **106-19** | 2019 | UDP Transfer Format 2 added. No time format changes. | No impact on time parsing. UDP framing is handled upstream. |
| **106-22** | 2022 | **Time Data Format 2 (0x12) — Network Time** added. PTPv2 (IEEE 1588-2019) time. | **NEW**: Must parse 0x12 CSDW and NTP/PTP time message format. |
| **106-23** | 2023 | No Chapter 10 changes. | No impact. |

### 1.2 Current Support Status

| Feature | 106-04 | 106-05 | 106-07+ | 106-22+ | Status |
|---------|--------|--------|---------|---------|--------|
| 48-bit RTC | ✅ | ✅ | ✅ | ✅ | **Done** |
| Time F1 CSDW (0x11) | ✅ | ✅ | ✅ | ✅ | **Done** |
| BCD Day-of-Year | ✅ | ✅ | ✅ | ✅ | **Done** |
| BCD Day-Month-Year | ✅ | ✅ | ✅ | ✅ | **Done** |
| Secondary hdr Ch4 BWT | ✅ | ✅ | ✅ | ✅ | **Done** |
| Secondary hdr IEEE-1588 | — | ✅ | ✅ | ✅ | **Done** |
| Secondary hdr ERTC | — | — | ✅ | ✅ | **Done** |
| RTC correlation engine | ✅ | ✅ | ✅ | ✅ | **Done** |
| Multi-channel correlation | ✅ | ✅ | ✅ | ✅ | **Done** |
| Time jump detection | ✅ | ✅ | ✅ | ✅ | **Done** |
| Pre-05 unbounded OOO tolerance | ✅ | — | — | — | **Done (v0.4.0)** |
| 04/05 CSDW time source delta | ✅ | ✅ | — | — | **Done (v0.4.0)** |
| Version field detection (07+) | — | — | ✅ | ✅ | **Done (v0.4.0)** |
| Time Data Format 2 (0x12) | — | — | — | ✅ | **Done (v0.2.0)** |
| Ch11 packet format awareness | — | — | — | ⚠️ | **Planned** |

---

## 2. Phased Roadmap

### Phase 1: Production Hardening (v0.3.0)

Target: Make the existing implementation bulletproof for 106-07+ files.

| ID | Item | Priority | Effort | Status |
|----|------|----------|--------|--------|
| P1-01 | Run all 10 fuzz targets for 1 hour each on real hardware | Critical | 1 day | Ready |
| P1-02 | Run benchmarks on target NVMe hardware, document baseline | Critical | 0.5 day | Ready |
| P1-03 | CI/CD pipeline: `cargo test`, `cargo clippy`, `cargo fmt --check` | Critical | 0.5 day | ✅ Done (v0.3.0) |
| P1-04 | `#[deny(missing_docs)]` and complete rustdoc for all public items | High | 1 day | ✅ Done (v0.3.0) |
| P1-05 | ~~Add `CHANGELOG.md` with keep-a-changelog format~~ | — | — | ✅ Done (v0.1.0) |
| P1-06 | Publish to crates.io | High | 0.5 day | ✅ Done (v0.2.0) |
| P1-07 | Validate against irig106.org sample Ch10 files | High | 1 day | Not started |
| P1-08 | `proptest` property-based tests | Medium | 0.5 day | ✅ Done (v0.3.0) |
| P1-09 | **CLI: distinguish NTP/PTP in channel display** | Medium | 0.5 day | ✅ Done (v0.3.0) |

### Phase 2: Legacy Version Support (v0.4.0)

Target: Handle files from 106-04 and 106-05 recorders that are still in archives.

| ID | Item | Priority | Effort | Status |
|----|------|----------|--------|--------|
| P2-01 | **CSDW time source version awareness** | High | 1 day | ✅ Done (v0.4.0) |
| P2-02 | **Pre-105 out-of-order tolerance** | High | 1 day | ✅ Done (v0.4.0) |
| P2-03 | **Version field handling** | Medium | 0.5 day | ✅ Done (v0.4.0) |
| P2-04 | **Version-aware CSDW dispatch** | Medium | 1 day | ✅ Done (v0.4.0) |
| P2-05 | **Test corpus: real 106-04/05/07 files** | High | 1 day | Not started |

**P2-05 Detail — Legacy File Validation:**

Validation of the version-aware CSDW parsing and OOO tolerance requires real Ch10 files
from pre-07 recorders. Known legacy recorder models that produced 106-04/05 files:

- **Ampex DCRsi** — widely deployed in early 2000s flight test programs
- **L-3 Communications MARS** (Multi-channel Airborne Recording System) — common in DoD programs
- **Curtiss-Wright / Acra KAM-500** — early generation units before 106-07 firmware updates

Action items:
1. Search flight test data archives for Ch10 files with TMATS version field = 0
2. If real files unavailable, synthesize minimal valid 106-04-era packets with known time values and a version=0 TMATS CSDW
3. Add Ch4 BWT samples from the same recorders to validate GAP-03 (bit layout)
4. Add validated files to fuzz corpus and integration test suite

### Phase 3: Time Data Format 2 — Network Time ✅ COMPLETE (v0.2.0)

Delivered in v0.2.0. All items complete:

| ID | Item | Status |
|----|------|--------|
| P3-01 | Time F2 CSDW parser (0x12) | ✅ `TimeF2Csdw` in `network_time.rs` |
| P3-02 | PTPv2 time message decoding | ✅ `PtpTime` with 48-bit seconds + 32-bit nanos |
| P3-03 | NTP time message decoding | ✅ `NtpTime` with fractional → nanos conversion |
| P3-04 | TAI ↔ UTC offset handling | ✅ `LeapSecondTable` with 28 built-in entries |
| P3-05 | F2 correlation integration | ✅ `TimeCorrelator::add_reference_f2()` |
| P3-06 | L1/L2/L3 requirements for F2 | ✅ 16 L1 merged into L1_Requirements.md + L2/L3 addendum |
| P3-07 | Fuzz targets for F2 parsers | ✅ `fuzz_ntp`, `fuzz_ptp` |
| P3-08 | Update CLI tool | ✅ `ch10time` handles 0x12 packets |

### Phase 4: Performance Optimizations (v0.5.0)

Target: Reduce the hot path below 15 ns and optimize channel-filtered correlation.

| ID | Item | Priority | Effort | Status |
|----|------|----------|--------|--------|
| P4-01 | **Channel-indexed correlation** | High | 2 days | ✅ Done (v0.5.0) |
| P4-02 | **Cached jump detection** | Medium | 1 day | ✅ Done (v0.5.0) |
| P4-03 | **BCD lookup table** | Low | 0.5 day | ✅ Done (v0.5.0) |
| P4-04 | **`AbsoluteTime` as total nanoseconds** | Medium | 2 days | ✅ Done (v0.7.0) |
| P4-05 | **SIMD BCD decode** | Low | 1 day | Deferred (speculative, no profiling evidence) |
| P4-06 | **Benchmark with criterion** | Medium | 0.5 day | ✅ Done (v0.5.0) |

### Phase 5: Chapter 11 and Streaming Support (v0.6.0)

Target: Handle Ch11 packet formats and real-time UDP stream correlation.

| ID | Item | Priority | Effort | Status |
|----|------|----------|--------|--------|
| P5-01 | **Ch11 packet format awareness** | Medium | 1 day | ✅ Done (v0.6.0) |
| P5-02 | **Streaming correlator** | High | 3 days | ✅ Done (v0.6.0) |
| P5-03 | **UDP transfer header time handling** | Medium | 1 day | ✅ Done (v0.7.0) — `docs/udp_framing.md` |
| P5-04 | **Time quality metrics** | Medium | 2 days | ✅ Done (v0.6.0) |
| P5-05 | **Async correlation API** | — | — | Won't do — application concern, not library concern. The `StreamingTimeCorrelator` provides synchronous `&mut self` methods that work naturally in async contexts via `Arc<Mutex<_>>` or channel-based patterns. Adding `tokio` to a `#![no_std]` parsing library would violate layering and contaminate the dependency tree. |

### Phase 6: Ecosystem Integration (v0.8.0)

Target: Wire `irig106-time` into the TelemetryWorks crate ecosystem. This phase
**must complete before 1.0.0** because integration may surface API issues that
require breaking changes. Better to discover them now than after the semver freeze.

#### Dependency Chain

```
P6-01: irig106-types         (no dependencies — start here)
  │
  ├─► P6-02: irig106-core    (depends on P6-01)
  │     │
  │     ├─► P6-03: irig106-ch10-reader  (depends on P6-02)
  │     │
  │     └─► P6-04: irig106-decode       (depends on P6-02)
  │
  └─► P6-06b: irig106-studio WASM integration  (depends on P6-01)
```

#### Work Items

Each item has work in **two repos**: the downstream crate (consumer) and
`irig106-time` itself (provider). Changes surfaced in the provider during
integration feed into Phase 7 (P7-01).

| ID | Item | Priority | Effort | Depends On |
|----|------|----------|--------|------------|
| P6-01 | **Migrate shared types to `irig106-types`** | High | 2 days | — |
| P6-02 | **Wire `irig106-core`** | High | 1 day | P6-01 |
| P6-03 | **Wire `irig106-ch10-reader`** | High | 2 days | P6-02 |
| P6-04 | **Wire `irig106-decode`** | High | 1 day | P6-02 |
| P6-05 | ~~Wire `irig106-write` BCD encoding~~ | — | — | ✅ `to_le_bytes()` shipped in v0.4.0. |
| P6-06a | **WASM build verification** | Medium | — | ✅ Done (v0.7.0) — CI verifies `wasm32-unknown-unknown` build. |
| P6-06b | **`irig106-studio` WASM integration** | Medium | 1 day | P6-01 |
| P6-08 | **MSRV policy** | Medium | — | ✅ Done (v0.7.0). MSRV 1.87 → 1.60. Replaced `u16::is_multiple_of` (1.87) with `util::is_leap_year` and `u64::abs_diff` (1.60) with `util::abs_diff_u64`. Constrained by `dep:` namespaced features in `Cargo.toml`. |

#### Detailed Scope Per Item

**P6-01 — Migrate shared types to `irig106-types`**

| Side | Work |
|------|------|
| `irig106-types` (new crate) | Create crate. Define `Rtc`, `Ertc`, `Ch4BinaryTime`, `Ieee1588Time`, `TimeSource`, `TimeFormat`, `DateFormat`. Carry all derives (`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, serde). |
| `irig106-time` | Add `irig106-types` dependency. Replace local type definitions with re-exports (`pub use irig106_types::Rtc;`). Verify all existing tests pass — this must be a drop-in replacement. |

**P6-02 — Wire `irig106-core`**

| Side | Work |
|------|------|
| `irig106-core` | Change `PacketHeader` to use `Rtc` from `irig106-types` instead of raw `u64`. Update packet parsing to construct `Rtc` values. |
| `irig106-time` | Verify that `Rtc::from_le_bytes` and `Rtc::from_raw` signatures work for `irig106-core`'s use case. If not, fix them here (tracked as P7-01). |

**P6-03 — Wire `irig106-ch10-reader`**

| Side | Work |
|------|------|
| `irig106-ch10-reader` | Add `irig106-time` dependency. Build a `TimeCorrelator` during file iteration. Replace "Not available" time display with correlated absolute times. |
| `irig106-time` | Verify `TimeCorrelator` API is ergonomic for the reader's iteration pattern. May need new convenience methods (e.g., `correlate_or_raw`). If so, add them here. |

**P6-04 — Wire `irig106-decode`**

| Side | Work |
|------|------|
| `irig106-decode` | Use `IntraPacketTime` and `IntraPacketTimeFormat` for message-level timestamps in payload decoders (1553, ARINC 429, etc.). |
| `irig106-time` | Verify `parse_intra_packet_time` works with real decoder output. May need additional `IntraPacketTime` methods for downstream formatting. |

**P6-06b — `irig106-studio` WASM integration**

| Side | Work |
|------|------|
| `irig106-studio` | Import `irig106-time` as WASM module. Load a Ch10 file, parse time packets, correlate data packet timestamps, display in the UI. |
| `irig106-time` | P6-06a (WASM build check) is already done. Integration may reveal `alloc` usage patterns that don't work in the browser. Fix here if needed. |

### Phase 7: Validation, Hardening, and Security (v0.9.0)

Target: Prove the crate works against real-world data, fix any API issues
surfaced by Phase 6 ecosystem integration, and produce the evidence artifacts
required for the cyber security report. This is the last opportunity for
breaking changes before the 1.0.0 semver freeze.

#### 7A. Real-World Data Validation

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P1-07 | **Validate against irig106.org sample files** | High | 1 day | Parse sample Ch10 files from irig106.org and verify round-trip correctness. |
| P2-05 | **Legacy file corpus** | High | 1 day | Acquire or synthesize 106-04/05/07 files from Ampex DCRsi, L-3 MARS, Acra KAM-500. Validate version-aware parsing. |
| GAP-03 | **Ch4 BWT multi-vendor validation** | Medium | 0.5 day | Validate Ch4 BinaryTime bit layout against real samples from multiple recorder vendors. Blocked on P2-05. |
| P7-01 | **Fix API issues from ecosystem integration** | High | TBD | Any breaking changes required by P6-01 through P6-04 integration feedback. |
| P7-02 | **Integration test suite with real files** | High | 1 day | End-to-end tests using real Ch10 files from P1-07 and P2-05 corpora. |

#### 7B. Performance Validation

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P1-02 | **Benchmark baseline on target hardware** | Critical | 0.5 day | Run criterion + zero-dep benchmarks on NVMe workstation. Save baseline with `cargo bench -- --save-baseline v0.9.0`. Document machine spec and results in `docs/benchmark_results.md`. |
| P7-B01 | **Regression check vs. v0.7.0 baseline** | High | 0.5 day | Compare v0.9.0 performance against v0.7.0 baseline. Document any regressions > 5% on hot-path benchmarks. If regressions exist, either fix or justify in the security report. |

#### 7C. Security Analysis and Cyber Report

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P1-01 | **Run all 10 fuzz targets for 1 hour each** | Critical | 1 day | Document results in `docs/fuzz_report.md` using the template in `CONTRIBUTING.md`. Generate fuzz coverage report. |
| P7-S01 | **`cargo audit` — dependency vulnerability scan** | High | 0.25 day | Run `cargo audit` and `cargo deny check`. Zero advisories required. Include output in security report. |
| P7-S02 | **`cargo geiger` — unsafe code audit** | High | 0.25 day | Confirm zero `unsafe` blocks in library code. Include output in security report. |
| P7-S03 | **Code coverage measurement** | High | 0.5 day | Run `cargo-tarpaulin` or `cargo-llvm-cov`. Target: > 80% line coverage on library code. Document per-module breakdown. Generate LCOV for SonarQube. |
| P7-S04 | **SonarQube / SonarCloud analysis** | High | 0.5 day | Run SonarQube static analysis. Capture: code smells, complexity metrics, duplication, security hotspots. Export dashboard for security report. |
| P7-S05 | **Produce cyber security report** | Critical | 1 day | Compile `docs/security_report.md` with sections: executive summary, static analysis, dependency audit, fuzz testing, code coverage, unsafe audit, VCRM summary, benchmark baseline. See `CONTRIBUTING.md` for the full report outline. |

All security tools are run **locally on demand** (not in CI) to manage cost and
nightly-toolchain requirements. Results are committed as report artifacts in `docs/`.
See `CONTRIBUTING.md` § "Security Analysis and Reporting" for tool installation
and usage instructions.

### Phase 8: Stable Release (1.0.0)

Target: Declare stable API with complete documentation, full requirements
traceability, and architecture diagrams suitable for hosting on docs.rs.
No breaking changes without major version bump after this release.

#### 8A. Documentation (hosted on docs.rs)

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P8-D01 | **Crate-level rustdoc overview** | High | 1 day | Expand `lib.rs` module docs into a comprehensive introduction rooted in IRIG 106 Chapter 10 concepts. Start with "what is IRIG 106", "what is a Ch10 file", "what is an RTC", then walk through the crate's modules and how they map to the standard. |
| P8-D02 | **Per-module rustdoc with examples** | High | 2 days | Every `pub mod` gets a module-level doc block with IRIG 106 context, a "Quick Start" code example, and cross-references to related modules. Every `pub fn` and `pub struct` gets at least one `/// # Example` doc test. |
| P8-D03 | **Architecture diagrams** | High | 1 day | Visual diagrams embedded in rustdoc (SVG or Mermaid rendered to SVG). Diagrams include: (1) Module dependency graph, (2) Data flow from raw Ch10 bytes → wire format → AbsoluteTime → correlated output, (3) Correlation engine internals (reference points, interpolation, channel indexing), (4) Time Data Format 1 vs Format 2 pipeline comparison. |
| P8-D04 | **IRIG 106 primer in docs** | Medium | 1 day | Standalone `docs/irig106_primer.md` (and linked from rustdoc) covering: RTC semantics, time packet types (0x11, 0x12), BCD encoding, secondary headers, intra-packet timestamps. Written for developers who have never seen a Ch10 file. |
| P8-D05 | **Usage guide finalization** | Medium | 0.5 day | Review and finalize `docs/usage.md` with complete end-to-end examples for every major workflow. Ensure all code examples compile as doc tests. |

#### 8B. Requirements Traceability and VCRM

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P8-V01 | **L1/L2/L3 requirements audit** | High | 1 day | Review all three requirements documents. Ensure every public function, method, struct, and enum has a traced L3 requirement. Add missing requirements for v0.6/v0.7 features (streaming, quality, recording events, util helpers). |
| P8-V02 | **Requirement → Code forward trace** | High | 1 day | For every L3 requirement, verify at least one code item (`pub fn`, `pub struct`, `impl`) carries a `/// **Traces:** L3-XXX-NNN` annotation. Produce a forward-trace table: requirement → source file → line. |
| P8-V03 | **Code → Test reverse trace** | High | 1 day | For every `pub fn`/`pub struct`, verify at least one test exercises it and carries a `Traces` annotation in the test module's doc table. Produce a reverse-trace table: source item → test name → test file. |
| P8-V04 | **Verification Cross-Reference Matrix (VCRM)** | Critical | 2 days | A single document (`docs/VCRM.md`) connecting the full chain: L1 → L2 → L3 → Source (file:line) → Test (file:test_name) → Test Type (unit/integration/property/fuzz). Every row is a requirement; every column is a verification artifact. No empty cells — if a requirement has no test, that's a gap to fill. |
| P8-V05 | **VCRM gap closure** | High | TBD | Fill any gaps identified by P8-V04. Add missing tests, missing requirements, or missing trace annotations until the VCRM has zero empty cells. |

#### 8C. Release

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P6-07 | **Semantic versioning freeze** | High | — | Declare 1.0.0 stable API after all ecosystem wiring, validation, documentation, and VCRM are complete. |
| P8-R01 | **Final API review** | High | 0.5 day | Confirm all public types, methods, and error variants are correct after Phase 6/7 feedback. |
| P8-R02 | **1.0.0 release notes** | Medium | 0.5 day | Comprehensive release notes covering the full journey from 0.1.0 to 1.0.0. |

---

## 3. Known Gaps and Technical Debt

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| GAP-01 | ~~No `Display` for `AbsoluteTime`~~ | — | ✅ Resolved (v0.3.0). `impl Display` with ISO-like output. |
| GAP-02 | ~~No `Serialize`/`Deserialize` (serde)~~ | — | ✅ Resolved (v0.5.0). Feature-gated behind `serde` feature on all public data types except `TimeError`. |
| GAP-03 | Ch4 BinaryTime decode is simplified | Medium | The combined high/low word interpretation assumes a specific bit layout. Need to validate against real Ch4 BWT samples from legacy recorders (Ampex DCRsi, L-3 MARS, Acra KAM-500). Blocked on P2-05 file corpus. |
| GAP-04 | ~~No leap second handling for Format 1 time sources~~ | — | ✅ Resolved (v0.6.0). `LeapSecondTable::offset_for_f1(year, doy)` and `is_near_leap_second(unix_seconds, window_secs)`. |
| GAP-05 | ~~`AbsoluteTime::sub_nanos` doesn't handle year rollover~~ | — | ✅ Resolved (v0.5.0). Correctly decrements year across day-1 boundary with leap-year-aware day count. |
| GAP-06 | ~~No `From`/`Into` conversions to `chrono` or `time` crates~~ | — | ✅ Resolved (v0.6.0). Feature-gated `chrono` interop with `From<AbsoluteTime>` and reverse. |
| GAP-07 | ~~Correlation doesn't handle RTC reset mid-recording~~ | — | ✅ Resolved (v0.4.0). `TimeCorrelator::detect_rtc_resets(channel_id)` with `RtcReset` struct. |
| GAP-08 | ~~No support for time data in Ch10 Recording Events (0x02)~~ | — | ✅ Resolved (v0.6.0). `RecordingEvent` and `RecordingEventType` in `recording_event` module. |
| GAP-09 | ~~DMY `to_absolute` doesn't validate day-for-month~~ | — | ✅ Resolved (v0.3.0). `days_in_month()` rejects Feb 30, Jun 31, etc. |
| GAP-10 | ~~No RTC drift estimation~~ | — | ✅ Resolved (v0.3.0). `TimeCorrelator::drift_ppm(channel_id)`. |
| GAP-11 | ~~Missing `to_le_bytes()` (encode) for BCD and CSDW types~~ | — | ✅ Resolved (v0.4.0). `to_le_bytes()` on `Rtc`, `TimeF1Csdw`, `TimeF2Csdw`, `NtpTime`, `PtpTime`, `DayFormatTime`, `DmyFormatTime`. |
| GAP-12 | ~~CLI channel display loses NTP/PTP protocol identity~~ | — | ✅ Resolved (v0.3.0). `network_protocol` field + Proto column in channels table. |
| GAP-13 | ~~`lib.rs` crate docs don't mention `network_time`~~ | — | ✅ Resolved (v0.3.0). |
| GAP-14 | ~~Pub items missing `///` doc comments~~ | — | ✅ Resolved (v0.3.0). `#[deny(missing_docs)]` enforced. |
| GAP-15 | ~~`unix_seconds_to_ymd_pub` is `pub` but should be crate-internal~~ | — | ✅ Resolved (v0.3.0). Changed to `pub(crate)`. |
| GAP-16 | ~~`unix_seconds_to_ymd` can overflow `u16` year~~ | — | ✅ Resolved (v0.3.0). `saturating_add` + `u16::MAX` guard. |
| GAP-17 | **Future: `YearDoyTime` intermediate type** | Low | `AbsoluteTime` is a DOY-based timestamp where year is metadata, not a calendar proof. `CalendarTime` is the fully-validated calendar type. A `YearDoyTime` (DOY + year, validated that DOY fits that year — e.g., DOY 366 only in leap years) would sit between them. Not worth the API complexity unless year-sensitive logic grows significantly. Revisit during Phase 8 API review. |

---

## 4. External Dependencies to Watch

| Item | Why | When |
|------|-----|------|
| IRIG 106-24/25 release | May add new time formats or modify existing ones | ~2024-2025 (check RCC site) |
| IEEE 1588-2019 (PTPv2) adoption in recorders | PTP support delivered in v0.2.0 | Done |
| Chapter 11 adoption rate | Drives urgency of Phase 5 | Increasing — IRIG 106-17+ systems |
| `irig106-types` crate readiness | Blocks Phase 6 shared type migration | When you build out that crate |
| Rust edition 2024 stabilization | Enables newer Cargo features, edition bump | Rust 1.85+ |
| WebAssembly threads proposal | Could enable parallel correlation in `irig106-studio` | Browser support TBD |

---

## 5. Version Release Plan

| Version | Phase | Key Deliverables | Target |
|---------|-------|-----------------|--------|
| **0.1.0** | — | Initial release: 8 modules, 124 tests, benchmarks, fuzz targets | Released |
| **0.2.0** | Phase 3 | Time Data Format 2 (0x12), NTP, PTPv2, LeapSecondTable, correlator F2 integration | Released |
| **0.3.0** | Phase 1 | CI/CD, `#[deny(missing_docs)]`, proptest, `Display`, drift_ppm, calendar validation, CLI Proto column | Released |
| **0.4.0** | Phase 2 | Version detection, version-aware CSDW, OOO window, RTC reset detection, `to_le_bytes()` encoding | Released |
| **0.5.0** | Phase 4 | Channel-indexed O(log n) correlation, BCD LUT, criterion benchmarks, serde, sub_nanos year fix | Released |
| **0.6.0** | Phase 5 | Streaming correlator, Ch11 awareness, quality metrics, recording events, chrono interop, F1 leap seconds | Released |
| **0.7.0** | Pre-1.0 | AbsoluteTime u64 restructure (P4-04), MSRV 1.87→1.60 (P6-08), WASM CI (P6-06), UDP docs (P5-03), API audit (Hash/Copy on 25+ types) | Current |
| **0.8.0** | Phase 6 | Ecosystem wiring: irig106-types migration, irig106-core/decode/reader integration | Next |
| **0.9.0** | Phase 7 | Validation: real-file testing, fuzz/benchmark on hardware, security analysis (SonarQube, cargo-audit, coverage), cyber security report | Planned |
| **1.0.0** | Phase 8 | Stable API: complete rustdoc with architecture diagrams, VCRM with zero gaps, IRIG 106 primer, semver freeze | Planned |
