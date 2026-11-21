---
status: accepted
date: 2026-11-21
decision-makers: Joey
---

# 0022 — Virtual workspace with crates and apps

Use a virtual workspace modelled on Praevis-Quant/praevis-thetadata, superseding
the earlier root-package layout in ADR-0007. The library is in
`crates/irig106-time/`; the CLI is in `apps/irig106-time-cli/` for testing and
simple examples. Package names and the `irigtime` binary name are unchanged.

The root Cargo.toml has no package: it holds workspace membership, shared package
metadata with one lockstep version, and workspace dependencies with an exact
internal library path/version pin. Keep fuzz excluded or independent; fuzz and
benchmark placement follow the reference model. This is a layout change only.
