# S17b4 — streamed correlate JSON, additive revision 4

Base main b0053f5 (#88). Correlate JSON projects the same native Query as text.
New options.query_rtc and report.correlation require additive revision4; previous
views emit null for both. Existing types/units/meanings stay intact. Queries
have no packet identity: report.rows remains null instead of inventing one.
No dependency, library, Review/Design or counter-selection policy change.

| Lineage | Clauses covered | Still open |
| --- | --- | --- |
| L1-CLI-006 → L2/L3-CLI-106 | Query JSON, wide strings, text/finding parity, ordinary/reset/error/no-reference goldens, streaming projection | CSV JSON, complete qualified provenance, stale-table and full multi-command reset goldens |

No whole-requirement markers; regenerated matrix stays198 tested/206 verified
of288. Native source scale/basis remain explicitly unknown/unresolved; counter
query labels are not data-channel association evidence. No TMATS dependency.
Schema definitions and emitted key sets match in focused tests; this is a shape
check, not a claimed independent full metaschema validation. Docs snapshot syncs
from the packaged canonical schema. Inventory golden edits are metadata-only:
revision4, query_rtc=null and correlation=null, not regenerated captured output.

Tests-only55e93b8 has five runtime failures on main: four correlate JSON tests
reject --format and the lazy-projection test lacks the correlation object.
All pass after implementation. Hand-derived Fig11-12/13 (§11.2.3.2) and primary
header (§11.2.1.1) fixtures from S17b3 pin ordinary RTC5s clocks12:00:05 and
13:00:05, reset candidates12:00:05/12:01:05 and an unchanged second channel.
Day100 starts after99days; native count is (99*86400+seconds-of-day)*1e9.
Year remains unknown. No encoder or captured output supplies expected values.
Full correlation objects pin every field; schema key sets, ASCII/determinism,
text stderr/exit parity, context-error counts, unresolved segment body and
no-reference reason are asserted. Strict format errors precede input and a
failed writer remains exit4. Default text and its merged reset ordering stay
unchanged. The lazy unit requires empty answers metadata, then actual emission.

Metadata contains only any-channel clock/issue and an empty answers marker.
The emitter traverses channel-bounded native answers and disk-backed reset
records, building/sorting/emitting one JSON answer at a time. No expanded answer
DOM or all-segment queue. CI adds correlate JSON to dense reset/steady workloads
under32MiB, checking complete inventory, answer counts and any-channel withholding.
Large probes run only in CI. Local work is focused tests/mutants, fmt, trace,
schema-sync/diff and small Python guard/workload checks, not full CI equivalent.

Six isolated compiling mutants fail the new tests: query RTC emitted in ns,
reset label lost, segment forced1, reset answers omitted, context issue dropped,
and eager answer DOM built in metadata. Sources restored. The generic clock
projection and canonical ASCII formatter are reused; existing inventory, packet
and correlate text regressions pass with only additive metadata golden edits.
