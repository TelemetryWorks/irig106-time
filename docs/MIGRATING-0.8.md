# Migrating to the unreleased 0.8.0 rebuild

Target: IRIG 106-24R1, with edition-qualified legacy readings. This is a migration
plan for current main, not a release announcement. Workspace versions are still
0.7.0; do not infer availability from a proposed `version = "0.8"` dependency.
Published 0.1–0.7 are prototype releases with known answer errors (T-1..T-17).
Re-read original bytes; do not merely relabel previously computed timestamps.

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
[historical development evidence](history/development-notes-through-s17.md),
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
