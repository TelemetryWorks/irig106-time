# S10a preparation — secondary own time and RTC/ERTC consistency

Preparation starts from main `e068372e5fd8dab70c897867812542323a2fe325`.
Owner GO 2026-11-15 is conditional on PR31 merging; the coordinator will relay
that merge. This preparation is analysis/fixtures only: no production changes,
new API, requirements/Design/Review changes, implementation claims or PR.
After the relay, ordinarily merge current main before tests/implementation.

## Sources and constraints to preserve

Existing requirement/contract/register quotations, not fresh page rendering:

- 106-24R1 §11.2.1.2 / Figures11-4/5/6: "The length of the packet secondary
  header is fixed at 12 bytes (96 bits)". Time occupies[0..8), header-reserved
  bytes[8..10), stored checksum[10..12). Figure11-4 is printed11-10/PDF18;
  Figures11-5/6 printed11-11/PDF19. Retain both Chapter4's internal reserved
  word[0..2) and the header-reserved word independently.
- §11.2.1.1g, printed11-9/PDF17: "The 32 bits indicating seconds shall be
  placed into the MSLW portion of the secondary header and the 32 bits
  indicating nanoseconds shall be placed into the LSLW portion". Secondary
  IEEE is LE nanoseconds then seconds, unlike Format2 PTP's seconds then ns.
  Epoch/scale/count convention must be supplied and retained, not inherited
  from an unrelated Format2 body just because both look like IEEE1588 time.
- Same section: "the 10-megahertz (MHz) RTC shall be synchronized with the
  ERTC (RTC = ERTC/100)". ERTC is one ns/tick, relative, full64 bits. Compare
  floor(ERTC/100) modulo2^48 with RTC; strict abs(delta)>1 tick emits TF025.
  S01 session/ordering guards and checked arithmetic apply; no Unix-epoch guess.
- §11.2.1.2c: "a 16-bit arithmetic sum of all secondary header bytes excluding
  the secondary header checksum word". RCC123-20 AppendixA-2
  uCalcSecHeaderChecksum (printed B-31) sums bytes. RDG008 remains suspect:
  byte validity gates trust, word-only matches emit TF027 AND TF012. Retain
  stored/byte/word results; never silently switch readings or claim settlement.
- §11.2.1.2a: "all channels that have a secondary header must use the same
  time source in bits 2-3 of the packet flags". Track formats per channel,
  retain each packet's actual flags/dispatch, report mixed formats as TF013.
  Missing headers do not imply format00; reserved11 is not a fourth format.
- Same paragraph: "The applicable data bit to which the 48-bit value of the
  packet secondary time value (if enabled) applies shall correspond to the
  first bit of the data in the packet body". Keep caller-supplied cited data-
  type exceptions as metadata, never invent one or alter the packet counter.
- Contract6.6/L3-ANS-102: retain own and counter answers separately. Only a
  compatible normalized epoch/scale allows signed own-minus-counter comparison.
  Default tolerance is reference resolution (Format1:10ms), overridable;
  TF014 is strictly above tolerance, not at it. Unknown context is not delta0.

Existing decisions apply: D2/D3 answer/basis semantics; D6 Chapter4 explicit
origin/partial output, never nearest-packet guessing; D7 stale leap-table
last-offset fallback/TF021 and raw TAI retained; D11 provenance/validity separate
from severity; D12 cited policy defaults. Reuse S04/S06a/S07/S08 and shared
checked quantities; no saturation or new dependency.

## Hand-derived fixture ledger for tests-first work

All byte sequences below are hexadecimal; stored checksum is little endian.
They are derived from field weights/arithmetic, not produced by an encoder.

| Fixture | Exact operands / expected result |
|---|---|
| IEEE distinguishing words | seconds01020304, ns075bcd15=123456789: secondary `15 cd 5b 07 04 03 02 01 00 00 4e 01`; byte sum014e, word sumd876 |
| Format2 control | same values have body `04 03 02 01 15 cd 5b 07`; using that order for secondary must fail value assertions |
| Byte-only trust | IEEE fixture stored014e: trusted raw own time, independent counter retained |
| Word-only | same first10 bytes, stored d876 (`76 d8`): TF012+TF027, no trusted own time; counter answer remains |
| Neither | same first10 bytes, stored0000, or independently corrupt each stored-checksum byte: TF012, no silent word fallback |
| Both | `01 00 02 00 03 00 04 00 00 00 0a 00`: byte sum=word sum10; raw IEEE ns131073, seconds262147 |
| Bounds | every length0..11; valid12 plus trailing body bytes; trailing bytes excluded from both sums |
| Reserved fields | singly set each bit in[8..10); also each bit in[0..2) for Ch4, with hand-adjusted checksum: independent TF005 positions/raw bits, value kept |
| Value range | IEEE ns999999999 valid /1000000000 invalid; Ch4 usec9999 valid /10000 invalid; raw errors do not suppress independent counter time |
| ERTC resolution | raw0/1/99/100/199 -> expected RTC0/0/0/1/1, remainders0/1/99/0/99ns; full u64 retained |
| RTC/ERTC tick tolerance | ERTC199 -> expectedRTC1; supplied RTC(MASK,0,1,2,3) -> signed ticks(-2,-1,0,+1,+2); TF025 only at -2/+2 |
| RTC wrap | ERTC=(2^48-1)*100+99 with RTC0 -> +1 tick; ERTC=2^48*100+99 with RTCMASK -> -1 tick; no TF025 |
| Session context | relative ERTC without a matching session reference stays relative; no fabricated absolute year/epoch, cross-session rejection retained |
| Own discrepancy | compatible normalized own-minus-counter ±(9999999,10000000,10000001)ns at10ms tolerance: TF014 only at ±10000001; both answers retained |
| Unknown/incompatible comparison | incomplete Ch4 origin, unknown IEEE epoch/scale, or missing conversion context: labelled own partial/unresolved delta, counter answer independent |
| Mixed formats | same-format channels, every pair00/01/10, change on one channel, absent secondary header, reserved11; no cross-channel decoder selection |

Use typed offsets/units in evidence. Same-packet RTC/ERTC sign is observed RTC
minus floor(ERTC/100), modulo the48-bit counter. Preserve half-range ambiguity
with S01 ordering/context rules rather than silently selecting a sign.

## Requirement-level preparation ledger (no coverage claims)

| L2/L3 suffix | Planned additional evidence beyond existing parsers |
|---|---|
| ABS103 | distinct secondary/Format2 order, ns boundaries, explicit epoch/scale provenance, encode compatibility regression |
| ABS104 | relative1ns/full64-bit raw value, named-session correlation, same-packet floor relation |
| SEC101 | bounded12 bytes, Ch4 trust gate and independent internal/header reserved findings |
| SEC102 | IEEE own time after validation; first-data-bit/cited-exception applicability metadata |
| SEC103 | raw ERTC even without absolute context, validation and RTC disagreement separate |
| SEC104 | stored/byte/word aggregate evidence, trust only bytes; all synthetic readings; no local-recording/human-settlement claim |
| SEC105 | each reserved bit/field position retained, including simultaneous reserved words and unrelated errors |
| SEC106 | recording-wide formats by channel, per-packet dispatch retained, no missing-header default |
| ANS102 | own beside independent counter answer, compatible normalized discrepancy and tolerance boundaries, explicit fallbacks |
| FND105 | floor/modulo/raw counters/signed tick discrepancy, ±1/±2 and sub-tick remainder guards |
| ERR104 | failed own-time checksum cannot destroy a valid independent counter answer; failed-own evidence travels with it |
| RDG208 | byte trust/word comparison and TF012/027, suspect interpretation stays labelled; no human Review change |

Existing raw parser tests are baseline evidence, not new failing-first tests.
After merge, commit new adapter tests before implementation and record honestly
which failures are API absence versus runtime defects. Every finding emission
and fallback gets mutation-style checks. Whole markers only after the complete
requirement statement is exercised; partial clauses remain in this ledger.
If validation/adapter/context code exceeds about1500 changed lines, propose a
checksum/evidence versus conversion/comparison split before implementation.
Full bounded CI equivalent is for the eventual implemented slice, not this
analysis-only preparation. No Rust builds/full tests executed for this file.

## CLI naming carried forward (owner2026-11-15)

Later CLI units use crate `irig106-time-cli`, binary `irigtime` (L1), mounted
as `irig106 time` in `irig106-cli`. Owner says the crates.io name is being
reserved by a0.0.1 placeholder whose binary is `irigtime`. This is a naming
note for that later slice, not a dependency/version/CLI or publishing action.
