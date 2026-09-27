---
status: proposed
date: 2026-09-27
decision-makers: Joey
---

# The rebuilt crate is released as 0.8.0; users of 0.1.0 to 0.7.0 are told what was wrong

## Context and Problem Statement

`irig106-time` 0.0.1 to 0.7.0 are on crates.io (2026-03-21 to 2026-03-30).
Some of their answers are wrong: ERTC times 100 times too large (T-5),
intra-packet time formats selected from the wrong bit (T-1), Format 2 data
words misread (T-2, T-3), bodies that cannot be read by Data Length (T-12).
The rebuild changes the API. The changelog and roadmap already name the
next release 0.8.0 while `Cargo.toml` says 0.7.0.

## Decision Drivers

* Users of the published versions learn what was wrong
* Semantic versioning for 0.x: a breaking change is a new minor version

## Considered Options

* **0.8.0 for the rebuild, with a changelog section listing T-1 to T-12;
  the owner decides whether to yank 0.1.0 to 0.7.0**
* Yank 0.1.0 to 0.7.0 now
* Leave the published versions as they are, with no notice

## Decision Outcome

Proposed option: **0.8.0 for the rebuilt crate and `irig106-time-cli`
together** (ADR-0007), its changelog listing each finding and its effect on
0.7.0 answers, and the minimum Rust version change (ADR-0003). Whether to
yank the earlier versions, and whether to publish a README notice before
0.8.0, are left to the owner; yanking stops new projects from choosing them
and does not break existing lock files.

### Consequences

* Good: users learn which answers to distrust.
* Bad: until 0.8.0, users of 0.7.0 keep its answers.
