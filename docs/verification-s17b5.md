# S17b5 — CSV machine format and stale PTP adapter golden

Base main36d5a8c (#89). L1-CLI-006 requires plain text, CSV and versioned-schema
JSON; L2/L3-CLI-106 require consistent report data and documented CSV columns,
quoting and absence. None requires JSON wrapping of the csv command. The saved
coordinator plan is external (as plan-decisions-2026-11-13.md records); it is not
claimed inspected here. Apply the coordinator's current conditional instruction:
CSV remains its own machine format. csv --format json stays usage2; timeline
supplies packet JSON, correlate supplies query JSON. Description-only docs/schema
clarification keeps revision4 and emitted shape unchanged. No new dependency.

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
