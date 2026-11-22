# irigtime JSON report, stable v1, revision 9

The [canonical producer schema](../apps/irig106-time-cli/schema/irigtime-report-v1.schema.json)
is packaged with the CLI. The [docs snapshot](schema/irigtime-report-v1.schema.json)
is generated from it with `python3 scripts/sync-cli-schema.py`; the workspace
sync test enforces byte identity. The schema is the normative field/type/unit/null
contract. It is CLI-owned (ADR-0019), not an IRIG 106 format
or a library serialization API. The #78 schema review selected the shared
`telemetryworks/irigtime-report` envelope; the inventory contract was frozen as stable v1,
revision 2. Revision 3 added timeline packet rows/limit. Revision 4 adds
correlate query metadata/results, retaining previous field types, units and meanings. Revision 5 adds the
coordinator-approved `derived, stale leap table` basis value.

`summary`, `channels`, `jumps`, `timeline` and `correlate` accept `--format text|json`, default `text`.
Both render the same scanner/Report facts. `channels` does not evaluate jumps:
`report.jumps` and `options.threshold_ms` are **null**, rather than a false zero.
Timeline also leaves jumps/threshold null. `options.limit` is its effective
positive row cap as a decimal string; it is null for inventory commands.
Findings are command-specific; evaluating jumps can add findings. `highest_level`
is independent of `--fail-on`; that option only determines exit 1.
Stale PTP conversions raise TF-021 warning with the converted UTC, raw TAI,
applied offset and table currency/identity in the packet-located sample message.
UTC is kept, explicitly labelled possibly out of date; revision5 carries the
covered/stale distinction in every selected PTP clock basis.

The envelope always contains `schema`, `schema_version`, `schema_revision`,
`schema_status`, `tool`, `input`, `options`, `highest_level`, `findings`, `report`.
All nested defined keys are also present. Null has a **field-specific** meaning:
not observed (first/last), unknown (year), not applicable (F1 fields on F2), or
not evaluated (jumps on channels). Samples contain explicit lowercase levels,
a TF code when identified (otherwise null), message, channel and packet.
No Rust Debug spelling is a JSON identifier.

Consumer compatibility: ignore unknown fields within a supported major
`schema_version`; additions increment `schema_revision`. A revision MAY add values to options.command, row kind and similar enums for newly supported views. Consumers MUST treat unknown enum values as an unsupported view, not an error. Reserved shapes such as rows MAY gain fields until first emitted. Producer validation
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
`report.rows` remains null for inventory commands. Timeline emits the same
PacketRow observations as its text renderer, projecting and writing one row at a
time rather than retaining a JSON DOM for all rows, in file order, at most `--limit`
(default 100). Scanning and findings still cover the complete input. Rows carry
packet/channel/RTC, `offset_bytes` and `data_type`, direct `clock` or null, and
`kind` absolute/counter-only. An absent clock has an issue directing the reader
to command findings; zero is never substituted. Candidate segment answers stay
empty in timeline. Packet-row ambiguity candidates remain reserved. CSV is its
own machine format, not another JSON view; see [CSV columns](cli-csv-schema.md).

Packet/query clocks retain the selected reference's F1 stated/assumed basis or
covered/stale PTP conversion basis, independently of the packet's own FMT and
the channel's latest observation. IRIG-A/B/G use UTC / assumed, RDG-009; FMT4
uses UTC / stated FMT; FMT5 uses GPS / stated FMT. RTC remains unknown/unresolved. This does not establish a
data-to-time-channel relationship or change counter selection. Retained basis
metadata shares the existing reference cap/evictions; closed query segments keep
it in their private spool.
`association` uses the seam vocabulary association_unestablished /
association_conflicting / association_evidenced; timeline emits unestablished
only. Conflicting is unresolved for answers, never an automatic source pick.
After reset/pending classification, existing conservative counter-only data rows
remain so. No source, segment, year offset or UTC assumption is fabricated.
The row schema additions are revision 3; existing inventory fields are unchanged.

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


## Counter queries (revision 4)

`correlate --format json` renders the same prepared native Query as text.
`options.query_rtc` is the requested counter in100ns ticks as a decimal string;
it is null on other commands. `report.correlation` contains `any` (clock/issue)
and streamed `answers` (channel, one-based segment, labelled, clock/issue).
It is null on other commands. `report.rows` remains null for correlate: a query
has no packet index or byte offset. Its limit/threshold/jumps are also null.
All defined keys are present. An unresolved clock is null with a context issue;
a resolved clock has null issue. Native per-channel errors omitted by text stay
in command findings; unresolved printed segment rows are retained.

Across resets, any-channel is explicitly withheld; every matching segment answer
is emitted, with the existing ambiguity warning, and unaffected channels remain
answerable. Outside all reset ranges, the existing no-coverage warning remains.
Without references, any is an explicit missing-reference result, never zero time.
Ordinary answer order is channel ID; reset answers retain closure order then final
channel order, matching text. Projection holds one answer Value at a time.

These are counter-selection policy answers, not evidence of a data/time-channel
relationship. F1 stated/assumed and PTP conversion basis follow the selected
reference; RTC or unavailable scale/basis remains unknown/unresolved. No data association
is inferred. The reset query golden
covers two possible clocks and an unaffected channel. Complete multi-command
adapter qualification remains subsequent work. CSV has its own documented
columns/quoting rather than a JSON wrapper.


## Stale conversion labels (revision 5)

Every channel/segment/packet/query clock derived from a selected stale PTP
reference uses `basis: "derived, stale leap table"`. Covered PTP references keep
`derived, leap table`. Text uses `UTC (from TAI via stale leap table, possibly out
of date)` wherever that conversion is displayed. Covered PTP timeline and
correlate values now also gain `UTC (from TAI via leap table)` in text, and
their JSON row/answer clocks carry UTC / `derived, leap table`. These covered
output changes are coordinator-accepted. TF-021 findings still retain raw
operands/currency details.
A revision-4 consumer that does not know this new basis treats the view as
unsupported rather than rejecting the version-1 document. The regression uses
the frozen revision-4 enum set, not the producer's expanded set. CSV stays
basis-free; stale status is available through findings and the JSON views.


## Protocol/status facts (revision 6)

`report.rows[].network_time` records that packet's Format2 CSDW fields, separately
from its numeric clock's selected source. It remains present for reserved TS/NTF
and missing conversion context. Null means non-F2 or missing four-byte CSDW.
`report.correlation.any.network_time` and `answers[].network_time` instead record
the selected reference, including closed reset segments. They are null without
an applicable selected F2 answer. Keys are `ntf` (raw0..15), `protocol` (NTP,
IEEE1588-2002, IEEE1588-2008, reserved), `ts` (raw0..15), `status` (not-valid,
valid, reserved). Facts never establish data-channel association or make an
invalid reference usable. Older consumers ignore this additive field. Timeline
and correlate text append the same protocol/TS facts, explicitly prefixed
`[packet: ...]` and `[reference: ...]` respectively. CSV separately exposes
packet facts under its approved [19-column append-only contract](cli-csv-schema.md);
the CSV addition itself changes no JSON fields; policy metadata is revision7.


F1 clock qualification uses the already-reviewed version1 scale/basis enums:
that F1 label fix introduced no key/type/unit/enum/revision change in revision6. Text now
suffixes F1 row/query clocks with `UTC (assumed, RDG-009)`, `UTC (stated FMT)`,
`GPS (stated FMT)` or `scale unknown`. Native GPS values are not converted to
UTC. No year, origin or data-channel/time-channel association is inferred.
The19-column CSV remains basis-free and unchanged.


## Effective report policy (revision 7)

`report.policy` identifies the actual CLI nearest-reference counter-offset
algorithm and its versioned defaults, not the native library SharedPolicy or
Standard/Interpolate modes. It records the live30s/1024-reference bounds, separate
six-candidate-per-channel query brackets, immediate reset strictly above30s and
three strictly increasing behind-baseline references for a short reset. Bounds
are reader/resource policy, not standard packet limits. Main global-age and
isolated channel-age pools each have their own1024 shared budget; query brackets
preserve old requested RTCs independently of those live caches.

The installed leap table's ID, source, transition count and inclusive supported/
current-through Unix-second bounds are retained even if no PTP value uses it.
Counts, ns and Unix seconds are decimal strings. No table is fetched or host
clock consulted. Effective jump threshold remains in `options.threshold_ms`.
The policy ID changes with the CLI correlation algorithm; the defaults ID changes
with fixed defaults. These CLI identities must not be substituted for native
library policy IDs. Metadata applies to the whole report, including rows/answers,
and is emitted without findings. It does not establish source association or
claim a complete native answer-policy replay snapshot. Older consumers ignore
this additive field under the version1 compatibility rule.


Unreleased revision7 also records `finding_sample_cap` and `jump_sample_cap`
as decimal-string counts, each20 by default, from the separate constants used
by those sampling paths. Omitted/total counts remain exact. Isolated cache
window/cap coupling is tested against instantiated channel caches and actual
pooled evictions, not just the main cache or a repeated default literal.


Revision8 adds nullable `options.ntp_era` (decimal string -14..=59) and the
UTC basis values `caller-declared era`, with `, leap-ambiguous` or
`, leap coverage uncertain` when applicable. The explicit declaration applies
to all NTP packets in the input; it never converts invalid TS or affects PTP/F1.
No option means no nonzero NTP conversion, even with dated F1 references.
All-zero raw words are TF-029 unknown/unsynchronized, independent of era;
TF-030 keeps the ambiguous mapped value. Seconds0 with a nonzero fraction is
not the sentinel: with declared era0 it can map1900-01-01, but without a declaration
it remains needs-context (coordinator-confirmed2026-11-22, RDG-012).
`report.policy.ntp_era_source` is `caller-declared` iff the option is present,
otherwise `none`; `ntp_era_inference` is always `disabled`. These are additive
within unreleased revision8. The era range includes exactly those2^32-second
spans overlapping calendar years0..=9999; individual out-of-calendar instants
still need context. Counts/fractions use the checked
RDG-012 converter, never a TAI offset. Earlier consumers ignore the new option
and treat unknown basis values as unsupported views under the frozen-v1 rule.


## Automated producer validation (CI-only tooling)

The CLI CI job creates `$RUNNER_TEMP/schema-venv` without system site packages.
It installs `scripts/requirements-schema.txt` using that interpreter with
`-I -m pip install --require-hashes --only-binary=:all:`; both schema steps use
the same venv interpreter with `-I` (ignoring Python environment/user paths). jsonschema4.26.0 and every transitive
Python dependency are version-pinned with SHA-256 wheel hashes. Nothing is added
to a Cargo dependency tree or shipped with the CLI/library. CI uses Python3.12
Linux x86_64; the lock also permits Python3.11 Linux aarch64 for focused probes.

`scripts/validate_cli_reports.py` checks the canonical packaged schema against
the Draft2020-12 metaschema, then validates live summary, channels, jumps,
timeline and correlate JSON against that schema's current revision. It resolves
local `$defs` without fetching release URLs. The exact six fixture labels and thirty fixture/view combinations are pinned
by a test. Thirty small reports cover protocol/
status variants, stale PTP, unknown year, resets, ambiguity/counter-only output,
missing and declared NTP era, and the NTP sentinel. CSV is its own machine format,
not a JSON view. Full inventory goldens also validate. Negative tests reject
unknown keys, wrong revision/enums, numeric wide values and malformed schemas.
This checks producer structure; semantic goldens and RSS gates remain separate.
No schema revision is changed by adding this validation tooling.


## Recording-wide native spans (revision 9)

`report.native_sessions` is evaluated only for summary; other views emit null.
It uses the library's native SessionClock across every parsed header, with actual
late-bound/inference settings. It is separate from `reset_count` and per-channel
`channels[].segments`, which retain the CLI compatibility heuristic. `_ticks`
are signed unwrapped100ns coordinates; counter_span_ns=(high-low)*100. First/last
are observation order; high-water does not fall on late arrivals. Text and JSON
stream closed spans from a bounded private spool plus the active span. Native
ordering ambiguity yields needs-context and a reason, retaining only a labelled
known prefix; remaining report analysis continues. The associated code-null
WARNING is coordinator-accepted (PR #99 review). No TF code fits: TF-018 is for
established new-session facts, TF-019 for late packets and TF-024 for a flag bit.
CLI native policy-inferred sessions stay `start: "inferred"`; they do not emit
TF-018. This differs deliberately from the library's `TimelineBuilder`, which
records TF-018 for policy inference under L3-COR-105. The CLI reserves TF-018 for
established new-session facts; it does not forward those inferred library findings.
This is header-counter analysis, not calendar spans, reference quality, admission
or TMATS association.
