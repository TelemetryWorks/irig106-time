# Contributing to irig106-time

## Rust Version Requirements

This project has two Rust version floors:

| Context | Minimum Rust Version | Reason |
|---------|---------------------|--------|
| **Library users** | **1.60** (`dep:` feature syntax floor) | Anyone adding `irig106-time` to their `Cargo.toml` needs Rust 1.60+. All library source code compiles on 1.60. |
| **Repository developers** | **Latest stable** | Running benchmarks, fuzz tests, and property tests requires modern Rust. Dev-dependencies (criterion, proptest, libfuzzer-sys) and `std::hint::black_box` (Rust 1.66) require stable Rust newer than the library MSRV. |

The CI enforces this separation: the MSRV job runs `cargo check` (library only),
while the stable job runs `cargo test --all-features` (with dev-dependencies).

If you add new library code, verify it compiles on Rust 1.60 by avoiding APIs
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

### Interpreting Criterion Output

Criterion produces output like this for each benchmark:

```
correlate/100refs_any   time:   [14.1 ns 14.3 ns 14.5 ns]
                        change: [-2.1% -0.3% +1.6%] (p = 0.78 > 0.05)
                        No change in performance detected.
```

Here's how to read it:

- **`[14.1 ns 14.3 ns 14.5 ns]`** — The 95% confidence interval: lower bound,
  point estimate, upper bound. The true mean is likely in this range.
- **`change: [-2.1% -0.3% +1.6%]`** — Change compared to the last saved baseline.
  If no baseline exists, this line says "No change data available."
- **`p = 0.78 > 0.05`** — The p-value from a statistical test. If `p < 0.05`,
  the change is statistically significant. If `p > 0.05`, it's likely noise.
- **Summary line** — Criterion tells you if it detected a regression, improvement,
  or no change.

**What to watch for:**

| Signal | Meaning | Action |
|--------|---------|--------|
| `change: [+3% +5% +7%]` with `p < 0.05` | Statistically significant regression | Investigate. If > 5% on a hot path, fix before merging. |
| `change: [-8% -5% -2%]` with `p < 0.05` | Statistically significant improvement | Document in the PR and update the baseline. |
| `p > 0.05` | Change is within noise | No action. Do not chase noise. |
| Very wide confidence interval (e.g., `[10 ns 14 ns 20 ns]`) | Measurement instability | Close background applications, pin CPU frequency if possible, re-run. |

**Hot-path benchmarks** (regressions here matter most):
- `correlate` — Used on every data packet
- `add_nanos` / `sub_nanos` — Used in every correlation
- `bcd_day_parse` — Used on every Format 1 time packet
- `rtc_from_le_bytes` — Used on every packet header

### Creating and Managing Baselines

A **baseline** is a saved criterion measurement that future runs compare against.
You create one when you have a known-good state on a specific machine.

```sh
# Save a baseline (names it so you can compare later)
cargo bench -- --save-baseline v0.7.0

# Compare current performance against a saved baseline
cargo bench -- --baseline v0.7.0

# Compare two saved baselines against each other
cargo bench -- --load-baseline pr-123 --baseline v0.7.0
```

**When to create a baseline:**

- After tagging a release (e.g., `cargo bench -- --save-baseline v0.7.0`)
- Before starting a performance-sensitive change (save as `pre-change`)
- When setting up a new benchmark machine
- After hardware or OS changes on the benchmark machine

**Where baselines live:**

- Criterion stores baselines in `target/criterion/`. This directory is
  git-ignored (it's machine-specific and large).
- **Do not commit baselines to git.** They are machine-specific — numbers from
  your NVMe workstation are meaningless on a CI runner.
- Instead, document the **summary numbers** in `docs/benchmark_results.md` with
  the machine spec. The format is:

```markdown
## Baseline: v0.7.0 — [Machine Name]

**Date:** 2026-03-29
**Machine:** [CPU model], [RAM], [OS], [Rust version]
**Command:** `cargo bench -- --save-baseline v0.7.0`

| Benchmark | Mean | 95% CI | ops/sec |
|-----------|------|--------|---------|
| rtc_from_le_bytes | 6.7 ns | [6.5, 6.9] | 149M |
| correlate/100refs_any | 14.3 ns | [14.1, 14.5] | 70M |
| ... | ... | ... | ... |

**Hot-path headroom:** At 10 Gbps / 512B packets = 2.4M pkt/sec,
the correlation path at 14.3 ns provides [X]x headroom.
```

**For the cyber security report:** Include the baseline table, the machine spec,
and a statement that no regressions exceeded 5% between the baseline and the
release candidate. The criterion HTML report (`target/criterion/report/index.html`)
can be exported as supporting evidence.

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
separate from the library MSRV (1.60) and the dev tooling floor (stable).

**When to run fuzz tests:**

- After adding or modifying any `from_le_bytes` parser
- After changing validation logic (BCD range checks, checksum validation)
- After modifying the correlation engine's interpolation math
- As part of release validation (P1-01: all 10 targets × 1 hour)

### Interpreting Fuzz Output

A **clean run** looks like this:

```
#12345  REDUCE cov: 342 ft: 1205 corp: 89/4567b lim: 1024 exec/s: 6172
#50000  pulse  cov: 342 ft: 1205 corp: 89/4567b lim: 4096 exec/s: 5890
...
Done 180000 runs in 3600 second(s)
```

Here's what each field means:

| Field | Meaning |
|-------|---------|
| `#12345` | Number of test cases executed so far |
| `REDUCE` / `pulse` / `NEW` | `NEW` = found new code coverage. `REDUCE` = found a smaller input covering the same code. `pulse` = periodic heartbeat (no new coverage). |
| `cov: 342` | Number of unique code coverage edges reached. Should increase initially then plateau. |
| `ft: 1205` | Number of unique feature tuples (a finer-grained coverage metric). |
| `corp: 89/4567b` | Corpus size: 89 inputs totaling 4567 bytes. The fuzzer builds up interesting inputs over time. |
| `exec/s: 6172` | Throughput. Higher is better. Typically 1K–50K for Rust fuzz targets. |

**What to watch for:**

| Signal | Meaning | Action |
|--------|---------|--------|
| Run completes with no crashes | Parser handles all random inputs without panics | Document as "PASS" in fuzz report. |
| `cov` stops increasing early | Fuzzer has explored all reachable code paths | Good — means the parser is simple and well-covered. |
| `==ERROR: ...` or `SUMMARY: ...` | A crash was found (panic, OOB, overflow) | This is a **finding**. See "Handling Crashes" below. |
| `exec/s` is very low (< 100) | Target is too slow for effective fuzzing | The fuzz target may need simplification. |

### Handling Crashes

If the fuzzer finds a crash:

1. The crashing input is saved to `fuzz/artifacts/<target>/`
2. Reproduce it: `cargo +nightly fuzz run <target> fuzz/artifacts/<target>/<crash-file>`
3. File a bug, write a regression test, fix the code
4. Re-run the fuzz target to verify the fix
5. The crashing input stays in the corpus — future runs will re-test it

### Fuzz Coverage Report

To measure what percentage of the code the fuzzer actually reached:

```sh
# Generate coverage data (requires nightly + llvm-tools)
rustup component add llvm-tools-preview --toolchain nightly
cargo +nightly fuzz coverage fuzz_bcd_day

# Convert to human-readable report using grcov
cargo install grcov
grcov . --binary-path ./fuzz/target -s . -t html --branch --ignore-not-existing -o fuzz_coverage/
# Open fuzz_coverage/index.html in a browser
```

### Documenting Fuzz Results for the Security Report

For each release validation (P1-01), document results in `docs/fuzz_report.md`:

```markdown
## Fuzz Validation Report: v0.X.0

**Date:** YYYY-MM-DD
**Rust toolchain:** nightly-YYYY-MM-DD
**Duration per target:** 1 hour
**Machine:** [CPU model], [RAM], [OS]

| Target | Total Runs | Coverage Edges | Corpus Size | Crashes | Result |
|--------|-----------|----------------|-------------|---------|--------|
| fuzz_bcd_day | 180,000 | 342 | 89 inputs | 0 | PASS |
| fuzz_bcd_dmy | 165,000 | 298 | 76 inputs | 0 | PASS |
| ... | ... | ... | ... | ... | ... |

### Findings

[None / List any crashes with root cause, fix commit, and regression test]

### Coverage Summary

[Link to HTML coverage report or summary percentage]
```

This report becomes an appendix in the cyber security report, demonstrating
that all parsers handling untrusted input have been tested for panic-freedom
and memory safety under sustained random input.

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

## Security Analysis and Reporting

This section covers the tools and processes used to produce the cyber security
report required before 1.0.0. These tools are run **locally on demand**, not in
CI (to manage cost and complexity). Results are committed as report artifacts.

### Static Analysis with Clippy

Clippy is the first line of defense. It's already enforced in CI with `-D warnings`
(all warnings are errors). No additional setup needed — this is built into every
build.

### SAST with `cargo-audit` and `cargo-deny`

```sh
# Install (one-time)
cargo install cargo-audit
cargo install cargo-deny

# Check for known vulnerabilities in dependencies
cargo audit

# Comprehensive dependency policy check (licenses, bans, advisories)
cargo deny check
```

`cargo-audit` queries the RustSec Advisory Database. `cargo-deny` provides broader
policy enforcement including license compatibility and banned crate detection.

**For the security report:** Run `cargo audit` at release time and include the
output (either clean or with remediation notes) as an appendix.

### SAST with SonarQube (or SonarCloud)

SonarQube provides deeper static analysis: code smells, complexity metrics,
duplication detection, and security hotspot identification.

```sh
# Option 1: SonarCloud (free for open-source)
# Set up at https://sonarcloud.io, connect the GitHub repo, and configure
# a sonar-project.properties file:
#
#   sonar.projectKey=TelemetryWorks_irig106-time
#   sonar.organization=telemetryworks
#   sonar.sources=src
#   sonar.tests=tests,benches
#   sonar.rust.clippy.reportPaths=clippy-report.json
#   sonar.rust.coverage.reportPaths=lcov.info

# Option 2: Local SonarQube (Docker)
docker run -d --name sonarqube -p 9000:9000 sonarqube:community
# Then run the scanner with sonar-scanner CLI
```

**For the security report:** Export the SonarQube dashboard as PDF or screenshot.
Key metrics to capture: code coverage %, duplication %, complexity, security
hotspots (should be zero for a library with no I/O).

### Code Coverage with `cargo-tarpaulin` or `llvm-cov`

```sh
# Option 1: tarpaulin (simpler, Linux only)
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --out Html --output-dir coverage/
# Open coverage/tarpaulin-report.html

# Option 2: llvm-cov (cross-platform, requires nightly)
cargo install cargo-llvm-cov
cargo +nightly llvm-cov --all-features --html --output-dir coverage/
# Open coverage/html/index.html

# Generate LCOV format (for SonarQube integration)
cargo +nightly llvm-cov --all-features --lcov --output-path lcov.info
```

**For the security report:** Include the overall coverage percentage and a
statement about which modules have < 80% coverage (with justification if any).

### `cargo-geiger` — Unsafe Code Audit

```sh
cargo install cargo-geiger
cargo geiger
```

This crate uses **zero `unsafe` blocks**. `cargo-geiger` confirms this.
Include the output in the security report as evidence of memory safety.

### Generating the Security Report

The security report should be a single document (`docs/security_report.md` or
a formal PDF) with these sections:

1. **Executive Summary** — Crate purpose, IRIG 106 context, zero-unsafe claim
2. **Static Analysis** — Clippy (zero warnings), SonarQube dashboard summary
3. **Dependency Audit** — `cargo audit` output (zero advisories)
4. **Fuzz Testing** — Summary table from `docs/fuzz_report.md` (all targets PASS)
5. **Code Coverage** — Overall percentage, per-module breakdown
6. **Unsafe Code Audit** — `cargo-geiger` output (zero unsafe)
7. **VCRM Summary** — Reference to `docs/VCRM.md` (zero gaps)
8. **Benchmark Baseline** — Performance characteristics under load

This report is produced manually before the 1.0.0 release. The individual tools
can be scripted for convenience but are not in CI due to cost and nightly
requirements for some tools.
