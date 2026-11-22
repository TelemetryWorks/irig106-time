# S17 item5 — automated producer schema validation

Base main8e6288d (#96). This is developer/CI tooling only, implementing the
coordinator's item5 approval. No Rust source, Cargo dependency/lockfile,
shipped package contents, schema version/revision, Review or Design changes.

| Requirement chain | Covered clause | Still partial |
|---|---|---|
| L1-CLI-006 → L2/L3-CLI-106 | Automatically validate emitted reports against the current packaged schema, including its version, nested units/types and closed producer enums | Full qualified provenance and native sessions/quality are not claimed by a shape validator; remaining S17 items6/7 and the TMATS gate stay open |

JSON is our CLI contract (ADR-0019/Q5), not an IRIG106 wire representation.
Use jsonschema4.26.0's Draft202012Validator with check_schema before instance
validation. Sources: [package/release](https://pypi.org/project/jsonschema/4.26.0/),
[Draft2020-12 metaschema](https://json-schema.org/draft/2020-12/schema).
An empty referencing registry prevents remote reference retrieval; our report
$refs resolve locally, including before the release $id URL exists.

## Reproducible dev-only installation

scripts/requirements-schema.txt pins jsonschema and all five transitive
packages, with wheel SHA-256 values checked against PyPI JSON release metadata
and the downloaded bytes. CI installs using --require-hashes --only-binary=:all:.
Hashes permit CPython3.12 Linux x86_64 CI and3.11 Linux aarch64 local probes;
other environments need separately reviewed wheel hashes. No source builds or
runtime Python dependencies are introduced. A separate altered-jsonschema-hash
probe fails pip download with DO NOT MATCH THE HASHES; the real locked install
succeeds. This is validation tooling, not a new library JSON loading API.

## Evidence and tests-first honesty

Tests-only aeae87d: four tests do not pass on main: one CI-wiring assertion
failure and three import errors (validator/module/Python tooling unavailable),
not four reproduced product-behavior failures. The explicit view-set assertion
was added with implementation. Five focused tests now pass, including both full
inventory goldens and table-driven invalid reports (wrong revision, unknown key,
wide integers as numbers, invented nested basis and enabled era inference).
Malformed producer schema is rejected by the Draft2020-12 metaschema.

The live gate invokes the already-built irigtime binary30 times: summary,
channels, jumps, timeline and correlate on each of six small generated files.
Cases include unknown year/native GPS, NTP and both PTP protocols, stale leap
table, reserved NTF/TS and invalid TS, reset segments, ambiguity/counter-only
rows, no time packets, absent/declared NTP era and the zero sentinel. CSV is its
own machine format, not a sixth JSON view. Every report is ASCII and validates
against the packaged current revision8 schema; no captured output is accepted
in place of execution. Synthetic bytes are literal Fig11-12/13 and Fig11-15..17
fields with §11.2.1.1 header checksums, not production-encoder output.

Three targeted mutants fail the tests: skip instance validation, skip schema
metavalidation, omit correlate from the live views. Sources restored. These
probes demonstrate the new guard, not complete semantic mutation coverage.
Semantic golden tests, exit checks and the32MiB RSS gates remain separate.

Local checks: hash-locked install/negative digest probe,5 small Python tests,
30 live reports, targeted mutations, fmt, trace/schema-sync/diff checks. Full
platform/package/MSRV/deny/fuzz/RSS checks are delegated to PR CI. No whole
Requirements markers are added; coverage stays198 tested/206 verified of288.
