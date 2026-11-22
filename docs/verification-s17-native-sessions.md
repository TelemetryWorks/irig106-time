# S17 item6a — bounded native session/span foundation

Base main7dd0882 (#97). Item6 is split: this foundation supplies bounded native
header/session tracking; a following PR integrates summary spans and native
quality. CLI output, schema revision8, query/reference selection and the existing
30s/run per-channel reset heuristic are unchanged. This PR does NOT close item6.
No Review/Design, dependency, finding level or TMATS decision changes.

| Chain | Evidence in this PR | Deferred |
|---|---|---|
| L1-CLI-004 → L2/L3-CLI-104 | Bounded native session/header-span API needed by the joining/report loop | CLI summary integration, native reference admission/quality, qualified declarations |
| L1-COR-005/007 → L2/L3-COR-105/107 | SessionClock shares TimelineBuilder's native nearest-counter/session positioning; independent StreamTimeline parity controls | No new reference/correlation/quality guarantee claimed |

## Native policy, not the compatibility reset heuristic

The source is the existing native OfflinePolicy/contract5.2,5.4,5.6 and ADR-0010.
The contract defines a reset as "the counter goes backwards by more than the
late-packet bound"; default1100ms combines Ch10 §10.6.1's1000ms stream commit
and100ms generation policy basis. These are reader settings, not a wire reset
flag. No new standards page review is claimed. The standard's free-running
recorder counter is shared by all channels; this API consumes every header,
not just a selected time channel. Declared data/time association remains gated.

SessionClock extracts the exact native calculation already in TimelineBuilder:
nearest signed48-bit displacement against session high-water; exact half range
is ambiguous; a short forward wrap stays in the session; backward displacement
STRICTLY beyond late_ns starts an inferred session iff infer_sessions is on;
explicit boundaries take priority. TimelineBuilder now calls that shared pure
calculation. The stream engine is unchanged and provides an independent parity
control in both Standard and Interpolate modes.

SessionClock retains one fixed-size active SessionSpan, no ID set, reference
history or completed-session queue. The caller receives each closed span once
and can stream it to a sink; active() supplies the final EOF span. First/last
mean observation order; low/high are unwrapped extrema across all headers, so
late packets never lower the high-water mark. Counts and ticks are typed, not
JSON. Start provenance distinguishes initial/explicit/inferred. Input IDs must
strictly increase (stream convention); validation/arithmetic errors leave state
unchanged. The policy's actual late bound/infer flag are readable. SessionClock
does not perform calendar inference, admission, quality/drift or findings I/O.

## Honest tests-first and negative evidence

Tests-only80f425a fails compilation on main's missing session module/API, not a
reproduced runtime defect. Three new integration tests now pass. They compare
positions with both native engines on exact1100ms versus1100ms+100ns backward
steps, disabled inference, a custom200ms bound, forward wrap, short late headers
and explicit boundaries. At most one span is stored (<512bytes on this target,
no Drop); allocation-free scope is also established by inspecting primitive/
Option-only fields and nonallocating operations, not allocator instrumentation.
Span tests pin high/low ticks, first/last IDs, packet counts, closed-span handoff
and boundary provenance. Bad policy, duplicate/decreasing IDs and ambiguous half-
wrap leave the state intact; an explicit half-wrap boundary succeeds instead.

Ten compiling mutants fail at runtime: inclusive late bound; forced forward
order; explicit boundary not bypassing arithmetic; infer=false ignored;
high-water replaced by last; low-water replaced by last; count increments by2;
equal ID accepted; inferred provenance changed; closed span omitted. Restored.
Existing focused offline22 and stream17 tests pass; root-library Clippy/fmt pass.
Full package/platform/MSRV/no_std/WASM/security/fuzz/RSS remain PR CI evidence.
No whole Requirements markers: coverage remains198 tested/206 verified of288.

The #97 nit is included: test_report_schema.py pins every fixture's options by
label, including ntp-era1=[--ntp-era,1] and ntp-sentinel=[--ntp-era,0], others[].
An era1→era0 fixture-option mutant fails; six tooling tests pass after restore.

## Remaining item6 and release gates

The next integration must stream native spans to summary text/JSON and compute
native per-session quality without retaining all packets/references. Preserve
both the native1100ms grouping identity and the existing compatibility query/
reset behavior; do not relabel heuristic channel segments as native sessions.
Library quality reasons, scales, unknown years and rejected references must
remain honest. No native CLI summary/quality clause is marked covered here.
Item7 docs/migration and the full published TMATS0.1 + bounded time view + owner
approval gate remain open; no dependency or source association is invented.

CI45005105600 caught the public-type inventory omission on all three OSes before
remaining library tests: three new values and SessionClock were unclassified.
Register the Copy/Eq/Hash span/start/update values with actual trait assertions,
and the mutable clock with controllers (its Clone/Debug/Send/Sync assertions are
in the new tests). The guard remains exact; no wildcard/exemption was added.
The failed run was canceled to stop its remaining RSS work. Focused inventory
and session tests pass after correction; the succeeding run is final CI evidence.
