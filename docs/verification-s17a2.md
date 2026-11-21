# S17a2 — CLI JSON inventory report and library boundary

Stacked on frozen #76 at8da6a46; do not change that branch. Owner approval
2026-11-20: **serde_json = "1" in irig106-time-cli ONLY**, for report output.
No library serde/JSON API or forwarding. Cargo.lock pins1.0.149 (already present
for dev tools); CLI is the only new normal edge. `cargo info` reports MSRV1.68;
CI now checks the CLI on1.85 and runs cargo-deny with --locked --all-features.

## Scope and clause evidence

- L1-CLI-006 -> L2/L3-CLI-106: draft schema, summary/channels/jumps JSON over
  the same immutable Report as text, tool/path/size/effective-option provenance,
  explicit units, deterministic ordering, decimal wide values, bounded samples.
  See [schema contract](cli-report-schema.md). Not full qualified basis or all
  renderers/goldens; no whole requirement marker or new covered status.
- L1-API-010 -> L2/L3-API-110: production root cargo tree --all-features,
  normal/build edges only, rejects serde/serde_core/serde_json by real package
  name even when dependency edges are renamed. Dev-only tooling is excluded.
  This strengthens the existing removed-feature check; no feature is restored.
- CLI-103/107: strict --format text/json for three report commands; usage before
  input, writer/flush errors remain4 and threshold-only findings remain1.
  Text diagnostic ordering retained: channels before rendering, jumps after;
  summary findings are in the report. Same scan/configuration for both formats.

Tests-first ee7ff48 includes approved CLI dependency/lockfile and tests, not
production JSON code. Two runtime failures at the missing option; strict bad
options already passed. Wide/model assertions arrived with implementation,
not tests-first. Initial offline boundary attempt lacked optional Jiff sources;
fetching dependency metadata resolved that environment gap, not a code defect.

Focused tests: three JSON integration tests, wide/schema-field projection unit,
root boundary test. u64/usize above2^53 and i64MIN emitted as strings; generic
helper i128/u128 extrema separately. Schema field-set/unit checks are NOT a full
JSON Schema validator. No year/scale guess is hidden: clocks are labelled
unqualified inventory; complete qualified raw/stated/assumed basis and stale
leap/TFMT/reset goldens remain subsequent S17a work. Schema is draft/unfrozen;
the owner's single final schema review has not occurred.

Six negative probes: root serde and root serde_json each cause the dependency
boundary assertion to fail; JSON numeric count, numeric signed residual,
numeric file size, and swallowed writer failure each fail runtime assertions.
Temporary manifest/lock/source mutations restored; none committed. Local fmt,
focused tests/mutants and touched CLI Clippy only; full matrix, locked deny and
unchanged 32MiB S16 gate via PR CI. Coverage stays198 tested/206 verified of288.
No Review/Design fields changed, no publishing (publish=false untilS20).
