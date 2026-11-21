# S16b2b3c2 — channel isolation and labelled segment answers

Base main 21039d8 (#73 merged). Partial L1-CLI-004 -> L2/L3-CLI-104 reset/query
clauses only; no whole-requirement markers or Review/Design changes. Matrix
stays 198 tested / 206 verified of 288. No dependencies/unsafe/library I/O.

## Behavior and evidence

885ebb9 is tests-first: four requested-behavior tests fail at main at runtime
(global withholding/missing segment output/pending observations). The provisional
range test at main fails for missing output, not an exposed wrong archive. New
library discard test fails compilation with missing API, not a runtime failure.
Vectors use existing literal Fig11-13 BCD/primary-header checksums. C2 restores
channel8 time-observation rows byte-for-byte against an unchanged no-reset control
with identical frame offsets/indices; its RTC5s correlate line is identical.
The adversarial age fixture also includes a data row at RTC44.5s after a reset.
Under the coordinator clarification below it is intentionally withheld; its
matching numeric channel ID is not a time-source association.

Closed segment ranges are inclusive. An overlapping target gets every answer
labelled channel/segment, plus one WARNING per ambiguous channel, never Any
channel's silent choice. Unreset channel lines stay unchanged, including their
existing checked extrapolation behavior. No matching reset range gives a
warning rather than cross-segment extrapolation. Tests cover both range edges,
short-run provisional points excluded from the old range, and 128 segments:
128 answer lines, 127 reset + one ambiguity warning, first20 messages/108 more.

One/two late references remain classified at resolution/EOF. Candidate ingestion
now follows classification, with original arrival ordinals retained for library
tie handling. Valid decoded time observations are reported directly when
classification/reset context exists; they require no counter bridge. Unqualified data rows in reset/pending context withhold a value. Explicit
per-time-channel queries remain isolated; no data-to-time association is inferred. No emitted row is revised. A new source-selection or
TMATS policy is not invented; qualified CLI-104 joining stays partial.

## Bounds and operations

Normal-file any-channel lookup keeps the existing 1,024 compatibility cache.
Channel-local StreamingTimeCorrelator age windows share another 1,024-arrival
FIFO descriptor budget; actual retained pool count is <=that budget. Stale aged
descriptors may conservatively reduce retention. discard_reference removes the
exact queued RTC (first retained equal arrival), preserving typed loss guards;
normal FIFO removal is at the deque front. Reset removes only its channel.
Pool metadata is bounded by u16 channels, not file length. Tests cover 5,000
arrivals/cap, independent ages and unaffected-channel preservation.

Known queries keep <=6 active values/channel plus range/arrival scalars. Closed
answers use the existing cleanup-owned CsvSpool (private temp, bounded writer),
not a Vec of segments/answers. Match counts have <=65,536 keys. Output grows
on disk as required by the report, not in RAM; no host clock or library I/O.
Pending references are still <=2/channel. No process-RSS claim is made here.

Order: usage validation, input open, packet reads, lazy spool creation on the
first staged segment answer/closed-answer writes, then active-answer staging/
diagnostics/stdout copy. No-reset files create no segment temp. The
first fatal input/output failure wins; a spool failure stops before another
packet read. Injected write-error-before-bad-input and copy-failure tests pin
output4 over later input3/findings1. CsvSpool owns cleanup on every return.

Local focused CLI/library tests, mutants, fmt/Clippy/trace checks only; full
matrix via CI. Dense-F1 few-hundred-MiB all-six-command <64MiB RSS and README
warning removal remain the next slice. Other setup/scale/session/provenance
CLI-104 clauses are not promoted to whole coverage.


Thirteen runtime mutants killed: global withholding; shared age clock;
pending-row suppression; provisional archive pollution; lost closed archive;
missing/inclusive range guards; missing ambiguity/label; pool-cap removal;
ignored copy error; deferred writer error; wrong-RTC discard. Each compiled
and failed assertions. Sources restored before focused rerun.


## PR #74 review — three guards and the conservative data decision

The three requested tests already pass at 0eac5a1 (not claimed failing first).
1. Equal RTCs with different times: discard leaves the later time. The mutant
   replacing position with rposition compiles and fails this assertion.
2. Segment RTCs 0/39/40, then late35 and reset1/2/3: target38 still answers
   from segment1. Replacing range maximum with last RTC compiles and fails.
3. Target5 lies between old10..40/new1..3: assert the exact No reset segment
   covers requested RTC on channel7 WARNING, never silently empty output.
   Removing that finding compiles and fails the test. Sources restored.

Coordinator confirms conservative data rows until S17 qualified (TMATS)
joining supplies their governing time source AND segment. Chapter 10 RTC is
one counter for the whole recorder: a real reset affects every channel at
once. Channel-local bookkeeping tracks observation histories, not independent
hardware counters. A matching numeric packet/time-channel ID does not qualify
an association. Valid stated time observations and explicit channel queries
remain answerable; data's unqualified any-channel answers are withheld after
any reset (also during unresolved classification).

One small policy mismatch was found and reported before fixing: resolve_row
used observed same-ID membership to select a source for data rows. Tests-first
4f2a4cb reproduces it at 0eac5a1 (data8 incorrectly received 12:00:01). Remove
that shortcut; the new case returns N/A plus needs-context WARNING. Update
the prior data-row expectation deliberately; channel8 Time-F1 observation rows
and correlate remain identical to control. A fourth runtime mutant restores
the shortcut and fails. This is the only production change in this delta.

Local focused tests/mutants, fmt, touched Clippy and trace check; full CI via
GitHub. No new whole-requirement markers, coverage or release-gate claim.
