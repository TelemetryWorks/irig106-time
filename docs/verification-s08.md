# S08 — session year evidence and LY validation

Base main `dd1274b33cc37e8b7631f4f5fc8d53d19b7e8621` (S07 merged).
Scope: COR-110 / CSDW-104 year clauses; TF-032 owner WARNING2026-11-14.
No dependency, CLI, Review field or TF-001..031 row/default change.

## Sources and arithmetic

Inspected existing contract5.3/D9, requirements and prior page evidence; no
fresh visual-page inspection claimed. 106-24R1 Figures11-13/14,11-25/PDF33:
DOY carries no year; DMY carries four decimal year digits. §11.2.3.2a:
"Bit 8: Indicates if this is a leap year". Ch9 Table9-4 R-x\RI4:
"Date and time original recording was created". Year ranking is owner reader
policy, not a priority rule stated in those figures. No G\OD or host-clock API.

Literal fixtures place DOY366 as word2=0x0366; DMY2024-01-01 as word2=0x0101,
word3=0x2024. CSDW0x00000301 has external SRC1, DATE1 and LY1;
0x00000001 has DATE0/LY0. Seconds1 are word0=0x0100. December31/January1
transitions increment known years; a later2024 Jan1 anchor dates a preceding
DOY365 as2023. LY is checked independently and never supplies a year.
Nominal calendar support remains0..9999 with checked rollover failure, not
saturation. Unknown year remains a partial native calendar with TF-020.

A complete DMY second60 label supplies its readable year even when physical
normalization needs context. Fixture2024Jun30 23:59:60 is an allowed labelled
UTC calendar, not a claim that IERS scheduled a leap then. It cannot become an
ordinary time anchor; existing TF-022/raw leap evidence remains intact.
Explicit normalization rebases on year changes using its original table:
2024Jan1 Unix1704067200 +37 = TAI1704067237s;2030Jan1 Unix1893456000 +37
uses D7 last-known offset with TF-021.1971 is before the exact table baseline:
native capture retained, displaced physical answer withheld with a recorded
normalization context error. No new implicit table/scale conversion is added.

## Executed ordering and negative checks

Tests-only commit `be38bdc`: three tests compiled and failed on main:
TF-032 level None instead of Warning; later DMY failed to date earlier DOY;
unknown-year DOY366/LY0 emitted no TF-007. Five new-API tests could not compile
on main (missing years module/context/evidence APIs), not runtime failures.
Nine later tests/expanded assertions are after implementation. The leap-label
year probe failed the first implementation (None instead of2024), then passed
after separating readable year evidence from ordinary time admission.
Another failing-before-fix probe showed newly invalid day366 could mask a
previous eligible stream anchor. Invalidated retained references are now removed;
held fallback snapshots select a remaining eligible anchor before jump release.

Final focused suite:17 tests (3 existing-API +14 year integration tests), plus
the existing catalog/policy and complete feature regression suites. Targeted
mutations:34 isolated probes compiled and failed assertions, sources restored.
Groups cover DMY/caller/RI4 priority and context, first/latest/own DMY evidence,
rollover/late phase and session reset, reference/pending refresh, each emitted
TF-004/007/020/032, severity/value/raw/related evidence, Gregorian admission,
readable leap-year evidence, normalization rebase/missing context/stale TF-021,
and invalid-reference eviction/held fallback. Two initial probes survived;
late-phase and held-jump assertions killed them on rerun. A dated legacy RI4 projection also initially outranked explicit caller context;
a probe failed2022 versus2023, then a stable source-priority sort corrected it
while preserving both candidates. Full-suite verification
also found an old Format2 test treating32 as pending; it now checks unknown33. No fuzz campaign or fresh standard-page verification claimed.

## Clause ledger — suffix denotes L2 and L3

| Requirement | Covered | Remaining / qualification |
|---|---|---|
| COR-110 | Same-session eligible DMY→caller→governing RI4; raw/context/selected/contrary candidates and packet provenance; known rollovers including late references; unknown partial/TF-020; TF-032; offline later evidence, stream release-time evidence, session isolation | Governing RI4 extraction/ranges are supplied per reference; TMATS adapter S09. Configurable alternative complete year policies remain S12; no whole-requirement promotion |
| CSDW-104 | Raw LY preserved; known leap/common year mismatches and unknown-year day366/LY0 emit TF-007; impossible known-year366 additionally TF-004/unadmitted; warning alone keeps valid time; DMY leap-label year evidence remains usable | Existing decoder/bit extraction retained; no claim this narrow suite independently verifies every DATE/source/LY combination |
| FND-101 | All32 owner-approved catalog levels, exact TF-032 title, override validity independence and candidate/counter/related evidence; TF-001..031 table unchanged | Existing whole catalog markers retained; new year checks are partial-clause evidence |
| FND-102 | TF-007/020/032 value effects and levels; TF-004 for impossible calendar; normalized changed-year stale TF-021 reuse | Not linked: no single test here covers every contract6 degraded case, including other pending adapter/report cases |
| ANS/POL/STRM contributions | Exact year context in reference basis; held-answer refresh before interpolation AND jump/cap/EOF fallback; no emitted revision, no channel switch, per-session reset; governing-context changes retained | Complete shared schema/snapshots S12 and answer adapters S10; no new whole requirement claim |

## Boundedness and same-priority qualification

Offline finalization orders year evidence by unwrapped RTC/packet ID within each
session; the earliest eligible DMY anchors yearless references. Stream uses first observed DMY as its stable anchor and retains
constant-size phase, first DMY and latest DMY candidate in addition to its existing
bounded references/pending queue. Superseded contradictory packets remain in
offline references or emitted stream findings; caller retains emitted records,
not an unbounded hidden library history. Each basis has at most five candidates.
A DMY packet keeps its own explicit calendar. Other channels may supply year
facts without supplying absolute anchors or changing Option A commitment.

**Same-priority DMY choice is an explicit implementation proposal for owner
confirmation:** keep first eligible DMY as the stable anchor for yearless times,
retain each DMY's own calendar and all encountered contradictions in findings.
Figures11-13/14 and contract5.3 set DMY priority but do not settle conflicting
DMY versus DMY. Recommendation: retain this stable-first rule, not silent last
write. First/latest bounded summary is not a claim to retain all raw history in
every stream answer; the emitted finding stream is the durable evidence.

## Migration and verification

BREAKING additive field: ReferenceInput literals initialize `year_basis` with
Default. `.with_year_context(YearContext { caller, recording })` validates0..9999;
recording is caller-extracted governing RI4, never a TMATS origination date.
Original decoded value and context are immutable in YearBasis; existing public
ReferenceInput is still an explicit caller boundary. No complex Deserialize added.

Matrix remains160/288 (delta0); partial checks have no whole-requirement markers.
All owner acceptance/source changes are enumerated in the owner-decision audit.
Dated ADR/analysis/verification records retain history with subsequent-status
banners. No human Design/Review status changed and no release scope was deferred.
All18 local CI-equivalent checks passed with jobs/threads2: features, MSRV1.85, no_std WASM±serde,
clippy/fmt/docs, CLI build/clippy, benches, locked all-fuzz compile, trace and deny.
