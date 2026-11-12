# Rebuild slice 2 — wire readings, scale labels and checksum evidence

Base: main `30898a7e4467582fb34e2f86e723920f53fe51d5` (PR #6 merged).
Tests-only first commit: `afa411b` (two integration test files only).
No agents, real recordings or unrelated repositories were used. Source PDFs
were fetched/verified through the authorized rcc-106-standards archive.

## Page inspection

These pages were rendered with `pdftoppm` and inspected visually; this is not
an inference from extracted text. Printed page / one-based PDF page:

| Source | Pages | Checked result |
|---|---|---|
| 106-24R1 Chapter 11 | 11-27 / 35, Figures 11-15..17 | NTF bits 7–4, TS 3–0, reserved 31–8; NTF 0=NTPv3, 1=1588-2002, 2=1588-2008; TS 0 not valid, 1 valid; PTP seconds/nanos are each 32 bits |
| 106-17 Chapter 11 | 11-6 / 14, Table 11-4; 11-24 / 32, Figures 11-14..16 | Type 0x12 Network Time exists with current data type version 0x08; same NTF/TS and two-word bodies |
| 106-15 Chapter 10 | 10-27 / 39, Table 10-10 | 0x12–0x17 are reserved, not Network Time |
| 106-24R1 Chapter 11 | 11-25 / 33, Figures 11-13/14 and FMT list; 11-26 / 34, §11.2.3.3 | Three/four BCD words; second-tens is three bits; FMT4 UTC and FMT5 native GPS; NTP UTC/1900, PTP TAI/1970 |
| 106-24R1 Chapter 11 | 11-10 / 18, §11.2.1.2c | Secondary checksum says “a 16-bit arithmetic sum of all secondary header bytes” |
| RCC 123-20 | B-31 / 137 | `uCalcSecHeaderChecksum` iterates ten `unsigned char` bytes, unlike the primary header word-sum function on the same page |
| 106-24R1 Chapter 11 | 11-56 / 64, §11.2.7.3 / Figure 11-38 | Event Number is a setup-defined 12-bit number (4096 types), not a fixed started/stopped enum |
| RCC 200-16 | 1-1 / 13; 3-2 / 18, §3.6; A-1 / 56, A.2 | Range UTC practice; leap-day exception; “time changes are made on December 31 and on June 30 at 2400 hours”; GPS/TAI relationship |

T-4's earlier version-helper error was already fixed in PR #6: main supports
V17/V19/V22. This PR confirms that on pages and adds an explicit tri-state
availability report; it does not claim that existing helper failed on this main.
The header version is not used to infer recording edition (Q-EDITION).

## Executed before/after checks

Before changing implementation, on main plus the tests-only changes:

- `cargo test --test slice2_wire`: **0 passed, 9 failed**, executed assertions.
- `cargo test --test slice2_readings`: **compile failure**, missing reading,
  scale, status, availability and checksum-inspection APIs. These eight tests
  did not execute on main; compilation failures are not assertion failures.

After implementation both files pass, **9 + 8** tests. Boundary assertions were
expanded in those same cases after the first commit. Other existing tests were
adapted where they encoded the wrong wire format or assumed an NTP conversion.
Round trips remain supplements to the independent hand-packed fixtures.

| Finding | Before on main | After |
|---|---|---|
| T-2 | 0x01 -> PTP (status mistaken for format); 0x81 -> PTP; 0x11/21 flagged reserved | Independent NTF/TS; PTP revisions distinct; all 256 low bytes, all reserved bits, invalid/reserved TS preserved in raw readings |
| T-3 | Exact 8-byte PTP rejected; trailing filler entered nanos; encoder emitted 10 bytes | Two u32 LE words, 8-byte encoder, max seconds/nanos and truncations; no filler reads |
| T-4 | Existing supports_format_2 already correct; tri-state reporting API absent (compile failure) | Page-confirmed 106-17 availability; declared pre-17/unresolved editions remain distinct, no header inference |
| T-12-style bounds | No Data Length argument on network payload parser | Bounded new F1/F2 readers, consumed 10/12 or 12 bytes; every smaller length and nonzero following filler checked |
| Q-NTP boundary | Era-zero Unix/calendar conversion silently returned a value | Unix/absolute-NTP-epoch/calendar methods explicitly return NeedsContext; modular numeric/fraction views remain available |
| T-17 / RDG-009 | No scale/provenance reading API (compile failure) | All FMTs and NTFs checked; native GPS stays GPS; IRIG UTC is policy-labelled; RTC unknown/overridden keeps original unknown evidence |
| T-16 / RDG-010 | No leap-preserving reading API (compile failure); prototype second 60 rejected | Valid UTC candidate 23:59:60 retained as 60, known/unknown-year boundaries checked; GPS/TAI/RTC/unknown and wrong date/time rejected |
| RDG-008 | Byte-only header rejected, word-only accepted | Stored/byte/word sums retained; both/byte-only/word-only/neither tested; trusted own time requires bytes; updated T-10 fixtures |
| T-8 | Unsupported fixed event meanings exposed without deprecation | Types deprecated with ADR-0009/decode pointers, public API retained; inspection rather than a fabricated runtime regression |

## RDG-008

The standard's byte wording, its contrasting primary-header word wording, and
the RCC 123-20 code agree on bytes. This remains the owner's **suspect** reading;
no real-recording/vendor confirmation is claimed. `inspect_secondary_checksum`
returns all three sums and independent byte/word validity, including word-only
(TF-027 plus failed own-time TF-012). Validation/parsing require the byte reading;
there is no automatic fallback to words.

Hand arithmetic: bytes `01 02 00 ... 00` sum to 3; their LE word sum is 0x0201.
Stored 3 is byte-only, stored 0x0201 word-only, stored 4 neither. `01 00 ... 00`
with stored 1 satisfies both. All ten 0xFF bytes sum to 2550; five 0xFFFF words
sum modulo 65536 to 65531. The tests also check truncations and trailer exclusion.

Writing E=sum(even-index bytes), O=sum(odd-index bytes), the two readings differ
by 255*O modulo 65536. Since gcd(255,65536)=1 and 0<=O<=1275, they coincide only
when O=0. A recording with only such headers cannot settle the reading from its
checksums alone. Recording-wide aggregation and local vendor evidence remain
future work; this slice supplies per-header evidence without CLI changes.

## RDG-009

FMT4/NTP UTC, FMT5 GPS and PTP TAI are standard-stated labels. IRIG-A/B/G UTC is
an explicitly recorded default policy assumption based on range practice, not
standard-mandated; callers can change it. RTC defaults unknown. Policy overrides
retain the original unknown designation. No source nibble, host clock or era is
used as scale evidence. Native GPS is never projected to a UTC-like calendar
without conversion. TAI minus GPS = 35−16=19 seconds is a **derived** relationship
from RCC 200-16 A.2, recorded as a constant, not new standard wording or a
historical UTC offset conversion.

`CalendarReading` and `Scaled` retain scale evidence; NTP/PTP raw values expose
their stated scale. `TimeScaleTagged` marks unqualified legacy BCD/calendar,
secondary and counter values Unknown. Those old values have no FMT/policy
context, so assigning them UTC would be false. No correlation/reference/stream
scale integration is claimed: their source is unchanged. New qualified results
must not have their wrappers discarded and be treated as normalized answers.
NTP era/leap conversion stays unresolved even though its raw scale label is UTC.

## RDG-010

The three second-tens bits encode 60, but this alone does not authorize second
60 at arbitrary times. The accepted reading restricts it to UTC 23:59:60 on
June 30 or December 31 candidates. Without year these are DOY181/182/365/366;
with year only June30 and December31 for that Gregorian year are allowed.
Assumed-UTC IRIG is labelled as such. GPS/TAI/unknown scale and RTC format reject
second 60. Eligibility is not proof a leap was historically scheduled that day.

`CalendarReading` stores second 60 and subsecond fields directly, so it cannot
collapse to next-day midnight through a nanosecond-of-year representation.
Its public/deserialized fields are validated before projection; serde derives
retain the explicit fields and scale. The legacy `AbsoluteTime` representation
cannot carry the label; projection explicitly refuses it. No leap arithmetic,
leap-table redesign, chrono normalization or correlation change is delivered.
Legacy BCD body-only APIs remain strict because they lack scale context: use
the new explicit-layout/policy Format 1 decoder for leap support.

## Executed local CI equivalent

Stable Rust 1.99.1, MSRV 1.85.0; every Cargo command used CARGO_BUILD_JOBS=2.
All **18** commands passed (the workflow's commands plus CLI tests/format):

| Command | Result |
|---|---|
| `cargo test --all-features` | PASS |
| `cargo test --no-default-features` | PASS |
| `cargo test --features serde` | PASS |
| `cargo test --features chrono` | PASS |
| `cargo +1.85.0 check --no-default-features` | PASS |
| `cargo +1.85.0 check` | PASS |
| `cargo +1.85.0 check --features serde` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo doc --no-deps --all-features` with RUSTDOCFLAGS=-D warnings | PASS |
| `cargo test --manifest-path irig106-time-cli/Cargo.toml` | PASS |
| `cargo fmt --manifest-path irig106-time-cli/Cargo.toml -- --check` | PASS |
| `cargo build --manifest-path irig106-time-cli/Cargo.toml` | PASS |
| `cargo clippy --manifest-path irig106-time-cli/Cargo.toml --all-targets -- -D warnings` | PASS |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | PASS |
| `cargo build --target wasm32-unknown-unknown --no-default-features --features serde` | PASS |
| `python3 scripts/build-trace-matrix.py --check` | PASS |
| `cargo deny check` | PASS |

Feature-test execution, compilation, page inspection and the deprecation review
above are distinct evidence. No benchmarks, sustained fuzzing, real recordings,
or external credential/settings changes were run.

## Scope and traceability

Marker coverage: **30/286 (10.5%) -> 58/286 (20.3%)**, +28 countable links
(+14 L2, +14 L3). Zero orphan parents/unknown markers. Marker coverage is not
complete requirement semantics or line coverage. All 41 added/changed source
`Traces` tags were checked against current IDs and parent chains. Written analyses are
limited to this slice; human Design/Review fields remain unchanged.

Untouched source: correlation, streaming, CLI, Chapter4 calendar reconstruction.
Deferred: all Q-DEFAULT/Q-STREAM-* choices, Q-CH4, Q-EDITION inference,
Q-TFMT-DEFAULT, exit policy, NTP era/leap conversion Q-NTP, complete timeline
findings/aggregation, reference scale normalization, leap arithmetic, leap-table
coverage/currency improvements and actual event migration in irig106-decode.
PTP's explicit-offset legacy conversion/table code is unchanged; its broader
transition handling is not claimed fixed here. CLI output differences forced
by the library are disclosed in CHANGELOG: corrected NTF/status/PTP parsing and
explicit NTP conversion refusal; no CLI source or output formatting changed.
