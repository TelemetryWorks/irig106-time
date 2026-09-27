---
status: proposed
date: 2026-09-27
decision-makers: Joey
---

# `no_std` with `alloc`; `std` a default feature; WebAssembly built in CI

## Context and Problem Statement

The prototype is `#![no_std]` with `std` a default feature; `correlation`
and `network_time` need `alloc`. The ecosystem requires a WebAssembly build
(`irig106-tmats` L1-REL-003), and `irig106-tmats` stays `std` (its F6). A
time library is also useful in recorders and ground equipment without an
operating system.

## Decision Drivers

* Builds for `wasm32` (for `irig106-studio`, contract section 4.8)
* Usable on embedded targets
* The time timeline, the policy, and the findings need growable collections

## Considered Options

* **`no_std` with `alloc`; `std` default; `wasm32` built in CI**
* `std` only, like `irig106-tmats`
* `no_std` without `alloc`

## Decision Outcome

Proposed option: **`no_std` with `alloc`**. Decoding single values (time
packets, secondary headers, time stamps, time words) needs neither `std` nor
`alloc`; the time timeline and the correlators need `alloc`; `std` adds only
`std::error::Error` and conveniences. CI builds `wasm32-unknown-unknown`
with and without `serde`, and a `no_std` target without `std`.

### Consequences

* Good: one crate for browsers, recorders, and servers.
* Bad: no `std` collections or I/O in the library (which ADR-0006 forbids
  anyway).
