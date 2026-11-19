# S15a — library safety policy and legacy quality cleanup

Base57a46e1 (S14 merged). This is the first bounded S15 PR, not the completed
production boundary audit. Split: S15a library lint policy/quality; S15b CLI
unsafe ingestion/root; S15c traits, allocation/I/O and remaining module audit.
No Review/Design, dependency policy exception or release-scope change.

## Ordered evidence

- 0d20279 adds scripts/check-safety-policy.py first. On main it fails because
  the library root lacks forbid(unsafe_code), not a fabricated runtime defect.
- Explicitly enabling all nine accepted lints on the production library first
  produces two errors, src/quality.rs's min/max unwraps. Empty input was already
  guarded: these are forbidden operations, not a demonstrated panic on main.
- The quality control pins empty/singleton None spans and the two RTC extrema:
  (2^48-1)*100 =28,147,497,671,065,500ns. Initial test drafts used the wrong
  reference-slice call and expected Some(0) for the singleton; corrected to the
  existing API/convention before implementation. Corrected control passes main.
- Replace only quality's min/max unwraps with a domain-bounded fold. Preserve
  public fields, empty/singleton conventions, ordering and numeric semantics.
- Root forbid(unsafe_code) plus nine deny Clippy lints now apply to every library
  module. Nine cfg(test) modules get only the unwrap/expect/panic allowances
  actually needed by their test assertions. No root cfg_attr/test exemption,
  production allowance or blanket warnings allowance. Integration/dev crates
  do not inherit library root lints; their production boundaries are separate.

## Negative policy check

The script extracts the actual unconditional forbid/deny attributes from the
library root, requires all nine accepted Clippy names and unsafe forbid, and
compiles tiny independent fixtures with the installed clippy-driver. No crate
build per fixture and no new dependencies. It requires the matching diagnostic,
not just any compiler failure. Checked positive control must compile; unsafe
block, child allow(unsafe_code), unwrap, expect, panic, todo, unimplemented,
unreachable, dbg, stdout and stderr probes must fail. All twelve cases produce
the expected result locally. CI runs this after normal all-target Clippy.
This proves the policy rejects explicit violations, not arbitrary panic freedom
or that new production allowances could never be added in a future change.

## Clause coverage and remaining work

| Requirement lineage | This PR | Remaining S15 scope |
|---|---|---|
| L1-ERR-001 → L2/L3-ERR-101 | Remove two forbidden quality operations; bounded RTC extrema/empty control | Per-module slice/range/overflow/loop and allocation audit; panic lints do not catch every indexing panic |
| L1-API-009 → L2/L3-API-109 | Library forbid and negative child reallow check | CLI still uses unsafe file mmap; dependency unsafe is not claimed covered |
| L1-API-012 → L2/L3-API-112 | Nine library root denies; actual test-module allowances inspected; all-target/all-feature Clippy passes | CLI production/I/O-boundary work in S15b; final allowance audit |
| L2/L3-API-101/102/106 | No trait, allocation or no-I/O claim from lint success | S15c: meaningful trait assertions, fixed readers versus alloc recording state, closed dependency/I/O source audit |

Matrix remains196/288: partial clauses get no whole-requirement marker.
Locally: fmt, focused quality control, production-before and all-target/feature
after Clippy, negative policy fixtures. Full feature/MSRV/WASM/CLI/fuzz/deny
checks delegated to PR CI under owner compute rule2026-11-18.
