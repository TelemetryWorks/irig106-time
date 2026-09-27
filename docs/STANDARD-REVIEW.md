# `irig106-time` checked against IRIG 106 (2026-09-26)

> **Status: findings for the owner's review.** Written while preparing this
> crate's contract document (the ecosystem coverage plan,
> `irig106-docs/src/coverage.md`, puts `irig106-time` next). Every finding
> quotes the archived standard (`TelemetryWorks/rcc-106-standards`;
> 106-24R1 unless another edition is named) and names the code it concerns.
> Nothing in the code has been changed. The owner decided to rebuild the
> crate (2026-09-27, `docs/adr/0001`); each finding becomes a failing test
> before the rebuilt code fixes it (`docs/adr/0016`).

## Confirmed disagreements with the standard

### T-1 Intra-packet time format is taken from the wrong packet-flag bits

**Code:** `src/intra_packet.rs:40-51`, `IntraPacketTimeFormat::from_packet_flags`
— "If bit 2 (secondary header present) is clear, the format is always RTC.
If bit 2 is set, bits [3:2] select the format". L1-IPT-001 repeats "Packet
Flag bit 2 = 0".

**Standard** (Chapter 11 §11.2.1.1 g, packet flags): "Bit 7: Indicates the
presence or absence of the packet secondary header"; "Bit 6: Indicates the
IPTS time source. 0 = Packet header 48-bit RTC. 1 = Packet secondary header
time (bit 7 must be 1)"; "Bits 3-2: Indicate the packet secondary header
time format. 00 = Chapter 4 binary weighted 48-bit time format … 01 = IEEE
1588 time format … 10 = 64-bit binary extended relative time counter (ERTC)
… 11 = Reserved".

**Effect:** with bit 2 set, bits 3–2 can only be `01` or `11`, so the
Chapter 4 and ERTC formats are never selected, and the RTC is chosen or
rejected by the wrong bit.

**Change:** the intra-packet time is the header RTC when bit 6 is 0; when bit
6 is 1 (with bit 7 set), its format is bits 3–2. Tests for every
combination, including bit 6 set without bit 7.

### T-2 The Format 2 (network time) CSDW is read from the wrong bits

**Code:** `src/network_time.rs` — `TimeF2Csdw::time_protocol` reads bits
3–0 as 0 = NTP, 1 = PTP; `validate_reserved` treats bits 31–4 as reserved.

**Standard** (Chapter 11 §11.2.3.3, Figure 11-15, the same in 106-17,
106-19, and 106-24R1): "Reserved. Bits 31-8 are reserved"; "Network Time
Format (NTF). Bits 7-4 … 0x0 = Network Time Protocol Version 3 … 0x1 =
IEEE-1588-2002 … 0x2 = IEEE-1588-2008"; "Time Status (TS). Bits 3-0 … 0x0 =
Time Not Valid … 0x1 = Time Valid".

**Effect:** the protocol is read from the status field; IEEE 1588-2002 and
-2008 are not told apart; the validity of the time is not reported; a valid
packet's format bits are reported as reserved bits set.

**Change:** read NTF from bits 7–4 and TS from bits 3–0; reserved bits
31–8; expose the time status and never use a time marked "not valid" as a
reference without saying so.

### T-3 The PTP time body is read as 10 bytes; the standard gives 8

**Code:** `src/network_time.rs:217` onward — `PtpTime` holds "Seconds since
1970-01-01 00:00:00 TAI (48 bits)" and 32-bit nanoseconds; L1-PTP-001 says a
10-byte message.

**Standard** (Figure 11-17 in 106-24R1, Figure 11-16 in 106-17; the same in
106-19): two 32-bit words, "Time Unsigned Seconds" and "Time Unsigned
Nanoseconds".

**Change:** read 32-bit seconds and 32-bit nanoseconds (8 bytes), in line
with NTP's 8-byte body.

### T-4 Format 2 did not arrive in 106-22

**Code and docs:** `version.rs` and `supports_format_2()` treat Format 2 as
106-22 and later.

**Standard:** Chapter 11 of 106-17 already lists "0x12 Time Data, Format 2
Network Time" with data type version `0x08` (106-17) and defines the same
CSDW and bodies; 106-13 and 106-15 do not mention network time.

**Change:** Format 2 from 106-17.

### T-5 ERTC is converted as if it counted 100 ns

**Code:** `irig106-types/src/rtc.rs:184-185`, `Ertc::to_nanos` multiplies by
`NANOS_PER_TICK` (100), the RTC's tick.

**Standard** (§11.2.1.1 g): "64-bit binary extended relative time counter
(ERTC) with 1-nanosecond resolution. The counter shall be derived from a
free-running 1-gigahertz (GHz) clock … (RTC = ERTC/100)".

**Effect:** every ERTC time is 100 times too large.

**Change:** one ERTC tick is 1 ns (in `irig106-types`).

### T-6 Time source 3 is not "GPS" in any archived edition

**Code:** `irig106-types` `TimeSource::Gps = 3`; `csdw.rs:292-305` and
`version.rs:218-229` describe 3 as GPS from 106-05 (and "None" in 106-04).

**Standard:** the Format 1 CSDW's "Time Source (SRC). Bits 3-0" lists "0x0 =
Internal … 0x1 = External … 0x2 = Internal from RMM … 0x3 – 0xE = Reserved …
0xF = None" in 106-24R1, and the same values with 3 reserved in 106-05,
106-07, and 106-13 (Chapter 10 in those editions). 106-04's Chapter 10 is
not in the archive, so its value 3 cannot be checked.

**Change:** treat 3 as reserved unless a source shows otherwise.

### T-7 Fields of the Format 1 CSDW are missing

**Code:** `TimeF1Csdw` reads SRC (3–0), FMT (7–4), leap year (8), and date
format (9); `TimeFormat` has no value for `0xF`.

**Standard** (§11.2.3.2, Figure 11-12): "IRIG Time Source (ITS). Bits 15-12
provide dynamic information regarding the source of time for an internal
IRIG time code generator (TCG)" — freewheeling, locked to external IRIG,
GPS, NTP, PTP, or embedded time; bits 11–10 reserved; FMT "0xF = NONE (time
packet payload invalid)"; bits 31–16 reserved.

**Change:** add ITS, `NONE` (and treat such a payload as invalid), and
reserved-bit checks for bits 31–16 and 11–10.

### T-8 Recording event numbers do not have fixed meanings

**Code:** `src/recording_event.rs` — `RecordingEventType{Started=1,
Stopped=2, Overrun=3, IndexPoint(4..0xF), Reserved}`.

**Standard** (§11.2.7.3, Figure 11-38): "Event Number. Bits 11-0 represent an
unsigned binary that identifies 4096 individual events types defined in the
corresponding setup record recording event number. The event number shall
begin at 0x0 for the first event type defined in the setup record"; bits
27–12 are the event count and bit 28 the event occurrence state (between
.RECORD and .STOP, or not).

**Effect:** an event's meaning comes from the setup record (TMATS), not
from its number.

**Change:** decode the entry as the standard lays it out and take each
event's meaning from the TMATS description (`irig106-tmats`); decide
whether recording events belong in this crate at all.

### T-9 The version code `0x0F` is read as 106-23

**Code:** `version.rs:181-195` maps RCCVER `0x0E` to 106-22 and `0x0F` to
106-23.

**Standard** (§11.2.7.2, Figure 11-34): "0x0E = RCC 106-22"; "0x0F through
0xFF = Reserved". `0x0E` therefore means "106-22 or later"; there is no code
for 106-20, 106-23, or 106-24. (Recorded earlier as `irig106-tmats` ROADMAP
X1; the packet header's data type version is a different list, where
`0x0A` is 106-22.)

**Change:** `0x0F` and above are unknown; `0x0E` reads as "106-22 or later"
(with the shared mapping in `irig106-types`).

### T-12 Time packet bodies are read as longer than the standard defines

**Code:** `src/bcd.rs` — `DayFormatTime::from_le_bytes` requires 8 bytes
(four 16-bit words, "w3 is reserved") and `DmyFormatTime::from_le_bytes`
requires 10 (five words, "w4 is reserved"); L1-BCD-001 says an 8-byte
message.

**Standard:** Figure 11-13 (day format) shows three 16-bit words and Figure
11-14 (day, month, and year) four — the same in 106-13 (Figure 10-20),
106-17 (Figure 11-12), and 106-24R1. The body is therefore 6 or 8 bytes,
and the packet's "Data Length … does not include packet trailer filler"
(§11.2.1.1 d).

**Effect:** a caller that slices the body by Data Length — as it must, to
exclude filler — passes 6 bytes and gets `BufferTooShort`; one that passes
more reads filler as a "reserved" word. Found while writing the worked
example (`docs/TIME-IN-CHAPTER-10.md` section 7), where Data Length is 10:
the data word (4) and the body (6).

**Change:** read 6 bytes for the day format and 8 for day, month, and year,
and never read past Data Length.

### T-13 The correlation requirements quote the wrong handbook edition

*Found 2026-09-27, while rewriting L1.*

**Docs:** `docs/L1_Requirements.md` (prototype) L1-COR-001 to 004 cite "RCC
123-20 §6.6" and quote "It is better to use the clock and relative time
values from a time packet that occurs near the current data packet", "there
may be separate time channels for time derived from IRIG B, GPS, and an
internal battery backed up clock", and "there is a jump in input clock time
during a recording, such as when GPS locks for the first time".

**Handbook:** RCC 123-20 has no §6.6; its "Time Interpretation" is §5.6
(page 5-44), and it keeps only "When multiple time channels are available,
it is incumbent on the programmer or data analyst to determine and select
the best source of time for a particular data set" and the worked
correlation example. The quoted sentences are in **RCC 123-09 §6.6** (March
2009, page 6-36): "It is better to use the clock and relative time values
from a time packet that occurs near the current data packet as the data file
is decoded since there is some drift in the RTC during a recording session.
It also may be the case that there is a jump in input clock time during a
recording, such as when GPS locks for the first time, or when an IRIG time
source is reprogrammed" — and "It is usually most correct to select one time
channel only and to use this channel exclusively to correlate RTC time to
absolute clock time for all data packet types". RCC 123-16 §5.6 matches
123-20.

**Change:** cite 123-09 §6.6 for those sentences (done in the rewritten L1).
The handbook is guidance, not the standard; the requirements rest on Chapter
11 and cite the handbook as support.

### T-16 A leap second cannot be read

*Found 2026-09-27, at the completeness check of step 1.*

**Code:** `src/absolute.rs:199` — `if seconds > 59` returns `OutOfRange`, so
every time with second 60 is rejected; the prototype's L1-ERR-003 asks for
it ("seconds > 59").

**Standard:** RCC 200-16, which Chapter 11 cites for IRIG time ("The 10-MHz
RTC shall be captured … IAW IRIG 200", §11.2.3.2): "The SBS TOD code reads 0
seconds at 2400 each day excluding leap second days when a second may be
added or subtracted" (§3.6); "If required, time changes are made on December
31 and on June 30 at 2400 hours" (Appendix A.2). A UTC time of 23:59:60 on
those days is valid, and a Format 1 body can carry it (tens of seconds are
three bits, Figure 11-13).

**Change:** accept second 60 at 23:59:60 on a day that can be 30 June or 31
December (days 181, 182, 365, 366) in the UTC formats, and report it as a
leap second; reject it elsewhere. A reading for the register (RDG-010).

### T-17 The time scale of a time packet is ignored

*Found 2026-09-27, at the completeness check of step 1.*

**Code:** the Format 1 time format is decoded (`src/csdw.rs:79`) but used
nowhere else: references from every format are correlated and returned
alike, and `LeapSecondTable::offset_for_f1` treats "IRIG-B, GPS, internal
clock" (`src/network_time.rs:568`) the same.

**Standard:** FMT distinguishes "0x4 = Universal Coordinated Time (UTC) time
from GPS" from "0x5 = Native GPS Time" (§11.2.3.2 a), and RCC 200-16 says
"GPS time does not add or subtract leap seconds, and as of this writing, GPS
time is 16 seconds ahead of UTC" (Appendix A.2, written when TAI − UTC was
35 s, so TAI − GPS = 19 s). Native GPS time is therefore 18 s ahead of UTC
since 2017. NTP is UTC and PTP is TAI (§11.2.3.3).

**Effect:** times from a native-GPS time channel are presented as if they
were UTC, 18 s late; references of different scales could be combined.

**Change:** every reference carries its time scale — UTC, TAI, GPS, or
unknown (the recorder's own clock, FMT `0x3`) — and conversions go through
the leap-second table. IRIG-A, B, and G are UTC by assumption (RCC 200-16
§1: ranges "maintain Coordinated Universal Time (UTC) referenced to the
United States Naval Observatory (USNO) Master Clock"), labelled as assumed
and changeable by the policy (RDG-009).

## Readings to settle

### T-14 The IRIG time source field has no version of its own

*Found 2026-09-27, while rewriting L1.*

**Standard:** the Format 1 data word's "IRIG Time Source (ITS). Bits 15-12"
appears in Chapter 11 of 106-17 and every later edition; 106-05, 106-07,
106-13, and 106-15 (Chapter 10) say "Reserved. Bits 31-12 are reserved".
Yet Table 11-4 gives Time Data, Format 1 the "Current Data Type Version"
`0x06` (106-13) in both 106-17 and 106-24R1: the packet header cannot say
whether bits 15-12 carry ITS. Because "All reserved bit fields in packet
headers or CSDWs shall be set to zero (0x0)" (§11.2.1 f), a recording made
under 106-15 carries `0000`, which 106-17 defines as "IRIG TCG freewheeling
(no or loss of time source)".

**Reading (proposed):** take ITS as defined only when the setup record
declares 106-17 or later (RCCVER `0x0C` or above, Figure 11-34); otherwise
report it as "not reported", never as "freewheeling". ITS applies "when FMT
is IRIG-A, B, or G" (§11.2.3.2) in any case. An entry for the reading
register (ADR-0012): RDG-007.

### T-15 The secondary header checksum: bytes or 16-bit words?

*Found 2026-09-27, while rewriting L1.*

**Code:** `src/secondary.rs:28-45`, `validate_secondary_checksum` — "the
16-bit sum of the first 5 little-endian u16 words (bytes [0..10])".

**Standard:** "a 16-bit arithmetic sum of all secondary header bytes
excluding the secondary header checksum word" (§11.2.1.2 c; the same words
in 106-05, 106-07, 106-13, 106-15, and 106-17). The packet header's checksum
is worded differently: "a 16-bit arithmetic sum of all 16-bit words in the
header" (§11.2.1.1 j). **Handbook:** RCC 123-20 Appendix A-2
(`irig106ch10.c`, `uCalcSecHeaderChecksum`) adds the ten bytes one at a
time into a 16-bit sum — with the comment "MAKE THIS 16 BIT UNSIGNEDS LIKE
ABOVE", so its author was unsure too.

**Effect:** if the standard means bytes, the crate rejects valid secondary
headers and accepts invalid ones.

**Reading (proposed, suspect):** sum bytes, as the wording and the handbook
do; confirm against real recordings (locally, never in CI) before L2, and
report which reading a recording's headers satisfy. Register entry RDG-008.

## To confirm on the page image

### T-10 Chapter 4 binary time

**Code:** `src/absolute.rs:610-630` takes the low 17 bits of the combined
high and low words as tens of milliseconds and the bits above as the day of
year; `irig106-types` `Ch4BinaryTime::from_secondary_bytes` reads the 8
bytes as unused, high, low, microseconds. (Already open as GAP-03.)

**Standard:** Chapter 4 §4.7 — "The microsecond time word shall have a
resolution of 1 microsecond … the maximum value of the counter is 9999";
"The low order time word shall have a resolution of 10 milliseconds";
"The high order time word shall have a resolution of 655.36 seconds when
binary weighted" — which reads as one count of 10 ms across both words
(17 bits of 10 ms reach only about 21.8 minutes). Chapter 11 Figure 11-4
places "Micro-Seconds Word" and "Reserved" in the first long word and
"High Order Time Word" and "Low Order Time Word" in the second. Figure 4-4,
which shows the bit layout, is an image; both need checking on the page
before the code changes.

### T-11 A caption error in the standard

Chapter 11 of 106-24R1 captions the Format 2 packet structure "Table 11-17.
General Time Data Packet, Format 1"; it describes Format 2.

## Repository matters

- **CI will not find `irig106-types`.** `Cargo.toml` uses `path =
  "../irig106-types"`, and CI checks out only this repository; until
  `irig106-types` 0.1.0 is published and the path is dropped, CI fails
  after commit `a755a5c` (ROADMAP P6-01; `irig106-tmats` ROADMAP X7).
- **Stale citations.** The crate documentation cites "IRIG 106-17 Chapters
  10/11 and RCC 123-20"; the current edition is 106-24R1.
- **Version and MSRV.** `Cargo.toml` says 0.7.0 while the changelog and
  roadmap say 0.8.0 (ADR-0017 proposes 0.8.0 for the rebuild). *Resolved
  2026-09-27:* `rust-version` was 1.60 while CI checked 1.78; the ecosystem
  now uses edition 2024 and Rust 1.85 everywhere, and CI checks 1.85
  (ADR-0003).
- **Requirements.** L3-CSDW-008 omits `Gps`; L1/L2-API-003 say "zero
  required dependencies" although `irig106-types` is now required; several
  L2 signatures are stale (ROADMAP P7-03).
- **RCC 200** ("IRIG Serial Time Code Formats"), which Chapter 11 cites for
  IRIG time formats. *Resolved 2026-09-27:* archived as the release
  `rcc-200-16` of `TelemetryWorks/rcc-106-standards`, both byte-different
  copies (TRMC and irig106.org).
