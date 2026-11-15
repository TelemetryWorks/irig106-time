# Adoption of published irig106-types 0.1.0

Started from main58b1cf0, independently of frozen S09a PR29 and defect PR28.
The root manifest changes from git7bfe47f to version="0.1", retaining
`default-features=false`. CLI/fuzz already depend on core by path and had no
direct types declaration; do not add unused dependencies. Regenerate all three
lockfiles with cargo update -p irig106-types --precise0.1.0. Other locked packages
are byte-for-byte unchanged. Remove only the types allow-git entry; unknown-git
remains deny. No source, CLI output, version bump or publication/workflow change.

## Artifact and API verification

- Actual downloaded archive SHA256:
  `2f5ddd5baf4fbc4fa5579bd739561274bc5fc9cc36ba50d610b0442cc0635420`,
  equal to the owner-supplied checksum and all three lock entries.
- Packaged .cargo_vcs_info.json commit:
  `3c517358694646652939c236968b202ebf76e28a`, matching the supplied tag commit.
  No tag or publishing action is performed here.
- Cargo metadata: version0.1.0, registry source, Rust1.85/edition2024. Features,
  dependency requirements and licence equal the previous git dependency.
- All11 Rust source files compare identical after removing rustdoc lines.
  The only changed files are lib.rs and units.rs: ecosystem description, serde
  documentation and whole-second TAI/UTC limitations before1972 are clarified.
  All non-doc declarations, macro definitions and method bodies are identical.
  Thus time's signed quantities, checked Option conversions, enums, version
  mappings and wire layouts retain their existing API/implementation.

This is dependency-source adoption, not new behavior: no new unit tests or
failing-first claim. Existing semantic tests plus all consumers verify integration.
All18 local CI-equivalent checks pass with jobs/threads2: feature suites,
MSRV1.85, no_std WASM±serde, clippy/fmt/docs, CLI, benches, locked all-fuzz compile,
trace and cargo-deny. Cargo build/check/test/clippy/doc paths use --locked,
including CLI and fuzz. Actionlint/diff checks pass; hosted checks are verified
at the reported head. No fuzz campaign, merge or publish. Trace markers unchanged.
