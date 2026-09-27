---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Degraded time is reported as findings with stable identifiers; never repaired silently

## Context and Problem Statement

Real recordings break the rules: no time packets, packets before the first
time packet, invalid or malformed time packets, undeclared time channels,
weak sources, failed secondary-header checksums, gaps, jumps, resets, no
year, a stale leap-second table (`docs/TIME-IN-CHAPTER-10.md` section 6).
The prototype returns errors for some and nothing for others; the jump
detector needs a threshold from the caller.

## Decision Drivers

* Say exactly what is wrong, give what time can be given, and label it
* The same finding identified the same way in every tool and every release

## Considered Options

* **Findings with stable identifiers and a default severity the caller can
  change**, alongside the answers
* Errors only: refuse any time that is not perfect
* Silent best effort

## Decision Outcome

Accepted by the owner on 2026-09-27: "Accept the proposed ADRs".

Chosen option: **findings**. Every case of section 6 is a finding with a
stable identifier, the evidence (packet, channel, counter), and a default
severity the caller can change; what the crate does in each case is a
setting of the policy (ADR-0010). Errors remain for input the crate cannot
read at all (a buffer too short, a BCD digit above 9). The time timeline
(contract section 5.5) holds the findings of a recording.

### Consequences

* Good: tools report the same problem the same way.
* Bad: identifiers are a public contract; one retired is never reused.
