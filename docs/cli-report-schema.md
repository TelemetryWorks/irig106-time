# irigtime JSON report, stable v1, revision 2

The [canonical producer schema](../apps/irig106-time-cli/schema/irigtime-report-v1.schema.json)
is packaged with the CLI. The [docs snapshot](schema/irigtime-report-v1.schema.json)
is generated from it with `python3 scripts/sync-cli-schema.py`; the workspace
sync test enforces byte identity. The schema is the normative field/type/unit/null
contract. It is CLI-owned (ADR-0019), not an IRIG 106 format
or a library serialization API. The #78 schema review selected the shared
`telemetryworks/irigtime-report` envelope; it is now frozen as stable v1,
revision 2.

`summary`, `channels` and `jumps` accept `--format text|json`, default `text`.
Both render the same scanner/Report facts. `channels` does not evaluate jumps:
`report.jumps` and `options.threshold_ms` are **null**, rather than a false zero.
Findings are command-specific; evaluating jumps can add findings. `highest_level`
is independent of `--fail-on`; that option only determines exit 1.

The envelope always contains `schema`, `schema_version`, `schema_revision`,
`schema_status`, `tool`, `input`, `options`, `highest_level`, `findings`, `report`.
All nested defined keys are also present. Null has a **field-specific** meaning:
not observed (first/last), unknown (year), not applicable (F1 fields on F2), or
not evaluated (jumps on channels). Samples contain explicit lowercase levels,
a TF code when identified (otherwise null), message, channel and packet.
No Rust Debug spelling is a JSON identifier.

Consumer compatibility: ignore unknown fields within a supported major
`schema_version`; additions increment `schema_revision`. Producer validation
remains strict. Incompatible shape/type/unit/meaning changes require a new major
schema and a breaking tool release. The reviewed draft-to-stable transition is
revision 2. Future contract changes require review and follow the version/revision
rules. The `$id` identifies the intended version-pinned v0.8.0 release artifact;
that URL becomes resolvable at release. The schema freeze creates no tag and
publishes no package.

Every `usize` and signed/unsigned 64/128-bit quantity is a decimal string.
Small wire/date fields remain integers. Units follow names: `_ns`, `_ms`,
`_bytes`, `_rtc` (100 ns ticks). Native `clock_count_ns` is relative to day 1,
not Unix time. Clock scale and basis are closed enums, including
`derived, leap table` for converted PTP. The native channel scale can differ
from the scale of its converted first/last clocks. Reserved formats/statuses
retain raw fields. DATE and LY are included.

Input names remain exactly as given, never canonicalized or made absolute.
Lossy display names have a flag and lossless platform representation:
`posix-bytes` or `windows-utf16le`, in lowercase hex. JSON stdout is ASCII-only;
non-ASCII characters use Unicode escapes, including surrogate pairs. Tool and
library versions are lockstep; source revision comes from the build's
`IRIGTIME_SOURCE_REVISION`, else `GITHUB_SHA`, else null. **Release builds must
set IRIGTIME_SOURCE_REVISION to the actual release commit.** In PR builds,
GITHUB_SHA can name a temporary merge commit, not the reviewed branch head;
that fallback is diagnostic build provenance, not a release identity. Runtime
never probes Git.
Objects/channel/type IDs order deterministically; findings/jumps keep observation
order. Text retains its existing human-readable labels.

File-wide first/last RTCs are the first/last complete packets in file order,
across channels and resets. `rtc_duration_ns` is unavailable after any reset.
Channel first/last rows also span resets. Each channel's `segments` gives isolated
first/last clocks and RTCs, indexed from 1; a channel with no admitted reference
has zero segments. Closed segments are spooled to a private temporary file and
streamed during rendering; only active rows and chain endpoints stay in memory.
The whole artifact can grow with reset count, while producer memory stays bounded.
The reserved `report.rows` shape documents future absolute/counter-only/ambiguous
timeline/csv rows, but is always null in these three commands. No such adapter
is implemented or claimed here.

Complete JSON is produced on success/findings (exits 0/1). No report is started
on usage/input errors (2/3). An output error (4) can leave partial, or even fully
written but unsuccessfully flushed, JSON: discard stdout when exit 4 occurs.
Existing command-specific stderr diagnostics remain. Writer errors take
precedence over finding thresholds. No new dependency or library serde.

## Reset warning versus catalogued reset finding

TF-018 is an **information** finding for a counter reset/new session. The CLI's
approved reset heuristic currently emits a **warning with code null**. It is not
TF-018 at a different level, and the JSON keeps that distinction.

Coordinator-accepted 2026-11-21 (owner may override): heuristic reset warnings keep code null; TF-018/information is reserved for an established new-session fact. Do not relabel this warning TF-018 or silently change its
approved severity, counts or --fail-on behaviour. This PR makes no reset mapping
change or new finding assignment.
