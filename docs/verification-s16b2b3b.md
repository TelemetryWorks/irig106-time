# S16b2b3b — bounded CLI correlation and online report state

Base main 68b3378 (#70 merged). Partial L1-CLI-004 -> L2/L3-CLI-104 only;
no whole-requirement test markers. Matrix stays 198 tested / 206 verified of
288. No Review/Design changes. No dependency, unsafe or library I/O added.

## Resource bounds and report semantics

Every command now uses the library StreamingTimeCorrelator with eviction:
1,024 global references plus a 30-second raw-RTC window, inclusive at the old
edge. These are documented resource policies, not IRIG-mandated numbers.
Duplicate/frozen counters still obey the count cap. A quiet channel can lose
all anchors. No command retains all references; arbitrary old queries may
therefore require replay instead of extrapolation. Summary shows retained,
accepted and evicted counts separately. No host clock or session inference.

One previous accepted time/RTC per channel drives online jumps. Count every
jump, retain first 20 details globally, then show N more with exact total.
Summary uses 1 second, jumps uses its validated threshold; comparison is >.
Indices now mean accepted-observation order. Unlike the old batch detector,
late references are adjacent in observation order, not re-sorted by RTC.
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
rows (default 50); CSV rows use the existing disk spool. Output can grow on
disk, not in a packet-row vector. No process-RSS bound is claimed yet.

## Explicit severity proposal

QueryBeforeRetainedWindow and NoReferencePoint -> WARNING, proposed pending
coordinator confirmation. Reason: the counter/packet remains readable but
absolute context is unavailable, analogous to needs-context. No TF ID is
invented and no accepted TF severity is changed. Surface the finding for each
actual attempted answer, including CSV/timeline rows and per-channel queries.
Default error threshold returns 0 for warning-only input; warning threshold
returns 1. Operational failures retain precedence and CSV transaction rules.

## Honest executed evidence

Tests-first e07be99 adds two integration tests; both fail at base main at
runtime: summary retains 1,100 references, and 200 jump details are unbounded.
After the fix, the 1,100-reference/1 Hz fixture retains 31 at the inclusive
30-second edge; RTC-zero queries report the typed old-window warning, with
threshold-dependent exits. The other fixture counts 200 jumps but samples 20.
Wire fields derive from Chapter 11 Fig11-14 (§11.2.3.2) as existing reference()
fixtures: literal BCD seconds toggle 20/25; header RTC changes recompute the
§11.2.1.1 primary checksum. These are resource-policy/report tests, not new
standard interpretations or decoder-generated encoder vectors.

Additional implementation-time tests cover 5,000 frozen-RTC references (cap
1,024, 3,976 evicted, 4,999 signed +/-5s jumps, 20 samples, exact levels), gap
extrema 1/3 seconds, strict threshold +/-1s versus +2s, and NoReferencePoint
rows under error/warning thresholds. They are not claimed failing first.
Local checks: fmt, touched CLI suite and Clippy, focused mutations, matrix.
Full matrix, platform and existing sparse-file RSS run in PR CI.

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
