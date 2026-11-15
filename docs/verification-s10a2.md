# S10a2 — explicit own epochs, session ERTC and TF-014

Preparation started at PR37 c103d42 (S10a1), carrying its review fixes before
PR37 merged. Main8016942 was ordinary-merged for PR opening after PR37/39/38
merged. This record supplements verification-s10a.md; final checks follow below.

## Sources and interpretation boundary

Reused page-reviewed quotations, not fresh page rendering in this slice:
106-24R1 §11.2.1.1g, printed11-9/PDF17: "The 32 bits indicating seconds shall
be placed into the MSLW portion of the secondary header and the 32 bits
indicating nanoseconds shall be placed into the LSLW portion"; Figs11-5/6,
printed11-10/PDF18. Same section: "the 10-megahertz (MHz) RTC shall be
synchronized with the ERTC (RTC = ERTC/100)". Section11.2.1.2a: "The applicable
data bit to which the 48-bit value of the packet secondary time value (if
enabled) applies shall correspond to the first bit of the data in the packet
body". Chapter4 §4.7/Fig11-4 and T10's derived full32-bit count apply unchanged.

Contract6.6/L3-ANS-102 requires compatible normalized scales, signed own minus
counter and a strict above-tolerance finding, default reference resolution.
D6 requires explicit Chapter4 origin; D7 keeps last-known-offset UTC with TF021
past table coverage, raw TAI retained. RDG010 preserves second60. Epoch/count
interpretation for own IEEE is caller context, never inferred from Format2.

The adapter retains the exact borrowed offline/stream answer and full header
inspection, context/table and comparison policy snapshots. Caller supplies a
dated epoch, scale, source citation and Continuous or Civil counting for IEEE
or Chapter4. Continuous counts SI time; Civil counts nominal86400-second days.
A civil insertion-window coordinate is unresolved (not NTP-specific TF030).
All assigned own scales, including derived UTC, are labelled Policy. No nearby
packet origin, implicit Unix epoch, host clock or changed counter answer.

ERTC uses this packet's named-session counter calibration, corrected by signed
RTC/ERTC phase and0..99ns remainder; it never uses ERTC as Unix seconds. Stream
interpolation applies the released bracket slope with a difference of floors,
retaining old policy/fallback/basis; Standard/fallback retain the1ns unit rate.
Exactly half a counter range has unresolved signed phase, not a guessed zero.
Comparison normalizes the actual released calendar with the explicit adapter
table. The original counter table remains in its basis. Ambiguous/unconverted
network references cannot acquire a unique instant through their display date.
Applicability must match exactly, including cited exceptions. Metadata identity
mismatch is an error; missing conversion context retains counter/raw own facts.

## Hand-derived fixtures and honest failing-first evidence

Tests-only b556bbd has five tests and fails E0432, absent own_time API, on c103d42.
This is compile-time API evidence, not runtime failures of existing decoding.
Six later tests and added edge assertions are post-prototype evidence.
No existing raw decoder defect or fresh all-edition/local-recording check claimed.

| Fixture/source | Independent arithmetic |
|---|---|
| Fig11-5 | LE ns123456789 =15 cd 5b07, seconds01020304 =04 03 02 01; encoder compared with literal words, not just roundtrip; Format2 has reversed words |
| Explicit epoch | caller2017-01-01 UTC has TAI1483228837; secs1/ns305419896 gives1483228838.305419896TAI; counter+1s gives signed+305419896ns; a TAI epoch instead gives−36694580104ns |
| Fig11-4/T10 | low8000,usec270f,high0 gives327689999000ns; usec2710 rejected, reserved word1 warns and retains value |
| Fig11-6 | raw0/1/99/100/101/199/200 at RTC1 gives own−counter−100/−99/−1/0/1/99/100ns; maxu64/asymmetric words retain all bits; half-range unresolved |
| Contract6.6 | Format1 resolution10ms: delta10000000ns no14;10000001ns TF014; caller tolerance0 and level override are independent |
| S04/RDG010 | 2016-12-31 23:59:59UTC+1SI second =23:59:60; starting at60+1SI second reaches midnight; unscheduled June label rejected as context, no leap credit |
| D7 | caller2028TAI uses last offset37 with TF021/UTC retained; empty table keeps nativeTAI without UTC;1969 signedTAI stays negative;2200UTC is not truncated to32-bit seconds |
| Stream floor | RTC1s vs absolute1.000010s maps midpoint0.500005s;99ns remainder stays99;10ms own phase maps10000100ns; base100ns plus99999ns maps100100ns, difference100000ns |

Eleven focused tests pass.32 isolated mutants compiled and failed runtime
assertions, with sources restored: suppress each14/21/22;14 inclusive boundary,
sign, severity, related packet/session and rawRTC; converted severity; RTC sign
and remainder; stream unit-rate substitution; floor-of-difference; session,
packet, rawRTC, applicability and citation guards; leap roundtrip/label;
GPS offset; native Policy label; stale status; encoder word swap/ns bound;
unknown-scale guess; civil ambiguity guard; ambiguous-network guard; half-range
zero guess; missing-origin guess; missing-reference guess. One initially
mis-targeted stream mutant failed compilation and is excluded; its corrected
runtime substitution is included. No compile-error mutant is counted.

## Clause ledger and marker scope

| L2/L3 suffix | S10a2 evidence / remaining |
|---|---|
| ABS103 | Whole marker: independent IEEE encode/decode order, ns bounds, Format2 control, explicit scale/epoch conversion |
| ABS104 | Full64 relative value plus named-session absolute calibration now exercised; existing raw marker remains; no absolute epoch inferred |
| SEC101/105/106 | Reuses S10a1 bounded/checksum/reserved/per-packet format inspection; no format registry substitution |
| SEC102 | Whole marker: valid own calendar, raw words, first-bit/cited applicability and corrupted checksum |
| SEC103 | Whole marker: relative raw0/1/99/100/max/asymmetric, absent reference, bad checksum, independent phase findings and session scope |
| ANS102 | Whole marker: all formats, normalized signed delta, equality/above boundary, partial/no reference, checksum failure, unknown scale and applicability |
| FND105 | S10a1 floor/modular/tolerance/raw evidence integrated; existing clause tests retained, no new whole marker |
| ERR104 | Counter retained unchanged for offline and both stream modes on bad own checksums; checksum reading combinations remain S10a1 evidence, no new whole marker |
| RDG208/SEC104 | No all-edition/local-recording or human settlement claimed; suspect reading unchanged |

S10b stamp adapters, new complete encoder API (approved post0.8 deferral), JSON
snapshot schema and CLI propagation remain outside this slice. Matrix links
cannot express clause completeness. New whole links only ABS103/SEC102/SEC103/
ANS102:178→186/288 (+8 L2/L3); no other requirement promoted or rewritten.
No Review/Design fields or finding defaults change. No new owner decision.

Full verification and main-merge prerequisite are recorded below when executed.

Pre-merge preparation:18/18 locked checks pass (jobs/threads2): four feature
suites, three MSRV1.85 checks, clippy/fmt/docs, CLI build/clippy, no_std WASM
with/without serde, trace check, benches, all10 fuzz targets and cargo-deny.
Opening check: main8016942 ordinary-merged; all18 checks pass in a fresh target.
Dependency-only main3bedbd4 was then ordinary-merged; all18 checks repeated and
passed with its lockfiles. PR37's merge/final check gate was pending then.

Carried PR37 review head a56a6a2 by ordinary merge, including its four evidence
assertions and corrected figure pages; matching S10a2 citations corrected.

The initial reused-target no-default run reported missing module imports; a
fresh target with incremental off passes all18 with no Rust code changes.
