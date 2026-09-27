---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# The library performs no I/O

## Context and Problem Statement

The library works on bytes and values its callers supply; opening files and
walking packets belong to the packet reader (`irig106-core`) and, until it
exists, to this repository's CLI (contract section 1.3). The ecosystem
decided the same for `irig106-tmats` (its ADR-0010).

## Decision Drivers

* Usable in a browser, an embedded recorder, and a server alike
* Testable with byte fixtures alone

## Considered Options

* **No I/O in the library**; files, memory maps, and packet walking in the
  CLI and the packet reader
* Convenience functions that open a file and correlate it

## Decision Outcome

Chosen option: **no I/O in the library**. `irig106-time` has no file,
network, clock, or environment access; the leap-second table is data built
in or supplied by the caller, never fetched. `irig106-time-cli` reads files
(ADR-0007).

### Consequences

* Good: the library stays `no_std` capable (ADR-0014) and deterministic —
  the same input gives the same answer on any machine.
* Bad: a caller wanting "time for this file" writes the joining loop or
  uses `irig106-time-cli`'s library.
