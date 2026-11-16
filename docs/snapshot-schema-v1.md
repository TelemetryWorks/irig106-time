# Snapshot JSON v1 — value-schema review candidate

Status: candidate, **not frozen**. Owner Q5 (2026-11-13) requires a versioned
schema, units, exact decimal strings for64/128-bit quantities, validation and
raw/stated/assumed provenance. Review the complete value/policy/report family
once before freeze; S12b1 implements the value portion below, S12b2 supplies
policy/answer reports. This is reader protocol, not an IRIG106 mandate.

## Common rules

The serde feature uses alloc without enabling serde/std. Top-level snapshots
are objects with mandatory schema/version; unknown versions/schemas/fields,
duplicate fields and duplicate unit keys reject. No implicit migration, unit
conversion, host clock, defaults version or external table lookup occurs.
Policy default versions belong to the subsequent policy/report schema; raw
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

## Scope boundary and compatibility

Native CalendarReading, ScaleEvidence, CalendarTime and PtpTime now validate
on Deserialize. Native leap-table serialization also uses string epochs;
old numeric epoch input rejects. No conversion/reader policy changes.
Complex OfflineTimeline/StreamTimeline state Deserialize stays absent, per Q3.
Full policies, findings, reference/session links and answer reports remain the
next S12b review part and0.8 blockers; raw legacy diagnostic serialization is
not claimed to satisfy this versioned report contract yet. The complex-state
deferral does not excuse those value/report validation and precision checks.

S12b2 audit checklist (review atbb5de7c): legacy correlation Rtc/u64/i64 nanos
and indexes, quality max/min/span nanoseconds and counts, recording_event Rtc,
and streaming Rtc still serialize as numbers. These are Q5 release blockers;
new snapshot wrappers alone do not complete the legacy serde audit. The Q3
complex-state-load deferral does not cover those scalar/report serializers.
