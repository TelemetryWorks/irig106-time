# S07 — Chapter 4 explicit calendar/partial context API

Base main `8d3bd38fcda78c1ae82fa4f6a70d5e638bac0331` (S06b2 merged).
Scope: L2/L3-ABS-102 and WORD-101 context clauses, Q-CH4/D6 owner2026-11-12.
No dependency, policy default, finding ID/level, Design/Review field or CLI change.

## Sources and hand derivations

Inspected existing governing requirements, architecture and T-10 page evidence.
No new page-image inspection is claimed. T-10 records visual checks of106-24R1
Ch4 Fig4-4, printed4-10/PDF14 and §4.7,4-11/PDF15; Ch11 Fig11-4,11-10/PDF18.
§4.7: "resolution of 655.36 seconds", "10 milliseconds", "three least
significant bits of the BCD Julian date", "at a rate necessary to resolve
time ambiguities". RCC123-20 `enI106_Ch4Binary2IrigTime`, B-43/PDF149 applies
no origin. Explicit context/partial behaviour is owner policy, not an epoch
specified by the standard. No nearest-time-packet option exists.

- Binary: `(high*65536+low)*10000000 + usec*1000` ns. Low0x8000 gives
  327680000000ns; high1/low0x1234/usec9999 gives701969999000ns; maximum
  0xffffffff count plus9999us gives42949672959999000ns. These are elapsed
  counts, not day fields. Shared secondary/IPTS bytes remain
  reserved/usec/low/high, each16-bit LE; reserved bytes are caller-retained.
- Fig4-4 BCD high0x31c5 = fragment1 at15..13, tens-hour2 at12..11,
  hour3 at10..7, tens-minute4 at6..4, minute5 at3..0. Low0x5678 =
  tens-second5 at14..12, second6 at11..8, tenths7 at7..4, hundredths8 at3..0.
  Binary1234us gives23:45:56.781234. Reserved lowbit15 is separate TF-005
  evidence, value kept; it remains count data for binary (T-10 DERIVED reading).
- Explicit caller interpretation `LowThreePackedBcdBits`: day131 packs as0x131,
  low3bits1; day139 also gives1. This does NOT make fragment1 a full day,
  and differs from binary131&7=3. It is a labelled caller-selected reading,
  not a resolution of §4.7. No interpretation is selected by default.
- Complete BCD context retains the edition-dependent unresolved caveat:
  106-04..15 "three least significant characters of the BCD Julian date";
  106-17+ "three least significant bits of the BCD Julian date". Caller edition
  labels which wording applies; absent edition labels both as undeclared.
- Nominal origin2023 day365 23:59:59.9 plus0.200003s becomes2024 day1
  00:00:00.100003. Year2024 day366 crosses to2025; year9999 overflow errors.
  Scale remains explicitly unknown. These calendars do not claim physical
  UTC/leap elapsed arithmetic; use the S04 normalization path where applicable.

## Executed tests-first and mutation evidence

Tests-only commit `53bb115`: one existing-API test compiled and failed on main
because legacy `to_absolute()` invented a date. Nine new-API tests could not
compile on main (missing `ch4` module), not nine executed assertion failures.
One later test adds independent partial-context validation and valid extrema;
scale assertions were added after implementation. Initial focused suite:11 tests; after review:12 tests.

All28 initial isolated mutation probes compiled and failed assertions, original sources
restored. A first day-range mutant failed type inference; it is NOT counted as
assertion evidence. A type-preserving removal was rerun and failed the partial
context assertion. No fuzz campaign is claimed.

| Mutation group | Observable assertion failure |
|---|---|
| Binary low-top masking, high shift, count scale, omitted microseconds | Exact elapsed count fixtures differ |
| Microsecond range guard | 10000/65535 incorrectly accepted under either weighting |
| Ignore origin, accept undated origin, invent absent origin | Exact dated rollover or partial/calendar result differs |
| Missing-origin and binary-year labels | Explicit missing-context flags differ |
| BCD digit guard and total-range guard | Independent partial decimal/range fixtures accepted |
| Year/day guards and leap-year context | Invalid supplied partial context accepted; no later calendar constructor can mask the missing guard |
| Fragment guard, binary-ordinal rather than packed-BCD comparison, raw fragment | Explicit mismatch/collision/value fixtures fail |
| Guess interpretation, missing year/day/interpretation flags | Eight context subsets/calendar eligibility or missing labels differ |
| Drop reserved evidence or reject reserved warning | Raw/evidence differs or usable calendar is discarded |
| Resolve wording, classify older wording as bits, guess undeclared edition | Unresolved/edition labels differ for all supported editions |
| Implicit UTC scale | Explicit unknown scale assertion differs |

## Clause ledger — each suffix denotes L2 and L3

| Requirement | Covered by this slice | Remaining / qualification |
|---|---|---|
| ABS-102 | Shared raw binary projection; full count/usec checks, no implicit origin; labelled partial elapsed count, explicit dated-origin nominal calendar; withdrawal of legacy17-bit split | Existing shared secondary/IPTS parsers/encoders remain unchanged; their prior fixture links do not claim this new context work. Secondary/stamp common answers and aggregate reserved findings remain S10; complete encoder audit S13 |
| WORD-101 | Explicit weighting, full binary count; all Fig4-4 BCD groups/ranges; separate reserved-bit evidence; year/full-day/interpretation required; missing flags/raw values; packed-BCD fragment mismatch/collisions; unknown/older/current wording always unresolved; no nearest reference | BCD packed-bits field reading stays caller policy/proposal, not a settled historical standard meaning. No PCM/1553 body extraction; common answer/finding attachment is S10b |
| ANS-101 / POL-101 contributions | Exact context and raw words retained; scale explicitly unknown; binary overflow/context failures checked | No whole answer/policy/schema coverage claimed; immutable serde/schema S12; caller must retain extraction provenance and secondary reserved bytes for S10 |

Tests are partial-clause evidence, so no new whole-requirement Requirements
markers. Existing matrix status is160/288, delta0; WORD-101 remains Draft in
that coarse matrix despite this API's clause evidence. Do not promote it by
attaching one narrow digit/context test. Full human Review remains unchanged.

## Verification and migration

All18 full bounded CI-equivalent checks passed: feature suites, MSRV1.85, no_std WASM±serde,
clippy/fmt/docs, CLI build/clippy, benches, locked all-fuzz compile, trace and
deny; jobs/threads2. Existing secondary/intra-packet/stream/offline regressions
remain required. No CLI output change. No new release deferral/owner question.

Legacy `Ch4BinaryTimeExt::to_absolute()` stays callable and documented with a
warning, but now validates microseconds and returns NeedsContext rather than
incorrectly splitting a binary day. Use `Ch4Words::from(raw).binary(None)` for
partial elapsed time, or `.binary(Some(dated_origin))` and inspect calendar and
context. BCD uses `.bcd(BcdContext { .. })`; returned raw words/wording/missing
labels survive reconstruction. Fields of readings are private; no unchecked
Deserialize or host-clock/nearest-reference fallback is added.

## PR25 review — legacy validation order

Tests-only revision of reviewed head98179ac. Added
`legacy_ch4_conversion_rejects_invalid_microseconds_before_missing_context`:
Ch4BinaryTime with microseconds10000 must return OutOfRange with field
`ch4_microseconds`, value10000 and max9999, before NeedsContext. The fixture
uses §4.7's maximum9999 (4-11/PDF15) and Fig11-4's binary usec word
(11-10/PDF18); no new visual-page inspection is claimed.

The assertion passes on unchanged production code. Isolated removal of the
legacy call to `Ch4Words::from(*self).binary(None)?` compiled, then failed this
exact assertion: the mutant returns NeedsContext instead of OutOfRange.
Original source restored; no production changes. Total focused tests12;
mutation evidence28 initial +1 review =29 assertion-killed mutants.
No whole-requirement marker added; matrix160/288 unchanged. Full18 bounded
local CI-equivalent checks rerun for this revision, including CLI/benches/fuzz,
MSRV/no_std/features, formatting/lints/docs, trace and deny; jobs/threads2.
