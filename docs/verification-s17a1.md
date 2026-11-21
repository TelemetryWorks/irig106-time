# S17a1 — bounded report view and text labels

Base main 5702b18 (#75 merged). No Review/Design field changes. No new
requirement marker: partial clauses only, matrix stays 198 tested /206 verified.

## Reviewable split and dependency proposal

1. This PR: immutable report projection shared by summary/channels/jumps;
   readable jump Expected/Actual and exact observed NTF/TS labels.
2. Versioned CLI JSON report/schema plus tool/input/options provenance, using
   this report data; deterministic ordering, wide integers as decimal strings.
   Propose **serde_json in the CLI only** for correct string escaping and JSON
   rendering, with no library serde forwarding. No dependency is added now;
   owner approval 2026-11-20 applies to the next slice only; #76 adds none. No JSON
   loading API is proposed. Schema review occurs once before freeze, not here.
3. Complete qualified report/basis wiring and requested missing-year, stale
   leap-table, inferred/default TFMT, GPS/network and reset goldens. No TMATS
   dependency is authorized or added; joining needs its own API proposal first.

## Clause audit

| Chain | Covered here | Still required |
| --- | --- | --- |
| L1-CLI-004 -> L2/L3-CLI-104 | Same bounded scan, immutable report for three views; no copies of references/packets | Fully qualified joining, all command report envelopes |
| L1-CLI-005 -> L2/L3-CLI-105 | Summary/channels distinguish NTF0/1/2 and TS0/1, native GPS control | CSV/JSON variants; qualified basis across changing declarations/formats |
| L1-CLI-006 -> L2/L3-CLI-106 | Shared typed read-only model; BTreeMap order, bounded finding/jump samples, human jump clocks | Versioned JSON/schema/units/exact integers, provenance, schema review and remaining full goldens |
| L1-CLI-007 -> L2/L3-CLI-107 | Injected failing writer for each report command remains exit4 | Existing S16 evidence remains; no new whole coverage claim |

The model borrows bounded maps/sample slices and copies scalar counts; no new
packet history or extra correlator. Renderers cannot mutate scanner state.
Observed Format 2 inventory now survives conversion failure (including NTP era
needs-context and invalid TS); no fake F1 source/date/leap labels. Native scale
labels describe the wire: UTC for NTP, TAI for PTP. Existing PTP UTC conversion
still uses the leap table; complete currency/assumption basis and qualified
admission are not claimed by these inventory labels. No host-clock inference.

## Hand-derived fixtures and honest evidence

- 106-24R1 §11.2.3.2 Fig11-12 (11-23/PDF31), Fig11-13 (11-25/PDF33):
  day100 12:00:00 ->12:00:06 at RTC+1s, so expected100:12:00:01.000000000,
  actual100:12:00:06.000000000, delta+5000ms. FMT5/SRC1 is 0x51 (GPS).
- §11.2.3.3 Fig11-15..17 (11-27/PDF35): NTF7..4 values0/1/2 and TS3..0
  values0/1 generate 00/01/10/11/20/21. Eight-byte bodies are seconds then
  fraction/nanoseconds; PTP1483315237 TAI minus37 =1483315200 UTC (2017-01-02).
  No library encoder/decoder supplies expected bytes; prior S13 page audit
  supplies citations, not a new rendered-page inspection.
- Tests-first 7150516: two runtime failures on main (raw-nanosecond jump line;
  missing NTF/TS inventory). GPS control and output-failure control already
  pass on main; no claim they failed first. The model projection test arrived
  with implementation, not tests-first.
- Focused tests pass: four integration tests plus one projection test. Five
  runtime mutants killed: raw jump count, PTP2008 labelled2002, status always
  Valid, omit unconverted inventory, discard finding counts. Source restored
  after each; focused tests pass afterward. No mutation code is committed.
- Local fmt, touched CLI Clippy, trace/diff checks; full matrix and unchanged
  S16 32MiB reset/steady RSS gate run only in PR CI; PR checks provide the run evidence.

Publishing stays disabled until S20. No schema is frozen by this PR.

## PR76 delta review

Tests-first 3181a81: five new regressions fail at the reviewed head: mixed-table
literal golden, 365/366 rollover, reference-free query count, fractional/basis
retention, reference-free segment finishing (four failures in the first run,
then the fifth added/tested separately). Nine integration tests now pass, plus
the existing immutable projection test. The earlier NTF prose test now checks
compact columns in channels and the long native-scale labels in summary.

Columns have fixed widths for Source/Format/Date/Leap, short Proto/Status/Scale,
counts and First clock. Last is the final column. The mixed golden is derived
from explicit human cell values, not captured CLI output: F1, NTP, PTP2008,
RSVD3. First/Last independently retain their basis, including UTC (from TAI via
leap table) across a later F2->F1 change. Unknown F1 scale is labelled unknown;
FMT4/5 are stated UTC/GPS. Nine fractional digits are retained.

Coordinator's year-boundary decision is reader/report policy: for forward,
nonzero RTC and end-day365/366 ->day1, choose the cyclic explanation only if
closer than the linear clock jump and within the previously adopted half-year
bound. Known year/LY constrains365/366; otherwise consider both. Normal crossings
have zero residual. A +5s correction remains a jump. Render the appropriate
expected/actual phase as001 (+1 year); do not modulo the unrelated endpoint.
Insufficient multi-year display context is labelled calendar context required.
First/Last's offset is an observed within-segment phase, not an invented year;
reset restarts this phase. This does not replace S08 or qualified joining.

Never-admitted inventory channels stay in the table/summary inventory, but are
excluded from per-channel correlate queries, jump counts and finish_segments.
Previously admitted channels whose cache is empty still report the accepted
NoReferencePoint/window warning; that policy is not weakened.

Nine targeted runtime mutants killed: truncate fractions to ms, RSVD->known
scale, retain F2 label on F1, query empty inventory, finish empty segment, count
empty jump channel, disable year unwrapping, drop First UTC basis, suppress real
boundary jumps. Each source edit was restored; no mutation is committed. Local
focused tests/Clippy/fmt/trace checks only; CI carries the matrix and32MiB gate.
No serde_json edge, no Review/Design change, no whole coverage gain. #77 remains
frozen and was not changed by this review fix.
