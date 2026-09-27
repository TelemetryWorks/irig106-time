---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Edition 2024 and Rust 1.85 across the ecosystem

## Context and Problem Statement

The crates disagreed. `irig106-time` and `irig106-types` used edition 2021
and declared Rust 1.60 while their CI checked 1.78 (the oldest toolchain
that reads a version 4 `Cargo.lock` and the sparse registry);
`irig106-tmats` used edition 2024 and Rust 1.85; `irig106-core`,
`irig106-decode`, `irig106-write`, `irig106-index`, `irig106-cli`, and
`irig106-ch10-reader` used edition 2024 without declaring a Rust version.
Crates that depend on each other must build on the same toolchain, and
`irig106-cli` will mount the time and TMATS CLIs as libraries.

## Decision Drivers

* One Rust version for users of any crate of the ecosystem
* The CI's MSRV check must test what the manifest declares
* Edition 2024 needs Rust 1.85, the release that stabilized it

## Considered Options

* **Edition 2024 and Rust 1.85 everywhere**
* Keep each crate's own edition and Rust version
* Edition 2021 and an older Rust version everywhere

## Decision Outcome

Chosen option: **edition 2024 and Rust 1.85 everywhere** — the owner's
decision, 2026-09-27 ("align on 2024/1.85"). Rust 1.85 is the lowest version
that supports edition 2024, so it is the lowest the ecosystem can declare.

Applied here on 2026-09-27: `irig106-time`, `irig106-time-cli`, and
`irig106-types` moved to edition 2024 and `rust-version = "1.85"`; their CI
MSRV jobs check 1.85; rustfmt applied the 2024 style edition; the CLI's one
use of `is_multiple_of` (stable from 1.87) was replaced. All tests pass on
1.85 and on stable. The crates already on edition 2024 declare
`rust-version = "1.85"` at their next change.

### Consequences

* Good: one toolchain floor for the whole ecosystem; `irig106-cli` can
  depend on every CLI library without raising anyone's floor.
* Good: the MSRV job now checks the declared version, not a newer one.
* Bad: users of `irig106-time` on Rust 1.60 to 1.84 cannot take the next
  release; for 0.x crates this is allowed in a minor release and is named
  in the changelog.
* APIs newer than 1.85 stay out of library code; clippy's
  `incompatible_msrv` lint enforces it from the manifest.
