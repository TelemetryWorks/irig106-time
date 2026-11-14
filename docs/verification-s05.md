# S05 — RDG-012 NTP era / UTC / ambiguity

Baseline main `a81517f2e3324e2b7910a8a1651c797f07cdc6b0` (S04 merged).
Scope: L2/L3-NTP-101..104, TAI-107 and RDG-212. No dependencies, finding
levels, Design/Review fields, CLI source, or PTP conversion policy change.

## Tests-first and executed evidence

- `ef19bb5`: **three controls passed on main**: correct wire decoding, accepted
  floor, and strict contextless compatibility. No main assertion failure is
  claimed: old absolute conversion was intentionally unimplemented.
- `5997bc3`: nine new conversion tests could not compile on main (new APIs
  absent), not nine executed failures. Their existing trace IDs did not prove
  the revised conversion semantics until exercised through the new API.
- `36ca079`: explicit dated UTC leap-pivot assertion failed against the first
  uncommitted implementation, not main. Fixed by retaining original second60
  while folding only its era-comparison coordinate. Later range/operand/date/
  negative-step assertions strengthen coverage without a main-first claim.
- Final focused suite: **12 tests** (three controls, nine conversion tests).
  Contextless legacy helpers still return NeedsContext; they cannot preserve
  era assumptions/currency. Their diagnostic text now names the qualified API,
  not a pending Q-NTP. This may appear in the old CLI; no output format changes.

## Sources, arithmetic and explicit boundaries

Primary RFC5905 §6/Figure4 p14 HTML and RFC1305 E.8/Figure8 pp78–79 PDF text
refreshed from RFC Editor on2026-11-14. IRIG106-24R1 11.2.3.3 (11-26/PDF34)
and Fig11-16 (11-27/PDF35) reuse reviewed RDG-012 evidence; **no new page-image
inspection claimed**. RFCs remain outside the RCC archive; no archival change.

- [RFC5905 §6](https://www.rfc-editor.org/rfc/rfc5905.html#section-6): raw era0
  values2208988800/2272060800/3155587200 minus2208988800 give UTC0/63072000/
  946598400 (1970/1972/1999-12-31). Era1:4294967296+63104-2208988800 =
  2086041600 (2036-02-08, leap-year day39). Post2016 leap:3692304000 minus
  epoch =1483315200 (2017-01-02), never shifted37. Prior day1969 =-86400.
- [RFC1305 E.8](https://www.rfc-editor.org/rfc/rfc1305.pdf): "knowledge of all
  previous leap seconds is lost". Duplicate coordinate follows [B,B+1), not
  accumulated offsets. Its prose/figure inconsistent example date remains
  flagged in RDG-012; tests use actual2016 insertion B1483228800, not that date.
- Era resolution is caller→independent same-session pivot→labelled era0 if no
  pivot exists. Strict half-window is2^31seconds; compare exact rational
  differences in units1/(2^32*1e9)seconds, avoiding floats/rounded pivot loss.
  Exact ties, cross-session or conflicting supplied evidence remain unresolved;
  no fallback masks them. No host-clock/environment time is read. Caller/date
  constructors are explicit provenance assertions; as_pivot rejects assumed NTP.
- Raw seconds AND fraction zero wins even over bad context/declared era. Fraction1
  with seconds0 is time even though its floor view is0ns. Floor boundaries include
  0/1/4/5/0x80000000/0xFFFFFFFF; no carry, quantization1ns, raw words preserved.
- S04 table currency is independent of its final transition. Both adjacent civil
  coordinates need coverage for certainty at a narrowed start; otherwise direct
  UTC stays available with uncertainty. Covered positive insertions retain
  ordinary midnight plus possible second60 and TF-030, never a definite TF-022.
  Negative steps/baseline rows do not become positive-insertion ambiguity.
- Pre1972 uses the labelled RFC proleptic convention, not S04's physical integer
  offset model. Signed mapping survives pre1970/calendar-range limits; an
  unsupported calendar is reported separately without wrapping the Unix value.

## Clause-level coverage

| L2/L3 pair | S05-covered clauses | Remaining / boundary |
|---|---|---|
| NTP-101 | Reused correct two-LE-word reader, asymmetric fixture, truncations and trailing filler exclusion; canonical output retains raw words. | Packet Data Length/status/admission are the Format2 boundary S06a, not inferred here. |
| NTP-102 | Declared precedence; approximate/dated pivots, IDs/session/scale/year; strict full-precision half-era/ties/conflicts; labelled era0; circular-assumption export refused. | Provenance credibility is the explicit caller boundary, not automatic source discovery. |
| NTP-103 | Reused accepted floor; exact raw fraction through era selection/export; rounding/quantization basis, no carry. | No physical interval reconstruction across UTC leap insertions. |
| NTP-104 | Signed epoch arithmetic, RFC dates, zero sentinel before era/rounding, positive ambiguity window/both labels, raw/value effects and TF-029/030; uncovered UTC kept/uncertainty. | Legacy helpers remain contextless refusals. Format2 timeline/CLI adapters and report schema remain planned work. |
| TAI-107 | NTP ambiguity distinct from explicit second60/TF-022; source leap pivot retained; no NTP leap indicator invented. | Existing F1/PTP explicit-label clauses reused, not newly certified by NTP tests. |
| RDG-212 | Written analysis applied, raw/era/scale/assumption/currency and floor preserved, new edge tests and independent dates. | Human Review stays pending. PTP D7 reused unchanged; release source refresh/schema remain their checks. |

No new release-scoped deferral. Matrix links do not express clause completion.
Fuzz harness now exercises canonical raw retention; compile check is not a campaign.

## Negative checks and full CI

**37 isolated mutations** compiled and failed executed tests: declared/default
precedence/labels; cross-session/conflicting/tied pivots; fraction precision,
negative quotient and signed era; +/-37 shifts; sentinel/floor; rejected index,
proleptic/evidence metadata; coverage/narrowed start/negative step/window;
ordinary/possible leap labels; date validation/provenance; circular pivot and
scale/raw pivot assumptions; TF-029/030 omission/override/operands. All source
restored. Full bounded CI covers features, MSRV1.85, clippy/fmt/docs, no_std
wasm±serde, CLI, benches, locked all-fuzz-target compile, trace and cargo-deny.
