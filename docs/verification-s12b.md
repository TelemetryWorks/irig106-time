# S12b1 — exact value snapshots and checked leaf loading

Base main34990ef. Pre-code split: S12b1 value/units/version primitives and
leaf-load guards; S12b2 complete policy/answer/report snapshots and remaining
legacy aggregate audit. These are review parts, not new release deferrals.
Only complex timeline-state Deserialize has owner Q3's post0.8 deferral.
The complete schema remains a candidate until review before freeze (owner Q5).

## Clause ledger

| L2/L3 pair | S12b1 coverage | Still required |
|---|---|---|
| POL-103 | Version1 value snapshots with required schema/units, exact typed values and full leap table/currency; calendar scale provenance survives roundtrip. | Full policy/default-version snapshot/delta and answer/finding/reference tables, deterministic replay from loaded report: S12b2. |
| API-107 | Decimal wrappers cover exact unsigned/signed64/128 boundaries without JS float coercion, including all shared i128 types; normal serde uses alloc, no serde/std. | Complete report-wide field inventory and portability audit, combined with S12b2's snapshots. |
| API-110 | Typed scalar domains, RTC width, calendar/scale/date/PTP validators and leap-table constructor checked on supported load; unknown/duplicate version/unit fields rejected; shared source vocabulary tested. | Policy/answer links, remaining aggregate/value loaders, full schema review and report roundtrip: S12b2. Unsupported complex engine loads remain omitted, not unchecked. |

No new whole Requirements markers: these tests cover clauses, not the complete
POL/API statements. Existing S04 markers retain their scope; the fixture now
uses string epochs. Matrix remains192/288 (delta0). No Review/Design fields,
findings, defaults, wire parsing or epoch/leap conversion policies changed.

Schema candidate: [snapshot-schema-v1.md](snapshot-schema-v1.md). IRIG106 is
silent on JSON representation; sources are owner2026-11-13 Q5/Q3, ADR-0010/0011,
L2/L3 POL-103, API-107/110 and the recorded release deferrals. Calendar/leap
fixtures reuse reviewed RDG-010 and S04 IERS boundaries; no new page-image
checks or leap-history update is claimed.

## Tests-first / executed evidence

Tests-only **3dafc56** precedes implementation. At main34990ef, four native-load
tests fail at runtime: day0 calendar, contradictory scale provenance, inconsistent
CalendarTime month/DOY and PTP nanoseconds1e9 were accepted. The two new snapshot
API tests fail to compile because the module is absent; no runtime-failing-first
claim for them. Three later snapshot tests and extra assertions were developed
against the prototype. Final dedicated suite: four native-load plus five
snapshot tests; existing S04 load-validation test remains green.

The fixture serializer/deserializer exercises actual Serde traits using a typed
value tree, with a JSON emitter; it is not a new runtime JSON parser/dependency.
Exact JSON spelling is asserted. Separately Python json.loads parsed emitted
scalar i128-min and full28-row leap JSON and verified wide fields stayed strings.
There is no floating-point stage. Tests also demonstrate bare shared SignedNanos
Serde uses a numeric token, proving why a wrapper is necessary.

Boundary fixtures: 2^53+1, u64/i64 and u128/i128 extrema, signed Unix−1;
RTC2^48−1 accepted/2^48 rejected; ERTC u64-max remains1ns units. Noncanonical
spellings, numeric tokens, wrong kind/unit/version/schema, unknown/duplicate
fields/units and positional top-level snapshots reject. Partial native-GPS
calendar retains unknown year and assumed scale; genuine2016-36623:59:60 is
kept, forbidden year/day/scale labels reject. Leap loading reuses constructor
checks for duplicate transitions and invalid currency bounds. Current shared
Reserved(3)/InternalFromRmm remain distinct from removed Gps/InternalRtc;
native GPS FMT and ExternalGps ITS remain valid.

Complex engine-state Deserialize is already absent (inspection); compile-fail
doctests enforce that for OfflineTimeline/StreamTimeline. No unchecked loader
was added and no value/report work was silently put under the Q3 deferral.

## Negative checks

All18 temporary mutations caused runtime assertion failures, never counted
compile errors, and were restored before final checks:

| Mutation | Failing subject |
|---|---|
| Decimal emits numeric token | Exact JSON strings/shared-type roundtrip |
| Skip canonical spelling | +1/01/−0/whitespace/exponent rejected |
| Skip schema check | Other schema rejected |
| Skip version check | Future version rejected |
| Skip scalar unit | ns cannot become rtc_ticks |
| Skip calendar unit | Fraction unit cannot become us |
| Skip leap unit | UTC epochs cannot become tai_s |
| Accept duplicate units | Duplicate nanoseconds key rejected |
| Permit positional top-level | Complete scalar tuple rejected |
| Skip CalendarSnapshot constructor validation | Manually invalid calendar rejected |
| Skip native calendar validation | Day/time/leap load guards |
| Skip stated-scale comparison | Contradictory Policy declaration rejected |
| Skip effective-scale comparison | Contradictory Standard effective scale rejected |
| Skip origin comparison | Unknown cannot be Standard |
| Bypass CalendarTime factory | Inconsistent month/DOY rejected |
| Bypass PTP factory | Nanoseconds1e9 rejected |
| Ignore unknown leap metadata | Unknown source field rejected |
| Serialize UnixSeconds with shared numeric serde | All leap epoch tokens stay strings |

## Resource / dependency boundary

No dependency added. Serde is the existing optional dependency, now explicitly
alloc+derive/default-features=false. The test-only protocol fixture avoids a
serde_json dependency. Full snapshots own caller-sized values; they do not add
stream queues or fetch tables. Loaders allocate only input-driven strings/maps/
vectors; callers still bound input size, not a promise of recoverable global OOM.

Restored-source validation: all18 local CI-equivalent checks passed with
jobs2/threads2 and --locked: feature suites, MSRV1.85, clippy/fmt/rustdoc,
CLI, no_std/WASM with/without serde, trace freshness, benches, all10 fuzz
harnesses compiled and cargo-deny. Actionlint/diff checks passed. Normal-edge
cargo-tree inspection shows serde alloc+derive without serde/std. New complex
state compile-fail doctests pass. No machine-schema validation or browser JS
execution is claimed; Serde token checks plus independent Python JSON parsing
establish the emitted string representation. Policy/report-wide proof remains
S12b2 and no schema was frozen by this implementation.

## Review atbb5de7c — unknown fields and golden scalar units

Tests-only update: three added tests (five native-load plus seven snapshot tests,
12 dedicated tests total) and stronger exact-output assertions in the scalar
kind table. All pass on unchanged production code; no new bug/design change.
Valid controls load before adding exactly one unknown field at each layer.
S13 is checked independently for calendar and leap-table envelopes; L1 poisons
the table payload, L3 its first entry, W1/W2 the native leaf object.

Each of the five guard-removal mutations fails at runtime: S13 (both envelope
tests fail), L1 TableWire, L3 LeapSecondEntry, W1 CalendarTimeWire and W2 PtpWire.
S14 is extended to **every scalar kind**: an independent table pins full emitted
JSON (schema/version/kind/unit/quoted decimal), not units inferred from the
producer or loader. Twelve independent wrong-unit mutations all fail, including
ERTC ns→ertc_ticks. In total17 distinct mutations were killed; S13 was repeated
after splitting its two test wrappers to prove both fail independently. No
compile-failure counting, no source mutation retained, no new whole markers;
regenerated matrix remains192/288 (delta0). Schema is still not frozen.

### Explicit S12b2 carry-forward — Q5 release blockers

Inspection confirms legacy derives still emit wide integer JSON numbers:

- correlation: ReferencePoint.rtc; TimeJump.index, expected_nanos/actual_nanos
  (u64), delta_nanos (i64); RtcReset.index and rtc_before/rtc_after.
- quality: TimeQuality.max_rtc_gap_ns (and min_rtc_gap_ns/rtc_span_ns plus
  usize counts in total_refs/channel_count/refs_per_channel).
- recording_event: RecordingEvent.rtc (deprecated does not excuse Q5 if emitted).
- streaming: StreamingRef.rtc.

These remain in the already scoped S12b2 legacy aggregate/wide-field audit.
The new versioned wrappers do not make these direct derives exact. Q5 is NOT
complete until retained serializers/loads are audited and corrected, or removed
through an authorized API transition. Q3's complex timeline-state rehydration
deferral does not exempt these scalar/report fields. No serializer fix is
included in this tests-only revision.

Restored-source review checks: all18 local CI-equivalent checks pass again,
jobs2/threads2 and --locked, including all12 dedicated tests across enabled
feature suites, MSRV1.85, clippy/fmt/rustdoc, CLI, no_std/WASM, matrix, benches,
all10 fuzz targets and cargo-deny. Actionlint/diff checks pass. Production
code unchanged; no Q5-completion or schema-freeze claim added.

## S12b2a outcome / remaining second part

[verification-s12b2a.md](verification-s12b2a.md) covers complete checked policy
snapshots and removal of both Serde traits on all six listed legacy aggregates.
The number-emission paths enumerated above are closed; the preceding sections
remain the dated S12b1/review evidence. No runtime correlation changes.

S12b2b still owns the qualified AnswerView/report family and loaded replay/link
validation, plus remaining direct adapter-wide fields (PtpTime.seconds,
Format1/2Reading.consumed, RTC/ERTC enum variants). This is an implementation
split, not an owner release deferral. Whole coverage remains192/288; Q5 and the
schema's one owner review are not claimed complete by this first part.
