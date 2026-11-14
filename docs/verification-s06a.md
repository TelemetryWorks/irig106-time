# S06a — Format 2 model and conversion/admission boundary

Baseline main `12dec35e6524d817d827bbce533a81ce7f3a6cef` (S05 merged).
Scope is the saved plan's S06a, not S06b: eight L2/L3 pairs, format-neutral
capture metadata, bounded raw Format 2, conversion and admission eligibility.
No timeline/stream/CLI/dependency or finding-level changes. TF-032 remains
pending; no Design/Review field is changed. These types have no frozen JSON
schema; that remains S12. No new release-scoped deferral.

## Sources and hand-derived fixtures

Source inspection uses the governing L2/L3, existing STANDARD-REVIEW T-2/3/4,
RDG-009/012 analysis, and S04/S05 verified conversion contracts. This slice
claims no fresh page-image inspection or new standard interpretation.

- 106-24R1 §11.2.3.3 / Figures11-15..17, printed11-26/27, PDF34/35:
  "Network Time Format (NTF). Bits 7-4", "Time Status (TS). Bits 3-0",
  "Reserved. Bits 31-8 are reserved". Literal `01`, `11`, `21` low bytes
  separate NTF0/1/2 from valid TS1. All16×16 combinations are checked;
  reserved NTF3..15 has no decoded body, regardless of TS or policy.
- Figure11-17 eight-byte body `[04 03 02 01 15 cd 5b 07]` means
  seconds0x01020304 / nanoseconds123456789, never secondary-header order.
  Exactly12 payload bytes include the CSDW; no fifth body word or filler read.
- "The PTP is referenced in International Atomic Time with an epoch of
  January 1, 1970. The PTP time does not include leap seconds" (§11.2.3.3).
  Epoch0 remains TAI. Unix2017-01-02=1483315200, TAI adds37=1483315237;
  UTC projection subtracts37 using the S04 table, while raw TAI stays intact.
  NTP with the same numeric seconds instead maps to
  1483315237−2208988800=−725673563 (signed, proleptic pre1972 basis).
- RFC5905 §6 Figure4 p14: era1 offset63104 maps to
  4294967296+63104−2208988800=2086041600 (2036-02-08).
  The full caller declaration is preserved, not replaced by era0.
- RFC1305 E.8/Figure8 pp78–79 / RDG-012: the 2017 duplicate coordinate
  Unix1483228800 +2208988800=NTP3692217600, fraction0x80000000=0.5s,
  produces TF-030 WARNING/value kept. All-zero NTP produces TF-029 and no
  eligible absolute reference. There is no TAI-UTC adjustment to NTP.
- 106-17 Table11-4 printed11-6/PDF14 (T-4): Format2 starts at106-17.
  Explicit pre17 editions get TF-026 but keep raw/converted data;17+ and
  unknown remain distinct. Header data type version8 is retained, not used
  to infer a recording edition (106-24R1 §11.2.1.1 e, "at or below").
- D7 / owner2026-11-13: PTP after coverage retains UTC using the last known
  offset, TF-021 WARNING and raw TAI. Empty/pre-start table leaves UTC absent,
  while native TAI remains eligible. This is not a new fallback or default.

## Tests-first and executed evidence

1. `existing_bounded_wire_control` passed on unchanged main: bounded wire
   decoding already worked. No runtime failure on main is claimed.
2. Tests-only commit `693539c` added that control and seven new boundary tests.
   The new tests could not compile on main: `irig106_time::format2` did not
   exist. This is missing-API evidence, not seven assertion failures.
3. After adding the API, corrected test spelling to the existing shared
   `Rtc::from_raw`, S05 `era_basis`, and exact ERR/COR L1 ancestors; removed
   a redundant import. These compile repairs are in the implementation commit.
4. Added the ninth test (explicit era / unresolved half-era context), capture
   guards, raw epoch/equal-numeric scale checks and independent undeclared /
   malformed operand assertions after implementation. They are not described
   as main-first failures. Added the tenth test after the first full CI run:
   explicit inserted PTP second60 retains TF-022 and capture evidence; it also
   is not main-first. All ten focused tests pass.
5. Two initial mutations survived (valid-but-undeclared exclusion was masked
   by an invalid TS in the fixture; malformed finding operand unasserted).
   Added independent assertions. Final24 isolated mutations all compiled and
   failed by assertion, with the original production file restored afterward.

## Clause-level ledger

Each row names both L2 and L3 with the same suffix. Trace markers indicate
links, not proof that every clause of a composite requirement is complete.

| L2/L3 pair | S06a-covered clauses | Remaining / boundary |
|---|---|---|
| F2CSDW-101 | Immutable observation, four-byte LE CSDW, separately retained NTF/TS and raw eight-byte body, bounded12-byte consumption, neutral capture identity/RTC with no fake Format1 fields | Encoding closure is S13; engine insertion is S06b |
| F2CSDW-104 | All NTFs, reserved raw retained/TF-006/no body time,2002/2008 distinguished in raw reading while both use PTP conversion | S06b carries this full evidence through engine bases; numeric TFMT comparison is S09a |
| F2CSDW-105 | All TS with all NTFs; TS0 TF-003, reserved TS TF-006, valid-only default; explicit readable-invalid opt-in labelled without rewriting status | Reference eligibility is exposed, not yet wired into either engine; S06b |
| F2CSDW-106 | Every bit8..31, separate TF-005 exact mask and capture; malformed body cannot hide it; NTF/TS warnings remain separate | Engine/report propagation S06b/S17; existing raw parser unchanged |
| PTP-102 | Raw typed TAI/nanoseconds and1970 epoch; no parse-time subtraction; S04 table-qualified UTC retained separately, including D7 stale/empty cases | Mixed-format normalized arithmetic/admission in engines S06b |
| TAI-105 | Native NTP stated UTC / PTP stated TAI; equal numeric seconds have different epochs; conversion and basis never overwrite native scale | Partial overall requirement: GPS/Format1 existing paths unchanged; normalized mixed references S06b/S10b |
| TAI-106 | Reserved NTF remains unknown and cannot be admitted by a scale/status opt-in; stated NTP/PTP cannot be relabelled by this API | Format1 unknown/assumed scale conversion and full policy/schema remain their planned slices |
| EDN-101 | Declared04..15 TF-026,17/19/20/22/23/24/24R1 accepted, unknown retained, observed0x12/header version separate; warning does not discard value | Caller supplies declaration; S06b/S16 adapters carry it without inferring edition |
| COR-106 (boundary contribution) | TIMEIN/native-readable/status eligibility, invalid/undeclared opt-ins and snapshots; malformed/reserved NTF/NTP zero/unresolved era never admitted | No default engine admission changes until S06b; governing declaration ranges S09 |
| NTP-102/104, PTP-104 (reuse) | Session-compatible caller context reaches S05 unchanged; TF-029/030 and S04 TF-021/022 facts retained with capture RTC and permanent IDs | Full converter coverage remains S04/S05; S06a adds adapter checks, not independent re-verification of all algorithms |

The plan explicitly assigns **COR-108 raw-network-rate exception to S06b**:
"unless it is recorded at the raw network rate of the NTP or PTP frames"
(106-24R1 §11.2.3.3, printed11-26/PDF34). This slice does not evaluate gaps,
produce TF-016 or change inert engine metadata. Rate qualification, mixed-format
offline/stream insertion, scale-normalized jumps/drift/quality, preserving prior
eligible references, stream commitment and no revision all remain S06b.

Matrix links increase **152→156/288** (F2CSDW-101 and PTP-102 L2/L3).
The matrix cannot express clause-level partial coverage. This record controls
those limits; no partial requirement is deleted, waived or claimed complete.

## Targeted negative checks

Final24 isolated mutants compiled and failed focused assertions:
TF-003 identity; reserved TS TF-006; reserved NTF TF-006; TF-005 omitted;
zero-mask operand cleared; malformed TF-004 omitted; malformed operand renamed;
TF-026 omitted; TF-008 omitted; sentinel/unresolved NTP admitted; invalid TS
admitted by default; undeclared valid reference admitted by default; invalid
opt-in ignored; undeclared opt-in ignored; native TAI lost without UTC; PTP
relabeled UTC; caller era discarded; cross-session context allowed; NTP
conversion findings dropped; PTP conversion findings dropped; converted finding
RTC removed; severity override ignored; session0 accepted; pending-level override
accepted. These are local mutation checks, separate from tests-first evidence.

A direct PTP insertion fixture uses TAI1483228836=B+old_offset36
with B=1483228800 at the2017 transition; it retains UTC2016 day366
23:59:60.5 and TF-022 Information, rather than TF-030.

Severity overrides never change recorded TS, conversion status or admission.
Native conversion can remain visible when admission rejects it: ValueValidity
Kept records that retained diagnostic value, not timeline eligibility. Rejected
references have no opt-in-use labels; eligible opt-ins retain explicit invalid /
undeclared labels and policy snapshot. No hidden clock, file or network I/O.

Full bounded CI equivalent: all feature tests, MSRV1.85 default/no_std/serde,
clippy/fmt/rustdoc, wasm no_std±serde, CLI build/clippy, benches, locked all-fuzz
compile, trace freshness and cargo-deny; worker/test-thread caps2. Compile
checks are not a fuzz campaign. No merge or publish performed.

## Review follow-up at 8da5588 — tests only

Add four focused tests against unchanged reviewed code: **14 focused tests**
now pass. These are review coverage additions, not new main-first failures.
Total executed local negative checks: **24 initial + 8 review = 32** isolated
assertion-killed mutations. No production change or newly discovered bug.

Each review mutant was applied alone to `src/format2.rs`, compiled, and run with
`cargo test --test s06a_format2 review_` (worker/test-thread caps2). Every mutant
failed a new assertion; the original file was restored and its diff checked empty.

| Isolated mutation | New assertion that fails |
|---|---|
| Declaration gate consults include_invalid instead of include_undeclared | Valid TS/undeclared channel with only include_invalid remains ineligible |
| admission.undeclared records the raw undeclared flag on rejection | Rejected reference's undeclared-use label must be false |
| Known-format malformed body classified UnsupportedFormat | PTP nanos1000000000 with NTF2 must match NetworkConversion::Malformed |
| Readable TS0 TF-003 evidence marked Unusable | Both NTP and PTP fixtures retain ValueValidity::Kept despite default admission rejection |
| Converted TF-021 override alone ignored | Stale PTP must report Information, not default Warning |
| Converted TF-022 override alone ignored | Explicit inserted PTP second must report Warning, not default Information |
| Converted TF-029 override alone ignored | Zero NTP must report Information, not default Warning |
| Converted TF-030 override alone ignored | Ambiguous NTP must report Information, not default Warning |

Override fixtures reuse the hand-derived coordinates above. Each separately
asserts unchanged conversion, factual evidence and admission, with the actual
policy snapshot retained. The PTP inserted-second case still asserts second60;
zero NTP stays unusable and ambiguity keeps its mapped value. The reserved-NTF
TF-006 validity mutant is equivalent and is not counted as killed.

Regenerated matrix links remain **156/288**; clause boundaries and S06b deferrals
are unchanged. Full bounded CI equivalent rerun for this tests-only revision.
