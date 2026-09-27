# irig106-time — Level 1 Requirements

> **Status: draft for the owner's review** (step 1 of the plan,
> `docs/TIME-IN-CHAPTER-10.md` section 8.2). Rewritten on 2026-09-27 for the
> rebuild (`docs/adr/0001`) from the contract document
> `docs/TIME-IN-CHAPTER-10.md`, the ADRs in `docs/adr/`, and the archived
> standard (`TelemetryWorks/rcc-106-standards`), citing **106-24R1**. It
> replaces the prototype's L1 (version 0.2.0, 2026-03-27, IRIG 106-17 baseline,
> kept at the tag `prototype-0`). L2 and L3 are rewritten after the review.
> Every ADR it rests on is accepted (2026-09-27).

## Purpose

This document holds the Level 1 (L1) SHALL-statement requirements for
`irig106-time`, the ecosystem's library for IRIG 106 time, and for
`irig106-time-cli`, its command-line crate (binary `irigtime`). L1 says
**what** the product must do; L2 decomposes each L1 into design decisions,
L3 into implementation obligations.

## Scope

"The library" binds `irig106-time`; "the CLI" binds `irig106-time-cli`; "the
project" binds both and their release. Out-of-scope items are listed under
**Non-requirements**. The release that delivers each requirement is set by
`docs/ROADMAP.md`, not here.

## Conventions

### Identifiers

Each requirement has a permanent identifier `L1-<CATEGORY>-<NNN>`. A retired
identifier is never reused. The prototype's identifiers whose meaning
survives keep their numbers (the statement is revised and re-cited); those
whose meaning was wrong are retired and replaced by new numbers — see
**Prototype requirements** at the end. Non-requirements use `NR-<NNN>`.

### Language and metadata

Every requirement uses SHALL. Each carries:

- **Statement** — the obligation;
- **Source** — the standard (section, figure, table, with its words where
  they matter), an ADR, the contract document, or a finding of
  `docs/STANDARD-REVIEW.md`;
- **Verification** — Test (T), Analysis (A), Inspection (I), or
  Demonstration (D).

Citations without an edition are 106-24R1: "§11.x" is Chapter 11, "Chapter
10 §10.x" and "Chapter 4 §4.x" are named. "Contract" is
`docs/TIME-IN-CHAPTER-10.md`. RCC 123 is the Chapter 10 Programmers'
Handbook, cited as support, never as the rule.

## Table of categories

| Code | Title |
|------|-------|
| `RTC` | The relative time counter |
| `ABS` | Absolute time values |
| `IPT` | Packet flags and intra-packet time stamps |
| `SEC` | The secondary header |
| `CSDW` | Format 1 channel-specific data word |
| `BCD` | Format 1 time bodies |
| `F2CSDW` | Format 2 channel-specific data word |
| `NTP` | Format 2 NTP time |
| `PTP` | Format 2 PTP time |
| `F2COR` | Format 2 references |
| `TAI` | Time scales and leap seconds |
| `WORD` | Time words inside data |
| `ENC` | Encoding |
| `EDN` | Editions |
| `DECL` | Time declarations from the setup record |
| `COR` | Correlation over a recording (the time timeline) |
| `ANS` | Answers and their basis |
| `POL` | The time policy |
| `FND` | Findings |
| `RDG` | Readings between TMATS and the packets |
| `STRM` | Streaming |
| `ERR` | Errors and robustness |
| `API` | Library form and portability |
| `CLI` | The command-line crate |
| `REL` | Versioning and release |
| `TST` | Verification |

---

## L1-RTC: The relative time counter

### L1-RTC-001

**Statement**: The library SHALL represent the relative time counter as a 48-bit unsigned count of 100-nanosecond ticks.

**Source**: "This is a free-running 10-MHz binary counter represented by 48 bits that are common to all data channels" (§11.2.1.1 i). Prototype identifier kept.

**Verification**: T

### L1-RTC-002

**Statement**: The library SHALL read the counter from the six little-endian bytes of the packet header (bytes 16 to 21) and from the low six bytes of an intra-packet time stamp in counter form.

**Source**: §11.2.1.1 i, Table 11-2; "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b). Prototype identifier kept, widened to time stamps.

**Verification**: T

### L1-RTC-003

**Statement**: The library SHALL compute the signed difference between two counter values of one session, correctly across the 48-bit wrap.

**Source**: 2^48 ticks at 10 MHz is about 325.8 days; the counter "shall remain free-running during each session" (§11.2.1.1 i). Prototype identifier kept.

**Verification**: T

### L1-RTC-004

**Statement**: The library SHALL convert a tick count to a duration with nanosecond precision.

**Source**: §11.2.1.1 i (10 MHz). Prototype identifier kept.

**Verification**: T

---

## L1-ABS: Absolute time values

### L1-ABS-001

**Statement**: The library SHALL represent absolute time to the nanosecond as a day of year and a time of day, with the year optional.

**Source**: IEEE 1588 and ERTC times resolve 1 ns (§11.2.1.1 g); the day format carries no year (Figure 11-13); contract 3.2. Prototype identifier kept.

**Verification**: T

### L1-ABS-002

**Statement**: The library SHALL read and write the 64-bit Chapter 4 binary weighted time of the secondary header and of intra-packet time stamps as Figure 11-4 lays it out, with its two least significant bytes zero.

**Source**: "00 = Chapter 4 binary weighted 48-bit time format. The two lsbs of the 64-bit packet secondary header time and IPTS shall be zero-filled" (§11.2.1.1 g); Figure 11-4; Chapter 4 §4.7. The word layout is to be confirmed on the page images of Figures 11-4 and 4-4 before L2 (T-10). Prototype identifier kept.

**Verification**: T

### L1-ABS-003

**Statement**: The library SHALL read and write IEEE 1588 time as 32-bit seconds in the most significant long word and 32-bit nanoseconds in the least significant long word, and SHALL reject nanoseconds of 1,000,000,000 or more.

**Source**: "The 32 bits indicating seconds shall be placed into the MSLW portion of the secondary header and the 32 bits indicating nanoseconds shall be placed into the LSLW portion" (§11.2.1.1 g); Figure 11-5. Prototype identifier kept.

**Verification**: T

### L1-ABS-004

**Statement**: The library SHALL read and write the extended relative time counter as a 64-bit count of 1-nanosecond ticks, least significant long word first.

**Source**: "64-bit binary extended relative time counter (ERTC) with 1-nanosecond resolution … (RTC = ERTC/100)" (§11.2.1.1 g); Figure 11-6 (LSLW, MSLW); T-5. Prototype identifier kept; the resolution is now stated.

**Verification**: T

---

## L1-IPT: Packet flags and intra-packet time stamps

### L1-IPT-005

**Statement**: The library SHALL read the time bits of the packet flags as §11.2.1.1 g defines them: bit 7, secondary header present; bit 6, the time stamp source; bits 3–2, the secondary header time format (`00` Chapter 4, `01` IEEE 1588, `10` ERTC, `11` reserved).

**Source**: "Bit 7: Indicates the presence or absence of the packet secondary header"; "Bit 6: Indicates the IPTS time source. 0 = Packet header 48-bit RTC. 1 = Packet secondary header time (bit 7 must be 1)"; "Bits 3-2: Indicate the packet secondary header time format" (§11.2.1.1 g); T-1. Replaces L1-IPT-001 to 004.

**Verification**: T

### L1-IPT-006

**Statement**: The library SHALL read an intra-packet time stamp as the 48-bit counter plus 16 high-order zero bits when packet flags bit 6 is 0, and in the format of bits 3–2 when bit 6 is 1.

**Source**: "These 8 bytes contain time in either 48-bit RTC format (plus 16 high-order zero bits) or 64-bit format as specified in the packet flags in the packet header" (§11.2.1.3 b); §11.2.1.1 g; T-1.

**Verification**: T

### L1-IPT-007

**Statement**: The library SHALL NOT read a time from a time stamp whose packet has bit 6 set with bit 7 clear, or bits 3–2 equal to `11`, and SHALL report the packet.

**Source**: "(bit 7 must be 1)"; "11 = Reserved" (§11.2.1.1 g); contract 6.6.

**Verification**: T

---

## L1-SEC: The secondary header

### L1-SEC-001

**Statement**: The library SHALL read the 12-byte secondary header — 8 bytes of time, 2 reserved bytes, a 2-byte checksum — with its time in Chapter 4 binary weighted format when packet flags bits 3–2 are `00`.

**Source**: "The length of the packet secondary header is fixed at 12 bytes (96 bits)" (§11.2.1.2); Figure 11-4. Prototype identifier kept.

**Verification**: T

### L1-SEC-002

**Statement**: The library SHALL read the secondary header's time in IEEE 1588 format when packet flags bits 3–2 are `01`.

**Source**: §11.2.1.2 a; Figure 11-5. Prototype identifier kept.

**Verification**: T

### L1-SEC-003

**Statement**: The library SHALL read the secondary header's time in ERTC format when packet flags bits 3–2 are `10`.

**Source**: §11.2.1.2 a; Figure 11-6. Prototype identifier kept.

**Verification**: T

### L1-SEC-004

**Statement**: The library SHALL verify the secondary header checksum as §11.2.1.2 c defines it, and SHALL NOT use the header's time when the checksum fails.

**Source**: "a 16-bit arithmetic sum of all secondary header bytes excluding the secondary header checksum word" (§11.2.1.2 c); whether "bytes" means bytes or 16-bit words is open (T-15: the prototype sums words; the words of the standard and RCC 123-20's `uCalcSecHeaderChecksum` sum bytes); contract 6.6. Prototype identifier kept; the reading is settled in L2.

**Verification**: T

### L1-SEC-005

**Statement**: The library SHALL report a secondary header whose reserved bytes are not zero.

**Source**: "Reserved. These 2 bytes are reserved and shall be zero filled" (§11.2.1.2 b).

**Verification**: T

### L1-SEC-006

**Statement**: The library SHALL report channels of one recording whose secondary headers use different time formats, and SHALL read each packet in its own format.

**Source**: "all channels that have a secondary header must use the same time source in bits 2-3 of the packet flags" (§11.2.1.2 a); contract 6.6.

**Verification**: T

---

## L1-CSDW: Format 1 channel-specific data word

### L1-CSDW-001

**Statement**: The library SHALL read the 32-bit channel-specific data word of Time Data, Format 1 (data type `0x11`) as Figure 11-12 lays it out.

**Source**: §11.2.3.2 a; Figure 11-12; Table 11-4. Prototype identifier kept.

**Verification**: T

### L1-CSDW-002

**Statement**: The library SHALL read the time source (SRC, bits 3–0) with the values `0x0` internal, `0x1` external, `0x2` internal from RMM, `0xF` none, and `0x3` to `0xE` reserved.

**Source**: §11.2.3.2 a, "0x3 – 0xE = Reserved"; T-6. Prototype identifier kept; value 3 is no longer "GPS".

**Verification**: T

### L1-CSDW-003

**Statement**: The library SHALL read the time format (FMT, bits 7–4) with the values `0x0` IRIG-B, `0x1` IRIG-A, `0x2` IRIG-G, `0x3` real-time clock, `0x4` UTC from GPS, `0x5` native GPS, `0xF` NONE, and `0x6` to `0xE` reserved, and SHALL treat a payload marked NONE as invalid.

**Source**: "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); T-7. Prototype identifier kept.

**Verification**: T

### L1-CSDW-004

**Statement**: The library SHALL read the leap-year indicator (bit 8).

**Source**: "Bit 8: Indicates if this is a leap year" (§11.2.3.2 a). Prototype identifier kept.

**Verification**: T

### L1-CSDW-005

**Statement**: The library SHALL read the date format (bit 9): 0, day of year (Figure 11-13); 1, day, month, and year (Figure 11-14).

**Source**: "0 = IRIG day available (Figure 11-13) 1 = Month and year available (Figure 11-14)" (§11.2.3.2 a). Prototype identifier kept.

**Verification**: T

### L1-CSDW-006

**Statement**: The library SHALL read the IRIG time source (ITS, bits 15–12) with its eight defined values and `1000` to `1111` reserved, SHALL treat it as meaningful only when FMT is IRIG-A, B, or G, and SHALL report it as not reported when the recording's declared edition predates 106-17.

**Source**: "IRIG Time Source (ITS). Bits 15-12 provide dynamic information regarding the source of time for an internal IRIG time code generator (TCG) when FMT is IRIG-A, B, or G" (§11.2.3.2 a); T-7; the field is reserved before 106-17 and Format 1's data type version did not change (T-14, a reading for the register).

**Verification**: T

### L1-CSDW-007

**Statement**: The library SHALL report a Format 1 data word whose reserved bits (31–16, 11–10) are not zero.

**Source**: "All reserved bit fields in packet headers or CSDWs shall be set to zero (0x0)" (§11.2.1 f); §11.2.3.2 a; T-7.

**Verification**: T

---

## L1-BCD: Format 1 time bodies

### L1-BCD-003

**Statement**: The library SHALL report a time body whose always-zero bits are not zero.

**Source**: Figures 11-13 and 11-14; Table 11-16, "0 Always zero". Prototype identifier kept.

**Verification**: T

### L1-BCD-004

**Statement**: The library SHALL reject a binary-coded decimal digit greater than 9.

**Source**: "the time data words are inserted in the packet in binary-coded decimal format" (§11.2.3.2 b). Prototype identifier kept.

**Verification**: T

### L1-BCD-005

**Statement**: The library SHALL read and write the day-format body as three 16-bit words (6 bytes): tens and hundreds of milliseconds, units and tens of seconds; units and tens of minutes and hours; units, tens, and hundreds of days.

**Source**: Figure 11-13; Table 11-16; T-12. Replaces L1-BCD-001 (8 bytes).

**Verification**: T

### L1-BCD-006

**Statement**: The library SHALL read and write the day, month, and year body as four 16-bit words (8 bytes), adding units and tens of months and units, tens, hundreds, and thousands of years.

**Source**: Figure 11-14; Table 11-16; T-12. Replaces L1-BCD-002 (10 bytes).

**Verification**: T

---

## L1-F2CSDW: Format 2 channel-specific data word

### L1-F2CSDW-001

**Statement**: The library SHALL read the 32-bit channel-specific data word of Time Data, Format 2 (data type `0x12`) as Figure 11-15 lays it out.

**Source**: §11.2.3.3 a; Figure 11-15. Prototype identifier kept.

**Verification**: T

### L1-F2CSDW-004

**Statement**: The library SHALL read the network time format (NTF, bits 7–4): `0x0` NTP version 3, `0x1` IEEE 1588-2002, `0x2` IEEE 1588-2008, `0x3` to `0xF` reserved.

**Source**: "Network Time Format (NTF). Bits 7-4" (§11.2.3.3 a); T-2. Replaces L1-F2CSDW-002.

**Verification**: T

### L1-F2CSDW-005

**Statement**: The library SHALL read the time status (TS, bits 3–0): `0x0` time not valid, `0x1` time valid, `0x2` to `0xF` reserved.

**Source**: "Time Status (TS). Bits 3-0 indicate the status of the network time" (§11.2.3.3 a); T-2. Replaces L1-F2CSDW-002.

**Verification**: T

### L1-F2CSDW-006

**Statement**: The library SHALL report a Format 2 data word whose reserved bits (31–8) are not zero.

**Source**: "Reserved. Bits 31-8 are reserved" (§11.2.3.3 a); §11.2.1 f; T-2. Replaces L1-F2CSDW-003 (bits 31–4).

**Verification**: T

---

## L1-NTP: Format 2 NTP time

### L1-NTP-001

**Statement**: The library SHALL read and write the NTP body as two little-endian 32-bit words: seconds, then fractions of a second.

**Source**: "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (Figure 11-16). Prototype identifier kept.

**Verification**: T

### L1-NTP-002

**Statement**: The library SHALL interpret NTP seconds as seconds since 1900-01-01 00:00:00 UTC, leap seconds included.

**Source**: "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds" (§11.2.3.3). Prototype identifier kept.

**Verification**: T

### L1-NTP-003

**Statement**: The library SHALL convert NTP fractions (units of 2^-32 s) to nanoseconds.

**Source**: NTP version 3 (RFC 1305, cited by §11.2.3.3 a). Prototype identifier kept.

**Verification**: T

### L1-NTP-004

**Statement**: The library SHALL convert NTP time to Unix time and to absolute time.

**Source**: Contract 3.3. Prototype identifier kept.

**Verification**: T

---

## L1-PTP: Format 2 PTP time

### L1-PTP-002

**Statement**: The library SHALL interpret PTP seconds as seconds since 1970-01-01 00:00:00 TAI, without leap seconds.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds" (§11.2.3.3). Prototype identifier kept.

**Verification**: T

### L1-PTP-003

**Statement**: The library SHALL reject PTP nanoseconds of 1,000,000,000 or more.

**Source**: Figure 11-17 ("Time Unsigned Nanoseconds"). Prototype identifier kept.

**Verification**: T

### L1-PTP-004

**Statement**: The library SHALL convert PTP (TAI) time to UTC by the leap-second offset in force at that time.

**Source**: §11.2.3.3; L1-TAI. Prototype identifier kept.

**Verification**: T

### L1-PTP-005

**Statement**: The library SHALL read and write the PTP body as two little-endian 32-bit words: seconds, then nanoseconds (8 bytes).

**Source**: "Time Unsigned Seconds", "Time Unsigned Nanoseconds" (Figure 11-17; the same in 106-17 and 106-19); T-3. Replaces L1-PTP-001 (10 bytes, 48-bit seconds).

**Verification**: T

---

## L1-F2COR: Format 2 references

### L1-F2COR-001

**Statement**: The library SHALL take Format 2 time packets as references, as it does Format 1.

**Source**: "The Format 2 Network Time packet data type is used to extract and encapsulate absolute time from NTP or IEEE-1588 PTP and time tag it with the RTC" (§11.2.3.3). Prototype identifier kept.

**Verification**: T

### L1-F2COR-002

**Statement**: The library SHALL keep each reference's time scale (UTC or TAI) and SHALL NOT combine references of different scales without converting through the leap-second table.

**Source**: §11.2.3.3; contract 4.1 rule 4. Prototype identifier kept.

**Verification**: T

---

## L1-TAI: Time scales and leap seconds

### L1-TAI-001

**Statement**: The library SHALL provide a leap-second table mapping dates to the TAI−UTC offset.

**Source**: L1-PTP-004. Prototype identifier kept.

**Verification**: T

### L1-TAI-002

**Statement**: The library SHALL build in a leap-second table current at its release.

**Source**: Contract 6.8. Prototype identifier kept.

**Verification**: I

### L1-TAI-003

**Statement**: The library SHALL let the caller supply or extend the table.

**Source**: Contract 6.8. Prototype identifier kept.

**Verification**: T

### L1-TAI-004

**Statement**: The library SHALL state the last date to which its table is known to be current, and SHALL label a conversion after that date as possibly out of date.

**Source**: "The leap-second table does not reach the date … the conversion is given with the last known offset and labelled as possibly out of date" (contract 6.8).

**Verification**: T

---

## L1-WORD: Time words inside data

### L1-WORD-001

**Statement**: The library SHALL turn a set of Chapter 4 time words — high order, low order, and microsecond — into absolute time, binary or BCD weighted as the caller states.

**Source**: "High and low order time words shall be binary or binary coded decimal (BCD) weighted, and microsecond words shall be binary weighted"; resolutions of 1 µs (to 9999), 10 ms, and 655.36 s binary or one minute BCD; "For BCD, the days field shall contain the three least significant bits of the BCD Julian date" (Chapter 4 §4.7); `C-d\PTM`, `C-d\BTM` (Chapter 9); contract 3.8. Layout to be confirmed with T-10.

**Verification**: T

### L1-WORD-002

**Statement**: The library SHALL turn network time words from data — PTP time, seconds, or nanoseconds; NTP time, seconds, or fractions — into absolute time.

**Source**: `C-d\NTM`: "PTP Time (64-bit)", "PTP Seconds (32-bit)", "PTP Nanoseconds (32-bit)", "NTP Time (64-bit)", "NTP Seconds (32-bit)", "NTP Fractions (32-bit)" (Chapter 9, Table 9-11); contract 3.8.

**Verification**: T

---

## L1-ENC: Encoding

### L1-ENC-001

**Statement**: The library SHALL encode the Format 1 data word and both Format 1 bodies byte for byte as Figures 11-12 to 11-14 lay them out, with reserved and always-zero bits zero.

**Source**: §11.2.3.2; §11.2.1 f; contract 2.2, 4.11.

**Verification**: T

### L1-ENC-002

**Statement**: The library SHALL encode the Format 2 data word and both Format 2 bodies byte for byte as Figures 11-15 to 11-17 lay them out, with reserved bits zero.

**Source**: §11.2.3.3; §11.2.1 f; contract 2.2, 4.11.

**Verification**: T

### L1-ENC-003

**Statement**: The library SHALL encode secondary-header times, their checksum, and intra-packet time stamps in each format of packet flags bits 3–2.

**Source**: §11.2.1.1 g, §11.2.1.2, §11.2.1.3 b.

**Verification**: T

---

## L1-EDN: Editions

### L1-EDN-001

**Statement**: The library SHALL accept Format 2 time packets in recordings of 106-17 and later, and SHALL report them in recordings declared earlier.

**Source**: 106-17 Chapter 11 lists "0x12 Time Data, Format 2 Network Time" with data type version `0x08` (106-17); 106-13 and 106-15 do not define it; T-4.

**Verification**: T

### L1-EDN-002

**Statement**: The library SHALL take the edition of a recording from `irig106-types`' two code lists — the packet header's data type version and the setup record's version — and SHALL treat unassigned codes as unknown.

**Source**: Table 11-4 and §11.2.1.1 e ("0x0A = RCC 106-22"); Figure 11-34 ("0x0E = RCC 106-22", "0x0F through 0xFF = Reserved"); T-9; ADR-0015.

**Verification**: T

### L1-EDN-003

**Statement**: The library SHALL read the time packets, secondary headers, and time stamps of every edition from 106-04 onward, and SHALL cite each difference between editions it relies on.

**Source**: "References to RCC 106-04 through RCC 106-15 refer to Chapter 10, while RCC 106-17 onward refer to Chapter 11" (§11.2.1.1 e); ADR-0004.

**Verification**: T

---

## L1-DECL: Time declarations from the setup record

### L1-DECL-001

**Statement**: The library SHALL accept, as plain data for each setup record, the counter from which it governs, its time channels with their `R-x\TTF-n`, `R-x\TFMT-n`, and `R-x\TSRC-n`, each channel's `R-x\SHTF-n`, the declared recording-format version, and `R-x\RI4`.

**Source**: Contract 2.3 C, 4.5, 5.3; `irig106-tmats` L1-CH10-008 (the configuration timeline); ADR-0005.

**Verification**: T

### L1-DECL-002

**Statement**: The library SHALL work without time declarations, and SHALL say in every answer and finding whether declarations were given.

**Source**: A recording's setup record can be missing or unreadable; contract 6.4.

**Verification**: T

---

## L1-COR: Correlation over a recording (the time timeline)

### L1-COR-001

**Statement**: The library SHALL build, from the time packets and the counters of a recording's packets in file order, a time timeline of references that pair counter values with absolute time.

**Source**: "Time is treated like another data channel" (§11.2.3.2); "Since Time Data, Format 1 packets contain both the absolute input time value and the RTC clock value at the instant the absolute time was valid, these packets can be used to relate RTC values to the input absolute time source" (RCC 123-20 §5.6); contract 5.5. Prototype identifier kept; citation corrected (T-13).

**Verification**: T

### L1-COR-002

**Statement**: By default, the library SHALL take the reference of the chosen time channel nearest in counter value to the counter being converted.

**Source**: "It is better to use the clock and relative time values from a time packet that occurs near the current data packet as the data file is decoded since there is some drift in the RTC during a recording session" (RCC 123-09 §6.6; T-13); contract 5.2 rule 5; a policy setting (L1-POL-001). Prototype identifier kept.

**Verification**: T

### L1-COR-003

**Statement**: The library SHALL use one time channel per session — the caller's choice; otherwise the channel the setup record declares with an external source; otherwise the channel with the most valid references; otherwise the lowest channel ID — and SHALL state which and why.

**Source**: "When multiple time channels are available, it is incumbent on the programmer or data analyst to determine and select the best source of time for a particular data set" (RCC 123-20 §5.6); "It is usually most correct to select one time channel only and to use this channel exclusively" (RCC 123-09 §6.6); contract 5.2 rule 4; a policy setting. Prototype identifier kept.

**Verification**: T

### L1-COR-004

**Statement**: The library SHALL detect a time jump — absolute time and counter time advancing by different amounts between consecutive references of a channel beyond a threshold — and SHALL report it.

**Source**: "there is a jump in input clock time during a recording, such as when GPS locks for the first time, or when an IRIG time source is reprogrammed" (RCC 123-09 §6.6; T-13); contract 5.4; default threshold the format's resolution (L1-POL-001). Prototype identifier kept.

**Verification**: T

### L1-COR-005

**Statement**: The library SHALL divide a recording into sessions, starting a new session when the counter goes backwards beyond the late-packet bound, and SHALL NOT use a reference across a session boundary.

**Source**: The counter "shall remain free-running during each session (e.g., recording)" (§11.2.1.1 i); contract 5.2 rule 1.

**Verification**: T

### L1-COR-006

**Statement**: By default, the library SHALL take references only from time packets on channels the governing setup record declares as time channels, and only when their time is valid.

**Source**: Contract 5.2 rules 2 and 3; "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); "0x0 = Time Not Valid" (§11.2.3.3 a); policy settings.

**Verification**: T

### L1-COR-007

**Statement**: The library SHALL keep using a channel's references after its reported source changes, each labelled with the source it reported, and SHALL report the change.

**Source**: "If the time source is external (0x1) and lock on the external source is lost then the time source shall indicate Internal (0x0). Once lock on the external time source is regained, time source shall once again indicate external (0x1)" (§11.2.3.2 a); contract 5.2 rule 6.

**Verification**: T

### L1-COR-008

**Statement**: The library SHALL report a reference gap: consecutive valid references of the chosen channel further apart than one second plus a tolerance.

**Source**: "the time packet will be generated at a minimum frequency of 1 hertz" (§11.2.3.2; Chapter 10 §10.6.2); the Format 2 exception "unless it is recorded at the raw network rate" (§11.2.3.3); contract 5.4.

**Verification**: T

### L1-COR-009

**Statement**: The library SHALL accept packets whose counter is earlier than one already seen within the late-packet bound (default 1100 ms), and SHALL report later ones.

**Source**: "all other packet generation times shall be equal to or less than 100 milliseconds (ms)"; "all other packets shall have a stream commit time equal to or less than 1000 ms" (Chapter 10 §10.6.1 b–c); contract 5.4.

**Verification**: T

### L1-COR-010

**Statement**: The library SHALL find the year of each session — from a day-month-year time packet, then the caller, then `R-x\RI4` — or leave it unknown, SHALL say where it came from, SHALL check it against the leap-year bit, and SHALL advance it across 31 December.

**Source**: Figures 11-13 and 11-14; "Date and time original recording was created" (`R-x\RI4`, Chapter 9); contract 5.3.

**Verification**: T

### L1-COR-011

**Statement**: The library SHALL give, for each session, its span in absolute time, and for a span of absolute time, the counter ranges that cover it.

**Source**: Contract 3.6, 4.9, 5.5.

**Verification**: T

### L1-COR-012

**Statement**: The library SHALL report dynamic packets that precede a session's first time packet, and sessions with no time packet.

**Source**: "A time data packet shall be the first dynamic data packet at the start of each session. Only static Computer-Generated Data, Format 1 packets may precede the first time data packet" (§11.2.3.2; "of each recording", Chapter 10 §10.6.2); "If the time data packet source is None, at least one time data packet is required" (§11.2.3.2); contract 6.1, 6.2.

**Verification**: T

### L1-COR-013

**Statement**: The library SHALL provide measures of the references per channel: count, smallest and largest spacing, references per second, and drift in parts per million.

**Source**: Contract 3.5; the prototype's `compute_quality`.

**Verification**: T

---

## L1-ANS: Answers and their basis

### L1-ANS-001

**Statement**: The library SHALL give every absolute time with its basis: the session; the time channel and the rule that chose it; the reference (counter and time); the distance to it; its position (between references, before the first, beyond the last); the reference's source, format, and ITS; the year and its origin; the corrections applied; and the policy.

**Source**: Contract 3.1, 3.5, 7.3; ADR-0011.

**Verification**: T

### L1-ANS-002

**Statement**: For a packet with a valid secondary header, the library SHALL give the header's time as the packet's own time alongside the time derived from its counter, and SHALL report a difference beyond the policy's tolerance.

**Source**: "The applicable data bit to which the 48-bit value of the packet secondary time value (if enabled) applies shall correspond to the first bit of the data in the packet body" (§11.2.1.2 a); contract 6.6.

**Verification**: T

### L1-ANS-003

**Statement**: The library SHALL give the absolute time of an intra-packet time stamp as L1-IPT-006 reads it, with its basis.

**Source**: §11.2.1.3 b; contract 3.4, 4.6.

**Verification**: T

### L1-ANS-004

**Statement**: The library SHALL give the absolute time of a set of time words (L1-WORD) with its basis.

**Source**: Contract 3.8, 4.6.

**Verification**: T

### L1-ANS-005

**Statement**: For a counter in a session with no usable reference, the library SHALL give counter time only — the elapsed time from the session's first packet — labelled as such.

**Source**: Contract 6.1, 6.2.

**Verification**: T

---

## L1-POL: The time policy

### L1-POL-001

**Statement**: The library SHALL provide a time policy holding every setting of contract section 5.6, each with its documented default.

**Source**: Contract 5.6; ADR-0010.

**Verification**: T

### L1-POL-002

**Statement**: The library SHALL let the caller change each setting of the policy.

**Source**: "I would like for it to be selectable as well" (owner, 2026-09-27); ADR-0010.

**Verification**: T

### L1-POL-003

**Statement**: The library SHALL record in every answer and every time timeline the policy it used, or the settings that differ from the defaults.

**Source**: ADR-0010.

**Verification**: T

---

## L1-FND: Findings

### L1-FND-001

**Statement**: The library SHALL report each problem with time as a finding with a stable identifier, a default severity the caller can change, and its evidence (packet, channel, counter), and SHALL never reuse an identifier.

**Source**: Contract section 6; ADR-0013.

**Verification**: T

### L1-FND-002

**Statement**: The library SHALL report each case of contract section 6 as a finding.

**Source**: Contract 6.1 to 6.9.

**Verification**: T

### L1-FND-003

**Statement**: The library SHALL report packets whose flags mark an RTC sync error.

**Source**: "Bit 5: RTC sync error. 0 = No RTC sync error. 1 = RTC sync error has occurred" (§11.2.1.1 g); found while writing the architecture (2026-09-27), not yet in the contract.

**Verification**: T

### L1-FND-004

**Statement**: The library SHALL report where a time channel's packets differ from its declarations (`R-x\TTF-n`, `R-x\TFMT-n`, `R-x\TSRC-n`, `R-x\SHTF-n`), treating a differing source as information.

**Source**: Contract 3.7, 6.4; the source may change legitimately (§11.2.3.2 a).

**Verification**: T

### L1-FND-005

**Statement**: The library SHALL report a packet whose ERTC and RTC disagree beyond one RTC tick.

**Source**: "the 10-megahertz (MHz) RTC shall be synchronized with the ERTC (RTC = ERTC/100)" (§11.2.1.1 g).

**Verification**: T

---

## L1-RDG: Readings between TMATS and the packets

### L1-RDG-001

**Statement**: The library SHALL compare TMATS values with packet values only through readings recorded in a register, each naming its register entry.

**Source**: Contract 3.7; T-14; ADR-0012.

**Verification**: I (the register and the code's references to it)

---

## L1-STRM: Streaming

### L1-STRM-001

**Statement**: The library SHALL provide a streaming form that applies the same policy and gives the same answers, with memory bounded by the late-packet bound and the spacing of references rather than by the length of the recording.

**Source**: Contract 4.8; architecture section 6.

**Verification**: T

---

## L1-ERR: Errors and robustness

### L1-ERR-001

**Statement**: The library SHALL return a typed error for input it cannot read and SHALL NOT panic on any input.

**Source**: Prototype identifier kept.

**Verification**: T (property tests and fuzzing)

### L1-ERR-002

**Statement**: The library SHALL report an invalid binary-coded decimal digit as an error that names the field.

**Source**: §11.2.3.2 b. Prototype identifier kept.

**Verification**: T

### L1-ERR-003

**Statement**: The library SHALL reject out-of-range time fields: hours above 23, minutes or seconds above 59, a day of year of 0 or above 366, a month of 0 or above 12, and a day of month beyond its month.

**Source**: Figures 11-13, 11-14. Prototype identifier kept; day 0 is rejected, never turned into day 1 (contract 8.3).

**Verification**: T

### L1-ERR-004

**Statement**: The library SHALL report a secondary header checksum failure.

**Source**: §11.2.1.2 c. Prototype identifier kept.

**Verification**: T

### L1-ERR-005

**Statement**: The library SHALL NOT read beyond the bytes the caller gives it, and SHALL read a time packet body within its Data Length.

**Source**: "Data Length … does not include packet trailer filler and data checksum" (§11.2.1.1 d); T-12.

**Verification**: T

---

## L1-API: Library form and portability

### L1-API-001

**Statement**: The library's public value types SHALL implement `Debug`, `Clone`, `PartialEq`, and, where their contents allow, `Copy`, `Eq`, and `Hash`.

**Source**: Prototype identifier kept.

**Verification**: I

### L1-API-002

**Statement**: The library SHALL build without `std`, with `alloc` for the recording layer.

**Source**: ADR-0014. Prototype identifier kept.

**Verification**: I (CI builds a target without `std`)

### L1-API-004

**Statement**: The library SHALL use distinct types for quantities of different units or epochs — ticks, nanoseconds, UTC and TAI seconds, offsets.

**Source**: Prototype identifier kept; `irig106-types` newtypes.

**Verification**: I

### L1-API-005

**Statement**: The library SHALL depend on no crate of the ecosystem other than `irig106-types`.

**Source**: ADR-0005. Replaces L1-API-003 ("zero required runtime dependencies").

**Verification**: I (the manifest, checked in CI)

### L1-API-006

**Statement**: The library SHALL perform no I/O: no files, network, clock, or environment.

**Source**: ADR-0006.

**Verification**: I

### L1-API-007

**Statement**: The library SHALL build for `wasm32-unknown-unknown`, with and without `serde`.

**Source**: `irig106-tmats` L1-REL-003; contract 4.8; ADR-0014.

**Verification**: I (CI job)

---

## L1-CLI: The command-line crate

### L1-CLI-001

**Statement**: The project SHALL provide `irig106-time-cli` as a library and a binary named `irigtime`.

**Source**: ADR-0007; owner, 2026-09-26 and 2026-09-27.

**Verification**: I

### L1-CLI-002

**Statement**: The CLI's library SHALL expose an entry point and its commands and renderers so that `irig106-cli` can mount them as `irig106 time`.

**Source**: ADR-0007; ROADMAP P6-10.

**Verification**: T

### L1-CLI-003

**Statement**: The CLI SHALL parse its arguments without an argument-parsing crate.

**Source**: ADR-0008.

**Verification**: I

### L1-CLI-004

**Statement**: The CLI SHALL provide `summary`, `channels`, `jumps`, `timeline`, `csv`, and `correlate`, each showing the basis and the findings of what it reports.

**Source**: Contract 4.12; ADR-0011, ADR-0013.

**Verification**: T

### L1-CLI-005

**Statement**: The CLI SHALL report network time as NTP or PTP, never as GPS.

**Source**: ROADMAP P6-10; §11.2.3.3.

**Verification**: T

---

## L1-REL: Versioning and release

### L1-REL-001

**Statement**: The project SHALL release `irig106-time` and `irig106-time-cli` together at one version, the CLI pinning the library exactly.

**Source**: ADR-0007.

**Verification**: I

### L1-REL-002

**Statement**: The project SHALL use edition 2024 and declare Rust 1.85 as its minimum version, and CI SHALL check that version.

**Source**: ADR-0003.

**Verification**: I (CI MSRV job)

### L1-REL-003

**Statement**: The first rebuilt release SHALL list in its changelog each finding of `docs/STANDARD-REVIEW.md` that changes an answer of 0.7.0.

**Source**: ADR-0017.

**Verification**: I

---

## L1-TST: Verification

### L1-TST-001

**Statement**: The project SHALL have, for each finding of `docs/STANDARD-REVIEW.md` that concerns the code, a test that fails on the prototype and passes on the fix.

**Source**: ADR-0016.

**Verification**: I

### L1-TST-002

**Statement**: The project SHALL test the worked example of contract section 7 byte for byte, including its basis.

**Source**: Contract 7.6; ADR-0016.

**Verification**: T

### L1-TST-003

**Statement**: The project's fixtures SHALL be derived from the standard's figures and tables, each citing its source, and SHALL NOT be produced by the library's own encoders alone.

**Source**: ADR-0016.

**Verification**: I

### L1-TST-004

**Statement**: The project SHALL NOT commit real recordings or use them in CI.

**Source**: Owner's standing direction; ADR-0016; `irig106-tmats` ADR-0018.

**Verification**: I

---

## Non-requirements

| ID | Not done | Why |
|----|----------|-----|
| NR-001 | Decoding IRIG serial time codes (RCC 200) as signals | Recorders deliver time in packets; RCC 200 is cited where Chapter 11 defers to it (contract 3.10) |
| NR-002 | Opening files or walking packets in the library | ADR-0006; the CLI and `irig106-core` do it |
| NR-003 | Reading TMATS | `irig106-tmats` (ADR-0005) |
| NR-004 | Finding time stamps and time words in data bodies | `irig106-decode` (contract 4.6) |
| NR-005 | Decoding recording events | `irig106-decode` (ADR-0009) |
| NR-006 | Assembling packets | `irig106-write` (contract 4.11) |

---

## Prototype requirements

The prototype's 53 L1 requirements (0.2.0, kept at `prototype-0`), and what
became of each.

| Prototype | Now | Why |
|-----------|-----|-----|
| L1-RTC-001 to 004 | kept | re-cited to §11.2.1.1 i |
| L1-ABS-001 to 004 | kept | re-cited; ERTC's 1 ns now stated (T-5); Chapter 4 layout pending T-10 |
| L1-CSDW-001 to 005 | kept | re-cited; value 3 reserved (T-6), NONE added (T-7) |
| L1-BCD-001 | **retired** → L1-BCD-005 | said 8 bytes; the day body is 6 (T-12) |
| L1-BCD-002 | **retired** → L1-BCD-006 | said 10 bytes; the day, month, and year body is 8 (T-12) |
| L1-BCD-003, 004 | kept | re-cited |
| L1-SEC-001 to 004 | kept | re-cited; the checksum reading open (T-15) |
| L1-IPT-001 to 004 | **retired** → L1-IPT-005 to 007 | selected the format by bit 2 (T-1) |
| L1-COR-001 to 004 | kept | handbook citations corrected (T-13) |
| L1-ERR-001 to 004 | kept | re-cited; day 0 rejected |
| L1-API-001, 002, 004 | kept | — |
| L1-API-003 | **retired** → L1-API-005 | "zero required runtime dependencies" no longer true |
| L1-F2CSDW-001 | kept | — |
| L1-F2CSDW-002 | **retired** → L1-F2CSDW-004, 005 | read the protocol from bits 3–0 (T-2) |
| L1-F2CSDW-003 | **retired** → L1-F2CSDW-006 | reserved bits are 31–8, not 31–4 (T-2) |
| L1-NTP-001 to 004 | kept | re-cited |
| L1-PTP-001 | **retired** → L1-PTP-005 | said 10 bytes with 48-bit seconds (T-3) |
| L1-PTP-002 to 004 | kept | re-cited |
| L1-F2COR-001, 002 | kept | re-cited |
| L1-TAI-001 to 003 | kept | L1-TAI-004 added |

Ten prototype identifiers are retired; 43 are kept.

---

## Sources

| Source | Where |
|--------|-------|
| IRIG 106-24R1 Chapter 11 (§11.2.1, §11.2.1.1–4, §11.2.3.2–3, Table 11-4, Figures 11-4 to 11-6, 11-12 to 11-17, 11-34) | `rcc-106-standards`, release `106-24R1` |
| IRIG 106-24R1 Chapter 10 (§10.6.1, §10.6.2) | the same |
| IRIG 106-24R1 Chapter 4 (§4.7) | the same |
| IRIG 106-24R1 Chapter 9 (`R-x\TTF-n`, `R-x\TFMT-n`, `R-x\TSRC-n`, `R-x\SHTF-n`, `R-x\RI4`, `C-d\PTM`, `C-d\BTM`, `C-d\NTM`) | the same |
| IRIG 106-05 to 106-15 Chapter 10, 106-17 to 106-24 Chapter 11 (edition differences) | the editions' releases |
| RCC 123-09 §6.6, RCC 123-20 §5.6 and Appendix A-2 | releases `rcc-123-09`, `rcc-123-20` |
| RCC 200-16 | release `rcc-200-16` |
