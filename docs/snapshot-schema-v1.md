# Snapshot JSON v1 — value/policy-schema review candidate

> **Historical library JSON work — superseded owner2026-11-17.**
> [ADR-0019](adr/0019-json-belongs-to-the-cli.md) moves JSON to the CLI.
> Keep native AnswerView/replay; remove library serde in0.8.0. Evidence below
> records its original scope and does not establish current library JSON
> obligations or a frozen CLI schema. PRs47/48 are closed; branches retained.


Status: candidate, **not frozen**. Owner Q5 (2026-11-13) requires a versioned
schema, units, exact decimal strings for64/128-bit quantities, validation and
raw/stated/assumed provenance. Review the complete value/policy/report family
once before freeze; S12b1 implements values and S12b2a complete policies.
S12b2b supplies answer/report replay and completes the adapter audit. This is reader protocol, not an IRIG106 mandate.

## Common rules

The serde feature uses alloc without enabling serde/std. Top-level snapshots
are objects with mandatory schema/version; unknown versions/schemas/fields,
duplicate fields and duplicate unit keys reject. No implicit migration, unit
conversion, host clock, defaults version or external table lookup occurs.
Policy snapshots require defaults_version separately from wire version; raw
value snapshots do not select policy defaults.

All64/128-bit integer quantities are **JSON strings**, including signed shared
irig106-types values. Decimal spelling is canonical ASCII base10: `0`, a
nonzero digit followed by digits, or minus followed by a nonzero digit and
digits. No `+`, leading zero, `-0`, whitespace, exponent, fractional spelling,
JSON number or out-of-range value. Pointers/counts must not pass through floats.
Small version/date/offset fields retain their declared16/32-bit JSON numbers.
Native unwrapped shared Serialize is NOT this contract: shared i128 serializes
as a number. Use the wrappers/field adapter, not bare shared Serialize.

Canonical emitted payloads are objects. Existing validated native leaf Serde
readers also retain positional-struct input compatibility; versioned envelopes
and their units map must be objects. No unchecked native state is reconstructed.
Calendar year may be absent/null (unknown); output deterministically emits null.

## Scalar

`ScalarSnapshot` emits these fields in order:

```json
{"schema":"irig106-time/scalar","version":1,"kind":"signed_nanos","unit":"ns","value":"-170141183460469231731687303715884105728"}
```

| kind | unit | domain |
|---|---|---|
| rtc | rtc_ticks | raw unsigned48-bit,10MHz; no epoch |
| ertc | ns | unsigned64-bit,1ns/tick; no epoch |
| nanos / signed_nanos | ns | unsigned64 / signed128 duration |
| signed_ticks | rtc_ticks | signed128 unwrapped count; session supplied separately |
| unix / signed_unix | unix_s | unsigned64 / signed128,1970 UTC Unix-style coordinate |
| tai | tai_s | unsigned64,1970 TAI coordinate |
| u64 / i64 / u128 / i128 | count | dimensionless count at the named width/sign |

Value must be a string within the exact native range. RTC rejects2^48 instead
of masking it. Kinds and units must agree; no alias/default kind is inferred.
`Decimal<T>` is the underlying format wrapper, not an epoch/unit conversion.

## Calendar

`CalendarSnapshot` has schema `irig106-time/calendar`, version1, `units`, `value`.
Units must exactly equal `{year:year,day_of_year:day,hours:h,minutes:min,seconds:s,
nanoseconds:ns}`. Value is a validated CalendarReading with its complete scale
object `{stated,effective,origin}`. Gregorian/date/range and RDG-010 leap-label
rules apply on load. No year or scale is inferred by the loader.

Scale states are canonical: Standard requires equal known stated/effective;
Policy requires stated Unknown and a known effective scale; Unspecified requires
both Unknown. Contradictory combinations reject rather than being repaired.
Native GPS remains GPS. Raw UTC labels and unknown year remain distinguishable.

## Leap table

`LeapTableSnapshot` has schema `irig106-time/leap-table`, version1, `units`, `value`.
Units exactly equal `{effective_unix:unix_s,tai_utc_offset:s,supported_start:unix_s,
known_current_through:unix_s}`. Value contains the full `entries` and `metadata`.
All UnixSeconds fields, including both metadata bounds, are decimal strings.
Offsets are signed32-bit JSON numbers. Metadata retains id/source independently
of the final transition. The normal LeapSecondTable::try_new rules validate
history/order/steps/coverage; nothing is sorted, deduplicated or silently fetched.

## Policy (S12b2a)

`PolicySnapshot` has schema `irig106-time/policy`, version1,
**defaults_version:1**, units and value, all required. The unit map is exactly:

```json
{"buffer_bytes":"byte","counter":"rtc_ticks","duration":"ns","identity":"count","ntp_fraction":"2^-32 s","unix":"unix_s"}
```

Value is one externally tagged variant: `{"Offline":{...}}` or
`{"Stream":{...}}`. These are settings, not serialized engine queues/timelines.
All durations, IDs, signed/unwrapped ticks, era numbers and platform counts
are decimal strings. Counts must fit the receiving platform's usize; oversized
caps on32-bit/WASM reject instead of truncating. Channels/versions/fractions
and year/date fields remain narrow numeric values. Nested calendar/leap
snapshots carry their own exact units and checked version envelopes.

Offline fields (all materialized, even with no findings): shared,
gap_interval_ns, version, late_ns, infer_sessions, channel,
channel_preference, include_undeclared, include_invalid, strategy,
preceding_on_tie, back_extrapolate, stop_on_loss, gap_tolerance_ns,
jump_threshold_ns, finding_levels. Stream adds id, mode, offline,
hold_tolerance_ns, hold_interval_ns, raw_network_rate, caps.
Caps contains packets/bytes/references. Null is explicit; shared/channel/
jump_threshold_ns cannot be omitted to select a future default.

The shared object contains id, caller_year, year_order, scales, ieee, ch4,
table, counter_applicability, own_tolerance, declarations, use_tfmt_default,
ntp. Year order is the complete validated priority/subset, not a hidden D9
lookup. Epochs include a CalendarSnapshot origin, Continuous/Civil counting
and caller source. Table contains full transitions, identity/source and
coverage, not a live registry ID. A Civil origin at second60 rejects.

Declarations use three distinct variants: `"CallerPrepared"` (caller-prepared
declarations retained), `"Absent"` (explicit declaration-free mode), and
`{"Present":[...]}` (possibly empty setup list). Each setup preserves session,
start/end counter range, setup_packet, provenance, declared_version, ri4,
recording_year and all raw channel attributes. No G\OD or inserted TFMT default
is introduced. The setup loader validates sessions, ranges, overlaps, duplicate
channels and year bounds using the native constructors. Those constructors
sort setup ranges as before; scalar payloads and declaration spelling remain
unchanged.

The ntp array contains key/session/declared/pivots; both identities must agree.
Declared has era/id. Pivots retain session/id/seconds/fraction/scale/origin/
calendar, including signed pre1970 coordinates, exact NTP fractions, source
packet IDs, Declared versus PivotDerived and stated versus policy UTC.
Approximate and dated pivots reconstruct through their factories; resolved
pivots validate era/coordinate/calendar agreement and reject AssumedZero as
independent evidence. Provenance is a retained caller assertion, not a proof
of the file author's authenticity. No host clock is read.

Finding overrides are an array of [numeric TF ID, severity name] pairs, rather
than an integer-keyed JSON map. Duplicate IDs reject, as do unknown catalog IDs.
NTP context keys also reject duplicates; no last-write-wins data loss occurs.
Unknown fields reject at every new DTO layer; versioned top-level snapshots
and pivot objects require maps. Other nested Serde structs can accept positional
input, as do the existing native leaves; the canonical emitted JSON is objects.
This compatibility is explicit for the owner's schema review, not a claim of
validation against a machine-readable JSON Schema artifact.

Loads re-run OfflinePolicy/StreamPolicy/SharedPolicy, YearOrder, declaration,
calendar, leap-table and NTP pivot checks. Stream modes/caps/hold arithmetic,
stream/shared ID agreement, session links, nonempty provenance and unsupported
stream lookup choices reject on invalid input. Settings are never repaired or
silently filled from defaults.

## Scope boundary and compatibility

Native CalendarReading, ScaleEvidence, CalendarTime and PtpTime validate on
Deserialize (S12b1). Native leap-table serialization uses string epochs; old
numeric epoch input rejects. Native ScalePolicy now rejects unknown fields.
Complex OfflineTimeline/StreamTimeline state Deserialize stays absent, per Q3.

S12b2a removes both direct Serde traits from six legacy aggregate types:
correlation ReferencePoint/TimeJump/RtcReset, quality TimeQuality,
recording_event RecordingEvent and streaming StreamingRef. This closes the
listed legacy number-emission paths without inventing an unchecked loader.
Their runtime APIs remain unchanged. Consumers must explicitly construct
versioned values/policies now; the replacement qualified report family arrives
in S12b2b. There is no silent replacement legacy wire shape.

Answer/report finding and reference/session links, loaded replay validation,
and the remaining direct adapter serializers are still0.8 blockers in S12b2b.
In particular, audit PtpTime.seconds, Format1/2Reading.consumed, and the shared
RTC/ERTC variants of IntraPacketTime/SecondaryHeaderTime there. They are not
exempted by Q3. This part does not claim global Q5 completion.

## One-review preparation checklist (not a freeze)

The value and complete-policy candidate is documented and emitted by tests.
Before requesting the owner's one review, S12b2b must supply answer/report
shapes, exact wide-field inventory, load-time link checks and deterministic
loaded replay controls, then consolidate the entire family here. Review the
schema/version/defaults-version separation, unit spellings, null/absent states,
platform-count portability, old-policy provenance and unknown-field/version
rules together. No schema freeze or owner approval is recorded by either code
or this document.
