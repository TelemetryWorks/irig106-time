# irig106-time-cli

A command-line tool for inspecting IRIG 106 Chapter 10 time data. Reads a Ch10 file, extracts all time packets, builds a correlation table, and produces diagnostic outputs.

## Install

The standalone binary is now `irigtime` (formerly `ch10time`); update scripts
and invocations. The package name stays `irig106-time-cli`. The ecosystem host
can mount the same implementation as `irig106 time` by calling
`irig106_time_cli::run(&["--help"], &mut stdout, &mut stderr)`. Arguments exclude
the executable/host subcommand; the runner returns `ExitClass` without exiting.
Strict argument/threshold validation follows in S16a3. No CLI publication or
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

## Interim input-memory limit (before S16)

The current CLI reads the whole file into an owned buffer. Files larger than
available memory may be refused with exit3 or killed by the OS. This is an
interim safety change only: S16 must use bounded-memory streaming (BufReader or
chunked reads), never whole-file loading, mmap or new unsafe code. No CLI publish
or release tag before that implementation and its larger-than-buffer test land.

## Dependencies

- `irig106-time` — time parsing and correlation (the core library)
- File I/O uses a safe owned byte snapshot (S15b): whole-file RAM use; no mmap/zero-copy claim.

## License

Apache-2.0
