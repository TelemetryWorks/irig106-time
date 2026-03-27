# irig106-time

Nanosecond-precision time handling for IRIG 106 Chapter 10 telemetry data.

This crate decodes the complex time system embedded in IRIG 106 recordings — the 48-bit Relative Time Counter, BCD-encoded absolute time messages, four intra-packet timestamp formats, secondary header time, and the correlation engine that ties them all together.

## Why This Exists

IRIG 106 Chapter 10 separates *when data was recorded* (a free-running 10 MHz counter) from *what time it was* (an external clock source like GPS or IRIG-B). These two notions of time are recorded independently, may drift apart, and must be correlated after the fact. This crate is the single source of truth for that correlation across the entire TelemetryWorks ecosystem.

## Features

- **48-bit RTC** — Newtype with wrap-safe arithmetic, 100 ns resolution
- **BCD decoding** — Day-of-Year and Day-Month-Year format time messages
- **CSDW parsing** — Time Data Format 1 (0x11) channel-specific data word
- **Four timestamp formats** — RTC, Chapter 4 BWT, IEEE-1588, Extended RTC
- **Secondary headers** — Checksum validation and time extraction
- **Correlation engine** — Nearest-point interpolation, multi-channel support, GPS lock jump detection
- **`#![no_std]`** — Works on embedded, WASM, and standard targets
- **Zero dependencies** — Only `core` and `alloc`
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
| `error` | TimeError enum, Result alias |

## Performance

Benchmarked on the hot path (RTC extraction followed by nearest-point correlation):

| Operation | ns/iter | ops/sec |
|-----------|---------|---------|
| `Rtc::from_le_bytes` | 6.7 | 148 M |
| `Rtc::from_raw` | 0.3 | 3,090 M |
| BCD day parse + to_absolute | 12.4 | 80 M |
| CSDW all fields | 1.7 | 594 M |
| Correlation (100 refs) | 14.3 | 70 M |
| **Full hot path** | **31.0** | **32 M** |

At 10 Gbps with 512-byte average packets (2.4M pkt/sec), the hot path provides 13x headroom.

## Testing

```sh
cargo test              # 124 tests (104 unit + 10 integration + 10 property)
cargo bench             # 23 zero-dependency benchmarks
cargo +nightly fuzz run fuzz_bcd_day   # 8 fuzz targets available
```

## Requirements Traceability

Every public type and function traces through three levels of requirements back to the IRIG 106-17 standard and RCC 123-20 Programmer's Handbook:

- **37 L1 requirements** — What the crate SHALL do
- **78 L2 requirements** — Testable functional behaviors
- **65 L3 specifications** — Struct layouts, algorithms, constants

See `docs/L1_REQUIREMENTS.md` for the full chain.

## Standard Version Support

The crate currently targets IRIG 106-07 through 106-23. Support for legacy 106-04/05 files and the new Time Data Format 2 (0x12, network time) from 106-22 are on the roadmap. See `docs/ROADMAP.md`.

## License

Apache-2.0
