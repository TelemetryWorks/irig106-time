# CLI Commands

| Command                                    | Description                                             |
|--------------------------------------------|---------------------------------------------------------|
| `irigtime summary <file>`                  | Packet counts, time channels, RTC range, jump detection |
| `irigtime channels <file>`                 | Per-channel time source inventory                       |
| `irigtime jumps <file> [--threshold-ms N]` | Discontinuity detection                                 |
| `irigtime timeline <file> [--limit N]`     | Per-packet RTC + resolved absolute time                 |
| `irigtime csv <file> [--output path]`      | Full timestamp export                                   |
| `irigtime correlate <file> <rtc_hex>`      | Resolve one RTC against all channels                    |

Text clocks use `DDD:hh:mm:ss.fffffffff` (with a year prefix when known),
including correlate and reset segments. IRIG-A/B/G are `UTC (assumed, RDG-009)`
with Scale `UTC*`; RTC is unknown scale, FMT4 UTC and FMT5 GPS are stated.

## Shared irig106 exit convention

Coordinator decision: irig106-time establishes this convention; irig106-tmats
will adopt it. IRIG106 does not prescribe tool arguments or exit codes.

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
Correlate lazily stages closed-segment rows in owned system temp during analysis;
temp creation/writes can give output4 before later framing input3. It stops
before further input after a spool error; rendering/copy follows successful
analysis. Other commands complete analysis before rendering.

The correlation cache retains at most 1,024 references globally and 30 seconds
of RTC history. A quiet channel may be fully evicted. Unavailable live-window
answers produce findings, not invented extrapolations. Coordinator-approved
level for QueryBeforeRetainedWindow / NoReferencePoint: warning; counter data
remains readable. No new TF ID. The correlate target is known before scanning:
retain its nearest RTC brackets per channel (at most six values/channel), so old RTCs
remain answerable without retaining a whole-file index.
CSV/timeline leave the corresponding absolute field empty. Summary/jumps count
all non-backward-reference jumps online, display the first 20 details and an
omitted count. Indices are accepted-observation order, not file-wide RTC sort.
First/last/count and gap extrema are fixed-size per-channel statistics.
Backward RTCs emit an out-of-order warning with channel, previous/current RTCs
and zero-based packet index; skip the jump/gap update for that pair, retaining
the late reference for correlation. Jump/gap pairs retain their last non-backward
baseline; Last remains the latest arrival.

Finding output retains the first 20 generated messages with exact per-level
counts and an `N more findings` line. Sampling never changes --fail-on.
Existing read-only/non-writable CSV destinations give 4 unchanged; rename must
not silently bypass write permissions. This check also precedes packet framing.

| Code | Meaning |
| --- | --- |
| 0 | Success; no findings at/above the threshold |
| 1 | Findings meet the selected threshold |
| 2 | Usage error |
| 3 | Fatal unreadable/structurally invalid input |
| 4 | Output write/flush failure |

See the CLI README for current scanner severity levels. Physical bounds/no
established sync are input failures; contained decode issues, recovered sync,
cut final packets/headers and padding remain findings. --output --limit still
means an output filename of --limit; summary file --help still gives2.


Reset-qualified correlate output labels each matching segment on a reset
channel and warns once when inclusive ranges overlap. Unreset-channel lines
stay unchanged. Explicit time-channel answers are isolated; a pending late run does not suppress
another's stated time observations. Data rows after a reset or during unresolved
classification still require a qualified source/segment association (S17); their
any-channel answers remain needs-context. Numeric channel equality is not enough.


Summary/channels/jumps also accept `--format text|json` (default text). The
[stable shared JSON envelope](cli-report-schema.md) records effective options,
command-specific structured findings, exact wide values and reset segments.
`channels` does not evaluate jumps; its jumps/threshold are null. JSON stdout is
ASCII-only. Names remain as given. The producer schema is strict and consumers
ignore unknown fields within a supported schema version. The schema is frozen as
stable v1, revision 3. Discard output on operational failure (an output error may leave a
partial JSON stream).

Packet CSV column units, quoting and absent values: [CSV column schema](cli-csv-schema.md).


Timeline accepts `--format text|json` using the shared packet-row model.
JSON revision 3 adds `options.limit` and bounded `report.rows` with explicit
counter-only clocks and unresolved association. The entire input is analysed;
the row cap never hides findings. Inventory `rows` and non-timeline `limit` are
null. CSV/correlate JSON remain subsequent work; no qualified time-source link
is inferred from channel IDs. See [the report schema](cli-report-schema.md).
