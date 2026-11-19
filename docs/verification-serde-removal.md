# Library serde removal — ADR-0019 implementation

Baseline main d1278b5 includes approved JSON-boundary PR50 and Jiff PR51.
Owner2026-11-17: library Rust data only; CLI owns JSON report output under
L1-CLI-006. No new release deferral, Review/Design field, finding level,
publishing policy or shared-types feature is changed.

## Removal and retained boundaries

Remove the serde feature, direct optional dependency and irig106-types/serde
forwarding. Remove src/snapshot.rs and its four helper modules, AnswerOutput,
serde derives/field adapters/serialize_struct lists, private wire DTOs and
serde-only Serialize/Deserialize implementations. Remove six serde-only
integration-test files (31 tests), their token-tree helper, and ten serde-only
compile-fail doctests with the JSON module. No serde reference remains in src.
The root lockfile drops the library/shared-types serde dependency edges;
serde packages remain transitively for dev dependencies. This is not a library
serde dependency. CLI/fuzz lockfiles require no change; their builds run in CI.
`irig106-types`0.1.0 and its own optional serde feature remain untouched.

Keep S12a2's AnswerView, AnswerBasis, CounterBasis, native policy/evidence,
all adapter envelopes and replay(). Keep validated Rust constructors and binary
wire encoders. Native source token inspection against main (after excluding
serde-gated items/attributes and formatting, ignoring comments) matches across
all49 retained root Rust files. The new JSON-boundary comments do not alter
native signatures, calculations, validation, policy or fallback behaviour.
This mechanical inspection is supplementary, not a replacement for tests/CI.

All three serde-specific workflow commands are removed (stable tests, MSRV,
WASM); existing all-features now means std+jiff. Keep default/no-default and
optional Jiff tests, MSRV1.85, WASM without defaults and with Jiff, CLI, fuzz,
clippy/docs/deny/trace checks. Do not create a hidden compatibility serde flag.
No CLI JSON renderer/schema/load API is implemented here; CLI report work is
still a0.8.0 requirement. CHANGELOG has one consolidated library serde-breaking
entry/migration target; historical published0.1–0.7 entries remain history.

Queued nits are included: README removes the serde bullet and now correctly
states required irig106-types/optional Jiff; project_structure updates the
removed chrono module, dependency row and dependency diagram to Jiff/no-serde;
test_index uses Jiff doctests/adapters; architecture Targets explicitly lists
`--no-default-features --features jiff` for WASM. Prototype counts in those
historical documents are not claimed to be current rebuilt coverage.

## Tests-first and focused evidence

- e8bf61b adds only a feature-rejection test before removal is committed.
  Its first run fails because Cargo accepts serde on main. However, that first
  draft used `metadata --no-deps`, which does not validate requested feature
  names: the test also failed after feature removal. Its initial failure is
  NOT sufficient boundary-regression evidence.
- ad05fe8 corrects the test to resolve the actual Cargo feature graph without
  compiling. Cargo rejects the absent feature before compilation. Diagnostic
  matching accepts either Cargo wording while requiring an absent-feature
  message naming serde; unrelated resolution failures are not accepted.
- The corrected test passes after removal. A targeted mutation restoring even
  an empty `serde = []` feature fails at runtime (Cargo accepts it); manifest
  is restored afterward. No claim of broad algorithm mutation testing: native
  algorithm code is unchanged.
- Focused local run: `cargo test --test library_boundary --test s12a2_answers
  --test s04_leaps --locked` passes29 cases:1 boundary,15 native envelope/replay
  and13 native leap-table cases, jobs2/test threads2. Owned offline/stream/
  own/stamp/word replay, interpolation, policy/fallback, tamper guards and raw
  context remain covered by the retained native suite.
- Local fmt is run. Full feature/MSRV/WASM/CLI/fuzz/deny/clippy/docs checks are
  deliberately left to PR CI per owner2026-11-18 compute policy. No local full
  matrix or unchanged expensive checks are rerun; final report cites CI run ID.

## Honest traceability

Regenerate TRACE-MATRIX, removing the deleted s04_serde validation artifact
from TAI-001/003 and their L2/L3 lineage. The native s04_leaps/s04_existing tests
still verify those requirements. The other removed S12b tests had no whole
Requirements markers. No requirement had only a deleted serde artifact;
no obsolete artifact remains as coverage. L2/L3 rollup stays192/288 (delta0).

No new whole-requirement marker is assigned to the narrow boundary test.
API-110/POL-103 remain at their prior rollup pending full clause/inspection
review; the negative-feature probe alone cannot claim native data/provenance/
replay, target or dependency guarantees. Current clause evidence:

| Requirement lineage | Evidence covered here | Boundary |
|---|---|---|
| L1-API-010 -> L2/L3-API-110 | Serde feature/dependency/forwarding/JSON module removed; feature resolver rejects serde; native S12a2 replay retained | No CLI schema/output implementation or library JSON load obligation. |
| L1-POL-003 -> L2/L3-POL-103 | Typed immutable policy/basis and native replay are unchanged; positive replay suite passes | CLI report provenance/format is CLI-106, not this library. |
| L1-API-005 -> L2/L3-API-105 | No library serde dependency or shared-types forwarding; optional Jiff retained | Shared types' own feature is outside this removal. |
| L1-API-007 -> L2/L3-API-107; L1-REL-002 -> L2/L3-REL-102 | CI feature/target matrix updated to std/no_std/Jiff; no serde commands | Compiler/platform verification comes from this PR's GitHub CI. |
| L1-CLI-006 -> L2/L3-CLI-106 | JSON ownership/migration documented only | CLI JSON renderer and schema remain release work; no coverage added. |
