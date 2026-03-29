# Contributing to irig106-time

## Rust Version Requirements

This project has two Rust version floors:

| Context | Minimum Rust Version | Reason |
|---------|---------------------|--------|
| **Library users** | **1.56** (Edition 2021 floor) | Anyone adding `irig106-time` to their `Cargo.toml` needs Rust 1.56+. All library source code compiles on 1.56. |
| **Repository developers** | **Latest stable** | Running benchmarks, fuzz tests, and property tests requires modern Rust. Dev-dependencies (criterion, proptest, libfuzzer-sys) and `std::hint::black_box` (Rust 1.66) require stable Rust newer than the library MSRV. |

The CI enforces this separation: the MSRV job runs `cargo check` (library only),
while the stable job runs `cargo test --all-features` (with dev-dependencies).

If you add new library code, verify it compiles on Rust 1.56 by avoiding APIs
stabilized after that version. See `src/util.rs` for the MSRV dependency table
and the pattern for providing backward-compatible helpers.

## Building and Testing

```sh
# Run all tests (requires stable Rust)
cargo test --all-features

# Individual feature combos
cargo test --no-default-features
cargo test --features serde
cargo test --features chrono

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt -- --check

# Build documentation
cargo doc --no-deps --all-features

# CLI
cd irig106-time-cli && cargo clippy --all-targets -- -D warnings && cd ..
```

## Running Benchmarks

Benchmarks use two harnesses: criterion (for statistical rigor) and zero-dep
`std::time::Instant` (for environments without criterion). Both are in `benches/`.

```sh
# Run all criterion benchmarks (requires stable Rust)
cargo bench

# Run a specific benchmark group
cargo bench -- rtc
cargo bench -- bcd
cargo bench -- correlation
cargo bench -- absolute_time
cargo bench -- ntp
cargo bench -- ptp

# Generate HTML reports (criterion outputs to target/criterion/)
cargo bench
# Open target/criterion/report/index.html in a browser

# Quick zero-dep benchmark (same binary, no criterion overhead)
cargo bench --bench time_benchmarks
```

**When to run benchmarks:**

- After changing hot-path code (`add_nanos`, `sub_nanos`, `correlate`, BCD decode)
- After changing internal representations (like the P4-04 `u64` restructure)
- Before and after any SIMD or lookup-table optimization
- When establishing baseline numbers for a new machine (document in `docs/benchmark_results.md`)

**Interpreting results:** Criterion reports include statistical noise analysis.
Look for regressions exceeding 5% on the `correlate` and `add_nanos` benchmarks —
those are the hottest paths in production telemetry pipelines.

## Running Fuzz Tests

Fuzz testing uses `cargo-fuzz` with `libfuzzer-sys`. There are 10 fuzz targets
covering all wire-format parsers and the correlation engine.

```sh
# Install cargo-fuzz (one-time, requires nightly)
cargo install cargo-fuzz

# List available fuzz targets
cargo fuzz list

# Run a specific target (runs until stopped with Ctrl+C)
cargo +nightly fuzz run fuzz_bcd_day
cargo +nightly fuzz run fuzz_bcd_dmy
cargo +nightly fuzz run fuzz_rtc
cargo +nightly fuzz run fuzz_csdw
cargo +nightly fuzz run fuzz_ieee1588
cargo +nightly fuzz run fuzz_ntp
cargo +nightly fuzz run fuzz_ptp
cargo +nightly fuzz run fuzz_secondary_header
cargo +nightly fuzz run fuzz_intra_packet
cargo +nightly fuzz run fuzz_correlation

# Run a target for a fixed duration (e.g., 1 hour)
cargo +nightly fuzz run fuzz_bcd_day -- -max_total_time=3600

# Run all 10 targets for 1 hour each (P1-01 validation)
for target in $(cargo fuzz list); do
    echo "=== Fuzzing $target for 1 hour ==="
    cargo +nightly fuzz run "$target" -- -max_total_time=3600
done

# View the crash corpus (if any crashes found)
ls fuzz/artifacts/fuzz_bcd_day/
```

**Important:** Fuzz testing requires **nightly Rust** (`cargo +nightly`). This is
separate from the library MSRV (1.56) and the dev tooling floor (stable).

**When to run fuzz tests:**

- After adding or modifying any `from_le_bytes` parser
- After changing validation logic (BCD range checks, checksum validation)
- After modifying the correlation engine's interpolation math
- As part of release validation (P1-01: all 10 targets × 1 hour)

## Test Organization and When to Expand

### Test Types

| Type | Location | Count | Purpose |
|------|----------|-------|---------|
| **Unit tests** | `src/*_tests.rs`, `src/util.rs` | 184 | Test individual functions and methods in isolation. Each test traces to an L3 requirement. |
| **Integration tests** | `tests/pipeline.rs` | 68 | Test cross-module workflows: BCD→correlate→resolve, NTP→PTP round-trip, MSRV helpers through the public API. |
| **Property tests** | `tests/properties.rs` | 17 | Proptest-based tests that verify invariants over random inputs: round-trip encode/decode, field ranges, arithmetic commutativity. |
| **Fuzz targets** | `fuzz/fuzz_targets/` | 10 | Long-running random-input testing for panic-freedom and memory safety on all parsers. |
| **Benchmarks** | `benches/` | 2 files | Performance regression detection on hot paths. |

### When to Add Each Type

**Add a unit test when:**

- You add or modify a public method
- You fix a bug (write the test that would have caught it first)
- You add a new error variant or validation rule
- The test traces to a specific L3 requirement

**Add an integration test when:**

- You add a new cross-module workflow (e.g., "parse F2 CSDW → decode NTP → correlate → resolve")
- You add functionality that exercises multiple modules together
- You need to verify behavior through the public API only (no `pub(crate)` access)

**Add a property test when:**

- You implement encode + decode (round-trip property: `decode(encode(x)) == x`)
- You implement arithmetic that should be commutative or associative
- You want to verify field range invariants over random inputs
- The function accepts arbitrary byte slices (proptest finds edge cases faster than hand-written tests)

**Add a fuzz target when:**

- You add a new `from_le_bytes` or `from_raw` parser
- You add a new wire format
- You want to prove panic-freedom for untrusted input

### Requirement Traceability

Every test should trace to a requirement. Use the doc-comment table format at the
top of each test module:

```rust
//! | Test | Validates | Traces |
//! |------|-----------|--------|
//! | `my_new_test` | Description of what it validates | L3-XXX-NNN |
```

See `docs/L1_Requirements.md` → `L2_Requirements.md` → `L3_Requirements.md` for
the full requirements hierarchy.

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --all-targets --all-features -- -D warnings` before pushing
- Use `#![deny(missing_docs)]` — every public item needs a doc comment
- Trace public items to requirements with `/// **Traces:** L3-XXX-NNN`
- Keep the MSRV table in `src/util.rs` updated if you introduce new API dependencies
