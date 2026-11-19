# S15e — review guard follow-up

Follows PR58's Linux CSV flush regression and interim whole-file input warning.
No S16 streaming implementation, CLI publication or release tag is authorized.

## Protected allowances and production I/O

The shared lexical/module-scope scanner lists file, line, protected names and
scope for every allow/expect affecting the nine accepted lints or unsafe_code.
The present tree has 19: nine library test-module allowances, nine named CLI
print-function allowances and one CLI test-module allowance. Explicit protected
lint allowances are permitted only in cfg(test) modules or for the exact print
channel of the named CLI functions. Unsafe and broad blanket allowances have
no exemption. Conditional attributes and raw lint identifiers are checked.

The I/O probe now scans all library source, following ordinary inline/external
module declarations and explicit paths. Only actual test scopes are excluded;
a *_tests.rs filename alone grants no exemption. Production extern crate std
and std:: paths fail, including child modules. Comments and literals are masked.
This is a conservative source-scope check, not a Rust macro expander. Generated
code, dependencies, foreign imports and callback implementations still require
the S15c/d source/dependency inspection; the guard is not a universal I/O proof.

## Evidence

- Tests-first commit aa17f13 specifies the guard interface; before the helper
  exists it fails import, not a reproduced Rust runtime defect. The original 13 scanner
  fixtures pass, including permitted test/CLI controls and rejected scopes.
- Actual-source negative probe: add root allow(clippy::unwrap_used) and a
  documented production unwrap. Ordinary library Clippy accepts the bypass;
  check-safety-policy.py rejects that exact root allowance as FORBIDDEN.
- Actual-source negative probe: add extern crate std and std::env::var to
  src/util.rs. check-library-no-io.py rejects both production occurrences.
  The Rust probe is never executed. Both files are restored after the probes.
- On restored source, both root compiler/Clippy policy sets pass their safe
  controls and reject unsafe/reallow plus all nine lint violations. The I/O
  compile control succeeds and clock/environment/file/network probes fail
  because std is unavailable. These probes emit metadata only.
- The focused fixed_scalar_recording_metadata_derive_inventory test passes
  with SecHdrTimeFormat added to the root re-export assertions. Formatting and
  all-target/all-feature Clippy are checked locally; the remaining matrix is
  delegated to PR CI under the owner's compute rule.

L2/L3-API-106 now explicitly replace the planned different-host-time/environment
replay with the std-unavailable compile probe plus source/dependency inspection.
No Statement, Source, Design or Review field changes. No new whole-requirement
markers: tested196/288 and verified202/288 remain unchanged. CI runs the scanner
fixtures before the two probe scripts; PR triggers are again main only, with
the temporary work/time-s15* allowance removed.

## Review follow-up — aliases and source indirection

Merged main 4d4f66e ordinarily before the review fixes; no merge conflicts.
Tests-first commit e70b62d adds five methods: before the fix, the 18-method suite
reports 12 assertion failures for unlisted aliases/suppressors and 7 subcase
errors for the absent source_errors interface (including the new safe control).
After the fix, all 18 methods pass. These are scanner-policy failures, not a
Rust runtime bug or a claim that compiler lints caught the bypasses.

All allow/expect occurrences of renamed_and_removed_lints and unknown_lints
are now listed and forbidden outside actual cfg(test) modules, including named
CLI functions. The four old clippy::option_unwrap_used, result_unwrap_used,
option_expect_used and result_expect_used aliases are listed as protected too.
Tests exercise both allow and expect for each spelling and the combined
renamed_and_removed_lints + option_unwrap_used bypass.

Production library and CLI source may not use include!, even for non-.rs files.
Reject the invocation rather than attempting expansion or following its path.
Both guard entry points reject library source indirections; the safety guard
also checks CLI source. Simple literal #[$ metavariable attributes are rejected
in production source, including macro_rules templates. No macro expansion is
attempted; other generated/obfuscated forms remain subject to code review. The
guard prevents accidental drift, not determined obfuscation. Actual test scopes,
comments and string literals retain explicit positive controls.

Four standalone metadata-only negative probes use the actual root policy and
Clippy -D warnings: (1) renamed alias + production unwrap, (2) non-.rs include
hiding an unwrap allowance, (3) non-.rs include hiding extern crate std/std::,
and (4) macro_rules emitting #[$l(clippy::unwrap_used)]. All four compile under
Clippy but fail the new source scanner. No fixture function or I/O executes.
The automated scanner fixtures also test both library and CLI include/macro
locations. Existing root lint and std-unavailable compile probes still pass.
Only scripts and documentation change; coverage remains unchanged. Formatting,
focused scanner/probe checks and the trace check run locally; full PR CI is
used for the remaining matrix, with no repeated local Rust build.
