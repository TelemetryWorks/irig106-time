# Project Structure - irig106-time

**Date:** 2026-03-29

---

## Repository Layout

```text
irig106-time/
|-- Cargo.toml                         # Zero required deps, std feature gate
|-- CHANGELOG.md                       # Keep-a-changelog format
|-- LICENSE                            # Apache-2.0
|-- README.md
|
|-- src/
|   |-- lib.rs                         # #![no_std], module wiring, re-exports
|   |-- error.rs                       # TimeError enum, Result<T>, Display
|   |-- error_tests.rs                 # 7 unit tests
|   |-- rtc.rs                         # Rtc newtype (48-bit, 10 MHz)
|   |-- rtc_tests.rs                   # 18 unit tests
|   |-- absolute.rs                    # AbsoluteTime, Ch4BinaryTime, Ieee1588Time, Ertc
|   |-- absolute_tests.rs              # 23 unit tests
|   |-- csdw.rs                        # TimeF1Csdw, TimeSource, TimeFormat, DateFormat
|   |-- csdw_tests.rs                  # 14 unit tests
|   |-- bcd.rs                         # DayFormatTime, DmyFormatTime (BCD wire format)
|   |-- bcd_tests.rs                   # 13 unit tests
|   |-- secondary.rs                   # Secondary header time, checksum validation
|   |-- secondary_tests.rs             # 10 unit tests
|   |-- intra_packet.rs                # IntraPacketTime, 4-format dispatch
|   |-- intra_packet_tests.rs          # 8 unit tests
|   |-- correlation.rs                 # TimeCorrelator, ReferencePoint, TimeJump, RtcReset
|   |-- correlation_tests.rs           # 11 unit tests
|   |-- network_time.rs                # Format 2 (0x12): NTP, PTP, LeapSecondTable
|   |-- network_time_tests.rs          # 22 unit tests
|   |-- version.rs                     # Irig106Version, detect_version
|   |-- version_tests.rs               # 10 unit tests
|   |-- packet_standard.rs             # PacketStandard::Ch10/Ch11
|   |-- packet_standard_tests.rs       # 5 unit tests
|   |-- streaming.rs                   # StreamingTimeCorrelator (sliding window)
|   |-- streaming_tests.rs             # 10 unit tests
|   |-- quality.rs                     # TimeQuality, compute_quality
|   |-- quality_tests.rs               # 6 unit tests
|   |-- recording_event.rs             # RecordingEvent, RecordingEventType
|   |-- recording_event_tests.rs       # 9 unit tests
|   |-- chrono_interop.rs              # chrono From conversions (feature-gated, 4 tests)
|   `-- util.rs                        # MSRV helpers: is_leap_year, abs_diff_u64 (18 tests)
|
|-- tests/
|   |-- pipeline.rs                    # 68 integration tests
|   `-- properties.rs                  # 17 property-based tests (proptest)
|
|-- benches/
|   |-- time_benchmarks.rs             # 28 benchmarks (zero-dep, std::time::Instant)
|   `-- correlation_bench.rs           # Criterion benchmarks for correlation at scale
|
|-- fuzz/
|   |-- Cargo.toml                     # libfuzzer-sys harness
|   `-- fuzz_targets/
|       |-- fuzz_bcd_day.rs
|       |-- fuzz_bcd_dmy.rs
|       |-- fuzz_rtc.rs
|       |-- fuzz_secondary_header.rs
|       |-- fuzz_intra_packet.rs
|       |-- fuzz_ieee1588.rs
|       |-- fuzz_csdw.rs
|       |-- fuzz_correlation.rs
|       |-- fuzz_ntp.rs
|       `-- fuzz_ptp.rs
|
`-- docs/
    |-- L1_Requirements.md             # 53 L1 -> IRIG 106 standard (incl. Format 2)
    |-- L2_Requirements.md             # L2 -> functional (incl. Format 2)
    |-- L3_Requirements.md             # L3 -> design specs (incl. Format 2)
    |-- architecture.md                # Data flow, packet layouts, ASCII diagrams
    |-- benchmark_results.md           # Current benchmark snapshot
    |-- cli_commands.md                # CLI command quick reference
    |-- project_structure.md           # This file
    |-- ROADMAP.md                     # Phased release plan
    |-- security.md                    # Threat model, fuzzing guide
    |-- shared_types_for_irig106_types.md
    |-- test_index.md                  # All tests documented
    |-- usage.md                       # Integration examples for downstream crates
    |-- why_separate_repo.md           # Why time is its own crate
    `-- diagrams/
        |-- correlation_flow.mermaid
        |-- ecosystem.mermaid
        |-- module_deps.mermaid
        `-- traceability.mermaid
```

## Companion: irig106-time-cli

```text
irig106-time-cli/
|-- Cargo.toml                         # deps: irig106-time + memmap2
|-- README.md                          # CLI usage documentation
`-- src/
    `-- main.rs                        # ch10time: summary, channels, jumps, timeline, csv, correlate
```

### CLI Commands

| Command | Description |
|---------|-------------|
| `ch10time summary <file>` | Packet counts, time channels, RTC range, jump detection |
| `ch10time channels <file>` | Per-channel time source inventory |
| `ch10time jumps <file> [--threshold-ms N]` | Discontinuity detection |
| `ch10time timeline <file> [--limit N]` | Per-packet RTC + resolved absolute time |
| `ch10time csv <file> [--output path]` | Full timestamp export |
| `ch10time correlate <file> <rtc_hex>` | Resolve one RTC against all channels |

---

## Metrics

| Metric | Value |
|--------|-------|
| Source modules | 17 |
| Source lines (excl. tests) | ~3,900 |
| Unit tests | 184 |
| Integration tests | 68 |
| Property-based tests | 17 |
| Doc tests | 4 default / 5 with `--all-features` |
| Fuzz targets | 10 |
| Benchmarks | 28 (zero-dep) + criterion suite |
| **Total tests** | **269** |
| Required runtime deps | **0** (serde, chrono optional) |
| L1 requirements | 53 (37 base + 16 F2) |
| `#![no_std]` | Yes |
| `unsafe` blocks | **0** |

---

## Benchmark Results (release build)

```text
  Benchmark                           ns/iter      ops/sec
  ----------------------------------------------------------
  rtc_from_le_bytes                     6.7 ns     148.5M/s
  rtc_from_raw                          0.3 ns    3090.0M/s
  rtc_to_nanos                          0.3 ns    2869.9M/s
  rtc_elapsed_ticks                     0.6 ns    1625.8M/s
  bcd_day_parse                         5.9 ns     168.2M/s
  bcd_dmy_parse                         7.8 ns     128.5M/s
  bcd_day_full_pipeline                12.4 ns      80.5M/s
  csdw_from_le_bytes                    0.3 ns    3043.6M/s
  csdw_all_fields                       1.7 ns     593.7M/s
  sec_checksum_validate                 1.9 ns     516.2M/s
  ipt_parse_rtc48                       3.0 ns     331.4M/s
  corr_100refs_any                     14.3 ns      69.9M/s
  corr_3600refs_any                    22.6 ns      44.3M/s
  HOT_rtc_to_absolute                  31.0 ns      32.2M/s
```

**Hot-path: 31 ns = 32.2M ops/sec.** At 10 Gbps / 512B packets = 2.4M pkt/sec -> **13x headroom.**

---

## File Dependency Graph

```text
lib.rs -> error.rs           (no deps)
       -> rtc.rs             (no deps)
       -> absolute.rs        -> error.rs
       -> csdw.rs            (no deps)
       -> bcd.rs             -> error.rs, absolute.rs
       -> secondary.rs       -> error.rs, absolute.rs
       -> intra_packet.rs    -> error.rs, rtc.rs, absolute.rs
       -> correlation.rs     -> error.rs, rtc.rs, absolute.rs, network_time.rs  (requires alloc)
       -> network_time.rs    -> error.rs, absolute.rs  (requires alloc)
       -> version.rs         (no deps)
       -> packet_standard.rs -> version.rs
       -> quality.rs         -> correlation.rs  (requires alloc)
       -> recording_event.rs -> absolute.rs
       -> streaming.rs       -> error.rs, rtc.rs, absolute.rs, network_time.rs  (requires alloc)
       `-> chrono_interop.rs -> absolute.rs, chrono  (feature = "chrono")
```
