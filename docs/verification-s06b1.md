# S06b1 — mixed-format offline references and explicit TAI arithmetic

Base main `2524e6b4a975c039e5bd6dc2b357629382c764cc` (S06a merged).
The saved plan allows sequential offline/stream PRs if the combined work would
exceed ~1500 handwritten changed lines. Working-note split: S06b1 shared model,
normalization, offline admission/query/quality and rate evidence; S06b2 stream
adapters, commitment, normalized interpolation/barriers and raw-rate policy.
No new release deferral, owner decision, dependency or Design/Review change.

## Source and arithmetic inspection

No fresh page-image inspection is claimed. Reuse the S04/S05/S06a checked
sources and governing L2/L3; literal fixtures cite 106-24R1 Figures11-14..17,
printed11-25..27/PDF33..35; RFC5905 §6 Figure4 p14; RDG-009/010/012.

- §11.2.3.3: "will be generated at a minimum frequency of 1 hertz unless it
  is recorded at the raw network rate of the NTP or PTP frames" (11-26/PDF34).
  Both endpoints must be explicitly caller-declared raw network references to
  qualify the gap. TF-016 remains the configured policy gap; evidence field
  `standard_minimum_1hz_applies=0` prevents calling it a standard 1 Hz breach.
  Neither endpoint flag, arrival rate, Format1 nor a global hint alone suffices.
- 2017-01-02 Unix1483315200 +37 = TAI1483315237; NTP adds2208988800
  =3692304000. FMT5 native GPS00:00:18 +19 = TAI00:00:37 (RDG-009 inferred).
  Equal physical instants have equal normalized coordinates, not equal raw words.
- RTC1s / absolute1.005s => residual+5000000ns / drift+5000ppm. Coarser
  Format1 10ms suppresses a default jump; explicit1ms does not (accepted C3).
- At2017 insertion, B=1483228800; UTC23:59:59 is TAI B+35, the inserted
  second is B+36, midnight B+37. A2s bracket has0ppm, not a false -1s jump.
  A query at the inserted second yields qualified reading23:59:60; legacy
  AbsoluteTime cannot represent it. It is derived, not a new TF-022 read event.
- NTP [B,B+1) remains ambiguous: capture-point mapped value/raw/TF-030 retained,
  no unique physical bracket or displaced answer invented, even for same-UTC
  neighbours. Era/table/floor context is retained from S05, never the host clock.
- D7: native PTP TAI stays usable; its stale UTC projection keeps last offset
  and TF-021. UTC→TAI normalization using the last known offset separately
  records table/offset/currency and raises TF-021; no offset means needs context.

## Executed evidence and honest ordering

Tests-only commit `f99a961` added five integration tests. They could not compile
on main: the format2 adapter/normalization/qualified-answer APIs did not exist.
No runtime failure on main is claimed. Seven later tests/assertion expansions
cover GPS/unknown scale, insertion, capture/invalid bodies, stale normalization,
stream preflight, coarser jump/drift and undeclared policy; they are not main-first.
Test fixture refactoring preserves literal CSDW/body words, not an encoder oracle.
All **12 focused tests** pass. The old offline/stream/edition regressions pass,
including the RCC123-20 §5.6 and contract§7 literal worked-example baselines.
S05 era context/pivots are caller-supplied; this slice does not automatically
promote a timeline calendar into independent era evidence.

**35 isolated mutations** compiled and failed assertions; all sources restored:
offset sign; GPS19 lost; native scale relabelled; ambiguity normalized/shifted;
network re-normalization bypass; insertion flattened; only-one-endpoint raw flag;
invalid/undeclared engine switch ignored; engine levels ignored; capture packet,
channel, RTC, declaration or session unchecked; physical delta reversed; finer
jump threshold; gap/jump omitted; stale warning omitted/level ignored/offset zeroed;
each forwarded TF-003/004/006/008/021/026/029/030 lost; stream preflight removed;
unknown scale invented; stale warning no longer value-kept; same-UTC ambiguity
allowed into unqualified physical comparisons. Mutation evidence is
separate from missing-API tests-first evidence. This is not a fuzz campaign.

## Clause-level ledger (each suffix names both L2 and L3)

| Requirement | S06b1 coverage | Remaining |
|---|---|---|
| F2COR-101 | Native NTF/TS/body/scale/precision/capture/session and actual admission policy in reference/basis; no fake Format1 fields; shared offline ranking | Stream insertion/selection S06b2; declaration ranges S09 |
| F2COR-102 | Explicit TAI normalization using owned table; native/stated/assumed scale and offsets retained; unknown/unqualified/ambiguous arithmetic blocked | Stream bracket barriers/interpolation S06b2; Format1 explicit leap-input adapter still S10b |
| COR-101 | Offline Format2 references/indexes and legacy/normalized Format1 coexist | Governing ranges S09; no automatic year inference |
| COR-104 | Normalized residuals, mixed precision/coarser threshold, insertion continuity; unknown scales produce no false jump | Stream discontinuities S06b2; no offline drift-corrected query mode |
| COR-106 | Engine policy refreshes boundary admission/levels; invalid/undeclared opt-ins, malformed/reserved/zero rejection; previous usable reference preserved | Stream adapter S06b2; declaration comparison S09 |
| COR-108 | Offline policy gaps carry raw-network exception evidence, exact delta/related packet; every raw flag combination | Stream raw_network_rate remains inert metadata until S06b2; no stream exception claimed |
| COR-113 | Normalized mixed-scale drift, measured interval/spacing and inherited counters; ambiguous/unknown context has no drift answer | Session year inference S08; stream-related quality/basis integration S06b2 |
| STRM-101/202/203/204 | **Not implemented in S06b1.** Compile compatibility and atomic preflight reject new mixed/normalized inputs | S06b2: bounded insertion, commitment, interpolation, hold/gap/caps, prior-valid retention, normalized barriers and no revision |
| ANS-101/105 contribution | Qualified native calendar plus original reference/conversion/normalization/table basis; compatibility field absent at second60 | Full adapter/schema work remains S10/S12 |

Q-STREAM-CHANNEL Option A remains mandatory for S06b2: caller-fixed, else
TMATS external TIMEIN/TSRC=E, else lowest eligible inventory ID, undecidable
counter-only; never offline most-valid ranking, never lost-lock switching.
No emitted-answer revision, session/discontinuity bridge or stream completion
is claimed by this PR. Existing stream tests remain regressions, not new support.

Matrix **156→160/288** counts links, not completed clauses. The matrix cannot
represent these partial requirements; this ledger supplies the limits.
New source/format optionality, network/normalization fields, NetworkEvidence year
origin and qualified OfflineAnswer.reading are breaking migration points listed
in CHANGELOG. Existing callers can keep unnormalized Format1 behaviour.

Full bounded CI equivalent: feature tests, MSRV1.85, no_std WASM±serde,
clippy/fmt/rustdoc, CLI build/clippy, benches, locked all-fuzz-target compile,
trace freshness and cargo-deny; build jobs/test threads2. No merge/publish.
