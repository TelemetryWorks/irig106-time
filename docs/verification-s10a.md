# S10a1 — secondary validation, evidence and RTC/ERTC consistency

Initial preparation started from main `e068372e5fd8dab70c897867812542323a2fe325`.
Owner GO 2026-11-15 is conditional on PR31 merging; the coordinator will relay
that merge. The initial preparation commit contained analysis/fixtures only,
with no production/API or requirements/Design/Review changes. GitHub then
confirmed PR31 merged; current S10a1 implementation/evidence is recorded below.

## Sources and constraints to preserve

Existing requirement/contract/register quotations, not fresh page rendering:

- 106-24R1 §11.2.1.2 / Figures11-4/5/6: "The length of the packet secondary
  header is fixed at 12 bytes (96 bits)". Time occupies[0..8), header-reserved
  bytes[8..10), stored checksum[10..12). Figure11-4 is printed11-10/PDF18;
  Figures11-5/6 printed11-10/PDF18. Retain both Chapter4's internal reserved
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
analysis-only preparation. No Rust builds/full tests were executed at the initial preparation stage.

## CLI naming carried forward (owner2026-11-15)

Later CLI units use crate `irig106-time-cli`, binary `irigtime` (L1), mounted
as `irig106 time` in `irig106-cli`. Owner says the crates.io name is being
reserved by a0.0.1 placeholder whose binary is `irigtime`. This is a naming
note for that later slice, not a dependency/version/CLI or publishing action.

## Merge and split before implementation

GitHub confirms PR31 merged as mainca63a20; preparation branch merged that
main (c5204a0). The conditional owner GO is now satisfied. To keep reviews
bounded, split the planned work before writing production code:

1. S10a1: twelve-byte inspection, both checksums/trust, independent reserved
   evidence, IEEE/Ch4 range validation, raw relative ERTC, same-packet RTC/ERTC
   check, format registry/aggregation, applicability metadata, and a borrowed
   counter-answer carrier that cannot discard the caller's independent answer.
2. S10a2: epoch/scale/calendar conversions and session correlation, normalized
   own-minus-counter comparison/TF014/tolerance and full own-answer adapters.

S10a1 must not promote ANS102 or conversion clauses by partial tests. SEC104/
RDG208 real-recording/human-settlement work stays explicit and suspect; no
Review field changes. Later conversion must use D7 rather than inventing a
no-UTC stale-table rule. Source/clock assumptions require explicit provenance.
No code changes to existing correlation/streaming/CLI are planned in S10a1.


## S10a1 implemented boundary and evidence

Implementation adds secondary_packet inspection/audit and a borrowed counter
carrier. It is additive: existing raw parsers, correlation, streaming and CLI
code are unchanged. Raw header storage is capped at12 bytes. Audit state is a
first format plus at most one entry per u16 channel and checked u64 counters;
there is no packet/answer queue. Format facts come from primary flags even when
an independent secondary checksum fails. Reserved selectors do not establish
a canonical format; missing headers do not imply00. Same-channel returns to
the original format remain changes. Actual packet flags always select dispatch.

TF004/005/006/012/013/025/027/028 use catalog levels with independent factual
validity. CRC/field errors keep the original borrowed offline OR stream answer.
The caller supplies the same packet's counter answer; the carrier does not
prove compatibility or compute a discrepancy. Metadata errors return Result
without consuming the caller's answer. Reserved fields remain Kept even with
Error override; checksum/range failures remain Unusable even with Information.
Invalid format has separate ReservedPacketTimeFormat/UndefinedSecondaryTimeFormat
evidence. First-data-bit/caller-cited exceptions and source bytes are retained.
No clock epoch or UTC/TAI/GPS label is guessed; raw own scale stays Unknown.

Same-packet ERTC checks use floor(ns/100), modulo2^48 and observed-minus-expected
nearest signed phase. The sub-tick remainder is a shared NanosDuration, phase
distance is a nonnegative SignedRtcTicks, and full raw ERTC remains shared.
Exactly half-range has known unsigned distance outside tolerance, TF025 and
AmbiguousCounterDelta for its sign; no guessed signed zero. Both raw counters,
expected value, signed delta when unique and unsigned distance are evidence.
This does not correlate ERTC to an absolute date or select another session.

### Honest tests-first execution

Existing raw baseline s10a_existing passes at the merged main: distinct IEEE
words and ns=10^9 rejection were already correct. No runtime defect claimed.
Tests-only862a264 had the missing new module plus three fixture/API assumptions
(no Ieee1588Time::to_le_bytes, Ertc::to_nanos returns u128, integer inference).
Corrected tests-only733bc67 fails ONLY with E0432: secondary_packet API absent.
It precedes any implementation commit. This is compile-time API evidence, not
seven runtime failures. Later override, metadata, stream, format-registry and
raw-operand guards are explicitly post-prototype tests, not failing-first claims.

Final focused tests: nine new inspection/audit tests plus one passing raw
baseline. Header/checksum arithmetic is tabulated above. The ERTC fixtures add
each of eight literal byte weights for their checksum without a crate encoder;
reserved-bit fixtures use one set bit's BYTE weight, not its word weight.
No fresh page inspection, local recording, fuzz campaign or encoder oracle.

31 isolated mutants compiled and failed runtime assertions, sources restored:
all eight finding suppressions (004/005/006/012/013/025/027/028); absence default00;
unbounded raw prefix; each reserved field skipped; wrong per-packet decoder;
word-only admission; ERTC*100 instead of/100; tick tolerance >=1 or>2; signed
evidence reversed; lost remainder; half-range guessed zero; format finding
suppressed; checksum both misclassified; ignored level override; missing citation,
overflowed byte offset or mismatched cited type accepted; header error propagated
instead of returning the counter carrier; reserved format poisoning the registry;
return-to-canonical change hidden; checksum failure hiding format facts; and
word sum losing modulo arithmetic. No compile-error mutant counts.

| L2/L3 suffix | S10a1 clauses | Remaining / marker scope |
|---|---|---|
| ABS103 | distinct IEEE raw order and ns bounds; Unknown scale, no Format2 epoch reuse | calendar/context conversion and encoder compatibility audit in S10a2; no new whole marker |
| ABS104 | full64 relative1ns, remainders and same-packet floor relationship | absolute session correlation in S10a2; no new whole marker |
| SEC101 | bounded12, Ch4 byte-trust gate, raw values and both reserved fields | whole marker supported by comprehensive raw/bounds/reserved test |
| SEC102 | validated IEEE raw result and first-bit/cited-exception metadata | own calendar/context adapter S10a2; no whole marker |
| SEC103 | validated raw relative ERTC without a reference; RTC disagreement separate | absolute session adapter S10a2; no whole marker |
| SEC104 | stored/byte/word trust and per-recording aggregation | real-recording/human settlement remains; no new whole marker |
| SEC105 | all32 reserved-bit positions, both fields, raw bits, original bytes, Kept semantics | whole marker; no reserved warning discards a value |
| SEC106 | all defined format pairs, per-channel change, reserved/missing exclusion, actual dispatch and failed-checksum independence | whole marker |
| ANS102 | borrowed independent answer beside raw own evidence | epoch/scale normalization, compatibility/discrepancy/TF014/tolerance all S10a2; no whole marker |
| FND105 | floor/modulo, ±ticks/wrap, raw/expected/delta/remainder and half-range sign unresolved | conservative clause evidence in this partial adapter slice; no new whole marker before S10a2 integration |
| ERR104 | checksum errors preserve original borrowed offline/stream answer and all checksums | full final own-answer integration S10a2; no new whole marker |
| RDG208 | strict byte trust, word-only TF012+027 and aggregate evidence | suspect status/local-recording/human review unchanged; no new whole marker |

S10a2 is remaining0.8 work, NOT an owner-approved release deferral. RDG008's
Design/Review and all finding rows remain unchanged. New whole markers are
limited to SEC101/105/106; all other new evidence is clause-level only.


Final local verification:18/18 locked checks pass with jobs/threads2: all feature
suites, MSRV1.85, no_std WASM±serde, CLI build/clippy, benches, all-fuzz compile,
fmt/docs/trace/cargo-deny; actionlint/diff clean. Matrix176->178/288 (+2: SEC106
L2/L3); SEC101/105 were already marked and gain tests. No other status promoted.
Matrix test-link status cannot express clause completeness: earlier Implemented
raw-parser rows are not evidence that S10a2 conversions/adapters or human/local-
recording settlement are complete. This ledger carries those remaining clauses.
No Review/Design field changes; hosted checks are verified at reported head.

### PR37 review: page citations and finding evidence

Correct the figure citations to printed11-10/PDF18: Figures11-4/5/6 and
§11.2.1.2a-c share that page; printed11-11/PDF19 is Packet Body §11.2.1.3.
No new page rendering or change to the accepted word layout is claimed.
Ordinary-merged main3bedbd4. Existing tests now assert TF028 Unusable, TF025
and TF013 Kept, and checksum-finding offset header+10 (24+10=34), not+8.
Each isolated reviewer mutant X5/X24/X30/X15 compiled and failed these runtime
assertions; production restored. Ten focused tests pass; matrix178/288 unchanged.
All18 locked local CI-equivalent checks repeated successfully after the merge
and assertions; actionlint, trace check and diff clean. No production behaviour
or requirement/Review/Design changes.
