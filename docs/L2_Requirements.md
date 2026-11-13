# irig106-time — Level 2 Requirements

> **Status: requirements updated for owner decisions, 2026-11-13; independent review pending.**
> Rebuild requirements, not implementation evidence. Replaces the prototype
> document. Derives from the current L1, the contract, accepted ADRs, the
> architecture, the owner's 2026-11-11 answers, and the nine L2 decisions
> of 2026-11-12/13 (see L2's accepted-decision table).
> No Review field or owner-set Design status is changed by this draft.

## Scope and conventions

The checked-out L1 at main `980d3af` contains **124 defined entries in 27
categories**, not the 144 mentioned in the assignment. Every one has a
complete L1 → L2 → L3 path; no missing L1 requirements have been invented. The prototype
identifiers below are retired, never reassigned; new base IDs start at
101 and additional decompositions at 201. They do not imply existing test
coverage. Implementation is a later tests-first step (ADR-0016). There are **144
active requirements at this level**; all 27 L1 categories are covered.

Every obligation uses SHALL. Verification is planned: T = Test, I =
Inspection, A = Analysis, D = Demonstration. A planned check is not an
**Evidence** artifact. The generated TRACE-MATRIX remains the live status
source. Existing prototype tests do not verify a rebuilt obligation unless
reviewed and explicitly linked to it in the later implementation work.

**Source convention:** every §, figure and table without another edition
means **IRIG 106-24R1**; Chapters 4/9/10/11 are named where needed.
The base Source lines retain the L1 citations and quoted source words
verbatim, including historical prototype commentary. Completed page checks
are recorded separately in STANDARD-REVIEW T-10 and RDG-002. Requirements based on contract/ADR/owner policy
are product decisions, not claims that the standard prescribes an
algorithm. Quotes are verbatim; derived arithmetic and proposed choices
are identified as such. Archive provenance and rendered-page checks are
in STANDARD-REVIEW T-10 and INTERPRETATIONS RDG-002. RFC 1305/5905 are external
primary sources, not archived there; RDG-012 cites their sections and pages.

Shared-value contracts target the corrected irig106-types semantics of
contract section 8.6 (T-5/T-6/T-7/T-9), not the current prototype dependency.
A source's unresolved context is retained explicitly. The accepted 2026-11-12
decisions are distinguished from remaining owner proposals; neither is
silently treated as implemented code.
Human Review stays pending under the register's two-person rule.

L2 states functional boundaries, admission rules and externally observable
results. Every entry has exactly one L1 parent. Its L3 child supplies data
layout/algorithm obligations and planned cases. Additional STRM and RDG
entries decompose complex policy and reading obligations.

## Shared types and edition qualification

The target shared API is `irig106-types` merge **c59e004**, described in
its `docs/STANDARD-VALUES-REVIEW.md`: `Ertc::to_nanos(self) -> u128`
returns **1 ns per tick**; `TimeSource::Gps` is removed in favour of
`Reserved(3)`; source 2 is `InternalFromRmm` (formerly `InternalRtc`);
`TimeFormat::None` and `IrigTimeSource` exist; version-list decoding uses
`Irig106Version::from_setup_record` and `from_data_type_version`.
These are documentation targets, not changes to the dependency or Rust code.

**Chapter 4 exception:** Do not use c59e004 Ch4BinaryTime::from_secondary_bytes or its delegating from_intra_packet_bytes as-is: they swap high/usec relative to Figure 11-4. Per accepted ADR-0015 the shared fix is merged in irig106-types 48f1834 (types PR #4): reserved/usec/low/high layout and full binary count. Adopt that corrected revision in the separate code slice; this documentation change does not update dependencies.


**Edition qualification applies throughout this document:** modern SRC,
source None and FMT None obligations apply from **106-05**, after the
layout is selected by L2/L3-EDN-201. Resolved **106-04** uses a distinct EXT
value (external-present bit 0, reserved bits 3–1), reserves FMT 6–15 and
bits 31–12, and never feeds EXT to `TimeSource::from_raw`. Unknown edition
retains raw fields and an unresolved layout; do not label nibble 3 None,
FMT 15 None, or EXT 0 internal SRC merely by guessing. Scope the accepted
RDG mappings/source comparisons to their applicable resolved layouts;
legacy-to-declaration comparisons require separately documented analysis.
The existing L1 statements describe the 106-24R1 baseline; these cited
legacy differences qualify their L2/L3 decomposition without rewriting L1.

## Owner decisions recorded 2026-11-12/13

These nine decisions are accepted requirements for future implementation,
not evidence that current Rust code implements them. Sources and the exact
changed requirement/parent/Source lines are listed in
[the decision record](analysis/owner-decisions-2026-11-12.md).

| ID | Accepted behaviour and basis |
|---|---|
| Q-DEFAULT | Stream default **standard**, preceding reference (RCC 123-20 §5.6, printed 5-44; ADR-0010); **interpolate** explicitly selectable. Offline nearest unchanged. |
| Q-STREAM-BOUND | Hold **1 s + 100 ms** policy tolerance (Chapter 10 §10.6.2 and §10.6.1 b); gap tolerance **0 ns** independently; explicit caller progress and caps, no hidden queue. Cap/timeout/EOF releases extrapolated from preceding reference, counter-only if none. §10.6.1 c's <=1000 ms commit permits late-written time packets; the hold tolerance is reader policy. |
| Q-STREAM-CHANNEL | **Option A** committed before data: caller-fixed, else TMATS external TIMEIN (`R-x\CDT-n=TIMEIN`, `R-x\TSRC-n=E`, Table 9-4), else lowest eligible ID; no stream future-reference-count ranking. If not decidable in advance, counter-only until caller chooses. No revisions or lost-lock switches (RCC 123-20 §5.6; §11.2.3.2 note). |
| Q-CH4 | Explicit caller binary origin; BCD year and full day. Otherwise labelled partial, never invented day 1/year/origin. No nearest-time-packet option now. Chapter 4 §4.7's older characters/current bits wording remains unresolved and reported. |
| Q-NTP | Owner: "make this as correct as possible". **RDG-012** records UTC-following reckoning, era declaration/pivot/labelled-era0 precedence, all-zero sentinel, leap ambiguity and table currency; PTP last-known-offset fallback with TF-021 outside coverage; floor rounding accepted owner 2026-11-13. |
| Q-FLAGS | **Refine** (owner 2026-11-12): source=1 requires presence=1 and a defined secondary format; source=0/format=11 keeps RTC with TF-006 warning, separately reporting unreadable secondary time if present. [Page checks and truth table](analysis/Q-FLAGS-2026-11-12.md); L1-IPT-007 → L2-IPT-107 → L3-IPT-107. |
| Q-CLI-EXIT | Align with `irig106-tmats` (owner 2026-11-12): **0 success; 1 threshold findings; 2 usage; 3 unreadable/invalid Chapter 10/time input; 4 output-write failure**. Shared ecosystem convention, never renumber; IRIG 106 is silent. [Pinned draft source and planned checks](analysis/Q-CLI-EXIT-2026-11-12.md). |
| Q-EDITION | Accepted owner 2026-11-13: explicit declaration/override **wins**, basis **override**, contradictory opted-in G\106/header evidence retained side by side; without override, caller opt-in G\106 packet edition → **declared, assumed from TMATS** only when uncontradicted by packet evidence, else unresolved/raw bits with evidence shown (and a finding for a no-override conflict), no SRC/None invention. TMATS ADR-0028 vocabulary; no fallback selects EXT. [Sources and planned fixtures](analysis/Q-EDITION-2026-11-13.md). |
| Q-TFMT-DEFAULT | Accepted owner 2026-11-13: by default missing required TFMT is reported and no comparison made; caller opt-in compares default A with default-derived basis, retaining the missing finding and raw absence. RDG-002 / Table 9-4 / Ch9 §9.5.1 a / TMATS ADR-0023. |

All nine original L2 questions are decided. Q-FLAGS-ERROR is also resolved
as TF-028 (owner 2026-11-13). Other explicitly marked proposals remain.

**NTP fraction floor (round toward zero) is accepted**, owner 2026-11-13;
retain the raw fraction and quantization basis. RFC 1305/5905 archiving is
proposed, not performed. Interpolation rounding/ties and mode-change flushing
remain proposed where marked.

Interpolation rounding, midpoint ties, mixed-resolution
jump thresholds, and mode-change flushing are explicitly proposed design
choices in the requirements. They need review as part of this draft; none
changes an accepted Design field. RDG-008 remains suspect; page inspection
does not settle it or replace its real-recording evidence.

## RTC

#### L2-RTC-101

**Parent**: L1-RTC-001

**Statement**: The library SHALL expose the shared Rtc type without adding an absolute epoch; retain its raw 48-bit value and distinguish counter ticks from durations.

**Source**: "This is a free-running 10-MHz binary counter represented by 48 bits that are common to all data channels" (§11.2.1.1 i). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Raw zero, maximum, one-bit boundaries, and a value with bit 48 set; confirm the chosen constructor contract and that no calendar date is produced.

#### L2-RTC-102

**Parent**: L1-RTC-002

**Statement**: The library SHALL provide separate header-counter and eight-byte counter-stamp readers, each checking its own minimum length and padding.

**Source**: §11.2.1.1 i, Table 11-2; "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b). Prototype identifier kept, widened to time stamps.

**Verification Method**: Test (T)

**Planned verification**: Distinct bytes 01 02 03 04 05 06 decode to 0x060504030201; truncate at every byte; vary both padding bytes independently.

#### L2-RTC-103

**Parent**: L1-RTC-003

**Statement**: The library SHALL resolve wrap-safe counter differences within a named session; require ordering context for differences that cannot be uniquely unwrapped.

**Source**: 2^48 ticks at 10 MHz is about 325.8 days; the counter "shall remain free-running during each session" (§11.2.1.1 i). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Maximum to zero is +1 tick and reverse is -1; equal counters are zero; exercise half-range ambiguity and reject a cross-session subtraction.

#### L2-RTC-104

**Parent**: L1-RTC-004

**Statement**: The library SHALL convert signed counter deltas to typed nanosecond durations without truncation or overflow.

**Source**: §11.2.1.1 i (10 MHz). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Zero, +1, -1, maximum session-supported delta, and destination overflow; 150000 ticks equals 15000000 ns.

## ABS

#### L2-ABS-101

**Parent**: L1-ABS-001

**Statement**: The library SHALL represent day-of-year, time-of-day, optional calendar year, and leap-second state without collapsing an unknown year into a fabricated one.

**Source**: IEEE 1588 and ERTC times resolve 1 ns (§11.2.1.1 g); the day format carries no year (Figure 11-13); "excluding leap second days when a second may be added or subtracted" (RCC 200-16 §3.6); contract 3.2; T-16. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Day 0/1/365/366, Gregorian 1900/2000/2100, unknown year, midnight and year-end, 23:59:59 to 23:59:60 to 00:00:00 with explicit leap context.

#### L2-ABS-102

**Parent**: L1-ABS-002

**Statement**: The library SHALL decode and encode the Chapter 4 secondary/intra-packet representation with the shared corrected word order, preserve raw values and separate reserved findings, and require explicit caller origin for calendar reconstruction.

**Source**: "resolution of 655.36 seconds", "10 milliseconds", "three least significant bits of the BCD Julian date", "at a rate necessary to resolve time ambiguities" (106-24R1 Chapter 4 §4.7, printed 4-11 / PDF 15); RCC 123-20 `enI106_Ch4Binary2IrigTime`, printed B-43 / PDF 149, applies word weights without an origin. Explicit caller context and partial results are owner policy (2026-11-12); historical characters/bits wording remains unresolved (T-10). Figure 11-4, printed 11-10 / PDF 18; raw layout already confirmed in T-10 and adopted from types 48f1834. Retained earlier source: "00 = Chapter 4 binary weighted 48-bit time format. The two lsbs of the 64-bit packet secondary header time and IPTS shall be zero-filled" (§11.2.1.1 g); Figure 11-4; Chapter 4 §4.7. Layout confirmed 2026-11-12, T-10. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Retain distinct-word and low=0x8000 fixtures and usec9999/10000; raw values survive reserved warnings; no-origin result stays labelled partial, explicit-origin reconstruction is separately qualified.

#### L2-ABS-103

**Parent**: L1-ABS-003

**Statement**: The library SHALL keep secondary-header IEEE 1588 seconds/nanoseconds distinct from the reversed word sequence of a Format 2 PTP body.

**Source**: "The 32 bits indicating seconds shall be placed into the MSLW portion of the secondary header and the 32 bits indicating nanoseconds shall be placed into the LSLW portion" (§11.2.1.1 g); Figure 11-5. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Distinct seconds=0x01020304 and ns=123456789; ns=999999999/1000000000; compare with a Format 2 body containing seconds first.

#### L2-ABS-104

**Parent**: L1-ABS-004

**Statement**: The library SHALL use the corrected shared Ertc value as nanoseconds, retaining its relative nature and full 64-bit range.

**Source**: "64-bit binary extended relative time counter (ERTC) with 1-nanosecond resolution … (RTC = ERTC/100)" (§11.2.1.1 g); Figure 11-6 (LSLW, MSLW); T-5. Prototype identifier kept; the resolution is now stated.

**Verification Method**: Test (T)

**Planned verification**: ERTC=1 gives 1 ns, 100 gives 100 ns; max u64 and asymmetric bytes; T-5 regression must distinguish the prototype factor of 100.

## IPT

#### L2-IPT-105

**Parent**: L1-IPT-005

**Statement**: The library SHALL decode flags once into secondary-presence, stamp-source, secondary-format, and RTC-sync-error fields; unrelated flag bits must not alter time dispatch.

**Source**: "Bit 7: Indicates the presence or absence of the packet secondary header"; "Bit 6: Indicates the IPTS time source. 0 = Packet header 48-bit RTC. 1 = Packet secondary header time (bit 7 must be 1)"; "Bits 3-2: Indicate the packet secondary header time format" (§11.2.1.1 g); T-1. Replaces L1-IPT-001 to 004.

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

**Statement**: The library SHALL return no time from a stamp and report TF-015 (error) when bit 6=1 with bit 7=0, or TF-028 (error) when bit 6=1 with bits 3–2=11 (undefined secondary format). When bit 6=0 with bits 3–2=11, it SHALL read the padded 48-bit RTC stamp normally and report TF-006 (warning, value kept). If bit 7=1 in that RTC case, the secondary header time itself remains unreadable and SHALL be reported separately as TF-028 without discarding the RTC stamp.

**Source**: "0 = Packet header 48-bit RTC."; "1 = Packet secondary header time (bit 7 must be 1)."; "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved" (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17); "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b, printed 11-11 / PDF 19). Value retention is owner 2026-11-12, Q-FLAGS; TF-006 reserved-value warning and unchanged TF-005/TF-006 levels are confirmed owner 2026-11-13; page images and older equivalent wording: `docs/analysis/Q-FLAGS-2026-11-12.md`. Owner 2026-11-13, Q-FLAGS-ERROR accepts TF-028 "Secondary-derived time has an undefined format" at error level, architecture section 8; UndefinedSecondaryTimeFormat maps to TF-028. TF-001..TF-027 remain unchanged.

**Verification Method**: Test (T)

**Planned verification**: Exhaust all 256 flag bytes against the source/presence/format rule, independently of other flag findings. Literal cases 0x0C and 0x8C decode RTC with TF-006; 0x8C also reports unreadable secondary time separately; it is TF-028 (error). 0x4C and 0xCC return no stamp time/errors for undefined secondary format, with 0x4C also lacking the required header; 0x40 returns no stamp time/error for absent required header. Use hand-authored padded RTC bytes, short lengths and nonzero padding as separate validation cases. Use an independent hand-transcribed truth table for diagnostics and exact literal error-variant assertions. Flag decoding tests are implemented; future findings emission/report integration remains separate.

## SEC

#### L2-SEC-101

**Parent**: L1-SEC-001

**Statement**: The library SHALL parse a secondary header as a bounded twelve-byte structure and decode Chapter 4 time only after validation.

**Source**: "The length of the packet secondary header is fixed at 12 bytes (96 bits)" (§11.2.1.2); Figure 11-4. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Twelve bytes with distinguishing words and an independently computed checksum; lengths 0 through 11; trailing body bytes must not enter the checksum.

#### L2-SEC-102

**Parent**: L1-SEC-002

**Statement**: The library SHALL for secondary format 01, return the IEEE 1588 value plus header validation results, not a Format 2 body.

**Source**: §11.2.1.2 a; Figure 11-5. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Seconds/nanoseconds in different values, boundary nanoseconds, bad checksum, and a packet-specific applicability supplied as metadata.

#### L2-SEC-103

**Parent**: L1-SEC-003

**Statement**: The library SHALL for secondary format 10, return the relative ERTC value and validate it independently of calendar correlation.

**Source**: §11.2.1.2 a; Figure 11-6. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: ERTC=1/99/100 and all-high bits; checksum failure suppresses trusted own time, while absent references must not fabricate absolute time.

#### L2-SEC-104

**Parent**: L1-SEC-004

**Statement**: The library SHALL use the RDG-008 byte-sum reading for trust decisions and calculate the alternative word sum for diagnostics; retain its suspect status.

**Source**: "a 16-bit arithmetic sum of all secondary header bytes excluding the secondary header checksum word" (§11.2.1.2 c); whether "bytes" means bytes or 16-bit words is open (T-15: the prototype sums words; the words of the standard and RCC 123-20's `uCalcSecHeaderChecksum` sum bytes); contract 6.6. Prototype identifier kept; the reading is settled in L2.

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

**Source**: §11.2.3.2 a; Figure 11-12; Table 11-4. Prototype identifier kept. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Verification Method**: Test (T)

**Planned verification**: One-bit field walking and asymmetric bytes; data type 0x11 dispatch; every truncated word length.

#### L2-CSDW-102

**Parent**: L1-CSDW-002

**Statement**: The library SHALL select EXT versus SRC only after resolving edition evidence under L2-EDN-201; read 106-04 EXT bit 0 with bits 3–1 reserved, and from 106-05 use corrected shared SRC values, never GPS or legacy None for raw 3.

**Source**: §11.2.3.2 a, "0x3 – 0xE = Reserved"; T-6. Prototype identifier kept; value 3 is no longer "GPS". Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Verification Method**: Test (T)

**Planned verification**: Resolved 106-04: all sixteen EXT nibbles, with bit 0 present/absent and every reserved bit 3–1; raw 3 is present plus reserved-bit evidence, never None. Resolved 106-05+: all sixteen SRC/FMT values, including Reserved(3), InternalFromRmm for 2 and FMT None for 15. Unknown/conflicting edition plus header 0x01 must not silently select EXT; a later explicit declaration with header 0x01 must not be treated as 106-04; never invent a legacy setup version code.

#### L2-CSDW-103

**Parent**: L1-CSDW-003

**Statement**: The library SHALL expose FMT with edition qualification: resolved 106-04 reserves 6–15; from 106-05 FMT 15 is TimeFormat::None and cannot produce a default usable reference.

**Source**: "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); T-7. Prototype identifier kept. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Verification Method**: Test (T)

**Planned verification**: For resolved 106-05+, all FMT values crossed with SRC=0/1/3/15; None yields TF-003 and reserved values TF-006. In resolved 106-04, FMT 0xF is reserved, not None; unknown edition preserves that distinction as unresolved.

#### L2-CSDW-104

**Parent**: L1-CSDW-004

**Statement**: The library SHALL carry the LY bit with each Format 1 value for later checks against the resolved day and year.

**Source**: "Bit 8: Indicates if this is a leap year" (§11.2.3.2 a). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: LY 0/1 with day 365/day 366 and known leap/common years; extraction must not depend on DATE or source bits.

#### L2-CSDW-105

**Parent**: L1-CSDW-005

**Statement**: The library SHALL select the six-byte or eight-byte Format 1 body solely by DATE bit 9.

**Source**: "0 = IRIG day available (Figure 11-13) 1 = Month and year available (Figure 11-14)" (§11.2.3.2 a). Prototype identifier kept.

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

**Source**: "All reserved bit fields in packet headers or CSDWs shall be set to zero (0x0)" (§11.2.1 f); §11.2.3.2 a; T-7. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Verification Method**: Test (T)

**Planned verification**: Set each bit of all three masks independently (106-04 EXT, 106-05–15 SRC, 106-17+ ITS); distinguish reserved bits from valid fields, preserve unknown-layout ambiguity, and never treat old zero ITS bits as certain freewheeling.

## BCD

#### L2-BCD-103

**Parent**: L1-BCD-003

**Statement**: The library SHALL validate the zero positions in each date-format figure without consuming a fictitious extra reserved word.

**Source**: Figures 11-13 and 11-14; Table 11-16, "0 Always zero". Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: A fixture per figure, each always-zero bit set individually, and adjacent filler made nonzero to prove it is not parsed.

#### L2-BCD-104

**Parent**: L1-BCD-004

**Statement**: The library SHALL validate each BCD digit before combining digits, returning a field-specific error for an invalid nibble.

**Source**: "the time data words are inserted in the packet in binary-coded decimal format" (§11.2.3.2 b). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: All 0..15 digit patterns at every digit position, separating reserved-bit violations from invalid decimal digits.

#### L2-BCD-105

**Parent**: L1-BCD-005

**Statement**: The library SHALL read and encode the day body as exactly three little-endian u16 words and preserve its 10 ms resolution.

**Source**: Figure 11-13; Table 11-16; T-12. Replaces L1-BCD-001 (8 bytes).

**Verification Method**: Test (T)

**Planned verification**: Contract section 7 bytes 35 27 45 13 87 01 decode day 187 13:45:27.350; exactly 6 bytes succeeds, shorter fails; 10 ms steps and second 60 under RDG-010.

#### L2-BCD-106

**Parent**: L1-BCD-006

**Statement**: The library SHALL read and encode the calendar body as exactly four little-endian u16 words, validating the resulting calendar date.

**Source**: Figure 11-14; Table 11-16; T-12. Replaces L1-BCD-002 (10 bytes).

**Verification Method**: Test (T)

**Planned verification**: Independently packed 2000-02-29 and 1900-02-29; exactly 8 bytes and every shorter length; December 31 rollover; no fifth word read. T-12 regression.

## F2CSDW

#### L2-F2CSDW-101

**Parent**: L1-F2CSDW-001

**Statement**: The library SHALL decode Format 2 data words independently of the Format 1 representation and distinguish network format from time validity.

**Source**: §11.2.3.3 a; Figure 11-15. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Data words 0x00000001, 0x00000011, 0x00000021 represent valid NTP/PTP 2002/PTP 2008; short words and T-2 regression.

#### L2-F2CSDW-104

**Parent**: L1-F2CSDW-004

**Statement**: The library SHALL dispatch NTP and the two PTP versions from NTF bits 7-4 while preserving unsupported values.

**Source**: "Network Time Format (NTF). Bits 7-4" (§11.2.3.3 a); T-2. Replaces L1-F2CSDW-002.

**Verification Method**: Test (T)

**Planned verification**: All sixteen values; vary TS independently and verify NTF=2 is accepted as PTP 2008 rather than reserved.

#### L2-F2CSDW-105

**Parent**: L1-F2CSDW-005

**Statement**: The library SHALL carry TS validity into the reference-admission decision; a reserved status is not implicitly valid.

**Source**: "Time Status (TS). Bits 3-0 indicate the status of the network time" (§11.2.3.3 a); T-2. Replaces L1-F2CSDW-002.

**Verification Method**: Test (T)

**Planned verification**: Every status with all three defined NTFs; explicit policy admission of invalid time remains labelled and cannot change its recorded status.

#### L2-F2CSDW-106

**Parent**: L1-F2CSDW-006

**Statement**: The library SHALL report each nonzero bit in the Format 2 reserved region without rejecting valid NTF bits as reserved.

**Source**: "Reserved. Bits 31-8 are reserved" (§11.2.3.3 a); §11.2.1 f; T-2. Replaces L1-F2CSDW-003 (bits 31–4).

**Verification Method**: Test (T)

**Planned verification**: Walking bits 8..31 and valid low-byte fields; NTF=1/2 must not trigger reserved-bit findings.

## NTP

#### L2-NTP-101

**Parent**: L1-NTP-001

**Statement**: The library SHALL read and write a bounded eight-byte NTP time body as seconds then fractional word.

**Source**: "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (Figure 11-16). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Seconds=0x01020304 and fraction=0x80000000; short 0..7 bytes, trailing bytes untouched, independently supplied wire expectations.

#### L2-NTP-102

**Parent**: L1-NTP-002

**Statement**: The library SHALL retain the raw UTC-labelled seconds/fraction and resolve era from caller-declared era, else a labelled RFC 5905 pivot using a caller approximate date or independently dated absolute reference in the same session, else era 0 as a labelled policy assumption; never read the host clock.

**Source**: "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. "Eras cannot be produced by NTP directly" and "they can be derived from external means" (RFC 5905 §6, p.12); "if the client is set within 68 years of the server" (§6, p.15); "0 h 1 January 1900 UTC" (§6, p.13). Evidence precedence and labelled era-0 fallback are owner policy (2026-11-12), specified in RDG-012.

**Verification Method**: Test (T)

**Planned verification**: Explicit era wins; caller-date and same-session dated-reference pivots before/after 2036; exact half-era ambiguity and conflicting pivots; no evidence gives labelled era 0; no host-clock dependence; do not relabel an assumed NTP date as independent evidence.

#### L2-NTP-103

**Parent**: L1-NTP-003

**Statement**: The library SHALL compute the nanosecond view of the unsigned 2^-32-second fraction by floor (round toward zero), preserve the original word, and record rounding/quantization in the basis (owner 2026-11-13).

**Source**: "32-bit fraction field" (RFC 5905 §6, p.13); "Time Unsigned Seconds Fractions" (106-24R1 Figure 11-16, printed 11-27 / PDF 35). Floor (round toward zero for the unsigned fraction) is accepted owner policy, 2026-11-13, not standard wording (RDG-012).

**Verification Method**: Test (T)

**Planned verification**: Fraction 0 -> 0 ns, 0x80000000 -> 500000000 ns, 0xFFFFFFFF -> 999999999 ns under accepted floor; adjacent representability boundaries, raw-word preservation, no 1e9 carry; owner floor decision 2026-11-13.

#### L2-NTP-104

**Parent**: L1-NTP-004

**Statement**: The library SHALL convert resolved NTP seconds to UTC civil/Unix-style time by subtracting 2208988800 seconds, without adding or subtracting cumulative TAI-UTC; return no absolute time for an all-zero seconds/fraction timestamp and retain leap-ambiguity and table-currency evidence.

**Source**: "2,208,988,800" (RFC 5905 §6 Figure 4, p.14, 1 Jan 1970); "a value of zero is a special case representing unknown or unsynchronized time" (§6, p.14); "knowledge of all previous leap seconds is lost" (RFC 1305 Appendix E.8, p.78); Figure 8 (p.79) repeats the same NTP number for UTC second 60 and midnight. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`.

**Verification Method**: Test (T)

**Planned verification**: RFC 5905 Figure 4: 1970, 1972, 1999-12-31 and 2036 era 1; a post-2016-leap date must not acquire +/-37 s; negative pre-1970 Unix time; exact all-zero sentinel vs zero seconds/nonzero fraction; [B,B+1) boundaries with a covered insertion; missing/stale table keeps value and records uncertainty.

## PTP

#### L2-PTP-102

**Parent**: L1-PTP-002

**Statement**: The library SHALL give Format 2 PTP a stated TAI scale and the standard's 1970 epoch, independent of NTP UTC.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds" (§11.2.3.3). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Raw epoch 0 remains TAI; values around a leap transition; NTP/PTP raw seconds with the same number must retain distinct epoch/scale metadata.

#### L2-PTP-103

**Parent**: L1-PTP-003

**Statement**: The library SHALL reject a PTP body with a nanosecond word outside a single second.

**Source**: Figure 11-17 ("Time Unsigned Nanoseconds"). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: 0, 999999999, 1000000000, and 0xFFFFFFFF; reject malformed values even if TS=valid.

#### L2-PTP-104

**Parent**: L1-PTP-004

**Statement**: The library SHALL retain raw TAI and convert it to UTC using the applicable TAI-UTC offset. Outside leap-table coverage it SHALL use the last known offset, label the conversion possibly out of date and raise TF-021 (warning), recording table identity, coverage, offset and currency.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012.

**Verification Method**: Test (T)

**Planned verification**: Covered transition before/during/after and historical offset; exact coverage endpoints and one tick beyond; stale table and long no-new-leap interval. Outside coverage with a known last offset: UTC = TAI minus that offset, UTC retained with possibly-out-of-date label and TF-021 warning, raw TAI retained. Check table/offset provenance; empty table cannot invent an offset.

#### L2-PTP-105

**Parent**: L1-PTP-005

**Statement**: The library SHALL read and encode the IRIG Format 2 PTP body as eight bytes, separate from general IEEE 1588 network-message layouts.

**Source**: "Time Unsigned Seconds", "Time Unsigned Nanoseconds" (Figure 11-17; the same in 106-17 and 106-19); T-3. Replaces L1-PTP-001 (10 bytes, 48-bit seconds).

**Verification Method**: Test (T)

**Planned verification**: Distinct 32-bit words, max seconds, short 0..7 bytes, exactly 8 bytes with two nonzero following filler bytes; T-3 regression.

## F2COR

#### L2-F2COR-101

**Parent**: L1-F2COR-001

**Statement**: The library SHALL build Format 2 references using the packet header RTC as the capture point and the decoded body as absolute time.

**Source**: "The Format 2 Network Time packet data type is used to extract and encapsulate absolute time from NTP or IEEE-1588 PTP and time tag it with the RTC" (§11.2.3.3). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Mixed Format 1/2 timeline, valid/invalid TS, each NTF, and absent declarations; no use of packet arrival wall-clock time.

#### L2-F2COR-102

**Parent**: L1-F2COR-002

**Statement**: The library SHALL prevent arithmetic between differently scaled references unless an explicit conversion is recorded.

**Source**: §11.2.3.3; contract 4.1 rule 4. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Same physical instant expressed as NTP UTC and PTP TAI, GPS versus UTC, unknown scale, and a stale-table conversion; equal raw seconds are not equal instants by default.

## TAI

#### L2-TAI-101

**Parent**: L1-TAI-001

**Statement**: The library SHALL represent leap-table transitions as dated, ordered offset records with explicit coverage information.

**Source**: L1-PTP-004. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Empty table, unordered entries, equal-date conflicting offsets, before-first and exact-boundary lookups.

#### L2-TAI-102

**Parent**: L1-TAI-002

**Statement**: The library SHALL keep a release-reviewed built-in table and its known-current-through date as separate data from the last offset change.

**Source**: Contract 6.8. Prototype identifier kept.

**Verification Method**: Inspection (I)

**Planned verification**: Planned inspection: table/source diff and release checklist, including a period with no new leap transition; no live network fetch inside the library.

#### L2-TAI-103

**Parent**: L1-TAI-003

**Statement**: The library SHALL allow a caller-supplied table to replace or extend built-in data through the same validation path.

**Source**: Contract 6.8. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Valid extension, replacement, conflicting boundary, unsorted extension, and proof that previously issued answers retain the table provenance they used.

#### L2-TAI-104

**Parent**: L1-TAI-004

**Statement**: The library SHALL state table identity, coverage and known-current-through separately from the final transition and record NTP leap-ambiguity coverage limits. PTP outside coverage SHALL retain TAI and convert UTC using the last known offset, label it possibly out of date and raise TF-021 (warning), as contract 6.8/6.9 and L1-TAI-004 require.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Retained earlier source: "The leap-second table does not reach the date … the conversion is given with the last known offset and labelled as possibly out of date" (contract 6.8).

**Verification Method**: Test (T)

**Planned verification**: Covered transition before/during/after and historical offset; exact coverage endpoints and one tick beyond; stale table and long no-new-leap interval. Outside coverage with a known last offset: UTC = TAI minus that offset, UTC retained with possibly-out-of-date label and TF-021 warning, raw TAI retained. Check table/offset provenance; empty table cannot invent an offset. NTP direct UTC is retained with leap-ambiguity/currency uncertainty and no cumulative offset adjustment.

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

**Statement**: The library SHALL accept and retain explicit UTC second 60 only under RDG-010 date/time constraints, and label an NTP reading leap-ambiguous rather than inventing second 60 when Format 2 lacks a leap indicator.

**Source**: "time changes are made on December 31 and on June 30 at 2400 hours" (RCC 200-16 Appendix A.2, printed A-1 / PDF 56); "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (106-24R1 Figure 11-16, printed 11-27 / PDF 35); RFC 1305 Appendix E.8 Figure 8, p.79. RDG-010 remains accepted; RDG-012 refines the NTP case.

**Verification Method**: Test (T)

**Planned verification**: Retain Format 1 known/unknown-year UTC leap cases; NTP covered [B,B+1) ambiguity is distinct from explicit second 60, normal values outside, and unknown table coverage.

## WORD

#### L2-WORD-101

**Parent**: L1-WORD-001

**Statement**: The library SHALL reconstruct Chapter 4 binary calendar time only with an explicit caller origin, and BCD calendar time only with explicit year and full day; otherwise return a labelled partial reading while reporting the unresolved edition-dependent characters/bits wording.

**Source**: "resolution of 655.36 seconds", "10 milliseconds", "three least significant bits of the BCD Julian date", "at a rate necessary to resolve time ambiguities" (106-24R1 Chapter 4 §4.7, printed 4-11 / PDF 15); RCC 123-20 `enI106_Ch4Binary2IrigTime`, printed B-43 / PDF 149, applies word weights without an origin. Explicit caller context and partial results are owner policy (2026-11-12); historical characters/bits wording remains unresolved (T-10). Retained earlier source: "High and low order time words shall be binary or binary coded decimal (BCD) weighted, and microsecond words shall be binary weighted"; resolutions of 1 µs (to 9999), 10 ms, and 655.36 s binary or one minute BCD; "For BCD, the days field shall contain the three least significant bits of the BCD Julian date" (Chapter 4 §4.7); `C-d\PTM`, `C-d\BTM` (Chapter 9); contract 3.8. Layout confirmed 2026-11-12, T-10.

**Verification Method**: Test (T)

**Planned verification**: Binary count 0/0x8000/full count with absent/provided origin; BCD with absent year/full day and explicit context; day-fragment disagreement; older characters/current bits reported rather than silently harmonized; no nearest-packet setting or day-1 repair.

#### L2-WORD-102

**Parent**: L1-WORD-002

**Statement**: The library SHALL represent full network-time words separately from component-only words; use RDG-012 for full NTP era/UTC mapping and require missing components before constructing a complete time.

**Source**: "PTP Time (64-bit)", "NTP Time (64-bit)" (106-24R1 Chapter 9 Table 9-11, `C-d\NTM`); "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Retained earlier source: `C-d\NTM`: "PTP Time (64-bit)", "PTP Seconds (32-bit)", "PTP Nanoseconds (32-bit)", "NTP Time (64-bit)", "NTP Seconds (32-bit)", "NTP Fractions (32-bit)" (Chapter 9, Table 9-11); contract 3.8.

**Verification Method**: Test (T)

**Planned verification**: All six Table 9-11 kinds; full NTP with declared/pivot/assumed era and zero sentinel; component words missing seconds/fraction/ns stay partial; full PTP outside table retains TAI and converts UTC with the last known offset, possibly out of date with TF-021.

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

**Statement**: The library SHALL keep setup-record RCCVER and packet data-type-version values in distinct typed domains using corrected irig106-types mappings.

**Source**: Table 11-4 and §11.2.1.1 e ("0x0A = RCC 106-22"); Figure 11-34 ("0x0E = RCC 106-22", "0x0F through 0xFF = Reserved"); T-9; ADR-0015. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Verification Method**: Test (T)

**Planned verification**: All assigned/reserved shared codes in both lists, especially 0x0A/0x0D/0x0E/0x0F, preserving raw unknown codes. T-9 regression.

#### L2-EDN-103

**Parent**: L1-EDN-003

**Statement**: The library SHALL select explicitly cited edition differences without extrapolating missing archive evidence into claims of verified legacy support.

**Source**: "References to RCC 106-04 through RCC 106-15 refer to Chapter 10, while RCC 106-17 onward refer to Chapter 11" (§11.2.1.1 e); ADR-0004. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Verification Method**: Test (T)

**Planned verification**: An independent fixture per available edition touched by a reading, unknown edition/code, pre-07 declared edition absent, and source-backed exceptions. Include the archived 106-04 full-PDF fixture, EXT/SRC transition and version-evidence conflicts under L2-EDN-201.

#### L2-EDN-201

**Parent**: L1-EDN-002

**Statement**: The library SHALL give an explicit caller declaration/override precedence when selecting packet layout (basis override), retaining and showing contradictory opted-in G\106 and header data-type-version evidence side by side in the basis. Without an override, use caller-opted-in G\106 as packet edition (basis declared, assumed from TMATS) only when not contradicted by packet evidence; otherwise leave layout unresolved, preserve raw bits and invent no SRC/None labels, retaining the evidence and reporting a conflict only when packet evidence contradicts the opted-in declaration. Differing declarations alone do not automatically produce a conflict finding; a lower header version allowed by at-or-below wording or an unadopted different TMATS-file edition is not proof of conflict. Keep setup and data-type version evidence distinct and applicable only where defined. Basis vocabulary and override precedence follow TMATS ADR-0028; no fallback assumption selects EXT.

**Source**: "EXT" (106-04 §10.6.3.3 Figure 10-15, printed 10-30 / full PDF 288; EXT definitions printed 10-31 / full PDF 289); "SRC" (106-05 Figure 10-16a, printed 10-37 / PDF 41). "a value at or below the release version of the standard applied to the data types" (106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14). No setup version: 106-04 §10.6.7.2.2/Figure 10-34, printed 10-48 / full PDF 306; "Reserved. (Bits 31-0) are reserved." (106-05 Figure 10-34b, printed 10-55 / PDF 59); "CH10VER" (106-07 Figure 10-50, printed 10-72 / PDF 76, only 0x07 assigned). G\106 is "VERSION OF IRIG 106 STANDARD USED TO GENERATE THIS TMATS FILE" (106-04 Table 9-1, printed 9-8 / full PDF 186). Basis vocabulary: "override", "declared", "fallback" (TelemetryWorks/irig106-tmats `docs/adr/0028-edition-declarations-and-validation-basis.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`). Owner 2026-11-13, Q-EDITION: explicit caller override wins with contradictory evidence retained side by side; without override use opt-in TMATS only when uncontradicted, else unresolved. The earlier conflicting-evidence brief applied to the no-override case, as clarified in review. ADR-0028: "No automatic conflict finding"; "differing declarations are shown side by side" (same pinned commit, lines 59–63; caller-override precedence context line 11). STANDARD-REVIEW T-6; 106-24R1 Table 9-2 printed 9-10 / PDF 16 (index 9-9 / PDF 15); analysis: `docs/analysis/Q-EDITION-2026-11-13.md`.

**Verification Method**: Test (T)

**Planned verification**: Header 0x01 alone -> unresolved/raw bits, no SRC/None labels; 106-04/05 reserved setup CSDW never becomes a version field. Explicit 106-04 -> EXT/basis override; explicit 106-05 -> SRC/basis override. Explicit override wins even with contradictory opted-in G\106 or header data-type-version evidence: retain all evidence side by side in the basis. Without override, G\106=04 without opt-in -> metadata only/unresolved; with opt-in and no contradictory packet evidence -> EXT/basis declared, assumed from TMATS; opted-in 05 -> SRC with the same assumption label. Without override, opted-in G\106 contradicted by packet evidence -> unresolved plus finding with evidence shown. Missing/malformed usable declarations -> unresolved. Differing declarations alone do not automatically create a conflict; a header at or below the adopted release is only a hint. Later CH10VER 0x07 tests remain distinct; no fictional 106-04/05 setup code or fallback EXT. All are planned fixtures.

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

**Source**: "Time is treated like another data channel" (§11.2.3.2); "Since Time Data, Format 1 packets contain both the absolute input time value and the RTC clock value at the instant the absolute time was valid, these packets can be used to relate RTC values to the input absolute time source" (RCC 123-20 §5.6); contract 5.5. Prototype identifier kept; citation corrected (T-13).

**Verification Method**: Test (T)

**Planned verification**: Contract 7 recording, malformed time packet between valid references, empty recording, setup changes, multiple channels, and repeated finalization/query semantics documented.

#### L2-COR-102

**Parent**: L1-COR-002

**Statement**: The library SHALL keep whole-recording nearest-reference lookup distinct from streaming preceding/interpolating modes; make ties deterministic and record the selection rule.

**Source**: "It is better to use the clock and relative time values from a time packet that occurs near the current data packet as the data file is decoded since there is some drift in the RTC during a recording session" (RCC 123-09 §6.6; T-13); contract 5.2 rule 5; a policy setting (L1-POL-001). Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Exact reference, midpoint tie, before-first default/override, beyond-last, wrap, and no reference from another session or channel. Compare streaming only with the same reference strategy, not offline nearest by accident.

#### L2-COR-103

**Parent**: L1-COR-003

**Statement**: The library SHALL select one time channel per session with separate offline and irrevocable stream rules: offline retains valid-reference-count ranking; stream uses committed Option A without future counts.

**Source**: "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. Retained earlier source: "When multiple time channels are available, it is incumbent on the programmer or data analyst to determine and select the best source of time for a particular data set" (RCC 123-20 §5.6); "It is usually most correct to select one time channel only and to use this channel exclusively" (RCC 123-09 §6.6); contract 5.2 rule 4; a policy setting. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Offline count ranking unchanged; fixed caller choice; one/multiple declared external TIMEIN channels with ID ties; complete inventory lowest-ID fallback; incomplete/unknown inventory counter-only until caller choice; future counts/lost lock never alter prior answers.

#### L2-COR-104

**Parent**: L1-COR-004

**Statement**: The library SHALL measure jumps from consecutive same-channel references using the configured threshold and compatible scales.

**Source**: "there is a jump in input clock time during a recording, such as when GPS locks for the first time, or when an IRIG time source is reprogrammed" (RCC 123-09 §6.6; T-13); contract 5.4; default threshold the format's resolution (L1-POL-001). Prototype identifier kept.

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

**Statement**: The library SHALL retain references and their reported source through lost/regained lock by default; irrevocable stream source changes SHALL NOT trigger channel switching or revision of emitted answers.

**Source**: "the time source shall indicate Internal (0x0)" (106-24R1 §11.2.3.2 note, printed 11-26 / PDF 34). "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. Retained earlier source: "If the time source is external (0x1) and lock on the external source is lost then the time source shall indicate Internal (0x0). Once lock on the external time source is regained, time source shall once again indicate external (0x1)" (§11.2.3.2 a); contract 5.2 rule 6.

**Verification Method**: Test (T)

**Planned verification**: SRC external->internal->external on committed channel; a competing channel gains more references; chosen channel unchanged and transition labelled; stop-on-loss withholds rather than switches.

#### L2-COR-108

**Parent**: L1-COR-008

**Statement**: The library SHALL report chosen-channel reference spacing above 1 s with default gap tolerance 0 ns, separately from the interpolation hold tolerance and any raw-network rate exception.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "unless it is recorded at the raw network rate of the NTP or PTP frames" (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Retained earlier source: "the time packet will be generated at a minimum frequency of 1 hertz" (§11.2.3.2; Chapter 10 §10.6.2); the Format 2 exception "unless it is recorded at the raw network rate" (§11.2.3.3); contract 5.4.

**Verification Method**: Test (T)

**Planned verification**: Spacing at1 s,1 s+1 ns,1.05 s and1.1 s; zero/default and explicit nonzero gap tolerances independent of hold; raw-network exception labelled.

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

**Statement**: The library SHALL include stream mode/channel commitment, hold/gap/cap fallback, and NTP era evidence, rounding, leap ambiguity and table currency in each applicable answer's reproducible basis. Retain the absolute-or-counter-only answer record and every contributing reference.

**Source**: "Eras cannot be produced by NTP directly" and "they can be derived from external means" (RFC 5905 §6, p.12); "if the client is set within 68 years of the server" (§6, p.15); "0 h 1 January 1900 UTC" (§6, p.13). Evidence precedence and labelled era-0 fallback are owner policy (2026-11-12), specified in RDG-012. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. Retained earlier source: Contract 3.1 (with the precision of a reference: "consistent with the resolution with the time packet body format (10 milliseconds [ms] as measured by the 10-MHz RTC)", §11.2.3.2), 3.2, 3.5, 7.3; ADR-0011.

**Verification Method**: Test (T)

**Planned verification**: Reproduce answers from basis only; declared/pivot/default era distinguishable; lost-lock channel stable; cap/timeout/EOF fallback labelled; ambiguity and stale/missing table survive serialization.

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

**Statement**: The library SHALL represent cited defaults and every accepted stream policy setting explicitly: standard mode, committed Option A channel selection, 100 ms hold tolerance, 0 ns gap tolerance, caller progress and caller-set resource caps.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader.

**Verification Method**: Test (T)

**Planned verification**: Policy snapshots distinguish offline and stream, default/override values, separate gap/hold fields, explicit caps and progress, no host-clock reads or hidden unbounded queues.

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

**Statement**: The library SHALL default streaming to standard (preceding-reference) while allowing explicit interpolate selection; keep offline nearest as a separate accepted default.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Verification Method**: Test (T)

**Planned verification**: Omitted mode selects Standard; explicit Standard/Interpolate; RCC example1000000->1150000 yields15 ms; offline-nearest remains distinct and every answer retains selected mode.

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

**Statement**: The library SHALL compare declarations and packets only using RDG-001..006 mappings, keeping source changes informational and inferred I mappings labelled. For absent required TFMT always report the missing attribute; by default skip its comparison. Only caller opt-in enables comparison using A, labelled default-derived, retaining the missing finding and raw absence.

**Source**: Contract 3.7, 6.4; the source may change legitimately (§11.2.3.2 a). "R/R Ch 10 Status: R"; "Default: A" (106-24R1 Chapter 9 Table 9-4 TFMT row, printed 9-72 / PDF 78). "A value of “R” requires that the attribute be specified in the TMATS file whenever the attribute is allowed." (§9.5.1 a, printed 9-5 / PDF 11). "If the R/R Ch10 Status field is “R”, then the attribute must be included in the TMATS file if all other conditions apply even if it has a default."; "Default: This identifies the default value required to process a TMATS file when the file itself does not contain the attribute." (printed 9-6 / PDF 12). RDG-002; owner 2026-11-13, Q-TFMT-DEFAULT = opt-in labelled default. TelemetryWorks/irig106-tmats `docs/adr/0023-validation-passes-and-effective-values.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: effective resolver "never writes to the document". Opt-in comparison and provenance are owner policy; no defaults are inserted.

**Verification Method**: Test (T)

**Planned verification**: Each declared/observed mapping and mismatch, missing fields, configuration boundary, I inference warning (I ↔ 0x3 labelled inferred, owner Q1), and external->internal loss of lock information (TF-011). Every accepted letter/FMT pair in both directions, all mismatches and unknown/non-TIMEIN cases. Explicit A compares as explicit. Absent TFMT without opt-in raises missing-required finding and performs no comparison. Absent TFMT with opt-in compares default A to FMT 0x1/mismatch, labels default-derived and still raises missing-required finding. Verify raw declaration remains absent; no default insertion. These comparison tests belong to the later declaration/report implementation, not this targeted flag-code change. Source changes remain informational and setup-range boundaries remain traceable.

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

**Statement**: The project SHALL specify and verify RDG-002: use its accepted TFMT Format 1 named-letter/FMT mapping. Missing required TFMT SHALL always be reported; by default make no TFMT comparison when absent. Only on explicit caller opt-in use Table 9-4 default A for comparison, labelled default-derived, while preserving the missing-required-attribute finding and raw missing state. Explicit declarations remain explicit; never insert a default into TMATS. Record published default A separately from the explicit/missing declaration even when default comparison is disabled.

**Source**: 106-24R1 Chapter 9 Table 9-4, printed9-72/PDF78; Chapter 11 §11.2.3.2 a; RDG-002.

**Verification Method**: Inspection (I)

**Source supplement**: "R/R Ch 10 Status: R"; "Default: A" (106-24R1 Chapter 9 Table 9-4 TFMT row, printed 9-72 / PDF 78). "A value of “R” requires that the attribute be specified in the TMATS file whenever the attribute is allowed." (§9.5.1 a, printed 9-5 / PDF 11). "If the R/R Ch10 Status field is “R”, then the attribute must be included in the TMATS file if all other conditions apply even if it has a default."; "Default: This identifies the default value required to process a TMATS file when the file itself does not contain the attribute." (printed 9-6 / PDF 12). RDG-002; owner 2026-11-13, Q-TFMT-DEFAULT = opt-in labelled default. TelemetryWorks/irig106-tmats `docs/adr/0023-validation-passes-and-effective-values.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: effective resolver "never writes to the document". Opt-in comparison and provenance are owner policy; no defaults are inserted.

**Planned verification**: Every accepted letter/FMT pair in both directions, all mismatches and unknown/non-TIMEIN cases. Explicit A compares as explicit. Absent TFMT without opt-in raises missing-required finding and performs no comparison. Absent TFMT with opt-in compares default A to FMT 0x1/mismatch, labels default-derived and still raises missing-required finding. Verify raw declaration remains absent; no default insertion. These comparison tests belong to the later declaration/report implementation, not this targeted flag-code change. Cite Table 9-4 printed 9-72/PDF 78 visual check.

#### L2-RDG-203

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-003: Require tests for every TFMT letter/FMT pair, including the accepted inferred I->0x3 pair and the RCC 124-22 Chapter 7 example.

**Source**: Owner 2026-11-11 ROADMAP question 1; 106-24R1 Table 9-4 and§11.2.3.2 a; RCC 124-22 Chapter7 printed7-3/PDF 149; RDG-002/003.

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

**Planned verification**: Fixtures per touched archived edition:106-05/07/13/15/17 and current; every ITS value, non-IRIG FMT and missing edition. Include archived 106-04 reserved bits 31–12 and EXT bits 3–1; edition uncertainty is preserved under L2-EDN-201.

#### L2-RDG-208

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-008: Retain the suspect byte-checksum reading and diagnose word-only-valid headers until local recordings settle it.

**Source**: 106-24R1 §11.2.1.2 c and§11.2.1.1 j; RCC 123-20 Appendix A-2; RDG-008.

**Verification Method**: Inspection (I)

**Planned verification**: Byte-only, word-only, both, neither across the listed editions; reference handbook Appendix A-2 code; local recordings remain outside Git/CI and human Review pending.

#### L2-RDG-209

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-009: Assign stated and assumed scales separately, with GPS=TAI-19s as a recorded inference.

**Source**: 106-24R1 §11.2.3.2 a/§11.2.3.3; RCC 200-16 §1/Appendix A.2; RDG-009.

**Verification Method**: Inspection (I)

**Planned verification**: All FMT/NTF values, IRIG/RTC overrides, historical table transitions, cross-scale reference comparisons and stale table.

#### L2-RDG-210

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-010: Pin the accepted UTC leap-second reading with boundary and scale tests rather than accepting second 60 everywhere.

**Source**: 106-24R1 Figure 11-13; RCC 200-16 §3.6/Appendix A.2; RDG-010.

**Verification Method**: Inspection (I)

**Planned verification**: Known/unknown year, June 30/December 31 candidates and neighbours,23:59 versus other times, UTC/assumed UTC/TAI/GPS/unknown.

#### L2-RDG-211

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-011: Check first dynamic time packet per session as the accepted stricter reading of the recording/session wordings.

**Source**: 106-24R1 §11.2.3.2/§11.2.3.3, Chapter 10 §10.6.2, §11.2.1.1 i; RDG-011.

**Verification Method**: Inspection (I)

**Planned verification**: Two sessions each compliant, missing time in the second, static-only prefix, time None and malformed first time packet.

#### L2-RDG-212

**Parent**: L1-RDG-002

**Statement**: The project SHALL specify and verify RDG-012: UTC-following NTP reckoning without cumulative TAI-UTC adjustment, evidenced/policy-assumed era resolution, zero sentinel, leap ambiguity, accepted floor rounding and PTP last-known-offset fallback with TF-021.

**Source**: "2,208,988,800" (RFC 5905 §6 Figure 4, p.14, 1 Jan 1970); "a value of zero is a special case representing unknown or unsynchronized time" (§6, p.14); "knowledge of all previous leap seconds is lost" (RFC 1305 Appendix E.8, p.78); Figure 8 (p.79) repeats the same NTP number for UTC second 60 and midnight. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012.

**Verification Method**: Inspection (I)

**Planned verification**: Written all-edition analysis and RFC page checks; each listed edge gets a future failing test/fixture; floor rounding is accepted (owner 2026-11-13), and human Review remains pending.

## STRM

#### L2-STRM-101

**Parent**: L1-STRM-001

**Statement**: The library SHALL provide bounded, irrevocable streaming with default standard and selectable interpolate, committed Option A channel choice, explicit caller progress and caps; do not equate stream ranking with future-aware offline ranking.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader.

**Verification Method**: Test (T)

**Planned verification**: Long fixed-rate and burst streams remain within caps; incomplete advance channel inventory yields counter-only; later references/counts/lock changes cannot rewrite output; caller progress and EOF drive release.

#### L2-STRM-201

**Parent**: L1-STRM-001

**Statement**: The library SHALL in the default standard mode, answer immediately from the selected channel's latest eligible preceding reference without waiting for a future reference.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Verification Method**: Test (T)

**Planned verification**: RCC 123-20 example day 10012:30:25.000 RTC1000000 and query1150000 -> .015; explicit/default Standard, future-only reference, other-channel references and no usable preceding time.

#### L2-STRM-202

**Parent**: L1-STRM-001

**Statement**: The library SHALL when interpolate is explicitly selected, hold for the next compatible same-channel reference within the accepted hold/cap rules and interpolate only between valid bracketing references.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Verification Method**: Test (T)

**Planned verification**: 1 s RTC interval and1.000010 s absolute interval -> proposed midpoint0.500005 s; explicit Interpolate, endpoints, rounding proposal, wrap/equal counters and incompatible scale; timeout/cap/EOF fallback.

#### L2-STRM-203

**Parent**: L1-STRM-001

**Statement**: The library SHALL bound interpolate hold to the 1 s reference interval plus a default 100 ms policy tolerance, measured only by explicit caller progress; at timeout or EOF release using the preceding reference as extrapolated.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader.

**Verification Method**: Test (T)

**Planned verification**: Next reference at1 s,1 s+1 ns,1.05 s,at deadline/after deadline; gap finding inside hold; explicit progress stalls, EOF and no usable preceding reference; hold override remains labelled policy.

#### L2-STRM-204

**Parent**: L1-STRM-001

**Statement**: The library SHALL in both modes, packets before any usable preceding time reference get no absolute time; time source None supplies no absolute reference.

**Source**: Owner 2026-11-11 ROADMAP question 2; 106-24R1 Chapter 10 §10.6.2 first dynamic packet; Chapter 11 §11.2.3.2 source None/invalid FMT. Missing-time fallback is product policy; the standard gives no such algorithm.

**Verification Method**: Test (T)

**Planned verification**: Data-before-time, first time packet SRC None with a valid-looking body, FMT NONE, TS invalid, no time until EOF, and usable reference arriving later must not retroactively invent an early absolute answer.

#### L2-STRM-205

**Parent**: L1-STRM-001

**Statement**: The library SHALL require caller-set pending packet/byte and reference-state caps with no hidden unbounded queue; on cap, timeout or EOF release pending answers as extrapolated from the preceding reference, or counter-only when none is usable.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Verification Method**: Test (T)

**Planned verification**: Packet/byte cap exactly at/one beyond limit, reference flood, data burst, timeout/EOF, no reference, invalid cap configuration and discontinuity; no state exceeds caller caps and release reason is retained.

#### L2-STRM-206

**Parent**: L1-STRM-001

**Statement**: The library SHALL make mode changes explicit, preserve the original policy/fallback on released answers, and obey the committed stream channel decision without future count ranking.

**Source**: "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Verification Method**: Test (T)

**Planned verification**: Mode change with pending data preserves old answer basis and resource bounds; explicit channel commitment and no count-based or lost-lock reassignment; no retroactive answer changes.

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

**Source**: §11.2.3.2 b. Prototype identifier kept.

**Verification Method**: Test (T)

**Planned verification**: Digits A..F at each BCD position; invalid hour 24 with valid digits yields range error instead; zero-mask violations produce their own finding.

#### L2-ERR-103

**Parent**: L1-ERR-003

**Statement**: The library SHALL apply field and calendar range validation before constructing time values, with the RDG-010 exception explicitly isolated.

**Source**: Figures 11-13, 11-14; RDG-010. Prototype identifier kept; day 0 is rejected, never turned into day 1 (contract 8.3); second 60 is no longer always rejected (T-16).

**Verification Method**: Test (T)

**Planned verification**: Every minimum/maximum and one out-of-range value, Feb 29 in leap/common years, day 0, second 60 combinations, and no silent day 0->1 repair.

#### L2-ERR-104

**Parent**: L1-ERR-004

**Statement**: The library SHALL surface secondary checksum failure as a typed validation result and a finding when attached to a recording.

**Source**: §11.2.1.2 c. Prototype identifier kept.

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

**Source**: ADR-0014. Prototype identifier kept.

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

**Statement**: The library SHALL keep irig106-types as the library's only ecosystem dependency and accept all packet/TMATS context through plain values.

**Source**: ADR-0005. Replaces L1-API-003 ("zero required runtime dependencies").

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

**Statement**: The CLI SHALL expose host-callable commands and renderers so irig106-cli can mount the same implementation.

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

**Statement**: The CLI SHALL use five documented exit classes: 0 success; 1 findings at or above the chosen severity; 2 usage error; 3 input could not be read or is not valid Chapter 10/time input; 4 output could not be written. These shared ecosystem assignments SHALL agree with `irig106 tmats` and SHALL NOT be renumbered. Document classification precedence; usage validation precedes input access and input/output failures take precedence over finding-only outcomes.

**Source**: As `irig106-tmats` L1-CLI-006 (the existing L1 source); TelemetryWorks/irig106-tmats, `docs/CLI.md` §4, lines 88–91, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: "`0` success; `1` the check failed (`verify` mismatch, `validate` findings at or above the failure threshold); `2` usage error; `3` input could not be read or is not TMATS/Chapter 10; `4` output could not be written. To be fixed before 0.1 and never renumbered." Owner 2026-11-12, Q-CLI-EXIT = align with irig106-tmats; findings/severities ADR-0013, hand-rolled parser ADR-0008. IRIG 106 is silent on tool exit codes: no standard source exists. This is a shared ecosystem convention so `irig106 tmats` and `irig106 time` agree; code 3 specializes the domain to Chapter 10/time input. See `docs/analysis/Q-CLI-EXIT-2026-11-12.md`.

**Verification Method**: Test (T)

**Planned verification**: One test per numeric code: 0 successful output with no threshold findings; 1 finding at/above the chosen severity, including threshold changes; 2 hand-rolled parser usage errors (ADR-0008), including unknown options, missing arguments and usage error before opening a nonexistent input; 3 unreadable input and invalid Chapter 10/time input; 4 injected output-write failure. Verify the shared runner returns the classification for host invocation without exiting the process; input/output failure must not be reported as success or only findings. Code/tests belong to the CLI restructure step, with /// Requirements: markers for L1-CLI-007, L2-CLI-107 and L3-CLI-107, correct Traces tags and regenerated matrix linking requirement → code → test; these are planned checks, not executed tests.

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

**Accepted Q-FLAGS-ERROR, owner 2026-11-13:** TF-028 "Secondary-derived time has an undefined format" is ERROR and maps from UndefinedSecondaryTimeFormat. It reports selected or present secondary time with format 11; RTC stamps remain usable. TF-006 remains the separate reserved-VALUE warning, TF-005 reserved-BITS warning, and TF-015 missing-required-header error. With 0x4C, TF-015/TF-028/TF-006 evidence can coexist. TF-001..TF-027 are unchanged; the former numeric finding-assignment open item is resolved.
