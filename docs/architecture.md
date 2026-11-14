# irig106-time — Architecture

> **Status: proposal for the owner's review** (step 1 of the plan,
> `docs/TIME-IN-CHAPTER-10.md` section 8.2). Revised on 2026-09-27 against
> the contract document `docs/TIME-IN-CHAPTER-10.md`; it replaces the
> prototype's architecture (written for 0.1.0 on 2026-03-25, kept at the tag
> `prototype-0`). Decisions are in `docs/adr/` (ADR-0001 to ADR-0017); each
> section names the records it rests on. Rules cite the archived standard
> (`TelemetryWorks/rcc-106-standards`, 106-24R1 unless stated). Nothing here
> is code yet: type and module names are proposals, and signatures are left
> to L2 and L3.

## Contents

1. What the crate is for
2. Principles
3. The crate in the ecosystem
4. The layers and modules
5. The data model
6. How a recording is read
7. How a recording is produced
8. Errors and findings
9. Features, targets, and dependencies
10. The workspace and the CLI
11. From the prototype to the rebuild
12. Traceability and tests
13. Open points for the review

---

## 1. What the crate is for

Every Chapter 10 packet carries a relative time counter — "a free-running
10-MHz binary counter represented by 48 bits that are common to all data
channels" that "shall remain free-running during each session (e.g.,
recording)" (Chapter 11 §11.2.1.1 i). Absolute time arrives separately: in
time packets that pair it with a counter value at least once a second
(§11.2.3.2, §11.2.3.3), in secondary headers and intra-packet time stamps
(§11.2.1.2, §11.2.1.3 b), and in time words inside data (Chapter 4 §4.7).
The setup record declares where it is (`R-x\TTF-n`, `R-x\TFMT-n`,
`R-x\TSRC-n`, `R-x\SHTF-n`).

**`irig106-time` is the one place in the ecosystem where time is understood**
(contract section 1.2). It reads and writes the standard's time values, and
from a recording's time packets it builds a **time timeline** that turns any
counter value, secondary-header time, time stamp, or time word into absolute
time **with its basis** — which session, which time channel and reference,
how far away, from which source, with which year, under which policy.

What it does not do (contract section 1.3): open files or walk packets (the
packet reader, `irig106-core`); read TMATS (`irig106-tmats`); find time
stamps and time words in data bodies (`irig106-decode`); assemble packets
(`irig106-write`); present time (the tools).

---

## 2. Principles

| # | Principle | Record |
|---|-----------|--------|
| 1 | **Cite the standard.** Every rule traces to a section, figure, or table of 106-24R1; edition differences name both editions. | ADR-0004 |
| 2 | **A leaf library.** Depends only on `irig106-types`; everything else arrives as plain data. | ADR-0005 |
| 3 | **No I/O.** Bytes and values in, values out; the same input gives the same answer everywhere. | ADR-0006 |
| 4 | **An answer, not a time.** Every absolute time carries its basis. | ADR-0011 |
| 5 | **A policy, not hidden rules.** Every rule of time over a recording is a setting with a cited default, and every answer records the policy it used. | ADR-0010 |
| 6 | **Findings, not repairs.** Degraded time is reported with a stable identifier; nothing is silently corrected. | ADR-0013 |
| 7 | **Readings are reviewed.** Where TMATS and the packets name things differently, the reading is a register entry. | ADR-0012 |
| 8 | **Small and portable.** `no_std` with `alloc`; builds for WebAssembly. | ADR-0014 |
| 9 | **Tests from the standard.** Fixtures from the standard's figures; a failing test before each fix. | ADR-0016 |

---

## 3. The crate in the ecosystem

![Where time sits in Chapter 10 processing](diagrams/time-in-the-pipeline.svg)

*Two directions* (contract section 2). Reading: the packet reader gives
packets; `irig106-tmats` gives the time declarations of each setup record as
plain data; time packets become references; every other packet's counter,
secondary-header time, time stamps, and time words become absolute time for
the decoder and the tools. Producing: the crate encodes time packet data
words and bodies for `irig106-write`.

| Crate | Depends on | Does not depend on |
|-------|-----------|--------------------|
| `irig106-time` | `irig106-types` | `irig106-tmats`, `irig106-core`, `irig106-decode` |
| `irig106-time-cli` | `irig106-time` (`=X.Y.Z`), `irig106-types` | — |
| `irig106-decode` | `irig106-types`, `irig106-tmats`, `irig106-time` | — |
| `irig106-cli` | `irig106-time-cli` (and the other CLI libraries) | — |

![Which crate depends on which, as decided](diagrams/time-dependencies.svg)

*Decided dependencies only.* An arrow points to a crate depended on; the
tools on the right use this crate, and their other dependencies are not yet
decided. The legend lists the dependencies decided against.

What crosses each boundary is contract section 2.3 (A to H). Two of those
boundaries are new types in this crate: the **time declarations** (C), which
the caller fills from `irig106-tmats`'s plain data, and the **answer** (F),
which carries its basis.

---

## 4. The layers and modules

![The layers of irig106-time](diagrams/time-architecture.svg)

*Three layers.* Values (no allocation) read and write what the standard
defines, one field at a time; the recording layer (with `alloc`) builds the
time timeline and answers questions against it; the CLI library and its
binary sit on top. `irig106-types` holds the shared values underneath.

### 4.1 Layer 0 — `irig106-types` (shared values)

The counter (`Rtc`, 48 bits, 100 ns a tick), the ERTC (64 bits, **1 ns a
tick**, "RTC = ERTC/100", §11.2.1.1 g; T-5), the Chapter 4 and IEEE 1588
time values, the time sources and formats (with source 3 reserved, T-6, and
FMT `0xF` NONE, T-7), the unit and epoch newtypes, and the two edition code
lists — the packet header's data type version and the setup record's RCCVER
(T-9). Fixed there first (ADR-0015).

### 4.2 Layer 1 — values (no allocation)

| Module | Holds | Standard | Findings fixed |
|--------|-------|----------|----------------|
| `flags` | the packet flags' time bits: secondary header present (bit 7), time stamp source (bit 6), secondary-header time format (bits 3–2), RTC sync error (bit 5) | §11.2.1.1 g | T-1 |
| `secondary` | the 12-byte secondary header: 8 bytes of time in the format of bits 3–2 (Figures 11-4 to 11-6), 2 reserved bytes, the checksum and its check | §11.2.1.2 | T-10 (layout visually checked, 2026-11-12) |
| `stamp` | an 8-byte intra-packet time stamp: the 48-bit counter "plus 16 high-order zero bits", or the 64-bit secondary-header format when bit 6 is set | §11.2.1.3 b | T-1 |
| `format1` | the Format 1 data word (SRC, FMT, leap year, date format, ITS, reserved bits) and bodies: day format, 3 words (6 bytes); day, month, and year, 4 words (8 bytes); decode and encode | §11.2.3.2, Figures 11-12 to 11-14, Table 11-16 | T-6, T-7, T-12 |
| `format2` | the Format 2 data word (NTF bits 7–4, TS bits 3–0, reserved 31–8) and bodies: NTP seconds and fraction, PTP seconds and nanoseconds, 8 bytes each; decode and encode | §11.2.3.3, Figures 11-15 to 11-17 | T-2, T-3 |
| `words` | time words from data: Chapter 4 high/low/microseconds with explicit caller binary origin or BCD year/full day, otherwise labelled partial (no nearest-packet inference); network words use RDG-012 | Chapter 4 §4.7; contract 3.8 | — (new) |
| `calendar` | absolute time: day of year and time of day to the nanosecond, with an optional year; calendar dates; leap years; the leap second 23:59:60 | Figures 11-13, 11-14; RCC 200-16 §3.6, A.2 | T-16 |
| `scale` | time scales and epochs: UTC (FMT `0x4`, NTP from 1900, "includes leap seconds"), TAI (PTP from 1970, "does not include leap seconds"), GPS (FMT `0x5`, TAI − 19 s), unknown (FMT `0x3`), and IRIG-A, B, G as UTC by assumption; NTP UTC/Unix reckoning and labelled era evidence under RDG-012, without cumulative TAI-UTC; PTP UTC with last-known-offset fallback outside coverage, possibly out of date with TF-021; raw TAI retained; fraction floor accepted owner 2026-11-13 | §11.2.3.2 a, §11.2.3.3; RCC 200-16 §1, A.2; RFC 1305 Appendix E.8 pp.78–79, RFC 5905 §6 pp.12–15; RDG-009/012 | T-17 |
| `edition` | what exists in which edition: Format 2 from 106-17; Chapter 10 section numbers before 106-17, Chapter 11 after | §11.2.1.1 e | T-4 |

Every reader in this layer takes a byte slice bounded by the caller —
never more than Data Length (§11.2.1.1 d; T-12) — checks reserved bits
("All reserved bit fields in packet headers or CSDWs shall be set to zero",
§11.2.1 f), and reports what it cannot read as an error (section 8).

### 4.3 Layer 2 — the recording (with `alloc`)

| Module | Holds | Contract |
|--------|-------|----------|
| `declared` | **time declarations**: for each setup record, the counter range it governs and its time channels (`R-x\TTF-n`, `R-x\TFMT-n`, `R-x\TSRC-n`), each channel's `R-x\SHTF-n`, and the original recording date (`R-x\RI4`) — plain data the caller fills from `irig106-tmats` | 2.3 C, 4.5 |
| `reference` | a **reference**: one time packet as a point — its session, channel, counter, time, scale, source, format, ITS, validity, and the packet it came from | 3.2, 3.3 |
| `policy` | the **time policy**: every setting of contract section 5.6, with its cited default | 5.6; ADR-0010 |
| `timeline` | the **time timeline** and its builder: sessions, time channels and the one chosen per session, references, source changes, gaps, jumps, late packets, the year, the quality measures, and the findings | 5.2 to 5.5 |
| `answer` | the **answer**: an absolute time with its basis; the lookups — time for a counter, a secondary-header time, a time stamp, or a set of time words; the counter range for a span of absolute time | 3.1, 3.4, 3.8, 5.5 |
| `stream` | the same rules with bounded memory, answering as packets pass | 4.8 |
| `readings` | the reading register in code: TMATS letters against packet values, one entry per row of contract 3.7, each naming its register entry | 3.7; ADR-0012 |
| `findings` | findings: stable identifier, default severity, evidence | 6; ADR-0013 |
| `quality` | measures of the references: counts, spacing, drift per channel | 3.5 |
| `error` | errors for input that cannot be read at all | 8 |

### 4.4 Layer 3 — `irig106-time-cli`

A library — `args` (hand-rolled, ADR-0008), `input` (the packet reader until
`irig106-core` exists), `commands`, `report` (a model independent of the
output format), `render` (text, CSV, JSON), `run` — and the binary
`irigtime`; `irig106-cli` mounts `run` as `irig106 time` (ADR-0007). It
writes the joining loop of section 6 once, for every command.

Q-CLI-EXIT (owner 2026-11-12) adopts the shared ecosystem convention so
`irig106 tmats` and `irig106 time` agree: **0 success; 1 findings at or above
the chosen severity; 2 usage error; 3 unreadable or invalid Chapter 10/time
input; 4 output could not be written**. Never renumber. IRIG 106 is silent
on tool exit codes; no standard source exists. The pinned TMATS draft,
L1-CLI-007 → L2-CLI-107 → L3-CLI-107, and planned restructure-step tests
are recorded in [the Q-CLI-EXIT analysis](analysis/Q-CLI-EXIT-2026-11-12.md).
Owner 2026-11-13, plan Q6: usage is validated before input; the first fatal
operational failure determines 3 (input) or 4 (output), and later failures
become diagnostics. This does not renumber the classes.

---

## 5. The data model

![What an answer carries, and where it comes from](diagrams/time-answer.svg)

*The answer and the timeline.* A time timeline holds sessions; each session
has its time channels, the one chosen and why, and each channel's
references. An answer is the absolute time with its basis, drawn from one
reference of the chosen channel, and it records the policy.

### 5.1 The time timeline

| Part | Holds |
|------|-------|
| **Session** | a run of packets whose counter increases, allowing for late packets (default bound 1100 ms); its counter range; its span in absolute time; its chosen time channel and the rule that chose it; its year and where the year came from |
| **Time channel** (per session) | the channel ID; what the governing setup record declares for it (or that it is undeclared); its references; its source changes; its gaps and jumps |
| **Reference** | counter, absolute time, time scale (UTC, TAI, GPS, or unknown; assumed or stated), precision (the resolution of its format: 10 ms for Format 1), source, format, ITS, validity, and the packet it came from; whether it is used, and why not if not |
| **Governing setup record** | per counter range, which time declarations apply (from the caller, after `irig106-tmats` L1-CH10-008) |
| **Findings** | every finding of the recording, each with its evidence |
| **Quality** | reference counts, largest and smallest spacing, references per second, drift in parts per million |
| **Policy** | the policy the timeline was built with |

### 5.2 The answer

| Field | Meaning |
|-------|---------|
| **time** | the absolute time: day of year and time of day to the nanosecond, with the year when known |
| **session** | which session the counter lies in |
| **time channel** | which channel's references were used, and the rule that chose it (the caller, the declared external channel, the most valid references, the lowest ID) |
| **reference** | the reference used: its counter and time |
| **distance** | the counter difference to it, as a duration |
| **position** | between two references; before the first (extrapolated, by policy); beyond the last |
| **source** | the reference's source, format, and ITS (external IRIG-B locked; internal freewheeling; PTP valid) |
| **scale** | the time scale of the answer — UTC, TAI, GPS, or unknown — and whether it is stated by the standard or assumed by the policy |
| **precision** | the resolution of the reference's format (10 ms for Format 1; 1 ns for PTP), so a time is not read as more precise than its reference |
| **year** | known or not, and from where: a day-month-year packet, the caller, `R-x\RI4` |
| **corrections** | drift correction applied or not; PTP offset and table coverage/currency; NTP leap ambiguity (never a cumulative offset correction) |
| **era (NTP)** | raw seconds/fraction; declared, pivot-derived or era0-policy-assumed era; pivot/reference identity, rounding and leap-ambiguity findings (RDG-012) |
| **own time** | for a packet with a secondary header: its own time, and its difference from the counter-derived time when beyond the policy's tolerance |
| **policy** | the policy used, or the settings that differ from the defaults |

The worked example (contract section 7.3) is an answer: day 187
13:45:27.770; session 1; time channel 1, the declared external channel;
reference r1 = 100,000,000 at 13:45:27.350; distance 0.420 s; between two
references; external, IRIG-B, ITS "locked to external IRIG time signal";
UTC, assumed; precision 10 ms; year unknown; defaults.

### 5.3 The policy

The settings and defaults are contract section 5.6: session boundaries;
which time channels count (declared TIMEIN); which time counts (valid only);
choosing the **offline** time channel (caller, declared external, most valid
references, lowest ID); offline reference nearest; stream default Standard
(preceding reference) or explicit Interpolate, with channel commitment
described below; after a lost lock (keep, labelled); the year (day-month-
year packet, caller, `R-x\RI4`); reference gap (more than 1 s, default gap
tolerance 0 ns, independent of the hold tolerance); time jump (the format's resolution, 10 ms for Format 1); the
late-packet bound (1100 ms = the 1000 ms stream commit time plus the 100 ms
packet generation time of Chapter 10 §10.6.1 b–c); counter wrap (arithmetic,
fixed); before the first reference (counter time only); secondary header
against the counter (report beyond the reference's resolution); the time
scale of IRIG-A, B, G (UTC, assumed) and of the real-time clock (unknown);
and the severity of each finding (its default, section 8).

---

## 6. How a recording is read

![Reading a recording, step by step](diagrams/time-reading-flow.svg)

*The worked example, as a sequence.* While reading, the caller hands this
crate the time declarations, each time packet, and every other packet's
channel, counter, and flags; at the end it receives the time timeline, and
any consumer asks it for answers.

The caller walks the packets in file order — the joining loop of contract
section 4.2 — and feeds the builder three kinds of input.

1. **A setup record** (from `irig106-tmats`, as plain data): its time
   declarations and the counter from which it governs. The declarations
   decide which time channels count (policy: declared TIMEIN).
2. **A time packet** (`0x11`, `0x12`, Table 11-4): its channel, data type,
   counter, data word, and body — bounded by Data Length. Layer 1 reads it;
   the timeline records a reference, or a finding and no reference (invalid
   time, an undeclared channel, malformed bytes).
3. **Every other packet's channel, counter, and flags**: the counter going
   backwards beyond the late-packet bound starts a new session; a late packet
   within the bound is noted; flags bit 5 ("RTC sync error has occurred",
   §11.2.1.1 g) is a finding.

![Which reference governs a packet](diagrams/reference-selection.svg)

*Offline: which reference governs a packet* (contract section 5.2): sessions first,
declared time channels, valid time, one channel per session, then the
nearest reference — each rule a setting of the policy.

At the end the builder gives the time timeline: sessions, channel choice,
year, gaps, jumps, findings. Lookups against it give answers: for a counter
(any packet), for a secondary-header time (checksum first), for a time stamp
(bit 6 decides whether it is a counter or the secondary-header format), for
a set of time words (from `irig106-decode`).

**One pass or two.** Offline sees all references and retains nearest/count
ranking. Stream defaults to **standard**, answering immediately from the
preceding reference; **interpolate** remains explicitly selectable (owner,
2026-11-12; RCC 123-20 §5.6, printed 5-44, "clock time of a subsequent data
packet"; cited defaults under ADR-0010). In interpolate, hold is the 1 s
reference interval plus 100 ms policy tolerance, with explicit caller progress
and caps. At cap/timeout/EOF release as extrapolated from a usable preceding
reference, or counter-only without one. A gap>1 s is a finding even inside
that 1.1 s allowance. Chapter 10 §10.6.2 says "minimum frequency of 1 hertz";
§10.6.1b says "packet generation times shall be equal to or less than 100
milliseconds"; §10.6.1c says "stream commit time equal to or less than 1000 ms"
(all printed 10-26/PDF32). The commit allowance explains late-written time
packets; the tolerance is selected reader policy, not an IRIG reader mandate.
No hidden unbounded queue or host-clock progress is allowed.

**Stream channel commitment (Option A).** Before data choose caller-fixed,
else declared external TIMEIN (`R-x\CDT-n=TIMEIN`, `R-x\TSRC-n=E`, Table 9-4,
printed 9-50/9-72), else lowest eligible ID; lowest ID breaks equal preference
ties. If the inventory/declarations cannot settle the choice in advance,
return counter-only until the caller chooses. Do not use the offline most-valid-
references step, change channel on lost lock, or revise emitted answers.
RCC 123-20 §5.6 says "it is incumbent on the programmer or data analyst to
determine and select the best source of time"; §11.2.3.2's note says "the time
source shall indicate Internal (0x0)" on lost lock. Those source changes stay
labelled on the committed channel. This ranking is owner policy, not a
ranking prescribed by Table 9-4.

**Calendar/era reconstruction.** Chapter 4 requires explicit caller binary
origin or BCD year/full day; otherwise return labelled partial time. Never
invent day 1/year/origin or use a nearest-time-packet option now. Its §4.7
phrases "resolution of 655.36 seconds", "10 milliseconds", "three least
significant bits of the BCD Julian date" and "at a rate necessary to resolve
time ambiguities" (printed 4-11/PDF15) establish weights and ambiguity, not an
origin; RCC 123-20 `enI106_Ch4Binary2IrigTime` applies no origin (B-43/PDF149).
Older characters/current bits wording stays unresolved and reported.

NTP follows UTC reckoning under **RDG-012**: extend seconds by declared era,
else caller-date/same-session independently dated-reference pivot, else labelled
era0 policy assumption; subtract2208988800, never cumulative TAI-UTC. Never
use the host clock. All-zero seconds/fraction is unknown/unsynchronized, no
absolute answer. Format 2 lacks NTP leap bits: retain mapped values with
leap-ambiguous findings at covered insertion coordinates, and record uncertain
coverage when the table does not reach the date. NTP fraction floor rounding
is accepted (floor/round toward zero, owner 2026-11-13). PTP retains TAI;
UTC uses the applicable offset, or the last known offset outside coverage
with a possibly-out-of-date label and TF-021 warning. The contrary no-UTC
coordinator instruction is withdrawn. See
[the RFC/all-edition analysis](analysis/RDG-012-ntp-utc-era.md).

---

## 7. How a recording is produced

`format1` and `format2` encode what they decode: the data word and the body,
byte for byte as Figures 11-12 to 11-17 lay them out, with reserved bits
zero. `irig106-write` packs them into packets and keeps the rules the crate
cannot keep for it: a time packet first after the setup record, at least
once a second, the counter free-running through the session, and "RTC =
ERTC/100" when writing ERTC (contract section 4.11).

---

## 8. Errors and findings

| | Error | Finding |
|---|-------|---------|
| **When** | the input cannot be read at all | the input can be read, and something about time is wrong, missing, weak, or in disagreement |
| **Examples** | a buffer shorter than the figure's length; a BCD digit above 9 | no time packets in a session; a packet before the first reference; FMT `0xF` NONE; time status "Time Not Valid"; reserved bits set; an undeclared time channel; TMATS and the packets differ; a failed secondary-header checksum; mixed secondary-header formats; a gap, jump, reset, or late packet; no year; the leap-year bit disagrees; a stale leap-second table; RTC sync error |
| **Effect** | the value is not produced | the answer is given as the policy says, labelled |
| **Source** | Layer 1 | Layers 1 and 2, gathered in the timeline |

Every finding has a stable identifier, a default severity the caller can
change, and its evidence: the packet (file position when the caller gives
it), the channel, the counter. Identifiers are never reused. Nothing panics
on any input (L1-ERR-001).

**The findings** (owner-accepted identifiers; default levels accepted except
TF-032 (level pending owner in S08); TF-031 WARNING is owner-accepted
2026-11-14). *Error*: the value cannot be trusted and is not used; *warning*: the
recording breaks a rule of the standard; *information*: worth knowing,
nothing is wrong.

| ID | Finding | Default | Contract |
|----|---------|---------|----------|
| TF-001 | A session has no time packet | warning | 6.1 |
| TF-002 | A dynamic packet precedes the session's first time packet | warning | 6.2 |
| TF-003 | A time packet is marked invalid (FMT `0xF` NONE; TS "Time Not Valid") | warning | 6.3 |
| TF-004 | A time packet is malformed (a digit above 9, a field out of range, a short body) | error | 6.3 |
| TF-005 | Reserved bits are set (data word, body, secondary header) | warning | 6.3 |
| TF-006 | A reserved value is used (SRC `0x3`–`0xE`, FMT `0x6`–`0xE`, NTF `0x3`–`0xF`, TS `0x2`–`0xF`, ITS `1000`–`1111`, flags bits 3–2 `11`) | warning | 6.3, 6.6 |
| TF-007 | The leap-year bit disagrees with the day or the year | warning | 6.3, 6.8 |
| TF-008 | Time packets on a channel the setup record does not declare TIMEIN | warning | 6.4 |
| TF-009 | A declared time channel carries no time packets | information | 6.4 |
| TF-010 | TMATS and the packets differ (`TTF`, `TFMT`, `SHTF`) | warning | 6.4 |
| TF-011 | A channel's time source differs from `TSRC`, or changes (lock lost or regained) | information | 5.2, 6.4 |
| TF-012 | A secondary header fails its checksum | error | 6.6 |
| TF-013 | Channels use different secondary-header time formats | warning | 6.6 |
| TF-014 | A secondary header's time and the counter's differ beyond the tolerance | warning | 6.6 |
| TF-015 | Packet flags bit 6 is set without bit 7 | error | 6.6 |
| TF-016 | A reference gap | warning | 5.4, 6.7 |
| TF-017 | A time jump | warning | 5.4, 6.7 |
| TF-018 | A counter reset: a new session | information | 5.4, 6.7 |
| TF-019 | A packet later than the late-packet bound | warning | 5.4, 6.7 |
| TF-020 | A session's year is unknown | information | 5.3, 6.8 |
| TF-021 | A conversion falls after the leap-second table's last known date | warning | 6.8 |
| TF-022 | A leap second (23:59:60) was read | information | 3.2, 6.8 |
| TF-023 | A time scale is assumed or unknown | information | 3.2, 6.8 |
| TF-024 | Packet flags report an RTC sync error (bit 5) | warning | 6.7 |
| TF-025 | A packet's ERTC and RTC disagree ("RTC = ERTC/100") | warning | 6.6 |
| TF-026 | A Format 2 time packet in a recording declared before 106-17 | warning | 3.9 |
| TF-027 | Secondary headers satisfy only the other checksum reading (RDG-008) | information | 6.6 |
| TF-028 | Secondary-derived time has an undefined format | error | 6.6; §11.2.1.1 g |
| TF-029 | NTP time is zero (unknown or unsynchronized) | warning | 6.3; RFC 5905 §6 |
| TF-030 | NTP time falls in a leap-second ambiguity window | warning | 6.8; RFC 1305 E.8; RDG-012/RDG-010 |
| TF-031 | Packet-layout edition evidence is contradictory | warning | 3.9; §11.2.1.1 e; Ch9 G\106 |
| TF-032 | Year evidence is contradictory | level pending owner (S08) | 5.3; Figures 11-13/14 |

Owner 2026-11-13, plan Q1: TF-029 retains raw seconds/fraction but produces
no absolute value; TF-030 keeps the mapped value labelled leap-ambiguous.
RFC 5905 §6, p.14: "a value of zero is a special case representing unknown or
unsynchronized time". RFC 1305 Appendix E.8, pp.78–79: "the NTP timescale is
in effect reset to UTC as each timecode is received"; Figure 8 shows the
repeated insertion coordinate (RDG-012/RDG-010).

TF-031 is the existing no-override contradiction case, not a new conflict
rule: an explicit override wins and retains contrary evidence side by side;
differing declarations alone and lower permitted header versions are not
conflicts. Source: §11.2.1.1 e, printed 11-6, "a value at or below the release
version of the standard applied to the data types"; 106-04 Table 9-1, printed
9-8, G\106 is "VERSION OF IRIG 106 STANDARD USED TO GENERATE THIS TMATS FILE",
not necessarily the packet layout. TF-032 retains contradictory year evidence;
Figures 11-13/14 and contract 5.3 distinguish year-bearing and day-only evidence.
Owner 2026-11-14 accepts TF-031 WARNING: the readable time body is kept;
only EXT/SRC qualification is withheld. TF-032
remains without a default pending S08. IDs/titles are permanent. TF-001..028
are unchanged. The standards supply evidence, not these product finding IDs.
See [accepted plan decisions](analysis/plan-decisions-2026-11-13.md).


TF-028 source: 106-24R1 §11.2.1.1 g, printed 11-8/11-9:
"Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved";
"1 = Packet secondary header time (bit 7 must be 1)." Owner 2026-11-13
approves the error-level addition; TF-001..TF-027 are unchanged.

Q-FLAGS (owner 2026-11-12, refine): bit 6=1/bit 7=0 is an error with no
stamp time. Bit 6=1/format 11 is undefined-format error with no stamp time.
Bit 6=0/format 11 retains the padded RTC value with TF-006 warning; if bit 7=1,
the secondary header time is unreadable and reported separately, without
discarding the RTC stamp. These conditions are scoped to the affected value,
not a reason to discard every time in the packet. Verbatim sources and
page checks: [Q-FLAGS analysis](analysis/Q-FLAGS-2026-11-12.md);
L1-IPT-007 → L2-IPT-107 → L3-IPT-107.

**Accepted Q-FLAGS-ERROR, owner 2026-11-13:** TF-028 "Secondary-derived time has an undefined format" is ERROR and maps from UndefinedSecondaryTimeFormat. It reports selected or present secondary time with format 11; RTC stamps remain usable. TF-006 remains the separate reserved-VALUE warning, TF-005 reserved-BITS warning, and TF-015 missing-required-header error. With 0x4C, TF-015/TF-028/TF-006 evidence can coexist. TF-001..TF-027 are unchanged; the former numeric finding-assignment open item is resolved.

Weak time (contract 6.5) is not a finding: every answer's basis carries
it.

---

## 9. Features, targets, and dependencies

| | |
|---|---|
| **Dependencies** | `irig106-types` (required); `serde` and `chrono` (optional features) |
| **Features** | `std` (default: `std::error::Error`); `alloc` implied by the recording layer; `serde`; `chrono` (conversions to and from `chrono` types) |
| **Targets** | any Rust target; CI builds `wasm32-unknown-unknown` with and without `serde`, and a `no_std` target without `std` (ADR-0014) |
| **Edition and MSRV** | edition 2024, Rust 1.85 (ADR-0003) |
| **`serde`** | Versioned snapshot/report JSON with units; 64/128-bit quantities as decimal strings; validation on supported loads; raw/stated/assumed provenance. Review schema once before freeze. For 0.8.0 only complex timeline-state rehydration is deferred: omit/disable unsupported complex `Deserialize`; existing value validation and complete versioned outputs remain required (API-110, POL-103; owner 2026-11-13 Q3/Q5). |
| **`chrono`** | conversions between the crate's absolute times and `chrono`'s date-time types, for times whose year and scale are known |
| **Unsafe code** | none (`#![forbid(unsafe_code)]`) |
| **Lints** | Clippy's `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`, `print_stdout`, `print_stderr` denied in library code (L1-API-012) |
| **Dependencies** | `deny.toml`, checked by `cargo deny` in CI (L1-REL-005) |
| **Platforms** | tests on Windows, Linux, and macOS in CI (L1-REL-004) |
| **Documentation** | every public item documented (`#![deny(missing_docs)]`), citing the standard; `cargo doc` in CI |
| **Performance** | benchmarks for the hot paths — reading a time packet, answering for a counter, building a time timeline — with the budget set after the first measurement (as `irig106-tmats` L1-PERF-001 does); the prototype's measurements (`docs/benchmark_results.md`) are the baseline to beat |

---

## 10. The workspace and the CLI

One workspace, two crates, one version (ADR-0007): `irig106-time` and
`irig106-time-cli`, published together; the CLI pins the library with
`=X.Y.Z`. The layout follows `irig106-tmats`'s decision on W1 to W3 (a
virtual workspace with `crates/`), so the repositories look alike. The CLI's
commands are `summary`, `channels`, `jumps`, `timeline`, `csv`, and
`correlate`, each presenting what the library answers, with the basis and
the findings; network time is reported as itself, never as GPS (ROADMAP
P6-10).

---

## 11. From the prototype to the rebuild

The prototype (`prototype-0`) is not patched (ADR-0001); what it got right is
carried over through tests written from the standard (ADR-0016).

| Prototype module | In the rebuild | Findings |
|------------------|----------------|----------|
| `rtc` (from `irig106-types`) | Layer 0, unchanged | — |
| `absolute` | `calendar`; Chapter 4 binary time to `secondary` and `words` after the recorded page check; day 0 reported, not turned into day 1; the leap second accepted | T-10, T-16 |
| `bcd` | `format1` bodies, 6 and 8 bytes; digit and range checks kept | T-12 |
| `csdw` | `format1` data word, with ITS, NONE, reserved bits, source 3 reserved | T-6, T-7 |
| `network_time` | `format2` (data word NTF and TS; 8-byte PTP body; from 106-17) and `scale` (UTC, TAI, GPS, unknown; leap-second table, with its currency) | T-2, T-3, T-4, T-17 |
| `secondary` | `secondary`; mixed formats reported | T-10 |
| `intra_packet` | `flags` and `stamp`, bit 6 and bits 3–2 | T-1 |
| `correlation` | `timeline` and `answer`: the nearest-reference core kept, with sessions, channel choice, policy, basis, findings; late packets bounded at 1100 ms by default (the prototype uses 2 s) | — |
| `streaming` | `stream`, same policy and basis | — |
| `quality` | `quality`, feeding the basis | — |
| `version` | the mappings move to `irig106-types`; `0x0F` unknown; `edition` keeps what changes for time | T-9 |
| `packet_standard` | `edition` | — |
| `recording_event` | leaves the crate, for `irig106-decode` (ADR-0009) | T-8 |
| `error`, `util`, `chrono_interop` | `error`; `util` as needed; `chrono` feature | — |
| `irig106-time-cli/src/main.rs` | `irig106-time-cli` library and `irigtime` binary | P6-10 |

---

## 12. Traceability and tests

![Traceability for the rebuild](diagrams/time-traceability.svg)

*Four sources, one chain.* The standard, the contract, the decisions, and
the findings feed L1; L2 and L3 follow the review; the code implements L3
and the tests verify it; real recordings, used only locally, feed new
findings back.

Each L1 requirement cites the standard or an ADR; L2 and L3 name their
parents; each test names the requirements it verifies with a
`/// Requirements: L?-XXX-NNN` doc comment above `#[test]`, and each reading
it pins with `/// Interpretations: RDG-NNN`. `scripts/build-trace-matrix.py`
(adapted from `irig106-tmats`) generates `docs/TRACE-MATRIX.md` from the
requirement documents, the reading register (`docs/INTERPRETATIONS.md`), and
those markers; a CI job fails when the matrix is out of date. A requirement
verified by Inspection, Analysis, or Demonstration names its artifact on an
**Evidence** line, or it stays Draft. The prototype's
requirement identifiers that survive keep their numbers; those that were
wrong are retired and never reused (L1, section "Prototype requirements").

---

## 13. Open points for the review

1. ~~ADR-0011, 0012, 0013, 0014, 0016, 0017~~ — **accepted** (2026-09-27).
   0.1.0 to 0.7.0 are not yanked (owner, 2026-09-27).
2. ~~The finding identifiers and default severities of section 8
   (TF-001 to TF-027)~~ — **accepted** (owner, 2026-11-11).
3. ~~RTC sync error as a finding~~ — now in the contract (section 6.7) and
   TF-024.
4. ~~**T-10**, the Chapter 4 binary layout~~ — page images checked
   2026-11-12: Figure 11-4 (printed 11-10 / PDF 18) and Figure 4-4
   (printed 4-10 / PDF 14). See `STANDARD-REVIEW.md` T-10. Calendar origin
   and reconstruction require explicit caller context (owner 2026-11-12);
   missing context means labelled partial, no invented day/year/origin or
   nearest-packet option. Historical characters/bits wording remains unresolved.
5. ~~The one-pass answer (section 6)~~ — **both, a policy setting** (owner,
   2026-11-11): `standard` answers at once from the preceding reference;
   `interpolate` holds until the next reference and corrects for drift,
   bounded by the 1 Hz time-packet rate; see `docs/ROADMAP.md` question 2.
   Owner2026-11-12 accepted Standard as default, 100 ms hold/0 ns gap
   tolerances, explicit progress/caps and Option A channel commitment; see
   L2/L3 for accepted behaviour; interpolation floor, preceding midpoint tie,
   coarser mixed-resolution jump threshold and old-policy drain are accepted
   owner 2026-11-13 (plan C1–C4).
6. **The reading register** (`docs/INTERPRETATIONS.md`, RDG-001 to RDG-012):
   prior Design states remain unchanged; RDG-012 is accepted (owner 2026-11-12),
   with written RFC/all-edition analysis and floor rounding accepted, owner 2026-11-13.
   Two-person review remains pending.
   RDG-003 (`TFMT` I "Internal") is accepted as an inference;
   RDG-007 (ITS before 106-17, T-14) and RDG-008 (the checksum, T-15,
   suspect) change what L2 specifies; RDG-009 (time scales, T-17) and
   RDG-010 (the leap second, T-16) were added at the completeness check.
7. ~~**Page checks** before L2~~ — completed visually, 2026-11-12:
   Figures 11-4 and 4-4 confirm the word layout and weights (T-10);
   Table 9-4, printed 9-72 / PDF 78, places "Default: A" within
   `R-x\TFMT-n` (RDG-002). The accepted missing-value rule and all Design
   and Review fields are unchanged. These checks do not complete the
   register's all-edition analysis or human review.
