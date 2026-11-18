---
status: accepted
date: 2026-11-17
decision-makers: Joey
---

# 0020 — Jiff replaces chrono in 0.8.0

The owner replaces optional chrono interoperability with optional, off-by-default
Jiff interoperability in0.8.0. Remove chrono; do not ignore
[RUSTSEC-2026-0543](https://rustsec.org/advisories/RUSTSEC-2026-0543.html), which
reports all chrono versions unmaintained with no patched release.

Use `jiff::civil::DateTime` for naive conversions, with explicit scale evidence;
no Timestamp API is required. Require known year/scale, preserve fields and
provenance, and reject unrepresentable leap labels or years with typed errors.
Keep MSRV1.85, no_std/WASM support and default builds free of Jiff.

Jiff1.0.1 declares Rust1.85. Disable its default features so these civil adapters
need no clock, timezone database or platform I/O. Civil values do not perform
UTC/TAI/GPS conversion: callers use explicit existing conversion/table APIs
first when they need UTC; the scale wrapper prevents relabelling. This choice
implements the owner's request to match the existing naive conversion semantics.
JSON ownership remains the independent ADR-0019 decision; no merge/publication
is authorized here.
