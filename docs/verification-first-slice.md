# First rebuild slice — executed verification

Base: `be33a72ce34b93fad3a8b06ebcfe4ebc18989a99` (main be33a72). Tests-only first commit:
`5692427e632b1ebb876452b3ce3d7f896ff9791f`. Stable Rust 1.99.1; MSRV Rust 1.85.0;
all Cargo commands use `CARGO_BUILD_JOBS=2`. No agents were spawned.

## Before the fixes

With the prototype library and original e513af4 dependency, executed:

- `cargo test --test rebuild_wire`: **0 passed, 11 failed**, assertion failures.
- `cargo test --test rebuild_layout`: **compile failure**, missing explicit-layout,
  ITS and separate version APIs. These five tests did not execute on main.

The first commit changes only the two test files. All fixtures are independent
bytes/constants from the standard, not produced by the implementation encoder.
Additional boundary assertions were added to the same regression cases with the
fix; the originally committed assertions and expectations are unchanged.

| Finding | Before, executed against main | After |
|---------|-------------------------------|-------|
| T-1 | 0xc0 chose RTC, not Chapter 4; 0x04 chose IEEE, not RTC; exhaustive flags test failed | Correct bits 6/7/3–2; all 256 flags checked; strict L1-IPT-007 retained |
| T-5 (shared adoption) | ERTC raw1 returned100 ns | Returns1 ns from types 48f1834 |
| T-6 | SRC3 was Gps; SRC2 InternalRtc | Reserved(3), InternalFromRmm; explicit legacy EXT, no inference |
| T-7 | FMT15 was Reserved(15); explicit ITS/layout APIs absent (compile failure) | Modern None; legacy FMT15 reserved; all ITS/FMT combinations and per-layout masks pass |
| T-9 | setup15 was V23; new code-list API absent (compile failure) | Unknown(15), zero unknown, separate shared setup/header lists over all256 byte values |
| T-10 (shared adoption) | Figure11-4 fields decoded (9999,4660,1) rather than (1,4660,9999) | reserved/usec/low/high; top binary low bit retained; short/reserved/usec cases checked |
| T-12 | six-byte day body rejected (expected8); eight-byte DMY rejected (expected10) | Exact6/8-byte reads/encoding; all truncations and nonzero following filler checked |

After fixes, `cargo test --test rebuild_wire --test rebuild_layout` passes
**11 + 5 tests**. The old unit/integration assertions encoding incorrect GPS,
V23, Pre07 and BCD-length assumptions were corrected. The local version table
is deleted; version types/mappings come from irig106-types 48f1834. Caller
layout selection is a separate required argument, not derived from a header
version, G\106 or an unresolved policy.

## Full local CI-equivalent commands

These are executed checks, not inferred results. Commands correspond to
.github/workflows/ci.yml on this Linux host; GitHub performs the OS matrix.

| Command | Result |
|---------|--------|
| `cargo test --all-features` | PASS |
| `cargo test --no-default-features` | PASS |
| `cargo test --features serde` | PASS |
| `cargo test --features chrono` | PASS |
| `cargo +1.85.0 check --no-default-features` | PASS |
| `cargo +1.85.0 check` | PASS |
| `cargo +1.85.0 check --features serde` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo doc --no-deps --all-features` | PASS |
| `cargo build --manifest-path irig106-time-cli/Cargo.toml` | FAIL (CLI compatibility; exit 101) |
| `cargo clippy --manifest-path irig106-time-cli/Cargo.toml --all-targets -- -D warnings` | FAIL (CLI compatibility; exit 101) |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | PASS |
| `cargo build --target wasm32-unknown-unknown --no-default-features --features serde` | PASS |
| `python3 scripts/build-trace-matrix.py --check` | PASS |
| `cargo deny check` | PASS |

Library tests with all features: 176 unit, 67 pipeline, 17 property,
16 new regression tests, and 10 doctests passed. No-default, serde and chrono
feature runs also pass. MSRV, Clippy, docs, WASM, cargo-deny and trace checks pass.
No benchmarks or sustained fuzz campaigns were run for this bounded slice.

## CLI scope blocker

The requested dependency upgrade removes TimeSource::InternalRtc/Gps and adds
TimeFormat::None. The **untouched** CLI still names the removed source variants
and has an exhaustive format match without None. Both its build and Clippy fail
with E0599 (two variants) and E0004 (one missing arm). These are not test failures
in the rebuilt library. The three compatibility edits needed are:

1. Rename the source2 match to InternalFromRmm (label Internal from RMM).
2. Remove the source Gps arm; source3 is already handled by Reserved.
3. Add a TimeFormat::None match arm. Native GPS FMT remains Gps.

The user's explicit no-CLI-change instruction prevents applying these edits.
No CLI source or lockfile change is committed: the local CLI checks' generated
lockfile was restored after recording their errors. The separate CLI lockfile
must be regenerated with an authorized CLI adoption. No skip or weakening of CI
is introduced. This PR remains draft pending that scope decision.

## Traceability and limitations

Generated marker coverage increases from **0/286** to **24/286 (8.4%)**:
12 L2 and12 L3 entries now have tests. There are zero orphan parents or unknown
markers. This counts linked tests, not complete semantic implementation or line
coverage. In particular ABS-102 calendar reconstruction is not delivered by
its raw-byte fixtures. RDG-007 is tested only for an explicitly declared layout;
the register's full analysis and recording-edition resolver remain pending.
No Design or Review field has changed.

Deferred: network_time (T-2/T-3 and its existing epoch/leap behaviour),
correlation and reference admission, streaming and all Q-DEFAULT/Q-STREAM-*
policies, calendar-origin/BCD-day reconstruction (Q-CH4), NTP conversion choices
(Q-NTP), Q-FLAGS relaxation, CLI commands/outputs/exit policy (Q-CLI-EXIT),
automatic edition/G\106 policy (Q-EDITION), TMATS defaults/comparisons
(Q-TFMT-DEFAULT), and the wider timeline/findings/release/workspace rebuild.
The prototype Chapter 4 to_absolute/calendar path is unchanged; use raw fields
from the corrected parser, not that unresolved conversion. Secondary checksum
RDG-008 remains the prototype word-sum path; the new layout fixture deliberately
uses an independently calculated word checksum and does not claim to fix it.
BCD leap-second support (T-16) and wider calendar changes are outside this slice.
