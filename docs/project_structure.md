# Project Structure — irig106-time

**Date:** 2026-03-27

---

## Repository Layout

```
irig106-time/
├── Cargo.toml                          # Zero required deps, std feature gate
├── CHANGELOG.md                        # Keep-a-changelog format
├── LICENSE                             # Apache-2.0
├── README.md
│
├── src/
│   ├── lib.rs                          # #![no_std], module wiring, re-exports
│   ├── error.rs                        # TimeError enum, Result<T>, Display
│   ├── error_tests.rs                  # 7 unit tests
│   ├── rtc.rs                          # Rtc newtype (48-bit, 10 MHz)
│   ├── rtc_tests.rs                    # 18 unit tests
│   ├── absolute.rs                     # AbsoluteTime, Ch4BinaryTime, Ieee1588Time, Ertc
│   ├── absolute_tests.rs              # 23 unit tests
│   ├── csdw.rs                         # TimeF1Csdw, TimeSource, TimeFormat, DateFormat
│   ├── csdw_tests.rs                   # 14 unit tests
│   ├── bcd.rs                          # DayFormatTime, DmyFormatTime (BCD wire format)
│   ├── bcd_tests.rs                    # 13 unit tests
│   ├── secondary.rs                    # Secondary header time, checksum validation
│   ├── secondary_tests.rs             # 10 unit tests
│   ├── intra_packet.rs                # IntraPacketTime, 4-format dispatch
│   ├── intra_packet_tests.rs          # 8 unit tests
│   ├── correlation.rs                  # TimeCorrelator, ReferencePoint, TimeJump
│   ├── correlation_tests.rs           # 11 unit tests
│   ├── network_time.rs                # Format 2 (0x12): NTP, PTP, LeapSecondTable
│   └── network_time_tests.rs          # 22 unit tests
│
├── tests/
│   ├── pipeline.rs                     # 15 integration tests (F1 + F2 pipelines)
│   └── properties.rs                  # 10 property-based tests (10K iters, zero deps)
│
├── benches/
│   └── time_benchmarks.rs             # 30 benchmarks (zero-dep, std::time::Instant)
│
├── fuzz/
│   ├── Cargo.toml                      # libfuzzer-sys harness
│   └── fuzz_targets/
│       ├── fuzz_bcd_day.rs
│       ├── fuzz_bcd_dmy.rs
│       ├── fuzz_rtc.rs
│       ├── fuzz_secondary_header.rs
│       ├── fuzz_intra_packet.rs
│       ├── fuzz_ieee1588.rs
│       ├── fuzz_csdw.rs
│       ├── fuzz_correlation.rs
│       ├── fuzz_ntp.rs                 # NEW in v0.2.0
│       └── fuzz_ptp.rs                 # NEW in v0.2.0
│
└── docs/
    ├── L1_Requirements.md              # 53 L1 → IRIG 106 standard (incl. Format 2)
    ├── L2_Requirements.md              # 78 L2 → functional
    ├── L2L3_REQUIREMENTS_F2_ADDENDUM.md # L2/L3 → Format 2
    ├── L3_Requirements.md              # 65 L3 → design specs
    ├── test_index.md                   # All tests documented
    ├── architecture.md                 # Data flow, packet layouts, ASCII diagrams
    ├── why_separate_repo.md           # Why time is its own crate
    ├── security.md                     # Threat model, fuzzing guide
    ├── usage.md                        # Integration examples for downstream crates
    ├── shared_types_for_irig106_types.md
    ├── project_structure.md           # This file
    ├── ROADMAP.md                      # Phased release plan
    └── diagrams/
        ├── ecosystem.mermaid
        ├── correlation_flow.mermaid
        ├── module_deps.mermaid
        └── traceability.mermaid
```

## Companion: irig106-time-cli

```
irig106-time-cli/
├── Cargo.toml                          # deps: irig106-time + memmap2
├── README.md                           # CLI usage documentation
└── src/
    └── main.rs                         # ch10time: summary, channels, jumps, timeline, csv, correlate
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
| Source modules | 9 |
| Source lines (excl. tests) | ~2,000 |
| Unit tests | 126 |
| Integration tests | 15 |
| Property-based tests | 10 |
| Fuzz targets | 10 |
| Benchmarks | 30 |
| **Total tests** | **151** |
| Required runtime deps | **0** |
| L1 requirements | 53 (37 base + 16 F2) |
| `#![no_std]` | Yes |
| `unsafe` blocks | **0** |

---

## Benchmark Results (release build)

```
  Benchmark                           ns/iter      ops/sec
  ──────────────────────────────────────────────────────────
  rtc_from_le_bytes                     7.1 ns     141.1M/s
  rtc_from_raw                          0.7 ns    1448.1M/s
  bcd_day_parse                         6.1 ns     164.9M/s
  csdw_from_le_bytes                    0.3 ns    2989.4M/s
  ntp_from_le_bytes                     1.1 ns     881.8M/s
  ptp_from_le_bytes                     1.6 ns     643.1M/s
  leap_table_lookup                     6.6 ns     152.2M/s
  f2_ntp_payload_parse                  2.7 ns     370.0M/s
  corr_100refs_any                     16.6 ns      60.2M/s
  corr_3600refs_any                    24.1 ns      41.5M/s
  HOT_rtc_to_absolute                  34.5 ns      29.0M/s
```

**Hot-path: 34.5 ns = 29M ops/sec.** At 10 Gbps / 512B packets = 2.4M pkt/sec → **12x headroom.**

---

## File Dependency Graph

```
lib.rs ──→ error.rs           (no deps)
       ──→ rtc.rs             (no deps)
       ──→ absolute.rs        → error.rs
       ──→ csdw.rs            (no deps)
       ──→ bcd.rs             → error.rs, absolute.rs
       ──→ secondary.rs       → error.rs, absolute.rs
       ──→ intra_packet.rs    → error.rs, rtc.rs, absolute.rs
       ──→ correlation.rs     → error.rs, rtc.rs, absolute.rs, network_time.rs  (requires alloc)
       ──→ network_time.rs    → error.rs, absolute.rs  (requires alloc)
```
