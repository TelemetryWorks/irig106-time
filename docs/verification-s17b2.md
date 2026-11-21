# S17b2 — bounded timeline JSON, stable v1 additive revision 3

Base: main df5581c (#86). This reviewable slice activates the reserved packet-row
shape for timeline only. Text and JSON render the same borrowed Report and
PacketRow values, including absent clocks, and retain identical findings/exit
classes. CSV remains on PacketRow with its existing atomic spool. Correlate's
shared rows, CSV JSON, complete qualification, stale-table and reset goldens
remain subsequent slices. No dependency, library, Review or Design changes.

## Clause-level coverage

| Lineage | Covered here | Still open |
| --- | --- | --- |
| L1-CLI-006 → L2/L3-CLI-106 | Timeline JSON rows, effective limit, exact decimal metadata, deterministic ordering, counter-only and unknown-year projection; text parity; writer exit4 | CSV/correlate JSON and full protocol/scale/policy/source provenance, remaining qualification goldens |
| L1-CLI-005 → L2/L3-CLI-105 | Shared report foundation only; no new full protocol clause claimed | Protocol/scale/status labels across all row renderers |

TRACE-MATRIX regenerated with no delta: 198 tested / 206 verified of288.
No whole-requirement markers: this slice does not complete either statement.
Sources: ADR-0019, owner Q5, frozen-schema compatibility rules and coordinator's
S17b assignment. JSON is a CLI contract, not prescribed by IRIG106.

## Schema compatibility and honest qualification

Revision3 adds options.limit (null on inventory), activates timeline report.rows,
and adds row offset_bytes, data_type, association and direct clock. Defined
keys always appear; usize/u64 quantities remain decimal strings, small wire/date
fields integers. Existing inventory fields/types/units/enums and schema_version1
are retained. schema_status stays stable. Inventory golden changes are limited
to revision3 and limit=null, edited directly rather than captured from runtime.
Canonical schema and docs snapshot stay byte-identical; new row keys/enums are
checked against schema definitions by the literal golden test. This shape test
is not claimed to be an independent full JSON Schema metaschema validator.

Timeline emits association_unestablished and no candidates. The seam also names
association_conflicting / association_evidenced; conflicting is unresolved for
answers. Current legacy packet correlation lacks qualified source lineage, so
numeric clocks explicitly retain unknown/unresolved scale/basis. Do not borrow
scale labels from an equal channel ID, infer a segment, assume UTC or invent a
year. Inventory clocks can separately retain stated scale. Data rows after reset
or pending classification retain their existing conservative counter-only result.
The generic row issue points to detailed command findings; this is not yet a
full typed per-answer basis. No TMATS dependency or guessed association is added.

## Tests-first, fixtures and negative evidence

Tests-only 999b447: all three new tests fail on main because the strict parser
returns usage2 for timeline --format. Implemented behavior passes. Extra schema
key/enum assertions were added alongside implementation, not claimed failed-first.

Literal bytes: 106-24R1 §11.2.1.1 supplies primary header layout/checksum; Figures
11-12/13 (§11.2.3.2) supply FMT4/DATE0, DOY100 12:00:04.12 (year unknown).
Packets: PCM at RTC0, 36-byte F1 at RTC10000000, PCM at RTC20000000; offsets0/24/60.
Data Length10 excludes two filler bytes. RTC tick100ns gives a1s shift for the
last PCM answer: 100:12:00:05.120000000. Day100 begins after99days; the reference
native count is8596804120000000ns. Expected bytes/clocks are hand-derived, not
encoded or obtained from captured output. Full row keys/values and stderr parity
are pinned. Limit1 still scans all3 packets and observes the later F1 segment.
Invalid/duplicate/missing format is usage before input; failed JSON writer is4.

Six compiling one-at-a-time mutants fail focused tests: rows dropped; counter-only
kind lost; offset forced0; raw RTC replaced with ns; UTC label invented; effective
limit ignored. Sources restored. Focused new tests and affected inventory/text/CSV
projection baselines pass. fmt, schema sync, trace/diff checks and12 small Python
workspace/workload tests pass locally. Full package/security/platform/RSS jobs
are delegated to PR CI, not claimed local execution.

## Resource boundary

Timeline retains at most caller-selected --limit PacketRow records (default100).
After the review fix below, JSON projection holds one row Value at a time; no
second, expanded all-row JSON DOM is retained. Findings/counts still cover all input. Segment history remains streamed
from its private archive. CI's dense reset and steady F1 workloads now also run
timeline JSON with limit1000 under32MiB, checking row counts, complete packet/
reference counts, expected warnings and complete segment inventory. No large
fixture or expensive memory probe is run locally or committed.


## #87 review fix — lazy row emission and compatibility clarification

The initial head eagerly expanded every retained PacketRow into a JSON Value.
The reviewer measured roughly437MiB at limit100000 and1.3GiB at300000; the earlier
limit1000 RSS pass did not establish an adequate large-cap serialization bound.
Those measurements are reviewer evidence, not locally executed probes.

Tests-first23cf50e reproduces eager projection: document() contains a projected
row instead of the empty marker. The writer now intercepts that marker, builds,
sorts and emits each row immediately. Inventory rows stay null. The unit also
asserts the final artifact contains the row; the unchanged exact integration
golden pins all output fields. PacketRow retention stays the same as text;
serialization adds only one row projection, not4.4KiB per retained row.

The existing CI steady/dense fixture now additionally runs timeline JSON with
limit100000 under32MiB, measured by GNU time on the CLI itself. The harness checks
exactly100000 row packet keys across bounded read chunks, no findings, complete
input counts and a complete JSON tail, without loading that large artifact.
The costly gate is run in PR CI only. Focused lazy-projection and timeline/
inventory golden tests run locally, alongside fmt/schema-sync/trace/diff checks.

The coordinator's enum/reserved-shape clarification is recorded verbatim in the
canonical schema description and cli-report-schema.md, then synchronized to the
docs schema copy. It stays within revision3; no version/revision/status increment,
field representation, row output or text/CSV behavior changes.
