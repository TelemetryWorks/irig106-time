---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Recording events move to `irig106-decode`; this crate keeps their time tags

## Context and Problem Statement

`src/recording_event.rs` gives event numbers fixed meanings (1 started, 2
stopped, 3 overrun, 4 to 15 index points). The standard gives them none:
"Event Number. Bits 11-0 represent an unsigned binary that identifies 4096
individual events types defined in the corresponding setup record recording
event number. The event number shall begin at 0x0 for the first event type
defined in the setup record" (Chapter 11 §11.2.7.3; T-8). The meaning is in
the setup record's `R-x\EV\…` attributes; the entry's time tag is an
intra-packet time stamp.

## Decision Drivers

* An event's meaning comes from the setup record, which `irig106-tmats`
  reads
* Data-type bodies belong to `irig106-decode` (`irig106-docs` coverage map)
* Time belongs here

## Considered Options

* **Decode recording events in `irig106-decode`, meaning from
  `irig106-tmats`; this crate turns each time tag into absolute time**
* Keep them here, with meaning handed in as plain data
* Move them to `irig106-tmats`

## Decision Outcome

Chosen option: **move to `irig106-decode`** — the owner's decision,
2026-09-26 ("Agree on your recoding event proposal"; contract section 4.13).
`recording_event.rs` and its fixed event types leave this crate when the modules are rebuilt;
the time tag is converted like any other intra-packet time stamp.

### Consequences

* Good: no fixed meanings the standard does not give.
* Bad: until `irig106-decode` decodes them, no crate of the ecosystem
  decodes recording events; the 0.8.0 changelog says so.
