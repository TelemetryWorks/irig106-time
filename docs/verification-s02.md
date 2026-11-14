# S02 — calendar, BCD diagnostics and raw Format 1 closure

Baseline main `df4c6f197fa604fa50beb0dde6a13e52aa6395f7` (S01b merged).
Scope: the six S02 L2/L3 pairs in the saved 0.8.0 plan. No Design/Review
fields, finding catalog rows or requirement statements change.

## Test-first provenance

- Tests-only `f445d0e`: five probes executed against main's implementation;
  **four assertion failures, one pass**. Gregorian day/year attachment, known
  year rollover, field-specific decimal errors and decimal-six/range distinction
  failed. The DATE-only/Data Length fixture passed: existing correct behaviour.
  The rollover probe subsequently gained Result handling for the changed API.
- Tests-only `9b9b2a4`: new reading/ingestion/arithmetic APIs were absent on main,
  so these tests could not execute. This is missing-API compilation evidence,
  not executed failing tests. A mistaken test constructor name (`stated`, corrected
  to existing `standard`) was also corrected before the first successful run.
- Tests-only `6cdf504`: the simultaneous bad-digit/zero-bit assertion executed
  against the first uncommitted implementation and failed because the independent
  TF-005 warning was lost. Its fix inspects bounded masks independently of BCD
  validity. Other refinements arrived after implementation; no main-first claim.
- Final focused suite: **12 tests** (`s02_existing`: five; `s02_readings`: seven).
  The scale-preservation assertions were added after the tests-only commits.
  Existing offline/stream, properties, wire and CLI checks remain regression
  evidence, not newly failing-first S02 tests.

## Sources and hand arithmetic

Source inspection reuses the existing cited requirements and page-check records;
**no new page image was rendered or visually inspected in S02**.

- 106-24R1 §11.2.3.2 a / Figures 11-13 and 11-14, printed 11-25 / PDF 33:
  "0 = IRIG day available (Figure 11-13) 1 = Month and year available
  (Figure 11-14)". DATE alone selects three/four body words: 6/8 body bytes,
  10/12 bytes including the data word. Nonzero adjacent filler is excluded.
- Figures 11-13/14 / Table 11-16, "0 Always zero": independently specified
  word masks are DOY `[8000, c080, fc00]`, DMY `[8000, c080, e000, c000]`
  (hex). Each set bit is tested separately, exact intersections retained in
  TF-005 raw fields, warning with the decoded value kept. Shared bounding also
  preserves that warning alongside an independently malformed decimal digit.
- §11.2.3.2 b: "the time data words are inserted in the packet in binary-coded
  decimal format". Literal word0=3456, word1=0112, DOY word2=0123 (hex) give
  123 / 01:12:34.560; DMY word2=0315, word3=2000 give 2000-03-15, day75
  (31+29+15). Tests do not generate these bytes with a production encoder.
  Every representable digit pattern is checked at every defined digit position;
  fields narrower than four bits cannot encode all A..F patterns. Decimal 6
  is valid as a digit but minute60 is a range error, not a decimal error.
- Gregorian 1900/2100 are common years, 2000 leap: Feb29 and DOY366 follow
  the actual year independently of raw LY. Unknown year remains unknown;
  stepping past its unambiguous bound requires context, not a guessed leap year.
- RCC 200-16 §3.6: "excluding leap second days when a second may be added
  or subtracted"; Appendix A.2 / RDG-010: UTC 2016 day366 23:59:60 remains
  a distinct label. Zero shift preserves it; nonzero leap-label arithmetic,
  unknown-scale arithmetic and UTC day crossings refuse missing context.
  Ordinary TAI/GPS nominal shifts preserve scale; no UTC leap table is inferred.

## Clause-level coverage

| L2/L3 pair | S02-covered clauses | Remaining / boundary |
|---|---|---|
| ABS-101 | Shared Gregorian/range validation, known-year rollover, unknown-year guard, checked nominal signed shifts; labelled leap second retained with typed context refusal. | Explicit leap-table elapsed arithmetic remains S04/S10. Legacy unscaled AbsoluteTime cannot represent second60; it is not a UTC elapsed-time oracle. No year inference. |
| CSDW-105 | DATE bit9 alone selects 6/8-byte body; exact Data Length bounds; filler excluded; common bounded word inspection reused for malformed findings. | Edition/ITS/EXT qualification remains S03; these tests do not resolve an edition. |
| BCD-103 | Every defined always-zero bit, exact word/mask operands in TF-005, value retained, no filler inspection, independent warning even with malformed BCD. | Canonical Format1 decoder/engine ingestion is the warning-preserving path. Legacy body-only strict parsers retain ReservedBitSet errors; they are not the canonical admission API. CLI adoption remains its planned slice. |
| BCD-104 | Every figure digit's representable patterns checked before arithmetic, exact field/word/digit/bit/raw error; decimal errors separated from total range and zero-bit findings. | No interpretation of reserved bits as extra decimal precision; no new layout inferred. |
| ERR-102 | Structured decimal cause retained through offline and stream TF-004 with original packet/channel/raw RTC; separate simultaneous zero-bit finding. | Generic already-decoded observe has no original parser cause to recover. Raw adapters for other formats and serialized diagnostic reports remain their slices. |
| ERR-103 | Field limits, invalid decimal vs range, Gregorian February/day/year checks independent of LY, day0 refusal, scale-qualified second60 validation. | Existing Chapter4 microsecond validation reused; Chapter4 context S06 and year/LY consistency inference S08 remain. Leap-history/table verification is not established by eligible-date validation. |

No new release-scoped deferral is introduced. Remaining clauses above still
block 0.8.0 as allocated by the plan. Matrix links cannot express partial clauses
or distinguish a canonical API from retained compatibility APIs: **128/288 links
(up from 116) is not full compliance**.

## Targeted negative checks and validation

Seventeen isolated source mutations each produced an executed assertion failure
in the focused suite (no compile failures counted): common-year day366 accepted;
known year not advanced; unknown-year leap guessed; leap-label shift normalized;
unknown-scale shift accepted; UTC day crossing accepted; shifted scale changed;
DATE ignored; zero masks lost; decimal A..F accepted; wrong digit word, ordinal
or raw operand; parse cause dropped; supplementary TF-005 omitted; zero-mask
operand lost; TF-005 downgraded. Sources were restored after each mutation.

Bounded full CI equivalent includes all feature test sets, MSRV1.85, clippy,
fmt, warnings-as-errors docs, CLI build/clippy, no_std wasm with/without serde,
trace freshness, benches build, locked type-check of every fuzz target and deny.
