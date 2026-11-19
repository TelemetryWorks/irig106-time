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
  exists it fails import, not a reproduced Rust runtime defect. All 13 scanner
  fixtures now pass, including permitted test/CLI controls and rejected scopes.
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
