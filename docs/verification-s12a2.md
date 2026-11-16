# S12a2 — owned answer replay and adapter envelopes

Base: main843ccb9, S12a1 merged. Scope: ADR-0010/0011, common ANS/POL
projection; no serde schema, CLI, inverse queries, new defaults or Review/Design
changes. The original S12a1 ledger is historical; this record closes its
common-view/typed-counter/adapter implementation items.

## Clause ledger

| L2/L3 pair | S12a2 evidence | Boundary / remaining verification |
|---|---|---|
| ANS-101 | Owned offline/stream basis keeps selection, reference(s), raw/source/year/scale/precision, full policy/qualifier/network table/era/pivot and release/correction metadata. Own/stamp/word envelopes retain actual raw/context/policy plus independent counter evidence. Replay recomputes values from these inputs after engine removal. | Serialization/roundtrip/load/schema proof is S12b. Replay uses the already selected references and release reason; it does not reconstruct a timeline or rerank channels. |
| ANS-105 | Actual first observation retained in core basis; checked signed shared SignedNanos duration in the common projection and typed native accessors. No time packets, invalid, undeclared, pre-first, wrap and negative query cases exercised. | Existing raw counter_ns fields remain compatibility API; complete public-type inventory is S14, not claimed here. |
| POL-101 | Full actual core policy and S12a1 qualification retained; adapters preserve independent explicit epochs, applicability, table/context, component bindings and severity/tolerance settings. No settings reconstructed from current defaults. | Existing per-setting engine/qualifier tests run unchanged. No exhaustive combination claim or extra policy/standard default. |
| POL-102 | Private frozen envelopes; native output consistency/session/arithmetic checked at capture. Old-policy fallback/boundary drains keep original identity, reason, progress/deadline and origin. Adapter replay uses stored inputs, not callbacks to live engines. | Versioned untrusted deserialization remains S12b; full timeline rehydration has the existing owner deferral. |

No whole Requirements markers are added for these partial verification scopes.
Historical links already mark these pairs Implemented; the matrix cannot express
that limitation. Regeneration remains **192/288 (delta0)**. No requirement was
deleted, newly deferred or marked covered by a projection-only assertion.

## Interface and limits

answer::AnswerView owns a private AnswerBasis and computed AnswerValue.
AnswerBasis::replay recomputes selected-reference arithmetic, or re-inspects raw
adapter inputs and repeats the original adapter conversion. It uses captured
normalization/table/era evidence; no host clock, new reference search, hidden
registry or output queue. Direct counter captures check the public native
result against replay; corrupt calendar/duration/session/channel/admission and
overflow cases reject. This is not the future serde/load constructor.

OfflineBasis and StreamBasis add session_origin, the actual first observed
packet key, independent of query and selected reference. Stream pending records
retain that origin across drains/boundaries. Existing counter_ns fields remain;
new native counter_duration() and common counter_duration use shared SignedNanos.
Caller struct literals must supply the actual session origin, not the query key.
SecondaryInspection now retains data_type/byte_offset even without a finding.

Counter findings are explicitly caller-selected: the core answer does not own
all recording/batch findings. with_counter_findings preserves them separately
inside an own/stamp envelope's counter basis; adapter findings remain unchanged.
Standalone words have no fabricated counter/policy ID. They retain the actual
WordContext/SecondaryPolicy, component binding, era and table snapshots used.
Legacy untagged core settings remain visibly untagged. All copies are bounded
by caller-sized input snapshots; stream queue/cap behaviour is unchanged.

## Tests-first and fixtures

Tests-only **61226e9** precedes implementation: the four initial tests fail to
compile at main843ccb9 (missing answer module and typed methods). This is not
four runtime failures. Nine further tests and extra assertions were developed
against the prototype; no retrospective failing-first claim. Final suite:13
integration tests, plus existing E1/E2/S01..S12a1 regressions in full CI.

Reused reviewed sources (source inspection, no new page-image checks claimed):
ADR-0010/0011; contract3.1/3.5/5.6/7.3; 106-24R1 §11.2.1.1g/1.3b
(11-9/11, PDF17/19), Figs11-14/17 (11-25/27, PDF33/35); Ch4 §4.7;
RFC5905 Fig4; RDG-010/012; S04 IERS snapshot and currency metadata.

Hand fixtures: DMY2017-01-01; RTC10ticks=1000ns; ERTC2050ns=20ticks+50ns;
wrapped origin2^48−2/query1 gives300ns. PTP reference1483228837TAI seconds
and +1.000010s across1s RTC yields midpoint0.500005s/correction5000ns.
An explicit20us jump threshold permits that fixture (default1ns precision would
correctly classify a10us discrepancy as a jump). A separate deliberate2s
threshold permits a−1ns absolute span across1s RTC: floor(−1/2)=−1ns, not0.
IEEE elapsed1s after2016-12-31 23:59:59 gives second60 with exactly one TF-022.
NTP era1 offset63104 gives2036-02-08; declared/pivot/assumed era evidence stays
distinguishable, zero remains TF-029, insertion ambiguity TF-030 and stale PTP
TF-021. The accepted TF-022 level is INFORMATION (an initial fixture incorrectly
expected WARNING and was corrected against the catalog, not by changing it).

Replay cases include own TF-014 with17ns tolerance/INFORMATION override,
checksum failure with independent counter retained; raw flags/partial Chapter4;
component binding and native GPS/TAI; EOF/timeout/packet cap/byte cap/mode change/
session boundary releases with old policy and actual origin. Byte-cap fixture
uses two packet slots so packet-cap precedence does not mask the byte limit.

## Negative checks

Each temporary mutation below produced a runtime assertion failure in
s12a2_answers and was restored; **18/18 killed**, no compile-failure counting:

| Mutation | Assertion subject |
|---|---|
| Duration100ns/tick→1 | Typed duration, negatives/wrap and independent counter |
| Skip native/replay consistency | Tampered public outputs rejected |
| Skip reference admission/channel checks | Rejected/cross-channel basis refused |
| Floor→truncate negative interpolation | −1/2ns remains−1ns |
| Erase supplied core findings | Exact caller-selected evidence |
| Default own context | Explicit epoch and own result retained |
| Default own policy | Tolerance, TF-014 and severity retained |
| Erase own findings | TF-012/014 raw/level evidence retained |
| Erase independent own counter | Counter origin/reading retained alongside own failure |
| Replace stamp query ID with enclosing packet | Exact captured resolver identity |
| Discard ERTC sub-tick remainder | 2050ns remains2050ns |
| Default word era context | Era1 calendar and context retained |
| Default word severity settings | TF-021/029/030 levels retained |
| Erase network conversion | Full network evidence retained |
| Erase partial Chapter4 result | Partial raw/interpretation retained |
| Drop secondary byte offset | Offset24 preserved even on valid input |
| Drop packet data type | Actual0x19 retained |
| Use query instead of stream session origin | Old drain duration/origin preserved |

## Restored-source validation

All18 local CI-equivalent checks passed, jobs2/threads2, --locked: four feature
suites, three MSRV1.85 checks, all-target clippy, fmt, warning-free rustdoc,
CLI build/clippy, no_std/WASM with/without serde, trace check, benches,
all10 fuzz targets compiled and cargo-deny. Actionlint/diff checks passed.
GitHub platform/security results are reported separately at the final PR head.

## Review at49cc8da — projection and native-capture guards

Tests-only follow-up: two added tests (15 integration tests total), plus explicit
projection assertions in the existing counter-only and own-envelope tests.
All pass on unchanged production code; no defect or new design choice found.
No whole markers added; regenerated matrix stays192/288 (delta0).

The stamp case preserves BackwardWithinOneWrap: originRTC10, raw stamp20
means query20−2^48, not Nearest's query20. Captured query/duration/order are
asserted; replay succeeds after dropping the timeline. The counter-only view
must expose NeedsContext with its actual no-admitted-reference reason. Own
projection must expose the selected17ns tolerance, including failed own decode.

One table-driven test has15 independently corrupted cases, each starting from
a native accepted control. Controls include valid offline/interpolated stream
answers and native counter-only answers. Each case changes one basis field;
A16/A17 replace one whole endpoint with another internally coherent native
reference. The stream control has zero drift (1s absolute per1s RTC). Its query
isRTC5000000, nativeTAI1483228837.5s. The alternative preceding endpoint is
RTC5000001/TAI1483228837.500000100; the alternative following endpoint is
RTC4999999/TAI1483228837.499999900. Thus arithmetic still produces the original
midpoint and zero correction when a bounds guard is removed: the generic
output-consistency check cannot mask the intended guard. No second output or
correction field is changed. This is a capture-consistency fixture, not a new
reference-selection rule or an untrusted serialized-load claim.

All13 requested survivor IDs are killed in15 independent temporary mutation
runs (A15/A18 have separately checked variants); every run fails at runtime,
not compilation. Each mutation was restored before final validation:

| ID | Mutant / assertion |
|---|---|
| A13 | Force replay order to Nearest; backward query/resolver identity test fails. |
| A14 | Erase counter-only issue; explicit NeedsContext assertion fails. |
| A20 | Erase projected tolerance; explicit17ns assertion fails. |
| A2 | Skip distance check; table admits distance1001 instead of1000. |
| A9 | Skip deferred offline-drift guard; table admits drift_corrected=true. |
| A15 | Remove position check, then distance check separately; counter-only Exact or distanceSome(0) is incorrectly admitted. |
| A18 | Remove offline reason guard, then stream reason guard separately; table admits contradictory reading/reason evidence. Both directions are in the table. |
| A19 | Skip offline policy validation; version0 incorrectly admitted. |
| A26 | Skip stream policy validation; packet-cap0 incorrectly admitted. |
| A8 | Skip following/release equivalence; counter-only releaseInterpolated without endpoints is admitted. Table also rejects a following endpoint with Standard release. |
| A16 | Skip negative-offset guard; coherent preceding endpoint lies after query. |
| A17 | Skip upper bracket bound; coherent following endpoint lies before query. |
| A25 | Skip mode guard; interpolated release accepted under Standard policy. |

Restored-source review validation: all18 local CI-equivalent checks pass again,
jobs2/threads2, --locked (four feature suites including all15 integration tests,
MSRV1.85, clippy/fmt/rustdoc, CLI, no_std/WASM, matrix, benches, all10 fuzz
targets and cargo-deny). Actionlint/diff checks pass. Production code unchanged.
