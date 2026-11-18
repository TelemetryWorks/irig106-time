# Optional Jiff civil adapters — verification (0.8.0)

Owner2026-11-17; ADR-0020; baseline main121c412. Independent of PR50/ADR-0019.
No advisory ignore or Review/Design/finding-level change. Whichever PR merges
second merges current main, retaining both decisions and regenerating the matrix.

## Decision, types and resource boundaries

RUSTSEC-2026-0543 classifies all chrono versions unmaintained with no patched
release. Replace chrono0.4.45, its feature/module and infallible conversions.
Jiff1.0.1 and jiff-core1.0.1 declare Rust1.85; verify with the actual1.85 toolchain,
not metadata alone. Sources: [RustSec](https://rustsec.org/advisories/RUSTSEC-2026-0543.html),
[Jiff crate metadata](https://docs.rs/crate/jiff/1.0.1/source/Cargo.toml),
[Jiff civil DateTime](https://docs.rs/jiff/1.0.1/jiff/civil/struct.DateTime.html),
[Jiff features](https://docs.rs/jiff/1.0.1/jiff/#crate-features).

`jiff::civil::DateTime` matches the old naive field representation, not a UTC
instant. Return `Scaled<DateTime>` and require explicit scale on reverse input.
Preserve stated/effective/origin evidence; TAI/GPS labels/fields stay TAI/GPS.
Callers wanting UTC use existing explicit table conversion first and retain its
full basis, including stale currency. No Timestamp, timezone DB, wall clock,
parsing or default epoch/year is introduced. No implicit conversion/table lookup.
The known-year/known-scale refusal was already the requirement, unlike the old
unchecked1970/midnight fallback. Second60 cannot fit Jiff civil time and refuses.

Optional `jiff = [dep:jiff]` is off by default; dependency default features are
false. No Jiff std/alloc/timezone/JS features are requested: civil adapters need
none. The library retains its existing alloc layer. Target-specific dependency
inspection (`cargo tree --no-default-features --features jiff --target
wasm32-unknown-unknown -e features -i jiff-core`) shows only the direct Jiff/core
chain, no target std/alloc features. Cargo metadata without a target can union
host/optional feature sets; that union is not evidence of WASM std activation.
Normal default library/CLI/fuzz dependency trees exclude Jiff entirely. CLI/fuzz
lockfiles already exclude unused optional dependencies and remain unchanged;
locked builds/type-checks verify that. Only the root lockfile changes.

## Ordered evidence, not manufactured runtime failures

- Requirements/ADR first:8467c0e rewrites L1-API-011 and L2/L3-API-111.
- Tests/dependency harness first:328a0e3 adds optional Jiff and twelve initial
  test cases before any adapter implementation. The cleaned pre-implementation
  run fails compilation ONLY for missing `jiff_interop` and the new typed
  CivilYearOutOfRange error. No runtime-first claim. An earlier exploratory
  draft also used wrong fixture constructors (`LeapSecondTable::new` and
  `PtpTime::new`); corrected to existing builtin/convert_ptp before that commit.
- The six old unit cases are ported: AbsoluteTime->civil; CalendarTime->civil;
  civil->CalendarTime; roundtrip; missing year; calendar->DOY projection.
  Missing year now expects refusal, not the old1970 default, as already required.
- Seven additional edge cases bring the final suite to13. Gregorian fixtures
  use hand arithmetic (Apr10 common DOY100; Jul18 leap DOY200), year0/1900/2000
  century rules and year9999. No fixture is generated from the adapter under test.

## Clause coverage

| Requirement lineage | Covered evidence | Remaining boundary |
|---|---|---|
| L1-API-011 -> L2-API-111 -> L3-API-111: known year/scale and typed refusal | no_year_refuses; unknown_scale tests all four entry points; malformed public readings retain native range errors | No automatic UTC conversion or host year/scale inference; deliberate scope. |
| Same: fields, ranges and precision | Six ported cases; DOY/year0/9999/-1/-9999/century checks; clock/subsecond0 and999999999 extrema; malformed day0/common366/year10000/h24/m60/s61/ns1e9 | Jiff cannot represent second60; explicit refusal is the specified result. |
| Same: scale and leap semantics | UTC/TAI/GPS and assumed-UTC provenance tests; four eligible RDG-010 leap dates refuse; forbidden date remains malformed; covered/stale explicit PTP table conversion retains original status outside the projection | Adapters preserve civil fields, not source-reference/offset-table envelopes; caller retains those original records. |
| Same: optionality/no_std/WASM/MSRV; API-007/REL-102 | Actual1.85 no-default+jiff check, no-default+jiff tests, WASM+jiff build, default dependency exclusion, full native checks | Compilation/dependency inspection evidence, not a runtime feature-switch assertion. |

Tests are linked here at clause level; code Traces carry the full API lineage.
No new whole-requirement Requirements marker is added for a runtime case that
cannot itself verify dependency/MSRV/target clauses. Matrix remains192/288;
no claim of coverage from the obsolete chrono tests, and no false new status.
No new release deferral; no work on independent library JSON removal.

## Targeted mutations

Each mutant was applied alone, `cargo test --features jiff --test jiff_interop
<case> --locked` executed, and source restored. Twelve distinct runtime kills:

| Mutation | Killing case |
|---|---|
| Drop unknown-scale guard | unknown_scale_refused_in_every_entry_point |
| Missing year defaults1970 | no_year_refuses_instead_of_legacy_default1970 |
| Drop explicit leap-label refusal | explicit_second60_rejected_never_clamped_or_defaulted |
| Omit native reading validation | malformed_fields_reject_before_target_construction |
| Add1 to forward DOY | doy_boundaries_and_year_limits_are_checked |
| Zero forward nanoseconds | absolute_to_jiff_ports_absolute_to_chrono |
| Zero forward seconds | absolute_to_jiff_ports_absolute_to_chrono |
| Replace forward scale/provenance with UTC | scales_and_assumption_provenance_are_not_relabelled |
| Wrap negative reverse year as u16 | doy_boundaries_and_year_limits_are_checked |
| Replace reverse DOY with1 | jiff_to_calendar_ports_chrono_to_calendar_time |
| Zero reverse nanoseconds | jiff_to_calendar_ports_chrono_to_calendar_time |
| Replace reverse scale/provenance with UTC | scales_and_assumption_provenance_are_not_relabelled |

The later clock-extrema test strengthens boundaries/Display; the twelve kills
above were observed on the preceding12-test suite, not retroactively claimed
as tests-first failures on main. This is targeted mutation testing, not an
exhaustive cargo-mutants campaign or a claim that no possible mutant survives.

## Final checks

Full CI equivalent uses jobs2/test threads2, locked manifests: all feature
suites, no-default+jiff tests,1.85 default/no-default/serde/no-default+jiff,
clippy/fmt/rustdoc, CLI build/clippy, WASM no-default/serde/jiff, benches, all ten
fuzz targets, cargo-deny, trace freshness. Actionlint1.7.12 validates workflows.
No credential/policy changes, exception, publication, merge or ignored advisory.

Executed final result:21/21 local CI-equivalent checks PASS, including
cargo-deny with the unchanged policy. The first trace check detected stale
RDG interpretation links after test tags were added; regeneration and --check
passed, then the remaining checks passed. No failed compilation/mutation is
counted as a green runtime test. Actionlint/diff checks pass. The actual WASM
fingerprints for Jiff and jiff-core both list enabled features=[]; no target
std/alloc features are activated, even though the library recording layer
uses alloc. No default Jiff dependency; no chrono anywhere in the three
lockfiles. CLI and all ten fuzz targets remain checked with --locked.
