# Why `irig106-time` Is a Separate Repository

**Document:** WHY_SEPARATE_REPO.md
**Date:** 2026-03-25

---

## The Short Answer

Time in IRIG 106 is not a utility — it is infrastructure. Every single crate in
the ecosystem needs time handling, but the complexity of IRIG time (BCD decoding,
multiple clock sources, RTC correlation, drift detection) belongs in exactly one
place. A dedicated crate prevents duplication, ensures consistent behavior, and
makes the time subsystem independently testable to aerospace-grade rigor.

---

## 1. Time Touches Everything

Every IRIG 106 Chapter 10 data packet has a 48-bit RTC in its header. Every
payload message may carry an intra-packet time stamp. Every analysis tool needs
to convert these relative timestamps to human-readable wall-clock time. Without a
shared time library:

- `irig106-core` would need RTC parsing
- `irig106-decode` would need intra-packet timestamp parsing
- `irig106-write` would need RTC serialization
- `irig106-ch10-reader` would need time display formatting
- `irig106-cli` would need everything above
- `irig106-studio` would need everything above again

That is six copies of the same BCD decoder, six copies of the same correlation
logic, six opportunities to get the 48-bit wrap-around math wrong.

---

## 2. Time in IRIG 106 Is Surprisingly Complex

### 2.1 It Is Not a Simple Timestamp

Unlike Unix timestamps or ISO 8601 strings, IRIG 106 time is a *system* with
moving parts:

| Component | Description |
|-----------|-------------|
| **48-bit RTC** | Free-running 10 MHz counter with no inherent meaning — it could start at zero or a random value on power-on |
| **Time Data Packets** | Periodic packets that pair the RTC with an absolute clock reading |
| **Multiple Clock Sources** | IRIG-B, GPS, UTC, internal battery-backed clock — each on its own channel, each potentially disagreeing |
| **BCD Encoding** | The absolute time is encoded in Binary Coded Decimal with two different layouts (Day-of-Year and Day-Month-Year) |
| **Drift** | The RTC crystal drifts ~10 ppm; software must use the *nearest* reference point, not an old one |
| **Time Jumps** | GPS lock events cause sudden corrections in absolute time while RTC continues smoothly |
| **Format Variants** | Secondary headers and intra-packet timestamps come in four formats (RTC, Ch4 BWT, IEEE-1588, ERTC) selected by packet flag bits |

### 2.2 Correctness Is Critical

In flight test telemetry, time is the primary key for correlating data across
channels. If the 1553 bus data says the control surface moved at T₁ and the
strain gauge data shows a load spike at T₂, the time correlation accuracy
determines whether you can trust the causal analysis. A 1 ms error in time
correlation could mean the difference between "the actuator caused the load"
and "the load caused the actuator response."

---

## 3. Single Responsibility, Independently Versioned

### 3.1 Change Frequency

Time handling changes for different reasons than packet parsing or payload
decoding:

- A new IRIG 106 revision adds a time format → only `irig106-time` changes
- A BCD decoding bug is found → fix in one place, all consumers get it
- The correlation algorithm is improved → no rebuild of the core parser

### 3.2 Testing Surface

Time code needs a specific, deep test suite:

- BCD edge cases (day 366 in leap year, midnight rollover, reserved-bit validation)
- RTC wrap-around arithmetic at the 48-bit boundary
- Multi-channel correlation with conflicting time sources
- GPS lock jump detection with configurable thresholds

These 114 tests would bloat any other crate's test suite and make CI slower for
changes that have nothing to do with time.

### 3.3 Dependency Isolation

`irig106-time` has **zero required dependencies** — only `core` and `alloc`.
This means:

- It compiles for `no_std` embedded targets (e.g., recorder firmware)
- It compiles for WASM (e.g., `irig106-studio` browser viewer)
- It adds no transitive dependency weight to the ecosystem

---

## 4. The Ecosystem Layering

```
Layer 0 (types):    irig106-types       ← shared newtypes (Rtc, etc.)
Layer 1 (core):     irig106-time        ← time interpretation
                    irig106-core        ← packet structure
Layer 2 (semantic): irig106-decode      ← payload meaning
                    irig106-write       ← packet construction
Layer 3 (tools):    irig106-ch10-reader ← file analysis
                    irig106-cli         ← command-line tools
                    irig106-studio      ← WASM viewer
```

`irig106-time` sits at Layer 1 because it is infrastructure that Layer 2 and
Layer 3 crates depend on but should not have to implement themselves.

---

## 5. Real-World Scenarios This Design Supports

### Scenario A: Multi-Source Flight Test Recording

A recorder captures IRIG-B time on channel 3 and GPS time on channel 7.
Pre-flight, only IRIG-B is available. At T+45 minutes, GPS locks. The analysis
software:

1. Parses Time F1 packets from both channels → `irig106-time::bcd`
2. Feeds them into a single `TimeCorrelator` → `irig106-time::correlation`
3. Detects the GPS lock jump → `detect_time_jump(channel=7, threshold=1s)`
4. Lets the analyst choose: use IRIG-B (stable, pre-lock) or GPS (accurate, post-lock)
5. Resolves every data packet's RTC to the selected time source

### Scenario B: Ground Station Live Stream

A ground station receives UDP-streamed Chapter 10 packets in real time.
Packets arrive out-of-order by up to 1 second (per IRIG 106-05). The
correlation engine receives time packets as they arrive and resolves
data packet timestamps using the nearest reference — the same code path
as offline file analysis.

### Scenario C: Synthetic Data Generation

`irig106-write` needs to generate valid Time F1 packets with correct BCD
encoding. It uses the same types (`DayFormatTime`, `TimeF1Csdw`) in reverse
to serialize. One source of truth for the wire format means round-trip
correctness is guaranteed.

---

## 6. Why Not Just Use `chrono`?

External time libraries solve a different problem. They handle civil time
(time zones, calendars, leap seconds). IRIG 106 time is:

- **Day-of-year based** (no months/weekdays in DOY format)
- **BCD encoded** on the wire (not a simple integer)
- **Relative to an arbitrary counter epoch** (the RTC)
- **Domain-specific** in its concept of "time source" and "time format"
- **Required to be `no_std`** for embedded and WASM targets

`chrono` is a fine crate, but it would be an awkward, heavyweight dependency
that doesn't model the actual problem. The 400 lines of time logic in this
crate are a better fit than 40,000 lines of calendar math we don't need.

---

## 7. Summary

| Concern | Without `irig106-time` | With `irig106-time` |
|---------|----------------------|---------------------|
| BCD decoding | Duplicated 6× across crates | Single implementation, 114 tests |
| RTC correlation | Each tool reimplements | Shared engine with multi-channel + jump detection |
| Wire format truth | Scattered struct definitions | One source of truth, migration path to `irig106-types` |
| `no_std` support | Each crate decides independently | Guaranteed by design |
| Versioning | Time bugs require coordinated releases | Fix once, bump once |
| Aerospace rigor | Requirements scattered or absent | L1→L2→L3 traced to IRIG 106 standard |
