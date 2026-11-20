# irig106-time-cli

A command-line tool for inspecting IRIG 106 Chapter 10 time data. Reads a Ch10 file, extracts all time packets, builds a correlation table, and produces diagnostic outputs.

## Install

The standalone binary is now `irigtime` (formerly `ch10time`); update scripts
and invocations. The package name stays `irig106-time-cli`. The ecosystem host
can mount the same implementation as `irig106 time` by calling
`irig106_time_cli::run(&["--help"], &mut stdout, &mut stderr)`. Arguments exclude
the executable/host subcommand; the runner returns `ExitClass` without exiting.
Arguments are validated before input is opened; the shared --fail-on threshold
is now applied after output completes. No CLI publication or
release tag before S16b.

## From the root of irig106-time, build it with:
```sh
cargo build --manifest-path irig106-time-cli/Cargo.toml
```

For a release build:
```sh
cargo build --release --manifest-path irig106-time-cli/Cargo.toml
```

This produces the `irigtime` binary:
- debug: target/debug/irigtime
- release: target/release/irigtime

## You can also run it directly from the root with:
```sh
cargo run --manifest-path irig106-time-cli/Cargo.toml -- --help
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
  Channel   3  │  Source: GPS           Format: IRIG-B   Date: DOY   Leap: No   Packets: 1,806
                │  First: Day 100 12:30:25.340.000
                │  Last:  Day 100 13:00:37.340.000
  Channel   7  │  Source: External      Format: UTC      Date: DOY   Leap: No   Packets: 1,806
                │  First: Day 100 12:30:28.840.000
                │  Last:  Day 100 13:00:40.840.000

Time Jumps         : None detected (threshold: 1 second)
```

### channels

Per-channel time source inventory as a table.

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

  Any channel  → Day 100 12:30:25.340.000
  Channel   3  → Day 100 12:30:25.340.000
  Channel   7  → Day 100 12:30:28.840.000
```

## Interim report-memory gate (S16b)

Input is buffered and CSV rows are spooled to disk, without whole-file loading
or mmap. Reference and finding state is still retained in memory; recordings
can exhaust that state. S16b must finish its report-state audit and qualified joining before
this interim warning and the CLI publication/release gate are removed.

## CSV transactions

For `csv --output report.csv`, rows are spooled in the destination directory.
Exits 0 and 1 atomically replace the destination: findings are not a publication
failure. Exits 2, 3 and 4 preserve an existing file and clean up the temp.

The report temp is private (0600) while written. Before rename, existing-target
permissions are copied; new Unix files get 0666 masked by the process umask,
and new Windows files get platform defaults. Replacement creates a new inode:
other hard links keep the old contents. This copies standard file permissions,
not ownership or other inode metadata. Temp names combine pid/counter with a
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
is now the conventional src/lib.rs, with the thin binary in src/bin/irigtime.rs.
The S16b memory/publication gate still applies.

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
messages; complete qualified findings/report integration remains S16b. In
particular the old BCD parser still skips a reserved-bit body; this slice grades
its diagnostic and does not claim the qualified value-retention rebuild.

Examples: `irigtime summary flight.ch10 --fail-on warning` fails on warnings;
`--fail-on never` retains diagnostics while disabling only the finding exit.
Help/version also accept the option, e.g. `irigtime --version --fail-on info`.
