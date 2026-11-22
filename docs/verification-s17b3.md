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
