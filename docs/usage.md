# Usage — the 106-24R1 / upcoming 0.8.0 API

This guide describes current main, not the published prototype. Cargo versions
remain 0.7.0 until S20. Use the workspace checkout; `0.8` is not yet a registry
installation instruction. See [migration](MIGRATING-0.8.md), [CLI details](../apps/irig106-time-cli/README.md)
and the [release gate](RELEASING.md). Older examples are preserved only as
[prototype history](history/usage-before-s17-docs.md).

## 1. Workspace and features

From the virtual workspace root:

```sh
cargo run -p irig106-time --example qualified_usage
cargo run -p irig106-time --example qualified_usage --features jiff
cargo run -p irig106-time --example format2_usage
cargo run -p irig106-time-cli --bin irigtime -- --help
```

The library package is `crates/irig106-time`; the CLI package is
`apps/irig106-time-cli`. An external development project can use
`irig106-time = { path = "../irig106-time/crates/irig106-time" }`, adjusting the
path to its checkout. Disable default features for `no_std` + `alloc`; opt into
`jiff` for checked civil interop. MSRV is Rust 1.85, edition 2024. No library
serde/chrono feature or JSON API exists. Jiff does not infer UTC, year, timezone
or leap seconds. `irig106-types = "0.1"` supplies shared wire/quantity types.

## 2. Qualified readings and recording answers

[qualified_usage.rs](../crates/irig106-time/examples/qualified_usage.rs) is an
executed example of F1, native sessions, recording answers, replay, Chapter 4
partials and optional Jiff. Its first fixture is Ch11 Figures 11-12/11-13:
Data Length 10, FMT4 stated UTC, unknown year, 340 ms fraction. It supplies a
layout explicitly; a header data-type version alone is not edition evidence.

A reader supplies payload bytes and the header's Data Length, excluding filler.
Keep raw CSDW, `CalendarReading`, scale/basis and findings instead of projecting
everything into an unqualified `AbsoluteTime`. Reserved padding is reported
separately from the value. Bit6=0 selects RTC stamps even when format bits 3–2
are reserved (warning); bit 6=1/format 11 is undefined (TF-028 error), and bit 6=1
without bit 7 is an error. Secondary own time and counter-derived time remain
independent answers; ERTC ticks are 1 ns, RTC ticks 100 ns.

Feed real `PacketObservation`s in observation order to `TimelineBuilder`, with
`OfflinePolicy` and optional `SharedPolicy`; keep returned keys, timeline
findings and `AnswerView::offline`. `AnswerBasis::replay` needs no live engine.
The example deliberately enables `include_undeclared` and leaves TIMEIN false:
this is exploratory admission, not fabricated TMATS. Actual source selection
requires governing evidence. The older `TimeCorrelator` and
`StreamingTimeCorrelator` are compatibility APIs, not qualified recording paths.

For live answers use `StreamTimeline` and `StreamPolicy::new(id, BufferCaps)`.
Standard mode uses a preceding eligible reference; interpolation is opt-in.
The default hold is 1 s + 100 ms, with 0 ns gap tolerance. Caller progress,
buffer limits, mode switches and EOF explicitly release answers with their
fallback basis; never revise emitted answers or bridge sessions/discontinuities.
The channel is committed before data, using caller/inventory evidence rather
than offline ranking. There is no absolute time before a usable reference.

`SessionClock` tracks recording-wide native RTC sessions/spans under a frozen
policy. It is separate from CLI per-channel reset heuristics. `OrderedQuality`
requires already qualified, RTC-ordered references; it does not infer years or
admission. `SessionYearCursor` / `SessionYearResolver` support bounded replay:
rebuild original inputs, carrying only rejected leap labels, not other derived
malformed results. The CLI uses a private disk spool for this replay.

## 3. Calendar context, adapters and encoding

Chapter 4 uses `Ch4Words::binary(Some(dated_origin))` for nominal reconstruction,
or `binary(None)` for labelled elapsed time. BCD needs `BcdContext` with explicit
year, full day and interpretation. Historical characters-vs-bits day wording
remains unresolved and reported. Never split the binary count into a 17-bit day,
assume day 1/year/origin, or choose a nearest time packet as calendar context.

Event-body parsing belongs in irig106-decode; event numbers are setup-defined
12-bit values, not fixed started/stopped/overrun meanings. Pass caller-extracted
tags to `extracted::inspect_stamp`; see the [handoff](handoff-recording-events-to-decode.md).
Secondary/stamp/word answer adapters retain native values, independent counters,
explicit contexts and findings. `recording_event` and `Ch4BinaryTimeExt` are gone.

Use `format1::encode_format1`, `TimeF1Csdw::to_le_bytes(layout)`, BCD body
writers and Format 2 CSDW/PTP writers with checked `Result` handling. Precision
and wire-year limits can make an ordinary calendar unrepresentable. Raw
`as_raw().to_le_bytes()` is diagnostics, not validated canonical wire output.
A new complete-header encoder is deferred; do not claim existing packet flags
become valid merely because a time body can be encoded.

## Format 2 conversion

The following blocks match the packaged, executed
[format2_usage.rs](../crates/irig106-time/examples/format2_usage.rs). This is bounded
wire conversion, not declaration/channel admission. For recording integration,
use `Format2Input`/`Format2Observation` with timeline/stream adapters, retaining
capture, TS, protocol, scale and conversion basis.

```rust
use irig106_time::{
    Result,
    format2::NetworkConversion,
    network_time::{Format2Reading, LeapSecondTable, NetworkTime, decode_time_f2},
    ntp::{NtpContext, convert_ntp},
};

// Payload excludes headers; Data Length excludes trailer filler/checksum.
// Caller supplies this session's NTP evidence, never the host clock.
fn process_time_f2_packet(
    payload: &[u8],
    data_length: usize,
    ntp_context: &NtpContext,
    table: &LeapSecondTable,
) -> Result<(Format2Reading, Option<NetworkConversion>)> {
    let reading = decode_time_f2(payload, data_length)?;
    let conversion = if reading.is_usable() {
        match reading.time.map(|t| t.value) {
            Some(NetworkTime::Ntp(raw)) => Some(NetworkConversion::Ntp(convert_ntp(
                raw,
                ntp_context,
                Some(table),
            ))),
            Some(NetworkTime::Ptp(raw)) => Some(NetworkConversion::Ptp(table.convert_ptp(raw)?)),
            None => None,
        }
    } else {
        None // Invalid TS/reserved NTF is not promoted to absolute time.
    };
    Ok((reading, conversion)) // Retain raw/status and full conversion basis.
}
```

```rust
fn main() -> irig106_time::Result<()> {
    use irig106_time::{
        NtpTime, PtpTime,
        leap::PtpConversionStatus,
        ntp::{EraBasis, EraDeclaration},
    };
    let table = LeapSecondTable::builtin();
    let mut context = NtpContext::new(1); // Actual caller session.
    context.declared = Some(EraDeclaration {
        era: 0,
        id: "caller era".into(),
    });

    // Ch11 Fig11-16 / RDG-012: UTC follows NTP reckoning, not TAI offsets.
    let raw_ntp = NtpTime {
        seconds: 3_944_678_400,
        fraction: 1 << 31,
    };
    let ntp = convert_ntp(raw_ntp, &context, Some(&table));
    assert_eq!(ntp.unix_seconds().map(|s| s.get()), Some(1_735_689_600));
    assert_eq!(ntp.utc().map(|t| t.nanoseconds), Some(500_000_000));
    assert_eq!(ntp.raw(), raw_ntp);
    assert_eq!(ntp.basis().era_basis, Some(EraBasis::Declared));

    // Fig11-17: native TAI; table conversion retains leap/currency evidence.
    let raw_ptp = PtpTime {
        seconds: 1_735_689_637,
        nanoseconds: 0,
    };
    let ptp = table.convert_ptp(raw_ptp)?;
    assert_eq!(ptp.unix_seconds().map(|s| s.get()), Some(1_735_689_600));
    assert_eq!(ptp.status(), PtpConversionStatus::Covered);
    assert_eq!(ptp.raw(), raw_ptp); // Keep this and the basis, not just UTC.

    let mut payload = [0u8; 12];
    payload[0] = 1; // NTF0, TS1; seconds/fraction are little-endian words.
    payload[4..12].copy_from_slice(&raw_ntp.to_le_bytes());
    let (reading, conversion) = process_time_f2_packet(&payload, 12, &context, &table)?;
    assert!(reading.is_usable());
    assert_eq!(conversion, Some(NetworkConversion::Ntp(ntp)));
    payload[0] = 0; // Invalid TS: raw body remains, conversion withheld.
    let (reading, conversion) = process_time_f2_packet(&payload, 12, &context, &table)?;
    assert!(!reading.is_usable());
    assert!(reading.time.is_some());
    assert!(conversion.is_none());
    Ok(())
}
```

## CLI inspection and machine formats

```sh
irigtime summary flight.ch10 --format json --fail-on warning --ntp-era 0
irigtime channels flight.ch10 --format text
irigtime jumps flight.ch10 --threshold-ms 500 --format json
irigtime timeline flight.ch10 --limit 500 --format json
irigtime csv flight.ch10 --output times.csv --fail-on never
irigtime correlate flight.ch10 0x00989680 --format json --ntp-era 1
```

Options use separate values; names/levels are lowercase and case-sensitive.
`--format text|json` applies to five views, not `csv`. CSV is its own machine
format, with 19 columns appended only (never reordered/removed). Its NTF/TS
columns describe the packet, not the reference that supplied the clock. CSV
has no basis column; use findings/JSON for stale status and provenance.
`--limit` is a positive timeline row count (default 100); `--threshold-ms` is
an unsigned jumps threshold (default 1000). Duplicate/unknown/malformed options
give usage exit 2 before input is opened. Use `--` for dash-leading paths.

`--fail-on error|warning|info|never` is inclusive, default error; it is accepted
on every command. Exits:0 success,1 threshold findings,2 usage,3 fatal input,
4 output. Usage is first; the first fatal input/output failure outranks findings.
Findings do not withhold a completed CSV. CSV stages privately, atomically
replaces regular writable targets on 0/1, preserves them on 2..4, and rejects
symlinks/devices. A bad output directory can fail before input framing.

`--ntp-era N` explicitly declares an era in -14..=59, with no host-clock/pivot
guessing. Without it, nonzero NTP needs context. Only all-zero seconds+fraction
is the unsynchronised sentinel; era 0 plus a nonzero fraction can mean1900-01-01.
Era 1 extends2036–2172, not 2104. NTP fractions floor to ns. PTP stays native TAI;
UTC conversion uses the table, retaining stale last-known offsets with TF-021.
A converted stale UTC label says possibly out of date; it is not a covered value.

JSON uses the shared report model, ASCII-only strings, stable v1/revision 13,
units and decimal-string 64/128-bit quantities, tool/input/options/provenance,
per-command findings and deterministic order. Consumers ignore unknown fields
and treat unknown view enums as unsupported views. See the
[schema contract](cli-report-schema.md) and [canonical schema](../apps/irig106-time-cli/schema/irigtime-report-v1.schema.json).
JSON is not emitted on operational exits 2..4; a failing writer still exits4.

Input is buffered, with bounded caches/samples and disk-backed CSV, rows and
native year replay. Native sessions/quality are distinct from per-channel reset
segments. Summary can backdate DOY from same-session DMY; no evidence stays
partial. Reset-overlapping correlate answers are labelled by segment. Data rows
remain unresolved after resets without real source/segment association. Chapter9
provides no direct data-channel/time-channel link; no coincidence-based guess.
Full TMATS 0.1 plus a bounded time view and owner approval remain release gates.
