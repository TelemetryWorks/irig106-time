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


## Executed evidence and final clause ledger

Tests-only daacbac precedes implementation and fails at main2e562eb because
compare_qualified, mapping/audit APIs and new enum variants are absent. This
is compile-time API evidence, NOT seven runtime failures. One fixture's
inferred integer needed an explicit u8 once the API compiled. Tests-only
93c67f2 adds missing/edition/inventory guards before any implementation commit;
later formatted assertions are not retroactively claimed failing-first.
Seven final tests exercise4x16 TSRC pairs,3x256 SHTF flag bytes including
non-time channels, exhaustive existing TTF/TFMT vocabularies/mismatches,
explicit/default/missing TFMT, exact setup boundaries, source loss/regain,
EXT/unresolved/override contexts, reserved source3..14, session/channel isolation,
TF008/009, level overrides and raw/related finding evidence. Combined FND104
calls all mapping/default/missing suites, so its whole marker is not claimed
from one informational-source test. Literal Ch4 bytes[0,0,23,01,34,12,0,0]
(hex) give usec291 and low4660, byte checksum006a; dispatch comes from actual
flags80 even when SHTF says1 or is absent. No encoder/host-clock oracle, fresh
page review, real-recording comparison or fuzz campaign.

23 isolated mutations each compiled and failed runtime assertions: all4 source
pairs; all3 SHTF pairs; bit selection; bit7 presence; EXT/unresolved SRC guess;
non-TIMEIN source comparison; reserved SRC and flag findings; source level
mapped incorrectly to warning; reserved source admitted to transition history;
change suppressed; regain suppressed; session history mixed; TF008 suppressed;
TF009 suppressed; data packet counted as time; activity overrides ignored; range
identity lost. Sources restored. Initial range-identity mutation survived because
only the first range had observed packets; add a packet ONLY in the second
range and assert missing channels remain only in the first, then rerun all23:
all killed. This is a test gap corrected, not a production bug.

| L2/L3 suffix | Covered clauses | Remaining |
|---|---|---|
| RDG205 | four source pairs/mismatches, reserved3 not GPS, loss/regain information, original observed source authoritative, missing/unknown, layout qualification | Closed for caller-qualified packet/declaration facts; upstream edition parsing remains S03/caller |
| RDG206 |0/1/2 versus flags00/01/10, all256 flags, enabled-only, mixed TIMEIN/PCM channels, missing/unknown/reserved, wording retained, binary dispatch proof | Closed; Chapter4 calendar answers remain separate S07/adapters |
| FND104 | combined RDG001..006 comparisons, all mappings/mismatches/missing/context/boundary, inferredI warning, source difference/change information, TFMT opt-in default/raw absence/no insertion, standalone evidence/overrides | Closed at declaration comparison/audit boundary; automatic legacy answer/CLI propagation is S10b/S12, not this requirement |
| RDG101 | six centralized reading IDs and typed results; comparison-site inspection finds mappings only in declarations modules | No new whole marker from this slice: inspection/all consumer adoption remains explicit audit work |
| DECL102 | existing declaration-free operation and reduced comparison/activity basis | Universal legacy answer/parser-finding propagation stays S10b/S12; no whole marker |
| FND102/103 | TF008/009 default levels and scoped raw/related evidence, plus source findings | Only partial clauses; no claim of every contract6 finding or global final report coverage |

TF009 requires explicit scope completion; absence is per supplied range/channel,
not inferred from an early stream prefix. State is bounded by declared TIMEIN
inventory. Existing core engines may also report membership/source findings;
future report adapters must combine these facts deliberately, not blindly append
duplicate diagnostics. No hidden channel switch/admission or queue is added.


Final checks:18/18 bounded local CI-equivalent checks pass with locked Cargo
resolution, jobs/threads2: all feature suites, MSRV1.85, no_std WASM±serde,
CLI build/clippy, benches, all-fuzz compile, fmt/docs/trace and cargo-deny.
Actionlint/diff clean. Matrix170->176/288 (+6: FND104/RDG205/RDG206 L2+L3);
no other whole requirement promoted. Every Design/Review field is retained
byte-for-byte. Hosted checks are verified at the reported head. No merge or
publication; no new owner decision required by this slice.


Current-main update before reporting: PR28 merged as main71df0d5 while S09b
was being prepared. Ordinarily merge it into this branch, retaining both
CHANGELOG entries and regenerating TRACE-MATRIX. PR28's branch is not edited;
its year/leap fixes are inherited from main. All18 locked local checks reran and passed
on the combined source before opening the PR.
