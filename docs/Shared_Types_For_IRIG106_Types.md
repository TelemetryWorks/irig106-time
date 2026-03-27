# Shared Types for `irig106-types`

**Document:** SHARED_TYPES_FOR_IRIG106_TYPES.md
**Date:** 2026-03-25

---

## 1. Purpose

This document identifies types defined in `irig106-time` that are shared across multiple crates
in the TelemetryWorks IRIG 106 ecosystem. These types are candidates for migration into the
`irig106-types` foundational crate to eliminate duplication and ensure consistent definitions.

---

## 2. Migration Candidates

### 2.1 `Rtc` — 48-bit Relative Time Counter

**Current location:** `irig106-time::rtc::Rtc`
**Used by:** `irig106-core` (packet header RTC field), `irig106-time` (correlation),
`irig106-decode` (intra-packet timestamps), `irig106-write` (packet construction),
`irig106-ch10-reader` (currently uses raw `u64`)

```rust
/// 48-bit Relative Time Counter (10 MHz, 100 ns/tick).
///
/// Invariant: inner value <= 0x0000_FFFF_FFFF_FFFF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rtc(u64);
```

**Key API surface to preserve:**
- `Rtc::from_le_bytes([u8; 6]) -> Rtc`
- `Rtc::from_raw(u64) -> Rtc` (masks to 48 bits)
- `Rtc::as_raw(&self) -> u64`
- `Rtc::elapsed_ticks(&self, later: &Rtc) -> u64`
- `Rtc::elapsed_nanos(&self, later: &Rtc) -> u64`
- `Rtc::to_nanos(&self) -> u64`
- `Rtc::ZERO`, `Rtc::MAX`

### 2.2 `Ch4BinaryTime` — IRIG 106 Chapter 4 Binary Weighted Time

**Current location:** `irig106-time::absolute::Ch4BinaryTime`
**Used by:** `irig106-time` (secondary header, intra-packet), `irig106-decode` (message timestamps)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ch4BinaryTime {
    pub high_order: u16,
    pub low_order: u16,
    pub microseconds: u16,
}
```

### 2.3 `Ieee1588Time` — IEEE-1588 Precision Time

**Current location:** `irig106-time::absolute::Ieee1588Time`
**Used by:** `irig106-time`, `irig106-decode`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ieee1588Time {
    pub nanoseconds: u32,
    pub seconds: u32,
}
```

### 2.4 `Ertc` — 64-bit Extended Relative Time Counter

**Current location:** `irig106-time::absolute::Ertc`
**Used by:** `irig106-time`, `irig106-decode`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ertc(u64);
```

### 2.5 Time-Related Enums

**Current location:** `irig106-time::csdw`

```rust
/// Source of time applied to the recorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSource {
    Internal,
    External,
    InternalRtc,
    Gps,
    None,
    Reserved(u8),
}

/// Format of the external time source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    IrigB,
    IrigA,
    IrigG,
    Rtc,
    Utc,
    Gps,
    Reserved(u8),
}

/// Date representation format in time packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormat {
    DayOfYear,
    DayMonthYear,
}

/// Time format discriminant for secondary headers and intra-packet timestamps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecHdrTimeFormat {
    Ch4,
    Ieee1588,
    Ertc,
    Reserved(u8),
}
```

---

## 3. Migration Strategy

1. Define all types above in `irig106-types` with the exact API surface documented here.
2. Add `irig106-types` as a dependency in `irig106-time`.
3. Re-export from `irig106-time` for backward compatibility:
   `pub use irig106_types::{Rtc, Ch4BinaryTime, Ieee1588Time, Ertc, TimeSource, TimeFormat, DateFormat};`
4. Update `irig106-core`, `irig106-ch10-reader`, `irig106-decode`, and `irig106-write` to use
   `irig106-types` directly.
5. Remove duplicated definitions from each crate.

---

## 4. `#![no_std]` Considerations

All types listed above are `Copy` and require only `core`. The `irig106-types` crate should be
`#![no_std]` with an optional `std` feature that adds `std::error::Error` impls.
