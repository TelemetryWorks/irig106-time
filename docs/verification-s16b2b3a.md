# S16b2b3a — bounded correlation storage prerequisite

Base main 822ec14 (PR #69 merged). Split the remaining work into reviewable
slices: this storage primitive, CLI online/query integration, then dense-F1 RSS.
This PR does not wire the CLI or claim its report state is bounded yet.

## Clause coverage and resource policy

L1-CLI-004 -> L2/L3-CLI-104: partial bounded-reference prerequisite only.
No whole-requirement markers: coverage stays 198 tested / 206 verified of 288.
No requirement statement, Review/Design field, dependency, unsafe, library I/O,
publication or feature changes. README warning and CLI publish=false remain.

The existing StreamingTimeCorrelator compatibility API evicts by raw RTC age,
which cannot bound duplicate/frozen RTCs. The opt-in NonZeroUsize global count
cap handles that case. Evict smallest raw RTC, then lowest channel ID on ties;
equal-RTC entries within a channel evict in arrival order. Insertion temporarily
holds one extra entry. Retained entry count is at most the caller's cap; empty
map entries disappear and large backing allocations are copied into smaller
deques when fewer than half their slots remain. The test bounds aggregate
deque capacity by four times its configured cap while cycling channel bursts.
Allocator overhead is not a process-RSS guarantee.

The old constructor remains age-only, with the owner-directed breaking query
and duplicate-RTC fixes described below. This compatibility class still lacks session/scale/basis
semantics: modern stream/timeline remain recommended for recording answers.
These are resource-policy fixtures, not IRIG wire vectors or new standard
interpretations. No standard mandates an eviction order.

## Tests-first and executed evidence

23703d5 adds four tests before the API exists: baseline compilation fails with
E0599 for the missing constructor. This is a compile failure, not four runtime
failures. Tests cover frozen-counter duplicates, global versus per-channel cap,
age eviction, backing allocation reclamation and empty-map reclamation.
Two additional tests arrived with implementation: latest equal-RTC arrival in
a one-slot cache, and oldest-RTC eviction independent of channel/arrival order.
All 17 streaming unit tests pass (11 existing, six new).

Six individual runtime mutants are killed by focused tests: ignoring the count
cap; applying it per channel; retaining large vector allocations; retaining
empty channel entries; evicting maximum rather than minimum RTC/channel; and
using the old equal-RTC insertion order in the bounded cache. Each test run
compiled and failed an assertion; sources were restored and the focused suite
rerun. Local checks: fmt, focused tests/mutants, library Clippy -D warnings and
trace-matrix check. Full feature/MSRV/WASM/platform checks are delegated to CI.

## Remaining release gate

The CLI still uses unbounded TimeCorrelator and collected jump/quality state.
Next integration must retain query-specific nearest candidates for an arbitrary
requested RTC rather than incorrectly depending on the final live window.
Online jump/quality summaries must preserve finding counts and levels. No
command inherently needing all references has been justified by this PR.
The final dense-F1 small-packet RSS workload must exercise all six commands on
a few hundred MiB, about one F1 per ten packets, below 64 MiB each. Only after
that gate passes may the interim README warning be removed.


## PR #70 review — coordinator decisions and regression evidence

Tests-first e2bbd8f adds the one-hour clock-step regression under cap 2 and
under age eviction, a differing-time duplicate-RTC regression in both modes,
and a small seeded differential. All three fail at frozen head 7f3368b at
runtime. The earlier test-writing compilation errors were corrected before
this committed runtime baseline; they are not counted as behavioural failures.

Both constructors now insert after equal RTCs and choose the latest equal-RTC
arrival on exact-RTC lookup. For a channel query, any historical eviction on that channel
prevents extrapolation before its oldest retained reference. Any-channel
queries use any historical eviction and the oldest reference in the whole set.
The typed error is QueryBeforeRetainedWindow. No loss means the prior
pre-reference extrapolation remains allowed. A quiet channel can lose every
reference to global eviction: NoReferencePoint, with no anchor reservation.
Reappearance does not forget its loss history. A new isolation/history test
arrived with the fix, not claimed failing first.

Per-channel storage is VecDeque: age eviction pops the sorted prefix and count
eviction pops the front. No remove(0) shifts; geometric compaction amortizes
copies. Global selection still orders by (raw RTC, channel ID). Out-of-order
insertion remains linear, not a claim of constant-time insertion. Loss history
is a BTreeSet of evicted u16 channel IDs (at most 65,536 IDs), independent of
file length; it contains no reference values. No process-RSS claim is added.

All 21 focused tests pass. The fixed-seed differential makes 2,400 retained-
window comparisons, bounded versus age-only without eviction, including both
channel and any-channel queries; all agree. Pre-window probes reject after
loss. Eleven focused runtime mutants were individually killed: missing window
guard; missing age/count channel history; global loss incorrectly tainting an
unaffected channel; rejecting the exact retained edge; ignoring any-channel
loss; inserting duplicates first; reading duplicates first; ignoring the cap;
wrong global oldest order; and retaining oversized backing allocations.
Sources restored; fmt, focused suite, library Clippy and matrix check pass.
Full CI is delegated to the new PR-head run. No whole-requirement coverage
increase or README/release-gate lift.
