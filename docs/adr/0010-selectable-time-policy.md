---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# The rules of time over a recording are a selectable policy, recorded with every answer

## Context and Problem Statement

Turning a recording's counters into absolute time needs rules the standard
does not give: where a session ends, which time channel governs, which
reference within it, where the year comes from, and what counts as a gap, a
jump, or a late packet (`docs/TIME-IN-CHAPTER-10.md` sections 5.2 to 5.4).
The document proposes defaults, each justified from the standard — for
example a late-packet bound of 1100 ms, the stream commit time (1000 ms)
plus the packet generation time (100 ms) of Chapter 10 §10.6.1. Real
recordings and real users differ: a range may trust its internal clock, or
know the year.

## Decision Drivers

* Sensible, cited defaults that most callers never change
* Any rule changeable when a recording or a user needs it
* An answer that can be reproduced: the same bytes and the same policy give
  the same time

## Considered Options

* **A policy value holding every setting with its default; every answer
  records the policy used**
* Fixed rules
* Per-function parameters

## Decision Outcome

Chosen option: **a selectable policy** — the owner's decision, 2026-09-27:
"Your proposal and order is correct but I would like for it to be
selectable as well. 5.4 agree with the proposed, but again we should allow
it to be selectable". The settings and their defaults are the table of
contract section 5.6: session boundaries; which time channels count; which
time counts; choosing the time channel; the reference within the channel;
after a lost lock; the year; the reference gap; the time jump; the
late-packet bound; the counter wrap (fixed). Every answer records the policy
it used, or the settings that differ from the defaults.

### Consequences

* Good: the defaults are documented, cited, and testable one by one.
* Good: two answers can be compared knowing whether their policies differ.
* Bad: every combination of settings is behaviour to test; the tests cover
  each setting against the defaults, not every combination.
