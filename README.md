# irig106-time

> **0.8.0 decision (owner2026-11-17):** JSON belongs to `irig106-time-cli`
> (`irigtime`). The library's optional serde feature from0.1–0.7 is removed
> in0.8.0 (breaking); use CLI JSON output. Rust data and native AnswerView/replay
> remain. irig106-types' own serde feature is unchanged. See
> [ADR-0019](docs/adr/0019-json-belongs-to-the-cli.md). Library serde/JSON code
> and feature-specific CI commands are now removed; the version bump and CLI
> report/schema completion remain release work.


> **Status: prototype — do not rely on its answers for compliance or
> analysis.** The code in this repository (published as 0.1.0 to 0.7.0) is
> the prototype, kept at the git tag `prototype-0`. A check against IRIG
> 106-24R1 found seventeen disagreements, among them ERTC times 100 times too
> large, intra-packet time formats selected from the wrong flag bit, Format 2
> data words and PTP bodies misread, time packet bodies read past their
> length, leap seconds rejected, and native GPS time reported as UTC
> (`docs/STANDARD-REVIEW.md`, T-1 to T-17). The crate is being rebuilt
> documentation first and will be released as 0.8.0 (`docs/ROADMAP.md`,
> "Where we left off"); the features below describe the prototype.

Nanosecond-precision time handling for IRIG 106 Chapter 10 telemetry data.

This crate decodes the complex time system embedded in IRIG 106 recordings — the 48-bit Relative Time Counter, BCD-encoded absolute time messages, four intra-packet timestamp formats, secondary header time, and the correlation engine that ties them all together.

## Why This Exists

IRIG 106 Chapter 10 separates *when data was recorded* (a free-running 10 MHz counter) from *what time it was* (an external clock source like GPS or IRIG-B). These two notions of time are recorded independently, may drift apart, and must be correlated after the fact. This crate is the single source of truth for that correlation across the entire TelemetryWorks ecosystem.

## Features

- **48-bit RTC** — Newtype with wrap-safe arithmetic, 100 ns resolution
- **BCD decoding** — Day-of-Year and Day-Month-Year format time messages
- **CSDW parsing** — Time Data Format 1 (0x11) and Format 2 (0x12) channel-specific data words
- **Network Time** — NTP (RFC 5905, epoch 1900 UTC) and PTP/IEEE 1588 (epoch 1970 TAI) decoding with built-in leap-second table
- **Four timestamp formats** — RTC, Chapter 4 BWT, IEEE-1588, Extended RTC
- **Secondary headers** — Checksum validation and time extraction
- **Correlation engine** — Channel-indexed O(log n) nearest-point interpolation, multi-channel support, GPS lock jump detection, F1 and F2 reference points, drift estimation, and RTC reset detection
- **Streaming correlation** — Sliding-window correlator for live UDP streams with automatic max-age eviction
- **Quality metrics** — Reference point density, RTC gaps, per-channel drift assessment
- **Version detection** — IRIG 106 standard version from TMATS CSDW (106-04 through 106-23), version-aware CSDW parsing, configurable out-of-order tolerance
- **Packet standard** — Ch10/Ch11 provenance tracking (106-17 split)
- **Event time tags** — Generic extracted-stamp adapters; event-body decoding belongs in irig106-decode (ADR-0009), replacement availability not claimed
- **Encoding** — `to_le_bytes()` on all wire-format types for packet construction
- **`impl Display`** — ISO-like formatting for `AbsoluteTime`
- **`jiff`** — Optional checked civil conversions with explicit year/scale and nanosecond precision; unknown context/leap labels refuse
- **`#![no_std]`** — Works on embedded, WASM, and standard targets
- **Shared values** — `irig106-types` is required; `jiff` is optional
- **Edition 2024, MSRV 1.85** — the same across the IRIG 106 ecosystem (docs/adr/0003). Dev tooling (benchmarks, fuzz, proptest) requires latest stable. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
- **Zero `unsafe`** — Safe Rust throughout

## Quick Start

```rust
use irig106_time::{TimeF1Layout, format1::decode_time_f1,
    scale::{ScalePolicy, TimeScale}};

// Caller explicitly declares the packet layout; FMT4 states UTC.
// Ch11 Figures11-12/13: DOY100,12:30:25.340; Data Length10.
let bytes = [0x41,0,0,0,0x34,0x25,0x30,0x12,0,1];
let decoded = decode_time_f1(&bytes, 10, TimeF1Layout::SourceWithIts17,
    ScalePolicy::default()).unwrap();
let time = decoded.time.unwrap();
assert_eq!(time.scale.effective(), TimeScale::Utc);
assert_eq!(time.year, None); // No host year guessed.
assert_eq!(time.day_of_year, 100);
```

For recording answers use `timeline` offline or `stream` live with shared
policy; retain `AnswerView`/basis/findings. `correlation` and `streaming` are
unqualified compatibility engines, not recommended paths for new answers.

## Modules

| Module | Purpose |
|--------|---------|
| `rtc` | 48-bit Relative Time Counter newtype |
| `absolute` | AbsoluteTime, CalendarTime, explicit Chapter4 context in `ch4`; re-exports Ch4BinaryTime, Ieee1588Time, Ertc from `irig106-types` |
| `csdw` | Time F1 CSDW bitfield parsing, TimeSource/TimeFormat enums |
| `bcd` | BCD Day-of-Year and Day-Month-Year decoding |
| `secondary` | Secondary header checksum validation and time extraction |
| `intra_packet` | Intra-packet timestamp format dispatch |
| `correlation` | Unqualified compatibility engine; prefer `timeline` |
| `network_time` | Format 2 (0x12): NTP, PTP, LeapSecondTable, F2 CSDW |
| `version` | IRIG 106 version detection and version-aware parsing helpers |
| `packet_standard` | Chapter 10 / Chapter 11 provenance from IRIG 106 version |
| `streaming` | Unqualified compatibility engine; prefer `stream` |
| `quality` | Correlator health and reference-density metrics |
| `jiff_interop` | Feature-gated checked Jiff civil conversions |
| `error` | TimeError enum, Result alias |

## Performance

Benchmarked on the hot path (RTC extraction followed by nearest-point correlation):

| Operation | ns/iter | ops/sec |
|-----------|---------|---------|
| `Rtc::from_le_bytes` | 6.7 | 148 M |
| `Rtc::from_raw` | 0.3 | 3,090 M |
| BCD day parse + to_absolute | 12.4 | 80 M |
| CSDW all fields | 1.7 | 594 M |
| NTP parse | 1.1 | 882 M |
| PTP parse | 1.6 | 643 M |
| Leap table lookup | 6.6 | 152 M |
| Correlation (100 refs) | 14.3 | 70 M |
| **Full hot path** | **31.0** | **32 M** |

At 10 Gbps with 512-byte average packets (2.4M pkt/sec), the hot path provides 13x headroom.

## Testing

```sh
cargo test              # 269 tests (184 unit + 68 integration + 17 property)
cargo test --all-features  # includes Jiff civil doc tests
cargo bench             # 28 zero-dep benchmarks + criterion correlation suite
cargo +nightly fuzz run fuzz_bcd_day   # 10 fuzz targets available
```

## Requirements Traceability

Every public type and function traces through three levels of requirements back to the IRIG 106-17 standard, Chapter 11, and RCC 123-20 Programmer's Handbook:

- **53 L1 requirements** — What the crate SHALL do (37 base + 16 Format 2)
- **78+ L2 requirements** — Testable functional behaviors
- **65+ L3 specifications** — Struct layouts, algorithms, constants

See `docs/L1_Requirements.md` for the full chain.

## Standard Version Support

The crate targets IRIG 106-04 through 106-23, with version-aware CSDW parsing for the 106-04/05 time source mapping delta, configurable out-of-order tolerance for pre-105 files, and Time Data Format 2 (0x12, Network Time) support introduced in 106-22.

> **Note:** Legacy 106-04/05 support is based on specification compliance and synthetic test coverage. It has not yet been validated against real Ch10 files from legacy recorders (Ampex DCRsi, L-3 MARS, Acra KAM-500). The Ch4 Binary Weighted Time bit layout is similarly unverified against multi-vendor samples. See `docs/ROADMAP.md` items P1-07, P2-05, and GAP-03.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, Rust version
requirements, and instructions for running benchmarks, fuzz tests, and the
full test suite.

## License

Apache-2.0
