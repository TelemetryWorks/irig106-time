# S04 — validated leap tables and PTP→UTC

Baseline main `1f3bb6595b9f70ca7a4e935a7f5a97c05bc9fb61` (S03 merged).
Scope: L2/L3-TAI-101..104 and PTP-104; no dependency, finding-level, policy,
Design/Review field or CLI source change. No NTP conversion or schema freeze.

## Honest tests-first evidence

- `7fdc4df`: **two assertions failed on main**: 1972 TAI transition returned
  offset10 instead of11; conflicting insertion grew the table29 instead of28.
  Those probes later gained typed fields/Result handling for the changed API.
- `d887eba`: six new-API tests could not compile on main (missing API/typed
  field mismatch), not six executed failures. The existing28 offsets were
  already correct; they are reused, independently checked, not rewritten facts.
- `8d6520f`: narrowed-start leap assertion failed against the first uncommitted
  implementation, not main. UTC second60 repeats midnight's Unix coordinate;
  supported-start must be checked on the TAI side. Serde test passed against
  the first implementation; no failing-first claim for it.
- `c618370`: extension-shrinks-coverage assertion failed against that uncommitted
  implementation before its fix. Later custom-offset/evidence and legacy leap-refusal assertions are
  additions after implementation. Final focused suite: **12 tests**, including
  one serde-only test, plus compatibility/wire/property regressions.

## Source inspection and derivation

Implementation review on2026-11-14 compared **every28 transition** with
[IERS Leap_Second.dat](https://hpiers.obspm.fr/iers/bul/bulc/Leap_Second.dat),
updated through Bulletin C72 (2026-07-06), expiring2027-06-28. Raw SHA256:
`6cb6f5d4b819f2e568e25db4b0b26d89dedf031fdffb18bc94d40f4e94e268d7`.
[Bulletin C72](https://hpiers.obspm.fr/iers/bul/bulc/bulletinc.72) SHA256:
`310e172eadacacca3adf92cb9bd646fcc0d6320e35e996a106d2644ede3b52f0`.
It confirms no leap at December2026 end. Operational coverage conservatively
ends at00:00 UTC on the file's expiry date: known-current-through1814140799
includes every fraction of2027-06-27's last second. This explicit date-cutoff
policy is not an IERS-specified nanosecond instant. Final transition1483228800
(2017-01-01) is separate. Release must recheck source/currency; no live fetch
occurs in the library. The committed fixture contains numeric source facts;
expected Unix seconds use independent `(MJD-40587)*86400`, not an encoder.
[USNO history](https://maia.usno.navy.mil/ser7/tai-utc.dat) shows pre1972
fractional/rate offsets; this integer model refuses exact UTC there.

IRIG106-24R1 §11.2.3.3 (11-26/PDF34) states PTP's TAI scale/1970 epoch;
reuse reviewed standard-page evidence, **no fresh IRIG page-image inspection**.
For an insertion at UTC E, new offset b and old a=b-1: new TAI boundary E+b;
TAI E+b-2 is23:59:59, E+b-1 is23:59:60 with offset a, E+b is midnight with b.
The leap second's Unix-style coordinate repeats E; CalendarReading/basis retain
second60. Examples: E=78796800, offsets10/11; E=1483228800, offsets36/37.
Hypothetical future +/-1 fixtures are labelled caller data, not IERS predictions.
Only offset changes at the accepted June/December boundaries are installed.

## Clause-level coverage and limits

| Pair (L2/L3) | Covered in S04 | Remaining / boundary |
|---|---|---|
| TAI-101 | Typed fields; ordered UTC/TAI transitions, duplicates/unit steps/coverage validation; pre-start/empty context refusal; immutable entries and validated serde loads. | Standalone source credibility remains caller/release review, not inferred from a URL. |
| TAI-102 | Complete pinned IERS list/metadata review and independent fixture diff at implementation. | Fresh source/currency comparison must recur at release; this is not future-release certification. |
| TAI-103 | Checked atomic add; validated ordered extension/replacement; known-history conflicts rejected, coverage not narrowed by extension; previous answer basis owned/immutable. | Caller supplies versioned identity/source; no hidden network acquisition. |
| TAI-104 | Coverage distinct from final transition; stale UTC retained/labelled with actual last offset, TF-021/raw operands; missing offset retains TAI/no UTC; severity override cannot repair validity. | NTP direct-UTC ambiguity/currency is S05. Versioned report schema is S12/S17. |
| PTP-104 | Exact TAI-side intervals, leap labels, checked signed shared conversion, original TAI/nanos and full source/offset/currency basis. | Canonical converter is complete for this model; Format2 timeline adapters are S09. Legacy correlators still return bare calendars without currency/basis and refuse leap-label projection; they are not evidence of full answer compliance. |

No new release deferral. Compatibility APIs propagate missing context and no
longer fabricate a midnight for second60; the CLI indirectly inherits those
errors, with no direct CLI source/output-format change. Future evidence-bearing
adapter/report adoption remains a0.8 blocker. Matrix links are not full clauses.

## Negative checks and full verification

**32 isolated mutations** compiled and failed focused tests: duplicates/source/
pre1972/step validation; historical rewrite, input sort and coverage shrink;
approximate/strict-boundary TAI lookup; narrowed start; missing context; constant37
fallback; last-transition/endpoint currency; stale UTC/raw TAI/leap/offset/source/selected-transition
loss; TF-021/022 omission, validity/raw seconds/nanos/offset/expiry loss, severity
override ignored, TF-022 grade changed; serde bypass; each legacy correlator
normalizing the leap label into midnight. Sources restored each time.
Full bounded CI equivalent covers features, MSRV1.85, no_std wasm±serde,
clippy/fmt/docs, CLI, benches, locked all-fuzz-target type-check, trace and deny.
