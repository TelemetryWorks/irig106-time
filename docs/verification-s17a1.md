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
   coordinator approval is required before choosing/adding a version. No JSON
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
