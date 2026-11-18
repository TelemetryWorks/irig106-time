# S12b2b — exact answers and checked report replay

Base main5dfbdf1. Pre-code split into stacked review units: b1 exact answer
output + remaining adapter audit; b2 checked replay inputs/report/answer loads
and consolidated schema. This is not a release deferral: Q5 is complete only
with both, and the schema remains a candidate for the owner's one review.

## Clause ledger

| L2/L3 pair | b1 | b2 / remaining |
|---|---|---|
| API-107 | Exact serialization of the entire AnswerView graph, including shared quantities/map keys; native adapter lengths/RTC/ERTC/PTP fields fixed. | Full report roundtrip and schema inventory. |
| API-110 | Versioned answer-output with units, recomputation before capture, supported native leaf guards; no complex basis/engine Deserialize added. | Typed complete replay inputs, compare regenerated outputs on load, all adapter envelopes and schema review preparation. |
| POL-103 | Every answer retains full selected policy/context and prior qualification; all wide fields are strings. | Replay restores the actual policy/mode/progress inputs, including previous-policy drains. |

No whole Requirements markers added. Regenerated matrix192/288 (delta0).
Public engine-state Deserialize remains absent under Q3; reporting is not an
unchecked reconstruction of queues/indexes/anchors. No Review/Design changes.

## b1 evidence and migration

Tests-first d520015: five adapter tests failed at runtime on main5dfbdf1
(number tokens instead of strings). The new answer-output test failed to
compile; that first attempt also had incorrect packet/query setup, later
corrected. No claim that this was a clean missing-API-only failure. Added
loader negatives and qualified-graph assertions passed on the implementation.
Final b1 dedicated tests: six adapter + two answer-output tests.

Four temporary numeric-emission mutations (consumed counts, PTP seconds, RTC,
ERTC) each fail at runtime; no compile errors counted, mutations restored.
Native RTC loads enforce48-bit width; ERTC unsigned64; consumed counts are
platform-sized; decimal spellings/ranges use S12b1's checked Decimal parser.
PTP seconds is actually u32 in the implementation/Fig11-17 (11-28/PDF36), not
wide: it remains u32 but is quoted as requested in the audit. Values above
u32-max reject. Fraction/time conversion behavior is unchanged.

Breaking native Serde fields: PtpTime.seconds; Format1/2Reading.consumed;
IntraPacketTime.Rtc/Ertc and SecondaryHeaderTime.Ertc now emit strings.
PtpTime, RTC and ERTC supported loads accept canonical strings only. Raw derived
Format1/2Reading Deserialize is removed: a logical decoded reading must be
regenerated from bounded raw inputs, not loaded with unchecked consumed/body
metadata. Compile-fail docs enforce this. b2 supplies that validated report
path; no extra release deferral is claimed.

Exact field serialization uses one fixed token serializer so recursive
qualification evidence does not instantiate Serializer<Serializer<...>>.
All wide primitives/shared values are emitted as strings; narrow fields stay
numeric. Named numeric map keys become JSON member strings. No dependency was
added. The token tree is an internal protocol value, not an exposed unchecked
engine loader; finite input/output buffers remain caller-owned.

b1 verification: full CI equivalent recorded after completion below. No new
page-image checks: existing reviewed Fig11-17 wire width and RFC5905 Figure4
1970 coordinate are reused. JSON representation is owner Q5/ADR0010/0011,
not an IRIG106 mandate. First part alone does NOT complete Q5 or freeze schema.

b1 restored source: all18 local CI-equivalent checks pass, jobs2/threads2,
--locked, MSRV1.85, CLI, no_std/WASM±serde, benches, all10 fuzz builds, deny,
fmt/clippy/rustdoc and matrix. Source parsing/conversion/defaults unchanged.

## b2 — checked replay implementation

Tests-only44c9c17 precedes the API (two compile failures: types absent, not
runtime failures). Five final replay tests cover both clock paths, all three
adapter envelopes, full offline sessions, native findings/quality, TF021/029,
interpolation500005000ns and accepted old-policy draining. Prior qualification
is asserted by the supplied policy ID, not just its presence. The expanded
corruption/resource/leap tests and consolidated schema follow in the third
small review unit. No whole markers or release deferrals are added.

ReportSnapshot contains a complete typed input recipe plus the exact emitted
answers/findings/session records/measurements. Its loader validates schema,
defaults version and units, runs the original library paths into temporary
engines, and compares every regenerated output field to the cached report.
Unknown, duplicate, changed, dangling or out-of-range output fields cannot be
accepted as native state. JSON member order is immaterial; input array order
is preserved. Exact integer strings and finite approximate metric numbers are
separate token kinds. Additional fields on raw word/config leaves reject.

AnswerSnapshot selects a checked answer index within that self-contained
report. AnswerOutput alone is output-only: it does not claim a safe basis-only
loader without the original inputs. Policy snapshots include full tables and
contexts; no default table registry or host clock is consulted. An explicit
preparation list retains immediate-prior policy/normalization/network evidence.
All native conversion/selection/year/session behavior is delegated unchanged.

Correction (owner Option A, 2026-11-17): the original b2 loader bounded steps,
answers and raw bytes using **document-declared** limits, not independent caller
limits. That earlier "Caller limits" claim was incorrect. The revision below
adds independent caller ceilings before replay. Callers must also bound total
JSON bytes/policy graph size before parsing. Cached output nesting is capped at128. Streams explicitly
end with Finish; live pending queues/indexes/anchors are neither loaded nor
returned. This provides checked report replay, not Q3-deferred resumable engine
rehydration. The returned offline sessions are plain validated records.

The core replay implementation is covered across b1+b2: requested numeric paths
are fixed/removed; full versioned answer/timeline/report output and validated
loads retain raw/stated/assumed evidence. The full-inventory completion audit is b3. Schema review/freeze approval is NOT
claimed: the third unit prepares the owner's single review packet and expands
negative evidence. The implementation itself does not freeze the schema.

b2 restored source: all18 local CI-equivalent checks pass, jobs2/threads2,
--locked, incl MSRV1.85, CLI, no_std/WASM±serde, benches, all10 fuzz builds,
cargo-deny and matrix. Schema/corruption audit is the next stacked unit;
GitHub workflows target main, so stacked-base CI awaits retargeting.

## b3 — schema completion / negative audit

The candidate now enumerates all7 roots, every emitted value/config shape,
numeric widths, unit/epoch rules, enums/nulls/tuple order, validation boundaries
and version/migration rules. It is prepared for ONE owner review, not frozen.
Bare shared reexports retain types' Serde; all time-owned paths wrap them.

Inventory/test review found and closed additional edges before freeze:
explicit defaults_version on answer-output/answer headers; generic Scaled<T>
wide serialization (and removal of its unchecked generic Deserialize); checked
Ch4/IEEE decoded header/stamp subsecond loads; validated native year contexts;
qualified-edition and provided-year input recipes. Only serde/protocol support
changed, not native conversion/correlation/admission rules. Raw Ch4Words remain
raw diagnostic inputs. No native engine Deserialize is added. This is completion
of Q5, not a newly deferred value/load feature.

Tests-first eca827d fails at runtime on missing answer defaults metadata;
9de6666's scaled regression fails at runtime on bare u64 output. 02e60f9's new
provenance recipe test cannot compile before the variant/field exists (and its
initial legacy-source enum name required correction); no runtime-first claim.
30ce92a's decoded subsecond regression fails on accepting Ch4 microseconds10000.
The other negative/replay tests pass on their unchanged parent. Final b3 tests
also cover raw era/fraction ambiguity, immutable outputs, all resource ceilings,
selected/counter indexes, actual timeout/cap/EOF release progress and deadlines,
explicit session boundaries, unsigned max IDs and signed negative duration.

Override wins and produces no automatic TF031; opted-in G106 contradicted by
known header0x0A (106-22+) withholds SRC and raises TF031. Provided caller2023
versus recording2022 uses2023 with TF032. No new inference or finding level.
Schema text keeps unknown/unresolved qualification distinct from modern None.

Negative audit removes one guard/event at a time and runs the report suites;
only runtime assertion failures count. Every temporary mutation is restored.
One advance-event mutant initially survived because Finish also timed out the
pending answer; the strengthened test pins **release progress**, proving the
Advance event itself is executed. Exact totals/final checks follow below.

The full-stack audit also preserves explicit WordContext NTP input (including
mismatched session and zero-sentinel precedence). Its raw context is not silently
replaced by the policy's matching-session entry. a20fc46's new selector test
cannot compile before the field exists; its unchanged native behavior is tested
after adding the recipe field. Decoded Ch4/IEEE load guards and native year
context validation are separate from raw diagnostic word/context inputs.

Executed negative checks: **24 distinct report/schema/load mutations** fail at
runtime: cached-output check; report units/defaults; answer defaults/index;
step/raw/output ceilings; prior qualification; explicit Advance progress;
policy draining; dangling stamp index; nesting depth; raw network rate;
generic-wide output; session boundary; defaults emission; unit spelling;
Ch4/IEEE/year load guards; edition evidence; provided year; word NTP context.
Together with b1's four adapter numeric-emission mutations: **28 distinct kills**.
No compile errors or equivalent mutations are counted; all changes restored.
The initial Advance survivor was killed after adding progress/deadline values.

Final dedicated S12b2b suite:6 adapter +2 output +5 replay +13 audit tests (26).
No whole Requirements marker is claimed; matrix remains192/288 (delta0).
The schema inventory includes structural fields/variants plus all custom/shared
value shapes and explicit units/widths. Candidate owner review is pending;
there is no automatic freeze, Review status change, merge or publication.

Final restored stack: all18 local CI-equivalent checks pass, jobs2/threads2,
--locked/incremental disabled, including all feature suites, MSRV1.85, CLI,
no_std/WASM±serde, benches, all10 fuzz harnesses, cargo-deny, clippy/fmt/rustdoc,
trace freshness. Actionlint/diff checks pass. Python json.loads independently
parses a complete emitted report and confirms u64-max IDs remain strings in
source input, answer basis and session records, with explicit defaults versions.
This supplements actual Serde token/load tests; no browser JS or independent
machine-readable JSON-Schema validator execution is claimed. No dependencies
added; Serde still uses alloc without std. Remaining work is owner's single
schema review and separately authorized freeze, not another implementation slice.


## PR47 revision — independent caller load budgets (owner 2026-11-17)

Source: owner Option A: "the CALLER sets load limits, never the document."
This is our Q5 JSON protocol/resource policy, not an IRIG106 wire format or
standard requirement. No Review/Design/finding field changes. No new whole
Requirements markers; API-110/POL-103 remain clause-level; matrix192/288.

`ReportSnapshot::load(deserializer, &ReplayLimits)` and the corresponding
`AnswerSnapshot::load` preflight the raw report before any native replay.
Nested answer reports use the same caller budget, not an independently loaded
report or a document-controlled budget. Plain serde Deserialize routes through
`ReplayLimits::default()`. Every declared ceiling must be <= the caller's;
every actual input count must fit both. Cached answers/quality counts are
also checked before replay, then all cached content is still compared against
native regenerated results. Parsing/allocating the wire representation happens
before this preflight; callers MUST bound incoming JSON bytes separately,
including policy/table graphs and cached output. This is not a streaming JSON
allocation limit and does not deserialize/restore live engine state.

Defaults (explicit reader choices, not time-policy defaults):1024 packets,
4096 steps (including preparations, requests and quality measurements),1024
answers,4194304 raw payload/header bytes,32 quality queries,32768 packet/query
pairs. Offline answer count is requests; stream count includes every Data
observation plus requests, before execution. All packet observations count,
even unusable ones. Raw counts include Format1/2 bodies and Own/Stamp buffers.
A checked `packets * quality_queries` bound conservatively limits repeated
reference/session scans, even where actual references are fewer than packets.
Queries with no packets cost zero pairs but still count against query/step caps.
The caller may explicitly grant larger ceilings or lower them; document claims
can only lower them. Overflow rejects. Construction via `new(plan)` is trusted
caller construction and checks the plan's declared ceilings; it is not a loader.

Tests-first commit8c1d281: isolating the existing-API regression produced one
runtime failure at the unmodified loader: a report declaring usize::MAX steps
was accepted. The full new load/default API suite failed compilation because
those APIs did not exist (not claimed as runtime failures). Later expanded
coverage has six tests: all six declared budgets independently excessive,
small/large caller budgets on report and nested answer, actual packets/steps/
answers/raw/query/work excess with a poisoned RTC to prove preflight order,
cached counts, and valid offline/stream roundtrips. New-API tests are not
claimed as having run on the old API.

Four targeted mutations were individually run and killed by runtime assertions:
remove independent declared-budget comparison; remove quality-work bound;
replace the nested caller budget with default limits; skip cached-count bounds.
Production source was restored after each run. The quality mutation is killed
by the actual-size fixture, not a replay failure. The nested-default mutation
is killed by an explicitly authorized9000-step declaration with tiny content.

Validation: six targeted resource tests and the five original report tests
pass.17/18 full local CI-equivalent checks pass, including MSRV1.85,
no_std/WASM±serde, CLI, benches, all ten fuzz harness builds and trace matrix.
Cargo-deny fails on newly reported RUSTSEC-2026-0543: chrono0.4.45 unmaintained,
all versions affected, no safe upgrade. Cargo.lock/deny.toml are unchanged from
main121c412; no exception or dependency-policy change was made. This is an
external prerequisite, not claimed green. The ordinary main merge supplies a
pull_request synchronize event when pushed (retargeting alone did not run CI).


## PR48 carry-forward — owner Option A

Ordinary merge from revised PR47 carries the independent caller budgets and
six resource tests. Merge resolution preserves PR48's defaults_version1 answer
envelope validation while routing its nested raw ReportWire through the SAME
caller budget. QualifiedFormat1 payloads remain included in raw byte totals;
provided-year/network evidence paths and all prior negative tests remain.
The candidate schema now inventories all six required ReplayLimits fields,
units/defaults, preflight order, nested/default loader authority and the reader's
separate pre-parse JSON-byte obligation. This is a protocol revision while the
schema remains unfrozen, not an IRIG106 standard rule or new deferral.

The earlier b3 local all-green result above is historical. Revised stack runs
all18 checks again; cargo-deny encounters the same external chrono advisory
RUSTSEC-2026-0543 as PR47 with unchanged main lockfile/policy. No exception is
added. The current dedicated b2b test files total34 tests (6 adapters,4 output,
5 report,13 schema/load audit,6 caller-limit tests); earlier26 was historical,
before the two output-review tests and this revision. Matrix stays192/288 with
no additional whole-requirement marker.
