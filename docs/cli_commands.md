# CLI Commands

| Command                                    | Description                                             |
|--------------------------------------------|---------------------------------------------------------|
| `irigtime summary <file>`                  | Packet counts, time channels, RTC range, jump detection |
| `irigtime channels <file>`                 | Per-channel time source inventory                       |
| `irigtime jumps <file> [--threshold-ms N]` | Discontinuity detection                                 |
| `irigtime timeline <file> [--limit N]`     | Per-packet RTC + resolved absolute time                 |
| `irigtime csv <file> [--output path]`      | Full timestamp export                                   |
| `irigtime correlate <file> <rtc_hex>`      | Resolve one RTC against all channels                    |

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
Other commands complete their current analysis before rendering.

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
