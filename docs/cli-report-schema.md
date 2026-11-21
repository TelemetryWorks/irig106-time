# irigtime JSON report v1 — DRAFT

`summary`, `channels`, `jumps`: `--format json` (default `text`). Separate,
lowercase values; duplicates/unknown formats give usage2 before input access.
The [JSON Schema](schema/irigtime-report-v1.schema.json) lists every field/type.
This is TelemetryWorks' own CLI format, not an IRIG 106 wire representation.

| Section | Meaning and units |
| --- | --- |
| schema/schema_version/schema_status | `irigtime-report`, integer1, `draft`; not frozen. |
| tool | Executable name and lockstep package version; no host-clock timestamp. |
| input | Supplied path, lossy-display flag, hex native OsStr bytes with platform-dependent encoding label, size in bytes. Native bytes prevent loss of non-UTF-8 path evidence; they are not a portable path loader. |
| options | Command, format, effective fail-on, effective jump threshold in ms (summary1000, jumps chosen; channels null). |
| units | Explicit RTC100ns ticks, durations ns, counts items, size bytes; native clock counts are ns from day1, not Unix timestamps. |
| report | Exact totals/retained/evicted/reset counts, first/last RTC, ascending channel/type arrays, bounded jump/finding samples and omitted counts. |
| channel | Observed format/source, raw F2 word and status, native scale plus basis, segment count, first/last clock/RTC and min/max gaps. |
| clock | Nullable year, DOY and time components, native clock count; scale/basis is explicitly unqualified until qualified joining. No implicit UTC assertion. |
| jumps | Exact count/omitted, at most20 samples: channel/index/expected/actual ns and signed residual ns. |
| findings | Exact error/warning/info/omitted counts plus first20 messages; thresholds still see all findings. |

All 64/128-bit quantities and platform-sized counts are decimal strings,
including signed residuals; no floats. Small wire fields (u8/u16/u32), date
components and schema version are JSON integers. Null means unavailable, not
zero. Current report fields contain no 128-bit quantity; the decimal helper is
also tested at i128/u128 extrema. Ordering is ascending BTreeMap keys/IDs and
observation-order samples; identical input/path/options/tool version gives
identical JSON bytes. Text and JSON use the same bounded Report projection.

Draft v1 may change while qualified joining/provenance is completed. The
owner's single pre-freeze schema review is still to come; this is not that
review or authorization to freeze. After freeze, incompatible shape/type/unit
changes require a new schema version. No JSON loading API, library serde or
library JSON feature is provided. Owner-approved 2026-11-20: CLI-only
`serde_json = "1"`, locked to1.0.149 (declared MSRV1.68); workspace MSRV1.85.
