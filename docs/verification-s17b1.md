# S17b1 — packet row projection and CSV column contract

Base: main 1d6bbc7. Slice scope: extract the existing timeline/CSV observation
into report_rows::PacketRow and render both from it; document the 15-column CSV
schema, units, quoting and absence; fix counter-only rows with 14 fields. The
include/package guard now checks literal include! and #[path = "..."] as well
as include_str!/include_bytes!. No dependencies or library changes.

## Clause coverage (not whole-requirement coverage)

| Requirement lineage | Covered here | Remaining |
| --- | --- | --- |
| L1-CLI-005 → L2/L3-CLI-105 | Shared row foundation only; no new protocol/scale clause claimed | Protocol/status/scale projection for row commands, correlate shared row model, qualified joining |
| L1-CLI-006 → L2/L3-CLI-106 | Documented CSV columns/units/quoting/absence; exact decimal RTC/ns; text N/A | JSON row envelope/revision 3, complete provenance and report parity, remaining stale-table/reset goldens |

No whole Requirements markers are added. TRACE-MATRIX is regenerated with no
coverage delta: 198 tested / 206 verified of 288. No Review/Design fields change.
The frozen JSON v1/revision2 schema and emitted inventory JSON are untouched.
Future additive JSON row/options fields will increment schema_revision to 3.

## Tests-first and sources

Tests-only commit 6242a0c precedes implementation. On main, the counter-only
regression fails: 14 fields versus header15. Both include!/path negative probes
fail because neither escape is detected. The observed-clock parity baseline
passes on main after correcting the test's short labels from `Time F1`/`Analog`
to the actual `Time-F1`/`PCM-1`; those fixture spelling mistakes are not defects
or failed-first evidence. The field-escaping unit test was added with the
extracted renderer, not claimed as tests-first.

Decoder-independent literal bytes: 106-24R1 §11.2.3.2 Figures11-12/13, FMT4 UTC,
DATE0, fraction0x12 →120000000ns, seconds04/minutes00/hour12/DOY100, year unknown.
§11.2.1.1: 24-byte header, Data Length10, two filler bytes excluded, header
word-sum checksum hand-computed. Expected packet0/offset0/channel7/RTC0 and
100:12:00:04.120000000 are pinned in both projections. The no-time PCM packet
has zero Data Length and a hand-summed header in tests/common/mod.rs.
Both commands' finding diagnostics must also agree for each fixture.

RFC4180 §2 items5–7 supplies quoting/quote-doubling, not IRIG106. The existing LF
record dialect is documented explicitly. A fixed-size row remains one-at-a-time
in CSV, timeline remains limit-bounded; no new whole-file collection or JSON
loading is introduced. Numeric text is unchanged; the only CSV data change is
the extra absent final field on counter-only rows.

## Negative checks and validation

Three isolated renderer mutants compile and are killed by focused tests:
missing final counter-only field; calendar nanoseconds replaced with0; embedded
quote doubling removed. Restored source passes. The two temporary out-of-package
include!/path fixtures are rejected; the repository's actual includes/module
paths pass (no exclusions added). Guard covers literal paths, not arbitrary
macro expansion/computed file names; actual archive compilation remains in CI.
Local: fmt, focused packet-row and escaping tests, 13 existing atomic CSV tests, seven workspace guard tests,
trace check and diff check. Full platform/security/package/memory checks are
PR CI evidence, not claimed local execution.
