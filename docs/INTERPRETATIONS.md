# irig106-time — Reading register

> **Status: hand-written during the design phase** (ADR-0012, accepted
> 2026-09-27). Started at the completeness check of step 1; every entry is
> proposed, open, or suspect until the owner reviews it.

The IRIG 106 standard is the authority, but where time is concerned it is
not always consistent or explicit: TMATS and the packets name the same
things differently, fields arrived in later editions without a version to
say so, and some wording admits two readings. Each entry here records one
place where the design had to choose: the sources, quoted exactly; the
behaviour chosen; why; its review; and the tests that pin it. **Nothing in
the code may rely on a reading that is not listed here** (L1-RDG-001).

## Conventions

The form is `irig106-tmats`'s `docs/INTERPRETATIONS.md`, so one reviewer
reads both.

- **Identifiers** are `RDG-NNN`, permanent, never reused. (`irig106-tmats`
  uses `INT-NNN`; the prefixes differ so that a reference names its
  register.)
- **Sources** are quoted verbatim from the archived standard
  (`TelemetryWorks/rcc-106-standards`), 106-24R1 unless stated. "Table 9-4"
  is Chapter 9's Recorder-Reproducer Attributes Group; "§11.x" is Chapter 11.
- **Design** is the owner's decision on the behaviour: `accepted`,
  `suspect` (accepted but held in doubt until checked against real data),
  `proposed` (drafted while documenting, not yet reviewed), or `open` (no
  behaviour chosen; the implementation must not choose one).
- **Review** is the two-person rule of `irig106-tmats` ADR-0022: an author
  and a different reviewer, both people; tooling or AI drafting counts as
  neither. Until both are recorded the review is `pending`.
- **Test** names the tests that pin the behaviour. Each carries a
  `/// Interpretations: RDG-NNN` doc comment directly above `#[test]`;
  `scripts/build-trace-matrix.py` lists every entry and the tests that name
  it.
- **Origin** is the finding (`docs/STANDARD-REVIEW.md`) or contract section
  that raised it.
- **Development** — every entry is marked **intensive testing and deep
  analysis required**, as in `irig106-tmats` (owner direction, 2026-09-26):
  before its code is written, the entry needs a written analysis (every
  cited source re-read in every archived edition it touches, edge cases
  listed), a test per edge case, a fixture per edition it touches, and,
  where local recordings exercise it, a check against real data (local only,
  never in CI).
- **Analysis** links the written analysis, or says `not yet written`.

---

## TMATS against the packets

### RDG-001

**Title**: `R-x\TTF-n` 1 and 2 name the packet formats `0x11` and `0x12`

**Sources**:
- Table 9-4, `R-x\TTF-n` (Time Data Type Format): "0 Reserved", "1 Time data", "2 Network time"; "Allowed when: R\CDT is "TIMEIN"".
- Table 11-4: "0x11 Time Data, Format 1"; "0x12 Time Data, Format 2 Network Time".

**Behaviour**: `TTF` 1 declares Format 1 (`0x11`) time packets and `TTF` 2 declares Format 2 (`0x12`); `TTF` 0 is reported as reserved. When the packets' data type differs, the packets govern what they contain and the difference is reported.

**Reason**: Table 9-4 names the formats in words, Table 11-4 in numbers; the pairing is by name, not stated.

**Design**: accepted · **Review**: pending · **Origin**: contract 3.7, 6.4

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

### RDG-002

**Title**: `R-x\TFMT-n` letters A, B, G, N, U, X name Format 1 FMT values

**Sources**:
- Table 9-4, `R-x\TFMT-n`: "Indicate the format for the time. For additional information, see RCC 200-16. y is an optional last digit"; "A IRIG-A 1xy", "B IRIG-B 1xy", "G IRIG-G 1xy", "I Internal", "N Native GPS time", "U UTC time from GPS", "X None"; "Default: A" (visually confirmed in the `TFMT` row, printed page 9-72, PDF page 78).
- §11.2.3.2 a, FMT (bits 7–4): "0x0 = IRIG-B", "0x1 = IRIG-A", "0x2 = IRIG-G", "0x3 = Real-Time Clock", "0x4 = Universal Coordinated Time (UTC) time from GPS", "0x5 = Native GPS Time", "0xF = NONE (time packet payload invalid)".

**Required/default semantics (pages rendered and inspected):** Table 9-4's
TFMT row (9-72 / PDF 78) also says "R/R Ch 10 Status: R". Chapter 9
§9.5.1 a (9-5 / PDF 11) defines: "A value of “R” requires that the
attribute be specified in the TMATS file whenever the attribute is allowed."
On 9-6 / PDF 12, "Default: This identifies the default value required to
process a TMATS file when the file itself does not contain the attribute."
Its note states: "If the R/R Ch10 Status field is “R”, then the attribute
must be included in the TMATS file if all other conditions apply even if
it has a default." Thus **missing-required validation and default-based
processing are different questions**. A default does not make the TMATS
conforming. The accepted behaviour below still forbids silent substitution;
these source definitions do not record a new Design decision.

**Owner question Q-TFMT-DEFAULT:** should declared default A feed the
comparison when explicit TFMT is missing? Recommend explicit opt-in
application, labelled default-derived, **while retaining the missing-required
finding**. The default's role in processing supports that option; it does
not authorize dropping the finding. Keep the current accepted rule until
the owner decides. Planned tests must distinguish explicit A, absent TFMT
without default application, and absent TFMT with an approved labelled
default policy, including the required-field finding in the latter case.

**Behaviour**: A ↔ `0x1`, B ↔ `0x0`, G ↔ `0x2`, N ↔ `0x5`, U ↔ `0x4`, X ↔ `0xF`. A difference is reported. A missing `TFMT` on a TIMEIN channel is reported as missing, not silently read as IRIG-A; the table's default is recorded as a declared default, separately from an explicit value. Owner 2026-11-13, Q-TFMT-DEFAULT resolves the earlier question above: by default report the missing required attribute and make no TFMT comparison. Only explicit caller opt-in uses default A for comparison, labelled default-derived, while still raising the missing-required-attribute finding and preserving raw absence; never insert a default into TMATS (irig106-tmats ADR-0023).

**Page check (2026-11-12)**: visually inspected a `pdftoppm` rendering of
106-24R1 `Chapter9.pdf`, printed page **9-72**, one-based PDF page **78**,
Table 9-4. "Default: A" is inside the TIME FORMAT / `R-x\TFMT-n` row,
above the horizontal rule that starts TIME SOURCE / `R-x\TSRC-n`.
The row says "Allowed when: R\CDT is “TIMEIN”" and "Range: Enumeration";
its definition reads "Indicate the format for the time. For additional
information, see RCC 200-16." (footnote marker omitted). Thus the default
belongs to TFMT, not TSRC or the preceding TTF row. The figure does not
justify silently replacing the absent declaration in this library.
The archive fetcher verified SHA-256
`dfc5ebfb3e6f02f58389126863d43a45d89c7e2d0533a1c690d9537531c740cb`.
Reproduce with `pdftoppm -f 78 -l 78 -scale-to 2000 -png -singlefile
Chapter9.pdf table9-4`. [Archived Chapter 9](https://github.com/TelemetryWorks/rcc-106-standards/releases/download/106-24R1/Chapter9.pdf).
This resolves the page-placement question only; all-edition analysis,
edge-case tests, and two-person review remain outstanding.

**Reason**: The names match one for one. Applying the default silently would hide a non-conforming setup record.

**Design**: accepted · **Review**: pending · **Origin**: contract 3.7

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

### RDG-003

**Title**: `R-x\TFMT-n` I "Internal" has no clear FMT value

**Sources**:
- Table 9-4, `R-x\TFMT-n`: "I Internal".
- §11.2.3.2 a, FMT: "0x3 = Real-Time Clock"; SRC (bits 3–0): "0x0 = Internal (time derived from a clock in the recorder)".
- RCC 124-22 Chapter 7 example (page 7-3): `R-1\TFMT-1:I;` "Time format is internal", with `R-1\TSRC-1:I;` "Time source is internal". No archived source maps I to a FMT value.

**Behaviour**: I ↔ `0x3` (Real-Time Clock), an inference. A difference is reported as a warning, labelled inferred rather than stated.

**Reason**: The standard and both handbooks are silent. The other letters match FMT values one for one (B `0x0`, A `0x1`, G `0x2`, U `0x4`, N `0x5`, X `0xF`), which leaves I and `0x3`. Owner, 2026-11-11: tests for every letter ↔ value pair and for the RCC 124-22 example.

**Design**: accepted · **Review**: pending · **Origin**: contract 3.7

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

### RDG-004

**Title**: `R-x\TFMT-n` 0, 1, 2 name Format 2 NTF values

**Sources**:
- Table 9-4, `R-x\TFMT-n`: "0 Network Time Protocol Version 3 RFC-1305", "1 IEEE Std 1588-2002", "2 IEEE Std 1588-2008".
- §11.2.3.3 a, NTF (bits 7–4): "0x0 = Network Time Protocol Version 3", "0x1 = IEEE-1588-2002", "0x2 = IEEE-1588-2008".

**Behaviour**: 0 ↔ `0x0`, 1 ↔ `0x1`, 2 ↔ `0x2`; a difference is reported.

**Reason**: The lists match one for one.

**Design**: accepted · **Review**: pending · **Origin**: contract 3.7

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

### RDG-005

**Title**: `R-x\TSRC-n` letters name Format 1 SRC values; a difference is information

**Sources**:
- Table 9-4, `R-x\TSRC-n`: "I Internal", "E External", "R Internal from RMM", "X None".
- §11.2.3.2 a, SRC: "0x0 = Internal", "0x1 = External", "0x2 = Internal from RMM", "0xF = None"; "If the time source is external (0x1) and lock on the external source is lost then the time source shall indicate Internal (0x0)."

**Behaviour**: I ↔ `0x0`, E ↔ `0x1`, R ↔ `0x2`, X ↔ `0xF`. A packet reporting internal where TMATS declares external is reported as information (lost lock), not as a disagreement.

**Reason**: The standard itself makes the packets' source change while the declaration stays.

**Design**: accepted · **Review**: pending · **Origin**: contract 3.7, 5.2 rule 6

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

### RDG-006

**Title**: `R-x\SHTF-n` 0 "Chapter 4 BCD" names packet flags `00`, "Chapter 4 binary weighted"

**Sources**:
- Table 9-4, `R-x\SHTF-n`: "If enabled, the secondary header time format."; "0 Chapter 4 BCD", "1 IEEE-1588", "2 ERTC".
- §11.2.1.1 g, bits 3–2: "00 = Chapter 4 binary weighted 48-bit time format", "01 = IEEE 1588 time format", "10 = 64-bit binary extended relative time counter (ERTC)".
- Chapter 4 §4.7: "High and low order time words shall be binary or binary coded decimal (BCD) weighted".

**Behaviour**: 0 ↔ `00`, 1 ↔ `01`, 2 ↔ `10`. The secondary header is read as Chapter 11 defines it (binary weighted); the word "BCD" in Table 9-4 is noted, not followed.

**Reason**: Chapter 11 defines the packet; Chapter 9 describes it. Chapter 4 allows both weightings for PCM time words, which may be the source of the wording.

**Design**: accepted · **Review**: pending · **Origin**: contract 3.7

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

---

## Fields and wording

### RDG-007

**Title**: ITS (Format 1 bits 15–12) is read only for recordings declared 106-17 or later

**Sources**:
- §11.2.3.2 a: "IRIG Time Source (ITS). Bits 15-12 provide dynamic information regarding the source of time for an internal IRIG time code generator (TCG) when FMT is IRIG-A, B, or G"; "0000 = IRIG TCG freewheeling (no or loss of time source)".
- 106-05, 106-07, 106-13, 106-15 Chapter 10: "Reserved. Bits 31-12 are reserved".
- Table 11-4 (106-17 and 106-24R1): Time Data, Format 1, current data type version `0x06` (106-13).
- §11.2.1 f: "All reserved bit fields in packet headers or CSDWs shall be set to zero (0x0)".
- Figure 11-34: "0x0C = RCC 106-17".

**Behaviour**: ITS is reported only when FMT is IRIG-A, B, or G and the setup record declares 106-17 or later (RCCVER `0x0C` or above); otherwise it is "not reported", never "freewheeling". With no declared edition, `0000` is reported as "freewheeling or not reported".

**Reason**: The field arrived without a data type version change, so a 106-15 recording's zero bits would otherwise read as a freewheeling clock.

**Design**: accepted · **Review**: pending · **Origin**: T-14

**Test**: `tests/s03_edition.rs` (edition rows, all ITS values, non-IRIG FMT, unknown edition, reserved-field findings)

**Development**: intensive testing and deep analysis required · **Analysis**: [qualified edition fields](analysis/RDG-007-edition-fields.md)

### RDG-008

**Title**: The secondary header checksum sums bytes, not 16-bit words

**Sources**:
- §11.2.1.2 c: "a 16-bit arithmetic sum of all secondary header bytes excluding the secondary header checksum word" (the same in 106-05, 106-07, 106-13, 106-15, 106-17).
- §11.2.1.1 j (packet header, for contrast): "a 16-bit arithmetic sum of all 16-bit words in the header excluding the header checksum word".
- RCC 123-20 Appendix A-2, `uCalcSecHeaderChecksum`: adds the ten bytes one at a time into a 16-bit sum, with the comment "MAKE THIS 16 BIT UNSIGNEDS LIKE ABOVE".

**Behaviour**: the checksum is the 16-bit sum of the ten bytes. Each header is also checked by the other reading, and a recording whose headers satisfy only that one is reported, so real data can settle the question.

**Reason**: The wording says bytes, the packet header's contrasting wording says words, and the handbook's reference code sums bytes; the prototype sums words.

**Design**: suspect · **Review**: pending · **Origin**: T-15

**Test**: tests/slice2_wire.rs::rdg008_byte_only_checksum_is_valid, rdg008_word_only_checksum_is_not_trusted; tests/slice2_readings.rs::rdg008_reports_byte_word_both_and_neither

**Development**: intensive testing and deep analysis required · **Analysis**: [wire-slice analysis, limits and checked pages](verification-slice-two.md#rdg-008)

---

## Time scales and leap seconds

### RDG-009

**Title**: The time scale of each time format; IRIG time is UTC by assumption

**Sources**:
- §11.2.3.2 a, FMT: "0x4 = Universal Coordinated Time (UTC) time from GPS", "0x5 = Native GPS Time", "0x3 = Real-Time Clock".
- §11.2.3.3: "The NTP is referenced in UTC time … The NTP time includes leap seconds"; "The PTP is referenced in International Atomic Time … The PTP time does not include leap seconds".
- RCC 200-16 §1: "All Department of Defense (DoD) test ranges, facilities, and other government agencies such as the National Aeronautics and Space Administration (NASA) maintain Coordinated Universal Time (UTC) referenced to the United States Naval Observatory (USNO) Master Clock."
- RCC 200-16 Appendix A.2: "GPS time does not add or subtract leap seconds, and as of this writing, GPS time is 16 seconds ahead of UTC"; "There have been 35 leap seconds added to UTC"; "Ti = Ui + Li".

**Behaviour**: FMT `0x4` and NTP are UTC; PTP is TAI; FMT `0x5` is GPS time, TAI − 19 s; IRIG-A, B, and G are UTC by assumption, labelled "assumed"; FMT `0x3` is of unknown scale and is not converted unless the caller names its scale. Each assumption is a policy setting.

**Reason**: Chapter 11 states the scale for some formats only. The GPS offset is derived from RCC 200-16's own figures (TAI − UTC = 35 s when GPS was 16 s ahead of UTC), not stated as such; the IRIG assumption rests on range practice, not on a rule.

**Design**: accepted · **Review**: pending · **Origin**: T-17; contract 3.2

**Test**: tests/slice2_readings.rs::t17_f2_stated_scales_do_not_depend_on_counter_value, t17_all_fmt_scales_and_policy_provenance

**Development**: intensive testing and deep analysis required · **Analysis**: [wire-slice analysis, limits and checked pages](verification-slice-two.md#rdg-009)

### RDG-010

**Title**: Second 60 is a leap second on a day that can be 30 June or 31 December

**Sources**:
- RCC 200-16 §3.6: "The SBS TOD code reads 0 seconds at 2400 each day excluding leap second days when a second may be added or subtracted."
- RCC 200-16 Appendix A.2: "If required, time changes are made on December 31 and on June 30 at 2400 hours."
- Figure 11-13: tens of seconds (`TSn`) occupy bits 14–12, so second 60 can be encoded.

**Behaviour**: in UTC formats, 23:59:60 is accepted on day 181, 182, 365, or 366 (which can be 30 June or 31 December; with a known year, only the day that is) and reported as a leap second; second 60 at any other time, or in GPS, TAI, or real-time-clock time, is out of range.

**Reason**: Rejecting it loses a valid second of data; accepting it anywhere would hide malformed bodies.

**Design**: accepted · **Review**: pending · **Origin**: T-16; contract 3.2

**Test**: tests/slice2_readings.rs::t16_leap_second_retained_only_at_permitted_utc_dates, t16_calendar_known_year_and_wrong_time_boundaries

**Development**: intensive testing and deep analysis required · **Analysis**: [wire-slice analysis, limits and checked pages](verification-slice-two.md#rdg-010)

### RDG-011

**Title**: "The first dynamic data packet" is checked per session

**Sources**:
- §11.2.3.2: "A time data packet shall be the first dynamic data packet at the start of each session."
- §11.2.3.3 and Chapter 10 §10.6.2: "A time data packet shall be the first dynamic data packet at the start of each recording."
- §11.2.1.1 i: the counter "shall remain free-running during each session (e.g., recording)".

**Behaviour**: the rule is checked at the start of each session the time timeline finds (L1-COR-005); a recording is at least one session.

**Reason**: The session is the stricter of the two wordings and the unit within which the counter is continuous; a check per recording would miss a counter reset without a time packet after it.

**Design**: accepted · **Review**: pending · **Origin**: contract 6.1, 6.2

**Test**: not yet written

**Development**: intensive testing and deep analysis required · **Analysis**: not yet written

### RDG-012

**Title**: NTP follows UTC reckoning; era evidence, zero sentinel and leap ambiguity are explicit

**Sources**:
- 106-24R1 §11.2.3.3, printed 11-26 / PDF34: "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." / "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds."
- Figure 11-16, printed 11-27 / PDF35: "Time Unsigned Seconds", "Time Unsigned Seconds Fractions"; NTF0x0: "Network Time Protocol Version 3 (Request for Comment 1305)". Same text/body checked in 106-17/19/20/22/23/24; exact pages and older figure numbers in the analysis.
- RFC 1305 Appendix E.8, pp.78–79: "the NTP timescale is in effect reset to UTC as each timecode is received"; "knowledge of all previous leap seconds is lost"; 1972 value "2,272,060,800". Figure 8 duplicates the leap/midnight coordinate; its example-date inconsistency is flagged in the analysis.
- RFC 5905 §6, pp.12–15: "Eras cannot be produced by NTP directly"; "they can be derived from external means"; "0 h 1 January 1900 UTC"; "a value of zero is a special case representing unknown or unsynchronized time". Figure4 supplies the 1970/1972/1999/2036 fixtures; the pivot premise is "within 68 years of the server".

**Behaviour**: Resolve era from caller declaration, else a labelled caller-date/same-session dated-reference pivot, else era0 labelled policy-assumed; never use host time. NTP UTC civil/Unix-style seconds are era*2^32+raw_seconds-2208988800: never apply cumulative TAI-UTC. Raw seconds/fraction both zero yields unknown/unsynchronized, no absolute time and TF-029 WARNING, raw kept (owner 2026-11-13, plan Q1). Preserve raw fraction; floor-to-ns (round toward zero) is accepted owner 2026-11-13. With covered leap data, flag the duplicate insertion coordinate as TF-030 WARNING, leap-ambiguous, and keep the mapped value (owner 2026-11-13, plan Q1); without coverage record uncertainty. Format 2 carries no leap indicator and cannot uniquely label second60. PTP retains TAI and subtracts the applicable offset for UTC; outside table coverage use the last known offset, label the UTC answer possibly out of date and raise TF-021 (warning), per contract 6.8/6.9 and owner 2026-11-13. The contrary no-UTC instruction from the coordinator brief is withdrawn; it was not an owner decision.

**Reason**: The cited RFCs explain the Chapter 11 wording without double-counting leap offsets. Eras require external evidence or an explicit assumption, and the compact IRIG body loses native NTP leap indicators. Provenance and ambiguity must survive conversion.

**Design**: accepted · **Review**: pending · **Origin**: Q-NTP, owner 2026-11-12: "make this as correct as possible"

**Test**: not yet written for this new interpretation; future edge fixtures are enumerated in the analysis. Existing NeedsContext tests are not evidence for the new conversion behaviour.

**Development**: intensive testing and deep analysis required · **Analysis**: [written all-edition/RFC analysis and planned edge cases](analysis/RDG-012-ntp-utc-era.md)

**Rounding decision**: floor NTP fraction-to-ns conversion (round toward zero) accepted owner 2026-11-13; preserve the raw word and quantization basis. **Archive proposal**: RFC 1305/5905 are not in rcc-106-standards; propose archiving their publisher documents with hashes and provenance, but do not archive them in this task.
