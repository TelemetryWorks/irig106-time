# S09b — TSRC/SHTF and declaration findings

Base main2e562eb905f8e834fdfe2b71210fb94493f828cd (S09a merged,170/288).
PR28 is under review and excluded. Scope: plan S09b, RDG005/006, FND104,
TF008/009/010/011; no timeline/stream/CLI behavior change or TMATS dependency.

## Reading analysis before tests/implementation

Existing page/register evidence, not a fresh PDF rendering:

- 106-24R1 Ch9 Table9-4 printed9-72/PDF78: TSRC "I Internal", "E External",
  "R Internal from RMM", "X None". Ch11 §11.2.3.2a printed11-24/PDF32:
  "0x0 = Internal", "0x1 = External", "0x2 = Internal from RMM", "0xF = None".
  RDG005 maps I/E/R/X to0/1/2/15. "If the time source is external (0x1) and
  lock on the external source is lost then the time source shall indicate
  Internal (0x0)." Source mismatch/change is TF011 INFORMATION by owner policy,
  not an invented error or a channel-selection instruction. SRC3..14 remains
  reserved (TF006 WARNING), never GPS. No default TSRC is inserted.
- Same Table9-4: "If enabled, the secondary header time format.";
  "0 Chapter 4 BCD", "1 IEEE-1588", "2 ERTC". Ch11 §11.2.1.1g printed11-8/
  PDF16: "00 = Chapter 4 binary weighted 48-bit time format",
  "01 = IEEE 1588 time format", "10 = 64-bit binary extended relative time
  counter (ERTC)". Ch4 §4.7 printed4-10/PDF14: "High and low order time words
  shall be binary or binary coded decimal (BCD) weighted". RDG006 pairs0/1/2
  with00/01/10; packet dispatch follows Chapter11 binary weighting, while
  retaining the Chapter9 discrepancy. No guessed BCD secondary decoder.
- S03/owner Q-EDITION: unresolved/106-04 EXT layouts have no SRC comparison.
  Explicit override remains first; adopted G\\106 cannot override it. Retain
  QualifiedCsdw/resolution/raw bits, never infer SRC from nibble3 or header0x01.
- D8: only RDG001..006 comparisons. TF010 WARNING covers TTF/TFMT/SHTF;
  TF011 INFORMATION covers TSRC differences/changes. TF008 WARNING reports time
  packet types11/12 on non-TIMEIN channels; TF009 INFORMATION reports a TIMEIN
  declaration range with no such packets. These IDs/levels are product policy
  (architecture8/contract6.4), not standard-defined severities.

## API and edge decisions within accepted scope

Keep S09a compare(PacketFields) context-free; add compare_qualified with an
optional S03 QualifiedCsdw and primary flags, returning the basic results plus
TSRC/SHTF, original qualification/flags and all comparison findings. Supplied
CSDW/raw packet fields must agree. Absent qualified SRC is unavailable, not a
match. Source letters apply only to observed Format1 on TIMEIN. Missing/unknown
TSRC remains explicitly missing/unknown; it is not automatically called lost
lock, and no requiredness/default is invented. Defined differences emitTF011.
Reserved SRC also emitsTF006 without reinterpreting it. SHTF is per channel,
including non-time channels, and applies only with secondary-present bit7;
bits3-2 select the observed format. Missing/unknown SHTF is a TF010 diagnostic,
not a default. Disabled/unknown secondary presence never invents a comparison.

A caller-fed DeclarationAudit tracks only supplied declared TIMEIN inventory:
seen(range,channel) and previous qualified source(session,channel). State is
bounded by that inventory, not by packet count; no retained packet queue.
Observed time packet types count even if their body is invalid. Unknown/EXT/
reserved SRC cannot become a source-change endpoint. Missing/invalid source
observations do not erase the last qualified endpoint. Source change retains
previous numeric source/packet and current governing range; it may coexist
with a static-declaration mismatch. finish explicitly closes the caller's
supplied scope before TF009 can be asserted. Findings do not alter admission,
channel ranking, payload, raw flags, counters or any already emitted answer.

## Clause ledger (updated after execution)

FND104 requires combined RDG001..006/default/inference/source/range tests before
any whole marker. RDG205/206 require exhaustive pairs, missing/reserved/wrong
contexts, loss/regain, mixed channels and BCD-looking binary bytes. RDG101
inspection and DECL102 universal legacy propagation remain clause records;
this slice does not claim all-edition field evidence or full report adapters.
