# S17b3 — correlate shared report foundation

Base: main ce9abab (#87). Split the native query model from its JSON view to
keep each PR around600 changed lines. This slice completes correlate's move to
the shared Report; CSV/correlate JSON, qualified provenance, stale-table and
remaining reset JSON goldens stay OPEN. No public schema/argument/API change.
The schema guide merely folds the accepted compatibility clarification into
Consumer compatibility; frozen definitions and revision3 remain unchanged.

| Lineage | Clause evidence here | Remaining |
| --- | --- | --- |
| L1-CLI-006 → L2/L3-CLI-106 | Correlate prepares native answers/findings, then renders shared Report; byte-exact ordinary/multi-channel/reset text | Query/CSV JSON, schema additions, complete qualified provenance and goldens |
| L1-CLI-004 → L2/L3-CLI-104 | Bounded structured segment spool and channel-bounded ordinary answers | Qualified joining still awaits approved TMATS integration |

No whole-requirement markers or coverage increase. TRACE-MATRIX is regenerated
unchanged:198 tested /206 verified of288. No Review/Design fields, library code,
feature, dependency or external repository change. No data/time-channel link,
new reset/source policy or time-scale assumption is introduced.

## Honest tests-first and hand-derived fixtures

Tests-only cdd0669 precedes implementation. Both complete text goldens pass on
main (numeric/output baselines, not claimed failed-first features). The private
spool test fails at runtime on main because its first bytes are rendered text,
not a bounded structured record. Its assertion is retained; access to the spool
changes to get_mut() with the new RefCell. Roundtrip/record-bound/writer tests
arrived with implementation and are not claimed as tests-first failures.

106-24R1 §11.2.1.1 header, Figures11-12/13 (§11.2.3.2) FMT4/DATE0 UTC: DOY100,
seconds after12:00, unknown year; RTC ticks100ns. Primary checksums are summed
from literal words, Data Length10 excludes filler2. At target5s, channel7's
RTC0/40s references give12:00:05; channel8's RTC0/41s give13:00:05. A40→1s
backward reset on7 (second clock12:01:01) gives12:01:05 from segment2, while
segment1 still gives12:00:05. The query must print both in order and warn about
ambiguity, without withholding channel8. Expected full text is hand-authored,
not captured from a production encoder/output. Exact stderr is pinned for the
ordinary fixture; reset warning count/ambiguity are asserted.

Structured records retain channel/segment/label, native calendar/optional year,
ns fraction and unresolved issue. Unit fixtures distinguish year0 from unknown
year, preserve max ns and unknown-year day366, repeat reading to prove rewind,
and retain the exact NeedsContext text. Invalid/oversized/truncated private
records fail before callbacks. The oversized fixture is otherwise valid padded
JSON so removing only the length guard admits it. Writer failure remains exit4
above warning findings. Existing31 joining/reset/query regressions pass locally.

Four compiling isolated mutants fail focused tests: known year discarded,
segment labels lost, oversized private record accepted, all segment answers
omitted. The initial year-loss mutation failed type inference; it is excluded
from mutation evidence. The corrected type-preserving mutant fails at runtime.
Restored focused tests, fmt/trace/diff checks pass; full CI supplies platform,
package/security and dense reset/steady RSS evidence, not local full checks.

## Allocation and failure boundary

Only native ordinary answers are retained in a Vec (one per observed time
channel; the wire channel ID space is finite), with one any-channel result.
Reset answers remain on disk in existing private exclusive/cleaned spools;
record reads are capped at4096 bytes and advance over a fixed finite file.
Only one parsed record/model is live during emission. No all-segment collection,
whole-file read or new temp dependency for ordinary non-reset queries. Private
JSON is a CLI scratch format, not a published schema, untrusted input API,
library serde or the cancelled snapshot/rehydration feature.

All query preparation now precedes rendering. Successful/findings stdout/stderr
bytes and severity/counts remain unchanged. On scratch I/O failure, output may
stop before the old header was printed; exit4 still requires discarding partial
output. No claim of identical partial bytes on output failure. The existing
true-CLI32MiB dense RSS gate covers all reset answers and ordinary channels.


CI44981909032 exposed a combined-output ordering regression: reset findings
followed the long answer list, outside the RSS gate's diagnostic prefix. The
small merged-writer regression in tests-only210e6a3 fails at the initial head.
Header and answer rendering are now separate views of the same Query so the
original header → reset findings → answer body order is restored. The RSS gate
is unchanged. The focused regression passes; the subsequent CI run supplies
fresh complete memory evidence rather than treating the failed gate as a pass.

## #88 review delta — native error views and write limit

Tests only; production code and coverage statuses are unchanged. The any-channel
fixture contains20 one-second unknown-year references, DOY365 23:59:30..49;
RTC60s requires crossing the unresolved calendar boundary. It pins the exact
Any channel Warning text, exit1 with --fail-on warning, and both independent
Correlation query diagnostics (any-channel plus per-channel): warning=2,total=2.
Dropping only the any-channel scanner.note compiles but fails this new test.

The segment fixture has RTC0..5s at DOY365 23:59:50..55, then RTC20/21s at
DOY001 00:00:10/11. Three increasing RTC0/1/2 references behind the baseline
establish the accepted short-reset second segment. Targets10,11,12s select the
preceding DOY365 reference and require unknown-year context; segment1 must
print the exact unresolved: message and retain its Segment query warning.
Removing only the unresolved: prefix compiles and fails this new regression.
Payload fields are hand-weighted from106-24R1 Figure11-13 (BCD seconds/minutes/
hours, day low tens/units and high hundreds); the primary checksum is unaffected
by payload-only changes. No encoder or captured output generates these fixtures.

The optional write-limit test supplies a bounded4KiB static-context fixture,
requiring record-too-large failure before any bytes or per-channel match state
are written. A subsequent normal record is still the first match. Removing
only the write-side4096-byte guard compiles and fails the new test. All three
mutants are restored; focused regressions and fmt/trace/diff checks pass locally.
The complete platform/package/security/RSS matrix is PR CI evidence only.
