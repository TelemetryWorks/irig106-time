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
