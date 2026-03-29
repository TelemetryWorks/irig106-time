# UDP Transfer Header and Time Correlation

**Document:** udp_framing.md
**Crate:** irig106-time v0.7.0
**Date:** 2026-03-29

---

## Overview

IRIG 106 Chapter 10 defines two UDP transfer formats for streaming Ch10
packets over a network. Neither format introduces additional time fields —
time correlation uses the same packet-level RTC regardless of whether the
data arrived from a file or a UDP stream.

This document describes the interaction between UDP framing and the
`irig106-time` correlation engine.

## UDP Transfer Formats

### Format 1 (Simple)

The Format 1 UDP transfer header is 4 bytes:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | Sequence number (32-bit) |

The sequence number is a monotonically increasing counter used to detect
dropped packets. **No time fields are present.** The Ch10 packet follows
immediately after the 4-byte header.

### Format 2 (Extended)

The Format 2 UDP transfer header is 12 bytes:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | Sequence number (32-bit) |
| 4 | 1 | Type (0x00 = Full packet, 0x01 = Segmented) |
| 5 | 3 | Reserved |
| 8 | 4 | Segment offset (for segmented transfers) |

Again, **no time fields** are present in the transfer header. Segmented
transfers split large Ch10 packets across multiple UDP datagrams, but
the time information within the reassembled packet is identical to the
file-based format.

## Time Correlation with UDP Streams

### No Change to RTC Semantics

The 48-bit RTC in each Ch10 packet header has the same semantics whether
the packet is read from a file or received over UDP:

- Time Data Format 1 (0x11) and Format 2 (0x12) packets carry absolute
  time references paired with the packet's RTC
- Data packets carry their RTC in the primary header
- The correlation engine resolves data packet RTCs against the nearest
  time reference point

### Arrival Order vs. File Order

The key difference with UDP streams is **packet ordering**. File-based
processing sees packets in file order (which is write order). UDP streams
may deliver packets:

- Out of order (due to network routing)
- With gaps (due to packet loss)
- With duplicates (due to network retransmission)

This is why `irig106-time` provides two correlators:

| Correlator | Use Case |
|-----------|----------|
| `TimeCorrelator` | Post-processing complete files. Stores all reference points sorted by RTC. |
| `StreamingTimeCorrelator` | Live UDP streams. Sliding window with max-age eviction. Handles arrival-order insertion. |

### Recommended Pattern

```rust
use irig106_time::streaming::StreamingTimeCorrelator;
use irig106_time::{Rtc, AbsoluteTime};

// 60-second sliding window for a 1 Hz time source
let mut correlator = StreamingTimeCorrelator::new(60_000_000_000);

// For each received UDP datagram:
// 1. Strip the 4-byte (Format 1) or 12-byte (Format 2) transfer header
// 2. Parse the Ch10 packet header to extract the RTC
// 3. If it's a time packet (0x11 or 0x12), add as reference:
//    correlator.add_reference(channel_id, rtc, abs_time);
// 4. If it's a data packet, correlate:
//    let resolved = correlator.correlate(data_rtc, Some(channel_id));
```

### Sequence Number Gap Detection

The UDP sequence number can be used to detect dropped packets, but this
is a transport-layer concern — not a time correlation concern. If a time
reference packet is dropped, the correlator simply has a larger gap between
reference points. The `TimeQuality` metrics can flag this:

```rust
use irig106_time::quality::compute_quality;

// Periodically check correlation health
let q = compute_quality(/* snapshot of reference points */);
if let Some(gap) = q.max_rtc_gap_ns {
    if gap > 5_000_000_000 { // > 5 seconds without a time ref
        eprintln!("Warning: large time reference gap: {} ns", gap);
    }
}
```

## Summary

- UDP transfer headers carry **no time fields** — only sequence numbers
- Time correlation uses the **same RTC and time packet mechanisms** as file-based processing
- Use `StreamingTimeCorrelator` for live UDP streams
- Use sequence numbers for transport-layer gap detection, not time correlation
- Use `TimeQuality` to monitor correlation health
