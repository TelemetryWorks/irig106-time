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

Tests-only commit `be38bdc`: FOUR tests compiled and failed on main:
TF-032 level None instead of Warning; later DMY failed to date earlier DOY;
unknown-year DOY366/LY0 emitted no TF-007; and
`s01_findings::every_catalog_level_and_identity_is_explicit` failed the deliberate
TF-032 None→Warning catalog flip. The first three were in the initial probe run;
the fourth was locally reproduced from that exact tests-only commit during review
(previously omitted from this record). Five new-API tests could not compile
on main (missing years module/context/evidence APIs), not runtime failures.
Nine later tests/expanded assertions are after implementation. The leap-label
year probe failed the first implementation (None instead of2024), then passed
after separating readable year evidence from ordinary time admission.
Another failing-before-fix probe showed newly invalid day366 could mask a
previous eligible stream anchor. Invalidated retained references are now removed;
held fallback snapshots select a remaining eligible anchor before jump release.

Initial focused suite:17 tests; after first review:23; review2:33 tests (3 existing-API +30 year integration tests); current review3:39 tests (3+36), plus
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

Both engines define **first by recording observation order**: the first observed
eligible DMY packet (identified by packet ID), never the smallest RTC. Stream
requires increasing packet IDs; offline preserves its observation sequence.
Offline still sorts counters to classify rollovers, but then rebases the phase
to the first observed eligible calendar and selects first/latest DMY in observation
order. A later-observed DMY with an earlier RTC cannot replace the first anchor.

Stream retains constant-size phase, first DMY and latest DMY candidate in addition
to its bounded references/pending queue. Superseded contradictory packets remain
in offline references or emitted stream findings; caller retains emitted records,
not an unbounded library history. Each basis has at most five candidates. A DMY
packet keeps its own explicit calendar. Other channels may supply year facts
without supplying absolute anchors or changing Option A commitment.

**Same-priority DMY choice: owner-accepted2026-11-15 with the crossover condition below.** keep
the first observed eligible DMY as the stable anchor for yearless times, retaining
each DMY's own calendar and all encountered contradictions in findings.
**Case (a):** if a DMY arrives after an answer was released, offline finalization
can backdate earlier references from that same-session evidence; stream cannot
revise the emitted answer. Only retained/unreleased stream answers can gain it.
When retained candidates remain contradictory, TF-032 is emitted per affected
reference, including successive yearless references; it is not deduplicated to
one warning per session. A refresh with new candidate evidence can also emit a
finding for an earlier retained reference.

Figures11-13/14 and contract5.3 set DMY priority but do not settle conflicting
DMY versus DMY. The owner accepted stable-first2026-11-15 with phase advancement; first/latest bounded summary is
not a claim to retain all raw history in every stream answer. The emitted finding
stream is the durable evidence.

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

## PR26 review revision — sparse rollover and observation order (2026-11-15)

Tests-only commit `4da675e`: six review tests compiled against reviewed head
ea29738; FIVE failed assertions (offline/stream sparse rollover, late correction,
long gap and stable-first order). The RI4-only rollover test already passed and
is not claimed failing-first; it fails under the specified RI4 adjustment mutant.
Later boundary/observation-origin assertions and the fixture update below are
labelled post-implementation additions. Original catalog fourth failure reproduced
in a detached worktree of be38bdc, then the task-owned worktree was removed.

- Literal Fig11-13 words0x5900/0x2359/0x0365 or0x0366 give23:59:59 on
  common/leap year-end. At +86401s RTC, next reference is day2 midnight;
  both engines now return2024/2025 respectively with no false TF-007/017.
  The22f8631 rule treated any ordinal-day decrease with positive RTC as a
  rollover; review2 below supersedes that rule with a clock-consistency check. Late references with a greater ordinal day use the prior
  phase symmetrically. The late half-second fixture uses word0=0x5950;
  caller late tolerance2days is explicit test policy, not a changed default.
- RI4 alone, caller absent, advances at common/leap rollover in both engines;
  selected origin stays RecordingDate and supplied year remains in evidence.
- A conservative gap guard rejects eligible year evidence separated by365days
  or more in unwrapped RTC with NeedsContext.365days is the shortest Gregorian
  cycle; a gap can hide complete cycles/two boundaries near year end. This is a
  reader ambiguity guard, not an IRIG-specified threshold or guessed year count.
  Tests cover exactly365days and400days, bridging raw48-bit wraps using data
  observations at most100days apart. Unusable source-None observations are skipped.
  At22f8631 offline finish reported NeedsContext (superseded by partials in
  review2 below); stream rejects before caller progress,
  timeouts or packet ID commit, preserving all four held answers. The caller can
  retry with an explicit session boundary and appropriate date/year context.
- A later DMY with earlier RTC keeps the same first observed anchor in both
  engines. Successive yearless references each report TF-032 while contradictory
  candidates persist. Counter-order phase classification also preserves the
  first-observed caller/RI4 year origin: a late earlier-calendar-year reference
  cannot silently shift the original observed year from2023 to2024.
- The existing invalid-day366 fallback fixture formerly used a forward-RTC step
  back to2023Dec31. Under the requested any-decrease rule that is itself a year
  rollover. It now uses2024Jan1 00:00:02 at RTC day+1s: the preceding day366 is
  still invalid2023, the earlier valid anchor survives, and the committed variant
  still makes a +1s jump to test the held fallback. Literal bytes and expected
  post-jump seconds/date were updated; the original safety assertion is retained.

All review tests are partial-clause evidence: matrix160/288, no new whole markers.
Eleven isolated review mutations compiled and failed assertions, original sources
restored: Jan1-only forward/late rules, RI4 adjustment loss, gap guard removal,
365day-boundary relaxation, stream preflight removal, counter-order/reversed DMY
selection, per-reference TF-032 suppression, unusable-observation gap blocking,
and loss of the observation-year phase origin. Historical initial34 mutations
and these11 review runs are recorded separately; no fuzz campaign claimed.

This revision passed all23 focused tests and all18 bounded local CI-equivalent
checks (jobs/threads2), including CLI/benches/fuzz compile, MSRV/no_std/features,
lints/docs/format, trace and deny. Actionlint/diff clean; matrix unchanged160/288.

## PR26 second review — jumps versus rollovers; offline gap partials

Historical rule below is superseded by review3 nearest-explanation policy;
its gap/partial behavior and strict comparator guards remain in force.

Tests-only commit `1eadd5f`, against reviewed22f8631: the combined backward-jump
regression failed at its offline assertion (2024 rather than2023); its stream
cases were already present in that commit and are exercised by the final suite.
The changed offline-gap expectation also failed because finish returned
NeedsContext. Equal-counter/same-day-late/adjacent-day cases passed before the fix
and are not claimed failing-first; their specified comparator mutations fail.
Tolerance, recovery, normalization and leap-label probes were added after the
initial implementation. The normalization probe failed before its targeted fix.

**Rollover qualification:** compare full cyclic calendar positions with RTC
elapsed time, using the caller's `jump_threshold_ns` when set, otherwise the
coarser endpoint resolution (Format1 normally10ms, accepted C3 policy). The
residual bound is inclusive. Positive RTC and strictly decreasing ordinal day
are independent forward guards; late correction mirrors the order and interval.
The calendar check alone does not establish a year change. LY never selects a
year; unknown-year comparisons try both Gregorian lengths without assigning a
year. A nonmatching decrease remains in the same year and TF-017 reports the
clock jump through the existing timeline machinery. This tolerance is reader
policy, not a new timing guarantee claimed from IRIG106.

- Fig11-13 fixtures: day200 00:00:05 -> day199 23:59:59 is a6s backward
  reading at RTC+1s, so TF-017 residual=-7s, year2023 retained in both engines.
- Equal RTC at Dec31 23:59:59.99 -> Jan1 cannot advance the year, even though
  its10ms cyclic interval is within the default10ms resolution tolerance of0.
- A day200 00:00:04.50 reference arriving after00:00:05 with RTC-.5s stays
  in2023. It is ordinary late data, not a prior-year reference.
- Jan2 2023 23:59:59 -> Jan1 2024 midnight spans363days+1s and differs by
  only one ordinal day: the `day+1<last` mutation misses the valid rollover.
  Data observations bridge raw48-bit wraps in<=100day steps.
- A10ms calendar interval versus RTC10ms+100ns crosses with tolerance100ns,
  but not99ns or0. An explicit Dec31 23:59:60 label retains its own second
  when comparing to Jan1; this is a format-allowed fixture, not an IERS claim.

**Offline>=365day gaps:** preserve the timeline. Start a new year-evidence
continuity segment and keep yearless references as admitted partial clocks;
TF-020 INFORMATION applies because the affected reference's year is unknown
(contract5.3/6.8 and the existing architecture§8 row). It does not erase known
years before the gap. No identifier/title/level is invented or changed. Original
caller/RI4 context, signed gap ticks and a NeedsContext year_issue remain in the
basis; TF-020 carries the gap and unapplied session-origin years as evidence.
A session-origin year cannot count unknown cycles, so it is not silently reused.
A fully dated value keeps its own year; a DMY within the new segment can
re-establish the year for other references (including earlier references in
that segment). Pre-gap anchors cannot cross it.
Cached absolute normalization is cleared while year is unknown. Native capture
partials survive; physical normalization needs dated context again.

**Stream choice:** keep the approved reject/retry behavior. An eligible reference
after such a gap returns NeedsContext before progress/timeouts/packet identity
commit; explicit boundary/date context permits retry. Unusable observations do
not flush previous references. Offline can inspect/finalize all observations and
retain labelled partials; stream does not retroactively edit emitted answers.

Earlier year-only fixtures placed midnight year-end and Jan1 just1 RTC second
apart. They now use86400s (or86401s to Jan1+1s), and the ordinary late fixture
uses consistent half-second clock fields plus explicit2day caller late tolerance.
The invalid-day366 fallback fixture is again DMY2023Dec31 at RTC day+1s:
that backward day is correctly a jump without changing year; its held-fallback
and earlier-eligible-reference assertions are preserved.

Fourteen isolated mutations compiled and failed assertions, sources restored:
`>`→`>=` forward progress (equal-counter test), `day<last`→`day+1<last`
(adjacent-day true rollover), late `>`→`>=` (same-day late), bypassed clock check,
exclusive tolerance boundary, omitted explicit leap second, fatal offline gap,
reused session-origin year, cached absolute normalization, missing TF-020, wrong
gap evidence, missing post-gap dated anchor, signed unknown-year jump loss,
and a signed comparison bridging an unknown year gap. A first gap-evidence probe
survived; exact signed-field assertions killed it on rerun. Historical34+11
mutation runs remain dated evidence; these14 are the current review's runs.
No whole-requirement marker added; matrix remains160/288. FND-102 stays clause
coverage only. No Review/Design field, CLI, dependency or release-scope change.

The same-year signed clock comparison also works for unknown absolute years,
but only within a resolved relative phase and year-continuity segment. It reports
the same-7s residual without inventing a year and does not bridge unknown gaps.
The new unknown-year regression first failed with no TF-017, then passed after
that signed-relative comparison change. These additions are after the first
review2 tests-only commit, not extra main-first failures.

Exact surviving-mutant checks (isolated source edits, compiled assertions):

| Mutation | Killing assertion |
|---|---|
| `ticks > last` → `>=` | equal-counter year-end/day1: phase1 instead of0 |
| `day < last` → `day + 1 < last` | real Jan2→Jan1 rollover:2023 instead of2024 |
| late `day > last` → `>=` | same-day-.5s observation spuriously emits TF-017 |

A first full-check pass found a test-initializer clippy warning; its initializer
is corrected. Full bounded CI equivalent is rerun after these final changes.

Final review2 verification: all33 focused tests, all14 isolated mutation assertions,
and all18 bounded local CI-equivalent checks passed (jobs/threads2). Actionlint,
diff and trace checks pass; matrix160/288 unchanged. No merge or publication.


## PR26 third review — nearest calendar explanation (2026-11-15)

Tests-only commit `9cc7d82` runs against reviewed `e31e21b`. Five new tests
compiled: THREE runtime failures (New Year correction separately in offline and
stream, and the common-year bounded-explanation case); two controls already
passed (late larger-day jump and unknown-year rollover). They are not claimed
failing-first. The revised existing threshold test also changes its expected
year to2024 at all three thresholds. A sixth test, partial/late correction,
was added after implementation to cover signed findings and the mirrored rule.
An initial test-development run used an invalid session0 quality lookup; that
fixture was corrected before the tests-only commit and is not behavioral evidence.

**Reader policy, not an IRIG jump rule:** contract5.3 requires advancing the
year across31December; Figures11-13/14 define the calendar fields, but the
standard supplies no algorithm to disambiguate clock jumps. For strictly
forward RTC and a smaller ordinal day, compute the signed linear calendar span
and each allowed cyclic span (linear plus365/366days). Compare each span with
elapsed RTC: choose the smallest absolute residual. A known earlier year permits
only its Gregorian length; with no year, both lengths are considered without
assigning an absolute year. A tie keeps the same year. Late references mirror
this comparison using the candidate previous-year point and the retained later
point. Strict forward RTC/day order remain separate guards. Same-day late
observations and equal-counter day changes cannot advance the phase.

The selected residual must be at most half the shortest Gregorian year,
182.5days. This conservative reader bound is independent of the jump threshold;
above it the year is unresolved rather than guessed. Offline starts a fresh
year-evidence segment, preserves native partial clocks and emits existing TF-020
INFORMATION, including gap/context evidence. Stream preflights a constant-size
clock clone before progress, draining or identity commit, returns NeedsContext
and allows retry with an explicit boundary. This extends the existing full-cycle
gap behavior without adding a finding or changing the recording session.

Jump reporting remains the caller threshold or coarser endpoint resolution:
chosen residuals above it raise TF-017. Known-year comparisons use existing
absolute_delta; unknown-year cyclic corrections retain a separate signed
residual/related packet for TF-017 and never create an absolute delta or drift
across an unqualified year. Severity overrides remain factual-validity neutral.

Hand-derived Fig11-13 fixtures and observations:

- 2023 day365 23:59:59 -> day1 00:00:05 at RTC+1s: crossing span6s,
  residual+5s, versus linear span6s-365days. Both engines choose2024 and TF-017;
  day1 00:00:06 at RTC+2s stays2024. Existing day200->199 backward-jump
  fixture retains2023 and TF-017 residual-7s.
- Late day200 00:00:05 atRTC0.5s, observed after day199 23:59:59 atRTC1s:
  linear span-6s, residual-6.5s is nearest; the late value retains2023. A late
  Dec31 observed after Jan1 with a+5s correction instead selects the prior year.
- Known common2023 year-end->day1 at RTC+183days: allowed365day crossing span1s
  has residual183days-1s, outside182.5days; the forbidden366day hypothesis
  would give182days-1s. Offline must keep a partial with TF-020, stream must
  reject atomically. Intervening100day data makes48-bit RTC unwrap unambiguous.
- Unknown-year end->day1 at RTC+1s gets phase0->1, no absolute delta, no TF-017,
  no drift. Unknown-year+5s correction still emits TF-017 with+5s and related
  packet1 while remaining partial.

Eight isolated source mutations compiled and failed runtime assertions; each
source was restored immediately (no compile-error kills):

| Mutation | Test / failed observable |
|---|---|
| late-consistency-skipped (force late phase decrement) | `review3_late_larger_day_is_a_jump_not_a_prior_year`: signed-6.5s stream finding disappears |
| year-length-check-ignored | `review3_common_year_cannot_use_leap_hypothesis`: forbidden366day model gives a dated year instead of an offline partial |
| same-calendar-year-ignores-phase | `review3_unknown_year_rollover_has_no_delta_jump_or_drift`: invented cross-year delta emits TF-017 |
| year selection still gated at10ms | `review3_new_year_correction_*`:2023 or missing+5s TF-017 |
| always select cyclic explanation | existing backward-jump regression:2024 instead of2023 |
| omit182.5day residual bound | common-year bounded test: dated result instead of partial |
| omit partial cyclic TF-017 | partial/late control: missing signed+5s finding |
| reverse late comparison | partial/late control: late crossing loses signed+5s finding |

No whole-requirement markers added for these partial clauses. Matrix stays
160/288 (delta0); same-priority stable-first is subsequently owner-accepted2026-11-15 (see review4).
All39 focused tests and all18 bounded local CI-equivalent checks passed,
with jobs/threads2. Actionlint, diff and trace checks passed. Hosted checks are
verified at the reported head; no merge or publication.

## PR26 review4 — symmetric YEAR-PHASE policy (owner2026-11-15)

Supersedes the asymmetric review2/review3 selection rules; historical evidence
above is retained, not claimed as verification of the new algorithm.

**Owner-adopted YEAR-PHASE reader policy (2026-11-15):** For every strictly
forward RTC reference, consider phase change d in {-1, 0, +1}, independently
of ordinal-day order. The expected calendar span uses the real length of the
year crossed: +1 crosses the current year, -1 crosses the previous year;
unknown years consider both365/366day possibilities without inventing a year.
Choose the smallest absolute (expected span minus RTC elapsed) residual; a tie
keeps d=0. Equal counters keep the existing no-phase-change guard. Mirror the
calculation for late references, solving their phase relative to the retained
later point. Preserve explicit second60 in linear as well as crossing spans.
A best residual greater than182.5days must not be guessed (the exact boundary
is allowed): offline starts a new year-evidence segment, keeps partial values
with TF-020 and signed gap/context evidence; stream returns NeedsContext before
committing progress/identity, permitting explicit-boundary retry. Report TF-017
when the chosen residual strictly exceeds the jump threshold; equality is not
a finding. Threshold/level overrides never choose the phase.

Stable-first is owner-accepted2026-11-15 with the crossover condition: first
observed eligible DMY anchors the session phase and advances with this rule.
DMY2023Dec31 followed by consistent DOYJan1 resolves2024; a DMY2024Jan1 is
not contradictory to that advanced anchor. Only year evidence disagreeing with
the phase-adjusted anchor raises TF-032; retain contrary candidates. The raw
explicit year of a DMY remains in its own evidence/calendar. Offline may use
later evidence; stream uses evidence available at release and never revises.

Source distinction: contract5.3 says "A time that crosses midnight on 31 December
advances the year." Figures11-13/14 (106-24R1 printed11-25/PDF33) define the
calendar fields; "Bit 8: Indicates if this is a leap year" (§11.2.3.2a).
IRIG106 supplies no clock-jump/year-phase selection algorithm. These choices
and the182.5day bound are the owner's reader policy, not a standard mandate.


All clocks below are nominal UTC calendar labels; RTC elapsed is independent.
D=86400s; listed TF-017 residual is calendar minus RTC, at default10ms threshold.
Rows with unknown years retain None; no host date or LY inference. Every row is
exercised in offline and both stream modes, with explicit data bridging RTC wraps.

| Probe | Inputs (initial year; calendar labels; forward RTC elapsed) | Expected (year, day), phase/finding |
|---|---|---|
| A |2023;365 23:59:59 ->2 00:00:00;86401s |(2024,2), +1, no TF-017 |
| B1 / B3 |2024;366 23:59:59 ->1 /3 midnight;1s /172801s |(2025,1)/(2025,3), +1, no TF-017 |
| C |2023;200 00:00:05 ->199 23:59:59;1s |(2023,199),0, TF-017 -7s |
| D / E |2023;200 00:00:05 ->200 00:00:06;1s / equal RTC |(2023,200),0; no TF-017 / TF-017 +1s |
| F |2023;365 23:59:59 ->1 00:00:05 ->1 00:00:06;1s each |(2024,1) then(2024,1), +1 then0; TF-017 +5s then none |
| F' |same F, no year evidence |(None,1), phase+1 then0, TF-020 and TF-017 +5s; no absolute cross-phase delta/drift |
| G |2023;365 23:59:59 ->365 23:59:55;1s, then1 midnight at+5s |(2023,365) TF-017 -5s, then(2024,1) no jump |
| M |2024;1 00:00:02 ->365 23:59:59 ->1 midnight;1s each |(2023,365),-1 TF-017 -4s, then(2024,1),+1 no jump |
| M3 |2023;365 23:59:58 ->1 00:00:02 ->365 23:59:59 ->1 midnight;1s each |(2024,1) TF-017 +3s; (2023,365) TF-017 -4s; (2024,1) no jump |
| N |2023;10 midnight ->20 midnight;360D |(2024,20),+1; TF-017 +15D (inside bound) |
| P |2023;10 midnight ->300 midnight;1s |(2022,300),-1; TF-017 -(75D+1s) |
| tie |2023;10 midnight ->10 midnight;182.5D |(2023,10),0; TF-017 -182.5D, exact bound allowed |
| K |2023;365 23:59:59 ->1 midnight;182.5D+1s |(2024,1),+1; TF-017 -182.5D, exact bound allowed |
| L |same K, RTC another100ns later |offline(None,1), new segment TF-020 with gap/context; stream NeedsContext, retry boundary |
| late |2024; first1 00:00:05 atRTC1s, then late365 23:59:59 atRTC0 |late(2023,365),-1; TF-017 +5s in earlier->later orientation |
| unknown366 |no year;366 23:59:59 ->1 midnight;1s |(None,1),+1; TF-020, no TF-017; must consider366days |
| invalid366 |2023;365 23:59:59 ->366 midnight;1s |malformed/unadmitted TF-004 and LY finding where applicable; no phase repair |
| DMY crossover |DMY2023Dec31 ->DOYJan1 / DMY2024Jan1 / DMY2023Jan1, RTC1s |DOY2024; consistent2024 DMY no TF-032; same-year2023 DMY retains contrary evidence and TF-032 |


### Review4 execution and evidence

Docs-first commit `e90b0cd` records the adopted rule/case table before code.
Tests-only `72abcee` against036dd14 adds38 tests: EIGHT runtime failures,
M/M3/N/P separately offline and stream;30 controls passed, including all three
stable-first DMY crossover variants. These passing controls are not claimed
failing-first. Fixture development corrected an RTC getter and a late-reference
index before that commit; these were test setup errors, not behavior failures.

Tests-only follow-up `06c6ac0` has ELEVEN runtime failures in its full suite:
the eight M/M3/N/P engine failures, both annual_zero_residual cases, and
malformed_first_day366_cannot_become_a_year_anchor. Review5 replayed the commit's
src/years.rs and tests/s08_phase.rs with jobs/threads2:30 passed,11 failed.
The earlier record incorrectly treated malformed-first366 as a passing control
and described only the separately observed annual pair. Unknown-phase assertions
were added too, without an additional claimed runtime failure. The adopted rule bounds the BEST RESIDUAL, not RTC spacing:
a365day common-year interval with identical calendar position selects d=+1,
zero residual. The former unconditional365day guard is removed in both engines.
Existing365/400day implausible intervals still exceed the residual bound and
stay partial/rejected. Their signed gap now runs from the last usable reference,
not an intervening unusable source-None observation; the400day raw evidence
assertion is updated from400days-.5s to400days. No value/rejection assertion is
relaxed. This residual-versus-spacing distinction applies to the L3 rule/table.

Review4 focused suite was80 tests:41 new phase tests,36 prior year tests and3
existing-API probes. Each table row runs offline and both standard/interpolate
stream modes; literal cases assert calendar year/day AND exact signed TF-017
residual. Stable-first checks first-DMY Dec31 -> DOYJan1, consistent next-year
DMY with no TF-032, and contrary same-year DMY with TF-032. Unknown366 checks
phase+1 without inventing a year; linear second60, strict threshold/override,
malformed first-anchor and atomic bound rejection have dedicated controls.
No new whole-requirement marker; L3-COR-110 remains clause-level coverage.

The following nine local mutations compiled and failed assertions with source
restored each time. Correction (review5): these local variants were overclaimed
as killing all corresponding reviewer mutant families. The day366 test did NOT
kill the reviewer's unknown-year-365-only mutant, and the leap-label test killed
only the from half, not the to half. Missing coverage is added below.

| Mutant | Targeted test / failed observable |
|---|---|
| tie-picks-crossing | tie_keeps_phase:2024 instead of2023 |
| bound-inclusive (reject equality) | k_inclusive_half_year_bound: exact182.5day case becomes partial/error |
| local365-only loop restriction (not proof of reviewer P3) | unknown_leap_year_uses_366: lost phase+1 / erroneous NeedsContext under the narrower local variant; the reviewer365-only mutant survived |
| leap-label from half only | linear_span_keeps_an_explicit_leap_label: spurious TF-017 -1s; the to half remained untested |
| late-crossing-not-recorded | late_crossing_records_phase_and_signed_correction: unknown late +5s finding missing |
| tf017-crossing-inclusive | crossing_threshold_is_strict_and_override_is_respected: finding at exact5s threshold |
| tf017-crossing-ignores-threshold | same test:6s override ignored, spurious finding |
| offline-implausible-no-gap-evidence | l_outside_bound_keeps_gap_and_context_evidence: missing gap ticks/context reason |
| day366-guard-removed (allow malformed first anchor) | malformed_first_day366_cannot_become_a_year_anchor: next reference loses2023/NeedsContext |

The first365-only mutant attempt used an unimported vec macro and did not
compile; it is NOT counted. Rewritten to a fixed array, it compiled and failed
the expected runtime assertions on rerun. These nine local variations were rerun
successfully, then original production source restored; their limited reach must
not be described as proof that the surviving reviewer variants were killed. No fuzz campaign or
new visual-page inspection. All18 bounded local CI-equivalent checks pass
(jobs/threads2), including feature suites, MSRV1.85, no_std WASM±serde, CLI,
benches and all fuzz-target compile checks. Two clippy lints (nested condition,
fixture operator precedence) were corrected; completed test/MSRV checks passed
before the final equivalent layout/parenthesis fix, then clippy and all remaining
checks were rerun. Actionlint/diff/trace checks pass. Matrix160/288 (delta0).
Hosted CI is verified at the reported head. No merge/publication.


## PR26 review5 — seven surviving mutants; tests and record only

Base `ea7e3b4e9c81189a88ac45b4e5262b3ed879f115`; production bytes unchanged.
Added14 tests (offline and stream for each of seven scenarios; stream tests run
both Standard and Interpolate). All14 passed against the reviewed production
code before mutation. No failing-first behavior or production fix is claimed.
Current focused suite:94 tests =55 phase +36 prior year +3 existing-API tests.
No new Requirements marker: these are partial COR-110/finding clauses, not whole
requirement coverage. Matrix160/288 unchanged.

Source/fixture basis remains Fig11-13/14, printed11-25/PDF33, and the adopted
YEAR-PHASE reader policy/contract5.3. Hand-positioned MMDD words0x1231,0x0630,
0x0701, YYYY0x2016/0x2017 and second60 word0=0x6000 are independent of encoders.
No new visual-page check or fuzz campaign. June30 2016 second60 is a legal
calendar-label fixture for the reader's context path, not an IERS insertion
claim. The unknown-year day365 fixture intentionally does not use LY as year
or length evidence.

Executed isolated mutations (all compile, all fail runtime assertions):

| Probe / mutation | Fixture arithmetic and failed observable | Engine coverage |
|---|---|---|
| P3 unknown-year-365-only | unknown day365 23:59:59 ->day1 midnight atRTC86401s:366day hypothesis spans86401s, residual0; constraining unknown year to365days raises spurious TF-017 -86400s | offline and both stream modes |
| F8 crossed-always-current |2025 day1 ->day365 atRTC181days: previous2024 is366days, previous-year residual=-183days (same-year=+183days); both exceed182.5days. Substituting current2025's365days admits dated2024/day365 at-182days instead of offline TF-020 partial / stream NeedsContext | offline and both stream modes; raw100day bridge, gap/context and retry assertions |
| P4b drop to-end leap adjustment | late DMY2016Dec31 23:59:60 atRTC0 observed after2017Jan1 midnight atRTC1s: signed span-1s matches RTC-1s, no TF-017; mutant produces stream TF-017 -1s | both engines checked; only stream fails this mutant because offline sorts the leap first and uses the from branch |
| F20 drop to-end sign guard | DMY2016Dec31 23:59:58 ->23:59:60 atRTC+1s: nominal+2s minusRTC1s=TF-017 +1s; wrongly subtracting1s on a positive span hides the finding | offline and both stream modes |
| F19 drop from-end sign guard | DMY2016Dec31 23:59:60 ->23:59:58 atRTC+1s: nominal-2s minusRTC1s=TF-017 -3s; wrongly adding1s on a negative span changes it to-2s | offline and both stream modes |
| F21 gate manual findings on unknown year only | DMY2016Dec31 23:59:60 ->2017Jan1 00:00:05 atRTC+1s: physical label span6s, residual+5s; selected year is known but leap endpoint NeedsContext, so dropping unreadable from the gate loses TF-017 | offline and both stream modes |
| F23 record residual only when d!=0 | DMY2016Jun30 23:59:60 ->Jul1 00:00:05 atRTC+1s:6s label span, residual+5s, d=0; dropping unreadable from the record condition loses TF-017 | offline and both stream modes |

Each mutation was generated independently from the saved original source and
tested with its named scenario; production source was restored byte-for-byte
at completion. P4b's passing offline check is explicitly not a
mutant kill; its stream checks kill the missing to-end branch. Exact TF-017
residual lists assert missing, extra and wrong-sign findings, rather than merely
finding presence. F8 requires a partial, TF-020, gap/context and atomic stream
rejection with reusable identity at an explicit boundary.

### Correction: unknown-year366 LY fixture

Between tests-only06c6ac0 and reviewed ea7e3b4, wire() changed its test CSDW
for unknown-year day366 from byte1=0x00 to0x01 (LY bit8 set). DATE remains0;
no year was supplied. This removes an unrelated TF-007 warning from that fixture;
it does not infer a year or prove the365-only hypothesis mutant is killed.
The prior record omitted this test-wire change. New P3 uses day365 with no year
and LY clear; its zero residual requires the366day hypothesis independently.

All seven mutations were also rerun with the stream-mode array temporarily
restricted to Interpolate, independently of the Standard assertion that fails
first in the ordinary two-mode run. All seven Interpolate reruns compile and
fail, so both stream paths are checked rather than inferred. The original
production AND test source bytes were restored at completion; each mutant/mode
was generated independently from the saved originals.

All18 local CI-equivalent checks pass with jobs/threads2, including all feature
suites, MSRV1.85, no_std WASM±serde, CLI, benches and locked fuzz-target compile.
Actionlint/diff/trace pass; no campaign. Hosted CI is verified at the reported
head. No source,
configuration, requirement, decision, CHANGELOG or finding-catalog edits.
