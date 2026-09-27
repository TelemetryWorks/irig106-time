---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Documentation first: contract, decisions, architecture, requirements, tests, code

## Context and Problem Statement

The prototype's documents trail its code: `docs/architecture.md` describes
0.1.0, the requirements cite Chapter 10 section numbers from before 106-17,
and several disagreements with the standard (T-1 to T-12) sit in both the
requirements and the code, so the tests confirm them. `irig106-tmats` was
rebuilt documentation first (its ADR-0001), and the owner asked for the same
here: "document first, start section 1" (2026-09-26).

## Decision Drivers

* Every rule traceable to the archived standard before it is coded
* Decisions made by the owner, in writing, before code depends on them
* Tests derived from the standard, not from the code under test

## Considered Options

* **Documentation first**, in a fixed order
* Code first, with the documents brought up to date afterwards

## Decision Outcome

Chosen option: **documentation first**, in this order
(`docs/TIME-IN-CHAPTER-10.md` section 8.2; `docs/diagrams/time-plan.svg`):

1. **Decide and document**: the contract document (done, sections 1 to 8);
   these ADRs; `docs/architecture.md` revised against the contract; the L1
   requirements rewritten with 106-24R1 citations; L2 and L3 after the
   owner's review.
2. **Owner review** of step 1.
3. **Tests first**: a failing test for each finding, the worked example of
   section 7 byte for byte, fixtures from the standard's figures.
4. **The shared vocabulary** fixed in `irig106-types`, then published
   (ADR-0015).
5. **The modules**, as section 8.3 assesses them.
6. **The new capabilities**: the time timeline, the policy, the basis, the
   reading register, time words from data.
7. **Restructure**: the lockstep workspace and `irig106-time-cli`
   (ADR-0007).

Nothing in `src/` changes before the tests-first step, except what keeps the repository
building (such as the edition change of ADR-0003).

### Consequences

* Good: the owner reviews the design before any of it is coded.
* Good: requirements and tests can no longer confirm each other's errors,
  because both come from the standard.
* Bad: slower to a corrected release.
