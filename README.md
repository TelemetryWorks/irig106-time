# irig106-time

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
- **Recording events** — Data Type 0x02 event parsing with time context
- **Encoding** — `to_le_bytes()` on all wire-format types for packet construction
- **`impl Display`** — ISO-like formatting for `AbsoluteTime`
- **`serde`** — Optional `Serialize`/`Deserialize` on all public data types (except `TimeError`) via the `serde` feature gate
- **`chrono`** — Optional `From` conversions between `AbsoluteTime` and `chrono::NaiveDateTime`
- **`#![no_std]`** — Works on embedded, WASM, and standard targets
- **Zero required dependencies** — Only `core` and `alloc` (serde, chrono are optional)
- **MSRV 1.56** — Edition 2021 floor. No nightly features required
- **Zero `unsafe`** — Safe Rust throughout

## Quick Start

```rust
use irig106_time::*;
use irig106_time::bcd::DayFormatTime;
use irig106_time::csdw::TimeF1Csdw;

// Parse a Time Data Format 1 packet
let csdw = TimeF1Csdw::from_le_bytes([0x03, 0x00, 0x00, 0x00]);
assert_eq!(csdw.time_source(), TimeSource::Gps);

// Decode BCD time message (Day 100, 12:30:25.340)
let bcd_bytes: [u8; 8] = [0x34, 0x25, 0x30, 0x12, 0x00, 0x01, 0x00, 0x00];
let day_time = DayFormatTime::from_le_bytes(&bcd_bytes).unwrap();
let abs_time = day_time.to_absolute();

// Build a correlation table
let mut correlator = TimeCorrelator::new();
let ref_rtc = Rtc::from_raw(10_000_000); // 1 second into recording
correlator.add_reference(1, ref_rtc, abs_time);

// Resolve a data packet's RTC to absolute time
let data_rtc = Rtc::from_raw(10_150_000); // 15 ms later
let resolved = correlator.correlate(data_rtc, None).unwrap();
// → Day 100, 12:30:25.355
```

## Modules

| Module | Purpose |
|--------|---------|
| `rtc` | 48-bit Relative Time Counter newtype |
| `absolute` | AbsoluteTime, Ch4BinaryTime, Ieee1588Time, Ertc |
| `csdw` | Time F1 CSDW bitfield parsing, TimeSource/TimeFormat enums |
| `bcd` | BCD Day-of-Year and Day-Month-Year decoding |
| `secondary` | Secondary header checksum validation and time extraction |
| `intra_packet` | Intra-packet timestamp format dispatch |
| `correlation` | RTC-to-absolute time interpolation engine |
| `network_time` | Format 2 (0x12): NTP, PTP, LeapSecondTable, F2 CSDW |
| `version` | IRIG 106 version detection and version-aware parsing helpers |
| `packet_standard` | Chapter 10 / Chapter 11 provenance from IRIG 106 version |
| `streaming` | Sliding-window correlator for live telemetry streams |
| `quality` | Correlator health and reference-density metrics |
| `recording_event` | Data Type 0x02 recording event parsing |
| `chrono_interop` | Feature-gated `chrono` conversions |
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
cargo test --all-features  # includes chrono doc tests
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

## License

Apache-2.0
