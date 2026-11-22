# Migrating to the unreleased 0.8.0 rebuild

Target: IRIG 106-24R1, with edition-qualified legacy readings. This is a migration
plan for current main, not a release announcement. Workspace versions are still
0.7.0; do not infer availability from a proposed `version = "0.8"` dependency.
Published 0.1–0.7 are prototype releases with known answer errors (T-1..T-17).
Re-read original bytes; do not merely relabel previously computed timestamps.

## Silent change: the cited out-of-order default is now 1.1 seconds

**No compile error warns users of `TimeCorrelator::DEFAULT_OOO_WINDOW_NS`: its
u64 value changed from 2_000_000_000 (2s) in v0.7.0 to 1_100_000_000 (1.1s).**
The new `DEFAULT_OOO_WINDOW: NanosDuration` holds the same 1.1s value. Callers
using the constant for their tolerance/policy now get a narrower window.
Contract §5.4 / ADR-0010 cite the accepted 100ms packet-generation plus 1000ms
stream-commit late-packet allowance; this is a configurable reader default,
not a new guarantee for every recording/edition. Choose a different bound
explicitly when recording evidence requires it.

`TimeCorrelator::new()` / `Default` still store `None` (unbounded), not this
constant. The compatibility engine records/exposes its configured window;
these notes do not claim it enforces the native timeline's session policy.
Use the explicit 2s setting below to preserve a caller's former bounded setting.

## Nanosecond parameters are typed durations

Before (v0.7.0 source only, not executable against the rebuilt API):

```rust,ignore
let correlator = TimeCorrelator::with_ooo_window(Some(2_000_000_000));
let jumps = correlator.detect_time_jump(7, 1_000_000_000);
let live = StreamingTimeCorrelator::new(30_000_000_000);
```

After (inside a function returning `irig106_time::Result`):

```rust
use core::num::NonZeroUsize;
use irig106_time::{NanosDuration, StreamingTimeCorrelator, TimeCorrelator};
// Preserve an explicitly chosen old 2s window; do not inherit a changed default.
let correlator = TimeCorrelator::with_ooo_window(Some(NanosDuration::new(2_000_000_000)));
let jumps = correlator.detect_time_jump(7, NanosDuration::new(1_000_000_000));
let age = NanosDuration::new(30_000_000_000);
let age_only = StreamingTimeCorrelator::new(age);
let cap = NonZeroUsize::new(1024)
    .ok_or(irig106_time::TimeError::InvalidTimelineInput("example cap"))?;
let bounded = StreamingTimeCorrelator::with_reference_limit(age, cap);
```

`with_ooo_window` takes `Option<NanosDuration>`; `None` remains unbounded.
`detect_time_jump` takes `NanosDuration`, not a u64 named threshold_ns.
`StreamingTimeCorrelator::new` takes a typed max age; it is age-only and equal
RTCs can still grow without a count cap. `with_reference_limit` is new since
v0.7.0: it takes `(NanosDuration, NonZeroUsize)`, not an untyped age. Both
constructors share the retained-window error behavior described below.
Checked `NanosDuration::from_secs/millis/micros` return Option; handle None on
data paths. `new` above accepts an already expressed nanosecond count.

## Leap table and network conversions are typed and fallible

| v0.7.0 | Current signature / action |
| --- | --- |
| `offset_at_tai(u64) -> i32` | `offset_at_tai(TaiSeconds) -> Result<TaiUtcOffset>` |
| `offset_at_unix(u64) -> i32` | `offset_at_unix(UnixSeconds) -> Result<TaiUtcOffset>` |
| `offset_for_f1(u16, u16) -> i32` | Same date inputs, `Result<TaiUtcOffset>` output |
| `add(LeapSecondEntry) -> ()` | `add(LeapSecondEntry) -> Result<()>`; validation/conflicts can fail atomically |
| `is_near_leap_second(u64, u64) -> bool` | `(UnixSeconds, window_secs: u64) -> bool`; window stays seconds, not NanosDuration |
| `PtpTime::to_utc_seconds(i32) -> u64` | `(TaiUtcOffset) -> Result<u64>`; unsigned underflow refuses; signed helper remains available |
| `PtpTime::to_absolute(i32) -> Result<AbsoluteTime>` | Takes `TaiUtcOffset`; still deprecated, prefer `convert_ptp` |
| `NtpTime::to_unix_seconds() -> Option<u64>` | `Result<u64>`, but deprecated and refuses missing era context; use `convert_ntp` with evidence |

After (executed typed lookups and explicit-offset arithmetic):

```rust
use irig106_time::{
    LeapSecondEntry, LeapSecondTable, PtpTime, Result, TaiSeconds, TaiUtcOffset, UnixSeconds,
};
let table = LeapSecondTable::builtin();
// 2017-01-02 UTC, after the 2016 insertion: Unix1483315200; TAI+37s.
let offset = table.offset_at_tai(TaiSeconds::new(1_483_315_237))?;
let utc = UnixSeconds::new(1_483_315_200);
assert_eq!(table.offset_at_unix(utc)?, offset);
assert_eq!(table.offset_for_f1(2017, 2)?, offset);
assert!(!table.is_near_leap_second(utc, 0));
// Compile-check the checked mutator signature without inventing a new leap.
let _add: fn(&mut LeapSecondTable, LeapSecondEntry) -> Result<()> = LeapSecondTable::add;
let _typed_offset: TaiUtcOffset = offset;
let ptp = PtpTime {
    seconds: 1_483_315_237,
    nanoseconds: 0,
};
assert_eq!(ptp.to_utc_seconds(offset)?, utc.get()); // Explicit-offset arithmetic only.
let converted = table.convert_ptp(ptp)?; // Recording answer retains table/currency basis.
assert_eq!(
    converted.unix_seconds().map(|s| s.get()),
    Some(i128::from(utc.get()))
);
```

LeapSecondEntry fields `effective_unix` and `tai_utc_offset` are now
UnixSeconds and TaiUtcOffset; wrap supplied values before checked insertion.
Offset-only lookups/arithmetic do not carry full table-currency/leap basis.
Use `convert_ptp` for recording answers; empty/pre-supported tables can require
context, and stale covered-history fallback must retain its label/TF-021.
Never replace an error with a guessed 37s offset. NTP Option→Result is not a
new automatic era conversion: see the qualified [network example](usage.md#format-2-conversion).

## Public surface and build migration

| Previous surface | Current replacement / action |
| --- | --- |
| Root package, `src/`, sibling CLI | Virtual root workspace; library at `crates/irig106-time`, CLI at `apps/irig106-time-cli`; select `-p` in Cargo commands. One `[workspace.package]` version and exact internal pin, one root lock; independent excluded `fuzz/`. Benches/examples stay inside the library package (ADR-0022) |
| `ch10time` executable | `irigtime`; package stays `irig106-time-cli`. Host mounting as `irig106 time` calls `run` with OsString-compatible args excluding executable/subcommand, stdout and stderr; only main exits |
| Library `serde` feature, `snapshot`, `AnswerOutput`, Serialize/Deserialize | Removed, including `irig106-types/serde` forwarding. Use CLI JSON reports; retain typed `AnswerView` / `AnswerBasis::replay` in Rust. No library JSON loading/rehydration. Types crate's own serde feature is untouched (ADR-0019) |
| `chrono` feature/module and infallible conversions | Removed; optional off-by-default `jiff`. Use `jiff_interop::{reading_to_civil, absolute_to_civil, calendar_to_civil, civil_to_calendar}` with explicit scale/year. Result is scaled `jiff::civil::DateTime`, not an inferred UTC instant. Unknown context/second 60 refuses; no host clock/timezone. ADR-0020 / RUSTSEC-2026-0543 |
| `recording_event`, `RecordingEvent`, `RecordingEventType` and fixed meanings | Removed (T-8). Event numbers are setup-defined 12-bit values; body decoding belongs in irig106-decode. Keep raw metadata, use caller-extracted tags with `extracted::inspect_stamp`; replacement decoder availability is not claimed (ADR-0009) |
| Root/absolute `Ch4BinaryTimeExt`, including trait `to_absolute` | Removed. Use `ch4::Ch4Words::from(raw).binary(None)` for a partial or provide a dated origin. BCD needs year/full-day/interpretation context. The wrong 17-bit day split is retired, not a calendar model |
| NTP `to_absolute`, `to_unix_seconds`, `to_nanos_since_ntp_epoch`; PTP `to_absolute(offset)` | Compiler-deprecated. Use `ntp::convert_ntp` / `LeapSecondTable::convert_ptp`; retain raw/era/scale/table/leap basis. NTP legacy helpers still refuse context-free conversion |
| Body-only BCD calendar projections; old `correlation`/`streaming` engines | Retained but explicitly unqualified compatibility APIs, not recommended recording paths. Prefer `format1`, `timeline`/`stream`, shared policy and `AnswerView`; no blanket compiler-deprecation claim |
| Implicit counter-session reconstruction | New public `session::{SessionClock, SessionSpan, SessionStart, SessionUpdate}`. Supply header order/boundaries/policy; IDs and RTC wraps are explicit. Native sessions differ from CLI per-channel reset heuristics |
| Local version enum / helpers | Shared `Irig106Version`; `Pre07`/`V23`, `from_version_field`, `is_pre_ordering_guarantee`, `has_gps_time_source`, `time_source_versioned` removed. Use setup/header-specific mappings, explicit ordering/layout evidence, and free `version::supports_format_2` |
| Old shared enum matches | `TimeSource::Gps` is `Reserved(3)`, not GPS/None; `InternalRtc` becomes `InternalFromRmm`; handle `TimeFormat::None`, `TimeF1Source::Unresolved`, `PacketStandard::Unknown` and `IntraPacketTimeFormat::InvalidFlags` |
| Local signed quantities / saturating conversions | Shared SignedNanos/SignedRtcTicks/SignedUnixSeconds from types 0.1; duration `from_secs/millis/micros` and TAI conversion use checked Option. Handle absence; use signed Unix for pre-1970. `AbsoluteTime::add_nanos/sub_nanos` return Result |
| Infallible/canonical wire writers | F1 CSDW takes explicit layout and returns Result; BCD/F2 CSDW/PTP writers return Result for invalid/unrepresentable values. DOY/DMY bodies are 6/8 bytes, F2 bodies 8 bytes. Preserve raw diagnostic words separately |
| Public struct literals / exhaustive matches | Policy/reference/answer/finding envelopes gained evidence fields; migrate to constructors/Default where provided and review matches. TimeError is non_exhaustive; some new public format enums are not |
| Silent extrapolation after legacy streaming eviction | `StreamingTimeCorrelator::new` (age-only) and `with_reference_limit` return `QueryBeforeRetainedWindow` before the retained window after eviction; a quiet channel can return NoReferencePoint. Handle these errors explicitly |

See [S14 removed/retained export audit](verification-s14b.md),
[event handoff](handoff-recording-events-to-decode.md), [current examples](usage.md),
and [wire corrections](../CHANGELOG.md#unreleased). Old per-slice notes are
[historical development evidence](https://github.com/TelemetryWorks/irig106-time/blob/7c1ca0de669b585a90374d71b62b4b74be984e06/CHANGELOG.md),
including superseded intermediate APIs; they are not migration instructions.

## CLI reports and operations

Use `--format text|json` on summary/channels/jumps/timeline/correlate; CSV is
its own machine format. Stable JSON v1/revision 13 carries units, exact wide
integers as decimal strings and provenance. Consumers ignore unknown fields
and handle unknown view enums; there is no library or CLI snapshot-input API.
CSV has 19 columns, appended only; NTF/TS columns are packet facts, not clock
source facts. See [schema](cli-report-schema.md) and [commands](cli_commands.md).

`--fail-on error|warning|info|never` defaults to error. Usage2 precedes opening
input; first fatal input 3/output 4 outranks findings 1, then success 0. Findings
publish a completed CSV; operational failures preserve the destination. Nonregular
and read-only CSV targets are rejected. Header checksum/bit 7 framing is validated;
recovered sync, final tails and padding are findings after a complete stream.
`--ntp-era <N>` accepts integers -14..=59 as a separate value; without it nonzero NTP needs
context. NTP is UTC reckoning, never plus37s; (0,0) alone is the sentinel.
`--limit` bounds timeline rows; `--threshold-ms` applies to jumps.

Summary native session quality uses bounded disk-backed year replay; later DMY
can backdate earlier DOY, preserving contradictions and missing context. It is
not qualified TMATS/data-channel joining. The [release gate](RELEASING.md) and
[remaining-work note](S17-deferred-followups.md) remain; no release/tag/publish here.


## Public API migration review gate

The source audit compares against annotated v0.7.0 (tag object
`a5d128bba917fc97aa003cc7f5f53997288aa5c9`, peeled commit
`4ea0006723fc8d75925bd57b107e6dd8893be66e`). Python3.12/Git are existing CI tools;
no new installed tool or shipped dependency. Reproduce the full listing:

```sh
git fetch --no-tags --depth=1 origin 4ea0006723fc8d75925bd57b107e6dd8893be66e
python3 scripts/public_api_audit.py --report > api-v0.7.diff
python3 scripts/public_api_audit.py --check
```

CI prints that complete diff and checks this reviewed fingerprint. Public
signatures, supporting type/trait bodies and constants (including private alias
indirection), impl headers, re-exports, cfg/derive/deprecation
attributes, macros, constant initializers and dependency/feature metadata are
included. Paths are normalized across the workspace move; the new CLI library
API has no v0.7.0 library counterpart (the old CLI was binary-only). On a change,
review the listing, update the relevant migration prose, then manually update
the marker below. The script never rewrites migration notes automatically.

This is a conservative source-review gate, **not** rustdoc/semver verification:
supporting private types/constants, public items in private modules and source-path changes may
produce false positives. All source cfg branches are listed, not target-expanded;
external re-export definitions, resolved transitive dependency APIs, macro
expansion and general function-body behavior need human/compiler/trait review.
A matching marker is a review acknowledgement by convention; it cannot prove
complete prose or semantic compatibility. Runtime-only policy changes still need review; this gate
specifically includes initializers so a changed public default cannot be hidden
behind an unchanged constant name/type.

API-DIFF-SHA256: 89af02bf79124957a1c54dcc83903d9ec365a21038fbe7fa1a41af91451031bd
