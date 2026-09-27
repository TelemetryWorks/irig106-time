---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Shared time values live in `irig106-types` and are fixed there first

## Context and Problem Statement

`irig106-types` holds the counter (`Rtc`), `Ertc`, `Ch4BinaryTime`,
`Ieee1588Time`, the time sources and formats, and the unit and epoch
newtypes (ROADMAP P6-01; `irig106-tmats` ADR-0009). Four findings are in
those values: ERTC counted at 100 ns (T-5), time source 3 as GPS (T-6), no
FMT `0xF` NONE or ITS (T-7), and one version mapping where the standard has
two lists (T-9). `irig106-types` 0.1.0 is not published, so this crate
depends on it by path and CI cannot build it.

## Decision Drivers

* One definition of each shared value, correct, for every crate
* A crate that CI can build from crates.io

## Considered Options

* **Fix the values in `irig106-types`, publish it, then rebuild on it**
* Keep local copies in `irig106-time` until later

## Decision Outcome

Chosen option: **fix in `irig106-types` first** — the shared-vocabulary step of the plan the
owner accepted with "start step 1" (contract section 8.2): ERTC at 1 ns a
tick; time source 3 reserved; FMT `0xF` NONE; the ITS values; the setup
record's version codes and the packet header's data type versions as two
mappings. Then `irig106-types` is published and the path dependency
dropped.

### Consequences

* Good: `irig106-tmats`, `irig106-core`, and `irig106-decode` share
  corrected values.
* Bad: this crate's rebuild waits on an `irig106-types` release; publishing
  is the owner's call (`irig106-types` commits stay local until then).
