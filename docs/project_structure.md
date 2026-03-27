# Project Structure

---

## Repository Layout

```
irig106-time/
├── Cargo.toml                          # Zero required deps, std feature gate
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
│   └── correlation_tests.rs           # 11 unit tests
│
├── tests/
│   ├── pipeline.rs                     # 10 integration tests
│   └── properties.rs                  # 10 property-based tests (10K iters, zero deps)
│
├── benches/
│   └── time_benchmarks.rs             # 23 benchmarks (zero-dep, std::time::Instant)
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
│       └── fuzz_correlation.rs
│
├── docs/
│   ├── L1_REQUIREMENTS.md                 # L1 → IRIG 106 standard
│   ├── L2_REQUIREMENTS.md                 # L2 → functional
│   ├── L3_REQUIREMENTS.md                 # L3 → design specs
│   ├── TEST_INDEX.md                      # All 124 tests documented
│   ├── ARCHITECTURE.md                    # Data flow, packet layouts, ASCII diagrams
│   ├── WHY_SEPARATE_REPO.md               # Why time is its own crate
│   ├── SECURITY.md                        # Threat model, fuzzing guide
│   ├── SHARED_TYPES_FOR_IRIG106_TYPES.md
│   ├── PROJECT_STRUCTURE.md               # This file
│   └── diagrams/
│       ├── ecosystem.mermaid
│       ├── correlation_flow.mermaid
│       ├── module_deps.mermaid
│       └── traceability.mermaid
└── irig106-time-cli/                      # Companion Crate: Example binary demonstrating some usage.
    ├── Cargo.toml                         # deps: irig106-time + memmap2
    └── src
        └── main.rs                        # ch10time: summary, channels, jumps, timeline, csv, correlate
```



## Performance Annotations

All hot-path functions carry `#[inline]`:

| Module | Inlined Functions |
|--------|-------------------|
| rtc | from_le_bytes, from_raw, as_raw, elapsed_ticks, elapsed_nanos, to_nanos |
| absolute | total_nanos_of_day, add_nanos, sub_nanos, from_le_bytes, to_nanos_since_epoch |
| csdw | from_raw, from_le_bytes, as_raw, time_source, time_format, is_leap_year, date_format |
| bcd | extract_bcd_digit, check_reserved, month_day_to_doy |

---

## Future Optimization Targets

| Item | Current | Target | Impact |
|------|---------|--------|--------|
| Channel-filtered correlation | O(n) scan, 304 ns | O(log n) with channel index, ~15 ns | 20x faster per-channel lookup |
| detect_time_jump | O(n) per call, 303 µs | Cache per-channel sorted slices | 100x for repeated calls |
| BCD decode | Branch per nibble | 256-byte lookup table | ~1-2 ns |

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
       ──→ correlation.rs     → error.rs, rtc.rs, absolute.rs  (requires alloc)
```
