---
status: accepted
date: 2026-11-13
decision-makers: Joey
---

# 0018 — 0.8.0 release scope and policy decisions

## Context

ADR-0017 sets the rebuild release to 0.8.0. The owner accepted the remaining-work
plan recommendations on 2026-11-13: "accept your recommendations". The
coordinator's assignment preserves the following qualifications and boundaries.

## Decision

- Add TF-029 zero NTP and TF-030 NTP leap ambiguity as WARNING findings, with
  raw/no-absolute and mapped-value-kept effects respectively. Add permanent
  TF-031 packet-layout conflict and TF-032 year conflict IDs/titles; their
  levels remain pending owner confirmation in S03 and S08. TF-001..028 stay unchanged.
- Inverse queries use half-open absolute ranges, all matching half-open RTC
  tick intervals, explicit quantization/basis, and a separate point query;
  never bridge gaps or jumps.
- Defer past 0.8.0 only inverse queries, the new complete secondary/stamp
  encoder API, complex timeline-state deserialization, and prebuilt binary
  distribution. Retain the requirements without marking them covered. Existing
  encoders/byte-sum behaviour, supported validated loads, complete versioned
  outputs and every other requirement remain blockers. Offline drift-corrected
  query mode is after 0.8.0.
- Release both crates to crates.io only, via Trusted Publishing/OIDC from
  GitHub Actions; no stored publishing token, no local publish and no prebuilt
  binaries in 0.8.0.
- Snapshot/report JSON has a versioned schema and units; 64/128-bit quantities
  are decimal strings. Validate supported loads, retain raw/stated/assumed
  provenance, and review the schema once before freeze. Omit/disable unsupported
  complex timeline Deserialize for 0.8.0.
- CLI classes remain 0–4: usage before input; first fatal operational failure
  determines 3 (input) or 4 (output); later failures become diagnostics.
- Accept interpolation floor, preceding midpoint tie, the coarser endpoint
  resolution as mixed-resolution jump threshold, and old-policy drain on mode
  change as the defaults (C1–C4).

## Scope

This refines ADR-0017 and ADR-0010 without changing their historical records.
Sources and requirement lineage are in the
[decision/source record](../analysis/plan-decisions-2026-11-13.md). These are
release and reader policies; no behaviour, workflow, credentials or publication
is implemented or authorized by this documentation change. Human Review fields
remain unchanged.
