---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# Baseline 106-24R1; every rule cites the archived standard; edition differences named

## Context and Problem Statement

The prototype cites "IRIG 106-17 Chapters 10/11 and RCC 123-20" and Chapter
10 section numbers from before 106-17 ("Ch10 §10.6.1.5"), which moved to
Chapter 11 at 106-17 ("References to RCC 106-04 through RCC 106-15 refer to
Chapter 10, while RCC 106-17 onward refer to Chapter 11", Chapter 11
§11.2.1.1 e). The ecosystem took 106-24R1 as its baseline and mirrors every
edition in `TelemetryWorks/rcc-106-standards` (`irig106-tmats` ADR-0016,
ADR-0017). Time depends on the edition in a few places: Format 2 exists from
106-17 (T-4); the packet layouts sit in Chapter 10 before 106-17 and in
Chapter 11 after.

## Decision Drivers

* Every rule checkable by anyone against a fixed copy of the standard
* Recordings from every edition since Chapter 10 appeared (106-04) must
  read
* Edition differences stated, not assumed

## Considered Options

* **Baseline 106-24R1; cite the archive; name each edition difference**
* Cite whichever edition the rule was first read in

## Decision Outcome

Chosen option: **baseline 106-24R1**. Every requirement, document, and
comment cites the 106-24R1 section, figure, or table, with its words where
they matter; an edition difference cites the editions on both sides (for
example T-4: Format 2 in 106-17's Chapter 11, absent from 106-13 and
106-15). The two version code lists stay apart: the packet header's data
type version (`0x0A` = 106-22) and the setup record's RCCVER (`0x0E` =
"106-22 or later"; `0x0F` and above reserved; T-9), both mapped in
`irig106-types` (ADR-0015).

RCC 200-16, "IRIG Serial Time Code Formats", which Chapter 11 cites for IRIG
time, is archived as the release `rcc-200-16` (2026-09-27; two byte-different
copies, from TRMC and irig106.org). Decoding IRIG time codes as signals stays
out of scope (contract section 3.10); RCC 200 is cited where Chapter 11
defers to it, such as when a Format 1 packet's counter value is captured
("IAW IRIG 200", §11.2.3.2).

### Consequences

* Good: a reviewer can check any rule on the archived page.
* Good: the requirements stop carrying pre-106-17 section numbers.
* Bad: rewriting every citation of the L1, L2, and L3 documents (L1 in
  step 1; L2 and L3 after the review).
