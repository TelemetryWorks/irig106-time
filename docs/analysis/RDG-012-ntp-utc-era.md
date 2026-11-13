# RDG-012 — NTP UTC reckoning, eras and leap ambiguity

Owner direction, 2026-11-12: **“make this as correct as possible”**. This is
written requirements analysis, not an implementation change or human Review.
RDG-012 Design is accepted; fractional floor rounding is accepted by the owner, 2026-11-13. Existing Review fields and other Design decisions are unchanged.

## Source and page checks

The archived chapter PDFs were fetched/verified using rcc-106-standards's
`scripts/fetch.py`, rendered with `pdftoppm`, and inspected visually on
2026-11-13. Text extraction was used to find pages, not as the only check.
Printed / one-based PDF page:

| Edition | §11.2.3.3 scale paragraph | NTF and NTP body | Outcome |
|---|---|---|---|
| 106-17 | 11-23 / 31 | 11-24 / 32, Figures 11-14/11-15 | Same NTP/PTP text and NTPv3, two 32-bit words |
| 106-19 | 11-25 / 33 | 11-26 / 34, Figures 11-15/11-16 | Same |
| 106-20 | 11-25 / 33 | 11-26 / 34, Figures 11-15/11-16 | Same; actual 106-20 page headers checked |
| 106-22 | 11-26 / 34 | 11-27 / 35, Figures 11-15/11-16 | Same |
| 106-23 | 11-26 / 34 | 11-27 / 35, Figures 11-15/11-16 | Same |
| 106-24 | 11-26 / 34 | 11-27 / 35, Figures 11-15/11-16 | Same |
| 106-24R1 | 11-26 / 34 | 11-27 / 35, Figures 11-15/11-16 | Baseline |

Verbatim baseline text, §11.2.3.3, printed 11-26:

> The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds.

> The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds.

Figure 11-16 labels its two 32-bit words **“Time Unsigned Seconds”** and
**“Time Unsigned Seconds Fractions”**. NTF 0x0 is **“Network Time Protocol
Version 3 (Request for Comment 1305)”** (the printed superscript footnote
marker is not part of the protocol name). There is no NTP leap-indicator
field in this body or the NTF/TS data word. The older 106-17 figure numbers
are different, as recorded above. These checks do not validate recordings.

RFCs are external primary sources, **not documents in the
rcc-106-standards archive**:

- [RFC 1305](https://www.rfc-editor.org/rfc/rfc1305.pdf), Appendix E.8,
  **“The NTP Timescale and Reckoning with UTC”**, printed pp.78–79 /
  PDF85–86. Its RFC Editor ASCII rendering explicitly lacks the original
  pagination; the publisher PDF was used for page citations and visually
  checked. At the 1972 epoch it says **“the NTP clock was set to
  2,272,060,800”** (p.78). It explains:

  > the NTP timescale is in effect reset to UTC as each timecode is received. Thus, when a leap second is inserted in UTC and subsequently in NTP, knowledge of all previous leap seconds is lost.

- [RFC 5905](https://www.rfc-editor.org/rfc/rfc5905.html#section-6), §6,
  pp.12–15. The prime epoch is **“0 h 1 January 1900 UTC”** (p.13).
  On eras (p.12):

  > Eras cannot be produced by NTP directly, nor is there need to do so. When necessary, they can be derived from external means, such as the filesystem or dedicated hardware.

  On the timestamp sentinel (p.14):

  > a value of zero is a special case representing unknown or unsynchronized time

  On the half-era premise (p.15):

  > if the client is set within 68 years of the server before the protocol is started, correct values are obtained even if the client and server are in adjacent eras.

  Figure 4, p.14, gives era0 **1 Jan 1970 = 2,208,988,800**, era0
  **1 Jan 1972 = 2,272,060,800**, era0 **31 Dec 1999 = 3,155,587,200**,
  and era1 **8 Feb 2036 = offset 63,104**. Section 6 p.15 gives
  **“s = era * 2^(32) + timestamp.”** NTPv4 era discussion is used as a
  documented interpretation of the same 32-bit timestamp words; the IRIG
  NTF value remains NTP **version 3**, not silently upgraded to version 4.

**Archive proposal, not performed:** add publisher RFC 1305 PDF/ASCII and
RFC 5905 PDF/ASCII to rcc-106-standards with source URLs, retrieval date,
size/hash verification and pagination notes. No RFC was added to that repo,
manifest or release by this task; temporary copies were used for inspection.

## Interpretation and conversion

“Includes leap seconds” is not an instruction to accumulate leap offsets
on top of an NTP count. For ordinary UTC civil reckoning the count already
follows the broadcast UTC numbering; RFC 1305 says historical leap knowledge
is lost. With resolved era e, raw seconds s and fraction f:

    extended_ntp_seconds = e * 4294967296 + s
    unix_style_seconds = extended_ntp_seconds - 2208988800

Use checked signed wide arithmetic, including pre-1970 answers, and report
range limitations without wrapping. This Unix-style civil coordinate does
not count inserted leap seconds. **Never add or subtract cumulative TAI−UTC
for NTP**: applying 37 s after the 2016 insertion is a second correction,
not increased accuracy. The same elapsed-coordinate subtraction cannot
measure physical intervals across a leap insertion without extra evidence.

RFC 5905 p.13 notes that UTC did not exist before 1972; its proleptic epoch
convention still supplies Figure 4's 1970 and earlier dates. This analysis
uses that convention, not a claim to reconstruct pre-1972 physical UTC.

## Era evidence and basis

Apply the owner's precedence:

1. Caller-declared era, labelled **declared**, including its input identity.
2. Else the RFC 5905 half-era pivot using a caller-supplied approximate date
   or an independently dated absolute reference in the **same session**,
   labelled **pivot-derived** with pivot value, scale/year, reference ID and
   session. A Format 1 packet with year can supply this; an undated DOY or
   unresolved Chapter 4 partial time cannot. Retain any scale assumption
   needed to use the pivot. Never bootstrap a prior era-0-assumed NTP result
   into supposedly independent dated evidence.
3. Else era0, explicitly labelled **policy-assumed**, not known from the
   packet or the host clock. It is correct for instants in era0's interval
   **1900-01-01 to just before 2036-02-07 06:28:16 UTC**. “Before 2036” must
   not be generalized to centuries before 1900, which require negative eras.

**Derived pivot procedure:** choose the unique candidate congruent to s
modulo 2^32 nearest the supplied pivot within a strict 2^31-second half-era
window (the RFC's approximately 68-year premise). An exact half-era tie,
conflicting supplied pivots, or a pivot not establishing a unique candidate
requires explicit context resolution, not an invented century or silent
fallback that claims evidence. With no supplied pivot, the accepted labelled
era0 fallback applies. Never inspect the execution environment's time.
The pivot provides era, not a new source of timestamp seconds/fraction.

## Unknown timestamp and accepted fraction rounding

Check raw **s=0 and f=0 before rounding or era conversion**. This is the
unknown/unsynchronized sentinel: retain raw evidence, emit a finding, and
produce no absolute time even if an era declaration was supplied. It also
coincides numerically with an era boundary; the accepted sentinel rule takes
precedence. Raw s=0, f!=0 is not the sentinel, even if rounding produces 0 ns.
PTP zero is not this NTP sentinel; its stated TAI epoch remains a valid raw
answer; UTC conversion records table currency and uses the last known offset
outside coverage, labelled possibly out of date with TF-021 warning.

**Accepted owner 2026-11-13:** floor (round toward zero for this unsigned
fraction)/truncate
`(u64(f) * 1000000000) / 4294967296`. It is deterministic, loses less than
1 ns, never produces a 1e9-ns carry, and preserves the original fraction for
lossless wire reproduction. Expected boundaries: 0 -> 0 ns,
0x80000000 -> 500000000 ns, 0xFFFFFFFF -> 999999999 ns. Mark rounding and
quantization in the basis. The rounding decision is now explicit; no
Design or Review field is changed.

## Leap ambiguity and table currency

RFC 1305 Appendix E.8 Figure 8 (p.79) shows the leap label and following
midnight with the same NTP number, distinguished there by leap bits. Thus
“cannot express 23:59:60” here means **cannot uniquely identify that civil
label from IRIG Format 2 seconds/fraction alone**. It does not mean NTP never
has leap information on its native network wire. TS validity is not a leap
indicator. RDG-010's explicit Format 1 second60 remains a distinct case.

For a positive insertion whose following-midnight Unix-style value is B,
Figure 8's duplicate coordinate gives the **derived ambiguity window
[B,B+1)**, equivalently extended NTP second B+2208988800 with any fraction.
When a supplied leap table covers that transition/date, emit a
**leap-ambiguous** finding; retain the ordinary mapped UTC value, raw timestamp
and both possible civil interpretations in the basis. Do not silently
normalize a known leap label, claim that an unambiguous leap was read, or
apply a cumulative offset. Determine the window from the unrounded raw
coordinate, not from a rounded view that might carry across an endpoint.

The RFC itself contains an inconsistency: p.78's explanatory paragraph names
31 December 1989 and “second 800”, whereas Figure 8 labels 31 December 1990
and repeats second400. This is reported, not silently corrected. The numerical
duplicate-coordinate relation, not that inconsistent example's historic
insertion date, supports the window above. Tests must use independently
defined covered insertion fixtures rather than copy its calendar date.

If coverage/currency does not reach the NTP date, keep the directly mapped
UTC value and record that leap ambiguity cannot be ruled out; do not report
“unambiguous” merely because no known transition matched. Table identity,
coverage and known-current-through date belong in the basis. Finding kinds
unknown/unsynchronized and leap-ambiguous are required; this documentation
PR allocates or repurposes no stable numeric finding ID.

PTP is different: retain TAI seconds since 1970 and nanos without any leap
adjustment during parsing. To obtain UTC, select the date-appropriate
TAI−UTC offset on the table's TAI-side intervals and subtract it, preserving
covered leap state. Record table identity and currency. **Outside supported
coverage use the last known offset**, label the UTC conversion possibly out
of date and raise **TF-021 (warning)**; retain TAI and offset/table provenance
(contract 6.8/6.9, L1-TAI-004; owner 2026-11-13). The earlier contrary no-UTC
instruction came from the coordinator brief and is withdrawn, not an owner
decision. No known offset at all is missing context, not permission to invent
one. The last transition is not the table's known-current-through date; NTP
direct UTC mapping remains independent of cumulative offsets.

## Planned independent verification, not tests executed here

| Civil fixture | Era / raw NTP seconds | Expected signed Unix-style seconds |
|---|---|---|
| 1970-01-01 | 0 / 2208988800 | 0 |
| 1972-01-01 | 0 / 2272060800 | 63072000 |
| 1999-12-31 | 0 / 3155587200 | 946598400 |
| 2036-02-08 | 1 / 63104 | 2086041600 |
| 2017-01-02, after a known leap insertion | 0 / 3692304000 | 1483315200, never +/-37 s |
| 1969-12-31 | 0 / 2208902400 | -86400 |

These are hand-derived RFC Figure 4/epoch-offset fixtures, with arithmetic
checked separately from any Rust implementation. Future tests must cover:

- Every era evidence route and its exact basis label, adjacent-era pivots,
  missing pivot, exact half-era tie/conflict, era rollover and raw-zero cases.
- Raw fractions 0/1/0x80000000/0xFFFFFFFF; exact original-word retention and
  sentinel detection before rounding. Floor is accepted owner 2026-11-13.
- A synthetic covered positive insertion with B-1, B, B+fraction and B+1;
  value retained and leap ambiguity distinct from a definite second60.
  Missing/stale/empty coverage must not fabricate certainty.
- PTP within coverage, its TAI-side leap transition, table before-start and
  after-end, and a long no-new-transition interval with current metadata:
  outside coverage UTC retained with last known offset, possibly-out-of-date
  label and TF-021; TAI always retained for readable raw input.
- Every archived edition above, not a fixture made by the crate's own
  encoder; no host clock, real recordings or extra dependencies in CI.

Existing slice-2 tests intentionally pin `NeedsContext` until a future,
separately authorized implementation slice replaces them. Their existing
trace links are not evidence that these revised requirements are implemented.
No source/tests, dependencies or API behaviour change in this documentation PR.
