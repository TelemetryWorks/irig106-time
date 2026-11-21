# S17a prerequisite — CLI JSON dependency and library boundary

PR77 merges main4b5f559 ordinarily after PR76. The merge was conflict-free.
Scope is only the approved dependency, library boundary guard and supporting
checks/docs. Versioned JSON report/schema implementation waits until PR77 merges.
The earlier prototype is preserved at68bbaba and local branch
work/time-s17a-json-preserved; it is not in this PR's final diff.

Owner approval2026-11-20: **serde_json = "1" in irig106-time-cli ONLY**, for
JSON report output. Cargo.lock pins1.0.149 (already resolved by dev tooling);
CLI is the only new normal dependency edge. `cargo info` reports MSRV1.68;
CI explicitly checks the CLI on1.85 and cargo-deny with --locked --all-features.
No library serde/JSON API, feature or forwarding; publish=false stays untilS20.

## Partial clause evidence

L1-API-010 -> L2/L3-API-110: root cargo tree --all-features --target all, normal/build edges
only, rejects serde/serde_core/serde_json by real package names, so renamed
edges cannot hide them. Coordinator PR77 review confirms dev dependencies
are out of scope: they never ship with the library, and criterion already brings
serde into the dev tree. The earlier dev-tree wording is withdrawn. Retain the existing
removed-serde-feature rejection test. This is boundary evidence, not a new
whole-requirement marker. Coverage stays198 tested/206 verified of288.
No Review/Design fields changed.

Negative controls from the original work: inject root serde, then root
serde_json independently (manifest and lock edge); each is rejected specifically
by the library-dependency-tree assertion. Temporary source/manifest/lock edits
were restored. An initial offline check lacked optional Jiff sources; normal
cargo tree dependency resolution resolved that environment gap, not a defect.
The final boundary check does not require an artificially warm offline cache.

Local checks: fmt and the two focused library_boundary tests. Full platform,
CLI MSRV, locked cargo-deny and unchanged S16 32MiB RSS gate via PR CI.
This PR neither implements JSON output nor requests the owner's schema review.
ADR-0019 and Q5 govern the subsequent CLI-owned versioned report/schema slice.

## Target-specific guard regression

Tests-first64ff1ee reproduces the Linux bypass: a renamed Windows-only serde
path dependency is absent from the host-filtered tree. The same library_tree
command helper is used by the real guard and the regression fixtures. Add
--target all while retaining --edges normal,build and --all-features/--locked.

The table-driven regression uses independent owned temporary packages and
local empty stubs; cargo generates fixture locks offline, never compiles stubs
or contacts a registry for them. Cases: cfg(windows) normal serde, wasm32 normal
serde_json, target_os=none build serde_core; all must be present by real package
name. A dev-only serde control must remain absent. Cleanup runs on success or
panic. No stub or transient lock is committed. All three library_boundary tests
and fmt/trace/diff checks pass locally; the full matrix runs in PR CI.
