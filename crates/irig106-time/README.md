# irig106-time

Time readers, qualified recording answers and wire encoders for **IRIG 106-24R1**.
This is the unreleased 0.8.0 rebuild; Cargo versions remain 0.7.0 until release
preparation. Published 0.1–0.7 behavior is the prototype, with the T-1..T-17
corrections described in the [migration guide](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/MIGRATING-0.8.md).

## Start with a bounded wire reading

Inside a function returning `irig106_time::Result`, decode the payload:

```rust
use irig106_time::{
    TimeF1Layout,
    format1::decode_time_f1,
    scale::{ScalePolicy, TimeScale},
};
// 106-24R1 Figures11-12/13: UTC, DOY10012:30:25.340; Data Length10.
// Layout is explicit caller evidence, not inferred from header version alone.
let payload = [0x41, 0, 0, 0, 0x34, 0x25, 0x30, 0x12, 0, 1];
let decoded = decode_time_f1(
    &payload,
    10,
    TimeF1Layout::SourceWithIts17,
    ScalePolicy::default(),
)?;
let time = decoded.time.ok_or(irig106_time::TimeError::NeedsContext {
    context: "example body",
})?;
assert_eq!(time.scale.effective(), TimeScale::Utc);
assert_eq!(time.year, None);
assert_eq!(time.nanoseconds, 340_000_000);
```

This snippet and the recording/session/Chapter 4/Jiff examples execute in
[qualified_usage.rs](https://github.com/TelemetryWorks/irig106-time/blob/main/crates/irig106-time/examples/qualified_usage.rs). Run from the workspace root:

```sh
cargo run -p irig106-time --example qualified_usage
cargo run -p irig106-time --example qualified_usage --features jiff
cargo run -p irig106-time --example format2_usage
```

For a development checkout, use a path dependency to `crates/irig106-time`.
Only after the release is published use `irig106-time = "0.8"`.
Default feature `std` can be disabled for `no_std` + `alloc`; optional,
**off-by-default** `jiff` provides checked `jiff::civil::DateTime` conversions.
There is no library `serde` or `chrono` feature, JSON loading, clock, file or
network I/O. Library normal/build dependency trees are checked on all targets.

## Choose the boundary that carries your evidence

| Task | API / retained context |
| --- | --- |
| Bounded F1 reading | `format1::decode_time_f1` with explicit layout, or `decode_qualified_time_f1` with edition evidence; keep `CalendarReading`, scale and reserved-bit findings |
| Bounded F2 reading | `network_time::decode_time_f2`; distinguish NTF and TS; native NTP is UTC, PTP is TAI |
| NTP / PTP conversion | `ntp::convert_ntp` / `LeapSecondTable::convert_ptp`; retain raw values, era/table/leap/coverage basis |
| Offline recording | `timeline::TimelineBuilder`, `OfflinePolicy` and optional `policy::SharedPolicy`; real governing declarations or an explicit undeclared-candidate override |
| Live recording | `stream::StreamTimeline`; standard mode by default; caller progress and buffer caps; interpolation is explicitly selected |
| Replayable answers | `answer::AnswerView` / `AnswerBasis::replay`; Rust data, not JSON state loading |
| Header sessions / candidate quality | `session::SessionClock`, `native_quality::OrderedQuality`; separate counters, time-source choice and year qualification |
| Caller-extracted stamps / words | `extracted::inspect_stamp` and word adapters; explicit session, flag and calendar context |
| Chapter 4 | `ch4::Ch4Words::binary` / `bcd`; absent origin/year/full-day/interpretation gives a labelled partial |
| Wire output | Checked CSDW/body encoders; handle `Result` and representability; new complete-header encoding remains deferred |

The older `correlation` and `streaming` engines remain unqualified compatibility
APIs. Their historical benchmark numbers are not performance claims for the
qualified engines or the disk-backed CLI. The old Chapter 4 trait and fixed
recording-event meanings are removed. Native GPS is not UTC; unknown year/scale
is not filled from the host. Leap labels stay explicit; Jiff civil conversion
refuses second 60, unknown year and unknown scale.

For flags, reserved padding/value findings can keep the decoded value. Bit 6
without bit 7 and undefined secondary-derived formats cannot yield that time.
Data Length bounds every reader; filler is not time data. Edition evidence
must be qualified: header data type version alone does not prove a 106-04 layout.
See the [usage guide](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/usage.md)
and [interpretations](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/INTERPRETATIONS.md).

## Inspection CLI and release status

The sibling package `irig106-time-cli` builds **`irigtime`** for testing and simple
examples. CLI-owned stable JSON is schema v1, revision 13; CSV has 19 append-only
columns. Use `--format`, `--fail-on`, explicit `--ntp-era` and timeline `--limit`
as described in the [CLI guide](https://github.com/TelemetryWorks/irig106-time/blob/main/apps/irig106-time-cli/README.md).
JSON is output, not library snapshot input. `irig106-types`' own serde feature is
unchanged. Root Cargo.toml is a virtual workspace; examples/benches belong here.

**Release gate:** full published `irig106-tmats` 0.1 plus its bounded time view,
then separate owner approval before adding any TMATS dependency. The time
library stays TMATS-independent; data/time-channel association needs real
relationship evidence. CLI publishing remains disabled pending S20. No release or
tag is made by these docs. See [RELEASING](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/RELEASING.md).

The trace matrix has 124 L1 and 144 each L2/L3 requirements: **198/288 tested,
206/288 verified**, with clause-level gaps recorded separately. These figures
are not a claim of full 0.8.0 completion. See the
[S17 ledger](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/verification-s17-completion.md).
Apache-2.0; see [LICENSE](LICENSE).
