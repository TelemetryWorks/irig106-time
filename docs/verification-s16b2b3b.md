# S16b2b3b — bounded CLI correlation and online report state

Base main 68b3378 (#70 merged). Partial L1-CLI-004 -> L2/L3-CLI-104 only;
no whole-requirement test markers. Matrix stays 198 tested / 206 verified of
288. No Review/Design changes. No dependency, unsafe or library I/O added.

## Resource bounds and report semantics

Every command now uses the library StreamingTimeCorrelator with eviction:
1,024 global references plus a 30-second raw-RTC window, inclusive at the old
edge. These are documented resource policies, not IRIG-mandated numbers.
Duplicate/frozen counters still obey the count cap. A quiet channel can lose
all anchors. No command retains all references. Correlate retains its known
target's two RTC bracketing groups (at most six values/channel) independently of cache eviction;
old targets remain answerable. Summary shows retained,
accepted and evicted counts separately. No host clock or session inference.

One previous accepted time/RTC per channel drives online jumps. Count every
jump, retain first 20 details globally, then show N more with exact total.
Summary uses 1 second, jumps uses its validated threshold; comparison is >.
Indices now mean accepted-observation order. Unlike the old batch detector,
backward-RTC references warn and skip that jump/gap pair; they remain accepted
for correlation and as the next pair's predecessor, not re-sorted by RTC.
The arithmetic uses nominal day-of-year nanoseconds (including whole days),
not only time of day. This is a compatibility diagnostic, not the qualified
session/year/scale-aware timeline engine; no COR/QUA coverage is claimed.

Per-channel count, first/last time/RTC and gap extrema update online. Gap means
forward modular RTC elapsed between adjacent observations, not a sorted-file
quality metric. Unknown extrema remain None until a pair exists. No growing
jump/reference vectors feed quality. u16 channels bound inventory/statistics
at 65,536 entries; data-type counts have at most 256. Live references have cap
plus one transient entry, with deque storage reclamation from #70. Diagnostic
strings and jump samples have cap 20 each. Timeline retains only caller-limited
rows (default 100); CSV rows use the existing disk spool. Output can grow on
disk, not in a packet-row vector. No process-RSS bound is claimed yet.

## Approved severity decision

QueryBeforeRetainedWindow and NoReferencePoint -> WARNING, approved by the
coordinator in the PR #71 acknowledgement. Reason: the counter/packet remains readable but
absolute context is unavailable, analogous to needs-context. No TF ID is
invented and no accepted TF severity is changed. Surface the finding for each
actual attempted answer, including CSV/timeline rows and per-channel queries.
Default error threshold returns 0 for warning-only input; warning threshold
returns 1. Operational failures retain precedence and CSV transaction rules.

## Honest executed evidence

Tests-first e07be99 adds two integration tests; both fail at base main at
runtime: summary retains 1,100 references, and the jump-detail output exceeds
20 samples. No exact baseline jump count was asserted.
After the fix, the 1,100-reference/1 Hz fixture retains 31 at the inclusive
30-second edge. The initial implementation returned an old-window warning for
RTC-zero queries; the review decision below replaces this with preserved target
brackets, so old correlate answers succeed. The other fixture counts 200 jumps but samples 20.
Wire fields derive from Chapter 11 Fig11-14 (§11.2.3.2) as existing reference()
fixtures: literal BCD seconds toggle 20/25; header RTC changes recompute the
§11.2.1.1 primary checksum. These are resource-policy/report tests, not new
standard interpretations or decoder-generated encoder vectors.

Additional implementation-time tests cover 5,000 frozen-RTC references (cap
1,024, 3,976 evicted, 4,999 signed +/-5s jumps, 20 samples, exact levels), gap
extrema 1/3 seconds, strict threshold +/-1s versus +2s, and NoReferencePoint
rows under error/warning thresholds. They are not claimed failing first.
Local checks: fmt, touched CLI suite and Clippy, focused mutations, matrix.
Full matrix, platform and existing sparse-file RSS run in PR CI. That sparse
probe now asserts exact warning totals rather than TF-001 sample text: CSV
produces 24,576 unresolved rows + TF-001, timeline (explicit --limit 50) 50 + TF-001, correlate two,
other commands one. TF-001 after the sample cap still counts but is not printed.
No local large-file generation or RSS rerun.

## Remaining gate / honest limitations

Next slice adds a few-hundred-MiB small-packet workload with one F1 per ten
packets across all six real commands, checking peak RSS <64 MiB each. Until
it passes, retain the interim README warning and publish=false. Existing
sparse transport RSS does not establish this dense reference/report bound.
Qualified TMATS/setup ranges, session/scale-aware F1/F2/own-time adapters and
full provenance remain outside this compatibility memory slice; CLI-104 is
not marked fully covered. No time-library -> TMATS dependency was introduced.

Ten individually compiled runtime mutants are killed: disabled count cap;
disabled age bound; unbounded jump samples; counting sampled jumps only;
threshold >= instead of >; reversed residual sign; old-window level made error;
empty-window level made error; silenced unresolved row; min gap computed as max.
Sources restored and focused tests rerun. No compile failures count as kills.


CI run 44828819381 caught a probe assertion error, not a CLI failure: the new
RSS expectation assumed default timeline limit 50, but args defaults to 100,
so 101 warnings were correctly reported. Pin the probe to explicit --limit 50;
production defaults and argument semantics are unchanged. No local RSS rerun.


## PR #71 requested delta — late pairs and known-target retention

Tests-first 53b294e: three runtime failures at frozen 6ddd75e (the late pair,
120-second old-target probe and revised existing old-query test). Midnight
crossing already passed; it is not claimed failing first. Late fixtures use
literal Fig11-13 DOY fields and valid primary checksums, with non-time input
also proving packet index rather than reference index. The late pair is tested
inside (0,2,1 seconds) and outside (0,40,1) the live window. Both give zero
jumps, exactly one out-of-order warning, and max gap 2/40 seconds respectively.
Outside-window timeline resolution additionally reports the approved live-window
warning; it never invents an absolute time from a discarded reference.

For correlate only, keep two RTC groups per u16 channel: closest <=target and
closest >target, with arrival provenance. Keep the first, second and latest arrivals in each
chosen group to preserve the full library's duplicate-boundary answers: at
most six values per channel, even under unlimited frozen-RTC duplicates. There is no global all-reference
list. At render, bounded candidate sets reuse TimeCorrelator's nearest/tie and
checked calendar shift; cross-channel duplicate RTC candidates keep original
arrival order. Per-channel render uses at most six points; any-channel render
uses at most six times the channel count plus O(channels) scratch/index storage.
The target is parsed before input opens; no extra input pass or host-clock use.
An interleaved-channel differential checks seven targets (before/inside/after,
exact/midpoint) against the full library correlator for any/channel1/channel2/
missing-channel answers, with nonlinear clocks and reverse observation order.
The first simpler global-bracket implementation failed that differential and
was corrected before pushing; no released behavior is claimed there. A further
75 comparisons cover conflicting duplicate RTC runs (1/2/3/9/32 arrivals/RTC/channel)
and pin before/exact/midpoint/after boundary answers to the full library. A
single-value group failed that probe and was replaced by bounded boundary values.

The requested 120-second file at target RTC 5s returns Day100 12:00:04.000.000
on one and two channels, with warning threshold success. A late PTP reference
outside the window is converted before live-cache eviction, captured for the
known query and dates correctly (2017-01-02, +1s). Its raw seconds fixture is
Fig11-15/17 NTF1/TS1 and TAI 1483315200+37, not a decoder round trip.
The former old-query warning test now verifies the preserved old answer;
live-window warning tests remain separate. Severity proposal wording is removed
from code/help/docs: both errors are approved WARNING, no new TF ID.

S17a handoff only: render Expected/Actual as day-of-year plus time of day in
human text, keeping raw nanoseconds in machine formats. No formatting/API
change for that deferred note here. No full CLI-104 marker or gate lift.


Review delta: 20 CLI unit tests plus 18 join integration tests pass. Eleven
focused runtime mutants are killed: missing late guard; jump/gap fall-through;
late level changed to info; reference index substituted for packet index;
unconfigured target cache; wrong preceding/following brackets; reversed
cross-channel arrival order; dropped latest/second duplicate representatives;
and day-of-year basis replaced by time-of-day basis. Sources restored before
focused rerun/Clippy. Other platform/full checks are delegated to CI.
