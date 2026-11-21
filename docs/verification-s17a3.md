# S17a3 — draft JSON report foundation and #78 schema delta

Base27befa7 (PR77 merged). CLI-only serde_json already approved/pinned. No new
dependency or time-library source changes. No Review/Design edits. Partial
L1-CLI-006 -> L2/L3-CLI-106 only; coverage stays198 tested/206 verified of288.

## Scope and schema decision

Strict --format text/json, one common scan/Report for summary/channels/jumps.
The dedicated #78 schema review is complete; the coordinator selected ALL
recommendations, implemented here for final **delta review**. The schema remains
draft, not frozen. The [schema itself](schema/irigtime-report-v1.schema.json)
contains the version, revision, strict producer/tolerant consumer, field units,
per-field null, wide-string, status-transition and output-failure rules.
[Consumer notes](cli-report-schema.md) explain the same contract.

The envelope is telemetryworks/irigtime-report, version1/revision1. Findings move
to the top level, with explicit levels/code/message/location, plus highest_level.
Closed format/status/scale/basis values replace Debug/free strings. Native raw
reserved values remain visible. DATE/LY and RTC-duration availability are included.
channels' unevaluated jumps and threshold are null. Threshold comes from the
scanner, so summary reports its actual1000ms even if the argument default changes.
Findings remain command-specific; no TF level or content/structural exit changes.

Input names remain as supplied. Lossless hex is POSIX bytes or Windows UTF16LE,
never OsStr::as_encoded_bytes. Custom serde_json Formatter emits ASCII-only JSON,
including surrogate pairs. Library version comes from the enforced exact
lockstep dependency; build source revision uses IRIGTIME_SOURCE_REVISION, then
GITHUB_SHA, then null. The pinned schema ID names the intended v0.8.0 artifact;
it resolves at release, without creating a tag here. Draft->stable is a reviewed
revision increment. Future timeline/csv ambiguity/counter-only row shapes are
reserved, not implemented.

## Bounded segment evidence

Channel rows retain their file-wide meaning. A per-channel segment array records
first/last admitted reference clocks and RTCs. One active row per channel stays
in memory. Closed rows go into a private temporary disk spool, linked with two
fixed offsets per channel; render follows each chain without collecting history.
The spool uses existing exclusive/private temp creation and Drop cleanup.
Interleaved-channel archive test pins order and converted clock basis. The wire
fixture pins isolated first/last endpoints across a reset and channel inventory
with no admitted references. All six text RSS probes remain; direct JSON
summary/channels/jumps probes now run for reset and steady phases at the same
32MiB ceiling. Only the harness loads the completed JSON artifact after measuring
the child process; producer history remains on disk. No large local workload.

## Tests-first and fixtures (honest)

Original tests-first46cab07: two runtime failures at missing --format; bad-format
control already passed. Projection/golden checks originally arrived with code.
For this delta, tests-only887edf7 adds the review projection regression; it fails
on the previous implementation at the ASCII assertion before any correction.
Other schema/segment fixtures arrive with the correction, not claimed failing
first. Missing-year literal: Fig11-12/13 (11-23/PDF31,11-25/PDF33), §11.2.1.1
header sumFC60, Data Length10 excluding DD filler. Golden clock:
day10012:00:00 =8596800000000000ns, unknown year, assumed UTC/RDG-009.
Run-specific path/platform encoding/version/build revision are separately checked
before normalizing golden provenance. Units/schema constants and structural
key/enum checks are NOT a general-purpose JSON Schema validator.

The projection fixture supplies independent, deliberately distinct report facts:
1100 references (evictions), two resets, three segments, unequal RTC/gap endpoints,
21 jumps/one sample, 1 error/2 info/25 warnings, 20 samples/8 omitted; SRC RMM2 and
None15, NTP TS3 (Fig11-15), fail-oninfo, invalid native filename bytes/code units.
These are projection facts, not falsely claimed decoder/segmentation fixtures.
A separate wire fixture uses hand-weighted DOY BCD (Fig11-12/13), NTP TS3
(Fig11-15), valid §11.2.1.1 checksums, 1200 references, a reset, unequal gaps and
trailing padding. Its1199 jumps and isolated segment endpoints are pinned.
1203 references minus one retained =1202 evicted (global age high-water behaviour).

All reviewer survivors were killed at runtime with one replacement at a time,
using review_projection_pins_every_observed_field_and_command_null. Sixteen
replacements cover the review's15 grouped survivors, testing first_rtc at BOTH
file and channel scope:

| Mutant | Failing assertion |
| --- | --- |
| info -> information | finding sample level =info |
| RMM2 ->3; None15 ->0 | both raw nibble values |
| status alwaysnull | reserved NTP TS status |
| format2_word always0 | raw CSDW =3 |
| error/info counts swapped | error1, info2 |
| threshold from args, including channels | actual17ms, then null for channels |
| min/max gaps swapped | min100/max300ns |
| jump omissions0; finding omissions0 | 20 and8 |
| first_rtc from last (file/channel) | first11 vs last99 |
| references_evicted0 | nonzero exact correlator count |
| reset_count0; segment_count1 | reset2, segments3 |
| name_lossyfalse | native non-UTF8 filename |

Each mutant compiled and failed a runtime assertion; source was restored after
each probe. Earlier seven killed mutants remain covered by version/wide extrema,
clock basis/year-offset and writer tests. ASCII Unicode/control/surrogate-pair
round-trip, native platform provenance, schema key/description/closed-enum
structure, channels null jumps and linked segment ordering have focused tests.
The write-failure case still returns4. No complete JSON can be promised after
an output failure: consumers must discard stdout on4.

## Remaining clauses / verification

No whole-requirement marker added. Complete qualified joining (including inferred/
default TFMT), stale-table and full network/GPS qualification goldens remain OPEN
for the subsequent S17 slices. The missing-year golden and reset/network inventory
fixtures do not claim those clauses. No JSON loading API, new dependency, library
serde, schema freeze, publication or release permission.

Local checks: fmt, focused CLI/report tests, touched CLI Clippy, the16 focused
projection mutants, small generator assertions and trace check only. Full platform,
MSRV, WASM, dependency, safety and large RSS checks are delegated to PR CI.
CI run and exact head are reported with the frozen update, not guessed here.

## Final schema delta — TS mapping and jump finding identity

Ordinary merge30889fb brings main a4d8bf8 (#79) into this PR. CHANGELOG was the
only conflict: retained BOTH the S17a3 draft JSON section and #79's release-gate
section, before the existing dependency entry. No release gate or schema work
was dropped. No Review/Design field changed; schema remains draft revision1.
Freeze (stable plus incremented revision) is a separate reviewed PR.

Tests-only87b09ce: the reserved-TS test fails against the merged
pre-fix code (TF-003 rather than TF-006); TS0 and jump identity controls already
pass and are not claimed failing first. Fix note_decode with an explicit TS0
branch (TF-003) and reserved-value branch (TF-006), both warning. Literal
Figure11-15 TS0 and every TS2..15 exercise the parser-to-JSON path. The jump
fixture uses Figure11-13 native +6s at RTC+1s; an intervening non-time packet
separates packet2 from reference1. Assert TF-017, channel7, packet2 and warning
in summary AND jumps output. No whole-requirement marker or coverage gain.

Correct the `never` threshold description. Release builds must set
IRIGTIME_SOURCE_REVISION to the release commit; PR GITHUB_SHA can be a temporary
merge commit. TF-018 remains information in the catalog. The approved CLI
heuristic reset remains warning/code null. Coordinator-accepted 2026-11-21 (owner may override): heuristic reset warnings keep code null; TF-018/information is reserved for an established new-session fact. No reset behaviour/level change.

Later follow-up, explicitly not implemented in this PR: the #79 consumer seam
should use tmats #9's association_unestablished / association_conflicting /
association_evidenced states; conflicting stays unresolved for answers. No TMATS
dependency or edit to that seam is part of this schema correction.

Focused delta evidence: all10 s17_json tests pass. Five one-at-a-time replacements
compile and fail runtime assertions: jump code None; sample channel None; TS0
mapped TF-006; reserved TS mapped TF-003; jump packet location replaced by the
reference index. The last is killed by packet2 versus reference1. Source restored
between probes. Local fmt, touched CLI Clippy and trace checks are the only other
checks; the full matrix/RSS/security checks run in GitHub CI. Coverage remains
198 tested /206 verified of288. No stable-schema change in this delta.
