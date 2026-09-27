---
status: proposed
date: 2026-09-27
decision-makers: Joey
---

# Readings between TMATS and the packets are a reviewed register

## Context and Problem Statement

TMATS and the time packets name the same things differently: TMATS
`R-x\TFMT-n` letters against the data word's FMT and NTF numbers,
`R-x\TSRC-n` letters against SRC, `R-x\SHTF-n` 0 "Chapter 4 BCD" against the
packet flags' "Chapter 4 binary weighted" (`docs/TIME-IN-CHAPTER-10.md`
section 3.7). Most pairs read directly; some do not — `R-x\TFMT-n` I
"Internal" has no clear FMT value. `irig106-tmats` keeps such readings in an
interpretations register (its ADR-0027, `docs/INTERPRETATIONS.md`), each
reviewed and marked for testing against real data.

## Decision Drivers

* A reading of the standard is a decision; it must be written down and
  reviewed, not buried in a `match`
* The same form as `irig106-tmats`, so one reviewer reads both

## Considered Options

* **A register of readings, each with its sources, its status, and the code
  that implements it**
* Readings in code comments only

## Decision Outcome

Proposed option: **a register**, `docs/INTERPRETATIONS.md` in this
repository, in `irig106-tmats`'s form: each entry quotes both sources, gives
the reading, its status (accepted, proposed, suspect, open), and "intensive
testing and deep analysis required" during development. The first entries
are the rows of contract section 3.7, `R-x\TFMT-n` I "Internal" open.

### Consequences

* Good: disagreements between TMATS and the packets are reported by a rule
  someone reviewed.
* Bad: another register to keep.
