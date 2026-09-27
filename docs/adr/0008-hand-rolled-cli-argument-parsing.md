---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Hand-rolled argument parsing for `irig106-time-cli`

## Context and Problem Statement

`irig106-time-cli` has six commands and a few options. `irig106-tmats` chose
hand-rolled parsing for its CLI (its ADR-0013), and ROADMAP P6-10 carries
the same for this one; `irig106-cli` mounts both.

## Decision Drivers

* Few dependencies, a small binary, and a short build
* The same parsing style in every CLI that `irig106-cli` mounts

## Considered Options

* **Hand-rolled parsing** in an `args` module
* An argument-parsing crate

## Decision Outcome

Chosen option: **hand-rolled parsing**, in the `args` module of
`irig106-time-cli`'s library, so that `irig106-cli` can call it with the
arguments after `time`.

### Consequences

* Good: no parser dependency; errors and help text under our control.
* Bad: help, errors, and option syntax are written and tested by hand.
