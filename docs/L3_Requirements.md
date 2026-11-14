# irig106-time — Level 3 Requirements

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

L3 states implementation obligations and planned verification cases, not
Rust source changes. Each entry names its immediate L2 parent and its L1
ancestor explicitly. Names are responsibility boundaries, not a commitment
to a new public type or dependency. See L2 for all nine accepted 2026-11-12/13 decisions; Q-TFMT-DEFAULT
is opt-in labelled default and Q-FLAGS-ERROR is TF-028. NTP floor rounding is accepted (owner 2026-11-13); Chapter 4 context is settled while characters/bits wording is not.

## 0.8.0 plan decisions accepted 2026-11-13

Owner: "accept your recommendations"; the coordinator's explicit scope and
finding-level qualifications are recorded in [the plan decision record](analysis/plan-decisions-2026-11-13.md).
TF-029/030 are WARNING; TF-031/032 IDs/titles are permanent but their levels
remain pending owner confirmation in S03/S08. C1–C4 are accepted.

Release-scoped deferrals only: COR-111 inverse queries; the new complete
ENC-103 secondary/stamp encoder API; API-110 complex timeline-state
rehydration only; REL-106 prebuilt binary distribution. Requirements remain,
not marked covered. Existing encoders/byte-sum behaviour, validated existing
public value loads, complete versioned outputs and every other requirement
remain 0.8.0 blockers. Offline drift-corrected query mode is after 0.8.0.

0.8.0 is crates.io-only, both crates in lockstep, via Trusted Publishing/OIDC
from GitHub Actions: no stored publishing token, no local publish, no prebuilt
binaries. JSON uses a versioned reviewed schema, units and decimal strings
for 64/128-bit quantities; supported loads validate and retain provenance.
CLI keeps classes 0–4; usage precedes input, first fatal operational failure
wins (3 input/4 output), later failures are diagnostics. These are owner/product
policies; IRIG 106 does not prescribe release authentication, JSON or exit codes.
This documentation implements none of them and starts no S01 work.

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

## RTC

**L3-RTC-101** · Parent: L2-RTC-101 · Verification: T

**L1 ancestor**: L1-RTC-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use irig106-types::Rtc; the valid raw range is 0 through 0xFFFF_FFFF_FFFF and one tick is 100 ns. Out-of-width construction must follow an explicitly named checked or masking operation, never an implicit calendar interpretation.

**Source**: "This is a free-running 10-MHz binary counter represented by 48 bits that are common to all data channels" (§11.2.1.1 i). Prototype identifier kept.

**Planned verification**: Raw zero, maximum, one-bit boundaries, and a value with bit 48 set; confirm the chosen constructor contract and that no calendar date is produced.

**L3-RTC-102** · Parent: L2-RTC-102 · Verification: T

**L1 ancestor**: L1-RTC-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Read header bytes [16..22) or the six supplied counter bytes as little-endian. For a counter-form stamp consume exactly eight bytes, decode [0..6), and report nonzero [6..8) as reserved padding; never reinterpret them as counter bits.

**Source**: §11.2.1.1 i, Table 11-2; "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b). Prototype identifier kept, widened to time stamps.

**Planned verification**: Distinct bytes 01 02 03 04 05 06 decode to 0x060504030201; truncate at every byte; vary both padding bytes independently.

**L3-RTC-103** · Parent: L2-RTC-103 · Verification: T

**L1 ancestor**: L1-RTC-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compute the modular difference modulo 2^48 in a wider signed representation and unwrap against session ordering. A short forward crossing of maximum to zero stays positive; do not infer an elapsed duration of multiple wraps from two raw counters. At a half-range tie, report ambiguity rather than choose a sign silently.

**Source**: 2^48 ticks at 10 MHz is about 325.8 days; the counter "shall remain free-running during each session" (§11.2.1.1 i). Prototype identifier kept.

**Planned verification**: Maximum to zero is +1 tick and reverse is -1; equal counters are zero; exercise half-range ambiguity and reject a cross-session subtraction.

**L3-RTC-104** · Parent: L2-RTC-104 · Verification: T

**L1 ancestor**: L1-RTC-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Multiply unwrapped signed ticks by 100 using checked i128 intermediates, returning a typed range error if the destination cannot hold the result; preserve sign.

**Source**: §11.2.1.1 i (10 MHz). Prototype identifier kept.

**Planned verification**: Zero, +1, -1, maximum session-supported delta, and destination overflow; 150000 ticks equals 15000000 ns.

## ABS

**L3-ABS-101** · Parent: L2-ABS-101 · Verification: T

**L1 ancestor**: L1-ABS-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Calendar values must validate day/month/year and nanoseconds before construction; preserve second=60 as a distinct labelled UTC leap second under RDG-010, not as the next midnight. Day zero is invalid. Arithmetic crossing year-end updates a known year and preserves unknown-year provenance otherwise.

**Source**: IEEE 1588 and ERTC times resolve 1 ns (§11.2.1.1 g); the day format carries no year (Figure 11-13); "excluding leap second days when a second may be added or subtracted" (RCC 200-16 §3.6); contract 3.2; T-16. Prototype identifier kept.

**Planned verification**: Day 0/1/365/366, Gregorian 1900/2000/2100, unknown year, midnight and year-end, 23:59:59 to 23:59:60 to 00:00:00 with explicit leap context.

**L3-ABS-102** · Parent: L2-ABS-102 · Verification: T

**L1 ancestor**: L1-ABS-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Eight-byte layout: [0..2) reserved, [2..4) microseconds, [4..6) low, [6..8) high, each u16 LE. Compute ns=((high*65536+low)*10000000)+microseconds*1000, usec<=9999. Keep T-10's DERIVED full binary count including low bit15 despite the shared-row zero; never split a binary day. Reserved words are separate TF-005 warnings, not discarded values. Absolute calendar reconstruction requires caller origin under the accepted Q-CH4 decision; absent origin returns labelled partial time, never day 1/year/origin from a nearby packet.

**Shared implementation prerequisite**: Do not use c59e004 Ch4BinaryTime::from_secondary_bytes or its delegating from_intra_packet_bytes as-is: they swap high/usec relative to Figure 11-4. Per accepted ADR-0015 the shared fix is merged in irig106-types 48f1834 (types PR #4): reserved/usec/low/high layout and full binary count. Adopt that corrected revision in the separate code slice; this documentation change does not update dependencies.

**Source**: "resolution of 655.36 seconds", "10 milliseconds", "three least significant bits of the BCD Julian date", "at a rate necessary to resolve time ambiguities" (106-24R1 Chapter 4 §4.7, printed 4-11 / PDF 15); RCC 123-20 `enI106_Ch4Binary2IrigTime`, printed B-43 / PDF 149, applies word weights without an origin. Explicit caller context and partial results are owner policy (2026-11-12); historical characters/bits wording remains unresolved (T-10). Figure 11-4, printed 11-10 / PDF 18; raw layout already confirmed in T-10 and adopted from types 48f1834. Retained earlier source: "00 = Chapter 4 binary weighted 48-bit time format. The two lsbs of the 64-bit packet secondary header time and IPTS shall be zero-filled" (§11.2.1.1 g); Figure 11-4; Chapter 4 §4.7. Layout confirmed 2026-11-12, T-10. Prototype identifier kept.

**Planned verification**: Retain distinct-word and low=0x8000 fixtures and usec9999/10000; raw values survive reserved warnings; no-origin result stays labelled partial, explicit-origin reconstruction is separately qualified. Retained raw-layout fixtures: Hand-derived 00 00 0f 27 34 12 01 00 gives 701969999000 ns; nonzero reserved bytes, microseconds=9999/10000, high=0xFFFF, short buffers, and missing calendar origin. T-10 must fail on the prototype. Independent top-bit fixture: `00 00 00 00 00 80 00 00` -> low=0x8000, high=usec=0 -> 327.68 s (327680000000 ns); accept under the DERIVED binary reading of T-10, and retain BCD bit 15=0. The low=0x1234 fixture alone cannot reject the opposing reading.

**L3-ABS-103** · Parent: L2-ABS-103 · Verification: T

**L1 ancestor**: L1-ABS-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Decode little-endian nanoseconds at [0..4) and seconds at [4..8); encode the same. Require nanoseconds <1000000000. Carry format and scale context into calendar conversion rather than assuming that every IEEE-1588-looking value is UTC.

**Source**: "The 32 bits indicating seconds shall be placed into the MSLW portion of the secondary header and the 32 bits indicating nanoseconds shall be placed into the LSLW portion" (§11.2.1.1 g); Figure 11-5. Prototype identifier kept.

**Planned verification**: Distinct seconds=0x01020304 and ns=123456789; ns=999999999/1000000000; compare with a Format 2 body containing seconds first.

**L3-ABS-104** · Parent: L2-ABS-104 · Verification: T

**L1 ancestor**: L1-ABS-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Read low u32 then high u32, combine to u64, and expose one nanosecond per tick through irig106-types. Correlation requires a session reference, not a presumed Unix epoch; compare RTC with integer Ertc/100 when both are available.

**Source**: "64-bit binary extended relative time counter (ERTC) with 1-nanosecond resolution … (RTC = ERTC/100)" (§11.2.1.1 g); Figure 11-6 (LSLW, MSLW); T-5. Prototype identifier kept; the resolution is now stated.

**Planned verification**: ERTC=1 gives 1 ns, 100 gives 100 ns; max u64 and asymmetric bytes; T-5 regression must distinguish the prototype factor of 100.

## IPT

**L3-IPT-105** · Parent: L2-IPT-105 · Verification: T

**L1 ancestor**: L1-IPT-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Extract presence=(flags>>7)&1, source=(flags>>6)&1, sync_error=(flags>>5)&1 and format=(flags>>2)&3; keep reserved format as an explicit value. Bit 2 is never a presence indicator.

**Source**: "Bit 7: Indicates the presence or absence of the packet secondary header"; "Bit 6: Indicates the IPTS time source. 0 = Packet header 48-bit RTC. 1 = Packet secondary header time (bit 7 must be 1)"; "Bits 3-2: Indicate the packet secondary header time format" (§11.2.1.1 g); T-1. Replaces L1-IPT-001 to 004.

**Planned verification**: All 256 flag bytes; compare flags differing only in unrelated bits. Cite T-1 on fixtures.

**L3-IPT-106** · Parent: L2-IPT-106 · Verification: T

**L1 ancestor**: L1-IPT-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Source=0 dispatches to the padded RTC reader; source=1 and presence=1 dispatches 00 to Chapter 4, 01 to IEEE 1588, 10 to ERTC. Use the same value readers as secondary headers, without expecting the header checksum in a stamp.

**Source**: "These 8 bytes contain time in either 48-bit RTC format (plus 16 high-order zero bits) or 64-bit format as specified in the packet flags in the packet header" (§11.2.1.3 b); §11.2.1.1 g; T-1.

**Planned verification**: Every source/presence/format combination and each short length; padded RTC and each secondary representation from independently authored bytes.

**L3-IPT-107** · Parent: L2-IPT-107 · Verification: T

**L1 ancestor**: L1-IPT-007

**Obligation**: The implementation SHALL extract source=(flags>>6)&1, presence=(flags>>7)&1 and format=(flags>>2)&3. Reject source=1/presence=0 as TF-015 (error), and source=1/format=3 as TF-028 undefined-format error with separate TF-006 warning; return no stamp time for either, retaining evidence of both when both apply. For source=0 decode exactly the low 48 counter bits with 16 high-order padding bits; format=3 yields TF-006 warning and keeps the RTC value. If presence=1/format=3, report the secondary header time as unreadable separately as TF-028; this error does not invalidate the RTC stamp. Other flags and reserved padding findings remain separate. Severity overrides must not create a time from an undefined format.

**Source**: "0 = Packet header 48-bit RTC."; "1 = Packet secondary header time (bit 7 must be 1)."; "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved" (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17); "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b, printed 11-11 / PDF 19). Value retention is owner 2026-11-12, Q-FLAGS; TF-006 reserved-value warning and unchanged TF-005/TF-006 levels are confirmed owner 2026-11-13; page images and older equivalent wording: `docs/analysis/Q-FLAGS-2026-11-12.md`. Owner 2026-11-13, Q-FLAGS-ERROR accepts TF-028 "Secondary-derived time has an undefined format" at error level, architecture section 8; UndefinedSecondaryTimeFormat maps to TF-028. TF-001..TF-027 remain unchanged.

**Planned verification**: Exhaust all 256 flag bytes against the source/presence/format rule, independently of other flag findings. Literal cases 0x0C and 0x8C decode RTC with TF-006; 0x8C also reports unreadable secondary time separately; it is TF-028 (error). 0x4C and 0xCC return no stamp time/errors for undefined secondary format, with 0x4C also lacking the required header; 0x40 returns no stamp time/error for absent required header. Use hand-authored padded RTC bytes, short lengths and nonzero padding as separate validation cases. Use an independent hand-transcribed truth table for diagnostics and exact literal error-variant assertions. Flag decoding tests are implemented; future findings emission/report integration remains separate.

## SEC

**L3-SEC-101** · Parent: L2-SEC-101 · Verification: T

**L1 ancestor**: L1-SEC-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Require 12 bytes; time=[0..8), header-reserved=[8..10), checksum=[10..12). For format 00 invoke L3-ABS-102 after checksum verification, preserving separate findings for its internal reserved word and header-reserved bytes.

**Source**: "The length of the packet secondary header is fixed at 12 bytes (96 bits)" (§11.2.1.2); Figure 11-4. Prototype identifier kept.

**Planned verification**: Twelve bytes with distinguishing words and an independently computed checksum; lengths 0 through 11; trailing body bytes must not enter the checksum.

**L3-SEC-102** · Parent: L2-SEC-102 · Verification: T

**L1 ancestor**: L1-SEC-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Reuse L3-ABS-103 on the first eight bytes after validating the twelve-byte header. Associate its time with the first data bit unless the caller supplies a cited data-type exception.

**Source**: §11.2.1.2 a; Figure 11-5. Prototype identifier kept.

**Planned verification**: Seconds/nanoseconds in different values, boundary nanoseconds, bad checksum, and a packet-specific applicability supplied as metadata.

**L3-SEC-103** · Parent: L2-SEC-103 · Verification: T

**L1 ancestor**: L1-SEC-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Reuse L3-ABS-104 after header validation; preserve the raw ERTC even when no absolute reference exists and report any RTC/ERTC disagreement separately.

**Source**: §11.2.1.2 a; Figure 11-6. Prototype identifier kept.

**Planned verification**: ERTC=1/99/100 and all-high bits; checksum failure suppresses trusted own time, while absent references must not fabricate absolute time.

**L3-SEC-104** · Parent: L2-SEC-104 · Verification: T

**L1 ancestor**: L1-SEC-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compute byte_sum=sum(header[0..10)) modulo 65536 and word_sum=sum(five little-endian u16 words) modulo 65536; compare both with the little-endian stored checksum. Only byte-sum validity admits trusted own time; word-only validity records TF-027 and TF-012 without silently switching algorithms.

**Source**: "a 16-bit arithmetic sum of all secondary header bytes excluding the secondary header checksum word" (§11.2.1.2 c); whether "bytes" means bytes or 16-bit words is open (T-15: the prototype sums words; the words of the standard and RCC 123-20's `uCalcSecHeaderChecksum` sum bytes); contract 6.6. Prototype identifier kept; the reading is settled in L2.

**Planned verification**: Headers matching byte only, word only, both, neither, and short input; all-edition synthetic cases plus local-only real recordings remain prerequisites to settling the suspect reading.

**L3-SEC-105** · Parent: L2-SEC-105 · Verification: T

**L1 ancestor**: L1-SEC-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Inspect [8..10) as header-reserved bytes for all formats and [0..2) additionally for Chapter 4 time. Emit TF-005 with field position and raw bits; do not overwrite bytes and claim a clean header.

**Source**: "Reserved. These 2 bytes are reserved and shall be zero filled" (§11.2.1.2 b).

**Planned verification**: Set each reserved bit singly; distinguish a nonzero time-reserved word from header-reserved bytes; checksum recomputed by hand to isolate reserved-bit handling.

**L3-SEC-106** · Parent: L2-SEC-106 · Verification: T

**L1 ancestor**: L1-SEC-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Track observed secondary formats by channel across the recording; compare validated flag formats, emit TF-013 on mixed formats, and retain the format on each result. Missing secondary headers do not imply format 00.

**Source**: "all channels that have a secondary header must use the same time source in bits 2-3 of the packet flags" (§11.2.1.2 a); contract 6.6.

**Planned verification**: Two same-format channels, each pair of different formats, absent headers, and a format change on a channel.

## CSDW

**L3-CSDW-101** · Parent: L2-CSDW-101 · Verification: T

**L1 ancestor**: L1-CSDW-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Require four bytes and decode one little-endian u32. Resolve layout through L3-EDN-201, then expose tagged EXT or SRC, FMT, LY, DATE, applicable ITS and the edition-specific reserved mask. Unknown layout preserves raw competing meanings; body dispatch uses DATE only after validity and edition checks.

**Source**: §11.2.3.2 a; Figure 11-12; Table 11-4. Prototype identifier kept. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Planned verification**: One-bit field walking and asymmetric bytes; data type 0x11 dispatch; every truncated word length.

**L3-CSDW-102** · Parent: L2-CSDW-102 · Verification: T

**L1 ancestor**: L1-CSDW-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Resolve layout under L3-EDN-201 first. For 106-04 expose LegacyExt { external_present: (word & 1)!=0, reserved: word & 0xE }; report nonzero reserved bits, never interpret the nibble as TimeSource. From 106-05 use TimeSource::from_raw(word & 0xF): 0 Internal, 1 External, 2 InternalFromRmm, 15 None, 3..14 Reserved(raw). In particular Reserved(3) never becomes Gps or legacy None. Unknown layout preserves raw EXT/SRC alternatives without assigning a source.

**Source**: §11.2.3.2 a, "0x3 – 0xE = Reserved"; T-6. Prototype identifier kept; value 3 is no longer "GPS". Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Planned verification**: Resolved 106-04: all sixteen EXT nibbles, with bit 0 present/absent and every reserved bit 3–1; raw 3 is present plus reserved-bit evidence, never None. Resolved 106-05+: all sixteen SRC/FMT values, including Reserved(3), InternalFromRmm for 2 and FMT None for 15. Unknown/conflicting edition plus header 0x01 must not silently select EXT; a later explicit declaration with header 0x01 must not be treated as 106-04; never invent a legacy setup version code.

**L3-CSDW-103** · Parent: L2-CSDW-103 · Verification: T

**L1 ancestor**: L1-CSDW-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: After L3-EDN-201, decode (word>>4)&0xF as B/A/G/RTC/UTC-from-GPS/native-GPS for 0..5. In 106-04, 6..15 are reserved; do not call shared from_raw(15) and promote its modern None meaning. From 106-05, 6..14 are reserved and 15 is TimeFormat::None (invalid payload). Preserve raw fields on unresolved layout. GPS is a format/scale, never SRC=3.

**Source**: "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); T-7. Prototype identifier kept. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Planned verification**: All sixteen FMT values for resolved 106-04 and 106-05+, checking FMT 15 reserved versus None; unresolved edition must not choose a meaning by enum conversion alone.

**L3-CSDW-104** · Parent: L2-CSDW-104 · Verification: T

**L1 ancestor**: L1-CSDW-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Extract bit 8 as recorded metadata, not as a replacement for a Gregorian leap-year calculation. Compare day 366 and a known year with LY in the timeline and emit TF-007 on contradiction.

**Source**: "Bit 8: Indicates if this is a leap year" (§11.2.3.2 a). Prototype identifier kept.

**Planned verification**: LY 0/1 with day 365/day 366 and known leap/common years; extraction must not depend on DATE or source bits.

**L3-CSDW-105** · Parent: L2-CSDW-105 · Verification: T

**L1 ancestor**: L1-CSDW-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: DATE=0 selects Figure 11-13 day-of-year; DATE=1 selects Figure 11-14 calendar-date fields. Body lengths exclude the four-byte data word and any trailer filler.

**Source**: "0 = IRIG day available (Figure 11-13) 1 = Month and year available (Figure 11-14)" (§11.2.3.2 a). Prototype identifier kept.

**Planned verification**: Toggle bit 9 alone on hand-written 6/8-byte payloads; Data Length 10 and 12 respectively; no access to filler.

**L3-CSDW-106** · Parent: L2-CSDW-106 · Verification: T

**L1 ancestor**: L1-CSDW-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Extract (word>>12)&0xF; use the eight §11.2.3.2 a values through IrigTimeSource::from_raw: Freewheeling, TimeCommand, Rmm, ExternalIrig, ExternalGps, ExternalNtp, ExternalPtp, Embedded for 0..7 and Reserved(raw) for 8..15. For non-A/B/G FMT report not-applicable; pre-106-17 report not-reported; absent edition and zero report freewheeling-or-not-reported; 8..15 remain reserved and evidenced.

**Source**: "IRIG Time Source (ITS). Bits 15-12 provide dynamic information regarding the source of time for an internal IRIG time code generator (TCG) when FMT is IRIG-A, B, or G" (§11.2.3.2 a); T-7; the field is reserved before 106-17 and Format 1's data type version did not change (T-14, a reading for the register).

**Planned verification**: Each ITS value with A/B/G and other FMT, declared 106-15/17/24R1/unknown; zero before 106-17 must never become certain freewheeling.

**L3-CSDW-107** · Parent: L2-CSDW-107 · Verification: T

**L1 ancestor**: L1-CSDW-007

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Resolve layout under L3-EDN-201. For 106-04 test mask 0xFFFFFC0E, including EXT reserved bits 3–1. From 106-05 through 106-15 test 0xFFFFFC00; from 106-17 test 0xFFFF0C00 under RDG-007. Unknown layout preserves the competing reserved-bit interpretations rather than claiming a modern source or ITS. Emit TF-005 with raw mask intersections.

**Source**: "All reserved bit fields in packet headers or CSDWs shall be set to zero (0x0)" (§11.2.1 f); §11.2.3.2 a; T-7. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Planned verification**: Set each bit of all three masks independently (106-04 EXT, 106-05–15 SRC, 106-17+ ITS); distinguish reserved bits from valid fields, preserve unknown-layout ambiguity, and never treat old zero ITS bits as certain freewheeling.

## BCD

**L3-BCD-103** · Parent: L2-BCD-103 · Verification: T

**L1 ancestor**: L1-BCD-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use per-word zero masks transcribed from Figures 11-13/14, not from the prototype encoder; retain the offending word/bit positions in TF-005. Only the three/four defined words are inspected.

**Source**: Figures 11-13 and 11-14; Table 11-16, "0 Always zero". Prototype identifier kept.

**Planned verification**: A fixture per figure, each always-zero bit set individually, and adjacent filler made nonzero to prove it is not parsed.

**L3-BCD-104** · Parent: L2-BCD-104 · Verification: T

**L1 ancestor**: L1-BCD-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Extract each figure-defined decimal digit by its mask/shift; check <=9 before multiplication. Bits that do not belong to a digit remain reserved/zero checks, not decimal digits.

**Source**: "the time data words are inserted in the packet in binary-coded decimal format" (§11.2.3.2 b). Prototype identifier kept.

**Planned verification**: All 0..15 digit patterns at every digit position, separating reserved-bit violations from invalid decimal digits.

**L3-BCD-105** · Parent: L2-BCD-105 · Verification: T

**L1 ancestor**: L1-BCD-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Word 0 has tens-ms at bits 0..3, hundreds-ms at 4..7, seconds-units at 8..11 and seconds-tens at 12..14. Word 1 has minutes-units at 0..3, minutes-tens at 4..6, hours-units at 8..11, hours-tens at 12..13. Word 2 has day-units at 0..3, day-tens at 4..7, day-hundreds at 8..9. Other bits are zero.

**Source**: Figure 11-13; Table 11-16; T-12. Replaces L1-BCD-001 (8 bytes).

**Planned verification**: Contract section 7 bytes 35 27 45 13 87 01 decode day 187 13:45:27.350; exactly 6 bytes succeeds, shorter fails; 10 ms steps and second 60 under RDG-010.

**L3-BCD-106** · Parent: L2-BCD-106 · Verification: T

**L1 ancestor**: L1-BCD-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use Figure 11-14: word 0/word 1 time fields as day format; word 2 day-units bits 0..3, day-tens 4..7, month-units 8..11, month-tens 12; word 3 year units at bits 0..3, tens 4..7, hundreds 8..11 and thousands 12..13, with bits 15..14 zero. Derive day-of-year only after validating the month/day/year.

**Source**: Figure 11-14; Table 11-16; T-12. Replaces L1-BCD-002 (10 bytes).

**Planned verification**: Independently packed 2000-02-29 and 1900-02-29; exactly 8 bytes and every shorter length; December 31 rollover; no fifth word read. T-12 regression.

## F2CSDW

**L3-F2CSDW-101** · Parent: L2-F2CSDW-101 · Verification: T

**L1 ancestor**: L1-F2CSDW-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Read one little-endian u32 from four bytes. Expose NTF, TS, and reserved bits; never reuse Format 1 SRC/FMT names or treat NTF as the low nibble.

**Source**: §11.2.3.3 a; Figure 11-15. Prototype identifier kept.

**Planned verification**: Data words 0x00000001, 0x00000011, 0x00000021 represent valid NTP/PTP 2002/PTP 2008; short words and T-2 regression.

**L3-F2CSDW-104** · Parent: L2-F2CSDW-104 · Verification: T

**L1 ancestor**: L1-F2CSDW-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Extract (word>>4)&15: 0=NTPv 3, 1=IEEE 1588-2002, 2=IEEE 1588-2008, 3..15=reserved(raw). Reserved formats produce TF-006 and no decoded body time.

**Source**: "Network Time Format (NTF). Bits 7-4" (§11.2.3.3 a); T-2. Replaces L1-F2CSDW-002.

**Planned verification**: All sixteen values; vary TS independently and verify NTF=2 is accepted as PTP 2008 rather than reserved.

**L3-F2CSDW-105** · Parent: L2-F2CSDW-105 · Verification: T

**L1 ancestor**: L1-F2CSDW-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Extract word&15: 0=not-valid, 1=valid, 2..15=reserved(raw). Emit TF-003 for not-valid and TF-006 for reserved; default timeline admits only TS=1.

**Source**: "Time Status (TS). Bits 3-0 indicate the status of the network time" (§11.2.3.3 a); T-2. Replaces L1-F2CSDW-002.

**Planned verification**: Every status with all three defined NTFs; explicit policy admission of invalid time remains labelled and cannot change its recorded status.

**L3-F2CSDW-106** · Parent: L2-F2CSDW-106 · Verification: T

**L1 ancestor**: L1-F2CSDW-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Test word & 0xFFFFFF00 and emit TF-005 with the nonzero mask. Keep NTF and TS diagnostics separate.

**Source**: "Reserved. Bits 31-8 are reserved" (§11.2.3.3 a); §11.2.1 f; T-2. Replaces L1-F2CSDW-003 (bits 31–4).

**Planned verification**: Walking bits 8..31 and valid low-byte fields; NTF=1/2 must not trigger reserved-bit findings.

## NTP

**L3-NTP-101** · Parent: L2-NTP-101 · Verification: T

**L1 ancestor**: L1-NTP-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Decode two little-endian u32 values at offsets 0 and 4. Retain raw fraction for lossless wire round trips, even when a nanosecond view is rounded; do not apply network byte order merely because the payload is named NTP.

**Source**: "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (Figure 11-16). Prototype identifier kept.

**Planned verification**: Seconds=0x01020304 and fraction=0x80000000; short 0..7 bytes, trailing bytes untouched, independently supplied wire expectations.

**L3-NTP-102** · Parent: L2-NTP-102 · Verification: T

**L1 ancestor**: L1-NTP-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Resolve the era in this order: explicit caller era; else a unique nearest-era candidate within the RFC 5905 half-era window of caller-supplied approximate date or an independently dated absolute reference in the same session; else labelled era-0 policy assumption when no pivot evidence exists. Record era, evidence kind, pivot/reference identity, session and any assumption in the basis. Conflicting or exactly half-era ambiguous supplied pivots require explicit resolution, not a silently chosen era. Never use the host clock or treat a prior era-0-assumed NTP answer as independent dated evidence. Era 0 covers 1900-01-01 through just before 2036-02-07 06:28:16 UTC, not all earlier centuries.

**Source**: "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. "Eras cannot be produced by NTP directly" and "they can be derived from external means" (RFC 5905 §6, p.12); "if the client is set within 68 years of the server" (§6, p.15); "0 h 1 January 1900 UTC" (§6, p.13). Evidence precedence and labelled era-0 fallback are owner policy (2026-11-12), specified in RDG-012.

**Planned verification**: Explicit era wins; caller-date and same-session dated-reference pivots before/after 2036; exact half-era ambiguity and conflicting pivots; no evidence gives labelled era 0; no host-clock dependence; do not relabel an assumed NTP date as independent evidence.

**L3-NTP-103** · Parent: L2-NTP-103 · Verification: T

**L1 ancestor**: L1-NTP-003

**Obligation**: The implementation SHALL compute ns=floor((u64(fraction)*1000000000)/2^32), accepted by owner 2026-11-13. For the unsigned fraction floor is round toward zero. Preserve the original fraction and record rounding/quantization in the basis. This loses less than 1 ns, never carries into seconds and does not change era selection.

**Source**: "32-bit fraction field" (RFC 5905 §6, p.13); "Time Unsigned Seconds Fractions" (106-24R1 Figure 11-16, printed 11-27 / PDF 35). Floor (round toward zero for the unsigned fraction) is accepted owner policy, 2026-11-13, not standard wording (RDG-012).

**Planned verification**: Fraction 0 -> 0 ns, 0x80000000 -> 500000000 ns, 0xFFFFFFFF -> 999999999 ns under accepted floor; adjacent representability boundaries, raw-word preservation, no 1e9 carry; owner floor decision 2026-11-13.

**L3-NTP-104** · Parent: L2-NTP-104 · Verification: T

**L1 ancestor**: L1-NTP-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use checked signed wide arithmetic: extended_seconds=era*2^32+raw_seconds; unix_seconds=extended_seconds-2208988800; apply the separately documented fraction rounding. UTC follows this Unix-style mapping: never add cumulative TAI-UTC (37 s would be a spurious shift after 2017). Before conversion, seconds=0 AND fraction=0 means unknown/unsynchronized regardless of era evidence: keep raw data, emit a finding, return no absolute time; seconds=0 with nonzero fraction is not the sentinel. Format 2 has no NTP leap indicator and cannot uniquely express 23:59:60. For a covered positive-leap transition with following-midnight Unix value B, NTP values mapping to [B,B+1) are leap-ambiguous under RFC 1305 Figure 8: emit a finding, keep the ordinary mapped value and ambiguity basis, never assert a definite leap label. Outside table coverage keep the NTP UTC answer and record coverage/currency uncertainty; absence of a detected ambiguity is not proof there was none. TF-029 WARNING SHALL report seconds=0 AND fraction=0, retaining raw data with no absolute value. TF-030 WARNING SHALL report a covered leap-second ambiguity window, keeping the ordinary mapped value labelled leap-ambiguous; it is not TF-022.

**Source**: "2,208,988,800" (RFC 5905 §6 Figure 4, p.14, 1 Jan 1970); "a value of zero is a special case representing unknown or unsynchronized time" (§6, p.14); "knowledge of all previous leap seconds is lost" (RFC 1305 Appendix E.8, p.78); Figure 8 (p.79) repeats the same NTP number for UTC second 60 and midnight. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: RFC 5905 Figure 4: 1970, 1972, 1999-12-31 and 2036 era 1; a post-2016-leap date must not acquire +/-37 s; negative pre-1970 Unix time; exact all-zero sentinel vs zero seconds/nonzero fraction; [B,B+1) boundaries with a covered insertion; missing/stale table keeps value and records uncertainty. Check exact TF-029/030 IDs, WARNING levels, raw retention, sentinel no-value and ambiguity value-kept effects.

## PTP

**L3-PTP-102** · Parent: L2-PTP-102 · Verification: T

**L1 ancestor**: L1-PTP-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Store TaiSeconds and nanos; do not subtract an offset during byte parsing. Conversion to UTC is a separate operation using the supplied leap table.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds" (§11.2.3.3). Prototype identifier kept.

**Planned verification**: Raw epoch 0 remains TAI; values around a leap transition; NTP/PTP raw seconds with the same number must retain distinct epoch/scale metadata.

**L3-PTP-103** · Parent: L2-PTP-103 · Verification: T

**L1 ancestor**: L1-PTP-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Validate nanos<1000000000 before constructing a PTP value; return an error naming nanoseconds and its raw u32. Do not carry overflow silently into seconds.

**Source**: Figure 11-17 ("Time Unsigned Nanoseconds"). Prototype identifier kept.

**Planned verification**: 0, 999999999, 1000000000, and 0xFFFFFFFF; reject malformed values even if TS=valid.

**L3-PTP-104** · Parent: L2-PTP-104 · Verification: T

**L1 ancestor**: L1-PTP-004

**Obligation**: The implementation SHALL select the applicable offset on TAI-side transition intervals and subtract TAI-UTC with checked arithmetic, preserving covered leap-second state and raw TAI. Outside coverage use the last known offset, retain UTC labelled possibly out of date and raise TF-021 (warning), as contract 6.8/6.9 and L1-TAI-004 require (owner 2026-11-13). Record table identity, offset, coverage and known-current-through independently of the final transition. If there is no known offset at all, preserve TAI and report missing context rather than inventing an offset; this does not change the stale-table fallback.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012.

**Planned verification**: Covered transition before/during/after and historical offset; exact coverage endpoints and one tick beyond; stale table and long no-new-leap interval. Outside coverage with a known last offset: UTC = TAI minus that offset, UTC retained with possibly-out-of-date label and TF-021 warning, raw TAI retained. Check table/offset provenance; empty table cannot invent an offset.

**L3-PTP-105** · Parent: L2-PTP-105 · Verification: T

**L1 ancestor**: L1-PTP-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Read seconds=u32 LE at[0..4), ns=u32 LE at[4..8); never read a 48-bit seconds field or a ten-byte message. Encode those same fields in that order.

**Source**: "Time Unsigned Seconds", "Time Unsigned Nanoseconds" (Figure 11-17; the same in 106-17 and 106-19); T-3. Replaces L1-PTP-001 (10 bytes, 48-bit seconds).

**Planned verification**: Distinct 32-bit words, max seconds, short 0..7 bytes, exactly 8 bytes with two nonzero following filler bytes; T-3 regression.

## F2COR

**L3-F2COR-101** · Parent: L2-F2COR-101 · Verification: T

**L1 ancestor**: L1-F2COR-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Reference data must include session, channel, packet identity, raw RTC, NTF, TS, scale, precision, and decoded body. Apply the same admission and channel-selection stages used for Format 1; invalid status is not a usable reference by default.

**Source**: "The Format 2 Network Time packet data type is used to extract and encapsulate absolute time from NTP or IEEE-1588 PTP and time tag it with the RTC" (§11.2.3.3). Prototype identifier kept.

**Planned verification**: Mixed Format 1/2 timeline, valid/invalid TS, each NTF, and absent declarations; no use of packet arrival wall-clock time.

**L3-F2COR-102** · Parent: L2-F2COR-102 · Verification: T

**L1 ancestor**: L1-F2COR-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Before computing jumps, drift, or interpolated time, normalize operands to an explicitly selected known scale using the leap table. Retain each input scale and all offsets applied; unknown scale blocks cross-scale arithmetic.

**Source**: §11.2.3.3; contract 4.1 rule 4. Prototype identifier kept.

**Planned verification**: Same physical instant expressed as NTP UTC and PTP TAI, GPS versus UTC, unknown scale, and a stale-table conversion; equal raw seconds are not equal instants by default.

## TAI

**L3-TAI-101** · Parent: L2-TAI-101 · Verification: T

**L1 ancestor**: L1-TAI-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Table construction validates strictly increasing effective instants and rejects conflicting duplicate dates; store typed TAI-minus-UTC seconds, and lookups select the latest transition not after the requested instant. Dates before the supported table start return unresolved.

**Source**: L1-PTP-004. Prototype identifier kept.

**Planned verification**: Empty table, unordered entries, equal-date conflicting offsets, before-first and exact-boundary lookups.

**L3-TAI-102** · Parent: L2-TAI-102 · Verification: I

**L1 ancestor**: L1-TAI-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Release verification must compare every transition and coverage date with an authoritative IERS list and record the source/version. A recent release date alone must not imply an independently verified coverage date.

**Source**: Contract 6.8. Prototype identifier kept.

**Planned verification**: Planned inspection: table/source diff and release checklist, including a period with no new leap transition; no live network fetch inside the library.

**L3-TAI-103** · Parent: L2-TAI-103 · Verification: T

**L1 ancestor**: L1-TAI-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Validate the merged transitions before installing them; reject inconsistent historical offsets and overlapping conflicting entries with a typed error. Pass the table by value/borrowed data into conversion APIs, never fetch it implicitly.

**Source**: Contract 6.8. Prototype identifier kept.

**Planned verification**: Valid extension, replacement, conflicting boundary, unsorted extension, and proof that previously issued answers retain the table provenance they used.

**L3-TAI-104** · Parent: L2-TAI-104 · Verification: T

**L1 ancestor**: L1-TAI-004

**Obligation**: The implementation SHALL select the applicable offset on TAI-side transition intervals and subtract TAI-UTC with checked arithmetic, preserving covered leap-second state and raw TAI. Outside coverage use the last known offset, retain UTC labelled possibly out of date and raise TF-021 (warning), as contract 6.8/6.9 and L1-TAI-004 require (owner 2026-11-13). Record table identity, offset, coverage and known-current-through independently of the final transition. If there is no known offset at all, preserve TAI and report missing context rather than inventing an offset; this does not change the stale-table fallback. For NTP outside coverage keep directly mapped UTC and record that leap ambiguity could not be ruled out; never apply the PTP offset correction to NTP.

**Source**: "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Retained earlier source: "The leap-second table does not reach the date … the conversion is given with the last known offset and labelled as possibly out of date" (contract 6.8).

**Planned verification**: Covered transition before/during/after and historical offset; exact coverage endpoints and one tick beyond; stale table and long no-new-leap interval. Outside coverage with a known last offset: UTC = TAI minus that offset, UTC retained with possibly-out-of-date label and TF-021 warning, raw TAI retained. Check table/offset provenance; empty table cannot invent an offset. NTP direct UTC is retained with leap-ambiguity/currency uncertainty and no cumulative offset adjustment.

**L3-TAI-105** · Parent: L2-TAI-105 · Verification: T

**L1 ancestor**: L1-TAI-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: FMT 4 and NTP->UTC stated; Format 2 PTP->TAI stated; FMT 5->GPS with TAI-GPS=19 s inferred through RDG-009; A/B/G->UTC assumed by default; RTC-format->unknown. Carry a caller override as policy provenance and retain original format.

**Source**: "0x4 = Universal Coordinated Time (UTC) time from GPS", "0x5 = Native GPS Time" (§11.2.3.2 a); §11.2.3.3; "GPS time does not add or subtract leap seconds, and as of this writing, GPS time is 16 seconds ahead of UTC" (RCC 200-16 Appendix A.2, when TAI − UTC was 35 s); "All Department of Defense (DoD) test ranges … maintain Coordinated Universal Time (UTC)" (RCC 200-16 §1); T-17; RDG-009.

**Planned verification**: Every Format 1 FMT and Format 2 NTF, IRIG override/unknown, GPS-to-TAI+19 s and UTC via a dated table; native GPS must not be labelled UTC without conversion (T-17).

**L3-TAI-106** · Parent: L2-TAI-106 · Verification: T

**L1 ancestor**: L1-TAI-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Return unresolved scale when no override exists. With an override, mark the answer assumed-by-caller and keep the original unknown designation in its basis; do not infer scale from the host clock or from SRC=external.

**Source**: FMT "0x3 = Real-Time Clock" (§11.2.3.2 a) states no scale; contract 6.8; RDG-009.

**Planned verification**: FMT 3 with no override, UTC/TAI/GPS overrides, external/internal SRC independent of scale, and serde round trips of assumption provenance.

**L3-TAI-107** · Parent: L2-TAI-107 · Verification: T

**L1 ancestor**: L1-TAI-007

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Require explicit 23:59:60 with nanos<1e9 on DOY181/182/365/366 when year unknown, or June30/December31 for a known year; allow labelled assumed-UTC IRIG and reject TAI/GPS/unknown second 60. Preserve the explicit leap label and TF-022. NTP seconds/fractions lack a leap indicator: apply RDG-012's covered insertion ambiguity window, keep the mapped value with a distinct leap-ambiguous finding, and do not report that an unambiguous second 60 was read. Use TF-022 for an explicit leap-second label read; use distinct TF-030 WARNING for an NTP ambiguity window, keeping and labelling the mapped value.

**Source**: "time changes are made on December 31 and on June 30 at 2400 hours" (RCC 200-16 Appendix A.2, printed A-1 / PDF 56); "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (106-24R1 Figure 11-16, printed 11-27 / PDF 35); RFC 1305 Appendix E.8 Figure 8, p.79. RDG-010 remains accepted; RDG-012 refines the NTP case. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Retain Format 1 known/unknown-year UTC leap cases; NTP covered [B,B+1) ambiguity is distinct from explicit second 60, normal values outside, and unknown table coverage. Distinguish TF-022 explicit second 60 from TF-030 ambiguity without a leap indicator.

## WORD

**L3-WORD-101** · Parent: L2-WORD-101 · Verification: T

**L1 ancestor**: L1-WORD-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Keep T-10's DERIVED full 32-bit binary count. For BCD Figure 4-4 high word: day fragment bits15..13, tens-hour12..11, hour10..7, tens-minute6..4, minute3..0; low word: bit15=0, tens-second14..12, second11..8, tenths7..4, hundredths3..0. Microseconds are binary0..9999 in both. The binary exception to the shared-row zero remains DERIVED from 655.36 s=2^16*0.01 s, with the zero retained for BCD; validate digit/ranges. The 106-24R1 BCD field interpretation remains a proposal, not a resolution of the older characters wording; do not split a binary day field or silently resolve the older BCD day wording. Require an explicit caller binary origin; BCD requires caller year and full day plus declared interpretation context. Without it return a labelled partial time with raw words, weighting and missing context. Never assume day 1/year/origin or infer it from the nearest time packet; no such opt-in is added now. Preserve/report 106-04..15 "characters" versus 106-17+ "bits" as unresolved, even when caller context permits a labelled reconstruction.

**Source**: "resolution of 655.36 seconds", "10 milliseconds", "three least significant bits of the BCD Julian date", "at a rate necessary to resolve time ambiguities" (106-24R1 Chapter 4 §4.7, printed 4-11 / PDF 15); RCC 123-20 `enI106_Ch4Binary2IrigTime`, printed B-43 / PDF 149, applies word weights without an origin. Explicit caller context and partial results are owner policy (2026-11-12); historical characters/bits wording remains unresolved (T-10). Retained earlier source: "High and low order time words shall be binary or binary coded decimal (BCD) weighted, and microsecond words shall be binary weighted"; resolutions of 1 µs (to 9999), 10 ms, and 655.36 s binary or one minute BCD; "For BCD, the days field shall contain the three least significant bits of the BCD Julian date" (Chapter 4 §4.7); `C-d\PTM`, `C-d\BTM` (Chapter 9); contract 3.8. Layout confirmed 2026-11-12, T-10.

**Planned verification**: Binary count 0/0x8000/full count with absent/provided origin; BCD with absent year/full day and explicit context; day-fragment disagreement; older characters/current bits reported rather than silently harmonized; no nearest-packet setting or day-1 repair. Retained raw-layout fixtures: One fixture per binary and BCD weighting, each bit group distinguished, decimal digit>9, day fragment collisions, 9999/10000 microseconds, and absent origin/day context returning partial time. Independent top-bit fixture: `00 00 00 00 00 80 00 00` -> low=0x8000, high=usec=0 -> 327.68 s (327680000000 ns); accept under the DERIVED binary reading of T-10, and retain BCD bit 15=0. The low=0x1234 fixture alone cannot reject the opposing reading.

**Edition wording (page images checked, §4.7):** 106-04 through
106-15 say "three least significant characters of the BCD Julian date";
106-17+ say "three least significant bits of the BCD Julian date".
Checked prose: 106-04 printed 4-8 / full-PDF 63; 106-05/07/09/11 printed
4-8 / PDF 10; 106-13/15 printed 4-10 / PDF 14; 106-17 printed 4-11 /
PDF 15; 106-24R1 printed 4-11 / PDF 15. Do not silently normalize
characters to bits. Owner2026-11-12 settled the explicit-context/partial-result
boundary, not this wording ambiguity; reconstruction may use only explicit
caller context with the unresolved wording reported.

**L3-WORD-102** · Parent: L2-WORD-102 · Verification: T

**L1 ancestor**: L1-WORD-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Accept caller-extracted C-d\NTM kind and raw values. Full NTP uses RDG-012 era evidence/assumption, zero sentinel, UTC mapping and leap ambiguity; full PTP retains raw TAI and converts to UTC using the applicable TAI-UTC offset. After table coverage ends use the last known offset, label UTC possibly out of date and raise TF-021 (warning), retaining table identity, coverage, offset and currency. If no known applicable offset exists, retain TAI and report missing context rather than inventing an offset; this does not change the stale-table fallback. Component-only seconds/fraction/ns need their matching components and applicable explicit context; do not invent zero components. An NTP era is resolved by RDG-012, not always required as an explicit caller declaration.

**Source**: "PTP Time (64-bit)", "NTP Time (64-bit)" (106-24R1 Chapter 9 Table 9-11, `C-d\NTM`); "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Retained earlier source: `C-d\NTM`: "PTP Time (64-bit)", "PTP Seconds (32-bit)", "PTP Nanoseconds (32-bit)", "NTP Time (64-bit)", "NTP Seconds (32-bit)", "NTP Fractions (32-bit)" (Chapter 9, Table 9-11); contract 3.8. "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012.

**Planned verification**: All six Table 9-11 kinds; full NTP with declared/pivot/assumed era and zero sentinel; component words missing seconds/fraction/ns stay partial; full PTP outside table retains TAI and converts UTC with the last known offset, possibly out of date with TF-021.

## ENC

**L3-ENC-101** · Parent: L2-ENC-101 · Verification: T

**L1 ancestor**: L1-ENC-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Validate field representability, date, and leap rules before writing; use Figure 11-12 masks and Figure 11-13/14 body slots. Preserve payload-invalid NONE explicitly and never silently encode unsupported reserved enums as named formats.

**Source**: §11.2.3.2; §11.2.1 f; contract 2.2, 4.11.

**Planned verification**: Golden bytes from both figures and contract 7, NONE, reserved enum handling, nonrepresentable sub-10-ms time (explicit precision error, not silent truncation), and year/day bounds.

**L3-ENC-102** · Parent: L2-ENC-102 · Verification: T

**L1 ancestor**: L1-ENC-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Construct CSDW=(NTF<<4)+TS with reserved 31..8 zero; write NTP seconds/fraction or PTP seconds/nanos as8 bytes. Check PTP nanos and raw 32-bit seconds range; report unrepresentable requested time instead of truncating.

**Source**: §11.2.3.3; §11.2.1 f; contract 2.2, 4.11.

**Planned verification**: NTF 0/1/2 with TS 0/1 expected words, independent NTP/PTP bytes, high 32-bit seconds overflow, and fraction preservation.

**L3-ENC-103** · Parent: L2-ENC-103 · Verification: T

**L1 ancestor**: L1-ENC-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Secondary header writes 8 time bytes, two zero reserved bytes, byte-sum checksum in LE; stamps write 8 bytes without a header checksum, including zero RTC high bytes or Chapter 4 low reserved bytes. Reject source=1 without presence=1 and source=1 with reserved format 11 under L1-IPT-007. Source=0 stamps remain RTC even if observed format=11, with its separate TF-006 warning; this does not authorize encoding an undefined secondary-header time or nonzero reserved fields.

**Source**: "0 = Packet header 48-bit RTC."; "1 = Packet secondary header time (bit 7 must be 1)."; "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved" (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17); "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b, printed 11-11 / PDF 19). Value retention is owner 2026-11-12, Q-FLAGS; TF-006 reserved-value warning and unchanged TF-005/TF-006 levels are confirmed owner 2026-11-13; page images and older equivalent wording: `docs/analysis/Q-FLAGS-2026-11-12.md`. Secondary-header output layout: §11.2.1.2; stamp layout: §11.2.1.3 b. Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes.

**Release scope**: Only the new complete secondary-header/stamp encoder API is deferred past 0.8.0 (owner 2026-11-13, plan Q3). Repair of already exposed/used encoders and correct byte-sum behaviour remain 0.8.0 blockers; the requirement is not deleted or marked covered. See [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Golden header for all three defined formats, RTC-form stamp including observed flags 0x0C/0x8C with separate findings, invalid secondary-derived flags 0x4C/0xCC/0x40, byte-vs-word checksum distinction, and no writes past provided output capacity.

## EDN

**L3-EDN-101** · Parent: L2-EDN-101 · Verification: T

**L1 ancestor**: L1-EDN-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: For declared editions earlier than 106-17 emit TF-026 when data type 0x12 appears; from 106-17 accept it. Unknown edition remains unknown, not assumed pre-17 or latest. Record declared and observed format separately.

**Source**: 106-17 Chapter 11 lists "0x12 Time Data, Format 2 Network Time" with data type version `0x08` (106-17); 106-13 and 106-15 do not define it; T-4.

**Planned verification**: Declared 106-15/17/19/22/24R1 and unknown; Format 2 data type version 0x08; T-4 regression.

**L3-EDN-102** · Parent: L2-EDN-102 · Verification: T

**L1 ancestor**: L1-EDN-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use Irig106Version::from_setup_record for setup RCCVER: 0x0C names V17, 0x0D V19 (applicable rules include 106-20 per owner), 0x0E V22 with note unchanged through 106-24R1; 0x0F..0xFF are Unknown(raw). Use from_data_type_version for the separate header list, where 0x0A names V22 and 0x01 V04. These decode code-list names, not exact recording editions; L3-EDN-201 resolves applicability. Never exchange the two domains or infer legacy EXT from V04 returned for a header byte alone.

**Source**: Table 11-4 and §11.2.1.1 e ("0x0A = RCC 106-22"); Figure 11-34 ("0x0E = RCC 106-22", "0x0F through 0xFF = Reserved"); T-9; ADR-0015. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Planned verification**: All assigned/reserved shared codes in both lists, especially 0x0A/0x0D/0x0E/0x0F, preserving raw unknown codes. T-9 regression.

**L3-EDN-103** · Parent: L2-EDN-103 · Verification: T

**L1 ancestor**: L1-EDN-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Maintain an edition rule table with source chapter/section and available fixture per change: Chapter 10 before 106-17, Chapter 11 thereafter, ITS applicability, Format 2 availability, and late-packet limits. Use the archived 106-04 full PDF §10.6.3.3/Figure 10-15 for the EXT layout, and 106-05 Figure 10-16a for SRC/None. Select the rule only after L3-EDN-201 resolves layout evidence; header 0x01 is not proof of a 106-04 recording.

**Source**: "References to RCC 106-04 through RCC 106-15 refer to Chapter 10, while RCC 106-17 onward refer to Chapter 11" (§11.2.1.1 e); ADR-0004. Edition qualification: 106-04 §10.6.3.3, Figure 10-15, printed 10-30–31 / PDF 288–289; 106-05 Figure 10-16a, printed 10-37 / PDF 41; 106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14; STANDARD-REVIEW T-6.

**Planned verification**: An independent fixture per available edition touched by a reading, unknown edition/code, pre-07 declared edition absent, and source-backed exceptions. Include archived 106-04 EXT/SRC transition fixtures and the version-evidence conflict cases of L3-EDN-201.

**L3-EDN-201** · Parent: L2-EDN-201 · Verification: T

**L1 ancestor**: L1-EDN-002

**Obligation**: The implementation SHALL give an explicit caller declaration/override precedence when selecting packet layout (basis override), retaining and showing contradictory opted-in G\106 and header data-type-version evidence side by side in the basis. Without an override, use caller-opted-in G\106 as packet edition (basis declared, assumed from TMATS) only when not contradicted by packet evidence; otherwise leave layout unresolved, preserve raw bits and invent no SRC/None labels, retaining the evidence and reporting a conflict only when packet evidence contradicts the opted-in declaration. Differing declarations alone do not automatically produce a conflict finding; a lower header version allowed by at-or-below wording or an unadopted different TMATS-file edition is not proof of conflict. Keep setup and data-type version evidence distinct and applicable only where defined. Basis vocabulary and override precedence follow TMATS ADR-0028; no fallback assumption selects EXT. Extract from_setup_record only from a field known to exist; 106-04 generic and 106-05 Format 1 CSDWs supply no version evidence, whereas 106-07 CH10VER first assigns 0x07. from_data_type_version decodes a header hint, not recording layout; V04 for 0x01 cannot establish EXT. Resolve layout before modern SRC/FMT constructors; raw nibble 3 must never become None by guessing. Report the existing no-override contradictory packet-layout evidence case as TF-031 "Packet-layout edition evidence is contradictory"; level pending owner (S03). Explicit override still wins with contradictory evidence retained side by side, without an automatic conflict finding; differing declarations or lower permitted header versions alone are not conflicts.

**Source**: "EXT" (106-04 §10.6.3.3 Figure 10-15, printed 10-30 / full PDF 288; EXT definitions printed 10-31 / full PDF 289); "SRC" (106-05 Figure 10-16a, printed 10-37 / PDF 41). "a value at or below the release version of the standard applied to the data types" (106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14). No setup version: 106-04 §10.6.7.2.2/Figure 10-34, printed 10-48 / full PDF 306; "Reserved. (Bits 31-0) are reserved." (106-05 Figure 10-34b, printed 10-55 / PDF 59); "CH10VER" (106-07 Figure 10-50, printed 10-72 / PDF 76, only 0x07 assigned). G\106 is "VERSION OF IRIG 106 STANDARD USED TO GENERATE THIS TMATS FILE" (106-04 Table 9-1, printed 9-8 / full PDF 186). Basis vocabulary: "override", "declared", "fallback" (TelemetryWorks/irig106-tmats `docs/adr/0028-edition-declarations-and-validation-basis.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`). Owner 2026-11-13, Q-EDITION: explicit caller override wins with contradictory evidence retained side by side; without override use opt-in TMATS only when uncontradicted, else unresolved. The earlier conflicting-evidence brief applied to the no-override case, as clarified in review. ADR-0028: "No automatic conflict finding"; "differing declarations are shown side by side" (same pinned commit, lines 59–63; caller-override precedence context line 11). STANDARD-REVIEW T-6; 106-24R1 Table 9-2 printed 9-10 / PDF 16 (index 9-9 / PDF 15); analysis: `docs/analysis/Q-EDITION-2026-11-13.md`. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Header 0x01 alone -> unresolved/raw bits, no SRC/None labels; 106-04/05 reserved setup CSDW never becomes a version field. Explicit 106-04 -> EXT/basis override; explicit 106-05 -> SRC/basis override. Explicit override wins even with contradictory opted-in G\106 or header data-type-version evidence: retain all evidence side by side in the basis. Without override, G\106=04 without opt-in -> metadata only/unresolved; with opt-in and no contradictory packet evidence -> EXT/basis declared, assumed from TMATS; opted-in 05 -> SRC with the same assumption label. Without override, opted-in G\106 contradicted by packet evidence -> unresolved plus finding with evidence shown. Missing/malformed usable declarations -> unresolved. Differing declarations alone do not automatically create a conflict; a header at or below the adopted release is only a hint. Later CH10VER 0x07 tests remain distinct; no fictional 106-04/05 setup code or fallback EXT. All are planned fixtures. Verify TF-031 only in the existing no-override contradiction case; retain override precedence. Do not choose a severity before S03 owner confirmation.

## DECL

**L3-DECL-101** · Parent: L2-DECL-101 · Verification: T

**L1 ancestor**: L1-DECL-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Store session/range start, channel IDs, optional TTF/TFMT/TSRC/SHTF, declared version and optional RI4 without parsing TMATS text. Changes update governing declarations without resetting the counter; preserve missing/unknown values, original spelling, and caller provenance.

**Source**: Contract 2.3 C, 4.5, 5.3; `irig106-tmats` L1-CH10-008 (the configuration timeline); ADR-0005.

**Planned verification**: Two configuration ranges in one session, exact change boundary, absent fields, multiple time channels, and original recording date distinct from TMATS origination date.

**L3-DECL-102** · Parent: L2-DECL-102 · Verification: T

**L1 ancestor**: L1-DECL-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Represent declarations as absent, not an empty-but-present setup record. With default declared-only admission no reference is eligible; an explicit policy may admit observed time channels labelled undeclared. Every answer/finding includes declaration-presence provenance.

**Source**: A recording's setup record can be missing or unreadable; contract 6.4.

**Planned verification**: No setup record with default policy gives counter-only; explicit undeclared-admission yields labelled absolute answers; present-empty declarations remain distinguishable.

## COR

**L3-COR-101** · Parent: L2-COR-101 · Verification: T

**L1 ancestor**: L1-COR-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: A builder consumes caller packet identity, channel, RTC and flags in file order, and bounded CSDW/body bytes for time packets. Retain both eligible and rejected references with their admission reason. Finalization returns immutable session/channel indexes and findings; library code never walks a file.

**Source**: "Time is treated like another data channel" (§11.2.3.2); "Since Time Data, Format 1 packets contain both the absolute input time value and the RTC clock value at the instant the absolute time was valid, these packets can be used to relate RTC values to the input absolute time source" (RCC 123-20 §5.6); contract 5.5. Prototype identifier kept; citation corrected (T-13).

**Planned verification**: Contract 7 recording, malformed time packet between valid references, empty recording, setup changes, multiple channels, and repeated finalization/query semantics documented.

**L3-COR-102** · Parent: L2-COR-102 · Verification: T

**L1 ancestor**: L1-COR-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Index eligible references by unwrapped counter within session/channel; binary-search neighbours, choose minimum absolute counter distance, and choose the preceding reference on an equal-distance tie (owner-accepted deterministic tie default, 2026-11-13, C2). Do not extrapolate before the first reference unless policy enables it. The basis SHALL retain the tie selection rule.

**Source**: "It is better to use the clock and relative time values from a time packet that occurs near the current data packet as the data file is decoded since there is some drift in the RTC during a recording session" (RCC 123-09 §6.6; T-13); contract 5.2 rule 5; a policy setting (L1-POL-001). Prototype identifier kept. Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above.

**Planned verification**: Exact reference, midpoint tie, before-first default/override, beyond-last, wrap, and no reference from another session or channel. Compare streaming only with the same reference strategy, not offline nearest by accident. Check the accepted preceding midpoint tie default and any explicit override.

**L3-COR-103** · Parent: L2-COR-103 · Verification: T

**L1 ancestor**: L1-COR-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Offline: caller choice, declared-external preference, most valid references, then lowest ID. Stream, before data: caller-fixed channel; else TMATS-declared external TIMEIN (R-x\CDT-n=TIMEIN and R-x\TSRC-n=E); else lowest eligible channel ID, using lowest ID to break equal preference ties. This requires a complete known candidate set/declarations; if selection cannot be decided in advance, return counter-only until the caller chooses. Commit the selected channel for the stream session; never use future valid-reference counts, revise emitted answers or switch on lost lock. An unavailable caller-selected channel yields no absolute time with evidence, never a silent substitute.

**Source**: "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. Retained earlier source: "When multiple time channels are available, it is incumbent on the programmer or data analyst to determine and select the best source of time for a particular data set" (RCC 123-20 §5.6); "It is usually most correct to select one time channel only and to use this channel exclusively" (RCC 123-09 §6.6); contract 5.2 rule 4; a policy setting. Prototype identifier kept.

**Planned verification**: Offline count ranking unchanged; fixed caller choice; one/multiple declared external TIMEIN channels with ID ties; complete inventory lowest-ID fallback; incomplete/unknown inventory counter-only until caller choice; future counts/lost lock never alter prior answers.

**L3-COR-104** · Parent: L2-COR-104 · Verification: T

**L1 ancestor**: L1-COR-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compute residual=(absolute_next-absolute_prev)-(rtc_delta*100 ns) after scale normalization; emit TF-017 only when abs(residual)>threshold. Default threshold is format resolution; heterogeneous-resolution handling uses the coarser resolution as the owner-accepted default (2026-11-13, C3). Preserve residual/sign and packet identities.

**Source**: "there is a jump in input clock time during a recording, such as when GPS locks for the first time, or when an IRIG time source is reprogrammed" (RCC 123-09 §6.6; T-13); contract 5.4; default threshold the format's resolution (L1-POL-001). Prototype identifier kept. Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above.

**Planned verification**: Threshold-1, exact threshold and threshold+1; positive/negative jumps, lock regain, no false jump at RTC wrap, different scales requiring conversion, and unknown scale unresolved. Check coarser mixed-resolution threshold by default and explicit threshold override at/below/above the boundary.

**L3-COR-105** · Parent: L2-COR-105 · Verification: T

**L1 ancestor**: L1-COR-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use unwrapped sequence context and a high-water counter. A short forward modular wrap stays in session; a backwards displacement beyond the late bound starts a new session and records TF-018. Retain a distinct session ID even when raw RTC ranges overlap; queries require session identity.

**Source**: The counter "shall remain free-running during each session (e.g., recording)" (§11.2.1.1 i); contract 5.2 rule 1.

**Planned verification**: Forward wrap near 2^48, small late packet, backwards reset beyond 1100 ms, repeated raw ranges in distinct sessions, and an ambiguous large displacement requiring caller context.

**L3-COR-106** · Parent: L2-COR-106 · Verification: T

**L1 ancestor**: L1-COR-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Default eligible iff channel declared TIMEIN and payload valid (FMT not NONE, TS valid, readable supported format). Record undeclared, invalid, malformed, and reserved rejections separately. Explicit policy inclusion of readable invalid/undeclared time must mark every resulting answer; malformed input cannot become a reference.

**Source**: Contract 5.2 rules 2 and 3; "0xF = NONE (time packet payload invalid)" (§11.2.3.2 a); "0x0 = Time Not Valid" (§11.2.3.3 a); policy settings.

**Planned verification**: All combinations of declaration presence, TIMEIN, Format 1 NONE, Format 2 TS, and policy overrides; raw invalid values remain visible without default admission.

**L3-COR-107** · Parent: L2-COR-107 · Verification: T

**L1 ancestor**: L1-COR-007

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Track SRC transitions with previous/current source and TF-011. A reported external-to-internal change does not change the stream's committed channel, reinterpret static TSRC declaration, or rewrite prior answers. Default reference retention remains labelled; independently detected jumps still prevent interpolation across discontinuities. A configured stop-on-loss option may withhold absolute answers but does not silently select another channel.

**Source**: "the time source shall indicate Internal (0x0)" (106-24R1 §11.2.3.2 note, printed 11-26 / PDF 34). "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. Retained earlier source: "If the time source is external (0x1) and lock on the external source is lost then the time source shall indicate Internal (0x0). Once lock on the external time source is regained, time source shall once again indicate external (0x1)" (§11.2.3.2 a); contract 5.2 rule 6.

**Planned verification**: SRC external->internal->external on committed channel; a competing channel gains more references; chosen channel unchanged and transition labelled; stop-on-loss withholds rather than switches.

**L3-COR-108** · Parent: L2-COR-108 · Verification: T

**L1 ancestor**: L1-COR-008

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compare same-channel valid reference spacing against 1000000000 ns+gap_tolerance; accepted default gap_tolerance=0 ns. Spacing >1 s is a finding even if within the 1.1 s hold allowance. Keep hold and gap tolerances separate policy fields. Preserve Chapter 11's raw-network Format 2 rate exception in the basis: a policy gap is not falsely called a 1 Hz standard violation.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "unless it is recorded at the raw network rate of the NTP or PTP frames" (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Retained earlier source: "the time packet will be generated at a minimum frequency of 1 hertz" (§11.2.3.2; Chapter 10 §10.6.2); the Format 2 exception "unless it is recorded at the raw network rate" (§11.2.3.3); contract 5.4.

**Planned verification**: Spacing at1 s,1 s+1 ns,1.05 s and1.1 s; zero/default and explicit nonzero gap tolerances independent of hold; raw-network exception labelled.

**L3-COR-109** · Parent: L2-COR-109 · Verification: T

**L1 ancestor**: L1-COR-009

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compare each unwrapped counter to the high-water value; a backward offset<=11000000 ticks is within the default 1100 ms bound. Record later-than-bound evidence alongside any inferred new session; do not use a late packet to lower the high-water mark. Caller boundaries take priority.

**Source**: "all other packet generation times shall be equal to or less than 100 milliseconds (ms)"; "all other packets shall have a stream commit time equal to or less than 1000 ms" (Chapter 10 §10.6.1 b–c); contract 5.4.

**Planned verification**: Offsets 1099.9999/1100/1100.0001 ms, consecutive late packets, caller session boundary and bound override, pre-05 unknown ordering evidence, and forward wrap.

**L3-COR-110** · Parent: L2-COR-110 · Verification: T

**L1 ancestor**: L1-COR-010

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Choose a same-session DMY packet first, caller year second, RI4 third by default; never G\OD. Keep unknown if absent. Detect contradictory candidates and preserve them for findings/owner-reviewed policy rather than overwriting silently. Increment known year after December 31 and revalidate LY/day 366. Report contradictory year candidates as TF-032 "Year evidence is contradictory"; level pending owner (S08). Preserve candidates and the existing year-evidence policy; do not select a severity before owner confirmation.

**Source**: Figures 11-13 and 11-14; "Date and time original recording was created" (`R-x\RI4`, Chapter 9); contract 5.3. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Each precedence branch, all sources absent, contradictory years, leap/common year, setup changes, and December 31->January 1 with known and unknown year. Retain conflicting year evidence and TF-032 identity; severity proposal/owner confirmation belongs to S08.

**L3-COR-111** · Parent: L2-COR-111 · Verification: T

**L1 ancestor**: L1-COR-011

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Query within a specified session and scale; partition inverse lookup at reference jumps, gaps and year/scale changes. Return a list of counter intervals plus basis, not a single interval that bridges disjoint clock segments; unresolvable endpoints stay unresolved. Absolute range queries SHALL use half-open [start, end) and return every matching half-open RTC tick interval with explicit quantization and basis. Provide a separate point query; never bridge gaps/jumps, and retain unresolved endpoints.

**Source**: Contract 3.6, 4.9, 5.5. Owner 2026-11-13: "accept your recommendations" for the 0.8.0 plan; [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). This is reader/release policy, not an added IRIG mandate.

**Release scope**: Deferred past 0.8.0 (owner 2026-11-13, plan Q3). The requirement and decided half-open semantics remain; no inverse-query coverage is claimed. Offline drift-corrected query mode is also after 0.8.0. See [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Monotonic span, backwards/forwards clock jump with multiple/no matches, multiple sessions, half-open start-included/end-excluded boundaries and separate point queries, unknown year/scale, and a request outside coverage. Check start included/end excluded, tick quantization, separate point queries, multiple matches and no gap/jump bridging; feature remains deferred past 0.8.0.

**L3-COR-112** · Parent: L2-COR-112 · Verification: T

**L1 ancestor**: L1-COR-012

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Track first time-packet identity independently of first eligible reference. Static Computer-Generated Format 1 may precede it; other dynamic packets emit TF-002. At finalization a session without any time packet emits TF-001; None-source presence does not fabricate time.

**Source**: "A time data packet shall be the first dynamic data packet at the start of each session. Only static Computer-Generated Data, Format 1 packets may precede the first time data packet" (§11.2.3.2; "of each recording", Chapter 10 §10.6.2); "If the time data packet source is None, at least one time data packet is required" (§11.2.3.2); contract 6.1, 6.2.

**Planned verification**: Static TMATS then time then data, data first, no time, time-source None first, malformed time first, and second session after RTC reset (RDG-011).

**L3-COR-113** · Parent: L2-COR-113 · Verification: T

**L1 ancestor**: L1-COR-013

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Return valid/rejected counts, min/max spacing, references-per-second with its measurement interval, and drift_ppm=1000000*((absolute_delta/rtc_duration)-1) when duration nonzero and scales compatible. For 0/1 point, duplicate RTC or unknown scale, return unavailable measures with reason.

**Source**: Contract 3.5; the prototype's `compute_quality`.

**Planned verification**: 0/1/2 references, irregular spacing, duplicates, exact 10 MHz=0ppm, known positive/negative drift, invalid reference exclusion, and separate channels.

## ANS

**L3-ANS-101** · Parent: L2-ANS-101 · Verification: T

**L1 ancestor**: L1-ANS-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Basis retains session, channel/selection reason, reference IDs with counter/time, signed distances, position, SRC/FMT/ITS, scale and stated/assumed origin, precision, year source, leap/drift corrections, declaration presence and policy/table identity. Interpolation retains both endpoints and its rounding; no claim of1 ns accuracy from10 ms references. Add committed stream channel and selection rule, hold/gap bounds, cap/timeout/EOF extrapolation reason, and caller progress. For NTP record raw seconds/fraction, era, declared/pivot/era0-assumed provenance, pivot/reference identity, rounding policy, zero/unsynchronized or leap-ambiguous finding and table coverage/currency. Do not call an assumed era standard-stated or normalize native GPS to UTC implicitly.

**Source**: "Eras cannot be produced by NTP directly" and "they can be derived from external means" (RFC 5905 §6, p.12); "if the client is set within 68 years of the server" (§6, p.15); "0 h 1 January 1900 UTC" (§6, p.13). Evidence precedence and labelled era-0 fallback are owner policy (2026-11-12), specified in RDG-012. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. Retained earlier source: Contract 3.1 (with the precision of a reference: "consistent with the resolution with the time packet body format (10 milliseconds [ms] as measured by the 10-MHz RTC)", §11.2.3.2), 3.2, 3.5, 7.3; ADR-0011.

**Planned verification**: Reproduce answers from basis only; declared/pivot/default era distinguishable; lost-lock channel stable; cap/timeout/EOF fallback labelled; ambiguity and stale/missing table survive serialization.

**L3-ANS-102** · Parent: L2-ANS-102 · Verification: T

**L1 ancestor**: L1-ANS-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Decode checksum-valid secondary time, correlate relative ERTC as needed, and compute signed own-minus-counter discrepancy only after scale normalization. Emit TF-014 above the policy tolerance (default reference resolution); unknown comparison context stays unresolved, not a zero discrepancy.

**Source**: "The applicable data bit to which the 48-bit value of the packet secondary time value (if enabled) applies shall correspond to the first bit of the data in the packet body" (§11.2.1.2 a); contract 6.6.

**Planned verification**: Equal times, at/above tolerance, invalid checksum, ERTC without reference, partial Chapter 4 context, and incompatible/unknown scales.

**L3-ANS-103** · Parent: L2-ANS-103 · Verification: T

**L1 ancestor**: L1-ANS-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Padded RTC uses the session/channel timeline; relative ERTC uses its corrected 1 ns scale and matching session; absolute secondary formats retain own-time provenance. Invalid flags return no stamp time with findings and packet evidence.

**Source**: §11.2.1.3 b; contract 3.4, 4.6.

**Planned verification**: Each supported stamp form, no reference, inconsistent flags, reserved format, RTC wrap, and secondary format different from a previous packet.

**L3-ANS-104** · Parent: L2-ANS-104 · Verification: T

**L1 ancestor**: L1-ANS-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Attach the extraction origin supplied by caller to the decoded words. Use calendar/scale reconstruction only when complete context exists; return partial/counter-only outcome and reason otherwise. Never reuse unrelated packet counters as an undocumented day/era guess.

**Source**: Contract 3.8, 4.6.

**Planned verification**: Binary/BCD/network kinds, full and component-only words, missing calendar context, explicit matching context, and a decoder-supplied packet/channel identity.

**L3-ANS-105** · Parent: L2-ANS-105 · Verification: T

**L1 ancestor**: L1-ANS-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use unwrapped RTC(first_packet)->RTC(query) and typed duration. Mark counter-only, session, unknown absolute scale/year, reference absence reason and findings. Data before first time must remain counter-only in both owner-selected stream modes; explicit offline back-extrapolation is separately labelled.

**Source**: Contract 6.1, 6.2.

**Planned verification**: No time packets, all invalid, undeclared default rejection, data before first reference, and first packet near RTC wrap.

## POL

**L3-POL-101** · Parent: L2-POL-101 · Verification: T

**L1 ancestor**: L1-POL-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Policy includes session boundaries, declaration/validity admission, channel ranking, offline reference strategy, source-loss handling, year ranking, gap/tolerance, jump threshold, late bound, pre-first handling, own-time tolerance, scale assumptions, finding severities, stream mode and buffer limits. Counter wrap is arithmetic, not mutable policy. Policy records offline-nearest separately from default stream Standard, selectable Interpolate, Option A committed channel, hold interval1 s+tolerance100 ms, gap tolerance 0 ns, progress/deadline basis and caller caps. Keep existing boundary/validity/source/year/scale/finding settings. Accepted owner defaults are not pending proposals; interpolation rounding/ties, coarser jump threshold and mode-change flushing are accepted owner 2026-11-13, C1–C4.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. Owner 2026-11-13 accepts plan C1–C4; [decision/source record](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Policy snapshots distinguish offline and stream, default/override values, separate gap/hold fields, explicit caps and progress, no host-clock reads or hidden unbounded queues.

**L3-POL-102** · Parent: L2-POL-102 · Verification: T

**L1 ancestor**: L1-POL-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Validate nonnegative bounds and supported enum combinations; retain immutable policy snapshots by ID. A stream-mode change drains pending packets using the old policy/deadline/cap/fallback basis before installing the new snapshot (owner-accepted transition, 2026-11-13, C4); never re-label an old answer with a new policy.

**Source**: "I would like for it to be selectable as well" (owner, 2026-09-27); ADR-0010. Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above.

**Planned verification**: Change each setting from default; invalid bounds, standard->interpolate->standard with queued packets, and no mutation of already issued answers. Assert accepted old-policy drain and immutable old/new snapshot identities.

**L3-POL-103** · Parent: L2-POL-103 · Verification: T

**L1 ancestor**: L1-POL-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Store a versioned complete snapshot or stable ID plus an included snapshot table. Serialization of a delta must identify the default-policy version; record stream mode, gap/hold tolerance, scale assumptions and caller overrides even if no finding occurred. Snapshot/report JSON SHALL use a reviewed versioned schema with units, decimal strings for 64/128-bit quantities, validation on load and retained raw/stated/assumed provenance. Review the schema once before freeze.

**Source**: ADR-0010. Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation.

**Planned verification**: Same bytes+same policy deterministic result; different policies distinguishable; serialized delta cannot silently adopt changed future defaults. Check 64/128-bit extrema as decimal strings, units/schema version, validation and full provenance. Complex timeline rehydration alone has the API-110 release deferral.

**L3-POL-201** · Parent: L2-POL-201 · Verification: T

**L1 ancestor**: L1-POL-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use explicit mode enum {Standard,Interpolate}; omitted stream mode selects Standard (owner 2026-11-12), not a proposed default. Its basis cites RCC 123-20 §5.6's subsequent-packet example and ADR-0010; callers explicitly select Interpolate for drift correction. The later owner 2026-11-13 C1 decision accepts interpolation floor; offline nearest remains unchanged.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. Owner 2026-11-13 accepts plan C1–C4; [decision/source record](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Omitted mode selects Standard; explicit Standard/Interpolate; RCC example1000000->1150000 yields15 ms; offline-nearest remains distinct and every answer retains selected mode.

## FND

**L3-FND-101** · Parent: L2-FND-101 · Verification: T

**L1 ancestor**: L1-FND-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Finding holds stable ID, effective/default severity, packet identity/offset if available, channel, raw RTC and relevant observed/expected values. Preserve IDs across releases; absent caller offsets remain absent. Never turn malformed bytes into valid time merely by lowering severity. Permanent IDs extend through TF-032. TF-001..TF-028 remain unchanged; TF-029/030 are WARNING with sentinel-no-value/ambiguity-value-kept effects. TF-031 "Packet-layout edition evidence is contradictory" has level pending owner (S03); TF-032 "Year evidence is contradictory" has level pending owner (S08). No default level is assigned to the latter two by this decision.

**Source**: Contract section 6; ADR-0013. "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved"; "1 = Packet secondary header time (bit 7 must be 1)." (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17). The standard defines the flags, not finding IDs. Owner 2026-11-13, Q-FLAGS-ERROR accepts TF-028 "Secondary-derived time has an undefined format" at ERROR; architecture section 8. TF-001..TF-027 remain unchanged. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Each ID and severity override, absent offset, serialization, duplicate findings with distinct evidence, and an immutable identifier list matching architecture 8. Verify all permanent identities; freeze existing TF-001..028 rows; distinguish approved WARNING levels 029/030 from pending levels 031/032.

**L3-FND-102** · Parent: L2-FND-102 · Verification: T

**L1 ancestor**: L1-FND-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Maintain a dispatch table mapping no-time TF-001, early-data 002, invalid 003, malformed 004, reserved 005/006, LY 007, undeclared 008, unused-declared 009, mismatch 010/011, checksum 012/027, mixed-format 013, own-time 014, flags 015, gap 016, jump 017, reset 018, late 019, year 020, stale-table 021, leap 022, scale 023, sync 024, ERTC 025, old-edition Format 2 026, undefined secondary-derived time 028 (error). Weak time alone stays basis information. Include TF-029 (zero NTP, WARNING, no absolute value/raw kept), TF-030 (NTP ambiguity, WARNING, mapped value kept/labelled), TF-031 (contradictory packet-layout evidence, level pending owner S03) and TF-032 (contradictory year evidence, level pending owner S08). No pending level may be silently defaulted.

**Source**: Contract 6.1 to 6.9. "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved"; "1 = Packet secondary header time (bit 7 must be 1)." (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17). The standard defines the flags, not finding IDs. Owner 2026-11-13, Q-FLAGS-ERROR accepts TF-028 "Secondary-derived time has an undefined format" at ERROR; architecture section 8. TF-001..TF-027 remain unchanged. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: At least one synthesized case per TF identifier, including simultaneous causes; exact error/warning/information defaults from architecture 8 without renaming or changing owner choices. Check the new value effects and approved levels; severity proposals for 031/032 remain for their implementing slices.

**L3-FND-103** · Parent: L2-FND-103 · Verification: T

**L1 ancestor**: L1-FND-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Test flags&0x20 and emit TF-024 with packet/channel/counter; preserve it in the timeline even if no timestamp can be decoded. Its absence must not certify recorder accuracy.

**Source**: "Bit 5: RTC sync error. 0 = No RTC sync error. 1 = RTC sync error has occurred" (§11.2.1.1 g); found while writing the architecture (2026-09-27), not yet in the contract.

**Planned verification**: Bit 5 alone, alongside each valid flag format, and alongside invalid bit 6/7 combinations; all other flag bits varied.

**L3-FND-104** · Parent: L2-FND-104 · Verification: T

**L1 ancestor**: L1-FND-004

**Obligation**: The implementation SHALL compare declarations and packets only using RDG-001..006 mappings, keeping inferred I ↔ 0x3 labelled inferred, and emit TF-010 for TTF/TFMT/SHTF differences and TF-011 for TSRC difference/change, retaining raw declaration, observed packet field, governing setup range and RDG ID. Absent required TFMT always produces its missing-attribute finding. Default policy skips comparison; caller opt-in compares effective A and records default-derived basis while retaining that finding and raw absence. A default-derived mismatch can coexist with the missing finding. Never insert A into TMATS. Record published default A separately from raw absence, even when comparison is disabled.

**Source**: Contract 3.7, 6.4; the source may change legitimately (§11.2.3.2 a). "R/R Ch 10 Status: R"; "Default: A" (106-24R1 Chapter 9 Table 9-4 TFMT row, printed 9-72 / PDF 78). "A value of “R” requires that the attribute be specified in the TMATS file whenever the attribute is allowed." (§9.5.1 a, printed 9-5 / PDF 11). "If the R/R Ch10 Status field is “R”, then the attribute must be included in the TMATS file if all other conditions apply even if it has a default."; "Default: This identifies the default value required to process a TMATS file when the file itself does not contain the attribute." (printed 9-6 / PDF 12). RDG-002; owner 2026-11-13, Q-TFMT-DEFAULT = opt-in labelled default. TelemetryWorks/irig106-tmats `docs/adr/0023-validation-passes-and-effective-values.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: effective resolver "never writes to the document". Opt-in comparison and provenance are owner policy; no defaults are inserted.

**Planned verification**: Each declared/observed mapping and mismatch, missing fields, configuration boundary, I inference warning (I ↔ 0x3 labelled inferred, owner Q1), and external->internal loss of lock information (TF-011). Every accepted letter/FMT pair in both directions, all mismatches and unknown/non-TIMEIN cases. Explicit A compares as explicit. Absent TFMT without opt-in raises missing-required finding and performs no comparison. Absent TFMT with opt-in compares default A to FMT 0x1/mismatch, labels default-derived and still raises missing-required finding. Verify raw declaration remains absent; no default insertion. These comparison tests belong to the later declaration/report implementation, not this targeted flag-code change. Source changes remain informational and setup-range boundaries remain traceable.

**L3-FND-105** · Parent: L2-FND-105 · Verification: T

**L1 ancestor**: L1-FND-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compute expected_rtc=(ertc/100) modulo 2^48 and signed modular difference to supplied RTC; emit TF-025 when abs(delta)>1 tick. Include both raw counters and delta; no multiplication of ERTC by100.

**Source**: "the 10-megahertz (MHz) RTC shall be synchronized with the ERTC (RTC = ERTC/100)" (§11.2.1.1 g).

**Planned verification**: ERTC residual 0/99ns, expected RTC wrap, delta-2/-1/0/1/2 ticks, and a packet with no ERTC where no comparison is made.

## RDG

**L3-RDG-101** · Parent: L2-RDG-101 · Verification: I

**L1 ancestor**: L1-RDG-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: A typed mapping result contains matched/differed/missing/unknown, raw operands and RDG ID. Callers consume it rather than repeat letter-to-nibble logic; implementation review must find no comparison without a register reference.

**Source**: Contract 3.7; T-14; ADR-0012.

**Planned verification**: Planned inspection: audit all comparison sites and their RDG links, including numeric Format 2 and SHTF readings; unknown letters/numbers must not match by fallback.

**L3-RDG-102** · Parent: L2-RDG-102 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: For every used RDG entry require a cited analysis, a test for each listed edge, and fixture per touched archived edition; mark unavailable evidence explicitly. Preserve every Review field and the owner-set Design state. Trace-matrix output is a checklist, not proof of semantic correctness or two-person review.

**Source**: ADR-0012; `docs/INTERPRETATIONS.md`, "Development" ("intensive testing and deep analysis required", owner, 2026-09-26).

**Planned verification**: Planned inspection: register-to-analysis-to-tests audit, including RDG-008 suspect dual checksum and unresolved human reviews. Do not mark page checks as the complete analysis. Current inventory and remaining gaps: [S00 audit](analysis/S00-decision-source-evidence-audit.md). This audit is not satisfaction evidence and does not promote this obligation.

**L3-RDG-201** · Parent: L2-RDG-201 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compare declared TIMEIN TTF with observed packet data type by RDG-001, retaining missing/unknown values and packet authority.

**Source**: 106-24R1 Chapter 9 Table 9-4 and Chapter 11 Table 11-4; RDG-001.

**Planned verification**: Every defined/reserved TTF, both packet types, missing TTF, non-TIMEIN and configuration boundaries.

**L3-RDG-202** · Parent: L2-RDG-202 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL map A ↔ 0x1, B ↔ 0x0, G ↔ 0x2, N ↔ 0x5, U ↔ 0x4, X ↔ 0xF. Keep raw declaration state separate from effective comparison value. If TFMT is absent, always report missing-required; with default policy disabled (the default), skip the comparison. Only caller opt-in uses effective A, basis default-derived, while retaining the finding. Explicit A retains explicit provenance; do not write an effective default back into TMATS. Record published default A separately from the explicit/missing declaration even when default comparison is disabled.

**Source**: 106-24R1 Chapter 9 Table 9-4, printed9-72/PDF78; Chapter 11 §11.2.3.2 a; RDG-002.

**Source supplement**: "R/R Ch 10 Status: R"; "Default: A" (106-24R1 Chapter 9 Table 9-4 TFMT row, printed 9-72 / PDF 78). "A value of “R” requires that the attribute be specified in the TMATS file whenever the attribute is allowed." (§9.5.1 a, printed 9-5 / PDF 11). "If the R/R Ch10 Status field is “R”, then the attribute must be included in the TMATS file if all other conditions apply even if it has a default."; "Default: This identifies the default value required to process a TMATS file when the file itself does not contain the attribute." (printed 9-6 / PDF 12). RDG-002; owner 2026-11-13, Q-TFMT-DEFAULT = opt-in labelled default. TelemetryWorks/irig106-tmats `docs/adr/0023-validation-passes-and-effective-values.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: effective resolver "never writes to the document". Opt-in comparison and provenance are owner policy; no defaults are inserted.

**Planned verification**: Every accepted letter/FMT pair in both directions, all mismatches and unknown/non-TIMEIN cases. Explicit A compares as explicit. Absent TFMT without opt-in raises missing-required finding and performs no comparison. Absent TFMT with opt-in compares default A to FMT 0x1/mismatch, labels default-derived and still raises missing-required finding. Verify raw declaration remains absent; no default insertion. These comparison tests belong to the later declaration/report implementation, not this targeted flag-code change. Cite Table 9-4 printed 9-72/PDF 78 visual check.

**L3-RDG-203** · Parent: L2-RDG-203 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Seven bidirectional pairs: B ↔ 0x0, A ↔ 0x1, G ↔ 0x2, I ↔ 0x3, U ↔ 0x4, N ↔ 0x5, X ↔ 0xF. RDG-003 I mapping is inferred, not stated; mismatch emits warning TF-010 labelled inferred. Handbook input remains plain values from caller, not parsed TMATS here.

**Source**: Owner 2026-11-11 ROADMAP question 1; 106-24R1 Table 9-4 and§11.2.3.2 a; RCC 124-22 Chapter7 printed7-3/PDF 149; RDG-002/003.

**Planned verification**: For each of7 letters, test matching FMT and every other nibble, reverse comparisons, missing/unknown values. Reproduce RCC 124-22 printed 7-3/PDF 149 R-1\TTF-1:1;, R-1\TFMT-1:I;, R-1\TSRC-1:I; ("Time format is internal", "Time source is internal"); compare FMT 3/SRC 0 and contradictory values.

**L3-RDG-204** · Parent: L2-RDG-204 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Map declared 0/1/2 to NTF 0/1/2 after TTF=2 context; retain both IEEE 1588 versions. Invalid/missing values cannot default to NTP.

**Source**: 106-24R1 Chapter 9 Table 9-4; Chapter 11 §11.2.3.3 a; RDG-004.

**Planned verification**: All 3 matching values, every pairwise mismatch, reserved NTF and numeric TFMT under an inconsistent TTF.

**L3-RDG-205** · Parent: L2-RDG-205 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Preserve observed source over declared source; lost/regained external lock can legitimately differ from the static declaration. SRC 3 remains reserved, not GPS.

**Source**: 106-24R1 Chapter 9 Table 9-4; Chapter 11 §11.2.3.2 a; RDG-005.

**Planned verification**: Each matching pair and mismatch, loss/regain sequence, missing TSRC and reserved SRC 3..14; information default unchanged.

**L3-RDG-206** · Parent: L2-RDG-206 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use Chapter 11 binary Chapter 4 decoding for 0 even though Table 9-4 says "Chapter 4 BCD"; record RDG-006 as the reason, not a guessed BCD packet parser.

**Source**: 106-24R1 Chapter 9 Table 9-4; Chapter 11 §11.2.1.1 g; Chapter 4 §4.7; RDG-006.

**Planned verification**: Every pair, mixed-channel values, missing SHTF, format 11 and BCD-looking bytes that must not change packet dispatch.

**L3-RDG-207** · Parent: L2-RDG-207 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Zero ITS before 106-17 means not-reported; unknown edition preserves freewheeling-or-not-reported ambiguity. Apply current meanings only to A/B/G with applicable declared edition.

**Source**: 106-24R1 Chapter 11 §11.2.3.2 a/Table 11-4/Figure 11-34; 106-05/07/13/15 Chapter 10 reserved31..12; RDG-007.

**Planned verification**: Fixtures per touched archived edition:106-05/07/13/15/17 and current; every ITS value, non-IRIG FMT and missing edition. Include archived 106-04 reserved bits 31–12 and EXT bits 3–1, with unresolved edition handling under L3-EDN-201.

**L3-RDG-208** · Parent: L2-RDG-208 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use byte sum for trust, word sum for comparison, emit TF-027 on word-only matches and TF-012 when byte trust fails. Never auto-switch per file while claiming the reading is settled.

**Source**: 106-24R1 §11.2.1.2 c and§11.2.1.1 j; RCC 123-20 Appendix A-2; RDG-008.

**Planned verification**: Byte-only, word-only, both, neither across the listed editions; reference handbook Appendix A-2 code; local recordings remain outside Git/CI and human Review pending.

**L3-RDG-209** · Parent: L2-RDG-209 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Apply L3-TAI-105 mappings and require override before unknown-scale conversion; retain inferred GPS relationship and policy assumption for IRIG UTC.

**Source**: 106-24R1 §11.2.3.2 a/§11.2.3.3; RCC 200-16 §1/Appendix A.2; RDG-009.

**Planned verification**: All FMT/NTF values, IRIG/RTC overrides, historical table transitions, cross-scale reference comparisons and stale table.

**L3-RDG-210** · Parent: L2-RDG-210 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use L3-TAI-107 date/time/scale checks, keeping the actual second 60 in the result and reporting TF-022.

**Source**: 106-24R1 Figure 11-13; RCC 200-16 §3.6/Appendix A.2; RDG-010.

**Planned verification**: Known/unknown year, June 30/December 31 candidates and neighbours,23:59 versus other times, UTC/assumed UTC/TAI/GPS/unknown.

**L3-RDG-211** · Parent: L2-RDG-211 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: A reset starts a new first-packet check; static Computer-Generated Data, Format 1 may precede time, dynamic data may not. Source None time counts as present but not absolute.

**Source**: 106-24R1 §11.2.3.2/§11.2.3.3, Chapter 10 §10.6.2, §11.2.1.1 i; RDG-011.

**Planned verification**: Two sessions each compliant, missing time in the second, static-only prefix, time None and malformed first time packet.

**L3-RDG-212** · Parent: L2-RDG-212 · Verification: I

**L1 ancestor**: L1-RDG-002

**Obligation**: The implementation SHALL implement RDG-012 only after its written analysis and edge-case tests. Preserve era, raw fractions, scale, policy assumptions, sentinel/ambiguity findings and table currency; never apply cumulative TAI-UTC to NTP or invent a leap indicator. Use accepted floor fraction rounding (owner 2026-11-13). PTP outside coverage uses the last known offset for UTC, labelled possibly out of date with TF-021 warning, while retaining TAI and offset/table provenance. Apply TF-029 WARNING to the all-zero NTP sentinel (no absolute value, raw kept) and TF-030 WARNING to NTP leap ambiguity (mapped value kept and labelled).

**Source**: "2,208,988,800" (RFC 5905 §6 Figure 4, p.14, 1 Jan 1970); "a value of zero is a special case representing unknown or unsynchronized time" (§6, p.14); "knowledge of all previous leap seconds is lost" (RFC 1305 Appendix E.8, p.78); Figure 8 (p.79) repeats the same NTP number for UTC second 60 and midnight. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012. Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: RFC 5905 Figure 4 date fixtures, era priority/pivot/assumption, all-zero timestamp, fraction boundaries, covered leap ambiguity and PTP coverage edges; implementation tests are not added by this documentation PR. Pin the two permanent IDs and their distinct value effects without changing TF-022.

## STRM

**L3-STRM-101** · Parent: L2-STRM-101 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Accept explicit monotonic caller progress and EOF, never host-clock reads. Before data commit Option A channel as L3-COR-103; undecidable choice yields counter-only until caller chooses. Keep bounded late window/reference neighbours/pending queue under mandatory caller caps; export findings incrementally. Default Standard and selectable Interpolate use the common answer/basis model but do not promise offline-nearest/count-ranking results. Never revise emitted answers.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader.

**Planned verification**: Long fixed-rate and burst streams remain within caps; incomplete advance channel inventory yields counter-only; later references/counts/lock changes cannot rewrite output; caller progress and EOF drive release.

**L3-STRM-201** · Parent: L2-STRM-201 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Compute preceding_absolute+signed_rtc_delta*100 ns from the committed channel at/before query counter. Standard is the accepted default; never select a future counter reference merely because its packet arrived first. Preserve configured late-order handling without changing the committed channel; include preceding-reference basis and no drift correction.

**Source**: "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Planned verification**: RCC 123-20 example day 10012:30:25.000 RTC1000000 and query1150000 -> .015; explicit/default Standard, future-only reference, other-channel references and no usable preceding time.

**L3-STRM-202** · Parent: L2-STRM-202 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use same-session, committed-channel, scale-compatible references r1>r0 and r0<=r<=r1. Owner-accepted interpolation arithmetic is t=t0+floor((r-r0)*(t1-t0)/(r1-r0)) using checked i128; this floor rounding is accepted owner 2026-11-13, C1. Record both references and drift correction; apply L3-STRM-203/205 release rules if the next reference is unavailable within the bound or a cap is reached. Never imply greater accuracy than the coarser reference resolution.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above.

**Planned verification**: 1 s RTC interval and1.000010 s absolute interval -> accepted midpoint0.500005 s; explicit Interpolate, endpoints, accepted floor rounding, wrap/equal counters and incompatible scale; timeout/cap/EOF fallback. Assert accepted floor rounding, including a non-integral result, endpoint basis and coarser precision.

**L3-STRM-203** · Parent: L2-STRM-203 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Hold bound=1000000000 ns+hold_tolerance, default hold_tolerance=100000000 ns (1.1 s). The caller supplies progress and the recorded deadline basis; do not read a wall clock. Gap tolerance is independently0 ns, so spacing>1 s is a finding during the hold allowance. At timeout/EOF release from a usable preceding reference as extrapolated with reason; absent one return counter-only. Chapter 10 §10.6.1c's <=1000 ms stream commit explains why time packets can appear late in file order; this does not guarantee interpolation succeeds. Keep hold and late-ordering bounds distinct, and retain the raw-network rate exception.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader.

**Planned verification**: Next reference at1 s,1 s+1 ns,1.05 s,at deadline/after deadline; gap finding inside hold; explicit progress stalls, EOF and no usable preceding reference; hold override remains labelled policy.

**L3-STRM-204** · Parent: L2-STRM-204 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Track observed time-packet presence separately from reference eligibility. For SRC=0xF or FMT=0xF/invalid TS, preserve packet/invalid evidence and do not correlate from that reference in stream modes. Before any eligible preceding reference return counter-only with TF-002 when it precedes the first time packet, and the applicable invalid/undeclared evidence otherwise.

**Source**: Owner 2026-11-11 ROADMAP question 2; 106-24R1 Chapter 10 §10.6.2 first dynamic packet; Chapter 11 §11.2.3.2 source None/invalid FMT. Missing-time fallback is product policy; the standard gives no such algorithm.

**Planned verification**: Data-before-time, first time packet SRC None with a valid-looking body, FMT NONE, TS invalid, no time until EOF, and usable reference arriving later must not retroactively invent an early absolute answer.

**L3-STRM-205** · Parent: L2-STRM-205 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Constructor requires explicit pending packet/byte and retained-reference caps. Before a cap would be exceeded, release pending data through the preceding-reference fallback, labelled extrapolated/cap reason; if no usable preceding time return counter-only with evidence. Timeout/EOF use the same release discipline, never silent dropping, a hidden queue, or an optional resource-error alternative in place of the accepted release rule. Reject invalid resource configuration separately. Do not interpolate across jump/reset/incompatible scale, nor revise emitted answers.

**Source**: "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate.

**Planned verification**: Packet/byte cap exactly at/one beyond limit, reference flood, data burst, timeout/EOF, no reference, invalid cap configuration and discontinuity; no state exceeds caller caps and release reason is retained.

**L3-STRM-206** · Parent: L2-STRM-206 · Verification: T

**L1 ancestor**: L1-STRM-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: On explicit mode change, drain pending data under the old snapshot before installing the new one (accepted owner 2026-11-13, C4). Retain old mode/deadline/cap/progress/fallback basis on existing answers; never revise output or cross session boundaries. Channel selection follows accepted Option A (L3-COR-103), not the former unresolved Q-STREAM-CHANNEL question; source loss never implicitly chooses another channel.

**Source**: "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above.

**Planned verification**: Mode change with pending data preserves old answer basis and resource bounds; explicit channel commitment and no count-based or lost-lock reassignment; no retroactive answer changes. Test the accepted old-snapshot drain with pending data and unchanged channel commitment.

## ERR

**L3-ERR-101** · Parent: L2-ERR-101 · Verification: T

**L1 ancestor**: L1-ERR-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Check slice lengths before indexing; checked arithmetic returns field/range/overflow errors. Value parsers use fixed-size operations; fuzz/properties exercise arbitrary inputs. Allocation failure handling is bounded by the caller's configured stream limits, not a claim of recoverable global allocator OOM.

**Source**: Prototype identifier kept.

**Planned verification**: All shorter lengths, arbitrary bytes per parser, arithmetic extremes, malformed field combinations, and repeated input; no catch-and-hide panic as parsing strategy.

**L3-ERR-102** · Parent: L2-ERR-102 · Verification: T

**L1 ancestor**: L1-ERR-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use an error variant carrying field identifier, digit index and raw bits; distinguish BCD digit failure from an invalid total hour/day or always-zero-bit finding. Timeline converts parse failure into TF-004 without losing the original error.

**Source**: §11.2.3.2 b. Prototype identifier kept.

**Planned verification**: Digits A..F at each BCD position; invalid hour 24 with valid digits yields range error instead; zero-mask violations produce their own finding.

**L3-ERR-103** · Parent: L2-ERR-103 · Verification: T

**L1 ancestor**: L1-ERR-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Validate hours<=23, minutes<=59, nanos<1e9, day 1..366, month 1..12, and day-of-month against year/month. Seconds<=59 except approved UTC 23:59:60 dates; unknown year never invents leap certainty. Microseconds in Chapter 4 require<=9999.

**Source**: Figures 11-13, 11-14; RDG-010. Prototype identifier kept; day 0 is rejected, never turned into day 1 (contract 8.3); second 60 is no longer always rejected (T-16).

**Planned verification**: Every minimum/maximum and one out-of-range value, Feb 29 in leap/common years, day 0, second 60 combinations, and no silent day 0->1 repair.

**L3-ERR-104** · Parent: L2-ERR-104 · Verification: T

**L1 ancestor**: L1-ERR-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Preserve stored, byte-calculated and word-calculated checksums; no trusted secondary time on byte-reading failure under RDG-008. The packet's independent RTC may still support a counter answer, labelled with the failed own-time evidence.

**Source**: §11.2.1.2 c. Prototype identifier kept.

**Planned verification**: Stored checksum corruption in either byte, byte-only/word-only cases, and counter-derived answer remaining available without claiming own time valid.

**L3-ERR-105** · Parent: L2-ERR-105 · Verification: T

**L1 ancestor**: L1-ERR-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Require 4-byte CSDW then 6/8-byte Format 1 body or 8-byte Format 2 body; slice only required fields within the bounded input. Extra trailing bytes are not read as a phantom reserved word. Document consumed byte counts and reject underlength before any field access.

**Source**: "Data Length … does not include packet trailer filler and data checksum" (§11.2.1.1 d); T-12.

**Planned verification**: Data Length 10/12 for Format 1 and 12 for Format 2, every smaller length, nonzero filler after the boundary, and a guard slice showing no overread. T-12 fixture.

## API

**L3-API-101** · Parent: L2-API-101 · Verification: I

**L1 ancestor**: L1-API-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use compile-time trait assertions for Debug/Clone/PartialEq and for Copy/Eq/Hash on fixed scalar values; document why collection-owning timeline types cannot be Copy. Do not add alternative shared type definitions to achieve derives.

**Source**: Prototype identifier kept.

**Planned verification**: Planned inspection: public-type inventory with derive assertions, including feature-gated types and corrected shared types; absence of Copy on owning collections justified.

**L3-API-102** · Parent: L2-API-102 · Verification: I

**L1 ancestor**: L1-API-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Core value modules must not allocate; recording modules may use alloc collections. std is a default feature adding std::error::Error/conveniences, with no std-only type leaking into no-default public signatures.

**Source**: ADR-0014. Prototype identifier kept.

**Planned verification**: Planned inspection: no-default host and no_std/WASM builds; review allocation paths in each value reader, and default/no-default API signature comparison.

**L3-API-104** · Parent: L2-API-104 · Verification: I

**L1 ancestor**: L1-API-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use Rtc/Ertc, typed nanosecond durations, UnixSeconds/TaiSeconds and TaiUtcOffset from corrected irig106-types; plain u32 is confined to raw wire fields. Checked conversion functions document units, epoch and sign instead of accepting interchangeable integer seconds.

**Source**: Prototype identifier kept; `irig106-types` newtypes.

**Planned verification**: Planned inspection: signature inventory and compile-fail examples for mixing RTC ticks with nanos or TAI with Unix seconds; explicit successful conversions.

**L3-API-105** · Parent: L2-API-105 · Verification: I

**L1 ancestor**: L1-API-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Dependency review checks normal and feature-enabled edges from irig106-time: no core/tmats/decode/write/index/CLI edge. Optional serde/chrono are allowed under their features; tests must not turn a temporary cross-repo path into a runtime dependency.

**Source**: ADR-0005. Replaces L1-API-003 ("zero required runtime dependencies").

**Planned verification**: Planned inspection: cargo-tree review for every feature combination, manifest edge inventory, and standalone library-use example using plain declarations.

**L3-API-106** · Parent: L2-API-106 · Verification: I

**L1 ancestor**: L1-API-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Library APIs accept slices/value records and never open files, fetch leap data, read wall/monotonic clocks, environment or network. Stream expiry receives explicit progress from caller. Do not infer year, scale or era from the execution environment.

**Source**: ADR-0006.

**Planned verification**: Planned inspection: forbidden I/O API audit and deterministic replay with different host time/environment; no_std alone is not proof of no I/O through foreign imports.

**L3-API-107** · Parent: L2-API-107 · Verification: I

**L1 ancestor**: L1-API-007

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Avoid host-only interfaces in exported library signatures; WASM compile checks cover no-default and no-default + serde. Serialization of u64/i128 quantities must preserve exact values rather than rely on JavaScript floating-point integers. In snapshot/report JSON, 64/128-bit quantities SHALL be decimal strings with units under the reviewed versioned schema; preserve exact signed values and raw/stated/assumed provenance.

**Source**: `irig106-tmats` L1-REL-003; contract 4.8; ADR-0014. Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation.

**Planned verification**: Planned inspection: two WASM builds plus boundary serialization fixtures for values above 2^53, alongside native feature checks. Exercise exact signed/unsigned 64/128-bit extrema and units in JSON; never pass them through floating-point.

**L3-API-108** · Parent: L2-API-108 · Verification: I

**L1 ancestor**: L1-API-008

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Require missing_docs lint and rustdoc warnings-as-errors; a human citation audit verifies section/figure accuracy because lint presence does not validate content. Include examples for unresolved context and both stream modes.

**Source**: ADR-0004; architecture section 9.

**Planned verification**: Planned inspection: rustdoc output and public-item citation checklist, invalid/missing context examples, and a compilable worked example.

**L3-API-109** · Parent: L2-API-109 · Verification: I

**L1 ancestor**: L1-API-009

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use forbid(unsafe_code) in library and CLI roots; dependency unsafe is separately assessed by dependency policy, not mislabeled as this project's source. Audit generated/included source paths as part of the source boundary.

**Source**: Architecture section 9.

**Planned verification**: Planned inspection: root attributes and all-target feature builds; a temporary compile-fail fixture proves unsafe cannot be reallowed in a child module.

**L3-API-110** · Parent: L2-API-110 · Verification: T

**L1 ancestor**: L1-API-010

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Serialize/deserialize answers, findings, policies, tables, timelines and value types; validate numeric/date ranges and reference/session links on input. Unknown raw values retain their explicit unsupported designation; large integers remain exact. Use the c59e004 shared serde vocabulary: InternalFromRmm, Reserved(3), TimeFormat::None and IrigTimeSource. Removed source variants Gps and InternalRtc must not be emitted or silently accepted as current values; test their rejection separately from native-GPS FMT and ExternalGps ITS, which remain valid. Snapshot/report JSON SHALL be versioned with units; 64/128-bit quantities SHALL be decimal strings. Validate supported loads and retain raw/stated/assumed provenance; review the schema once before freeze.

**Source**: Architecture section 9; tools store and exchange answers (contract 4.7 to 4.9). Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation.

**Release scope**: Only deserialization that rehydrates complex timeline state is deferred past 0.8.0 (owner 2026-11-13, plan Q3). Disable/omit unsupported complex Deserialize. Validating existing public value deserialization, versioned policy/answer/timeline output and JSON schema remain 0.8.0 blockers; this is not a blanket serde deferral. See [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Roundtrip every public type, malformed/out-of-range serialized fields, dangling IDs, unknown schema version, and values above 2^53; no bare absolute time substituted for an answer. Unsupported complex timeline Deserialize is disabled/omitted for 0.8.0; supported value loads remain validated and complete versioned output stays in scope.

**L3-API-111** · Parent: L2-API-111 · Verification: T

**L1 ancestor**: L1-API-011

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Require known year and known scale; normalize non-UTC scale explicitly with table provenance before producing a UTC-like chrono result. Unknown context returns a typed refusal. A leap second must be preserved through supported chrono representation or explicitly rejected, never normalized silently.

**Source**: Architecture section 9; L1-TAI-006.

**Planned verification**: Known UTC and explicitly converted TAI/GPS, unknown year/scale refusals, leap second, stale table and chrono range extremes, plus reverse conversions with declared scale.

**L3-API-112** · Parent: L2-API-112 · Verification: I

**L1 ancestor**: L1-API-012

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Workspace lint policy denies unwrap_used,expect_used,panic,todo,unimplemented,unreachable,dbg_macro,print_stdout,print_stderr. Allowances stay limited to cfg(test) code and CLI main as L1 permits; review allowances to prevent a blanket library exemption.

**Source**: L1-ERR-001 (no panics on any input); ADR-0006 (no I/O). Clippy's lints `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`, `print_stdout`, `print_stderr` enforce it; they replace the text-pattern semgrep rules of the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`). As `irig106-tmats` L1-ROB-002. From the first rebuilt code (the prototype has unaudited `unwrap` calls, ROADMAP P6-09).

**Planned verification**: Planned inspection: lint settings/inheritance audit and negative lint fixtures; exercise first rebuilt modules under all-target/all-feature Clippy with warnings denied.

## PERF

**L3-PERF-101** · Parent: L2-PERF-101 · Verification: I

**L1 ancestor**: L1-PERF-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Bench fixed, documented input sizes and hardware/runner conditions; record baseline distribution and allocation/retained-state measures. Compare with prototype baseline and 100 Mbps ecosystem objective without treating old numbers as a new guarantee; owner-reviewed budgets follow the first measurement.

**Source**: As `irig106-tmats` L1-PERF-001: a budget set after measuring rather than guessed. The prototype's measurements (`docs/benchmark_results.md`) are the baseline; the draft ecosystem L1 of the retired `irig106-rust` asked for "minimum 100 Mbps sustained" (REQ-L1-102), and the first measurement is compared with that figure.

**Planned verification**: Planned inspection: bounded CI benchmark artifacts for 0/1/many references and several channels, reproducible seed/workload, and explicit budget approval; no fabricated throughput requirement.

## CLI

**L3-CLI-101** · Parent: L2-CLI-101 · Verification: I

**L1 ancestor**: L1-CLI-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Expose args, input, commands, report, render and run modules. Main maps the returned exit classification to process exit. File I/O and packet walking stay outside the time library.

**Source**: ADR-0007; owner, 2026-09-26 and 2026-09-27.

**Planned verification**: Inspect workspace membership, binary naming and the public entry point; compare standalone invocation with an embedded library call.

**L3-CLI-102** · Parent: L2-CLI-102 · Verification: T

**L1 ancestor**: L1-CLI-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Run accepts injected arguments and input/output interfaces and returns a structured outcome; it must not call process::exit. The host strips its time subcommand before invoking the shared parser. No reverse dependency on irig106-cli.

**Source**: ADR-0007; ROADMAP P6-10.

**Planned verification**: Standalone and host-style calls produce the same report; injected I/O failures and help return without terminating the host.

**L3-CLI-103** · Parent: L2-CLI-103 · Verification: I

**L1 ancestor**: L1-CLI-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Validate unknown, missing and duplicate arguments, numeric limits, help/version and the -- terminator before input is opened. Keep syntax and help documentation synchronized.

**Source**: ADR-0008.

**Planned verification**: All six commands, every option, invalid limits, missing arguments, filenames beginning with a dash, and dependency inspection.

**L3-CLI-104** · Parent: L2-CLI-104 · Verification: T

**L1 ancestor**: L1-CLI-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Summary shows sessions/spans/quality; channels shows declarations/observations; jumps shows residuals/thresholds; timeline and csv show packet answers; correlate shows requested session/counter answers. No command independently converts time or assigns fixed event meanings.

**Source**: Contract 4.12; ADR-0011, ADR-0013.

**Planned verification**: A synthetic recording with both formats, two sessions, gaps/jumps, unknown year and counter-only answers; assert relevant provenance in each command.

**L3-CLI-105** · Parent: L2-CLI-105 · Verification: T

**L1 ancestor**: L1-CLI-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use decoded NTF and TS directly. Packet type 0x12 and reserved source 3 must never imply GPS. Keep protocol/version distinctions in text, CSV and JSON.

**Source**: ROADMAP P6-10; §11.2.3.3.

**Planned verification**: Each NTF with valid and invalid status in every renderer; native GPS Format 1 remains distinct from network time.

**L3-CLI-106** · Parent: L2-CLI-106 · Verification: T

**L1 ancestor**: L1-CLI-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: JSON includes schema version, basis, findings and policy identity with exact large integers. Document CSV columns, quoting and absent values; text distinguishes unresolved from zero. Incompatible schema changes require a version change. JSON SHALL use a reviewed versioned snapshot/report schema with units, decimal strings for 64/128-bit quantities, supported-load validation and raw/stated/assumed provenance; review the schema once before freeze.

**Source**: As `irig106-tmats` L1-CLI-006; `irig106-cli` and scripts rely on the output (contract 4.10); architecture section 4.4. Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation.

**Planned verification**: Golden outputs with quotes/commas/newlines, unknown year/scale, counter-only and values above 2^53; schema validation and report round trips. Check schema/version/units, exact decimal strings and provenance in golden reports; align with library snapshots.

**L3-CLI-107** · Parent: L2-CLI-107 · Verification: T

**L1 ancestor**: L1-CLI-007

**Obligation**: The implementation SHALL map the shared runner outcome to 0=success, 1=findings at/above chosen severity, 2=usage error, 3=unreadable or invalid Chapter 10/time input, 4=output-write failure. Validate usage in the hand-rolled parser before input access. Keep input and output failures distinct and do not turn failed or incomplete I/O into success or a finding-only outcome. Return classifications from the runner; only main maps them to process exit. These are accepted ecosystem assignments, not proposed numbers or IRIG 106 rules; never renumber them. After usage validation and before a finding-only outcome, the first fatal operational failure SHALL determine code 3 for input or 4 for output; later failures become diagnostics and do not replace that first fatal classification.

**Source**: As `irig106-tmats` L1-CLI-006 (the existing L1 source); TelemetryWorks/irig106-tmats, `docs/CLI.md` §4, lines 88–91, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: "`0` success; `1` the check failed (`verify` mismatch, `validate` findings at or above the failure threshold); `2` usage error; `3` input could not be read or is not TMATS/Chapter 10; `4` output could not be written. To be fixed before 0.1 and never renumbered." Owner 2026-11-12, Q-CLI-EXIT = align with irig106-tmats; findings/severities ADR-0013, hand-rolled parser ADR-0008. IRIG 106 is silent on tool exit codes: no standard source exists. This is a shared ecosystem convention so `irig106 tmats` and `irig106 time` agree; code 3 specializes the domain to Chapter 10/time input. See `docs/analysis/Q-CLI-EXIT-2026-11-12.md`. Owner 2026-11-13, plan Q6: usage before input; first fatal operational failure determines code 3 (input) or 4 (output), later failures are diagnostics. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). IRIG 106 is silent on process exit codes and their precedence.

**Planned verification**: One test per numeric code: 0 successful output with no threshold findings; 1 finding at/above the chosen severity, including threshold changes; 2 hand-rolled parser usage errors (ADR-0008), including unknown options, missing arguments and usage error before opening a nonexistent input; 3 unreadable input and invalid Chapter 10/time input; 4 injected output-write failure. Verify the shared runner returns the classification for host invocation without exiting the process; input/output failure must not be reported as success or only findings. Code/tests belong to the CLI restructure step, with /// Requirements: markers for L1-CLI-007, L2-CLI-107 and L3-CLI-107, correct Traces tags and regenerated matrix linking requirement → code → test; these are planned checks, not executed tests. Inject both input and output failures in each occurrence order, including diagnostic-output failure after an input failure; verify first fatal code wins and later diagnostics are retained.

## REL

**L3-REL-101** · Parent: L2-REL-101 · Verification: I

**L1 ancestor**: L1-REL-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Both packages use X.Y.Z and the CLI pins =X.Y.Z, with a local path for development and published resolution for packaging. Reject version drift or an unpublishable source. Verification does not authorize publication. Actual publication SHALL use crates.io Trusted Publishing/OIDC from GitHub Actions only; no stored publishing token and no local publish. 0.8.0 ships both crates to crates.io without prebuilt binaries; this documentation grants no publication authority.

**Source**: ADR-0007. Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes.

**Planned verification**: Planned inspection: manifest/package metadata and dry-run artifacts for both packages; intentionally mismatched versions/pins must fail. Inspect publisher workflow/identity and no stored token/local publish path in authorised release work; no such workflow or settings are changed here.

**L3-REL-102** · Parent: L2-REL-102 · Verification: I

**L1 ancestor**: L1-REL-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Both manifests declare the agreed edition/MSRV. CI checks library normal features on 1.85 without relying on newer dev dependencies; stable runs tests, lints and docs. Later APIs require compatible alternatives or a reviewed floor change.

**Source**: ADR-0003.

**Planned verification**: Planned inspection: metadata and supported MSRV feature matrix, including serde/chrono; failures cannot be hidden by running only a newer compiler.

**L3-REL-103** · Parent: L2-REL-103 · Verification: I

**L1 ancestor**: L1-REL-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Maintain finding-to-release notes with old/new meaning and effect, including shared types, MSRV, recording-event removal and CLI rename. Preserve published 0.1.0 through 0.7.0 per ADR-0017; no automatic yank.

**Source**: ADR-0017.

**Planned verification**: Planned inspection: changelog versus STANDARD-REVIEW and API diff; caption-only and documentation-only findings distinguished from behaviour changes.

**L3-REL-104** · Parent: L2-REL-104 · Verification: I

**L1 ancestor**: L1-REL-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Run supported builds/tests and CLI integration checks on each platform with exact commit, toolchain and features in artifacts. Missing dependencies are setup failures, not permission to omit a platform silently.

**Source**: As `irig106-tmats` L1-REL-004; the draft ecosystem L1 (REQ-L1-103, the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`)).

**Planned verification**: Planned inspection: three-platform results, CLI coverage, path/newline fixtures and failure propagation; current prototype results do not verify future rebuilt code.

**L3-REL-105** · Parent: L2-REL-105 · Verification: I

**L1 ancestor**: L1-REL-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Check resolved lockfiles and feature sets with cargo-deny for advisories, maintenance, yanks, licences and sources. Retain the reviewed irig106-types Git exception until a reviewed replacement; do not loosen global policy for one failure.

**Source**: `deny.toml`, adapted from the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`); as `irig106-tmats` L1-REL-005. Its first run found `crossbeam-epoch` below 0.9.20 (RUSTSEC-2026-0204, through `criterion`), fixed by updating the lock file.

**Planned verification**: Planned inspection: policy results and exception inventory, exact Git revision and negative policy cases; no settings or credential changes.

**L3-REL-106** · Parent: L2-REL-106 · Verification: I

**L1 ancestor**: L1-REL-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: The release plan binds binary digest, source commit, workflow identity and dependency inventory. Verification instructions detect an altered binary or missing attestation; this requirement grants no publishing or policy-change authority.

**Source**: As `irig106-tmats` L1-REL-006 and its `docs/RELEASING.md` step 7. Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes.

**Release scope**: Prebuilt binary distribution is deferred; 0.8.0 is crates.io-only (owner 2026-11-13, plan Q3/Q4). SBOM/provenance remains mandatory for every binary eventually distributed. No binary is shipped in 0.8.0; the conditional requirement is retained and not marked covered. See [0.8 plan decisions](analysis/plan-decisions-2026-11-13.md).

**Planned verification**: Planned inspection: reviewed release workflow and sample verification, including digest mismatch and missing provenance.

## TST

**L3-TST-101** · Parent: L2-TST-101 · Verification: I

**L1 ancestor**: L1-TST-001

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Inventory T-1 through T-17, marking caption-only, documentation-only and unresolved-source cases explicitly. Each code test cites the finding and requirement IDs; retain before/after evidence and use corrected shared-value contracts.

**Source**: ADR-0016.

**Planned verification**: Planned inspection: finding/test inventory with prototype-fail/fix-pass results; an unrelated passing test cannot stand in for a missing regression. Current inventory and remaining gaps: [S00 audit](analysis/S00-decision-source-evidence-audit.md). This audit is not satisfaction evidence and does not promote this obligation.

**L3-TST-102** · Parent: L2-TST-102 · Verification: T

**L1 ancestor**: L1-TST-002

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Use its 36-byte time packet, Data Length 10, RTC 100000000, day 187 at 13:45:27.350, and query RTC 104200000 giving 13:45:27.770 under offline nearest defaults. Assert session, channel, assumed UTC, 10 ms precision, unknown year, distance 0.420 s and policy.

**Source**: Contract 7.6; ADR-0016.

**Planned verification**: Independent byte array, perturbed length/flags/date cases and full answer comparison; explicitly selected preceding/interpolation variants must name their policy.

**L3-TST-103** · Parent: L2-TST-103 · Verification: I

**L1 ancestor**: L1-TST-003

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Each fixture names edition, chapter, page, figure/table and reading. Review bit positions and arithmetic independently; round trips supplement golden bytes and cannot replace them. Image-only layouts require visual inspection.

**Source**: ADR-0016.

**Planned verification**: Planned inspection: fixture citations and recalculation, Figures 11-4/4-4/11-12 through 11-17 and the RDG-003 handbook example.

**L3-TST-104** · Parent: L2-TST-104 · Verification: I

**L1 ancestor**: L1-TST-004

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Review binary fixtures for synthetic provenance and bounded size. Local recording results are separate evidence and never justify publishing raw data or fetching it in CI.

**Source**: Owner's standing direction; ADR-0016; `irig106-tmats` ADR-0018.

**Planned verification**: Planned inspection: fixture provenance/size audit, ignore rules and CI input inventory; synthetic coverage must not be called vendor validation.

**L3-TST-105** · Parent: L2-TST-105 · Verification: I

**L1 ancestor**: L1-TST-005

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Keep the existing L2 heading/Parent line and L3 one-line Parent/Verification formats. Separately inspect unique IDs, complete L1 coverage, orphans and unknown markers. Regeneration establishes freshness, not semantic correctness; unevidenced obligations remain Draft.

**Source**: Architecture section 12; as `irig106-tmats`'s `scripts/build-trace-matrix.py`.

**Planned verification**: Planned inspection: regenerate then --check; hierarchy/marker audit and an in-memory stale-output negative check without committing a stale artifact.

**L3-TST-106** · Parent: L2-TST-106 · Verification: I

**L1 ancestor**: L1-TST-006

**Obligation**: The implementation SHALL satisfy the following layout or procedure: Plan cargo-llvm-cov with --fail-under-lines 80 over library sources, recording features, commit and exclusions. Review uncovered parser/error branches; line coverage does not prove standard correctness. Report CLI coverage separately.

**Source**: As `irig106-tmats` L1-ROB-003; the draft ecosystem L1 ("minimum 80% code coverage", REQ-L1-111, the retired `irig106-rust` repository (its `semgrep-rust.yml`, its draft L1 specification, `CODE_ANALYSIS.md`, and `notes/SLSA1.md`; kept in `irig106-tmats` `docs/research/2026-09-26-irig106-rust/`)).

**Planned verification**: Planned inspection: coverage artifact, configured gate, a below-threshold failure and documented exclusions.

## Retired prototype identifiers

These definitions belonged to the prototype document replaced by this
draft. They are historical identifiers, not active requirements or aliases
for the new IDs. Existing test-table mentions must be reassessed during
the tests-first rebuild, not mechanically relabelled.

`L3-1588-001`, `L3-1588-002`, `L3-1588-003`, `L3-1588-004`, `L3-ABS-001`, `L3-ABS-002`, `L3-ABS-003`, `L3-ABS-004`, `L3-ABS-005`, `L3-ABS-006`, `L3-BCD-001`, `L3-BCD-002`, `L3-BCD-003`, `L3-BCD-004`, `L3-BCD-005`, `L3-BCD-006`, `L3-BCD-007`, `L3-CH4-001`, `L3-CH4-002`, `L3-CH4-003`, `L3-CH4-004`, `L3-CH4-005`, `L3-COR-001`, `L3-COR-002`, `L3-COR-003`, `L3-COR-004`, `L3-COR-005`, `L3-COR-006`, `L3-COR-007`, `L3-CSDW-001`, `L3-CSDW-002`, `L3-CSDW-003`, `L3-CSDW-004`, `L3-CSDW-005`, `L3-CSDW-006`, `L3-CSDW-007`, `L3-CSDW-008`, `L3-CSDW-009`, `L3-CSDW-010`, `L3-ERR-001`, `L3-ERR-002`, `L3-ERR-003`, `L3-ERR-004`, `L3-ERTC-001`, `L3-ERTC-002`, `L3-ERTC-003`, `L3-F2-001`, `L3-F2-002`, `L3-F2-003`, `L3-F2-004`, `L3-F2-005`, `L3-F2-006`, `L3-F2-007`, `L3-F2-008`, `L3-F2-009`, `L3-F2-010`, `L3-F2-011`, `L3-F2-012`, `L3-F2-013`, `L3-F2-014`, `L3-IPT-001`, `L3-IPT-002`, `L3-IPT-003`, `L3-RTC-001`, `L3-RTC-002`, `L3-RTC-003`, `L3-RTC-004`, `L3-RTC-005`, `L3-RTC-006`, `L3-RTC-007`, `L3-RTC-008`, `L3-RTC-009`, `L3-RTC-010`, `L3-RTC-011`, `L3-RTC-012`, `L3-RTC-013`, `L3-SEC-001`, `L3-SEC-002`, `L3-SEC-003`, `L3-SEC-004`.
