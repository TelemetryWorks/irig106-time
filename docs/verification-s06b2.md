# S06b2 — mixed-format stream integration

Base main `8bb5c39a716256620ec836988777ed97d3929689` (S06b1 merged).
Scope: the stream clauses deferred by S06b1. No new dependency, finding ID/level,
owner decision, Design/Review change or release-scoped deferral.

## Source and arithmetic evidence

Reuse the S04/S05/S06a/S06b1 checked sources; no fresh page-image inspection is
claimed. Fixtures cite106-24R1 Figures11-14..17, printed11-25..27/PDF33..35,
RDG-009/010/012, RFC5905 §6/Fig4 p14 and RCC123-20 §5.6 printed5-44/PDF78.
"will be generated at a minimum frequency of 1 hertz unless it is recorded at
the raw network rate of the NTP or PTP frames" (§11.2.3.3,11-26/PDF34).

- NTP3692304000 maps to UTC1483315200; PTP1483315237 is the same instant
  in TAI. RTC1s versus absolute1.000010s gives midpoint0.500005s and +5000ns
  correction. This fixture explicitly selects20us jump tolerance, not the default.
- Native GPS00:00:18 +19 = TAI00:00:37 (RDG-009 inferred), never UTC.
  At2017 insertion, UTC23:59:59 is TAI B+35, inserted60 is B+36, midnight B+37
  (B=1483228800). Qualified stream reading retains60; legacy time cannot.
- Gap probes cover all9 defined NTF pairs, both modes, all caller/policy rate
  declarations, gap tolerances0/100ms and spacing1s/1s+100ns/1.05s/1.1s.
  Q32 fixture input uses ceil(ns*2^32/1e9); its decoded floor is the intended ns.
  TF-016 still reports a policy gap; standard_minimum_1hz_applies=0 only if both
  actual network endpoints qualify. Format1 and noncommitted channels cannot
  gain the exemption. Rate claims/NTFs/policy IDs remain in the finding after eviction.

## Executed ordering and negative checks

Tests-only commit `e7fc2f9`: six integration tests compiled and failed on main at
its explicit S06b2 rejection. These are runtime failures, not missing-API errors.
Eight later tests/fixture expansions were added after implementation and are not
main-first. Retire only the temporary S06b1 test requiring mixed-input rejection.
Two initial fixture errors were corrected: jump residual62s, not63s; isolate
byte exhaustion with packet cap2. A later expectation was corrected to TF-024's
accepted default Warning (TF-021 was explicitly overridden to Information).

Later failing-before-fix probes found: TF-024 lost supplied byte offset300;
TF-016 needed self-contained endpoint rate provenance; and a late pre-jump
reference could replace the later segment (packet4 wrongly replaced packet2).
The last probe uses a1s late arrival inside the1100ms window, a +9s jump, then
requires the later anchor for future data and counter-only for the closed domain.
A bounded counter cut now prevents reopening; late detections retain the later
endpoint and relate the finding to the other packet, never to itself.

All14 focused stream tests pass. Seventeen existing E2 regressions and sixteen
remaining S06b1 offline tests pass, including the literal RCC/contract examples.
Final71 distinct isolated mutations compiled and failed test assertions, with
original sources restored. Two initial segment mutants survived; added checks
for premature draining and below-boundary queries kill them, plus the post-cut
insertion guard. Equivalent fixture helpers only factor capture identity/access.

Mutation groups: defaults/inventory/ID ties/future counts/noncommitted/future
references; normalization/leap/output/correction; capture atomicity/session;
rate claim source/policy/effective point/NTF evidence; every lifecycle/resource
fallback; unknown scale/jump/session cuts/late segments/source loss; no-time/early
findings; opt-ins and individual forwarding losses TF-003/004/005/006/008/021/
024/026/029/030, TF-018/019 and signed delta. This is not a fuzz campaign.

## Clause ledger — each suffix names L2 and L3

| Requirement | Stream clauses covered | Remaining / scope boundary |
|---|---|---|
| F2COR-101, COR-101/106 | Shared actual-policy admission, NTF/TS/body/capture, no fake Format1 fields, opt-in labels, raw findings/edition diagnostics | Governing declaration ranges/comparisons S09; no edition inference |
| F2COR-102 | Explicit TAI arithmetic, native UTC/TAI/GPS preserved, unknown/partial/ambiguous operands blocked; S04/S05 basis retained | Legacy unnormalized Format1 compatibility remains; use explicit normalization for physical/leap arithmetic; migration S14 |
| COR-103/107 | Mandatory Option A caller/external TIMEIN/lowest eligible ID; undecidable counter-only until choice; no counts/lost-lock switching; missing NTF SRC does not erase last reported source | Caller supplies complete inventory; TMATS adapters S09/S16 |
| COR-104/113 | Normalized residuals/correction/precision, barriers and drift endpoints; closed segments cannot reopen through late refs | Full quality summaries remain offline; schema/report aggregation S12/S17; no new aggregate stream-quality API claim |
| COR-108 | Raw-rate setting is active per observed reference; positive caller or snapshot declaration, both endpoints required; policy gaps distinct from standard applicability, levels unchanged | No frequency inferred from arrivals; frame declarations are caller evidence |
| STRM-101 | Mixed-input bounded irrevocable core, actual session keys, explicit progress/EOF, commitment before data, incremental findings | No file walking, host clock or future-count ranking; prototype streaming remains separate |
| STRM-202 | Compatible same-session/channel TAI brackets, checked floor, both endpoints/correction; derived leap reading, native GPS, ambiguity barrier | Floor accepted C1; old string label retained for compatibility; no automatic era-pivot collection |
| STRM-203 | Inclusive1s+100ms deadline, explicit progress, zero independent gap tolerance, caps/timeout/EOF fallback with rate qualification | Caller charges bytes and bounds table/pivot inputs; retained snapshot counts are capped, not a global allocator-OOM guarantee |
| STRM-204 | Immediate counter-only before usable preceding source, TF-002, source None/FMT NONE/invalid TS/zero/reserved/malformed handling, prior valid preserved, no revision | A mapped NTP value outside calendar projection cannot displace a usable anchor; canonical S05 signed result remains available |
| STRM-205/206 | Packet/byte/reference caps and labelled fallbacks; jump/reset/incompatible cuts, opt-in loss barrier, old-policy drain, no reopen/revision | Snapshot IDs/rate claims bind at observation; changes do not rewrite older endpoints |
| ANS-101, POL-101/102 contributions | Qualified reading/native basis, era/table/offset/fallback/mode/deadline/cap/progress evidence; correct finding session/RTC/offset | Complete answer adapters/policy/schema remain S10/S12; no whole-requirement promotion |

Only the complete COR-108 and STRM-204 tests carry new Requirements markers;
partial clause tests are ledger evidence only. Matrix remains160/288, with no
new requirement promoted. Human interpretation Review fields remain pending.
Full bounded CI equivalent: features/MSRV1.85/no_std WASM±serde/clippy/fmt/docs,
CLI build/clippy, benches, all-fuzz locked compile, trace and deny; jobs/threads2.
