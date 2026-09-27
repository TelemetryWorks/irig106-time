# What time processing gives Chapter 10 processing

> **Status: design draft, written one section at a time for the owner's
> review.** The contract document for `irig106-time`, in the shape of
> `irig106-tmats`'s `docs/TMATS-IN-CHAPTER-10.md`, as the ecosystem coverage
> map requires (`irig106-docs`, `src/coverage.md`: `irig106-time` next).
> Every rule and quotation is taken from the archived standard
> (`TelemetryWorks/rcc-106-standards`; baseline 106-24R1) and cited. Where
> the crate disagrees with the standard today, `docs/STANDARD-REVIEW.md`
> records it (T-1 to T-11); this document describes what the crate should
> give, and section 8 plans the changes.

## Contents

| Section | Status |
|---------|--------|
| 1. What `irig106-time` is for | **draft for review** |
| 2. Where time sits in Chapter 10 processing | to be written |
| 3. What the crate answers, question by question | to be written |
| 4. Contracts per consumer | to be written |
| 5. Time over a recording | to be written |
| 6. When time is missing, wrong, or disagrees | to be written |
| 7. A worked example | to be written |
| 8. What changes as a result | to be written |

---

## 1. What `irig106-time` is for

### 1.1 The problem: packets carry relative time, not absolute time

Every packet header carries a **relative time counter** (RTC): "a
free-running 10-MHz binary counter represented by 48 bits that are common
to all data channels. The counter shall be derived from a 10-MHz internal
crystal oscillator and shall remain free-running during each session (e.g.,
recording)" (Chapter 11 §11.2.1.1 i, 106-24R1). It orders packets across
channels to 100 ns, but it says nothing about the date or the time of day,
and after 2^48 ticks — about 325.8 days — it wraps.

Absolute time arrives in other places:

- **Time packets.** "Time is treated like another data channel"
  (§11.2.3.2). A Format 1 time packet (data type `0x11`, "IRIG/GPS/RTC")
  carries a time in binary-coded decimal — day of year, or day, month, and
  year — with its source and format in the channel-specific data word, and
  "will be generated at a minimum frequency of 1 hertz". A Format 2 packet
  (`0x12`, "Network Time") carries NTP time ("referenced in UTC time with an
  epoch of January 1, 1900. The NTP time includes leap seconds") or PTP time
  ("referenced in International Atomic Time with an epoch of January 1, 1970.
  The PTP time does not include leap seconds") (§11.2.3.3). Each time packet's
  header RTC pairs that absolute time with the counter.
- **A time packet comes first.** "A time data packet shall be the first
  dynamic data packet at the start of each session. Only static
  Computer-Generated Data, Format 1 packets may precede the first time data
  packet" (§11.2.3.2).
- **Some packets carry their own time.** A packet with a secondary header
  (packet flags bit 7) carries a 64-bit time in Chapter 4 binary, IEEE 1588,
  or extended RTC (ERTC, "1-nanosecond resolution … derived from a
  free-running 1-gigahertz (GHz) clock", with "RTC = ERTC/100") format; its
  intra-packet time stamps are the RTC, or that format when packet flags bit
  6 is set (§11.2.1.1 g, §11.2.1.2, §11.2.1.3 b).
- **TMATS says where time is.** The setup record declares which channels
  carry time (`R-x\CDT-n` = TIMEIN), their packet format (`R-x\TTF-n`), time
  format (`R-x\TFMT-n`), and time source (`R-x\TSRC-n`), each channel's
  secondary-header time format (`R-x\SHTF-n`), and the measurements that are
  time words inside data (`C-d\DCT` = PTM, NTM, BTM) — `irig106-tmats`
  `docs/TMATS-IN-CHAPTER-10.md` section 3.8.

![From a packet's relative time to absolute time](diagrams/rtc-to-absolute.svg)

*Correlation.* Time packets pair counter values with absolute time at least
once a second; a data packet's counter value becomes absolute time from the
nearest time packet — its time plus the counter difference at 100 ns per
tick. A packet with a secondary header may carry its own time instead. TMATS
says which channel carries time and in what format.

### 1.2 What `irig106-time` does

**`irig106-time` is the one place in the ecosystem where time is
understood**: it turns counters, time packets, and packet time stamps into
absolute time, and says how good that time is. It is a `no_std` library
(`src/lib.rs`) that works on bytes and values its callers supply.

| Area | What the crate does | Where today |
|------|---------------------|-------------|
| Relative time | The 48-bit RTC: from its 6 bytes, wrap-safe differences, nanoseconds | `Rtc` (from `irig106-types`) |
| Time packets | Format 1: the channel-specific data word and the day-of-year and day-month-year bodies, read and written; Format 2: the channel-specific data word, NTP and PTP bodies | `csdw`, `bcd`, `network_time` (T-2, T-3, T-7) |
| Packet time | The secondary header (with its checksum) and intra-packet time stamps, in Chapter 4 binary, IEEE 1588, and ERTC formats | `secondary`, `intra_packet`, `absolute` (T-1, T-5, T-10) |
| Correlation | Absolute time for any RTC from reference points — per channel, nearest reference, time jumps, RTC resets, drift, out-of-order packets — and a streaming form with bounded memory | `correlation`, `streaming` |
| Quality | A measure of how trustworthy the correlation is | `quality` |
| Leap seconds | The TAI–UTC offset, from a built-in table the caller can extend | `network_time` |
| Edition | The edition declared by the setup record's CSDW | `version` (T-9; moving to `irig106-types`) |
| Recording events | Recording event types | `recording_event` (T-8: whether they belong here is open) |

### 1.3 What it deliberately does not do

| Not done here | Done by | Why |
|---------------|---------|-----|
| Open files; walk packets | the caller — `irig106-core` when it exists; this repository's own `ch10time` CLI has a reader | The library works on bytes it is given |
| Read TMATS | `irig106-tmats`, which hands over the time attributes as plain data | One place understands the setup record (`irig106-tmats` ADR-0030) |
| Extract time words from PCM or bus data | `irig106-decode`, which finds the words; turning a Chapter 4 time word into absolute time is this crate's (section 3) | Data decoding belongs to the decoder |
| Write packets | `irig106-write`; this crate encodes time packet bodies | The library does not assemble packets |
| Present time | the tools | — |

### 1.4 Who relies on it

| Consumer | What it needs from time processing |
|----------|-----------------------------------|
| `irig106-decode` | Absolute time for every packet and, through intra-packet time stamps, for every message or sample |
| `irig106-studio` | A time axis: absolute time for anything shown |
| `irig106-index` | Seeking by time; the time span of each part of a recording |
| `irig106-ch10-reader`, `irig106-cli` | A recording's start, end, and duration; time gaps and jumps |
| `irig106-write` | Time packet bodies and channel-specific data words to write |
| `irig106-tmats` | Nothing at run time; the configuration timeline relates setup records to relative time on its own (`irig106-tmats` L1-CH10-008) |

Section 4 turns each row into a contract.

### 1.5 Promises the crate makes today

From its own documents and code:

1. **No panics on bad input**: "The crate shall return typed errors for all
   fallible operations; it shall not panic on invalid input data"
   (L1-ERR-001).
2. **Nanosecond precision**: "at least nanosecond precision to support all
   Chapter 10 time formats" (L1-ABS-001).
3. **`no_std`**, with `std`, `serde`, and `chrono` as options; every public
   item documented (`#![deny(missing_docs)]`).
4. **WebAssembly**: CI builds the crate for `wasm32` with and without
   `serde`.
5. **One dependency**: `irig106-types`, for the shared types.

Section 8 proposes what the ecosystem's other crates promise and this one
does not yet state — for example, that every absolute time says which
reference and which source it came from.

### 1.6 Questions for the next sections

- **Several time sources.** A recording may carry more than one time
  channel, and a time packet's source can change while recording ("If the
  time source is external (0x1) and lock on the external source is lost then
  the time source shall indicate Internal (0x0)", §11.2.3.2). Which reference
  governs a packet, and how the caller chooses (L1-COR-003), is for sections
  3 and 5.
- **TMATS and the packets say the same things differently.** TMATS writes
  the time format and source as letters (`R-x\TFMT-n`: A, B, G, I, N, U, X,
  0, 1, 2; `R-x\TSRC-n`: I, E, R, X); the time packet's data word carries
  numbers (FMT `0x0` IRIG-B … `0xF` NONE; SRC `0x0` internal … `0xF` none;
  NTF for network time). And TMATS names secondary-header time format 0
  "Chapter 4 BCD" where the packet flags define `00` as "Chapter 4 binary
  weighted 48-bit time format" (Chapter 4 §4.7 allows both weightings for
  time words in a PCM stream). Section 6 reconciles them.
- **Recording events.** Their meaning comes from the setup record's event
  definitions (T-8); whether they belong in this crate, in `irig106-core`, or
  with `irig106-tmats` is open.
- **Time inside data.** PCM, network, and 1553 time words (Chapter 4 §4.7;
  `C-d\PTM`, `NTM`, `BTM`) are found by `irig106-decode`; section 3 decides
  what this crate offers to turn them into absolute time.
- **Leap seconds.** NTP includes them and PTP does not; the table must stay
  current and say how current it is.
- **The findings of `docs/STANDARD-REVIEW.md`** (T-1 to T-11), the stale
  citations to Chapter 10 section numbers from before 106-17, and RCC 200's
  absence from the standards archive are planned in section 8.
