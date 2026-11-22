# irig106-time-cli

A command-line tool for inspecting IRIG 106 Chapter 10 time data. Reads a Ch10 file, extracts all time packets, builds a correlation table, and produces diagnostic outputs.

## Install

The standalone binary is now `irigtime` (formerly `ch10time`); update scripts
and invocations. The package name stays `irig106-time-cli`. The ecosystem host
can mount the same implementation as `irig106 time` by calling
`irig106_time_cli::run(&["--help"], &mut stdout, &mut stderr)`. Arguments exclude
the executable/host subcommand; the runner returns `ExitClass` without exiting.
Arguments are validated before input is opened; the shared --fail-on threshold
is now applied after output completes. Publication and release tags remain subject to the owner’s S20 decision;
`publish = false` stays in the manifest.

## From the root of irig106-time, build it with:
```sh
cargo build --manifest-path apps/irig106-time-cli/Cargo.toml
```

For a release build:
```sh
cargo build --release --manifest-path apps/irig106-time-cli/Cargo.toml
```

This produces the `irigtime` binary:
- debug: target/debug/irigtime
- release: target/release/irigtime

## You can also run it directly from the root with:
```sh
cargo run --manifest-path apps/irig106-time-cli/Cargo.toml -- --help
```

## Commands

### summary

Full file overview: packet counts, time channels, RTC range, time jump detection, and data type breakdown.

```sh
irigtime summary flight_042.ch10
```

```
================================================================
irigtime — IRIG 106 Time Analysis
================================================================

File               : flight_042.ch10
File Size          : 1,234,567,890 bytes
Total Packets      : 2,456,789
Time Packets (0x11): 3,612
Time Channels      : 2
Correlation Refs   : 3,612

RTC Range          : 0x00000098A600 → 0x0000E8D4A510
RTC Duration       : 3612.000 seconds (60.2 minutes)

Time Channels
─────────────
  Channel   3  │  Source: External      Format: IRIG-B   Date: DOY   Leap: No   Packets: 1,806
                │  First: 100:12:30:25.340000000 UTC (assumed, RDG-009)
                │  Last:  100:13:00:37.340000000 UTC (assumed, RDG-009)
  Channel   7  │  Source: External      Format: UTC      Date: DOY   Leap: No   Packets: 1,806
                │  First: 100:12:30:28.840000000 UTC (stated FMT)
                │  Last:  100:13:00:40.840000000 UTC (stated FMT)

Time Jumps         : None detected (threshold: 1 second)
```

### channels

Per-channel time source inventory as a table. UTC* means UTC by assumption
for IRIG-A/B/G (RDG-009); RTC is unknown scale. FMT4 UTC and FMT5 GPS are stated.

```sh
irigtime channels flight_042.ch10
```

### jumps

Time discontinuity detection with configurable threshold.

```sh
irigtime jumps flight_042.ch10
irigtime jumps flight_042.ch10 --threshold-ms 500
```

Flags GPS lock events, clock resets, or any point where absolute time jumps relative to what the RTC progression predicts.

### timeline

Per-packet table showing packet number, file offset, channel, data type, raw RTC, and correlated absolute time.

```sh
irigtime timeline flight_042.ch10
irigtime timeline flight_042.ch10 --limit 500
```

### csv

Export every packet's timestamp to CSV for analysis in pandas, Excel, or your data lake pipeline.

```sh
irigtime csv flight_042.ch10 --output times.csv
```

Columns: `packet_num, offset_hex, channel_id, data_type_hex, data_type_name, rtc_raw, rtc_nanos, day_of_year, hours, minutes, seconds, nanoseconds, year, month, day`

### correlate

Resolve a single RTC value against all available time channels. Useful for debugging specific packets.

```sh
irigtime correlate flight_042.ch10 0x00009896800
```

```
Resolving RTC 0x00009896800 (10000000 ticks, 1000000000 ns)

  Any channel  → 100:12:30:25.340000000
  Channel   3  → 100:12:30:25.340000000
  Channel   7  → 100:12:30:28.840000000
```

## Bounded input and report memory

Input is buffered and CSV rows are spooled to disk, without whole-file loading
or mmap. Finding samples and correlation storage are bounded, and jump/gap
statistics are accumulated online. The CI RSS gate requires true CLI peaks below 32MiB across all six commands
on both the 384MiB reset/late workload and a reset-free four-channel phase
with 400,000 F1 references, limited timeline output and CSV file spooling.
Measured true CLI peaks were 2,968–3,392KiB across both phases; an unbounded
reference-retention mutant failed at 46,692KiB. See [the measured evidence](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/verification-s16b-final-rss.md).
This workload is memory evidence, not complete qualified-joining coverage;
S17 joining remains incomplete. Publishing is still disabled until S20.

## CSV transactions

For `csv --output report.csv`, rows are spooled in the destination directory.
Exits 0 and 1 atomically replace the destination: findings are not a publication
failure. Exits 2, 3 and 4 preserve an existing file and clean up the temp.

The report temp is private (0600) while written. Before rename, existing-target
permissions are copied; new Unix files get 0666 masked by the process umask,
and new Windows files get platform defaults. Replacement creates a new inode:
other hard links keep the old contents. This copies standard file permissions,
not ownership or other inode metadata. An existing destination must also be
writable: a read-only file (including mode 444) gives exit 4 unchanged rather
than being replaced through directory permissions. Writability is checked
without truncation before staging and again before rename. Linux/macOS probes
are nonblocking and do not follow final symlinks; Unix also verifies opened-file
identity against the metadata snapshot.
Temp names combine pid/counter with a
random `RandomState` suffix; exclusive creation prevents following planted names.

Rejecting non-regular targets (symlinks, devices including `/dev/null`, pipes,
and directories) with exit 4 is intentional. Their targets remain untouched;
use stdout for streams. Without `--output`, system-temp staging is copied to
stdout after analysis succeeds, including reports with findings exit 1.
The runner flushes both host writers before file publication; rename remains
the last fallible operation. Temporary-storage/copy/permission failures are
output 4 and trigger cleanup.

## Dependencies

- `irig106-time` — time parsing and correlation (the core library)
- File I/O uses the shared safe buffered packet reader; no mmap or library I/O.

## License

Apache-2.0

## Strict arguments

All option values are separate arguments. Unknown/inapplicable/duplicate options,
missing values or extra positional arguments return2 before opening input.
`--threshold-ms` accepts unsigned decimal0..18446744073709 (default1000);
`--limit` accepts positive decimal fitting usize (default100). RTC is48-bit hex,
with an optional0x prefix. Help: `irigtime --help` or `irigtime COMMAND --help`.
Use `irigtime summary -- '-capture file.ch10'` for a dash-leading filename;
place options before `--`. Native OS paths are preserved. The library's source
is now the conventional apps/irig106-time-cli/src/lib.rs, with the thin binary in
apps/irig106-time-cli/src/bin/irigtime.rs.
The S16b memory gate has passed; publication remains disabled until S20.

## Shared irig106 exit convention

Established here by coordinator decision for later adoption by irig106-tmats.
This CLI policy is not an IRIG106 wire requirement. Codes: 0 success, 1 selected
findings, 2 usage, 3 fatal input, 4 output.

`--fail-on <level>` uses case-sensitive lowercase `error`, `warning`, `info`,
or `never`, as a separate argument. Default: `error`. Exit1 occurs if any
finding is at or above the selected level (`error > warning > info`); `never`
disables only exit1. Accept the option on every command, including help/version,
where a valid option has no effect. Unknown values and duplicates are usage2.

Precedence is usage2 first, then the first fatal input3/output4, then findings1,
then success0. Validate all usage before opening input; finish output and both
flushes before selecting a finding-only result. Diagnostics after fatal errors
are attempted where possible and cannot replace that first fatal class.
CSV temp creation and row writes interleave with input analysis. Order is usage
validation, input open, spool creation, packet reads/spool writes, destination
copy and host flushes, then file permissions/atomic rename. A missing `--output`
directory therefore gives 4 even when a readable input is structurally bad;
an unreadable input gives 3 before spool creation. Exit 1 still publishes CSV.
Other commands complete their current analysis before rendering.

Every command reports exact error/warning/info counts and at most the first
20 generated finding messages. A final `N more findings` line accounts for
omitted messages. Severity thresholds consider all findings, including an
error arriving after the sample fills. Jump/query findings count individually.
Summary retains its legacy `Errors (N)` heading; N is the total over all levels.

### Current scanner levels

| Current CLI finding | Level | Basis |
| --- | --- | --- |
| Recovered sync loss | warning | Coordinator reader policy |
| Truncated final packet or header | warning | Coordinator EOF policy |
| Trailing unsynced bytes after a stream | info | Coordinator EOF-padding policy |
| Malformed/short time data, invalid BCD digits, date/field ranges | error | TF-004 class |
| Reserved BCD bits | warning | Owner-approved TF-005; exception to the proposed generic decode-error grade |
| Invalid TS / reserved TS or NTF value | warning | Owner-approved TF-003 / TF-006 |
| Correlation needs context (NTP era or no known applicable leap offset) | warning | Coordinator context policy |
| No usable correlation reference | warning | CLI context finding |
| Reported time jump | warning | Owner-approved TF-017 |
| Other unusable correlation/query result | error | Affected answer cannot be produced |

**Flagged proposal conflict:** the proposed error level for a reserved BCD bit
conflicts with owner-approved TF-005 WARNING. We retain warning, recommending
that the existing TF level govern. Other malformed decode errors remain error.
No TF IDs or owner-approved levels change. These are current legacy scanner
messages; complete qualified findings/report integration remains S17. In
particular the old BCD parser still skips a reserved-bit body; this slice grades
its diagnostic and does not claim the qualified value-retention rebuild.

Examples: `irigtime summary flight.ch10 --fail-on warning` fails on warnings;
`--fail-on never` retains diagnostics while disabling only the finding exit.
Help/version also accept the option, e.g. `irigtime --version --fail-on info`.


## Bounded correlation and online diagnostics (S16b)

The compatibility cache retains at most 1,024 references. An isolated live
pool also caps references at 1,024, with independent 30s RTC windows per channel;
its FIFO descriptor budget is the last 1,024 classified arrivals. These are CLI resource policies, not IRIG limits.
A quiet channel can lose every reference; no anchor per channel is reserved.
The `correlate` target is known before the scan: retain its nearest bracketing
RTC brackets (at most six values per channel, including duplicate boundaries) independently of cache eviction, including late
references. Use the library's nearest/tie/checked arithmetic for the answer;
channel count bounds the retained candidates and temporary query index.
Unanswered live CSV/timeline queries report `QueryBeforeRetainedWindow` or
`NoReferencePoint`, leave the absolute field empty and count the finding.
Coordinator-approved level for both errors: **warning** (readable counter,
unavailable absolute context).
Default `--fail-on error` keeps exit 0; `--fail-on warning` makes it exit 1.
No TF register level is changed or new TF ID invented.

Jump detection compares consecutive non-backward references per channel, not a
file-wide RTC sort; reference indices are accepted-observation order. Summary
uses 1 second; `jumps` uses `--threshold-ms`; the comparison remains strictly
above the threshold. The first 20 jump details are shown with an omitted-detail
count and exact total/finding counts. Per-channel first/last/count and observed
modular-RTC gap extrema require constant state per u16 channel, not all time
packets. These compatibility diagnostics are not qualified session/scale-aware
library quality reports. A backward RTC is an out-of-order warning with channel,
both RTCs and zero-based packet index, not an invented jump magnitude. Skip that
jump/gap pair and preserve its baseline; the late reference remains accepted
for queries and the displayed Last arrival. Known old correlate targets remain answerable.


## RTC reset segments

A backward step >30 seconds is an immediate reset on that channel. Three
consecutive references behind the baseline, strictly increasing among
themselves, confirm a short reset starting at the first of those packets.
Report one reset WARNING with channel, first packet index and both RTCs,
instead of the run's out-of-order warnings. One or two late references are
reported as out-of-order when resolved or at EOF. Lookahead holds at most two
references per channel; message generation can therefore be delayed. Confirmed
runs replay against a fresh baseline, so genuine post-reset jumps remain visible.
Channel cache/query candidates are discarded at the reset; other channels stay.

Unreset channels keep answering during another channel's reset/classification.
Valid decoded time observations need no counter bridge. Data rows have no
qualified time-source association yet: after a reset or while classification
is pending, their any-channel answers report needs-context. Matching numeric
channel IDs do not establish an association. S17 TMATS joining will select the
governing time source and segment. Recorder RTC is one counter across all
channels; a real reset affects them all. No speculative values or row revisions.

Correlate labels each covered reset-channel segment. Overlapping inclusive
RTC ranges produce an ambiguity WARNING and every matching answer; never a
single silent pick. Unreset channel lines retain their existing form. Closed
answers spool to owned system temp; RAM retains active brackets and per-channel
counts, not all segments. Temp creation/writes occur after input open, during
analysis; an output failure wins over later input failure. Copy to stdout only
after analysis, then clean up on success/error. The dense-F1 memory gate has
passed; `publish = false` remains until the owner’s S20 decision.

## JSON inventory reports

Summary/channels/jumps accept `--format json` (default text), over the same report
model as text. See the [stable producer schema](https://github.com/TelemetryWorks/irig106-time/blob/main/apps/irig106-time-cli/schema/irigtime-report-v1.schema.json)
and [compatibility, units and provenance rules](https://github.com/TelemetryWorks/irig106-time/blob/main/docs/cli-report-schema.md).
The shared envelope is `telemetryworks/irigtime-report`, stable v1, revision 4. JSON is
ASCII-only; 64/128-bit quantities are decimal strings. Every defined key is
present; null meanings are field-specific. Consumers ignore unknown fields;
producer validation stays strict. The schema is frozen after review.

Findings are command-specific, structured, and top-level. `channels` does not
evaluate jumps, so its jumps/threshold are null. File-wide channel observations
can span resets; per-channel segments isolate them and are streamed from a
private disk spool. Input names stay as supplied, with a lossless platform-native
hex representation. Complete JSON accompanies exits0/1; discard output on2/3/4
(an output failure can leave partial JSON). Existing stderr diagnostics remain.
Qualified joining and remaining qualification goldens are still partial.


Timeline accepts `--format text|json` using the shared packet-row model.
JSON revision 3 adds `options.limit` and bounded `report.rows` with explicit
counter-only clocks and unresolved association. The entire input is analysed;
the row cap never hides findings. Inventory `rows` and non-timeline `limit` are
null. CSV is a separate machine format; no qualified time-source link
is inferred from channel IDs. See [the report schema](../../docs/cli-report-schema.md).


`correlate --format json` uses the same native query report as text. Stable v1
revision 4 adds `options.query_rtc` and `report.correlation`; both are null for
other commands. Query answers stream one at a time, preserving reset segments,
unresolved context and findings. Packet `rows` stays null for a counter query;
no packet identity or data/time-channel association is invented.


CSV is a distinct machine format with its own column/quoting contract, not a
JSON-envelope mode (`csv --format json` gives usage 2). Packet JSON is available
through timeline. Stale PTP keeps converted UTC with TF-021 warning, raw TAI,
actual offset/table identity and an explicit possibly-out-of-date label in the
sampled diagnostic. Exact per-level counts and --fail-on semantics still apply.
