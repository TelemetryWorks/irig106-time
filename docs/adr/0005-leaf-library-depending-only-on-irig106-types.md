---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# A leaf library: depends only on `irig106-types`; `irig106-decode` depends on it

## Context and Problem Statement

Time needs facts from other crates: the packet reader gives headers and
bodies, `irig106-tmats` says which channels carry time and in what format,
and `irig106-decode` finds intra-packet time stamps and time words in data.
Either this crate depends on those crates, or they hand it plain data
(`docs/TIME-IN-CHAPTER-10.md` sections 2.3, 2.4). `irig106-tmats` ADR-0030
chose plain data for `irig106-core` and `irig106-tmats`.

## Decision Drivers

* A small library any tool, and a browser build, can use alone
* One direction of dependency, no cycles
* The decoder turns what it finds into time without each tool joining the
  two

## Considered Options

* **Leaf library**: `irig106-time` depends only on `irig106-types`; callers
  hand it plain data; `irig106-decode` depends on `irig106-time`
* `irig106-time` depends on `irig106-tmats` and `irig106-core` and reads
  what it needs itself
* `irig106-decode` does not depend on `irig106-time`; each tool joins them

## Decision Outcome

Chosen option: **leaf library**. The dependency on `irig106-decode`'s side
was the owner's decision on 2026-09-26 ("In favour of depending"); the
leaf-library side follows `irig106-tmats` ADR-0030.

| Crate | Depends on | Does not depend on |
|-------|-----------|--------------------|
| `irig106-time` | `irig106-types` | `irig106-tmats`, `irig106-core`, `irig106-decode` |
| `irig106-tmats` | `irig106-types` | `irig106-time` |
| `irig106-decode` | `irig106-types`, `irig106-tmats`, `irig106-time` | — |

What crosses each boundary is the table of contract section 2.3 (A to H):
the time attributes of the setup record as plain values; each time packet's
channel, data type, counter, data word, and body; a packet's counter, flags,
secondary header, and time stamps.

### Consequences

* Good: `irig106-time` builds and tests alone, and for WebAssembly.
* Good: the time attributes have one reader (`irig106-tmats`) and the time
  rules one owner (this crate).
* Bad: each caller builds the plain values from the setup record; the
  joining loop (contract section 4.2) is written once per tool until a
  shared joining crate exists (`irig106-tmats` ADR-0030's option C).
