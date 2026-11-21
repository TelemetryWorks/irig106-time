# Virtual workspace move — owner 2026-11-21

Base main7d6f445f3053b187bf6f92b5b1bb20bca5bf289b, after #80/#81. Dedicated layout
slice; no further S17b work. [ADR-0022](adr/0022-virtual-workspace-layout.md)
supersedes only the earlier layout choice; names, version, features, public API,
wire parsing, answers, CLI output and publication policy remain unchanged.
The owner's CLI role is testing/simple examples. No publication/tag/merge or
Review/Design-field change is authorized here.

Reference inspected read-only: [Praevis at0ee33ca](https://github.com/Praevis-Quant/praevis-thetadata/tree/0ee33ca512664e2f3feffa219b4028e1ca0bdc1e),
root virtual workspace, crates/apps membership, inherited package metadata and
exact internal workspace edges; independent root fuzz. Its root bench/ contains
external Python drivers, not Cargo Rust benches. Here Rust benches/examples move
with their library so Cargo packaging keeps their targets; no external harness
or extra workspace package is invented.

## Moves and guards

`git mv`: src/, tests/, examples/, benches/ -> crates/irig106-time/;
irig106-time-cli/ -> apps/irig106-time-cli/; original library README -> library.
The new root README describes the workspace. Both packages carry a local README
and byte-identical Apache-2.0 LICENSE. Root Cargo.toml contains only [workspace]
and its package/dependency tables; resolver3, two members, fuzz excluded. Both
packages inherit metadata; the CLI inherits the exact internal dependency edge.
The shared lockfile and independent fuzz lockfile retain their existing graphs.

Tests-first266d00d updates the existing workspace guard (still3 tests), failing
on the old manifest's root package. It passes after the move. The19 existing
safety scanner fixtures now create their production source trees under the new
roots, retaining all negative bypass probes. Compiler/Clippy negative policy
probes and no-std I/O probes run at all relocated roots and pass. No allowance is
broadened. The library serde guard explicitly selects its member manifest for
feature rejection and retains package/target-all normal+build tree checks.
During focused checking an invalid cargo metadata -p attempt was corrected to
--manifest-path; that tooling mistake is not a library behaviour change.

All129 pre-move Rust files remain.125 are byte-identical; exactly four differ:
CLI report_json.rs and s17_json.rs adjust schema include paths; library
s14_usage_snippets.rs adjusts its root-guide include; library_boundary.rs selects
its member manifest. No test function is removed or renamed. All original
source/test/assets/examples/benches are also present in the package lists.

Trace regeneration changes ONLY paths: after normalizing old src/tests and CLI
roots to new paths and the explicit library selector in the doc command, the
entire matrix is byte-identical. Coverage remains
198 tested /206 verified of288. No Requirements markers or evidence statuses
are added. Current docs/links/commands follow new paths; dated historical
verification and pinned source descriptions keep their at-the-time path text.
ADR-0007's current-layout section and CLI-101 sources explicitly identify the
superseding decision rather than silently erase historical source quotes.

CI feature/MSRV/WASM/Clippy/doc commands select -p irig106-time, preserving the
previous root-package scope and avoiding CLI std feature unification. CLI paths,
Dependabot, trace roots and both safety/I/O scanner roots are updated. RSS tools
already select the CLI by package and discover executables via Cargo artifacts;
those selectors remain valid, and their existing dense six-command/JSON32MiB
gates run unchanged in CI. No large local fixture or full local suite is run.

## Packaging and focused execution

Executed cargo package -p irig106-time --list --allow-dirty and the CLI equivalent
before committing: library102 entries, CLI33. Both contain README.md, LICENSE,
Cargo.toml and src/lib.rs; CLI also src/bin/irigtime.rs. Every moved source,
test/golden/fixture, example and bench file under each package is listed. Package
lists are not publication, a publish dry run or a claim to run repository-doc
sync tests from an isolated published archive. CI repeats clean --locked lists.

Focused local execution: library_boundary (3), s14_usage_snippets (2), CLI s17_json
(10), s17_inventory_golden (1); metadata guard (3 Python tests), safety scanner
(19 Python tests), compiler/Clippy policy and no-I/O probes, fmt, trace and diff
checks. Full platform/MSRV/WASM/fuzz/deny/memory verification belongs to PR CI.

## CI test-count comparison

Baseline main7d6f445 CI [44930582502](https://github.com/TelemetryWorks/irig106-time/actions/runs/44930582502).
Counts below sum Rust test-result lines from the corresponding complete job log,
including subprocess positive controls, four library feature invocations, and
platform-specific CLI checks; they are not unique-test-name totals.

| Job | Result groups | Passed | Ignored |
| --- | ---: | ---: | ---: |
| Ubuntu stable |192|3001|0|
| macOS stable |195|3043|0|
| Windows stable |194|3033|0|
| CLI Build |16|139|1|

The move's exact-head CI logs must match each baseline row (and have zero failed
tests). The PR description records the observed before/after comparison and
run IDs after CI completes; these are expectations, not fabricated after-results.
The CLI ignored test is exercised separately by its existing RSS probe.
