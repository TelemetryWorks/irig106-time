---
status: accepted
date: 2026-11-17
decision-makers: Joey
---

# 0019 — JSON belongs to the CLI

## Decision

The library provides Rust data only. JSON report output belongs to the
`irig106-time-cli` crate and its `irigtime` (planned rename from ch10time) binary under L1-CLI-006. There is
no untrusted JSON loading in the library. Keep non-JSON AnswerView/replay.
The library's optional serde feature, available in 0.1–0.7, is removed in
0.8.0 (breaking); consumers use the CLI's JSON output instead.
`irig106-types`' own serde feature is untouched.

## Rationale and scope

The data library and CLI report format have separate responsibilities.
Owner Q5's versioned schema, units, exact decimal-string 64/128-bit quantities,
provenance and one review before freeze apply to the CLI JSON report format.
Q3's complex library timeline-state deserialization deferral is moot because
JSON loading leaves the library. This supersedes ADR-0018's library JSON/load
scope; its other release and policy decisions remain. IRIG 106 does not
prescribe JSON. This decision does not authorize a merge or publication.
