# S01 — typed quantities and stable finding evidence

> Current status (owner 2026-11-14): TF-031 WARNING is accepted (body kept,
> EXT/SRC qualification withheld); only TF-032 is pending (S08). Pending-level
> assertions and mutation descriptions below describe the historical S01 run,
> before S03. See [S03 verification](verification-s03.md).

Baseline main `e714f9a594e25143188f3d499ac2bc45f704a130` (PR F merged).
Scope: the S01 ledger in the coordinator's saved 0.8.0 plan, plus the
requested stale Q-CH4 comment correction. No conversion, edition resolver,
year inference, CLI restructure, schema or publication work is implemented.
No Design/Review fields, requirement statements or accepted finding rows change.

## Test-first provenance

- Tests-only commit `b5e1ccf`: `cargo test --test s01_existing` executed on
  main's implementation. **3 assertion failures, 1 pass**: accepting approved
  TF-029/030 overrides failed; offline and stream half-range rejection failed.
  The shared Rtc/Ertc baseline passed (correct code reused, not rewritten).
- The other seven initial tests (`s01_quantities`, `s01_findings`) could **not
  compile on main**, because the new APIs did not exist. This is missing-API
  evidence, not seven executed failing tests.
- Follow-up tests-only commit `e6b0bd0` added engine evidence assertions and
  an all-zero guard. The sentinel test executed against the then-uncommitted
  first implementation and **failed by assertion** before its guard fix. It
  was not an executed failure against main. The engine-evidence assertion
  arrived after the first implementation; no original failing-first claim.
- Final focused suite after review: **15 tests**, with five rustdoc cases
  (one successful duration example and four compile-fail unit/epoch/privacy cases).
  Compile-fail examples test unit/epoch mismatch; they do not count as runtime
  failures. Explicit successful imports/conversions are the positive control.

## Sources and hand-derived fixtures

Source inspection reuses existing cited requirements and archived-page records;
**no new page image was rendered or visually checked in S01**.

- 106-24R1 §11.2.1.1 i: "This is a free-running 10-MHz binary counter
  represented by 48 bits that are common to all data channels"; the counter
  "shall remain free-running during each session" (L2/L3-RTC-101/103/104).
  Thus 1 tick = 100 ns, 150000 ticks = 15000000 ns, max→0 = +1 tick,
  0→max = -1 nearest tick. Modulus=2^48; exactly 2^47 has no unique nearest
  sign. Explicit forward/backward context promises less than one full wrap;
  equal raw values do not prove a wrap count. Cross-session differences fail.
- Corrected shared `irig106-types` at `48f1834`: Rtc's `from_raw` deliberately
  masks; six-byte parsing retains all 48 bits. `checked_rtc` adds rejection
  without changing that shared masking contract. ERTC is 1 ns/tick, per
  §11.2.1.1 g / Figure 11-6 and T-5. The existing shared unsigned
  `NanosDuration(u64)` could not preserve a negative sign at S01; the then-local
  `SignedNanos` used checked i128 arithmetic and a shared-magnitude helper.
  S01b replaces those local definitions with the shared PR #8 types (below).
  Bounds test i128::MAX/100 and the next tick, both signs, i128::MIN, and u64
  magnitude limits; no saturation or truncation is used in the new conversions.
- Architecture §8 / owner 2026-11-13 assigns IDs/levels, not IRIG. A separately
  written literal test table checks all **30 approved default levels**.
  TF-001..028 catalog rows remain unchanged. TF-031/032 titles are present,
  but their levels are **None/pending owner S03/S08**; policy validation and
  finding construction reject attempts to emit/default/override those levels.
- RFC 5905 §6 p.14: "a value of zero is a special case representing unknown
  or unsynchronized time". TF-029 requires both raw NTP words zero, no mapped
  absolute value, raw kept. Zero seconds with fraction=1, or seconds=1 with
  zero fraction, cannot be labelled the all-zero sentinel.
- RFC 1305 Appendix E.8 pp.78–79 / RDG-012/RDG-010 describes the duplicate
  insertion coordinate. TF-030's helper **attaches caller-established evidence**,
  it does not detect the window or resolve the era. The plumbing fixture uses
  raw seconds3692217600 - epoch2208988800 = Unix1483228800 and raw fraction
  0x80000000 = 1/2 s, retaining both words and the supplied mapped whole seconds.
  The all-zero sentinel cannot be relabelled into a kept absolute value.
  Covered-window/table/era decisions and full converted fractional basis remain
  S04/S05 work; this fixture is not evidence those conversions are implemented.

## Clause-level coverage

| L2/L3 pair | S01-covered clauses | Remaining / boundary |
|---|---|---|
| RTC-101 | Shared Rtc identity; 48-bit extrema and bit48; explicit masking vs checked construction; no implicit calendar epoch. | Existing serde/value-load validation is audited in S12; no shared dependency source is changed here. |
| RTC-103 | Named nonzero session; checked modular signed difference; half-range refusal with explicit directional context; checked unwrapped PacketKey subtraction; offline/stream half-range refusal and explicit-boundary retry. | No inference of multiple wraps. Compatibility correlator APIs do not gain missing session metadata; later API/compatibility audits must not treat them as equivalent to the new checked boundary. |
| RTC-104 | Checked i128 ticks×100; both signs; typed overflow; checked unsigned shared magnitude; no negative-to-unsigned truncation. | Legacy calendar and network conversion arithmetic is not certified by these scalar tests; S02/S04/S05 remain blockers. |
| API-104 | New public quantity boundaries distinguish raw Rtc, signed ticks, signed nanos and shared NanosDuration. Shared epoch types retained; unit/epoch compile-fail examples; duration conversion has a successful control. S01b now re-exports shared SignedNanos/SignedRtcTicks/SignedUnixSeconds and uses checked conversions. | **Shared-type gaps closed by S01b:** [types PR #8](https://github.com/TelemetryWorks/irig106-types/pull/8), adopted at `7bfe47fc903efea342f3efba1ad0f527dd059a0d`, supplies signed durations/ticks/Unix and checked TAI→Unix. Local signed definitions are removed; checked Result adapters map None to QuantityOverflow. Pre-1970 signed values are retained. **Global API coverage remains partial:** legacy primitive signatures, calendar/network adapters and S02/S04/S05/S10/API audits remain; no schema freeze or complete-conversion claim. |
| FND-101 | Reusable stable catalog; approved defaults vs pending levels; configurable approved levels; factual effect independent of level; optional offset, raw fields/counter, related identities; both engines share catalog/evidence; malformed input stays unusable under Information override; TF-029/030 raw/value-effect plumbing. | **Partial producer coverage:** not every degraded-case producer supplies all domain-specific operands yet. Existing engine IDs/RTC/delta/related packet are populated; unknown offsets stay absent, not guessed from packet ID. Generic callers may supply actual offsets/expected fields. NTP producers S05, edition S03, year S08 and reports/validated schema S12/S17 remain. TF-031 WARNING is owner-accepted (2026-11-14); only TF-032 level requires owner confirmation in S08. |

No new release-scoped deferral is introduced. The four ADR-0018 deferrals are
unchanged; the remaining implementation/audit clauses above still block 0.8.0.
The matrix cannot represent a partially satisfied clause or a container test
versus a real producer. Its links are **not full compliance**. RDG-012 gains
an evidence-container test link, not full conversion/reading closure or human
Review. Existing E1/E2/slices 1/2 limitations remain in their own records.

## Targeted negative mutations

Each mutation was applied alone, compiled, caused an executed test assertion
failure, and was restored before the next run. No build failure is counted
as a killed mutation. These are later negative checks, not historical tests-first.

| Mutation | Assertion that caught it |
|---|---|
| Allow checked width to mask | out-of-width typed rejection |
| Silently choose a half-range sign | both offline/stream half-range regressions |
| Permit cross-session subtraction | named-session delta rejection |
| Use 1 ns instead of 100 ns per RTC tick | signed hand-arithmetic fixture |
| Wrap ticks×100 overflow | signed extrema must return QuantityOverflow |
| Convert negative duration through absolute value | negative unsigned conversion refusal |
| Give pending TF-031/032 a WARNING fallback | catalog pending/default/override tests |
| Change WARNING catalog defaults to ERROR | independent literal level table |
| Skip zero/ambiguity consistency guard | zero sentinel must not acquire an absolute value |
| Let severity selection repair factual validity | unchanged malformed evidence under Information override |
| Drop raw NTP fraction | exact half-second raw evidence |
| Drop engine raw RTC evidence | both engine evidence assertion |

All **12/12** mutations were caught by runtime assertions. No new stream
hold/cap/timeout/EOF fallback rule is introduced by this slice; existing E2
negative evidence and full-suite regressions remain applicable.

## Matrix and inspection checks

Matrix regenerated and checked: **116/288 (40.3%)** versus **106/288 (36.8%)**,
+10 linked entries / five L2/L3 pairs; 124 L1 / 144 L2 / 144 L3 unchanged.
Zero orphans and zero unknown markers. API-104 and FND-101 retain the partial
clauses above despite a linked row. No human verification state is promoted.

## Executed final CI equivalent

All **16 checks passed**, `CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=2`:
all-features/no-default/serde/chrono test runs (including existing E1/E2 and
wire regressions), three MSRV1.85 library checks, all-target/all-feature
clippy, fmt, warnings-as-errors rustdoc, CLI build/clippy, two no-default
WASM builds, trace check and cargo-deny. No dependencies were added/updated.
The CLI, manifests/lockfiles, CI, governing requirement text and human
Design/Review fields are unchanged. No merge, publish, tag or settings action.

## Review follow-up at d06529f

Tests-only `34664c5` strengthens already-correct code; it is not a new
main-failing behaviour fix. Each of these **10 isolated mutants compiled and
failed by assertion**, and was restored before the next check:

| Mutant | Independent assertion |
|---|---|
| Remove backward equal-counter guard | Equal counters in BackwardWithinOneWrap must give 0, not -2^48. |
| Remove zero-session guard | Session 0 vs session 0 must be rejected. |
| Drop TF-030 mapping check | Kept/nonzero raw, mapped=None is rejected. |
| Drop TF-030 validity check | Nonzero raw/mapping present, validity=Unusable is rejected. |
| Drop TF-029 mapping check | Zero raw/Unusable, mapping present is rejected. |
| Drop TF-029 validity check | Zero raw/no mapping, validity=Kept is rejected. |
| Wrong structured related packet | Both engines retain packet41, not current99. |
| Wrong structured delta sign | Both engines retain -500000000/+500000000 ns. |
| Drop legacy related packet | Both legacy views equal Some(41). |
| Reverse legacy delta sign | Both legacy views equal the signed hand-computed residual. |

The four NTP fixtures each violate **one** invariant only. Both residual
signs use 1 s of RTC and 0.5/1.5 s of absolute progress (§11.2.1.1 i).
Finding/FindingEvidence fields are private with getters/consuming builders;
a checked Finding cannot be altered or fabricated by a struct literal.
Privacy compile-fail examples have successful checked-construction controls
in the runtime suite. Title lookup is checked/table-based; observed finding
construction and both engines propagate Result. Offline finish() now returns
Result<OfflineTimeline>; existing call sites were migrated. The saturating
shared TAI→Unix call is removed from the non-saturating positive control.
RDG-010 is removed from the NTP container marker: it exercises no second60.
TF-031/032 levels and all Design/Review fields remain unchanged.

## S01b — shared quantity adoption

Baseline main `02ccac480472e9e62a5b8f04b0ee6e6af11e1f18` (S01 merged).
Adopt [irig106-types PR #8](https://github.com/TelemetryWorks/irig106-types/pull/8),
revision `7bfe47fc903efea342f3efba1ad0f527dd059a0d`. Root, CLI and fuzz lockfiles
resolve that revision; other packages/versions and deny.toml are unchanged.
At `7e13e10`, only fuzz dependency resolution had been refreshed; harness
compilation was missed. Follow-up `bfa5bd9` type-checked all ten targets, with
no instrumented fuzz campaign or runtime/corpus coverage claim.

The shared-type API-104 gaps above are closed: signed duration/tick/Unix values
have exact shared identity. Shared inherent conversions now return Option;
`rtc::signed_ticks_to_nanos`, `signed_nanos_magnitude` and
`signed_nanos_to_unsigned` retain time's Result/QuantityOverflow boundary.
No local shadow types or saturating fallback remain in those conversions.

Both former network_time to_unix call sites now use signed conversion.
`PtpTime::to_signed_utc_seconds` retains pre-1970 values with the supplied
offset. Its unsigned `to_utc_seconds` now returns Result<u64>, refusing values
outside u64. The legacy calendar/offline/stream Format 2 adapters propagate
that error before adding a reference; they no longer clamp a negative UTC
reading to Unix zero. This does not implement pre-1970 calendar reconstruction,
new leap-table validation/coverage policy, or the S04/S06 conversion adapters.
The existing approximate lookup/empty/before-first policies are preserved;
comparison uses signed i128, without clamping an approximate Unix value.

The initial duration-constructor audit missed fuzz_correlation. After its
correction, the caller audit covers library, CLI, tests, benches, examples and
fuzz harnesses. All ten fuzz targets were type-checked with
`CARGO_BUILD_JOBS=2 cargo check --manifest-path fuzz/Cargo.toml --bins --locked`.
Test and bench constants handle Option explicitly as bounded fixtures. The CLI threshold changes one expression
from from_secs(1) to new(1000000000), preserving exactly 1 second and threshold output. A pre-1970 PTP result
instead follows the existing CLI correlation-error path and is not inserted
or displayed as 1970 (source inspection; not a CLI-input execution).
Rustdoc/usage examples use explicit fixed nanoseconds or checked conversions.
No unwrap/expect was added to library/CLI data paths; existing unrelated
prototype arithmetic is not certified by this adoption.

Tests-only `b01aaf0` executed against main's old manifest/lock/source and
**failed by assertion**: PTP seconds0 - supplied offset10 = Unix -10 was
silently clamped into a 1970 calendar value. After adoption it is retained by
the signed API and rejected by the unsigned/calendar boundary. Three further
adoption tests were added during implementation, with no original failing-first
claim: shared type identity/typed overflow, signed PTP/refusal before insertion,
and checked duration/epoch extrema. Existing S01 signed limit tests remain.
Fixtures reuse §11.2.1.1 i (100 ns/tick) and §11.2.3.3's PTP epoch/TAI scale;
the offset is explicit caller input, not an inferred historical leap offset.
No new page-image inspection is claimed.

Five isolated mutations each compiled and failed an executed assertion:
restore the unsigned epoch clamp; clamp the signed epoch result; return zero
on tick-conversion failure; return zero on magnitude overflow; return zero on
negative/out-of-range unsigned-duration conversion. Each was restored before
the next check. The adoption test checks u64 constructor bounds and exact
u64::MAX+1 signed Unix evidence as well; no dependency source was mutated.

S02 is not started and requires a separate go after S01b merges. Shared-type
gap closure does not promote global API-104 or NTP/PTP implementation coverage,
and no Design/Review fields or pending TF-031/032 levels change.

S01b initial checks at `7e13e10`: **17/17 passed**, CARGO_BUILD_JOBS=2/RUST_TEST_THREADS=2:
the full 16-check CI equivalent (feature tests, MSRV1.85, clippy/fmt/rustdoc,
CLI build/clippy, WASM±serde, trace, cargo-deny), plus an explicit
`cargo build --benches --all-features`. The matrix remains 116/288 links,
zero orphans/unknown markers; coverage delta is 0. No new feature completion
is claimed by these adoption tests.

### Early fuzz review correction

The initial S01b caller audit missed fuzz_correlation's duration constructor.
`CARGO_BUILD_JOBS=2 cargo check --manifest-path fuzz/Cargo.toml --bins --locked`
executed before the fix and failed with E0308: expected NanosDuration, found
Option<NanosDuration>. The harness now names its fixed 1 ms threshold as
NanosDuration::new(1000000), with no multiplication, unwrap or overflow fallback.
The zero-threshold oracle is retained.

The all-target command also exposed older BCD harness API drift: private fields
and the removed DMY to_absolute method. Those harness sources and the BCD
library API were unchanged on main (source inspection, not a claimed successful
main all-target fuzz check). Only harness compatibility changes were made:
getters, checked to_calendar_time and existing invariants on successful
conversion; calendar rejection is an expected Result, not a new fuzz crash.
An unused NTP import was removed. No library BCD/calendar behaviour changed;
S02 has not started.

The new compile-only Fuzz targets compile CI job explicitly checks all ten bins
on stable, CARGO_BUILD_JOBS=2, locked dependencies and a ten-minute job limit.
This job was announced before addition; no permissions/credentials/settings or
fuzz campaign were added. CONTRIBUTING lists the same command in the local CI
equivalent. Root --all-targets does not include this separate package.
This is compile coverage, not instrumented fuzz execution or runtime/corpus
coverage, and changes no requirement or human Design/Review field.

After the fuzz correction, **18/18 local checks passed**: the prior 17-check
suite plus the locked all-fuzz-bin check. No warnings remained in that check.
The new CI job runs the same command; matrix links remain 116/288, delta0.
