---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Every absolute time carries its basis

## Context and Problem Statement

`TimeCorrelator::correlate` returns a bare `AbsoluteTime`. A time 100 ms
from a GPS-locked reference and one an hour past the last reference of a
freewheeling clock come back alike (`docs/TIME-IN-CHAPTER-10.md` sections
3.1, 3.5, 7.6: "The basis is the difference between a time and an
answer").

## Decision Drivers

* Weak time never mistaken for good time
* Consumers (the decoder, the index, the tools) can pass the basis on
  (contract section 4.1, rule 6)

## Considered Options

* **Every answer is the time with its basis**
* A bare time, with a separate call for its quality
* A bare time and a global quality score

## Decision Outcome

Accepted by the owner on 2026-09-27: "Accept the proposed ADRs".

Chosen option: **every answer carries its basis**: the session; the time
channel and why it was chosen; the reference used (counter and time); the
distance to it; its position (between two references, before the first, or
beyond the last); the reference's source, format, and IRIG time source;
where the year came from; whether drift correction was applied; and the
policy (ADR-0010). The worked example (contract section 7.3) shows one.

### Consequences

* Good: every consumer can decide what time it trusts.
* Bad: a larger answer than a time; callers that want only the time take
  it from the answer.
