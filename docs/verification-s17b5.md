# S17b5 — CSV machine format and stale PTP adapter golden

Base main36d5a8c (#89). L1-CLI-006 requires plain text, CSV and versioned-schema
JSON; L2/L3-CLI-106 require consistent report data and documented CSV columns,
quoting and absence. None requires JSON wrapping of the csv command. The saved
coordinator plan is external (as plan-decisions-2026-11-13.md records); it is not
claimed inspected here. Apply the coordinator's current conditional instruction:
CSV remains its own machine format. csv --format json stays usage2; timeline
supplies packet JSON, correlate supplies query JSON. Description-only docs/schema
clarification initially kept revision4; the review delta below extends revision5. No new dependency.

## Actual defect found and corrected

The CLI used the approved PTP converter but discarded its currency status. It
now preserves the owner D7/RDG-012 fallback: applicable last-known offset keeps
UTC, labelled possibly out of date by a packet/channel-located TF-021 WARNING.
The message pins the converted UTC, exact raw TAI seconds/ns, applied offset,
table identity and known-current-through boundary. Covered dates do not warn;
missing applicable offset still yields no UTC/NeedsContext, not TF-021. No host
clock, extra offset or saturation. Library/table/conversion logic is unchanged.

The new human/CSV golden covers all six commands and both PTP NTFs, with default
error threshold0, exact warning threshold1, never0. The five JSON views pin the
complete finding object, code/level/channel/packet, message and kept UTC clock.
CSV diagnostics remain separate from its calendar columns; raw operands are in
bounded sampled findings, not newly promised per-row raw-evidence columns.
Summary's existing human diagnostics are stdout; other text views use stderr.
The test's initial stderr-only expectation was corrected, not production output.

Tests-only5e25fc9 had two runtime failures on main (no stale warning); covered/
missing-offset and CSV-mode cases already passed and are baseline controls.
Fixtures are hand-derived:106-24R1 Figures11-15/17, NTF1/2 and TS1; little-endian
PTP seconds/nanoseconds, primary header checksum per11.2.1.1. Unix2030-01-02 is
1893542400; plus37s last known TAI-UTC gives raw1893542437, fraction123ns. UTC
DOY2 native count86400000000123ns. The S04 pinned IERS-C72 snapshot is known
through Unix1814140799;2030 is stale. Covered control2017-01-02 uses1483315237
TAI; TAI0 has no supported offset. No encoder/captured output generates goldens.

Five isolated compiling mutants fail tests: warning removed; Covered mislabelled
stale; TF-021 lowered to Information; raw TAI lost; UTC discarded. Source restored.
Focused tests, fmt, trace/schema-sync/diff checks pass locally; package/security,
platform and dense RSS evidence is from PR CI only, not a local full matrix.

## Clause status and remaining work

| Lineage | Evidence here | Still open |
| --- | --- | --- |
| L1-CLI-006 → L2/L3-CLI-106 | Separate CSV contract, cross-command stale UTC warning/value parity, exact sampled raw/table context | Complete protocol/scale/provenance adapter coverage, qualified policy/default evidence |
| L1-TAI-004 → L2/L3-TAI-104 and L1-PTP-004 → L2/L3-PTP-104 | CLI consumes existing accepted stale fallback; library requirements already separately evidenced | No new whole-requirement claim |

No whole Requirements markers. Matrix stays198 tested/206 verified of288. No
Review/Design fields change and S17 is NOT declared closed. Next: full-adapter
protocol/status/scale and reset goldens, then a clause audit/completion summary.
Keep the full-TMATS0.1+bounded-time-view publication/approval gate (ADR-0021);
no time→TMATS dependency or invented data/time-channel association is introduced.


## #90 option(a): per-conversion basis, revision 5

Coordinator decision: stale UTC must be labelled at the displayed clock, not
only in bounded finding samples. Revision5 adds the approved basis enum value
`derived, stale leap table`. Text uses the exact stale-table/currency suffix in
summary, channels, timeline and correlate. CSV remains unchanged/basis-free;
its stale status is available through findings and JSON views.

Tests-only751cb99 reproduces both review gaps. The coverage-edge pair uses
PTP TAI1814140836 and1814140837 (+123ns): UTC1814140799 (2027 DOY178 23:59:59)
is covered including its fraction; the next UTC second is stale. First/last,
segment, timeline and any/per-channel query clocks must distinguish them.
A reset fixture verifies private query records and a minimal revision4 consumer
using the frozen five-value enum set: supported major document, new enum view
unsupported, no document error. Added implementation-side tests cover three
reset windows (covered/stale/covered), last-state independence, and reset-own
packet rows. These extra tests are not claimed as tests-first failures.

No library or selection policy change. A bounded metadata deque mirrors the
legacy streaming correlator's ordering, ties and post-eviction channel lengths;
empty queues are removed and excess capacity compacted. Target brackets retain
basis alongside their existing bounded candidates and replay identical insertion/
search ordering. Differential tests compare metadata against native numeric
selection markers for equal RTCs, ties, late arrivals, count/age eviction and
clearing; they protect this metadata-only mirror if library internals change.
PTP basis follows the chosen reference; unknown non-PTP lineage does not inherit
another point's label. No data-to-time-channel relationship is invented.

Basis passes through native query preparation and bounded private scratch
records, then the existing one-row-at-a-time JSON emitter. No all-record DOM.
Revision4 inventory/row/query fields retain their types/units; existing goldens
change only revision5 unless a PTP clock now has the required basis. No Review
or Design fields change, no full-requirement markers or coverage increase.
Local validation remains focused tests/mutants, fmt, trace/schema-sync/diff;
platform/security/package and memory evidence comes from PR CI.

Five additional compiling label-plumbing mutants fail: stale JSON marked covered,
live row metadata dropped, target basis dropped, private query basis lost, and
query-answer basis ignored. Source restored. The prior TF-021 severity/content
and default/warning/never tests remain passing; covered and missing-offset
controls stay distinct. No new whole-requirement marker or coverage increase.


## #90 selected-reference regression and covered-output acceptance

The mixed-channel fixture supplies IRIG-B ch7 and stale PTP ch9 at equal RTCs,
in both arrival orders, then a decoded PTP probe after both anchors exist.
Fig11-13 supplies DOY100 12:00:00/01 BCD; Fig11-15/17 supplies stale PTP2008
TAI1893542437/38 (+123ns). The numeric correlator's lower-channel tie winner is
IRIG. Every IRIG-resolved row has no PTP basis in JSON or text, even when the
row's channel is9 with a stale latest reference. The new test passes unchanged
production code; it is a regression guard, not a claimed failed-first defect.
The compiling mutant that always uses decoded rows' own-channel latest basis
fails the new assertion. Checked independently with each packet order first;
both fail with stale UTC basis on an IRIG-resolved row. Source restored.

Coordinator acceptance explicitly includes covered-output changes: timeline
and correlate text gain the covered-table suffix; their JSON row/answer clocks
become UTC / derived, leap table. CHANGELOG and schema docs now say this plainly.
No schema/production-code/requirement-marker change in this delta; coverage stays
198 tested /206 verified of288. Local validation: focused stale tests and the
mutant, fmt/diff checks. Remaining checks are from the new PR CI run.
