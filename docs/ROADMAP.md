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
| Pre-05 unbounded OOO tolerance | ⚠️ | — | — | — | **Needed** |
| 04/05 CSDW time source delta | ⚠️ | ⚠️ | — | — | **Needed** |
| Version field detection (07+) | — | — | ⚠️ | ⚠️ | **Needed** |
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
| P1-03 | CI/CD pipeline: `cargo test`, `cargo clippy`, `cargo fmt --check` | Critical | 0.5 day | Not started |
| P1-04 | Add `#[deny(missing_docs)]` and complete rustdoc for all public items | High | 1 day | Not started |
| P1-05 | Add `CHANGELOG.md` with keep-a-changelog format | High | 0.5 day | Not started |
| P1-06 | Publish to crates.io as v0.1.0 | High | 0.5 day | Not started |
| P1-07 | Validate against irig106.org sample Ch10 files | High | 1 day | Not started |
| P1-08 | Add `proptest` as optional dev-dependency for richer property tests | Medium | 0.5 day | Not started |

### Phase 2: Legacy Version Support (v0.4.0)

Target: Handle files from 106-04 and 106-05 recorders that are still in archives.

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P2-01 | **CSDW time source version awareness** | High | 1 day | The time source field (bits [3:0]) had slightly different mappings between 106-04 and 106-05. Add `TimeSourceV04` vs `TimeSourceV05` variants, or a version parameter to the CSDW parser. |
| P2-02 | **Pre-105 out-of-order tolerance** | High | 1 day | Current correlator assumes ~1 sec OOO (per 106-05). Files from 106-04 recorders can have packets 5+ seconds out of order. The correlation engine needs a configurable OOO window or a sorting pre-pass. |
| P2-03 | **Version field handling** | Medium | 0.5 day | Add `Irig106Version` enum and `detect_version(tmats_csdw: u32) -> Irig106Version`. Pre-07 files have version=0. Correlator and CSDW parser should accept a version hint. |
| P2-04 | **Version-aware CSDW dispatch** | Medium | 1 day | `TimeF1Csdw::from_raw_versioned(raw: u32, version: Irig106Version)` that applies version-specific field mappings. |
| P2-05 | **Test corpus: real 106-04/05/07 files** | High | 1 day | Acquire or synthesize Ch10 files for each version and add them to the fuzz corpus and integration tests. |

### Phase 3: Time Data Format 2 — Network Time ✅ COMPLETE (v0.2.0)

Delivered in v0.2.0. All items complete:

| ID | Item | Status |
|----|------|--------|
| P3-01 | Time F2 CSDW parser (0x12) | ✅ `TimeF2Csdw` in `network_time.rs` |
| P3-02 | PTPv2 time message decoding | ✅ `PtpTime` with 48-bit seconds + 32-bit nanos |
| P3-03 | NTP time message decoding | ✅ `NtpTime` with fractional → nanos conversion |
| P3-04 | TAI ↔ UTC offset handling | ✅ `LeapSecondTable` with 28 built-in entries |
| P3-05 | F2 correlation integration | ✅ `TimeCorrelator::add_reference_f2()` |
| P3-06 | L1/L2/L3 requirements for F2 | ✅ 16 L1 + full L2/L3 addendum |
| P3-07 | Fuzz targets for F2 parsers | ✅ `fuzz_ntp`, `fuzz_ptp` |
| P3-08 | Update CLI tool | ✅ `ch10time` handles 0x12 packets |

### Phase 4: Performance Optimizations (v0.5.0)

Target: Reduce the hot path below 15 ns and optimize channel-filtered correlation.

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P4-01 | **Channel-indexed correlation** | High | 2 days | Current `nearest_for_channel` is O(n) linear scan (304 ns at 100 refs). Replace with per-channel sorted Vec or BTreeMap for O(log n) lookup. Target: ~15 ns. |
| P4-02 | **Cached jump detection** | Medium | 1 day | `detect_time_jump` currently rescans all refs every call (303 µs at 3600 refs). Cache per-channel sorted slices and invalidate on insert. |
| P4-03 | **BCD lookup table** | Low | 0.5 day | Replace per-nibble branch validation with a 256-byte LUT mapping byte → (tens, units, valid). Marginal gain (~1-2 ns) but eliminates branches. |
| P4-04 | **`AbsoluteTime` as total nanoseconds** | Medium | 2 days | Consider internal representation as a single `u64` (nanos since day 0 midnight) with lazy field extraction. Would make `add_nanos`/`sub_nanos` a single add/sub instead of multi-field carry chain. Breaking API change — evaluate carefully. |
| P4-05 | **SIMD BCD decode** | Low | 1 day | Experimental: use SIMD to decode all 4 BCD words in parallel. Only worthwhile if BCD decode shows up as a bottleneck in production profiling. |
| P4-06 | **Benchmark with criterion** | Medium | 0.5 day | Upgrade to criterion when Rust toolchain permits (currently blocked by Rust 1.75 in sandbox). Statistical benchmarks with confidence intervals. |

### Phase 5: Chapter 11 and Streaming Support (v0.6.0)

Target: Handle Ch11 packet formats and real-time UDP stream correlation.

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P5-01 | **Ch11 packet format awareness** | Medium | 1 day | IRIG 106-17 moved packet format definitions to Chapter 11. The wire format is identical for time fields, but software should recognize the provenance. Add `PacketStandard::Ch10` / `Ch11` metadata. |
| P5-02 | **Streaming correlator** | High | 3 days | A `StreamingTimeCorrelator` that accepts packets in arrival order (potentially out-of-order), maintains a sliding window of reference points, and garbage-collects old references. Needed for live UDP stream processing. |
| P5-03 | **UDP transfer header time handling** | Medium | 1 day | UDP Transfer Format 1 and 2 headers have sequence numbers but no additional time fields. Document the interaction between UDP framing and time correlation. |
| P5-04 | **Time quality metrics** | Medium | 2 days | Track metrics: reference point density (refs/sec), max RTC gap between references, drift estimate (ppm), time source availability per channel. Expose via `TimeCorrelator::quality()`. |
| P5-05 | **Async correlation API** | Low | 1 day | Optional `async` API for use in tokio-based streaming pipelines. Feature-gated behind `async` feature. |

### Phase 6: Ecosystem Integration (v1.0.0)

Target: Stable API, full ecosystem wiring, migration to `irig106-types`.

| ID | Item | Priority | Effort | Details |
|----|------|----------|--------|---------|
| P6-01 | **Migrate shared types to `irig106-types`** | High | 2 days | Move `Rtc`, `Ertc`, `Ch4BinaryTime`, `Ieee1588Time`, `TimeSource`, `TimeFormat`, `DateFormat` to the foundational crate. Re-export for backward compat. |
| P6-02 | **Wire `irig106-core` to use `irig106-time` types** | High | 1 day | `irig106-core::PacketHeader` should use `Rtc` from `irig106-types` instead of raw `u64`. |
| P6-03 | **Wire `irig106-ch10-reader` to use correlation** | High | 2 days | Replace the "Not available" time display in the reader with actual correlated times using `TimeCorrelator`. |
| P6-04 | **Wire `irig106-decode` intra-packet timestamps** | High | 1 day | Payload decoders should use `IntraPacketTime` for message-level timestamps. |
| P6-05 | **Wire `irig106-write` BCD encoding** | Medium | 1 day | Add `DayFormatTime::to_le_bytes()` and `DmyFormatTime::to_le_bytes()` for serialization. Verify round-trip with existing decode tests. |
| P6-06 | **Wire `irig106-studio` WASM** | Medium | 1 day | Verify `#![no_std]` + `alloc` compiles to wasm32-unknown-unknown. Add WASM-specific tests. |
| P6-07 | **Semantic versioning freeze** | High | — | Declare 1.0.0 stable API. No breaking changes without major version bump. |
| P6-08 | **MSRV policy** | Medium | — | Declare Minimum Supported Rust Version (suggest 1.70 for broad compat). |

---

## 3. Known Gaps and Technical Debt

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| GAP-01 | No `Display` or formatting for `AbsoluteTime` | Medium | CLI formats manually. Should have `impl Display` with ISO-like output. |
| GAP-02 | No `Serialize`/`Deserialize` (serde) | Low | Useful for JSON/CSV export. Feature-gate behind `serde` feature. |
| GAP-03 | Ch4 BinaryTime decode is simplified | Medium | The combined high/low word interpretation assumes a specific bit layout. Need to validate against real Ch4 BWT samples from multiple recorder vendors. |
| GAP-04 | No leap second handling | Medium | IRIG time codes can include leap second information. GPS time does not have leap seconds. UTC does. The crate currently ignores this. |
| GAP-05 | `AbsoluteTime::sub_nanos` doesn't handle year rollover | Low | Subtracting past day 1 wraps to day 366. Need year-aware arithmetic for multi-day recordings. |
| GAP-06 | No `From`/`Into` conversions to `chrono` or `time` crates | Low | Optional feature-gated interop with popular Rust time libraries. |
| GAP-07 | Correlation doesn't handle RTC reset mid-recording | Medium | Some recorders reset the RTC (not just wrap). Need a heuristic to detect resets vs. wraps. |
| GAP-08 | No support for time data in Ch10 Recording Events (0x02) | Low | Event packets carry timestamps that could be used as additional correlation points. |
| GAP-09 | `DmyFormatTime::to_absolute` day-of-year calculation doesn't validate day-for-month | Low | Feb 30 would be accepted. Need calendar validation. |
| GAP-10 | No RTC drift estimation | Medium | Given two reference points, drift_ppm could be computed. Useful for quality assessment. |
| GAP-11 | Missing `to_le_bytes()` (encode) for BCD and CSDW types | Medium | Needed by `irig106-write` for time packet construction. |

---

## 4. External Dependencies to Watch

| Item | Why | When |
|------|-----|------|
| IRIG 106-24/25 release | May add new time formats or modify existing ones | ~2024-2025 (check RCC site) |
| IEEE 1588-2019 (PTPv2) adoption in recorders | Drives urgency of Phase 3 | Active — some vendors already shipping |
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
| **0.3.0** | Phase 1 | CI/CD, rustdoc, crates.io publish, sample file validation | +2 weeks |
| **0.4.0** | Phase 2 | 106-04/05 legacy support, version-aware parsing | +1 month |
| **0.5.0** | Phase 4 | Channel-indexed correlation, perf optimizations | +2 months |
| **0.6.0** | Phase 5 | Streaming correlator, Ch11 awareness, quality metrics | +3 months |
| **1.0.0** | Phase 6 | Stable API, ecosystem wiring, irig106-types migration | +5 months |
