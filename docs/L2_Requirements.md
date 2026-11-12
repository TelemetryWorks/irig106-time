# irig 106-time — Level 2 Requirements

> **Status: draft for owner and independent review, 2026-11-12.**
> Rebuild requirements, not implementation evidence. Replaces the prototype
> document. Derives from the current L1, the contract, accepted ADRs, the
> architecture, and the owner's 2026-11-11 answers (ROADMAP questions 1–5).
> No Review field or owner-set Design status is changed by this draft.

## Scope and conventions

The checked-out L1 at main `980d3af` contains **124 defined entries in 27
categories**, not the 144 mentioned in the assignment. Every one has a
complete L1 → L2 → L3 path; no missing L1 requirements have been invented. The prototype
identifiers below are retired, never reassigned; new base IDs start at
101 and additional decompositions at 201. They do not imply existing test
coverage. Implementation is a later tests-first step (ADR-0016). There are **142
active requirements at this level**; all 27 L1 categories are covered.

Every obligation uses SHALL. Verification is planned: T = Test, I =
Inspection, A = Analysis, D = Demonstration. A planned check is not an
**Evidence** artifact. The generated TRACE-MATRIX remains the live status
source. Existing prototype tests do not verify a rebuilt obligation unless
reviewed and explicitly linked to it in the later implementation work.

**Source convention:** every §, figure and table without another edition
means **IRIG 106-24R1**; Chapters 4/9/10/11 are named where needed.
The Source lines retain the L1 citations and quoted source words; obsolete
prototype-ID commentary is omitted and completed page checks are annotated. Requirements based on contract/ADR/owner policy
are product decisions, not claims that the standard prescribes an
algorithm. Quotes are verbatim; derived arithmetic and proposed choices
are identified as such. Archive provenance and rendered-page checks are
in STANDARD-REVIEW T-10 and INTERPRETATIONS RDG-002.

Shared-value contracts target the corrected irig 106-types semantics of
contract section 8.6 (T-5/T-6/T-7/T-9), not the current prototype dependency.
A source's unresolved context is retained explicitly; Q-items below are
proposals requiring owner decisions, not silently implemented assumptions.
Human Review stays pending under the register's two-person rule.

L2 states functional boundaries, admission rules and externally observable
results. Every entry has exactly one L1 parent. Its L3 child supplies data
layout/algorithm obligations and planned cases. Additional STRM and RDG
entries decompose complex policy and reading obligations.

## Questions requiring owner decisions

| ID | Question | Recommended option and reason |
|----|----------|-------------------------------|
| Q-DEFAULT | Which stream mode is the default? | **standard**: immediate preceding-reference answers follow the RCC 123-20 §5.6 example; interpolate remains explicitly selectable. Offline nearest remains unchanged. |
| Q-STREAM-BOUND | What hold/gap tolerances and progress contract should streaming use? | Propose **100 ms hold tolerance**, separate **0 ns gap tolerance**, and explicit caller progress. A gap above 1 s is reported even during the hold allowance. Require caller buffer caps; no hidden unbounded queue. These are product choices, not standard-mandated tolerances. |
| Q-STREAM-CHANNEL | How can irrevocable streaming choose a channel when the offline ranking uses the final reference count? | Require a **caller-fixed channel or a stable declared ranking** for irrevocable answers; otherwise return counter-only until resolved. Do not silently revise already emitted answers when future counts change. |
| Q-CH4 | What supplies the binary calendar origin and missing BCD day bits? | Require **explicit caller reconstruction context**; return partial time without it. Neither visually checked figure supplies a complete calendar day. Do not assume day 1 or guess a truncated BCD day. |
| Q-NTP | How should NTP eras and the phrase “includes leap seconds” map to Unix/absolute time? | Preserve raw UTC-labelled seconds/fractions and **require explicit era plus a reviewed RFC 1305/leap mapping analysis** before ambiguous conversions are implemented. Never add all historical leap offsets by inference or assume the host's current era. |
| Q-FLAGS | Must reserved secondary format 11 reject an RTC-form stamp even when bit 6 is clear? | Keep **L1-IPT-007's rejection for this draft**; recommend an owner-approved L1 refinement limiting reserved-format rejection to secondary-derived stamps (bit 6 set). §11.2.1.1 g and §11.2.1.3 b select RTC by bit 6; current L1 wording is stricter. No L1 change is made here. |
| Q-CLI-EXIT | Accept the proposed numeric exit assignments? | **0 success, 1 findings threshold, 2 I/O/input failure, 3 usage**; document these before scripts depend on them. The four classes are L1; their numbers are a new interface proposal. |

Rounding (NTP fractions and interpolation), midpoint ties, mixed-resolution
jump thresholds, and mode-change flushing are explicitly proposed design
choices in the requirements. They need review as part of this draft; none
changes an accepted Design field. RDG-008 remains suspect; page inspection
does not settle it or replace its real-recording evidence.

## RTC

#### L2-RTC-101

**Parent**: L1-RTC-001

**Statement**: The library SHALL expose the shared Rtc type without adding an absolute epoch; retain its raw 48-bit value and distinguish counter ticks from durations.

**Source**: "This is a free-running 10-MHz binary counter represented by 48 bits that are common to all data channels" (§11.2.1.1 i).

**Verification Method**: Test (T)

**Planned verification**: Raw zero, maximum, one-bit boundaries, and a value with bit 48 set; confirm the chosen constructor contract and that no calendar date is produced.

#### L2-RTC-102

**Parent**: L1-RTC-002

**Statement**: The library SHALL provide separate header-counter and eight-byte counter-stamp readers, each checking its own minimum length and padding.

**Source**: §11.2.1.1 i, Table 11-2; "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b).

**Verification Method**: Test (T)

**Planned verification**: Distinct bytes 01 02 03 04 05 06 decode to 0x060504030201; truncate at every byte; vary both padding bytes independently.

#### L2-RTC-103

**Parent**: L1-RTC-003

**Statement**: The library SHALL resolve wrap-safe counter differences within a named session; require ordering context for differences that cannot be uniquely unwrapped.

**Source**: 2^48 ticks at 10 MHz is about 325.8 days; the counter "shall remain free-running during each session" (§11.2.1.1 i).

**Verification Method**: Test (T)

**Planned verification**: Maximum to zero is +1 tick and reverse is -1; equal counters are zero; exercise half-range ambiguity and reject a cross-session subtraction.

#### L2-RTC-104

**Parent**: L1-RTC-004

**Statement**: The library SHALL convert signed counter deltas to typed nanosecond durations without truncation or overflow.

**Source**: §11.2.1.1 i (10 MHz).

**Verification Method**: Test (T)

**Planned verification**: Zero, +1, -1, maximum session-supported delta, and destination overflow; 150000 ticks equals 15000000 ns.

## ABS

#### L2-ABS-101

**Parent**: L1-ABS-001

**Statement**: The library SHALL represent day-of-year, time-of-day, optional calendar year, and leap-second state without collapsing an unknown year into a fabricated one.

**Source**: IEEE 1588 and ERTC times resolve 1 ns (§11.2.1.1 g); the day format carries no year (Figure 11-13); "excluding leap second days when a second may be added or subtracted" (RCC 200-16 §3.6); contract 3.2; T-16.

**Verification Method**: Test (T)

**Planned verification**: Day 0/1/365/366, Gregorian 1900/2000/2100, unknown year, midnight and year-end, 23:59:59 to 23:59:60 to 00:00:00 with explicit leap context.

#### L2-ABS-102

**Parent**: L1-ABS-002

**Statement**: The library SHALL read and encode Chapter 4 binary secondary time using the visually checked Figure 11-4 layout; expose a binary duration until a calendar origin is supplied.

**Source**: "00 = Chapter 4 binary weighted 48-bit time format. The two lsbs of the 64-bit packet secondary header time and IPTS shall be zero-filled" (§11.2.1.1 g); Figure 11-4; Chapter 4 §4.7. The L1 source note still calls for the page check; it is now recorded in STANDARD-REVIEW T-10 (2026-11-12).

**Verification Method**: Test (T)

**Planned verification**: Hand-derived 00 00 0f 27 34 12 01 00 gives 701969999000 ns; nonzero reserved bytes, microseconds=9999/10000, high=0xFFFF, short buffers, and missing calendar origin. T-10 must fail on the prototype.

#### L2-ABS-103

**Parent**: L1-ABS-003

**Statement**: The library SHALL keep secondary-header IEEE 1588 seconds/nanoseconds distinct from the reversed word sequence of a Format 2 PTP body.

**Source**: "The 32 bits indicating seconds shall be placed into the MSLW portion of the secondary header and the 32 bits indicating nanoseconds shall be placed into the LSLW portion" (§11.2.1.1 g); Figure 11-5.

**Verification Method**: Test (T)

**Planned verification**: Distinct seconds=0x01020304 and ns=123456789; ns=999999999/1000000000; compare with a Format 2 body containing seconds first.

#### L2-ABS-104

**Parent**: L1-ABS-004

**Statement**: The library SHALL use the corrected shared Ertc value as nanoseconds, retaining its relative nature and full 64-bit range.

**Source**: "64-bit binary extended relative time counter (ERTC) with 1-nanosecond resolution … (RTC = ERTC/100)" (§11.2.1.1 g); Figure 11-6 (LSLW, MSLW); T-5.

**Verification Method**: Test (T)

**Planned verification**: ERTC=1 gives 1 ns, 100 gives 100 ns; max u64 and asymmetric bytes; T-5 regression must distinguish the prototype factor of 100.

## IPT

#### L2-IPT-105

**Parent**: L1-IPT-005

**Statement**: The library SHALL decode flags once into secondary-presence, stamp-source, secondary-format, and RTC-sync-error fields; unrelated flag bits must not alter time dispatch.

**Source**: "Bit 7: Indicates the presence or absence of the packet secondary header"; "Bit 6: Indicates the IPTS time source. 0 = Packet header 48-bit RTC. 1 = Packet secondary header time (bit 7 must be 1)"; "Bits 3-2: Indicate the packet secondary header time format" (§11.2.1.1 g); T-1.

**Verification Method**: Test (T)

**Planned verification**: All 256 flag bytes; compare flags differing only in unrelated bits. Cite T-1 on fixtures.

#### L2-IPT-106

**Parent**: L1-IPT-006

**Statement**: The library SHALL dispatch an eight-byte stamp by bit 6, using bits 3-2 for the secondary form only after flag validation.

**Source**: "These 8 bytes contain time in either 48-bit RTC format (plus 16 high-order zero bits) or 64-bit format as specified in the packet flags in the packet header" (§11.2.1.3 b); §11.2.1.1 g; T-1.

**Verification Method**: Test (T)

**Planned verification**: Every source/presence/format combination and each short length; padded RTC and each secondary representation from independently authored bytes.

#### L2-IPT-107

**Parent**: L1-IPT-007

**Statement**: The library SHALL return no stamp time and an evidenced finding for inconsistent flags or a reserved secondary format.

**Source**: "(bit 7 must be 1)"; "11 = Reserved" (§11.2.1.1 g); contract 6.6.

**Verification Method**: Test (T)

**Planned verification**: Bit 6 set with bit 7 clear for each format; format=11 with both sources; valid bit 6=0 stamps must remain RTC for formats 00/01/10.

## SEC

#### L2-SEC-101

**Parent**: L1-SEC-001

**Statement**: The library SHALL parse a secondary header as a bounded twelve-byte structure and decode Chapter 4 time only after validation.

**Source**: "The length of the packet secondary header is fixed at 12 bytes (96 bits)" (§11.2.1.2); Figure 11-4.

**Verification Method**: Test (T)

**Planned verification**: Twelve bytes with distinguishing words and an independently computed checksum; lengths 0 through 11; trailing body bytes must not enter the checksum.

#### L2-SEC-102

**Parent**: L1-SEC-002

**Statement**: The library SHALL for secondary format 01, return the IEEE 1588 value plus header validation results, not a Format 2 body.

**Source**: §11.2.1.2 a; Figure 11-5.

**Verification Method**: Test (T)

**Planned verification**: Seconds/nanoseconds in different values, boundary nanoseconds, bad checksum, and a packet-specific applicability supplied as metadata.

#### L2-SEC-103

**Parent**: L1-SEC-003

**Statement**: The library SHALL for secondary format 10, return the relative ERTC value and validate it independently of calendar correlation.

**Source**: §11.2.1.2 a; Figure 11-6.

**Verification Method**: Test (T)

**Planned verification**: ERTC=1/99/100 and all-high bits; checksum failure suppresses trusted own time, while absent references must not fabricate absolute time.

#### L2-SEC-104

**Parent**: L1-SEC-004

**Statement**: The library SHALL use the RDG-008 byte-sum reading for trust decisions and calculate the alternative word sum for diagnostics; retain its suspect status.

**Source**: "a 16-bit arithmetic sum of all secondary header bytes excluding the secondary header checksum word" (§11.2.1.2 c); whether "bytes" means bytes or 16-bit words is open (T-15: the prototype sums words; the words of the standard and RCC 123-20's `uCalcSecHeaderChecksum` sum bytes); contract 6.6.

**Verification Method**: Test (T)

**Planned verification**: Headers matching byte only, word only, both, neither, and short input; all-edition synthetic cases plus local-only real recordings remain prerequisites to settling the suspect reading.

#### L2-SEC-105

**Parent**: L1-SEC-005

**Statement**: The library SHALL report each nonzero secondary-header reserved field while retaining its raw evidence.

**Source**: "Reserved. These 2 bytes are reserved and shall be zero filled" (§11.2.1.2 b).

**Verification Method**: Test (T)

**Planned verification**: Set each reserved bit singly; distinguish a nonzero time-reserved word from header-reserved bytes; checksum recomputed by hand to isolate reserved-bit handling.

#### L2-SEC-106

**Parent**: L1-SEC-006

**Statement**: The library SHALL check recording-wide consistency of secondary formats without using one channel's format to decode another.

**Source**: "all channels that have a secondary header must use the same time source in bits 2-3 of the packet flags" (§11.2.1.2 a); contract 6.6.

**Verification Method**: Test (T)

**Planned verification**: Two same-format channels, each pair of different formats, absent headers, and a format change on a channel.

## CSDW

#### L2-CSDW-101

**Parent**: L1-CSDW-001

**Statement**: The library SHALL read a Format 1 data word independently from its body, retaining raw fields and diagnostics for reserved values.

**Source**: §11.2.3.2 a; Figure 11-12; Table 11-4.

**Verification Method**: Test (T)

**Planned verification**: One-bit field walking and asymmetric bytes; data type 0x11 dispatch; every truncated word length.

#### L2-CSDW-102

**Parent**: L1-CSDW-002

**Statement**: The library SHALL use the corrected shared source mapping, preserving reserved numeric values rather than presenting source 3 as GPS.

**Source**: §11.2.3.2 a, "0x3 – 0xE = Reserved"; T-6.

**Verification Method**: Test (T)

**Planned verification**: All sixteen SRC values, especially 3 and 15; T-6 regression independent of current types implementation.

#### L2-CSDW-103

**Parent**: L1-CSDW-003

**Statement**: The library SHALL expose every FMT value and prevent FMT NONE from producing a usable time reference.

**Source**: "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); T-7.

**Verification Method**: Test (T)

**Planned verification**: All sixteen FMT values crossed with SRC=0/1/3/15; NONE yields TF-003 and no default reference; reserved yields TF-006.

#### L2-CSDW-104

**Parent**: L1-CSDW-004

**Statement**: The library SHALL carry the LY bit with each Format 1 value for later checks against the resolved day and year.

**Source**: "Bit 8: Indicates if this is a leap year" (§11.2.3.2 a).

**Verification Method**: Test (T)

**Planned verification**: LY 0/1 with day 365/day 366 and known leap/common years; extraction must not depend on DATE or source bits.

#### L2-CSDW-105

**Parent**: L1-CSDW-005

**Statement**: The library SHALL select the six-byte or eight-byte Format 1 body solely by DATE bit 9.

**Source**: "0 = IRIG day available (Figure 11-13) 1 = Month and year available (Figure 11-14)" (§11.2.3.2 a).

**Verification Method**: Test (T)

**Planned verification**: Toggle bit 9 alone on hand-written 6/8-byte payloads; Data Length 10 and 12 respectively; no access to filler.

#### L2-CSDW-106

**Parent**: L1-CSDW-006

**Statement**: The library SHALL preserve raw ITS while exposing its edition- and format-qualified meaning under RDG-007.

**Source**: "IRIG Time Source (ITS). Bits 15-12 provide dynamic information regarding the source of time for an internal IRIG time code generator (TCG) when FMT is IRIG-A, B, or G" (§11.2.3.2 a); T-7; the field is reserved before 106-17 and Format 1's data type version did not change (T-14, a reading for the register).

**Verification Method**: Test (T)

**Planned verification**: Each ITS value with A/B/G and other FMT, declared 106-15/17/24R1/unknown; zero before 106-17 must never become certain freewheeling.

#### L2-CSDW-107

**Parent**: L1-CSDW-007

**Statement**: The library SHALL report reserved Format 1 bits using the recording's declared edition, separately from field-value validation.

**Source**: "All reserved bit fields in packet headers or CSDWs shall be set to zero (0x0)" (§11.2.1 f); §11.2.3.2 a; T-7.

**Verification Method**: Test (T)

**Planned verification**: Set each bit of both masks independently; no false positives on valid FMT/SRC/DATE/LY/ITS; old-edition ITS cannot be treated as a modern lock state.

## BCD

#### L2-BCD-103

**Parent**: L1-BCD-003

**Statement**: The library SHALL validate the zero positions in each date-format figure without consuming a fictitious extra reserved word.

**Source**: Figures 11-13 and 11-14; Table 11-16, "0 Always zero".

**Verification Method**: Test (T)

**Planned verification**: A fixture per figure, each always-zero bit set individually, and adjacent filler made nonzero to prove it is not parsed.

#### L2-BCD-104

**Parent**: L1-BCD-004

**Statement**: The library SHALL validate each BCD digit before combining digits, returning a field-specific error for an invalid nibble.

**Source**: "the time data words are inserted in the packet in binary-coded decimal format" (§11.2.3.2 b).

**Verification Method**: Test (T)

**Planned verification**: All 0..15 digit patterns at every digit position, separating reserved-bit violations from invalid decimal digits.

#### L2-BCD-105

**Parent**: L1-BCD-005

**Statement**: The library SHALL read and encode the day body as exactly three little-endian u16 words and preserve its 10 ms resolution.

**Source**: Figure 11-13; Table 11-16; T-12.

**Verification Method**: Test (T)

**Planned verification**: Contract section 7 bytes 35 27 45 13 87 01 decode day 187 13:45:27.350; exactly 6 bytes succeeds, shorter fails; 10 ms steps and second 60 under RDG-010.

#### L2-BCD-106

**Parent**: L1-BCD-006

**Statement**: The library SHALL read and encode the calendar body as exactly four little-endian u16 words, validating the resulting calendar date.

**Source**: Figure 11-14; Table 11-16; T-12.

**Verification Method**: Test (T)

**Planned verification**: Independently packed 2000-02-29 and 1900-02-29; exactly 8 bytes and every shorter length; December 31 rollover; no fifth word read. T-12 regression.

## F2CSDW

#### L2-F2CSDW-101

**Parent**: L1-F2CSDW-001

**Statement**: The library SHALL decode Format 2 data words independently of the Format 1 representation and distinguish network format from time validity.

**Source**: §11.2.3.3 a; Figure 11-15.

**Verification Method**: Test (T)

**Planned verification**: Data words 0x00000001, 0x00000011, 0x00000021 represent valid NTP/PTP 2002/PTP 2008; short words and T-2 regression.

#### L2-F2CSDW-104

**Parent**: L1-F2CSDW-004

**Statement**: The library SHALL dispatch NTP and the two PTP versions from NTF bits 7-4 while preserving unsupported values.

**Source**: "Network Time Format (NTF). Bits 7-4" (§11.2.3.3 a); T-2.

**Verification Method**: Test (T)

**Planned verification**: All sixteen values; vary TS independently and verify NTF=2 is accepted as PTP 2008 rather than reserved.

#### L2-F2CSDW-105

**Parent**: L1-F2CSDW-005

**Statement**: The library SHALL carry TS validity into the reference-admission decision; a reserved status is not implicitly valid.

**Source**: "Time Status (TS). Bits 3-0 indicate the status of the network time" (§11.2.3.3 a); T-2.

**Verification Method**: Test (T)

**Planned verification**: Every status with all three defined NTFs; explicit policy admission of invalid time remains labelled and cannot change its recorded status.

#### L2-F2CSDW-106

**Parent**: L1-F2CSDW-006

**Statement**: The library SHALL report each nonzero bit in the Format 2 reserved region without rejecting valid NTF bits as reserved.

**Source**: "Reserved. Bits 31-8 are reserved" (§11.2.3.3 a); §11.2.1 f; T-2.

**Verification Method**: Test (T)

**Planned verification**: Walking bits 8..31 and valid low-byte fields; NTF=1/2 must not trigger reserved-bit findings.

## NTP

#### L2-NTP-101

**Parent**: L1-NTP-001

**Statement**: The library SHALL read and write a bounded eight-byte NTP time body as seconds then fractional word.

**Source**: "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (Figure 11-16).

**Verification Method**: Test (T)

**Planned verification**: Seconds=0x01020304 and fraction=0x80000000; short 0..7 bytes, trailing bytes untouched, independently supplied wire expectations.

#### L2-NTP-102

**Parent**: L1-NTP-002

**Statement**: The library SHALL retain UTC scale and the 1900 epoch in an NTP value; expose unresolved era/leap interpretation explicitly.

**Source**: "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds" (§11.2.3.3).

**Verification Method**: Test (T)

**Planned verification**: 1900 epoch, 0xFFFFFFFF to0 era boundary, missing/provided era and a leap insertion; expected UTC/Unix mapping must be reviewed against Q-NTP before implementation.

#### L2-NTP-103

**Parent**: L1-NTP-003

**Statement**: The library SHALL offer a deterministic nanosecond view of the binary fractional word with a documented loss of precision.

**Source**: NTP version 3 (RFC 1305, cited by §11.2.3.3 a).

**Verification Method**: Test (T)

**Planned verification**: Fraction 0=0 ns, 0x80000000=500000000 ns, 0xFFFFFFFF=999999999 ns; reverse conversion is tested against representability, not assumed bit-exact.

#### L2-NTP-104

**Parent**: L1-NTP-004

**Statement**: The library SHALL convert NTP to absolute/Unix time only with sufficient epoch/era/leap context and carry conversion provenance.

**Source**: Contract 3.3.

**Verification Method**: Test (T)

**Planned verification**: Before/at/after 1970, dates beyond 2036 using explicit era, negative Unix time, unknown era, and leap-second conversion limitations.

## PTP

#### L2-PTP-102

**Parent**: L1-PTP-002

**Statement**: The library SHALL give Format 2 PTP a stated TAI scale and the standard's 1970 epoch, independent of NTP UTC.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds" (§11.2.3.3).

**Verification Method**: Test (T)

**Planned verification**: Raw epoch 0 remains TAI; values around a leap transition; NTP/PTP raw seconds with the same number must retain distinct epoch/scale metadata.

#### L2-PTP-103

**Parent**: L1-PTP-003

**Statement**: The library SHALL reject a PTP body with a nanosecond word outside a single second.

**Source**: Figure 11-17 ("Time Unsigned Nanoseconds").

**Verification Method**: Test (T)

**Planned verification**: 0, 999999999, 1000000000, and 0xFFFFFFFF; reject malformed values even if TS=valid.

#### L2-PTP-104

**Parent**: L1-PTP-004

**Statement**: The library SHALL convert TAI to UTC using the date-appropriate offset and report table currency with the answer.

**Source**: §11.2.3.3; L1-TAI.

**Verification Method**: Test (T)

**Planned verification**: A hand-constructed offset transition with before/at/after cases; historical offsets, stale table, and an insertion second that requires a leap label.

#### L2-PTP-105

**Parent**: L1-PTP-005

**Statement**: The library SHALL read and encode the IRIG Format 2 PTP body as eight bytes, separate from general IEEE 1588 network-message layouts.

**Source**: "Time Unsigned Seconds", "Time Unsigned Nanoseconds" (Figure 11-17; the same in 106-17 and 106-19); T-3.

**Verification Method**: Test (T)

**Planned verification**: Distinct 32-bit words, max seconds, short 0..7 bytes, exactly 8 bytes with two nonzero following filler bytes; T-3 regression.

## F2COR

#### L2-F2COR-101

**Parent**: L1-F2COR-001

**Statement**: The library SHALL build Format 2 references using the packet header RTC as the capture point and the decoded body as absolute time.

**Source**: "The Format 2 Network Time packet data type is used to extract and encapsulate absolute time from NTP or IEEE-1588 PTP and time tag it with the RTC" (§11.2.3.3).

**Verification Method**: Test (T)

**Planned verification**: Mixed Format 1/2 timeline, valid/invalid TS, each NTF, and absent declarations; no use of packet arrival wall-clock time.

#### L2-F2COR-102

**Parent**: L1-F2COR-002

**Statement**: The library SHALL prevent arithmetic between differently scaled references unless an explicit conversion is recorded.

**Source**: §11.2.3.3; contract 4.1 rule 4.

**Verification Method**: Test (T)

**Planned verification**: Same physical instant expressed as NTP UTC and PTP TAI, GPS versus UTC, unknown scale, and a stale-table conversion; equal raw seconds are not equal instants by default.

## TAI

#### L2-TAI-101

**Parent**: L1-TAI-001

**Statement**: The library SHALL represent leap-table transitions as dated, ordered offset records with explicit coverage information.

**Source**: L1-PTP-004.

**Verification Method**: Test (T)

**Planned verification**: Empty table, unordered entries, equal-date conflicting offsets, before-first and exact-boundary lookups.

#### L2-TAI-102

**Parent**: L1-TAI-002

**Statement**: The library SHALL keep a release-reviewed built-in table and its known-current-through date as separate data from the last offset change.

**Source**: Contract 6.8.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: table/source diff and release checklist, including a period with no new leap transition; no live network fetch inside the library.

#### L2-TAI-103

**Parent**: L1-TAI-003

**Statement**: The library SHALL allow a caller-supplied table to replace or extend built-in data through the same validation path.

**Source**: Contract 6.8.

**Verification Method**: Test (T)

**Planned verification**: Valid extension, replacement, conflicting boundary, unsorted extension, and proof that previously issued answers retain the table provenance they used.

#### L2-TAI-104

**Parent**: L1-TAI-004

**Statement**: The library SHALL label conversions later than the table's verified coverage date while returning the last known offset when available.

**Source**: "The leap-second table does not reach the date … the conversion is given with the last known offset and labelled as possibly out of date" (contract 6.8).

**Verification Method**: Test (T)

**Planned verification**: Exact coverage limit vs one instant after, unchanged offset over years, before first supported date, explicit caller extension updating currency.

#### L2-TAI-105

**Parent**: L1-TAI-005

**Statement**: The library SHALL assign scale and stated/assumed provenance before a time packet becomes a reference.

**Source**: "0x4 = Universal Coordinated Time (UTC) time from GPS", "0x5 = Native GPS Time" (§11.2.3.2 a); §11.2.3.3; "GPS time does not add or subtract leap seconds, and as of this writing, GPS time is 16 seconds ahead of UTC" (RCC 200-16 Appendix A.2, when TAI − UTC was 35 s); "All Department of Defense (DoD) test ranges … maintain Coordinated Universal Time (UTC)" (RCC 200-16 §1); T-17; RDG-009.

**Verification Method**: Test (T)

**Planned verification**: Every Format 1 FMT and Format 2 NTF, IRIG override/unknown, GPS-to-TAI+19 s and UTC via a dated table; native GPS must not be labelled UTC without conversion (T-17).

#### L2-TAI-106

**Parent**: L1-TAI-006

**Statement**: The library SHALL require a caller-supplied scale assumption to convert unknown-scale values.

**Source**: FMT "0x3 = Real-Time Clock" (§11.2.3.2 a) states no scale; contract 6.8; RDG-009.

**Verification Method**: Test (T)

**Planned verification**: FMT 3 with no override, UTC/TAI/GPS overrides, external/internal SRC independent of scale, and serde round trips of assumption provenance.

#### L2-TAI-107

**Parent**: L1-TAI-007

**Statement**: The library SHALL accept UTC second 60 only at the dates and times allowed by the accepted RDG-010 reading and report it without normalization.

**Source**: RCC 200-16 §3.6 and Appendix A.2 ("time changes are made on December 31 and on June 30 at 2400 hours"); Figure 11-13 (tens of seconds in three bits); T-16; RDG-010.

**Verification Method**: Test (T)

**Planned verification**: Leap/common-year June 30 and December 31, neighbouring days, other hours/minutes, known-year impossible day 366, each scale, and retained second 60 after serialization. T-16 regression.

## WORD

#### L2-WORD-101

**Parent**: L1-WORD-001

**Statement**: The library SHALL accept already extracted high/low/microsecond PCM words with an explicit binary-or-BCD weighting and reconstruction context.

**Source**: "High and low order time words shall be binary or binary coded decimal (BCD) weighted, and microsecond words shall be binary weighted"; resolutions of 1 µs (to 9999), 10 ms, and 655.36 s binary or one minute BCD; "For BCD, the days field shall contain the three least significant bits of the BCD Julian date" (Chapter 4 §4.7); `C-d\PTM`, `C-d\BTM` (Chapter 9); contract 3.8. The L1 page-check prerequisite is now recorded in STANDARD-REVIEW T-10 (2026-11-12).

**Verification Method**: Test (T)

**Planned verification**: One fixture per binary and BCD weighting, each bit group distinguished, decimal digit>9, day fragment collisions, 9999/10000 microseconds, and absent origin/day context returning partial time.

#### L2-WORD-102

**Parent**: L1-WORD-002

**Statement**: The library SHALL represent full network-time words separately from component-only seconds/fractions/nanoseconds, requiring context for missing components.

**Source**: `C-d\NTM`: "PTP Time (64-bit)", "PTP Seconds (32-bit)", "PTP Nanoseconds (32-bit)", "NTP Time (64-bit)", "NTP Seconds (32-bit)", "NTP Fractions (32-bit)" (Chapter 9, Table 9-11); contract 3.8.

**Verification Method**: Test (T)

**Planned verification**: All six Table 9-11 kinds, full-vs-component equivalence with explicit context, missing seconds, conflicting epoch/scale, and invalid PTP nanoseconds.

## ENC

#### L2-ENC-101

**Parent**: L1-ENC-001

**Statement**: The library SHALL encode validated Format 1 fields into a four-byte word and the selected six/eight-byte body, clearing all reserved/always-zero bits.

**Source**: §11.2.3.2; §11.2.1 f; contract 2.2, 4.11.

**Verification Method**: Test (T)

**Planned verification**: Golden bytes from both figures and contract 7, NONE, reserved enum handling, nonrepresentable sub-10-ms time (explicit precision error, not silent truncation), and year/day bounds.

#### L2-ENC-102

**Parent**: L1-ENC-002

**Statement**: The library SHALL encode Format 2 data words and bodies without confusing status, format, epochs, or byte order.

**Source**: §11.2.3.3; §11.2.1 f; contract 2.2, 4.11.

**Verification Method**: Test (T)

**Planned verification**: NTF 0/1/2 with TS 0/1 expected words, independent NTP/PTP bytes, high 32-bit seconds overflow, and fraction preservation.

#### L2-ENC-103

**Parent**: L1-ENC-003

**Statement**: The library SHALL encode a secondary header or a stamp using the same validated flags/format model and the documented checksum reading.

**Source**: §11.2.1.1 g, §11.2.1.2, §11.2.1.3 b.

**Verification Method**: Test (T)

**Planned verification**: Golden header for all three formats, RTC-form stamp, invalid flags, byte-vs-word checksum distinction, and no writes past provided output capacity.

## EDN

#### L2-EDN-101

**Parent**: L1-EDN-001

**Statement**: The library SHALL gate the edition diagnostic for Format 2 at 106-17 rather than 106-22, without discarding the packet's raw evidence.

**Source**: 106-17 Chapter 11 lists "0x12 Time Data, Format 2 Network Time" with data type version `0x08` (106-17); 106-13 and 106-15 do not define it; T-4.

**Verification Method**: Test (T)

**Planned verification**: Declared 106-15/17/19/22/24R1 and unknown; Format 2 data type version 0x08; T-4 regression.

#### L2-EDN-102

**Parent**: L1-EDN-002

**Statement**: The library SHALL keep setup-record RCCVER and packet data-type-version values in distinct typed domains using corrected irig 106-types mappings.

**Source**: Table 11-4 and §11.2.1.1 e ("0x0A = RCC 106-22"); Figure 11-34 ("0x0E = RCC 106-22", "0x0F through 0xFF = Reserved"); T-9; ADR-0015.

**Verification Method**: Test (T)

**Planned verification**: All assigned/reserved shared codes in both lists, especially 0x0A/0x0D/0x0E/0x0F, preserving raw unknown codes. T-9 regression.

#### L2-EDN-103

**Parent**: L1-EDN-003

**Statement**: The library SHALL select explicitly cited edition differences without extrapolating missing archive evidence into claims of verified legacy support.

**Source**: "References to RCC 106-04 through RCC 106-15 refer to Chapter 10, while RCC 106-17 onward refer to Chapter 11" (§11.2.1.1 e); ADR-0004.

**Verification Method**: Test (T)

**Planned verification**: An independent fixture per available edition touched by a reading, unknown edition/code, pre-07 declared edition absent, and source-backed exceptions. Missing 106-04 evidence stays an explicit validation gap.

## DECL

#### L2-DECL-101

**Parent**: L1-DECL-001

**Statement**: The library SHALL receive setup-record declarations as plain structured values associated with a governing range and provenance.

**Source**: Contract 2.3 C, 4.5, 5.3; `irig106-tmats` L1-CH10-008 (the configuration timeline); ADR-0005.

**Verification Method**: Test (T)

**Planned verification**: Two configuration ranges in one session, exact change boundary, absent fields, multiple time channels, and original recording date distinct from TMATS origination date.

#### L2-DECL-102

**Parent**: L1-DECL-002

**Statement**: The library SHALL allow declaration-free operation while keeping its reduced basis visible and default reference admission explicit.

**Source**: A recording's setup record can be missing or unreadable; contract 6.4.

**Verification Method**: Test (T)

**Planned verification**: No setup record with default policy gives counter-only; explicit undeclared-admission yields labelled absolute answers; present-empty declarations remain distinguishable.

## COR

#### L2-COR-101

**Parent**: L1-COR-001

**Statement**: The library SHALL build a timeline through an explicit ordered input API for declarations, time packets, and other packet metadata, returning sessions and evidenced references.

**Source**: "Time is treated like another data channel" (§11.2.3.2); "Since Time Data, Format 1 packets contain both the absolute input time value and the RTC clock value at the instant the absolute time was valid, these packets can be used to relate RTC values to the input absolute time source" (RCC 123-20 §5.6); contract 5.5.

**Verification Method**: Test (T)

**Planned verification**: Contract 7 recording, malformed time packet between valid references, empty recording, setup changes, multiple channels, and repeated finalization/query semantics documented.

#### L2-COR-102

**Parent**: L1-COR-002

**Statement**: The library SHALL keep whole-recording nearest-reference lookup distinct from streaming preceding/interpolating modes; make ties deterministic and record the selection rule.

**Source**: "It is better to use the clock and relative time values from a time packet that occurs near the current data packet as the data file is decoded since there is some drift in the RTC during a recording session" (RCC 123-09 §6.6; T-13); contract 5.2 rule 5; a policy setting (L1-POL-001).

**Verification Method**: Test (T)

**Planned verification**: Exact reference, midpoint tie, before-first default/override, beyond-last, wrap, and no reference from another session or channel. Compare streaming only with the same reference strategy, not offline nearest by accident.

#### L2-COR-103

**Parent**: L1-COR-003

**Statement**: The library SHALL select one eligible time channel per session using explicit caller choice, then declared-external preference, then valid-reference count, then channel ID.

**Source**: "When multiple time channels are available, it is incumbent on the programmer or data analyst to determine and select the best source of time for a particular data set" (RCC 123-20 §5.6); "It is usually most correct to select one time channel only and to use this channel exclusively" (RCC 123-09 §6.6); contract 5.2 rule 4; a policy setting.

**Verification Method**: Test (T)

**Planned verification**: Every ranking branch, equal counts, multiple external channels, unavailable explicit choice, configuration changes, and lock loss not causing automatic channel mixing.

#### L2-COR-104

**Parent**: L1-COR-004

**Statement**: The library SHALL measure jumps from consecutive same-channel references using the configured threshold and compatible scales.

**Source**: "there is a jump in input clock time during a recording, such as when GPS locks for the first time, or when an IRIG time source is reprogrammed" (RCC 123-09 §6.6; T-13); contract 5.4; default threshold the format's resolution (L1-POL-001).

**Verification Method**: Test (T)

**Planned verification**: Threshold-1, exact threshold and threshold+1; positive/negative jumps, lock regain, no false jump at RTC wrap, different scales requiring conversion, and unknown scale unresolved.

#### L2-COR-105

**Parent**: L1-COR-005

**Statement**: The library SHALL separate counter resets from wrap and bounded reordering before assigning a session; references cannot cross sessions.

**Source**: The counter "shall remain free-running during each session (e.g., recording)" (§11.2.1.1 i); contract 5.2 rule 1.

**Verification Method**: Test (T)

**Planned verification**: Forward wrap near 2^48, small late packet, backwards reset beyond 1100 ms, repeated raw ranges in distinct sessions, and an ambiguous large displacement requiring caller context.

#### L2-COR-106

**Parent**: L1-COR-006

**Statement**: The library SHALL apply reference eligibility as a recorded stage controlled by governing declarations and validity settings.

**Source**: Contract 5.2 rules 2 and 3; "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); "0x0 = Time Not Valid" (§11.2.3.3 a); policy settings.

**Verification Method**: Test (T)

**Planned verification**: All combinations of declaration presence, TIMEIN, Format 1 NONE, Format 2 TS, and policy overrides; raw invalid values remain visible without default admission.

#### L2-COR-107

**Parent**: L1-COR-007

**Statement**: The library SHALL keep references through source transitions by default while reporting source-change information, and offer the configured stop-on-loss option.

**Source**: "If the time source is external (0x1) and lock on the external source is lost then the time source shall indicate Internal (0x0). Once lock on the external time source is regained, time source shall once again indicate external (0x1)" (§11.2.3.2 a); contract 5.2 rule 6.

**Verification Method**: Test (T)

**Planned verification**: External->internal->external and RMM/None changes, stop-on-loss policy, lock return with jump, and packets retaining original reported source.

#### L2-COR-108

**Parent**: L1-COR-008

**Statement**: The library SHALL report a reference gap using the chosen channel's valid reference spacing and a configurable tolerance, retaining raw-network exceptions as policy evidence.

**Source**: "the time packet will be generated at a minimum frequency of 1 hertz" (§11.2.3.2; Chapter 10 §10.6.2); the Format 2 exception "unless it is recorded at the raw network rate" (§11.2.3.3); contract 5.4.

**Verification Method**: Test (T)

**Planned verification**: Exactly 1 s, plus one RTC tick, configured tolerance boundary, intervening invalid references, and raw-network Format 2 exception with rate metadata supplied by caller.

#### L2-COR-109

**Parent**: L1-COR-009

**Statement**: The library SHALL use 1100 ms as the default late-packet bound, exposing overrides and the distinction between a reset inference and a late arrival.

**Source**: "all other packet generation times shall be equal to or less than 100 milliseconds (ms)"; "all other packets shall have a stream commit time equal to or less than 1000 ms" (Chapter 10 §10.6.1 b–c); contract 5.4.

**Verification Method**: Test (T)

**Planned verification**: Offsets 1099.9999/1100/1100.0001 ms, consecutive late packets, caller session boundary and bound override, pre-05 unknown ordering evidence, and forward wrap.

#### L2-COR-110

**Parent**: L1-COR-010

**Statement**: The library SHALL resolve and report year provenance in the configured precedence, checking recorded LY and advancing calendar years at rollover.

**Source**: Figures 11-13 and 11-14; "Date and time original recording was created" (`R-x\RI4`, Chapter 9); contract 5.3.

**Verification Method**: Test (T)

**Planned verification**: Each precedence branch, all sources absent, contradictory years, leap/common year, setup changes, and December 31->January 1 with known and unknown year.

#### L2-COR-111

**Parent**: L1-COR-011

**Statement**: The library SHALL return session spans and all matching counter ranges for absolute-time spans, preserving discontinuities and uncertainty.

**Source**: Contract 3.6, 4.9, 5.5.

**Verification Method**: Test (T)

**Planned verification**: Monotonic span, backwards/forwards clock jump with multiple/no matches, multiple sessions, inclusive boundary convention documented, unknown year/scale, and a request outside coverage.

#### L2-COR-112

**Parent**: L1-COR-012

**Statement**: The library SHALL check the first dynamic packet rule for each inferred/caller-specified session, treating a None-source time packet as present but not as usable absolute time.

**Source**: "A time data packet shall be the first dynamic data packet at the start of each session. Only static Computer-Generated Data, Format 1 packets may precede the first time data packet" (§11.2.3.2; "of each recording", Chapter 10 §10.6.2); "If the time data packet source is None, at least one time data packet is required" (§11.2.3.2); contract 6.1, 6.2.

**Verification Method**: Test (T)

**Planned verification**: Static TMATS then time then data, data first, no time, time-source None first, malformed time first, and second session after RTC reset (RDG-011).

#### L2-COR-113

**Parent**: L1-COR-013

**Statement**: The library SHALL compute per-channel reference quality from explicitly identified eligible references and expose undefined measurements rather than NaN-like guesses.

**Source**: Contract 3.5; the prototype's `compute_quality`.

**Verification Method**: Test (T)

**Planned verification**: 0/1/2 references, irregular spacing, duplicates, exact 10 MHz=0ppm, known positive/negative drift, invalid reference exclusion, and separate channels.

## ANS

#### L2-ANS-101

**Parent**: L1-ANS-001

**Statement**: The library SHALL return an answer record with absolute-or-counter-only outcome, complete provenance, and both contributing references when interpolation was used.

**Source**: Contract 3.1 (with the precision of a reference: "consistent with the resolution with the time packet body format (10 milliseconds [ms] as measured by the 10-MHz RTC)", §11.2.3.2), 3.2, 3.5, 7.3; ADR-0011.

**Verification Method**: Test (T)

**Planned verification**: Contract 7 full expected basis, nearest/preceding/interpolated/extrapolated cases, unknown year/scale, weak references, and serialization preserving all provenance.

#### L2-ANS-102

**Parent**: L1-ANS-002

**Statement**: The library SHALL expose validated own time beside counter-derived time and compare them only with compatible epoch and scale context.

**Source**: "The applicable data bit to which the 48-bit value of the packet secondary time value (if enabled) applies shall correspond to the first bit of the data in the packet body" (§11.2.1.2 a); contract 6.6.

**Verification Method**: Test (T)

**Planned verification**: Equal times, at/above tolerance, invalid checksum, ERTC without reference, partial Chapter 4 context, and incompatible/unknown scales.

#### L2-ANS-103

**Parent**: L1-ANS-003

**Statement**: The library SHALL resolve a stamp through validated flag dispatch and return the same answer/basis model used for packet counters.

**Source**: §11.2.1.3 b; contract 3.4, 4.6.

**Verification Method**: Test (T)

**Planned verification**: Each supported stamp form, no reference, inconsistent flags, reserved format, RTC wrap, and secondary format different from a previous packet.

#### L2-ANS-104

**Parent**: L1-ANS-004

**Statement**: The library SHALL resolve caller-supplied time words without reading the containing payload, retaining weighting, reconstruction context, and origin in the answer.

**Source**: Contract 3.8, 4.6.

**Verification Method**: Test (T)

**Planned verification**: Binary/BCD/network kinds, full and component-only words, missing calendar context, explicit matching context, and a decoder-supplied packet/channel identity.

#### L2-ANS-105

**Parent**: L1-ANS-005

**Statement**: The library SHALL for a session lacking a usable reference, return elapsed counter time from its first packet with no absolute timestamp.

**Source**: Contract 6.1, 6.2.

**Verification Method**: Test (T)

**Planned verification**: No time packets, all invalid, undeclared default rejection, data before first reference, and first packet near RTC wrap.

## POL

#### L2-POL-101

**Parent**: L1-POL-001

**Statement**: The library SHALL represent every contract 5.6 setting as a documented policy field, adding the owner's switchable stream mode and a proposed default.

**Source**: Contract 5.6; ADR-0010.

**Verification Method**: Test (T)

**Planned verification**: A default snapshot and a test per field; defaults must match contract 5.6 and owner 2026-11-11 exceptions; no accepted default implied for Q-DEFAULT.

#### L2-POL-102

**Parent**: L1-POL-002

**Statement**: The library SHALL make every policy setting caller-selectable with validation and an explicit effective point for streaming changes.

**Source**: "I would like for it to be selectable as well" (owner, 2026-09-27); ADR-0010.

**Verification Method**: Test (T)

**Planned verification**: Change each setting from default; invalid bounds, standard->interpolate->standard with queued packets, and no mutation of already issued answers.

#### L2-POL-103

**Parent**: L1-POL-003

**Statement**: The library SHALL record enough policy information to reproduce each answer and timeline, including explicit non-default assumptions.

**Source**: ADR-0010.

**Verification Method**: Test (T)

**Planned verification**: Same bytes+same policy deterministic result; different policies distinguishable; serialized delta cannot silently adopt changed future defaults.

#### L2-POL-201

**Parent**: L1-POL-001

**Statement**: The library SHALL propose standard as the stream default for owner decision, independently of the accepted offline-nearest default.

**Source**: Owner2026-11-11 ROADMAP question2; RCC 123-20 §5.6; 106-24R1 Chapter 10 §10.6.2 only supplies cadence/first-packet constraints.

**Verification Method**: Test (T)

**Planned verification**: Default-mode acceptance test remains pending owner selection; both explicitly selected modes and serialized policy distinguishable regardless of default.

## FND

#### L2-FND-101

**Parent**: L1-FND-001

**Statement**: The library SHALL use the owner-accepted TF-001..TF-027 identities and severity defaults with structured evidence; caller severity overrides do not alter factual validity.

**Source**: Contract section 6; ADR-0013.

**Verification Method**: Test (T)

**Planned verification**: Each ID and severity override, absent offset, serialization, duplicate findings with distinct evidence, and an immutable identifier list matching architecture 8.

#### L2-FND-102

**Parent**: L1-FND-002

**Statement**: The library SHALL cover each contract 6 degraded case with the accepted architecture 8 finding and a documented answer effect.

**Source**: Contract 6.1 to 6.9.

**Verification Method**: Test (T)

**Planned verification**: At least one synthesized case per TF identifier, including simultaneous causes; exact error/warning/information defaults from architecture 8 without renaming or changing owner choices.

#### L2-FND-103

**Parent**: L1-FND-003

**Statement**: The library SHALL report flags bit 5 as an RTC-sync finding without confusing it with secondary presence or stamp source.

**Source**: "Bit 5: RTC sync error. 0 = No RTC sync error. 1 = RTC sync error has occurred" (§11.2.1.1 g); found while writing the architecture (2026-09-27), not yet in the contract.

**Verification Method**: Test (T)

**Planned verification**: Bit 5 alone, alongside each valid flag format, and alongside invalid bit 6/7 combinations; all other flag bits varied.

#### L2-FND-104

**Parent**: L1-FND-004

**Statement**: The library SHALL compare declarations and packets only using RDG-001..006 mappings, keeping source changes informational and inferred I mappings labelled.

**Source**: Contract 3.7, 6.4; the source may change legitimately (§11.2.3.2 a).

**Verification Method**: Test (T)

**Planned verification**: Each declared/observed mapping and mismatch, missing fields, configuration boundary, I inference warning, and external->internal loss of lock information.

#### L2-FND-105

**Parent**: L1-FND-005

**Statement**: The library SHALL check same-packet RTC/ERTC consistency using integer division and the L1 one-tick tolerance.

**Source**: "the 10-megahertz (MHz) RTC shall be synchronized with the ERTC (RTC = ERTC/100)" (§11.2.1.1 g).

**Verification Method**: Test (T)

**Planned verification**: ERTC residual 0/99ns, expected RTC wrap, delta-2/-1/0/1/2 ticks, and a packet with no ERTC where no comparison is made.

## RDG

#### L2-RDG-101

**Parent**: L1-RDG-001

**Statement**: The project SHALL centralize declaration/packet comparisons with permanent reading identifiers, including explicit inference provenance for RDG-003.

**Source**: Contract 3.7; T-14; ADR-0012.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: audit all comparison sites and their RDG links, including numeric Format 2 and SHTF readings; unknown letters/numbers must not match by fallback.

#### L2-RDG-102

**Parent**: L1-RDG-002

**Statement**: The project SHALL keep written analysis, edge-case tests and all-edition fixtures as prerequisites to implementing a reading; owner Design acceptance does not satisfy human Review.

**Source**: ADR-0012; `docs/INTERPRETATIONS.md`, "Development" ("intensive testing and deep analysis required", owner, 2026-09-26).

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: register-to-analysis-to-tests audit, including RDG-008 suspect dual checksum and unresolved human reviews. Do not mark page checks as the complete analysis.

#### L2-RDG-201

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-001: TTF maps 1->0x11 and 2->0x12; reserved 0 and mismatches are reported.

**Source**: 106-24R1 Chapter 9 Table 9-4 and Chapter 11 Table 11-4; RDG-001.

**Verification Method**: Inspection (I)

**Planned verification**: Every defined/reserved TTF, both packet types, missing TTF, non-TIMEIN and configuration boundaries.

#### L2-RDG-202

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-002: TFMT Format 1 named letters are compared through the accepted RDG-002 mapping; retain the page-confirmed A default separately from a missing explicit value.

**Source**: 106-24R1 Chapter 9 Table 9-4, printed9-72/PDF78; Chapter 11 §11.2.3.2 a; RDG-002.

**Verification Method**: Inspection (I)

**Planned verification**: Each pair in both directions, all mismatches, missing/unknown and a non-TIMEIN channel. Cite Table 9-4 printed 9-72/PDF 78 visual check.

#### L2-RDG-203

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-003: Require tests for every TFMT letter/FMT pair, including the accepted inferred I->0x3 pair and the RCC 124-22 Chapter 7 example.

**Source**: Owner2026-11-11 ROADMAP question1; 106-24R1 Table 9-4 and§11.2.3.2 a; RCC 124-22 Chapter7 printed7-3/PDF149; RDG-002/003.

**Verification Method**: Inspection (I)

**Planned verification**: For each of7 letters, test matching FMT and every other nibble, reverse comparisons, missing/unknown values. Reproduce RCC 124-22 printed 7-3/PDF 149 R-1\TTF-1:1;, R-1\TFMT-1:I;, R-1\TSRC-1:I; ("Time format is internal", "Time source is internal"); compare FMT 3/SRC 0 and contradictory values.

#### L2-RDG-204

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-004: Compare Format 2 numeric TFMT with NTF without routing it through Format 1 letter mappings.

**Source**: 106-24R1 Chapter 9 Table 9-4; Chapter 11 §11.2.3.3 a; RDG-004.

**Verification Method**: Inspection (I)

**Planned verification**: All 3 matching values, every pairwise mismatch, reserved NTF and numeric TFMT under an inconsistent TTF.

#### L2-RDG-205

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-005: Map TSRC I/E/R/X to SRC 0/1/2/15 and report source differences as information.

**Source**: 106-24R1 Chapter 9 Table 9-4; Chapter 11 §11.2.3.2 a; RDG-005.

**Verification Method**: Inspection (I)

**Planned verification**: Each matching pair and mismatch, loss/regain sequence, missing TSRC and reserved SRC 3..14; information default unchanged.

#### L2-RDG-206

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-006: Map SHTF 0/1/2 to flag formats 00/01/10, retaining the Chapter 9 wording discrepancy.

**Source**: 106-24R1 Chapter 9 Table 9-4; Chapter 11 §11.2.1.1 g; Chapter 4 §4.7; RDG-006.

**Verification Method**: Inspection (I)

**Planned verification**: Every pair, mixed-channel values, missing SHTF, format 11 and BCD-looking bytes that must not change packet dispatch.

#### L2-RDG-207

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-007: Gate ITS interpretation by declared recording edition and FMT, not Format 1 data-type version alone.

**Source**: 106-24R1 Chapter 11 §11.2.3.2 a/Table 11-4/Figure 11-34; 106-05/07/13/15 Chapter 10 reserved31..12; RDG-007.

**Verification Method**: Inspection (I)

**Planned verification**: Fixtures per touched archived edition:106-05/07/13/15/17 and current; every ITS value, non-IRIG FMT and missing edition. Record unavailable 106-04 evidence explicitly.

#### L2-RDG-208

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-008: Retain the suspect byte-checksum reading and diagnose word-only-valid headers until local recordings settle it.

**Source**: 106-24R1 §11.2.1.2 c and§11.2.1.1 j; RCC 123-20 AppendixA-2; RDG-008.

**Verification Method**: Inspection (I)

**Planned verification**: Byte-only, word-only, both, neither across the listed editions; reference handbook AppendixA-2 code; local recordings remain outside Git/CI and human Review pending.

#### L2-RDG-209

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-009: Assign stated and assumed scales separately, with GPS=TAI-19s as a recorded inference.

**Source**: 106-24R1 §11.2.3.2 a/§11.2.3.3; RCC 200-16 §1/AppendixA.2; RDG-009.

**Verification Method**: Inspection (I)

**Planned verification**: All FMT/NTF values, IRIG/RTC overrides, historical table transitions, cross-scale reference comparisons and stale table.

#### L2-RDG-210

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-010: Pin the accepted UTC leap-second reading with boundary and scale tests rather than accepting second 60 everywhere.

**Source**: 106-24R1 Figure 11-13; RCC 200-16 §3.6/AppendixA.2; RDG-010.

**Verification Method**: Inspection (I)

**Planned verification**: Known/unknown year, June 30/December 31 candidates and neighbours,23:59 vsother times, UTC/assumed UTC/TAI/GPS/unknown.

#### L2-RDG-211

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-011: Check first dynamic time packet per session as the accepted stricter reading of the recording/session wordings.

**Source**: 106-24R1 §11.2.3.2/§11.2.3.3, Chapter 10 §10.6.2, §11.2.1.1 i; RDG-011.

**Verification Method**: Inspection (I)

**Planned verification**: Two sessions each compliant, missing time insecond, static-only prefix, time None and malformed first time packet.

## STRM

#### L2-STRM-101

**Parent**: L1-STRM-001

**Statement**: The library SHALL provide bounded streaming with two switchable reference strategies and the same answer/policy model as batch queries evaluated with the same available inputs.

**Source**: Contract 4.8; architecture section 6.

**Verification Method**: Test (T)

**Planned verification**: Long synthesized stream at fixed input rate has bounded retained state; compare answers to batch with identical strategy and input prefix, mode changes, absent time channel, and deterministic replay without wall clock.

#### L2-STRM-201

**Parent**: L1-STRM-001

**Statement**: The library SHALL in standard mode, answer from the latest eligible reference of the selected channel at or before the query counter, without waiting for a future reference.

**Source**: Owner 2026-11-11, ROADMAP question2; RCC 123-20 §5.6, printed5-44/PDF78. This is handbook-supported product policy, not an IRIG-mandated algorithm.

**Verification Method**: Test (T)

**Planned verification**: RCC 123-20 example: RTC 1000000 at day 100 12:30:25.000; RTC 1150000 gives 12:30:25.015. Exact reference, future-only reference, and intervening other-channel references.

#### L2-STRM-202

**Parent**: L1-STRM-001

**Statement**: The library SHALL in interpolate mode, hold a packet for the next eligible reference of its selected channel and interpolate between the bracketing references to correct drift.

**Source**: Owner2026-11-11, ROADMAP question2; 106-24R1 Chapter 10 §10.6.2 (rate bound); architecture 6. Interpolation and its rounding are product policy, not standard wording.

**Verification Method**: Test (T)

**Planned verification**: A 1 s RTC interval with absolute interval 1.000010s gives 0.500005s at midpoint; exact endpoints, non-integral result, RTC wrap, equal endpoint counters and incompatible scales.

#### L2-STRM-203

**Parent**: L1-STRM-001

**Statement**: The library SHALL bound interpolation hold by the 1 Hz reference interval plus a caller-visible tolerance; timeout or end-of-recording falls back to the preceding reference, labelled extrapolated.

**Source**: Owner2026-11-11 ROADMAP question2; 106-24R1 Chapter 10 §10.6.2; Chapter 11 §11.2.3.3 raw-network exception; RCC 123-20 §5.6 supports preceding-reference fallback.

**Verification Method**: Test (T)

**Planned verification**: Reference at 1 s/just-after 1 s/at deadline/after deadline, no next reference, EOF before deadline, another channel arriving, and stalled progress. Owner fallback must be visible in basis.

#### L2-STRM-204

**Parent**: L1-STRM-001

**Statement**: The library SHALL in both modes, packets before any usable preceding time reference get no absolute time; time source None supplies no absolute reference.

**Source**: Owner2026-11-11 ROADMAP question2; 106-24R1 Chapter 10 §10.6.2 first dynamic packet; Chapter 11 §11.2.3.2 source None/invalid FMT. Missing-time fallback is product policy; the standard gives no such algorithm.

**Verification Method**: Test (T)

**Planned verification**: Data-before-time, first time packet SRC None with a valid-looking body, FMT NONE, TS invalid, no time until EOF, and usable reference arriving later must not retroactively invent an early absolute answer.

#### L2-STRM-205

**Parent**: L1-STRM-001

**Statement**: The library SHALL bound retained stream state with explicit caller limits and drain/reset behaviour that never bridges sessions or clock discontinuities.

**Source**: Architecture6; ADR-0010/0011/0014; L1-STRM-001. Resource caps and discontinuity handling are proposed product rules; 106-24R1 §11.2.1.1 i supplies the session-continuity constraint.

**Verification Method**: Test (T)

**Planned verification**: Fixed-rate long input, pathological data burst without reference, per-byte/per-packet limits, reference flood, duplicate counters, jump/reset during hold, and bounded exported findings with caller consumption.

#### L2-STRM-206

**Parent**: L1-STRM-001

**Statement**: The library SHALL make switching modes explicit, preserving the policy and fallback reason attached to every released answer.

**Source**: Owner2026-11-11 ROADMAP question2 (both switchable); ADR-0010/0011; architecture 6. Transition flushing is a proposed implementation rule.

**Verification Method**: Test (T)

**Planned verification**: Switch with empty/nonempty pending state, standard->interpolate->standard, two configurations, and reconstruction of each result from its retained policy snapshot.

## ERR

#### L2-ERR-101

**Parent**: L1-ERR-001

**Statement**: The library SHALL return structured errors on unreadable bytes without panics, unbounded parser loops or input-controlled allocation in value readers.

**Source**: Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: All shorter lengths, arbitrary bytes per parser, arithmetic extremes, malformed field combinations, and repeated input; no catch-and-hide panic as parsing strategy.

#### L2-ERR-102

**Parent**: L1-ERR-002

**Statement**: The library SHALL name the invalid decimal field and digit position in BCD errors while retaining packet context at the timeline boundary.

**Source**: §11.2.3.2 b.

**Verification Method**: Test (T)

**Planned verification**: DigitA..F at each BCD position; invalid hour 24 with valid digits yields range error instead; zero-mask violations produce their own finding.

#### L2-ERR-103

**Parent**: L1-ERR-003

**Statement**: The library SHALL apply field and calendar range validation before constructing time values, with the RDG-010 exception explicitly isolated.

**Source**: Figures 11-13, 11-14; RDG-010.3); second 60 is no longer always rejected (T-16).

**Verification Method**: Test (T)

**Planned verification**: Every minimum/maximum and one out-of-range value, Feb 29 in leap/common years, day 0, second 60 combinations, and no silent day 0->1 repair.

#### L2-ERR-104

**Parent**: L1-ERR-004

**Statement**: The library SHALL surface secondary checksum failure as a typed validation result and a finding when attached to a recording.

**Source**: §11.2.1.2 c.

**Verification Method**: Test (T)

**Planned verification**: Stored checksum corruption in either byte, byte-only/word-only cases, and counter-derived answer remaining available without claiming own time valid.

#### L2-ERR-105

**Parent**: L1-ERR-005

**Statement**: The library SHALL keep all time parsing inside caller-supplied Data Length with explicit consumed sizes and no access to trailer filler/checksum.

**Source**: "Data Length … does not include packet trailer filler and data checksum" (§11.2.1.1 d); T-12.

**Verification Method**: Test (T)

**Planned verification**: Data Length 10/12 for Format 1 and 12 for Format 2, every smaller length, nonzero filler after the boundary, and a guard slice showing no overread. T-12 fixture.

## API

#### L2-API-101

**Parent**: L1-API-001

**Statement**: The library SHALL apply the public-value derive contract to shared adapters, calendar values, policy snapshots, answers, findings and timeline records where their contents permit it.

**Source**: Prototype identifier kept.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: public-type inventory with derive assertions, including feature-gated types and corrected shared types; absence of Copy on owning collections justified.

#### L2-API-102

**Parent**: L1-API-002

**Statement**: The library SHALL separate allocation-free value operations from alloc-backed timeline/stream state while retaining a no_std library build.

**Source**: ADR-0014.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: no-default host and no_std/WASM builds; review allocation paths in each value reader, and default/no-default API signature comparison.

#### L2-API-104

**Parent**: L1-API-004

**Statement**: The library SHALL use distinct shared quantity types at public boundaries and explicit conversion helpers for unit/epoch changes.

**Source**: Prototype identifier kept; `irig106-types` newtypes.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: signature inventory and compile-fail examples for mixing RTC ticks with nanos or TAI with Unix seconds; explicit successful conversions.

#### L2-API-105

**Parent**: L1-API-005

**Statement**: The library SHALL keep irig 106-types as the library's only ecosystem dependency and accept all packet/TMATS context through plain values.

**Source**: ADR-0005.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: cargo-tree review for every feature combination, manifest edge inventory, and standalone library-use example using plain declarations.

#### L2-API-106

**Parent**: L1-API-006

**Statement**: The library SHALL keep every library result deterministic from bytes, caller context, policy and table, with all I/O in the CLI/caller.

**Source**: ADR-0006.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: forbidden I/O API audit and deterministic replay with different host time/environment; no_std alone is not proof of no I/O through foreign imports.

#### L2-API-107

**Parent**: L1-API-007

**Statement**: The library SHALL keep the public value and recording APIs buildable for wasm 32-unknown-unknown with serde disabled and enabled.

**Source**: `irig106-tmats` L1-REL-003; contract 4.8; ADR-0014.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: two WASM builds plus boundary serialization fixtures for values above 2^53, alongside native feature checks.

#### L2-API-108

**Parent**: L1-API-008

**Statement**: The library SHALL document public functions with preconditions, units/epochs, error/finding behaviour, requirement IDs and edition-specific citations.

**Source**: ADR-0004; architecture section 9.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: rustdoc output and public-item citation checklist, invalid/missing context examples, and a compilable worked example.

#### L2-API-109

**Parent**: L1-API-009

**Statement**: The library SHALL forbid unsafe Rust in both rebuilt crate roots and keep the prohibition effective in every target/feature.

**Source**: Architecture section 9.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: root attributes and all-target feature builds; a temporary compile-fail fixture proves unsafe cannot be reallowed in a child module.

#### L2-API-110

**Parent**: L1-API-010

**Statement**: The library SHALL provide version-aware serde representations for public values and provenance without allowing deserialization to bypass invariants.

**Source**: Architecture section 9; tools store and exchange answers (contract 4.7 to 4.9).

**Verification Method**: Test (T)

**Planned verification**: Roundtrip every public type, malformed/out-of-range serialized fields, dangling IDs, unknown schema version, and values above 2^53; no bare absolute time substituted for an answer.

#### L2-API-111

**Parent**: L1-API-011

**Statement**: The library SHALL limit chrono conversion to calendar/scale combinations whose meaning is explicit and whose target representation can preserve it.

**Source**: Architecture section 9; L1-TAI-006.

**Verification Method**: Test (T)

**Planned verification**: Known UTC and explicitly converted TAI/GPS, unknown year/scale refusals, leap second, stale table and chrono range extremes, plus reverse conversions with declared scale.

#### L2-API-112

**Parent**: L1-API-012

**Statement**: The library SHALL enforce the documented no-panic/no-print Clippy lints in production library code from the first rebuilt module.

**Source**: L1-ERR-001 (no panics on any input); ADR-0006 (no I/O). Clippy's lints `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`, `print_stdout`, `print_stderr` enforce it; they replace the text-pattern semgrep rules of the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`). As `irig106-tmats` L1-ROB-002. From the first rebuilt code (the prototype has unaudited `unwrap` calls, ROADMAP P6-09).

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: lint settings/inheritance audit and negative lint fixtures; exercise first rebuilt modules under all-target/all-feature Clippy with warnings denied.

## PERF

#### L2-PERF-101

**Parent**: L1-PERF-001

**Statement**: The project SHALL measure packet decoding, counter lookup and synthetic timeline construction before choosing budgets for the rebuilt code.

**Source**: As `irig106-tmats` L1-PERF-001: a budget set after measuring rather than guessed. The prototype's measurements (`docs/benchmark_results.md`) are the baseline; the draft ecosystem L1 of the retired `irig106-rust` asked for "minimum 100 Mbps sustained" (REQ-L1-102), and the first measurement is compared with that figure.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: bounded CI benchmark artifacts for 0/1/many references and several channels, reproducible seed/workload, and explicit budget approval; no fabricated throughput requirement.

## CLI

#### L2-CLI-101

**Parent**: L1-CLI-001

**Statement**: The CLI SHALL provide a reusable CLI library and a thin irigtime binary in the lockstep workspace.

**Source**: ADR-0007; owner, 2026-09-26 and 2026-09-27.

**Verification Method**: Inspection (I)

**Planned verification**: Inspect workspace membership, binary naming and the public entry point; compare standalone invocation with an embedded library call.

#### L2-CLI-102

**Parent**: L1-CLI-002

**Statement**: The CLI SHALL expose host-callable commands and renderers so irig 106-cli can mount the same implementation.

**Source**: ADR-0007; ROADMAP P6-10.

**Verification Method**: Test (T)

**Planned verification**: Standalone and host-style calls produce the same report; injected I/O failures and help return without terminating the host.

#### L2-CLI-103

**Parent**: L1-CLI-003

**Statement**: The CLI SHALL parse commands and options by hand in the args module without an argument-parsing dependency.

**Source**: ADR-0008.

**Verification Method**: Inspection (I)

**Planned verification**: All six commands, every option, invalid limits, missing arguments, filenames beginning with a dash, and dependency inspection.

#### L2-CLI-104

**Parent**: L1-CLI-004

**Statement**: The CLI SHALL implement the six commands over a shared joining loop and report model with basis and findings.

**Source**: Contract 4.12; ADR-0011, ADR-0013.

**Verification Method**: Test (T)

**Planned verification**: A synthetic recording with both formats, two sessions, gaps/jumps, unknown year and counter-only answers; assert relevant provenance in each command.

#### L2-CLI-105

**Parent**: L1-CLI-005

**Statement**: The CLI SHALL label Format 2 output as NTP, IEEE 1588-2002 or IEEE 1588-2008 with scale and status.

**Source**: ROADMAP P6-10; §11.2.3.3.

**Verification Method**: Test (T)

**Planned verification**: Each NTF with valid and invalid status in every renderer; native GPS Format 1 remains distinct from network time.

#### L2-CLI-106

**Parent**: L1-CLI-006

**Statement**: The CLI SHALL publish a versioned JSON report schema and consistent text/CSV views of the same report.

**Source**: As `irig106-tmats` L1-CLI-006; `irig106-cli` and scripts rely on the output (contract 4.10); architecture section 4.4.

**Verification Method**: Test (T)

**Planned verification**: Golden outputs with quotes/commas/newlines, unknown year/scale, counter-only and values above 2^53; schema validation and report round trips.

#### L2-CLI-107

**Parent**: L1-CLI-007

**Statement**: The CLI SHALL document exit classifications and their precedence so scripts can distinguish findings, bad input and usage errors.

**Source**: As `irig106-tmats` L1-CLI-006; findings and their severities (ADR-0013).

**Verification Method**: Test (T)

**Planned verification**: Every exit class, severity threshold changes, malformed input with findings, usage error with a nonexistent file, and host invocation returning a classification.

## REL

#### L2-REL-101

**Parent**: L1-REL-001

**Statement**: The project SHALL validate one workspace version and the exact CLI library pin before release packaging.

**Source**: ADR-0007.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: manifest/package metadata and dry-run artifacts for both packages; intentionally mismatched versions/pins must fail.

#### L2-REL-102

**Parent**: L1-REL-002

**Statement**: The project SHALL check Rust 1.85 and edition 2024 separately from current stable development checks.

**Source**: ADR-0003.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: metadata and supported MSRV feature matrix, including serde/chrono; failures cannot be hidden by running only a newer compiler.

#### L2-REL-103

**Parent**: L1-REL-003

**Statement**: The project SHALL review the 0.8.0 changelog against every finding that changes prototype answers and each consumer-visible removal.

**Source**: ADR-0017.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: changelog versus STANDARD-REVIEW and API diff; caption-only and documentation-only findings distinguished from behaviour changes.

#### L2-REL-104

**Parent**: L1-REL-004

**Statement**: The project SHALL check both crates on Linux, Windows and macOS with bounded CI resources.

**Source**: As `irig106-tmats` L1-REL-004; the draft ecosystem L1 (REQ-L1-103, the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`)).

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: three-platform results, CLI coverage, path/newline fixtures and failure propagation; current prototype results do not verify future rebuilt code.

#### L2-REL-105

**Parent**: L1-REL-005

**Statement**: The project SHALL enforce dependency policy with individually documented exceptions and pinned source identities.

**Source**: `deny.toml`, adapted from the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`); as `irig106-tmats` L1-REL-005. Its first run found `crossbeam-epoch` below 0.9.20 (RUSTSEC-2026-0204, through `criterion`), fixed by updating the lock file.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: policy results and exception inventory, exact Git revision and negative policy cases; no settings or credential changes.

#### L2-REL-106

**Parent**: L1-REL-006

**Statement**: The project SHALL require a verifiable SBOM and build-provenance artifact for each published irigtime binary.

**Source**: As `irig106-tmats` L1-REL-006 and its `docs/RELEASING.md` step 7.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: reviewed release workflow and sample verification, including digest mismatch and missing provenance.

## TST

#### L2-TST-101

**Parent**: L1-TST-001

**Statement**: The project SHALL require a prototype-failing regression before each code-affecting finding is fixed.

**Source**: ADR-0016.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: finding/test inventory with prototype-fail/fix-pass results; an unrelated passing test cannot stand in for a missing regression.

#### L2-TST-102

**Parent**: L1-TST-002

**Statement**: The project SHALL test the contract section 7 recording bytes and the complete expected basis.

**Source**: Contract 7.6; ADR-0016.

**Verification Method**: Test (T)

**Planned verification**: Independent byte array, perturbed length/flags/date cases and full answer comparison; explicitly selected preceding/interpolation variants must name their policy.

#### L2-TST-103

**Parent**: L1-TST-003

**Statement**: The project SHALL derive fixtures from cited standard figures/tables and hand-compute expected values independently of encoders.

**Source**: ADR-0016.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: fixture citations and recalculation, Figures 11-4/4-4/11-12 through 11-17 and the RDG-003 handbook example.

#### L2-TST-104

**Parent**: L1-TST-004

**Statement**: The project SHALL keep real recordings outside Git and CI; automated fixtures must be synthetic.

**Source**: Owner's standing direction; ADR-0016; `irig106-tmats` ADR-0018.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: fixture provenance/size audit, ignore rules and CI input inventory; synthetic coverage must not be called vendor validation.

#### L2-TST-105

**Parent**: L1-TST-005

**Statement**: The project SHALL generate traceability from the requirement hierarchy and test markers and fail CI on output drift.

**Source**: Architecture section 12; as `irig106-tmats`'s `scripts/build-trace-matrix.py`.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: regenerate then --check; hierarchy/marker audit and an in-memory stale-output negative check without committing a stale artifact.

#### L2-TST-106

**Parent**: L1-TST-006

**Statement**: The project SHALL measure library line coverage in CI with an 80% gate and retain requirement gaps separately.

**Source**: As `irig106-tmats` L1-ROB-003; the draft ecosystem L1 ("minimum 80% code coverage", REQ-L1-111, the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`)).

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: coverage artifact, configured gate, a below-threshold failure and documented exclusions.

## Retired prototype identifiers

These definitions belonged to the prototype document replaced by this
draft. They are historical identifiers, not active requirements or aliases
for the new IDs. Existing test-table mentions must be reassessed during
the tests-first rebuild, not mechanically relabelled.

`L2-ABS-001`, `L2-ABS-002`, `L2-ABS-003`, `L2-ABS-004`, `L2-ABS-005`, `L2-ABS-006`, `L2-ABS-007`, `L2-ABS-008`, `L2-API-001`, `L2-API-002`, `L2-API-003`, `L2-API-004`, `L2-BCD-001`, `L2-BCD-002`, `L2-BCD-003`, `L2-BCD-004`, `L2-BCD-005`, `L2-BCD-006`, `L2-BCD-007`, `L2-BCD-008`, `L2-BCD-009`, `L2-BCD-010`, `L2-COR-001`, `L2-COR-002`, `L2-COR-003`, `L2-COR-004`, `L2-COR-005`, `L2-COR-006`, `L2-COR-007`, `L2-CSDW-001`, `L2-CSDW-002`, `L2-CSDW-003`, `L2-CSDW-004`, `L2-CSDW-005`, `L2-CSDW-006`, `L2-CSDW-007`, `L2-CSDW-008`, `L2-ERR-001`, `L2-ERR-002`, `L2-ERR-003`, `L2-F2COR-001`, `L2-F2COR-002`, `L2-F2COR-003`, `L2-F2CSDW-001`, `L2-F2CSDW-002`, `L2-F2CSDW-003`, `L2-F2CSDW-004`, `L2-IPT-001`, `L2-IPT-002`, `L2-IPT-003`, `L2-IPT-004`, `L2-IPT-005`, `L2-NTP-001`, `L2-NTP-002`, `L2-NTP-003`, `L2-NTP-004`, `L2-NTP-005`, `L2-PTP-001`, `L2-PTP-002`, `L2-PTP-003`, `L2-PTP-004`, `L2-PTP-005`, `L2-RTC-001`, `L2-RTC-002`, `L2-RTC-003`, `L2-RTC-004`, `L2-RTC-005`, `L2-RTC-006`, `L2-RTC-007`, `L2-RTC-008`, `L2-RTC-009`, `L2-RTC-010`, `L2-RTC-011`, `L2-SEC-001`, `L2-SEC-002`, `L2-SEC-003`, `L2-SEC-004`, `L2-SEC-005`, `L2-TAI-001`, `L2-TAI-002`, `L2-TAI-003`, `L2-TAI-004`.
