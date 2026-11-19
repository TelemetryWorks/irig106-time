# S16b2a — command transport integration (partial CLI-104)

Base main 5202eb3. All six commands now open a File and use one PacketReader
scan/dispatch loop; no whole-file read, mmap or library I/O. Capture is at most
12 DATA bytes for time packets, zero for others; filler never reaches decoders.
Offsets remain u64. Every command analyses before rendering/opening output;
timeline's limit bounds retained rows, not scanning, so later structural input
cannot be hidden by the limit. No CLI publication/tag or dependency added.

## Clause ledger and release gate

L1-CLI-004 -> L2/L3-CLI-104: transport/dispatch, checksum recovery count, bit 7,
Data Length and file-level no-time TF-001 are covered here. No whole-statement
Requirements marker: qualified session/setup joining, approved time-API and
own-time adapters, common report/basis remain subsequent S16b slices. Current
legacy time/correlation processors are deliberately not claimed rebuilt here.
L1-CLI-007 -> L2/L3-CLI-107: preserve analyse-before-render and fail-on levels;
empty/no-complete-valid-packet input is now 3, not a warning-only file. Cut-final
headers/packets and unsynced padding AFTER an established stream stay findings.

Six real commands plus the reader worker run in CI on generated 384 MiB input
(24,576 non-time 16 KiB frames), each with a 64 MiB RSS ceiling; summary count and
TF-001 are checked. Output goes to temporary files, not a parent-memory CSV.
This proves buffered input on that workload, not bounded reference/finding/
report state on all recordings. Those legacy collections still need audit.
README now states the remaining report-memory risk; the release gate stays.
Trace totals remain 198/288 tested, 206/288 verified. No Review/Design changes.

## Tests first, fixture migration, mutations

- dfc40aa: three runtime failures on main (bit 7, checksum recovery, no valid
  header). b31d380: missing typed NTF error is a compile failure, not runtime.
- All CLI fixtures now carry valid primary checksums and secondary presence
  uses bit 7, independent of format bits 3-2 (§11.2.1.1). Structural-header
  mutations recompute checksums. Tail/threshold warnings prepend a valid
  reference; output-failure/dash-path tests use a valid non-time packet instead
  of an empty file. Their original finding/exit/flush intent is retained.
- Six runtime mutants killed independently: checksum ignored; bit 2 presence;
  recovery finding omitted; reserved NTF graded error; TF-001 omitted; timeline
  limit hides later fatal input. Sources restored; six joining tests pass.
  TF-001/limit controls were added after implementation, not failing-first.
- ReservedNetworkTimeFormat replaces field-name matching for NTF 3..15 in the
  legacy tuple parser and canonical encoder. Exact variants tested; warning
  TF-006 stays. Encoder truth table updated without losing cases/markers.
- Local: fmt, touched CLI tests/Clippy, affected encoder tests and trace check;
  full matrix and generated-file RSS delegated to PR CI. No large local file.

## TMATS dependency proposal (approval/publication pending)

[Public main](https://github.com/TelemetryWorks/irig106-tmats/tree/18b34e66b276cfc4869b06bcda42ed50612e4286) has parse_with_options and
ParseOptions attribute/value caps. [Registry index](https://index.crates.io/ir/ig/irig106-tmats) and downloaded 0.0.2 source
confirm only 0.0.1/0.0.2 are published; 0.0.2 explicitly contains no functionality.
Recommend a CLI-only irig106-tmats 0.1 dependency once its functional parser is
published, using bounded setup payloads, explicit parse limits and copying only
required declarations. No time-library dependency, defaults insertion or local
TMATS checkout. Do not substitute the placeholder or silently add a git pin.
Setup-range integration remains pending that decision and functional release.

First CI 44520257330 exposed two wiring errors (Rust tests passed): implicit
fixture-module paths confused the safety scanner, and the RSS runner assumed
Cargo's executable layout. Use explicit module paths and Cargo binary artifact
metadata. The safety probe and a temporary 64 KiB seven-process RSS smoke pass;
full 384 MiB measurement remains CI-only. No policy exemptions were added.

## Delta review: confirmed framing and reporting decisions

Coordinator confirms: no COMPLETE valid packet means input 3, including a lone
packet whose primary header/checksum is valid but whose body is truncated.
After a complete packet, a truncated tail remains a finding. TF-001 keeps its
owner-approved warning level: non-time-only input succeeds at fail-on error
and returns 1 at warning/info; `--fail-on never` keeps exit 0.

Three new tests pass without production changes: lone truncated packet on all
six commands; TF-001 error/warning/info/never thresholds on all six commands;
and exactly one timeline row (packet 0, total 2) with --limit 1. Nine joining tests
now pass. Each corresponding runtime mutant fails: both no-complete-packet
checks bypassed; TF-001 lowered to Information; row bound changed from < to <=.
Sources restored and suite rerun green. No whole-requirement marker is added;
coverage remains 198 tested / 206 verified. Full matrix remains PR CI only.

### S16b2b CSV recommendation (not implemented in this delta)

Use an exclusively created, cleanup-owned temporary CSV sink, writing each
released row immediately; never collect all rows in Vec<ResolvedPacket>.
After successful analysis, rewind/copy the sink to the injected writer or
requested output and flush. Disk use grows with the requested report, RAM does
not. This keeps destination output after input analysis. Treat temporary sink
creation/write failures as output 4, retain the first fatal operational failure
(input 3/output 4), and attempt later diagnostics without replacing it. Test
input failure before destination creation, destination failure after analysis,
temporary-write failure and cleanup. No parser/dependency decision is implied.
S16b2b must also bound finding samples with exact per-level counts, adopt
bounded streaming references/online jump and quality state, and run a dense
small-packet/F1 RSS workload before removing the interim README warning.
