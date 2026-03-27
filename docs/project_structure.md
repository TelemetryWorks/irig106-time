# Project Structure

---

## Repository Layout

```text
irig106-time/
|-- .gitignore                          # Rust, fuzz, coverage, and local artifact ignores
|-- Cargo.toml                          # Zero required deps, std feature gate
|-- Cargo.lock                          # Root lockfile for local development
|-- CHANGELOG.md                        # Release notes (Keep a Changelog)
|-- LICENSE                             # Apache-2.0
|-- README.md                           # Crate overview, quick start, performance summary
|
|-- src/
|   |-- lib.rs                          # #![no_std], module wiring, re-exports
|   |-- error.rs                        # TimeError enum, Result<T>, Display
|   |-- error_tests.rs                  # unit tests
|   |-- rtc.rs                          # Rtc newtype (48-bit, 10 MHz)
|   |-- rtc_tests.rs                    # unit tests
|   |-- absolute.rs                     # AbsoluteTime, Ch4BinaryTime, Ieee1588Time, Ertc
|   |-- absolute_tests.rs               # unit tests
|   |-- csdw.rs                         # TimeF1Csdw, TimeSource, TimeFormat, DateFormat
|   |-- csdw_tests.rs                   # unit tests
|   |-- bcd.rs                          # DayFormatTime, DmyFormatTime (BCD wire format)
|   |-- bcd_tests.rs                    # unit tests
|   |-- secondary.rs                    # Secondary header time, checksum validation
|   |-- secondary_tests.rs              # unit tests
|   |-- intra_packet.rs                 # IntraPacketTime, 4-format dispatch
|   |-- intra_packet_tests.rs           # unit tests
|   |-- correlation.rs                  # TimeCorrelator, ReferencePoint, TimeJump
|   `-- correlation_tests.rs            # unit tests
|
|-- tests/
|   |-- pipeline.rs                     # integration tests
|   `-- properties.rs                   # property-based tests (10K iters, zero deps)
|
|-- benches/
|   `-- time_benchmarks.rs              # benchmarks (zero-dep, std::time::Instant)
|
|-- fuzz/
|   |-- Cargo.toml                      # libfuzzer-sys harness
|   |-- Cargo.lock                      # fuzz harness lockfile
|   `-- fuzz_targets/
|       |-- fuzz_bcd_day.rs
|       |-- fuzz_bcd_dmy.rs
|       |-- fuzz_rtc.rs
|       |-- fuzz_secondary_header.rs
|       |-- fuzz_intra_packet.rs
|       |-- fuzz_ieee1588.rs
|       |-- fuzz_csdw.rs
|       `-- fuzz_correlation.rs
|
|-- docs/
|   |-- architecture.md                    # Data flow, packet layouts, ASCII diagrams
|   |-- benchmark_results.md               # Captured benchmark output snapshot
|   |-- cli_commands.md                    # `ch10time` command reference
|   |-- L1_Requirements.md                 # L1 -> IRIG 106 standard
|   |-- L2_Requirements.md                 # L2 -> functional
|   |-- L3_Requirements.md                 # L3 -> design specs
|   |-- project_structure.md               # This file
|   |-- ROADMAP.md                         # Version support matrix and phased roadmap
|   |-- security.md                        # Threat model, fuzzing guide
|   |-- shared_types_for_irig106_types.md # Shared type migration plan
|   |-- test_index.md                      # All 124 tests documented
|   |-- usage.md                           # End-to-end library usage guide
|   |-- why_separate_repo.md               # Why time is its own crate
|   `-- diagrams/
|       |-- ecosystem.mermaid
|       |-- correlation_flow.mermaid
|       |-- module_deps.mermaid
|       `-- traceability.mermaid
|
`-- irig106-time-cli/                      # Companion crate with `ch10time`
    |-- Cargo.toml                         # deps: irig106-time + memmap2
    |-- Cargo.lock                         # CLI lockfile
    |-- README.md                          # CLI install/build/run guide
    `-- src/
        `-- main.rs                        # ch10time: summary, channels, jumps, timeline, csv, correlate
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
| detect_time_jump | O(n) per call, 303 us | Cache per-channel sorted slices | 100x for repeated calls |
| BCD decode | Branch per nibble | 256-byte lookup table | ~1-2 ns |

---

## File Dependency Graph

```text
lib.rs --> error.rs           (no deps)
       --> rtc.rs             (no deps)
       --> absolute.rs        -> error.rs
       --> csdw.rs            (no deps)
       --> bcd.rs             -> error.rs, absolute.rs
       --> secondary.rs       -> error.rs, absolute.rs
       --> intra_packet.rs    -> error.rs, rtc.rs, absolute.rs
       --> correlation.rs     -> error.rs, rtc.rs, absolute.rs  (requires alloc)
```
