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
Both engines classify year phases in observation order, so the session origin
is applied before late day-366 validity checks (Q13/Q14 follow-up below).
Offline query and quality indexes remain sorted by RTC. First/latest DMY
selection also uses observation order. A later-observed DMY with an earlier RTC cannot replace the first anchor.

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
| Q14 late day 366 |caller 2017; DOY 1 midnight at RTC 1 s, then late DOY 366 23:59:59 at RTC 0 |late (2016,366), phase -1, admitted; no TF-004/007, both engines |
| Q13 late leap label |same Q14, late seconds=60 |late year evidence 2016, phase -1; NeedsContext/TF-022 retained, no TF-004/007, both engines |
| Q5 / Q5n / Q15n |2016 Dec 31 23:59:59 ->23:59:60 ->2017 Jan 1 00:00:00 ->00:00:01, RTC steps 1 s; DOY+caller or DMY, native/normalized |physical 59->00 span 2 s, then 1 s; no TF-017/drift; observed leap contribution +1 s retained; normalized TAI must not count it twice |
| DOY leap start |caller 2016; DOY 366 23:59:60 at RTC 0 -> DOY 1 midnight at RTC +1 s, no DMY |first label retains NeedsContext/TF-022 and year evidence 2016; next (2017,1), phase +1, no TF-017; raw and qualified offline/Standard/Interpolate |
| I5 forbidden DOY leap |caller2016;365 23:59:59 ->23:59:60 ->366 midnight, RTC1s |middle TF-004 ERROR/unusable, no leap credit; readable span1s minus RTC2s gives TF-017 -1s, drift -500,000ppm offline; both stream modes agree |
| I6 forbidden DOY leap |caller2017;182 23:59:59 ->23:59:60 ->183 midnight, RTC1s |same rejection/no-credit/-1s result; unknown year still permits all four potential dates |
| N2 segment guard |real2016Dec31 leap, then2017Jan1 ->365 at+181days (>182.5day best residual), followed by DMY2017Dec31 |new segment leap contribution0; cross-segment TF-017 +183days, never +183days-1s; offline path, stream rejects gap until explicit boundary |
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


## S08 follow-up — session starts on a yearless leap label

Base main `b6ccf4dc52e79742651dfb51c13de0c752487a09` (PR26 merged).
The former year_evidence guard accepted a NeedsContext leap label only when
its BODY supplied a year. A first DOY second 60 with caller 2016 was therefore
skipped for phase tracking and offline phase-origin selection. The next DOY
day 1 stayed at phase 0/year 2016, a year early. This is a pre-existing defect,
not a new calendar or leap-table policy.

The fix qualifies a retained explicit second 60 as calendar/phase evidence,
independent of whether its year comes from DMY or caller/session context. It
keeps the NeedsContext value and all rejection filters: source None/undeclared
leap observations cannot anchor phase, and no leap label is thereby admitted
as an absolute-time reference. Unknown year still stays unknown. Raw-layout
and edition-qualified inputs use the same guard/phase function. No API,
dependency, CLI, finding level or Review/Design field changes.

Sources: 106-24R1 Figure 11-13, printed 11-25/PDF 33 (DOY has no year),
RDG-010 (explicit second 60 retained), contract 5.3 and the owner-adopted
2026-11-15 YEAR-PHASE rule. First literal CSDW 0x00000101, words
0x6000/0x2359/0x0366 represents day 366 23:59:60 with LY=1/DATE=0/SRC=1
and caller 2016. Next CSDW 1, words 0/0/1 is day 1 midnight at RTC +1 s.
Cyclic nominal span 0 plus the explicit leap second gives 1 s, residual 0,
phase +1 and year 2017. First leap remains NeedsContext with TF-022; it cannot
anchor an absolute answer before the usable midnight reference. The scenario
is added to both copies of the S08 case table. No new page inspection or host
clock use.

Tests-only commit `02ae2c6` on main reproduced SIX runtime failures: raw and
edition-qualified input independently through offline, Standard and Interpolate;
each returned 2016 instead of 2017. The unusable-observation control already
passed and is not claimed failing-first. Before committing, a missing import
and an overbroad TF-002 expectation were corrected; this fix does not redefine
before-first-time-packet finding semantics. A post-fix unknown-year control
proves phase tracking does not invent a year. Eight focused follow-up tests;
combined S08 suite 102 (prior 94 plus 8). These are partial COR-110/RDG-010
clauses with no whole-requirement marker. Matrix stays 160/288, delta 0.
CHANGELOG S08 prose spacing is corrected alongside its fix note.

Executed isolated runtime mutations (generated from saved original source,
restored at completion; no compile-error kills):

| Mutation | Failed regression / observable |
|---|---|
| require a body year again | all six regression paths return 2016 instead of 2017 |
| omit raw leap evidence | all three raw paths return 2016 |
| omit qualified leap evidence | all three qualified paths return 2016 |
| bypass NeedsContext-only rejection filter | unusable source-None control wrongly supplies phase/year 2017 |
| omit from-end leap second in expected span | all six paths raise spurious TF-017 -1 s |
| offline origin requires absolute admission | both offline paths rebase on midnight and return 2016 |

Standard and Interpolate have separate test names, so neither mode's mutation
evidence is inferred from a failing assertion in the other. The negative control
also checks undeclared input in both engines/modes with correct production code;
its mutant fails first on source None. All six mutations compile and fail runtime
assertions, with production restored afterward. All 18 local CI-equivalent checks
pass with jobs/threads 2, including feature suites, MSRV 1.85, no_std WASM ±serde,
CLI, benches and all fuzz-target compile checks. Actionlint/diff/trace pass;
no campaign. Hosted CI is verified at the reported head. No merge or publication.


## S08 follow-up — late day 366 and native UTC leap spans

Base main `58b1cf0070779c0e2ca0805b954d17cc0d1bedf9` (PR27 merged).
Scope: two pre-existing defects, S08 case rows and CHANGELOG list spacing.
S09a is not started. No dependency, CLI, Review/Design or finding-level change.

**Q14 / Q13:** offline previously stamped in RTC order before establishing the
first observed year origin. A late DOY 366 hit the caller's 2017 common-year
check before its phase could shift to 2016. Phase classification now follows
observation order, exactly as stream does; resolve applies the selected phase
and year before day validity. YEAR-PHASE selection arithmetic and nearest-query
ordering remain unchanged. Offline query/quality indexes still sort RTC.

**Q5:** valid readable references at 59 and midnight were compared directly by
nominal Gregorian subtraction, skipping the non-admitted explicit second 60
between them. That span is physically 2 s but nominally 1 s, producing TF-017
-1 s and incorrect quality/drift. The existing signed year-phase calculation
now also retains the leap-label contribution it already applies to each step.
A constant-size checked signed sum is copied into YearBasis and exposed as
`observed_leap_adjustment() -> SignedNanos`. Native UTC span differences use
that contribution. It is not a TAI-UTC offset, an inferred leap table or a new
year source. Independent year-evidence segments never transport their offsets.
Normalized TAI coordinates take precedence and are not adjusted twice; other
scales/unknown-year guards remain. Leap labels stay NeedsContext, TF-022 and
unadmitted for absolute answers. This does not implement the deferred calendar/
answer adapters or leap-table-free rendering of arbitrary UTC instants.

Sources remain 106-24R1 Figures 11-13/14, printed 11-25/PDF 33, contract 5.3
and owner 2026-11-15 YEAR-PHASE/RDG-010. RCC 200-16 §3.6 explicitly says
"excluding leap second days when a second may be added or subtracted".
The native contribution comes from the observed second 60, never a host clock
or implicit table. Normalized controls explicitly call normalize_to_tai with
the validated builtin S04 table. No fresh page inspection or fuzz campaign.

Hand-derived Fig 11-13 words: DOY 366=0x0366, HHMM=0x2359, seconds 59/60 in
word0=0x5900/0x6000; day 1 midnight=0/0/1. DMY Fig 11-14 uses MMDD
0x1231/0x0101, YYYY 0x2016/0x2017. In Q14 caller 2017 and first observed
Jan 1 at RTC 1 s fixes phase 0; late Dec 31 at RTC 0 is phase -1/year 2016,
making day 366 valid. Q13 keeps its second-60 NeedsContext label with that
same 2016 year evidence. Rows are added to both S08 case-table copies.

Tests-only `27215f9` reproduced FIVE runtime failures on main: Q13/Q14 offline
selected 2017 instead of 2016; Q5 native DOY emitted TF-017 -1 s offline,
Standard and Interpolate. Ten controls already passed and are not claimed
failing-first. These Q5n tests explicitly attach the caller's actual 2016/2017
endpoint dates before normalize_to_tai; normalized DOY and normalized DMY
already passed. This distinguishes the executed fixture from a blanket claim
that every normalized DOY variant failed. No normalization error is ignored or
leap label silently converted to AbsoluteTime.

Post-implementation controls add native DMY and a real +5 s midnight correction
in all engines/modes, plus a fourth 00:00:01 reference and typed basis checks.
No failing-first claim for those additions. Native 59->60->00 has span 2 s and
zero drift; the fourth reference proves the +1 s contribution persists. With
midnight advanced 5 s, physical span 7 s minus RTC 2 s gives +5 s, rather than
a suppressed finding. The next 1 s interval has zero drift; adjacent-pair mean
is 1,250,000 ppm. All reported TF-017 residuals in that control must equal
+5 s; it does not assert one finding per packet when different related references
supply comparisons.

Eight isolated mutations compile and fail runtime assertions, independently
generated from saved originals and restored at completion:

| Mutation | Failed observable / scope |
|---|---|
| restore RTC-order phase classification | Q13/Q14 offline loses year 2016 or marks it malformed |
| omit previous-year candidate | Q13/Q14 fail in offline and both named stream modes |
| omit native leap contribution | Q5 raises spurious TF-017 -1 s in all three paths |
| reverse native contribution | Q5 raises TF-017 -2 s in all three paths |
| discard chosen leap adjustment | Q5 loses the observed contribution and raises TF-017 -1 s |
| drop retained leap sum | fourth reference raises spurious TF-017 -1 s |
| add observed correction to normalized TAI | Q5n/Q15n raise spurious TF-017 +1 s in all three paths |
| replace native physical span with RTC | real +5 s correction quality becomes zero drift; offline fails, stream manual finding remains a passing control |

An initial last-mutant harness failed to insert a local RTC variable after
formatting and did not compile; that attempt is not a kill. The corrected
single-behavior mutation compiled and failed the offline quality assertion.
Final suite: 21 new tests, combined S08 total 123 (prior 102 plus 21).
Standard and Interpolate have separate test names. Partial clauses only: no
whole-requirement markers; matrix 160/288 unchanged. A needless-range-loop
clippy warning was corrected to an equivalent mutable iterator, then all eight
mutations and all 18 bounded local CI-equivalent checks were rerun successfully
on the final code (jobs/threads 2): feature suites, MSRV 1.85, no_std WASM ±serde,
CLI build/clippy, benches, locked all-fuzz compile, fmt/docs/trace and cargo-deny.
Actionlint/diff pass. Hosted CI is verified at the reported head. No merge or
publication.


## PR28 review — known-year DOY leap validation

Owner2026-11-15 closes RDG-010's deferred-year validation gap. Its existing
June30/December31 restriction applies when caller/anchor/phase supplies the
DOY year, as DMY already does. Leap years allow182/366, common years181/365;
unknown years retain all four potential dates. The shared CalendarReading
validation is reused; a forbidden label is TF-004 ERROR with Unusable evidence,
not an absolute/phase anchor or observed leap contribution. No finding row,
Design or Review field changes. L3-TAI-107 clarifies this timing, retaining
L2-TAI-107/L1-TAI-007 and all quoted sources.

Sources remain RCC200-16 AppendixA.2, "If required, time changes are made on
December 31 and on June 30 at 2400 hours."; §3.6 "excluding leap second days
when a second may be added or subtracted"; 106-24R1 Fig11-13/14 (printed11-25/
PDF33) for wire weighting. No fresh page inspection claimed. Hand-derived
DOY365=0x0365,182=0x0182, seconds59/60=0x5900/0x6000, HHMM=0x2359.
I5/I6 readable endpoints span1s against2s RTC: TF-017 -1s, offline drift
(-1/2)*1e6=-500,000ppm. Forbidden labels contribute0. Separate tests cover
caller context in offline/Standard/Interpolate and earlier DMY context in all
three paths; all four unknown-year dates remain eligible. A future DMY control
also verifies offline removal of a previously unqualified contribution.

Tests-first commit5c3632e: SIX actual runtime failures at fec309e (I5/I6 in
three paths); N2 passed there and is a guard test, not a failing-first defect.
An initial test draft had an incorrect severity identifier and an N2 fixture
that accidentally crossed RTC half-range without an intermediate observation;
neither is counted as runtime defect evidence. The corrected N2 inserts a data
packet at100days, so the181day gap unwraps in one session. Its2017Jan1 ->
DOY365 span is364days, RTC181days: residual+183days. A previous-year hypothesis
crosses leap2016 and gives-183days, so both exceed182.5 and start a new offline
year-evidence segment. DMY2017Dec31 anchors it; segment0 contribution1s must
not be subtracted from segment1 contribution0. Stream cannot take this path:
it rejects that residual and requires an explicit session boundary.

Commit430ae08 adds unknown-year/anchor controls and the later-DMY test. The
later-DMY test was executed against the intermediate working fix and failed:
TF-004 was present but the old1s contribution survived. It is not claimed as a
separate committed-main runtime reproduction. Earlier-DMY controls were added
after the fix and are not claimed failing-first. Offline now replays phase
tracking after newly malformed leap labels are removed, retaining original raw
fields and year candidates. Rejected labels are skipped; replay is bounded by
the finite reference count, with no hidden queue. Stream emitted answers are
never revised.

Five isolated source mutations each compiled and failed runtime assertions:

| Mutation | Observable failure |
|---|---|
| N2 remove segment equality in leap_delta | cross-segment residual becomes183days-1s, expected+183days fails |
| omit known-year leap disqualification in resolve | I5 missing TF-004/Malformed |
| allow forbidden label into phase clock | I5 stream loses real TF-017 -1s |
| bypass future-label replay | future DMY retains a forbidden1s contribution |
| reject yearless leap candidates | unknown-year four-date control produces TF-004 |

All mutant sources restored. Final s08_late_leap suite33 tests, combined S08
135 (prior123 plus12). New tests exercise partial COR-110/TAI-107/RDG-010
clauses only: no new whole-requirement markers, matrix160/288 unchanged.
Full bounded CI equivalent and hosted CI results are reported at the final
head; no merge or publish.

Final local checks:18/18 pass (jobs/threads2), including feature suites,
MSRV1.85, no_std WASM±serde, CLI build/clippy, benches, locked all-fuzz
compilation, fmt/docs/trace/cargo-deny. Initial clippy found only test-helper
precedence; explicit parentheses fixed it, clippy and checks8..18 reran green.
No arithmetic or fixture values changed. Actionlint/diff clean; all earlier
L3 Source lines and Design/Review fields compare unchanged.
