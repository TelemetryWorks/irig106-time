# Package-local compile-time assets

Base main863ab90 after the virtual-workspace move (#82). Small packaging-only
follow-up before further S17b; no wire, conversion, report/schema contract,
version, dependency, finding level, Review or Design change.

The canonical stable v1/revision2 schema now lives at
apps/irig106-time-cli/schema/irigtime-report-v1.schema.json. CLI source and tests
include it within their package. The existing docs/schema path remains an
identical generated snapshot (so the frozen public schema identifier stays
valid), never a separately edited source. scripts/sync-cli-schema.py regenerates
or checks it; the workspace guard checks byte identity. Schema bytes are unchanged.

The library's two usage-snippet tests remain enabled. Their compiled example was
already package-local; the comparison now reads a packaged Markdown fixture
instead of root docs/usage.md. A repository sync test compares the two Rust blocks
in the fixture with section13 of the guide. Existing tests still compare them
against the compiled example; nothing is ignored or removed. ADR-0022 explicitly
places Cargo benches at crates/irig106-time/benches/, not top-level bench/.

Tests-first a72afc1: the new literal-include closure guard fails on the external
library guide include; schema/fixture checks fail because their package-local
files do not yet exist. After correction all6 workspace checks pass. Focused
Rust execution: the2 existing library snippet tests and10 CLI JSON tests pass;
fmt, schema-sync, trace and diff checks pass. No full local matrix or packaging
build: those builds are intentionally run in CI under the compute rule.

## CI proof

scripts/verify-packages.py invokes cargo package -p irig106-time
-p irig106-time-cli --locked WITH normal verification enabled. Batch selection
lets Cargo stage matching interdependent workspace packages. No no-verify,
publication, tag, credential or stored Cargo configuration change is used.
Cargo's documented package step extracts and builds the archive, not the source
tree: https://doc.rust-lang.org/cargo/commands/cargo-package.html.

It additionally extracts both actual .crate archives into an owned temporary
directory outside the checkout and runs cargo test --all-features --no-run on
both, so test-only includes are checked too (normal package verification alone
need not compile cfg(test) code). Both package file lists assert the new assets.
Every literal Rust include is also checked to remain inside its own package.

For isolated CLI test compilation only, a temporary command-line patch selects
the just-verified packaged library, not the checkout or the old published0.7
prototype. Only that temporary fixture's lock may update its path edge. The
workspace lockfile is asserted unchanged; no patch is written to manifests or
persistent config. Actual S20 release still publishes the matching library first.
Temporary extraction is cleaned up on failure/success. Jobs capped at2; normal
platform/MSRV/fuzz/deny/memory checks remain in CI. Run/head results are reported
in the PR description, not fabricated in this record.
