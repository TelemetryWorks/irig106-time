---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Rebuild the crate rather than patch the prototype

## Context and Problem Statement

`irig106-time` was written before the ecosystem's documentation-first
redesign, against IRIG 106-17 section numbers, and published to crates.io
as 0.0.1 to 0.7.0 (March 2026). Checking it against the archived standard
(106-24R1 baseline) found twelve disagreements (`docs/STANDARD-REVIEW.md`,
T-1 to T-12). Several are not local bugs but consequences of the design:

- **Packet flags are read wrongly** (T-1): the intra-packet time format is
  taken from bit 2, so the Chapter 4 and ERTC formats are never selected.
- **Format 2 is read wrongly** (T-2, T-3, T-4): the data word from the
  wrong bits, the PTP body as 10 bytes where the standard gives 8, and the
  format dated to 106-22 where it exists from 106-17.
- **ERTC is 100 times too large** (T-5) and **time source 3 is "GPS"**
  where every archived edition reserves it (T-6).
- **Time packet bodies are read past their end** (T-12): the day-of-year
  body is 6 bytes and the day-month-year body 8, but the crate asks for 8
  and 10, so a caller that slices by Data Length cannot read them.
- **Answers carry no basis.** A time 100 ms from a reference and one an
  hour past the last come back alike (`docs/TIME-IN-CHAPTER-10.md` section
  3.1); there are no sessions, no time timeline, no time policy (section
  5), and no findings for degraded time (section 6).
- **Recording events have fixed meanings** that the standard gives to the
  setup record (T-8).

The contract document (`docs/TIME-IN-CHAPTER-10.md`, sections 1 to 8)
describes a crate whose central types — an answer with its basis, a time
timeline of sessions, a selectable policy — do not exist in the prototype.

## Decision Drivers

* Fidelity to IRIG 106 (106-24R1 baseline), every rule citable
* Time that says what it rests on, so weak time is never taken for good time
* The owner's direction for the ecosystem: "slow, methodical and
  incremental", documentation first
* The prototype is published (0.1.0 to 0.7.0), so the rebuild is a new
  minor release with breaking changes (ADR-0017)

## Considered Options

* **Rebuild** against the contract document, documentation first, keeping
  what the assessment of section 8.3 marks "keep"
* **Refactor in place**: fix T-1 to T-12 in the current modules, then add
  the basis, sessions, and policy around them

## Decision Outcome

Chosen option: **rebuild** — the owner's decision, 2026-09-27: "Rebuild it,
align on 2024/1.85, add RCC 200, start step 1". This settles the question
section 8.2 had left for the review of step 2 ("refactor in place or
rebuild").

The prototype is kept at the git tag **`prototype-0`** (commit `a755a5c`,
the last code change; the crate says 0.7.0) and at the published 0.7.0.
Rebuilding does not mean discarding: the assessment of the contract
document's section 8.3 names what carries over — the RTC arithmetic,
`AbsoluteTime` and `CalendarTime`, the BCD digit and range checks, the
nearest-reference core of the correlator, the quality measures, the
leap-second table, the Chapter 10 to Chapter 11 move at 106-17 — and each
is carried over through a test written from the standard (ADR-0016), not
copied on trust.

### Consequences

* Good: the answer with its basis, the time timeline, and the policy are
  designed in, not bolted on.
* Good: every finding T-1 to T-12 becomes a test before the code that
  fixes it.
* Good: the prototype stays available for comparison at `prototype-0`.
* Bad: no corrected release until the rebuild reaches 0.8.0; users of
  0.7.0 keep its wrong answers meanwhile (ADR-0017 proposes what to tell
  them).
* The prototype's source stays on `main`, marked "prototype" in
  `docs/project_structure.md`, until the first rebuilt code lands.
