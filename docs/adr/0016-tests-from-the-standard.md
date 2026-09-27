---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Tests from the standard: a failing test per finding; fixtures from its figures; real data local only

## Context and Problem Statement

The prototype's tests pass while the code disagrees with the standard: the
pipeline tests build PTP bodies with 48-bit seconds, as L1-PTP-001 says
(T-3), and day-month-year bodies of 10 bytes (T-12,
`tests/pipeline.rs`); the round-trip tests (`ptp_to_le_bytes_round_trip` and
others) encode with the crate's own encoder and decode with its decoder, so
an error in both cancels out. `irig106-tmats` records its guard rails in ADR-0018 (reference tools
are oracles, not authorities; real data stays local) and ADR-0020.

## Decision Drivers

* A test must be able to fail when the code disagrees with the standard
* Real recordings are not ours to publish

## Considered Options

* **Tests from the standard**: expected values derived from the standard's
  figures and tables, by hand, with the figure cited
* Round-trip tests through the crate's own encoder and decoder

## Decision Outcome

Accepted by the owner on 2026-09-27: "Accept the proposed ADRs".

Chosen option: **tests from the standard**:

* each finding T-1 to T-12 gets a test that fails on the prototype before
  the fix;
* the worked example (contract section 7) becomes a test: its 36 bytes,
  decoded and correlated, with the basis of section 7.3;
* fixtures are built from Chapter 11's figures (11-12 to 11-17) and cite
  them; round trips supplement them and never replace them;
* each L1 requirement names its tests;
* real recordings are used only locally, never committed and never in CI.

### Consequences

* Good: the tests check the standard, not the code's own opinion.
* Bad: fixtures are written by hand and reviewed.
