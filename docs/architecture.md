# Architecture — irig106-time

**Document:** ARCHITECTURE.md
**Crate:** irig106-time v0.1.0
**Date:** 2026-03-25

---

## 1. Why Time Is Hard in IRIG 106

Time in IRIG 106 Chapter 10 is fundamentally different from everyday timestamps.
The standard separates *when data was recorded* (the free-running RTC) from *what
wall-clock time it was* (absolute time from an external source). These two notions
of time are recorded independently, may drift apart, and must be correlated after
the fact by software. This architectural decision drives the entire design of
`irig106-time`.

### The Core Problem

```
┌──────────────────────┐     ┌──────────────────────┐
│   Internal RTC       │     │  External Clock       │
│   (10 MHz counter)   │     │  (IRIG-B / GPS / UTC) │
│                      │     │                        │
│   - Free-running     │     │  - Absolute wall time  │
│   - Monotonic*       │     │  - May jump on lock    │
│   - No calendar      │     │  - May have multiple   │
│   - Wraps at 48 bits │     │    sources             │
└──────────┬───────────┘     └──────────┬─────────────┘
           │                            │
           ▼                            ▼
    ┌──────────────────────────────────────────┐
    │        Time Data Format 1 Packet         │
    │     (pairs RTC value with clock time)    │
    │                                          │
    │    RTC = 1,000,000                       │
    │    Absolute = Day 100, 12:30:25.340      │
    └──────────────────────────────────────────┘
           │
           ▼
    ┌──────────────────────────────────────────┐
    │         Correlation Engine               │
    │                                          │
    │  Data packet has RTC = 1,150,000         │
    │  Delta = 150,000 ticks × 100 ns = 15 ms │
    │  → Day 100, 12:30:25.355                 │
    └──────────────────────────────────────────┘
```

### Why This Crate Exists Separately

See [WHY_SEPARATE_REPO.md](WHY_SEPARATE_REPO.md) for the full rationale.

---

## 2. Module Architecture

```
                    ┌──────────────┐
                    │   lib.rs     │
                    │  (re-exports)│
                    └──────┬───────┘
           ┌───────────────┼───────────────┐
           │               │               │
     ┌─────▼─────┐  ┌─────▼──────┐  ┌─────▼────────┐
     │   error    │  │    rtc     │  │  absolute    │
     │            │  │            │  │              │
     │ TimeError  │  │ Rtc (48b)  │  │ AbsoluteTime │
     │ Result<T>  │  │            │  │ Ch4BinaryTime│
     │            │  │            │  │ Ieee1588Time │
     └────────────┘  └────────────┘  │ Ertc (64b)  │
                                     └──────────────┘
           │               │               │
     ┌─────▼─────┐  ┌─────▼──────┐  ┌─────▼────────┐
     │   csdw    │  │    bcd     │  │  secondary   │
     │           │  │            │  │              │
     │ TimeF1Csdw│  │DayFmtTime  │  │ SecHdrTime   │
     │ TimeSource│  │DmyFmtTime  │  │ checksum val │
     │ TimeFormat│  │            │  │              │
     │ DateFormat│  │            │  │              │
     └───────────┘  └────────────┘  └──────────────┘
                          │               │
                    ┌─────▼──────┐  ┌─────▼────────┐
                    │intra_packet│  │ correlation  │
                    │            │  │              │
                    │IntraPacket │  │TimeCorrelator│
                    │  Time      │  │ ReferencePoint│
                    │            │  │ TimeJump     │
                    └────────────┘  └──────────────┘
```

---

## 3. Data Flow Through the Crate

### 3.1 Time Packet Processing

```
 Raw Ch10 File
     │
     ▼
 ┌────────────────────────┐
 │ Packet Header (24 bytes)│
 │ ┌────────────────────┐  │
 │ │ Sync  │ ChanID     │  │
 │ │ PktLen│ DataLen     │  │
 │ │ Flags │ DataType    │  │   DataType = 0x11?
 │ │ RTC[6]│ Checksum    │──┼──────────────────┐
 │ └────────────────────┘  │                   │
 └────────────────────────┘                   ▼
                                     ┌─────────────────┐
                                     │ Parse CSDW (4B)  │
                                     │ → TimeF1Csdw     │
                                     └────────┬────────┘
                                              │
                               ┌──────────────┴──────────────┐
                               ▼                              ▼
                    date_format = DOY              date_format = DMY
                               │                              │
                    ┌──────────▼──────────┐       ┌──────────▼──────────┐
                    │ Parse BCD Day (8B)  │       │ Parse BCD DMY (10B) │
                    │ → DayFormatTime     │       │ → DmyFormatTime     │
                    └──────────┬──────────┘       └──────────┬──────────┘
                               │                              │
                               └──────────────┬───────────────┘
                                              ▼
                                    ┌──────────────────┐
                                    │   to_absolute()  │
                                    │ → AbsoluteTime   │
                                    └────────┬─────────┘
                                             │
                                             ▼
                                  ┌────────────────────┐
                                  │ correlator         │
                                  │  .add_reference(   │
                                  │    channel_id,     │
                                  │    rtc,            │
                                  │    absolute_time)  │
                                  └────────────────────┘
```

### 3.2 Data Packet Timestamp Resolution

```
 Any Data Packet (1553, PCM, etc.)
     │
     ▼
 ┌─────────────────────┐
 │ Extract RTC from     │
 │ packet header[16..22]│
 │ → Rtc::from_le_bytes │
 └──────────┬──────────┘
            │
            ▼
 ┌─────────────────────────────┐
 │ correlator.correlate(       │
 │   rtc,                      │
 │   Some(preferred_channel))  │
 │                             │
 │ 1. Find nearest ReferencePoint    │
 │ 2. delta = ref.rtc → target_rtc   │
 │ 3. abs_time = ref.time + delta_ns │
 └──────────┬──────────────────┘
            │
            ▼
     AbsoluteTime
     Day 100, 12:30:25.355_000_000
```

---

## 4. Packet Header Time Fields

The 24-byte primary header has a 6-byte RTC at bytes [16..22]:

```
 Byte offset:  0  1  2  3  4  5  6  7  8  9 10 11
              ├──────┤──────┤────────────┤────────────┤
              │ Sync │ChID  │ PktLength  │ DataLength │
              └──────┴──────┴────────────┴────────────┘

 Byte offset: 12 13 14 15 16 17 18 19 20 21 22 23
              ├──┤──┤──┤──┤──────────────────────┤──────┤
              │DV│Sq│Fl│DT│  RTC (48 bits LE)    │ Chk  │
              └──┴──┴──┴──┴──────────────────────┴──────┘
                          ▲
                 Packet Flags byte:
                 [1:0] = Checksum type
                 [2]   = Secondary header present
                 [3:2] = Time format (if sec hdr)
                 [6]   = Data overflow
                 [7]   = RTC sync error
```

---

## 5. Time Format Wire Layouts

### 5.1 BCD Day-of-Year Format (8 bytes)

```
 Word 0 (bits):  15  14 13 12  11 10  9  8   7  6  5  4   3  2  1  0
                ┌───┬────────┬────────────┬────────────┬────────────┐
                │rsv│ TSn    │    Sn      │   Hmn      │    Tmn     │
                │   │tens sec│ units sec  │hundreds ms │ tens ms    │
                └───┴────────┴────────────┴────────────┴────────────┘

 Word 1:         15 14 13 12  11 10  9  8   7   6  5  4   3  2  1  0
                ┌──────┬─────┬────────────┬───┬─────────┬────────────┐
                │ rsv  │THn  │    Hn      │rsv│  TMn    │    Mn      │
                │      │t.hr │ units hr   │   │tens min │ units min  │
                └──────┴─────┴────────────┴───┴─────────┴────────────┘

 Word 2:         15 14 13 12 11 10  9  8   7  6  5  4   3  2  1  0
                ┌──────────────────┬──────┬────────────┬────────────┐
                │     reserved     │ HDn  │   TDn      │    Dn      │
                │                  │h.day │ tens day   │ units day  │
                └──────────────────┴──────┴────────────┴────────────┘

 Word 3:         (reserved — all zeros)
```

### 5.2 Intra-Packet Time Stamp Formats (8 bytes each)

```
 48-bit RTC:    ┌────────────────────────────────────────────┬──────────┐
                │         RTC (48 bits, little-endian)       │ reserved │
                │   byte 0   byte 1   byte 2 ... byte 5     │ byte 6-7 │
                └────────────────────────────────────────────┴──────────┘

 IEEE-1588:     ┌────────────────────────┬────────────────────────┐
                │  Nanoseconds (32-bit)  │    Seconds (32-bit)    │
                │      little-endian     │     little-endian      │
                └────────────────────────┴────────────────────────┘

 Ch4 Binary:    ┌──────────┬──────────────┬──────────────┬──────────┐
                │ unused   │ High Order   │ Low Order    │ µseconds │
                │  2 bytes │   2 bytes    │  2 bytes     │ 2 bytes  │
                └──────────┴──────────────┴──────────────┴──────────┘

 64-bit ERTC:   ┌────────────────────────────────────────────────────┐
                │           ERTC (64 bits, little-endian)            │
                │     byte 0   byte 1   byte 2 ... byte 7           │
                └────────────────────────────────────────────────────┘
```

---

## 6. Ecosystem Crate Relationships

```
                    ┌────────────────┐
                    │  irig106-types │  (foundational types)
                    │                │
                    │ Rtc, Ertc      │
                    │ Ch4BinaryTime  │
                    │ Ieee1588Time   │
                    │ TimeSource...  │
                    └───────┬────────┘
                            │ depends on
              ┌─────────────┼──────────────┬──────────────┐
              │             │              │              │
    ┌─────────▼───┐  ┌─────▼──────┐ ┌─────▼─────┐ ┌─────▼──────┐
    │ irig106-core│  │irig106-time│ │irig106-   │ │irig106-    │
    │             │  │            │ │decode     │ │write       │
    │ Pkt header  │  │ BCD decode │ │ Payload   │ │ Serializer │
    │ traversal   │  │ Correlation│ │ semantics │ │            │
    └─────────────┘  └────────────┘ └───────────┘ └────────────┘
              │             │              │
              └─────────────┼──────────────┘
                            │
                    ┌───────▼────────┐
                    │ irig106-ch10-  │
                    │ reader / cli   │
                    │                │
                    │ High-level     │
                    │ file analysis  │
                    └────────────────┘
```

---

## 7. Requirements Traceability Chain

```
 IRIG 106-17 Chapter 10          RCC 123-20 Programmer's Handbook
 ──────────────────────          ─────────────────────────────────
 §10.6.1.1 (RTC)                 §5.3, §6.6 (Time Interpretation)
 §10.6.1.4 (Ch4 BWT)             §5.4 (Secondary Header)
 §10.6.1.5 (IEEE-1588)           §5.5.3 (Time Data Format 1)
 §10.6.5.2 (Time F1)             Figures 5-3 through 5-14
         │                               │
         ▼                               ▼
 ┌─────────────────────────────────────────────┐
 │  L1 Requirements (37)                       │
 │  "The crate shall..."                       │
 │  Directly mapped to standard sections       │
 └─────────────────────┬───────────────────────┘
                       │
                       ▼
 ┌─────────────────────────────────────────────┐
 │  L2 Requirements (78)                       │
 │  "FunctionName shall..."                    │
 │  Testable functional behaviors              │
 └─────────────────────┬───────────────────────┘
                       │
                       ▼
 ┌─────────────────────────────────────────────┐
 │  L3 Requirements (65)                       │
 │  Struct layouts, algorithms, constants      │
 │  Maps directly to source files              │
 └─────────────────────┬───────────────────────┘
                       │
           ┌───────────┴───────────┐
           ▼                       ▼
 ┌─────────────────┐     ┌─────────────────┐
 │  Source Code     │     │  Tests (114)    │
 │  8 modules       │     │  104 unit       │
 │  ~1500 lines     │     │  10 integration │
 └─────────────────┘     └─────────────────┘
```
