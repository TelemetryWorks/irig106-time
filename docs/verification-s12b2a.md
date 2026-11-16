# S12b2a — complete policy snapshots and legacy aggregate audit

Base main5fb323a (S12b1 merged). Pre-code split: S12b2a implements full policy
snapshots and closes the six named legacy aggregate serializers; S12b2b owns
qualified answer/report snapshots, loaded replay/link checks and remaining
adapter-wide fields. These are bounded review units, not release deferrals.
The full value/policy/report schema remains an unfrozen candidate for the
owner's one review after the second part. No Review/Design fields changed.

## Clause ledger

| L2/L3 pair | This part | Still required |
|---|---|---|
| POL-103 | Complete OfflinePolicy/StreamPolicy snapshot, including every SharedSettings field, defaults version1, full conversion table, provenance and null/absent distinctions; validates on load without defaults/host clock/registry. | Every answer/report must carry or resolve this policy and retain immediate-prior policy; loaded replay S12b2b. |
| API-107 | Every policy64/128-bit field and usize uses Decimal strings; includes signed ticks/Unix, era, identities, caps and durations. Six legacy direct number-emission paths removed. | Answer/report-wide inventory and direct adapter fields in S12b2b. |
| API-110 | Checked policy factories, session/key/pivot links, table/calendar/declaration constructors, duplicate maps and unknown fields; source provenance roundtrips. | Answer/finding/reference/session graph load/replay, complete schema review; complex engine-state Deserialize remains Q3-deferred and absent. |

No whole Requirements markers added: the tests exercise these clauses, not the
complete statements. New Traces tags name the correct POL/API parent chains.
Regenerated matrix remains192/288, delta0. Runtime correlation and conversion
algorithms, finding defaults, stream queues and CLI behavior are unchanged.

## Exact configuration inventory

Offline: shared, gap interval/tolerance, late displacement, version, inferred
sessions, fixed/preferred channel, invalid/undeclared admission, lookup strategy,
midpoint tie, back-extrapolation, stop-on-loss, jump threshold, finding levels.
Stream: id, mode, embedded offline settings, hold interval/tolerance,
raw-network declaration and all three caller caps.

Shared: id, caller year, exact priority/subset, scale assumptions, IEEE/Chapter4
epoch origins/counting/source, full validated leap table/coverage, cited counter
applicability, own tolerance, authoritative/prepared/absent/empty declarations,
TFMT-default opt-in, all per-session declared-era and pivot evidence.
Declarations retain raw attributes and governing range/setup identity, RI4 and
source; unknown spelling is not relabelled. No defaults are inserted.

All wide fields on these paths are Decimal, never shared numeric Serde. Narrow
calendar/fraction/version/channel/catalog fields stay numbers. NTP source
packet IDs and signed era values are strings too. Native pivot loading checks
origin/precision/scale/coordinate consistency; AssumedZero cannot be loaded as
independent resolved evidence. Stored provenance remains a caller assertion,
not a claim of cryptographic source authenticity.

## Legacy aggregate audit and migration

Both Serialize and Deserialize removed from:

- correlation ReferencePoint (rtc), TimeJump (index/expected/actual/delta),
  RtcReset (index and both counters);
- quality TimeQuality (counts/per-channel counts, all gap/span nanoseconds);
- recording_event RecordingEvent (rtc), despite its deprecation;
- streaming StreamingRef (rtc).

Their runtime shapes remain available; no unchecked replacement loader is
introduced. Consumers can explicitly snapshot typed values/policies now and
use qualified answer/report envelopes in S12b2b. Breaking trait removal is
listed in CHANGELOG; no legacy format alias or compatibility claim is made.
Compile-fail examples enforce absence of Serialize on all six, and the two
existing engine Deserialize compile-fail guards remain.

S12b2b must additionally cover PtpTime.seconds, Format1/2Reading.consumed and
RTC/ERTC variants in IntraPacketTime/SecondaryHeaderTime. Retaining the shared
numeric derives on those paths is not Q5 compliance; global completion is NOT
claimed here. Q3 does not excuse standalone values/adapters/reports.

## Tests-first evidence

Tests-only bab89a7 precedes implementation. On main5fb323a the six new legacy
compile-fail doctests **fail at runtime** because their Serialize bounds still
compile. Both policy tests **fail to compile** because PolicySnapshot/PolicyValue
are missing; these are not counted as runtime-failing-first tests. The three
additional graph/strict-layer tests were added against the implementation.

Final dedicated suite has six policy tests, plus the six new legacy guards.
Controls cover offline/stream/default/nondefault settings; complete nested
context,4 pivot origins/evidence cases, signed pre1970, leap-labelled epoch,
raw declaration absence, full pinned table, caps and old exact identities.
Fixtures reuse RFC5905 Figure4's1970 NTP seconds2,208,988,800 (fraction0x80000000
is0.5s) and the previously reviewed S04/RDG0102016-36623:59:60 label. Schema
representation is owner Q5/ADR0010/0011 policy, not an IRIG106 JSON mandate.
No new standard page checks or leap-history claims are made.

The dependency-free Serde value-tree fixture exercises actual Serialize/
Deserialize and emits JSON, without a new runtime parser/dependency. Separate
Python JSON parsing checks full-policy output and wide string values; this is
not a browser/WASM runtime execution claim. Policy units have an independent
exact JSON golden assertion. Platform counts reject values beyond usize.

## Negative checks and bounded verification

Temporary mutations (all restored) target defaults version, unit map,
duplicate override/context keys, cap minimum, shared/stream identity,
year-source uniqueness, Civil leap epoch, NTP key/session/pivot/declaration
links, resolved-pivot assumed era/calendar, dated coordinate and each new
private DTO unknown-field guard. Runtime assertion failures are counted;
compile errors are never counted as kills. Exact executed totals and final
CI result are recorded below after checks.

No dependencies added. Serde still uses alloc+derive without serde/std;
no_std/MSRV1.85/WASM and Q3 boundaries remain. Snapshot allocations are bounded
by caller input length; parsers must bound input sizes as before. No hidden
stream buffer or automatic fetch/table update is introduced.

## Additional sentinel guard found before review

A sixth regression failed at runtime on the prototype before the fix: a
resolved NTP pivot with seconds word0/fraction1 is valid, and floors to0ns.
Changing only its raw fraction to0 left the same rounded calendar but forged
unknown/unsynchronized time as independent era evidence. The loader now rejects
both words zero, independent of era, with checked epoch arithmetic. This was a
new snapshot-load defect, not a change to the S05 converter or TF029 policy.
Removing exactly this guard makes the regression fail again. The affected
resolved/datetime pivot checks were rechecked after the fix.

Executed negative checks: **24 distinct runtime-killed mutations** (14 policy/
pivot guards plus9 private DTO unknown-field guards, then the sentinel guard).
No retained mutation, compile-error kill or whole-requirement marker. Tests also
reject unknown fields at the five other nested object layers, and an independent
unit golden prevents producer/consumer unit drift from cancelling out.

Restored final source: all18 CI-equivalent checks passed with CARGO_BUILD_JOBS=2,
RUST_TEST_THREADS=2, --locked and incremental compilation disabled: four feature
suites, three MSRV1.85 checks, clippy/fmt/rustdoc, CLI build/clippy,
no_std/WASM±serde, trace freshness, benches, all10 fuzz harnesses, cargo-deny.
Actionlint and diff checks passed. Normal-edge cargo-tree inspection confirms
serde alloc+derive without serde/std. Python json.loads independently parsed
full policy output, checked exact signed/wide strings, both distinct epochs
and all28 pinned table entries. No browser JS or machine-JSON-Schema validator
execution is claimed. Schema remains candidate; no merge/publish performed.
