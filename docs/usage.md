# Usage Guide — irig106-time

**Document:** usage.md
**Crate:** irig106-time v0.2.0
**Date:** 2026-03-27

This guide shows how to use `irig106-time` in the context of a larger IRIG 106
application. Every example assumes you are reading or processing Chapter 10
data — either from files or live UDP streams.

---

## Table of Contents

1. [Adding the Dependency](#1-adding-the-dependency)
2. [Core Concepts](#2-core-concepts)
3. [Extracting RTC from Packet Headers](#3-extracting-rtc-from-packet-headers)
4. [Processing Time Packets (0x11)](#4-processing-time-packets-0x11)
5. [Building a Correlation Table](#5-building-a-correlation-table)
6. [Resolving Data Packet Timestamps](#6-resolving-data-packet-timestamps)
7. [Multi-Channel Time Sources](#7-multi-channel-time-sources)
8. [Detecting GPS Lock and Time Jumps](#8-detecting-gps-lock-and-time-jumps)
9. [Parsing Intra-Packet Timestamps](#9-parsing-intra-packet-timestamps)
10. [Parsing Secondary Header Time](#10-parsing-secondary-header-time)
11. [Working with All Four Time Formats](#11-working-with-all-four-time-formats)
12. [End-to-End File Processing](#12-end-to-end-file-processing)
13. [Processing Network Time Packets (0x12)](#13-processing-network-time-packets-0x12)
14. [Working with the Leap Second Table](#14-working-with-the-leap-second-table)
15. [Correlating with F1 + F2 Sources](#15-correlating-with-f1--f2-sources)
16. [Integration with irig106-core](#16-integration-with-irig106-core)
17. [Integration with irig106-decode](#17-integration-with-irig106-decode)
18. [Integration with irig106-write](#18-integration-with-irig106-write)
19. [WASM / no_std Usage](#19-wasm--no_std-usage)
20. [Error Handling Patterns](#20-error-handling-patterns)
21. [Performance Considerations](#21-performance-considerations)

---

## 1. Adding the Dependency

```toml
# Cargo.toml
[dependencies]
irig106-time = "0.2"

# For no_std environments (embedded, WASM):
# irig106-time = { version = "0.2", default-features = false }
```

The crate re-exports all key types at the root, so most code only needs:

```rust
use irig106_time::*;
```

For module-specific types that aren't re-exported:

```rust
use irig106_time::bcd::DayFormatTime;
use irig106_time::csdw::{TimeF1Csdw, DateFormat};
use irig106_time::secondary::{parse_secondary_header, SecHdrTimeFormat};
use irig106_time::intra_packet::{parse_intra_packet_time, IntraPacketTimeFormat};
use irig106_time::network_time::{
    parse_time_f2_payload, NtpTime, PtpTime, NetworkTime,
    NetworkTimeProtocol, TimeF2Csdw, LeapSecondTable, LeapSecondEntry,
};
```

---

## 2. Core Concepts

Before diving into code, understand the three layers of time in Chapter 10:

```
Layer 1: RTC (Relative Time Counter)
  - Every packet header carries a 48-bit RTC at bytes [16..22]
  - Free-running 10 MHz counter (100 ns per tick)
  - No inherent meaning — could start at zero or random on power-on
  - This is what you have for EVERY packet

Layer 2: Time Data Packets (Type 0x11 or 0x12)
  - Periodic packets (~1/sec/channel) that pair an RTC with absolute time
  - Format 1 (0x11): BCD-encoded IRIG/GPS/RTC time (Day-of-Year or Day-Month-Year)
  - Format 2 (0x12): Network time — NTP (UTC, epoch 1900) or PTP (TAI, epoch 1970)
  - Multiple channels possible (GPS on ch3, IRIG-B on ch7, PTP on ch10, etc.)
  - This is what you COLLECT into a correlation table

Layer 3: Correlation
  - Given any packet's RTC, find the nearest time reference point
  - Interpolate: abs_time = ref_time + (target_rtc - ref_rtc) * 100ns
  - F1 and F2 reference points can coexist in the same correlator
  - This is what you DO to get human-readable timestamps
```

---

## 3. Extracting RTC from Packet Headers

Every Chapter 10 packet has a 24-byte primary header. The RTC lives at bytes
16 through 21 (6 bytes, little-endian).

```rust
use irig106_time::Rtc;

/// Extract the RTC from a raw 24-byte packet header buffer.
fn extract_rtc(header_buf: &[u8]) -> Rtc {
    Rtc::from_le_bytes([
        header_buf[16], header_buf[17], header_buf[18],
        header_buf[19], header_buf[20], header_buf[21],
    ])
}

// If you already have the raw u64 from your packet parser:
let rtc = Rtc::from_raw(0x0000_0098_9680);

// Convert to nanoseconds since counter start:
let nanos = rtc.as_raw();    // raw tick count
let ns = rtc.to_nanos();     // ticks * 100

// Compute elapsed time between two RTCs (handles 48-bit wrap):
let earlier = Rtc::from_raw(10_000_000);  // 1 second in
let later = Rtc::from_raw(20_000_000);    // 2 seconds in
let elapsed_ns = earlier.elapsed_nanos(later); // 1_000_000_000 (1 second)
```

---

## 4. Processing Time Packets (0x11)

When your packet parser encounters `data_type == 0x11`, the payload contains
a 4-byte CSDW followed by a BCD time message. The CSDW tells you how to
interpret the time message.

```rust
use irig106_time::Rtc;
use irig106_time::csdw::{TimeF1Csdw, DateFormat};
use irig106_time::bcd::{DayFormatTime, DmyFormatTime};
use irig106_time::AbsoluteTime;

/// Process a Time Data Format 1 packet payload.
///
/// `payload` starts AFTER the packet header (and secondary header if present).
/// `rtc` is the RTC extracted from the packet header.
fn process_time_packet(payload: &[u8], rtc: Rtc) -> Option<(Rtc, AbsoluteTime)> {
    // Step 1: Parse the 4-byte CSDW at the start of the payload
    if payload.len() < 4 {
        return None;
    }
    let csdw = TimeF1Csdw::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]);

    // You can inspect the time source and format:
    // csdw.time_source() → GPS, External, Internal, etc.
    // csdw.time_format() → IrigB, IrigA, Gps, Utc, etc.
    // csdw.is_leap_year() → true/false

    // Step 2: Decode the BCD time message (starts at byte 4)
    let bcd_data = &payload[4..];
    let abs_time = match csdw.date_format() {
        DateFormat::DayOfYear => {
            let day_time = DayFormatTime::from_le_bytes(bcd_data).ok()?;
            day_time.to_absolute()
        }
        DateFormat::DayMonthYear => {
            let dmy_time = DmyFormatTime::from_le_bytes(bcd_data).ok()?;
            dmy_time.to_absolute()
        }
    };

    Some((rtc, abs_time))
}
```

---

## 5. Building a Correlation Table

As you scan through a file, feed every time packet into the correlator.

```rust
use irig106_time::{Rtc, AbsoluteTime, TimeCorrelator};

let mut correlator = TimeCorrelator::new();

// During your file scan, for each time packet:
// (channel_id comes from the packet header's Channel ID field)
let channel_id: u16 = 3;
let rtc = Rtc::from_raw(10_000_000);
let abs_time = AbsoluteTime::new(100, 12, 30, 25, 340_000_000).unwrap();

correlator.add_reference(channel_id, rtc, abs_time);

// The correlator maintains references sorted by RTC for efficient lookup.
// Typical recording: ~1 ref/sec/channel → ~3600 refs for a 1-hour file.
println!("Loaded {} reference points", correlator.len());
```

---

## 6. Resolving Data Packet Timestamps

This is the hot path — it runs for every data packet you want to timestamp.

```rust
use irig106_time::{Rtc, TimeCorrelator};

fn resolve_packet_time(
    correlator: &TimeCorrelator,
    packet_rtc: Rtc,
) -> String {
    match correlator.correlate(packet_rtc, None) {
        Ok(abs) => {
            let ms = abs.nanoseconds / 1_000_000;
            format!(
                "Day {:03} {:02}:{:02}:{:02}.{:03}",
                abs.day_of_year, abs.hours, abs.minutes, abs.seconds, ms
            )
        }
        Err(e) => format!("Time unavailable: {}", e),
    }
}

// In your packet processing loop:
// for each packet in file {
//     let rtc = extract_rtc(&packet_header);
//     let timestamp = resolve_packet_time(&correlator, rtc);
//     // Use timestamp in your analysis...
// }
```

---

## 7. Multi-Channel Time Sources

Real recordings often have multiple time sources — GPS on one channel, IRIG-B
on another. The correlator lets you choose.

```rust
use irig106_time::{Rtc, AbsoluteTime, TimeCorrelator};

let mut correlator = TimeCorrelator::new();

// Channel 3: GPS time (accurate but may jump on lock)
correlator.add_reference(
    3,
    Rtc::from_raw(10_000_000),
    AbsoluteTime::new(100, 12, 0, 3, 500_000_000).unwrap(), // GPS says 12:00:03.5
);

// Channel 7: IRIG-B time (stable but may have a fixed offset)
correlator.add_reference(
    7,
    Rtc::from_raw(10_000_000),
    AbsoluteTime::new(100, 12, 0, 0, 0).unwrap(), // IRIG-B says 12:00:00.0
);

let target = Rtc::from_raw(20_000_000); // 1 second later

// Use any available source (picks nearest RTC regardless of channel):
let any_time = correlator.correlate(target, None).unwrap();

// Use only GPS:
let gps_time = correlator.correlate(target, Some(3)).unwrap();

// Use only IRIG-B:
let irig_time = correlator.correlate(target, Some(7)).unwrap();

// These will differ by the offset between the sources.
// Per RCC 123-20 §6.6: "It is usually most correct to select one time
// channel only and use this channel exclusively."
```

### Choosing a Time Source

A common pattern is to let the user select, or auto-detect the best source:

```rust
/// Pick the best time channel based on source type.
fn pick_best_channel(correlator: &TimeCorrelator) -> Option<u16> {
    // Prefer GPS, then External/IRIG-B, then Internal
    let refs = correlator.references();
    
    // Collect unique channel IDs
    let mut channels: Vec<u16> = refs.iter().map(|r| r.channel_id).collect();
    channels.sort();
    channels.dedup();
    
    // In practice you'd check the CSDW TimeSource for each channel.
    // For now, return the channel with the most reference points:
    channels.into_iter()
        .max_by_key(|ch| refs.iter().filter(|r| r.channel_id == *ch).count())
}
```

---

## 8. Detecting GPS Lock and Time Jumps

Before GPS acquires satellites, the internal clock may be seconds off. When GPS
locks, absolute time jumps forward while the RTC continues smoothly.

```rust
use irig106_time::{Rtc, AbsoluteTime, TimeCorrelator};

let mut correlator = TimeCorrelator::new();

// Pre-GPS-lock (internal clock is ~5 seconds behind):
correlator.add_reference(3, Rtc::from_raw(10_000_000),
    AbsoluteTime::new(50, 14, 0, 0, 0).unwrap());

// 1 second later by RTC, but GPS corrects the clock:
correlator.add_reference(3, Rtc::from_raw(20_000_000),
    AbsoluteTime::new(50, 14, 0, 6, 0).unwrap()); // jumped +5 sec

// After GPS lock, time progresses normally:
correlator.add_reference(3, Rtc::from_raw(30_000_000),
    AbsoluteTime::new(50, 14, 0, 7, 0).unwrap());

// Detect jumps with a 1-second threshold:
let jumps = correlator.detect_time_jump(3, 1_000_000_000);

for jump in &jumps {
    let delta_ms = jump.delta_nanos as f64 / 1_000_000.0;
    println!(
        "Time jump on channel {}: {:+.1} ms at reference #{}",
        jump.channel_id, delta_ms, jump.index
    );
}
// Output: "Time jump on channel 3: +4000.0 ms at reference #1"

// IMPORTANT: For data before the GPS lock, correlation using pre-lock
// references will give the wrong time. Consider:
// 1. Only using post-lock references
// 2. Warning the user about pre-lock data
// 3. Offering both pre-lock and post-lock interpretations
```

---

## 9. Parsing Intra-Packet Timestamps

Many data types (1553, PCM, Ethernet, etc.) include per-message intra-packet
time stamps within the packet body. These are 8-byte structures whose format
depends on the packet flags.

```rust
use irig106_time::intra_packet::{parse_intra_packet_time, IntraPacketTimeFormat};
use irig106_time::{IntraPacketTime, Rtc, TimeCorrelator};

/// Determine the intra-packet time format from packet flags.
fn ipt_format_from_flags(packet_flags: u8) -> IntraPacketTimeFormat {
    IntraPacketTimeFormat::from_packet_flags(packet_flags)
}

/// Parse a single intra-packet timestamp and resolve it.
fn resolve_message_time(
    ipt_buf: &[u8],       // 8 bytes from the message header
    packet_flags: u8,
    correlator: &TimeCorrelator,
) -> Option<String> {
    let fmt = ipt_format_from_flags(packet_flags);
    let ipt = parse_intra_packet_time(ipt_buf, fmt).ok()?;

    match ipt {
        IntraPacketTime::Rtc(rtc) => {
            // Most common case: relative time → correlate
            let abs = correlator.correlate(rtc, None).ok()?;
            Some(format!("Day {:03} {:02}:{:02}:{:02}.{:09}",
                abs.day_of_year, abs.hours, abs.minutes, abs.seconds, abs.nanoseconds))
        }
        IntraPacketTime::Ieee1588(t) => {
            // Absolute time — no correlation needed
            Some(format!("{} seconds + {} ns since IEEE-1588 epoch",
                t.seconds, t.nanoseconds))
        }
        IntraPacketTime::Ch4(bwt) => {
            let abs = bwt.to_absolute().ok()?;
            Some(format!("Day {:03} {:02}:{:02}:{:02}",
                abs.day_of_year, abs.hours, abs.minutes, abs.seconds))
        }
        IntraPacketTime::Ertc(ertc) => {
            Some(format!("{} ns (ERTC)", ertc.to_nanos()))
        }
    }
}
```

### Example: Processing 1553 Messages

```rust
use irig106_time::Rtc;
use irig106_time::intra_packet::{parse_intra_packet_time, IntraPacketTimeFormat};

/// Walk 1553 messages within a packet body.
/// Each message has: [8-byte IPT][2-byte IPH][variable data]
fn walk_1553_messages(
    body: &[u8],
    msg_count: usize,
    packet_flags: u8,
    correlator: &irig106_time::TimeCorrelator,
) {
    let fmt = IntraPacketTimeFormat::from_packet_flags(packet_flags);
    let mut offset = 4; // skip CSDW

    for i in 0..msg_count {
        if offset + 8 > body.len() { break; }

        // Parse intra-packet time stamp
        let ipt = parse_intra_packet_time(&body[offset..offset + 8], fmt);
        offset += 8;

        // Parse intra-packet data header (2 bytes for 1553)
        if offset + 2 > body.len() { break; }
        let _ipdh = u16::from_le_bytes([body[offset], body[offset + 1]]);
        offset += 2;

        // Resolve to absolute time
        if let Ok(irig106_time::IntraPacketTime::Rtc(rtc)) = ipt {
            if let Ok(abs) = correlator.correlate(rtc, None) {
                println!("  Message {}: Day {} {:02}:{:02}:{:02}.{:06}",
                    i, abs.day_of_year, abs.hours, abs.minutes, abs.seconds,
                    abs.nanoseconds / 1_000);
            }
        }

        // Skip message data (size depends on 1553 word count)
        // ... your 1553 parsing logic here ...
    }
}
```

---

## 10. Parsing Secondary Header Time

Packets with bit 2 set in the flags byte have a 12-byte secondary header
between the primary header and the body.

```rust
use irig106_time::secondary::{
    parse_secondary_header, validate_secondary_checksum, SecHdrTimeFormat,
};
use irig106_time::SecondaryHeaderTime;

fn process_secondary_header(
    sec_hdr_buf: &[u8],   // 12 bytes after the primary header
    packet_flags: u8,
) {
    // Determine format from packet flags bits [3:2]
    let fmt = SecHdrTimeFormat::from_packet_flags(packet_flags);

    // Parse (validates checksum internally)
    match parse_secondary_header(sec_hdr_buf, fmt) {
        Ok(SecondaryHeaderTime::Ch4(bwt)) => {
            if let Ok(abs) = bwt.to_absolute() {
                println!("Secondary header: Day {} {:02}:{:02}:{:02}",
                    abs.day_of_year, abs.hours, abs.minutes, abs.seconds);
            }
        }
        Ok(SecondaryHeaderTime::Ieee1588(t)) => {
            println!("Secondary header: {} sec + {} ns",
                t.seconds, t.nanoseconds);
        }
        Ok(SecondaryHeaderTime::Ertc(e)) => {
            println!("Secondary header: {} ns (ERTC)", e.to_nanos());
        }
        Err(e) => {
            eprintln!("Secondary header error: {}", e);
        }
    }
}

// You can also validate the checksum separately if you want to
// skip parsing on failure:
fn is_secondary_valid(buf: &[u8]) -> bool {
    validate_secondary_checksum(buf).is_ok()
}
```

---

## 11. Working with All Four Time Formats

The crate handles four distinct time representations. Here is a quick reference
for when you encounter each one:

```rust
use irig106_time::*;
use irig106_time::absolute::{Ch4BinaryTime, Ieee1588Time, Ertc};

// ── 1. The 48-bit RTC ─────────────────────────────────────────────
// WHERE: Every packet header, most intra-packet timestamps
// WHAT:  Relative counter, no calendar meaning
// HOW:   Correlate against time packet reference points
let rtc = Rtc::from_le_bytes([0x80, 0x96, 0x98, 0x00, 0x00, 0x00]);
let ns = rtc.to_nanos(); // 1_000_000_000 ns = 1 second

// ── 2. Chapter 4 Binary Weighted Time ─────────────────────────────
// WHERE: Secondary headers (flags [3:2]=0b00), intra-packet timestamps
// WHAT:  Day-of-year + time-of-day in 10ms increments + microseconds
let bwt = Ch4BinaryTime {
    high_order: 0x0003,
    low_order: 0x4650,
    microseconds: 500,
};
let abs = bwt.to_absolute().unwrap();

// ── 3. IEEE-1588 (PTP) ────────────────────────────────────────────
// WHERE: Secondary headers (flags [3:2]=0b01), intra-packet timestamps
// WHAT:  Seconds + nanoseconds since PTP epoch
let ptp = Ieee1588Time { seconds: 1_700_000_000, nanoseconds: 500_000_000 };
let total_ns = ptp.to_nanos_since_epoch(); // single u64

// ── 4. Extended RTC (64-bit) ──────────────────────────────────────
// WHERE: Secondary headers (flags [3:2]=0b10), intra-packet timestamps
// WHAT:  Full 64-bit counter at 100 ns resolution
// WHEN:  Introduced in IRIG 106-07
let ertc = Ertc::from_le_bytes(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
let ns128 = ertc.to_nanos(); // u128 to avoid overflow
```

---

## 12. End-to-End File Processing

Here is a complete pattern for reading a Ch10 file and timestamping every
packet. This is the skeleton your application should follow.

```rust
use irig106_time::*;
use irig106_time::bcd::{DayFormatTime, DmyFormatTime};
use irig106_time::csdw::{TimeF1Csdw, DateFormat};
use irig106_time::network_time::{parse_time_f2_payload, LeapSecondTable};

const SYNC: u16 = 0xEB25;
const HEADER_SIZE: usize = 24;
const TIME_F1: u8 = 0x11;
const TIME_F2: u8 = 0x12;

struct AnalysisResult {
    packet_num: usize,
    channel_id: u16,
    data_type: u8,
    absolute_time: Option<AbsoluteTime>,
}

fn analyze_file(data: &[u8]) -> Vec<AnalysisResult> {
    let mut correlator = TimeCorrelator::new();
    let leap_table = LeapSecondTable::builtin();
    let mut results = Vec::new();
    let mut offset = 0;
    let mut pkt_num = 0;

    // ── Pass 1: Collect all time references ──────────────────────
    // (In practice you can do this in a single pass if time packets
    //  arrive before the data they need to timestamp, which the spec
    //  requires for the first time packet.)
    while offset + HEADER_SIZE <= data.len() {
        let sync = u16::from_le_bytes([data[offset], data[offset + 1]]);
        if sync != SYNC { offset += 1; continue; }

        let pkt_len = u32::from_le_bytes([
            data[offset + 4], data[offset + 5],
            data[offset + 6], data[offset + 7],
        ]) as usize;
        if pkt_len < HEADER_SIZE || offset + pkt_len > data.len() { break; }

        let channel_id = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let flags = data[offset + 14];
        let data_type = data[offset + 15];
        let rtc = Rtc::from_le_bytes([
            data[offset + 16], data[offset + 17], data[offset + 18],
            data[offset + 19], data[offset + 20], data[offset + 21],
        ]);

        if data_type == TIME_F1 {
            let body_start = HEADER_SIZE
                + if (flags & 0x04) != 0 { 12 } else { 0 };

            if offset + body_start + 12 <= offset + pkt_len {
                let payload = &data[offset + body_start..offset + pkt_len];
                let csdw = TimeF1Csdw::from_le_bytes([
                    payload[0], payload[1], payload[2], payload[3],
                ]);

                let abs = match csdw.date_format() {
                    DateFormat::DayOfYear => {
                        DayFormatTime::from_le_bytes(&payload[4..])
                            .ok()
                            .map(|t| t.to_absolute())
                    }
                    DateFormat::DayMonthYear => {
                        DmyFormatTime::from_le_bytes(&payload[4..])
                            .ok()
                            .map(|t| t.to_absolute())
                    }
                };

                if let Some(time) = abs {
                    correlator.add_reference(channel_id, rtc, time);
                }
            }
        }

        // Format 2: Network Time (NTP / PTP)
        if data_type == TIME_F2 {
            let body_start = HEADER_SIZE
                + if (flags & 0x04) != 0 { 12 } else { 0 };
            let payload = &data[offset + body_start..offset + pkt_len];

            if let Ok((_, net_time)) = parse_time_f2_payload(payload) {
                let _ = correlator.add_reference_f2(
                    channel_id, rtc, &net_time, &leap_table,
                );
            }
        }

        offset += pkt_len;
    }

    // ── Pass 2: Resolve all packet timestamps ────────────────────
    offset = 0;
    pkt_num = 0;
    while offset + HEADER_SIZE <= data.len() {
        let sync = u16::from_le_bytes([data[offset], data[offset + 1]]);
        if sync != SYNC { offset += 1; continue; }

        let pkt_len = u32::from_le_bytes([
            data[offset + 4], data[offset + 5],
            data[offset + 6], data[offset + 7],
        ]) as usize;
        if pkt_len < HEADER_SIZE || offset + pkt_len > data.len() { break; }

        let channel_id = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        let data_type = data[offset + 15];
        let rtc = Rtc::from_le_bytes([
            data[offset + 16], data[offset + 17], data[offset + 18],
            data[offset + 19], data[offset + 20], data[offset + 21],
        ]);

        results.push(AnalysisResult {
            packet_num: pkt_num,
            channel_id,
            data_type,
            absolute_time: correlator.correlate(rtc, None).ok(),
        });

        pkt_num += 1;
        offset += pkt_len;
    }

    results
}
```

---

## 13. Processing Network Time Packets (0x12)

*Added in v0.2.0.* Format 2 Network Time packets (Data Type 0x12) carry NTP or PTP
timestamps instead of BCD-encoded IRIG time. The workflow is simpler than Format 1
since there's no BCD decoding — just binary seconds and fractional/nanosecond fields.

```rust
use irig106_time::Rtc;
use irig106_time::network_time::{
    parse_time_f2_payload, NetworkTime, NetworkTimeProtocol,
    TimeF2Csdw, NtpTime, PtpTime, LeapSecondTable,
};

/// Process a Time Data Format 2 packet payload.
///
/// `payload` starts AFTER the packet header (and secondary header if present).
/// `rtc` is the RTC extracted from the packet header.
fn process_time_f2_packet(
    payload: &[u8],
    rtc: Rtc,
    leap_table: &LeapSecondTable,
) -> Option<irig106_time::AbsoluteTime> {
    let (csdw, network_time) = parse_time_f2_payload(payload).ok()?;

    match network_time {
        NetworkTime::Ntp(ntp) => {
            // NTP: UTC epoch (1900-01-01), no leap second conversion needed
            ntp.to_absolute().ok()
        }
        NetworkTime::Ptp(ptp) => {
            // PTP: TAI epoch (1970-01-01), must apply leap-second offset
            let offset = leap_table.offset_at_tai(ptp.seconds);
            ptp.to_absolute(offset).ok()
        }
    }
}
```

### NTP vs PTP: Key Differences

```rust
use irig106_time::network_time::{NtpTime, PtpTime, NTP_UNIX_EPOCH_OFFSET};

// ── NTP ──────────────────────────────────────────────────────────
// Epoch: 1900-01-01 00:00:00 UTC
// Timescale: UTC (leap seconds already applied)
// Resolution: 2⁻³² seconds ≈ 233 picoseconds
let ntp = NtpTime { seconds: 3_944_678_400, fraction: 1 << 31 };
let unix_secs = ntp.to_unix_seconds().unwrap();  // subtract 2,208,988,800
let nanos = ntp.fraction_as_nanos();             // ~500,000,000

// ── PTP ──────────────────────────────────────────────────────────
// Epoch: 1970-01-01 00:00:00 TAI (not UTC!)
// Timescale: TAI (no leap seconds — monotonic)
// Resolution: 1 nanosecond
let ptp = PtpTime { seconds: 1_735_689_637, nanoseconds: 0 };
let utc_secs = ptp.to_utc_seconds(37);           // TAI - 37 = UTC
// 1_735_689_637 - 37 = 1_735_689_600 = 2025-01-01 00:00:00 UTC
```

---

## 14. Working with the Leap Second Table

PTP time is in TAI, which diverges from UTC by the accumulated leap-second offset.
The crate ships a built-in table covering all 28 leap seconds from 1972 to 2017.

```rust
use irig106_time::network_time::{LeapSecondTable, LeapSecondEntry};

// Use the built-in table (28 entries, 1972–2017)
let table = LeapSecondTable::builtin();

// Look up offset for a known UTC timestamp
let offset_2024 = table.offset_at_unix(1_718_409_600); // mid-2024
assert_eq!(offset_2024, 37); // 37 seconds since 2017-01-01

let offset_1995 = table.offset_at_unix(800_000_000);   // ~1995
assert_eq!(offset_1995, 29);

// For PTP timestamps (which are in TAI), use offset_at_tai
// This approximates the UTC time first, then looks up the offset
let tai_offset = table.offset_at_tai(1_735_689_637);
assert_eq!(tai_offset, 37);
```

### Adding Future Leap Seconds

If a new leap second is announced after the crate was released, you can extend
the table at runtime:

```rust
use irig106_time::network_time::{LeapSecondTable, LeapSecondEntry};

let mut table = LeapSecondTable::builtin();

// Hypothetical: leap second on 2028-07-01
table.add(LeapSecondEntry {
    effective_unix: 1_845_849_600, // 2028-07-01 00:00:00 UTC
    tai_utc_offset: 38,
});

// Now lookups after 2028-07-01 return 38
assert_eq!(table.offset_at_unix(1_900_000_000), 38);
```

---

## 15. Correlating with F1 + F2 Sources

Real recordings may contain both Format 1 (BCD) and Format 2 (NTP/PTP) time
channels. The correlator handles both through separate methods.

```rust
use irig106_time::*;
use irig106_time::bcd::DayFormatTime;
use irig106_time::csdw::{TimeF1Csdw, DateFormat};
use irig106_time::network_time::{
    parse_time_f2_payload, NetworkTime, LeapSecondTable,
};

const TIME_F1: u8 = 0x11;
const TIME_F2: u8 = 0x12;

let mut correlator = TimeCorrelator::new();
let leap_table = LeapSecondTable::builtin();

// During your file scan, dispatch on data_type:
// (simplified — real code would parse headers first)
fn ingest_time_packet(
    data_type: u8,
    channel_id: u16,
    rtc: Rtc,
    payload: &[u8],
    correlator: &mut TimeCorrelator,
    leap_table: &LeapSecondTable,
) {
    match data_type {
        0x11 => {
            // Format 1: BCD
            let csdw = TimeF1Csdw::from_le_bytes([
                payload[0], payload[1], payload[2], payload[3],
            ]);
            let abs = match csdw.date_format() {
                DateFormat::DayOfYear => {
                    DayFormatTime::from_le_bytes(&payload[4..])
                        .ok().map(|t| t.to_absolute())
                }
                DateFormat::DayMonthYear => {
                    irig106_time::bcd::DmyFormatTime::from_le_bytes(&payload[4..])
                        .ok().map(|t| t.to_absolute())
                }
            };
            if let Some(time) = abs {
                correlator.add_reference(channel_id, rtc, time);
            }
        }
        0x12 => {
            // Format 2: NTP/PTP
            if let Ok((_, net_time)) = parse_time_f2_payload(payload) {
                let _ = correlator.add_reference_f2(
                    channel_id, rtc, &net_time, leap_table,
                );
            }
        }
        _ => {} // not a time packet
    }
}
```

### Using `add_reference_f2`

The `add_reference_f2` method handles all the epoch conversion internally:

```rust
use irig106_time::*;
use irig106_time::network_time::{
    NetworkTime, NtpTime, PtpTime, LeapSecondTable,
};

let mut correlator = TimeCorrelator::new();
let table = LeapSecondTable::builtin();

// NTP source on channel 5
let ntp_time = NetworkTime::Ntp(NtpTime {
    seconds: 3_944_678_400, // 2025-01-01 00:00:00 UTC
    fraction: 0,
});
correlator.add_reference_f2(5, Rtc::from_raw(10_000_000), &ntp_time, &table).unwrap();

// PTP source on channel 8
let ptp_time = NetworkTime::Ptp(PtpTime {
    seconds: 1_735_689_637, // 2025-01-01 00:00:00 TAI (UTC + 37)
    nanoseconds: 0,
});
correlator.add_reference_f2(8, Rtc::from_raw(10_000_000), &ptp_time, &table).unwrap();

// Both channels now resolve the same RTC to the same absolute time
let from_ntp = correlator.correlate(Rtc::from_raw(10_000_000), Some(5)).unwrap();
let from_ptp = correlator.correlate(Rtc::from_raw(10_000_000), Some(8)).unwrap();
assert_eq!(from_ntp.day_of_year, from_ptp.day_of_year);
assert_eq!(from_ntp.hours, from_ptp.hours);
```

---

## 16. Integration with irig106-core

When `irig106-core` provides packet header parsing, the integration simplifies:

```rust
// Future pattern (when irig106-core uses irig106-types::Rtc):
//
// use irig106_core::PacketHeader;
// use irig106_time::{TimeCorrelator, Rtc};
//
// fn process_packet(header: &PacketHeader, correlator: &TimeCorrelator) {
//     // irig106-core would provide header.rtc as an Rtc directly
//     let rtc: Rtc = header.rtc;
//     let timestamp = correlator.correlate(rtc, None);
//     // ...
// }

// Current pattern (irig106-ch10-reader uses raw u64):
// Convert the raw u64 to our Rtc type:
fn rtc_from_reader(raw_rtc: u64) -> irig106_time::Rtc {
    irig106_time::Rtc::from_raw(raw_rtc)
}
```

---

## 17. Integration with irig106-decode

Payload decoders need message-level timestamps from intra-packet time stamps.

```rust
// Future pattern for a 1553 decoder:
//
// use irig106_decode::mil1553::Message;
// use irig106_time::{TimeCorrelator, IntraPacketTime};
// use irig106_time::intra_packet::{parse_intra_packet_time, IntraPacketTimeFormat};
//
// fn decode_with_timestamps(
//     messages: &[Message],
//     correlator: &TimeCorrelator,
//     ipt_format: IntraPacketTimeFormat,
// ) {
//     for msg in messages {
//         let ipt = parse_intra_packet_time(&msg.timestamp_bytes, ipt_format);
//         if let Ok(IntraPacketTime::Rtc(rtc)) = ipt {
//             let abs = correlator.correlate(rtc, None).unwrap();
//             println!("{}: RT={}, SA={}, WC={}",
//                 format_time(&abs), msg.rt, msg.subaddress, msg.word_count);
//         }
//     }
// }
```

---

## 18. Integration with irig106-write

When constructing Ch10 files, you need to encode time data. The current crate
provides decode only. Encode (`to_le_bytes`) is planned for v0.2 (see ROADMAP.md).

The pattern will be:

```rust
// Future (after GAP-11 is resolved):
//
// use irig106_time::bcd::DayFormatTime;
// use irig106_time::csdw::TimeF1Csdw;
//
// // Build a time packet payload
// let csdw = TimeF1Csdw::build(TimeSource::Gps, TimeFormat::IrigB, false, DateFormat::DayOfYear);
// let day_time = DayFormatTime {
//     milliseconds: 340,
//     seconds: 25,
//     minutes: 30,
//     hours: 12,
//     day_of_year: 100,
// };
//
// let mut payload = Vec::new();
// payload.extend_from_slice(&csdw.to_le_bytes());
// payload.extend_from_slice(&day_time.to_le_bytes());
```

---

## 19. WASM / no_std Usage

The crate compiles to `no_std` targets including WebAssembly.

```toml
# Cargo.toml for a WASM project
[dependencies]
irig106-time = { version = "0.2", default-features = false }
```

```rust
#![no_std]
extern crate alloc;

use irig106_time::*;

// All types work in no_std. The correlation module requires `alloc`
// for Vec-backed reference storage.
pub fn correlate_in_browser(rtc_raw: u64) -> (u16, u8, u8, u8, u32) {
    let correlator: TimeCorrelator = todo!("populated from JS");
    let rtc = Rtc::from_raw(rtc_raw);
    match correlator.correlate(rtc, None) {
        Ok(t) => (t.day_of_year, t.hours, t.minutes, t.seconds, t.nanoseconds),
        Err(_) => (0, 0, 0, 0, 0),
    }
}
```

---

## 20. Error Handling Patterns

Every fallible function returns `Result<T, TimeError>`. The error enum is
`#[non_exhaustive]` so new variants can be added without breaking your code.

```rust
use irig106_time::{TimeError, Rtc, TimeCorrelator};
use irig106_time::bcd::DayFormatTime;

fn robust_time_parse(buf: &[u8]) -> String {
    match DayFormatTime::from_le_bytes(buf) {
        Ok(t) => {
            let abs = t.to_absolute();
            format!("Day {} {:02}:{:02}:{:02}", abs.day_of_year,
                abs.hours, abs.minutes, abs.seconds)
        }
        Err(TimeError::InvalidBcdDigit { nibble, position }) => {
            format!("Corrupt BCD: nibble 0x{:X} at {}", nibble, position)
        }
        Err(TimeError::ReservedBitSet { position }) => {
            format!("Non-conformant: reserved bit set at {}", position)
        }
        Err(TimeError::OutOfRange { field, value, max }) => {
            format!("Invalid time: {} = {} (max {})", field, value, max)
        }
        Err(TimeError::BufferTooShort { expected, actual }) => {
            format!("Truncated: need {} bytes, got {}", expected, actual)
        }
        Err(e) => format!("Error: {}", e),
    }
}

/// Pattern for correlation with graceful degradation:
fn resolve_or_raw(correlator: &TimeCorrelator, rtc: Rtc) -> String {
    match correlator.correlate(rtc, None) {
        Ok(t) => format!("Day {} {:02}:{:02}:{:02}.{:03}",
            t.day_of_year, t.hours, t.minutes, t.seconds,
            t.nanoseconds / 1_000_000),
        Err(TimeError::NoReferencePoint) => {
            // No time packets processed yet — fall back to raw RTC
            format!("RTC {} ({:.3}s)", rtc.as_raw(),
                rtc.to_nanos() as f64 / 1_000_000_000.0)
        }
        Err(e) => format!("Error: {}", e),
    }
}
```

---

## 21. Performance Considerations

### Hot Path vs. Cold Path

```
HOT PATH (every data packet):          COLD PATH (time packets only, ~1/sec):
  Rtc::from_le_bytes    ~7 ns            TimeF1Csdw::from_le_bytes   ~0.3 ns
  correlator.correlate  ~14-23 ns        DayFormatTime::from_le_bytes ~6 ns
  ────────────────────────────            to_absolute                 ~2 ns
  Total:                ~31 ns            add_reference               ~varies
                                          ──────────────────────────────
  Budget at 10 Gbps:    ~416 ns/pkt      Happens 1x/sec, not on hot path
  Headroom:             13x
```

### Tips for Maximum Throughput

1. **Build the correlator in a first pass** (or as time packets arrive), then
   resolve data packets in a second pass. This avoids interleaving slow BCD
   parsing with fast RTC correlation.

2. **Use `correlate(rtc, None)`** unless you specifically need per-channel
   resolution. The any-channel path uses binary search (O(log n), ~14 ns).
   Channel-filtered uses linear scan (O(n), ~304 ns at 100 refs).

3. **Don't allocate in the hot path.** `Rtc`, `AbsoluteTime`, and
   `IntraPacketTime` are all `Copy` types. No heap allocation occurs during
   correlation.

4. **The correlator is immutable after loading.** In a multi-threaded pipeline,
   build it in one thread, then share `&TimeCorrelator` (which is `Send + Sync`
   via the `Vec<ReferencePoint>` backing) across processing threads.
