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
fix. The later reviewer correction below replaces the erroneous reserved-word
rejection assertion; the other original value expectations remain unchanged.

| Finding | Before, executed against main | After |
|---------|-------------------------------|-------|
| T-1 | 0xc0 chose RTC, not Chapter 4; 0x04 chose IEEE, not RTC; exhaustive flags test failed | Correct bits 6/7/3–2; all 256 flags checked; strict L1-IPT-007 retained |
| T-5 (shared adoption) | ERTC raw1 returned100 ns | Returns1 ns from types 48f1834 |
| T-6 | SRC3 was Gps; SRC2 InternalRtc | Reserved(3), InternalFromRmm; explicit legacy EXT, no inference |
| T-7 | FMT15 was Reserved(15); explicit ITS/layout APIs absent (compile failure) | Modern None; legacy FMT15 reserved; all ITS/FMT combinations and per-layout masks pass |
| T-9 | setup15 was V23; new code-list API absent (compile failure) | Unknown(15), zero unknown, separate shared setup/header lists over all256 byte values |
| T-10 (shared adoption) | Figure11-4 fields decoded (9999,4660,1) rather than (1,4660,9999) | reserved/usec/low/high; top binary low bit retained; short/reserved/usec cases checked |
| T-12 | six-byte day body rejected (expected8); eight-byte DMY rejected (expected10) | Exact6/8-byte reads/encoding; all truncations and nonzero following filler checked |

After the initial fixes, `cargo test --test rebuild_wire --test rebuild_layout`
passed **11 + 5 tests**. The current reviewer-corrected suite passes **15 + 5**. The old unit/integration assertions encoding incorrect GPS,
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
| `cargo test --manifest-path irig106-time-cli/Cargo.toml` | PASS |
| `cargo fmt --manifest-path irig106-time-cli/Cargo.toml -- --check` | PASS |
| `cargo build --manifest-path irig106-time-cli/Cargo.toml` | PASS |
| `cargo clippy --manifest-path irig106-time-cli/Cargo.toml --all-targets -- -D warnings` | PASS |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | PASS |
| `cargo build --target wasm32-unknown-unknown --no-default-features --features serde` | PASS |
| `python3 scripts/build-trace-matrix.py --check` | PASS |
| `cargo deny check` | PASS |

Library tests with all features: 176 unit, 67 pipeline, 17 property,
20 rebuild regression tests, and 10 doctests passed. Both CLI tests pass. No-default, serde and chrono
feature runs also pass. MSRV, Clippy, docs, WASM, cargo-deny and trace checks pass.
No benchmarks or sustained fuzz campaigns were run for this bounded slice.

## Approved CLI compatibility adoption

At the earlier draft head 47974d0, CLI build and Clippy failed with two E0599
errors (removed InternalRtc/Gps source variants) and E0004 (missing FMT None).
The owner then explicitly authorized only the three compatibility edits and
CLI lockfile regeneration. Those edits are now applied:

| Location | Exact source change | User-visible difference versus the old types/CLI |
|----------|---------------------|-------------------------------------------------|
| src/main.rs line 450 | `TimeSource::InternalRtc => "Internal RTC"` replaced by `TimeSource::InternalFromRmm => "Internal from RMM"` | Source 2 displays Internal from RMM instead of Internal RTC |
| old src/main.rs line 451 | Removed `TimeSource::Gps => "GPS"` | Source 3 now uses the existing Reserved arm and displays Reserved instead of GPS |
| new src/main.rs line 464 | Added `TimeFormat::None => "None"` | Modern FMT 0xF displays None instead of Reserved; native GPS FMT 5 still displays GPS |

The paths above are relative to irig106-time-cli. Its Cargo.lock gained only
the source identity for irig106-types at 48f1834; package versions did not change.
At that compatibility commit, no other CLI behaviour changed. The later
reviewer-authorized T-12 length fix is recorded below. All **18** commands in
the current table pass after the review fixes with CARGO_BUILD_JOBS=2,
including the additional CLI tests and CLI format check. The earlier failures
are retained here as history, not current blockers.

## Reviewer corrections after b1fb219

Tests-only commit `5067b94` reproduces five failures before the corrections:
`parse_rtc48_reserved_ignored`, the secondary Chapter 4 fixture, the new
intra-packet reserved-word fixture, and both CLI consumer tests. The library
failures are `ReservedBitSet`; the CLI reports DOY/DMY buffer-too-short.
These are executed assertion failures, not inferred outcomes.

RTC stamps now decode only bytes [0..6) even with nonzero [6..8). Chapter 4
secondary and intra-packet readers likewise retain values when their internal
reserved word is set. The independent `validate_rtc48_reserved`,
`validate_ch4_reserved` and `validate_secondary_reserved` APIs report TF-005
warnings without discarding the parse result. The two Chapter 4 reserved fields
have separate positions and can both be reported. Tests walk every reserved bit,
check truncated validator inputs, and confirm unchanged time values.
Microseconds 10000 still produce `OutOfRange` on both paths, with or without a
reserved-word warning (TF-004). No severity policy or owner question is changed.

CLI `src/main.rs:256` changes `bcd_start + 8` to `bcd_start + 6` for DOY;
line 275 changes `bcd_start + 10` to `bcd_start + 8` for calendar BCD. Tests pass
exact consumer buffers (no filler) and every truncated body directly through
`process_time_packet`. Previously these exact bodies produced a buffer-too-short
error and no reference/channel; now they yield one reference/channel with the
specified time. Output formatting and exit policy are unchanged. This does not
claim to validate packet framing or implement a wider CLI Data Length policy.

All 33 added/changed `Traces` tags in the PR were checked against current L2/L3
IDs and their L1 parents, with no duplicates or bad chains. This includes the
DayFormatTime conversion tag, which incorrectly referenced calendar BCD. The
secondary checksum documentation explicitly identifies its deferred RDG-008
limitation instead of claiming the byte-sum requirement is complete. Fixture
comments mark the word-sum checksums for replacement when RDG-008 lands.
`Ch4BinaryTimeExt::to_absolute` has a linked Q-CH4 warning about its incorrect
17-bit split; the conversion itself remains deferred. CHANGELOG Unreleased
records migration notes for the public version/CSDW/flag/BCD/type changes.

## Traceability and limitations

Generated marker coverage increases from **0/286** to **30/286 (10.5%)**:
15 L2 and15 L3 entries now have tests. There are zero orphan parents or unknown
markers. This counts linked tests, not complete semantic implementation or line
coverage. In particular ABS-102 calendar reconstruction is not delivered by
its raw-byte fixtures. RDG-007 is tested only for an explicitly declared layout;
the register's full analysis and recording-edition resolver remain pending.
No Design or Review field has changed.

Deferred: network_time (T-2/T-3 and its existing epoch/leap behaviour),
correlation and reference admission, streaming and all Q-DEFAULT/Q-STREAM-*
policies, calendar-origin/BCD-day reconstruction (Q-CH4), NTP conversion choices
(Q-NTP), Q-FLAGS relaxation, CLI commands/exit policy (Q-CLI-EXIT) beyond the authorized compatibility and
BCD length fixes,
automatic edition/G\106 policy (Q-EDITION), TMATS defaults/comparisons
(Q-TFMT-DEFAULT), and the wider timeline/findings/release/workspace rebuild.
The prototype Chapter 4 to_absolute/calendar path is unchanged; use raw fields
from the corrected parser, not that unresolved conversion. Secondary checksum
RDG-008 remains the prototype word-sum path; the new layout fixture deliberately
uses an independently calculated word checksum and does not claim to fix it.
BCD leap-second support (T-16) and wider calendar changes are outside this slice.
