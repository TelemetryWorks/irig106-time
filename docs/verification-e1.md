# E1 offline core — clause-level verification record

Base main: `3c1cf2e` (PR10). Reviewed E1 head: `3554065`.
This record describes the first offline part, not completion of slice 3.
E2 streaming and the offline follow-up remain outside this PR. Requirement
statements and human Design/Review fields are unchanged.

## Test-first provenance, without overclaiming

Original tests-only commit: `ca146936d6d1b6692358defc58aa667e97eaef3a`.
**Only 2 of the original 13 tests failed first.** The late-default and drift
assertions failed against main; the two worked examples **passed on main as
numeric baselines**. The other **9 tests arrived with the implementation**.
Their assertions verify new behaviour, but were not executed failing on main.
No missing-API compilation failure is described as a behavioural failure.

The contract section 7.2 fixture copies its 36 bytes, including checksum and
Data Length, literally. Its section 7.3 basis assertion now requires
`ITS == ExternalIrig`, not merely a present ITS. RCC123-20 section 5.6,
printed 5-44/PDF78, supplies numeric times/counters rather than a complete
wire packet; its literal test envelope is hand-derived from Chapter11's
layouts. Neither fixture uses this crate's encoder as its oracle.

## Review tests first and controlled mutation evidence

Tests-only review commit: `8cac04a79f875c5a262e591e6c6a66135ed34951`.
Before any production fix, the full offline suite gave **20 passed /1 failed**:
`review_literal_reserved_src_keeps_readable_time` failed because a literal
CSDW `03 30 00 00` (SRC3, readable IRIG-B) was incorrectly excluded.
Its reserved-FMT exclusion assertion is also checked after that fix.

Each mutation below was applied **individually** to `src/timeline.rs` at the
reviewed head, with the tests-only commit present, and then restored before
the next mutation. Command per row:

```
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --test offline_timeline \
  --all-features TEST_NAME -- --exact
```

Every row compiled, executed its named test and returned exit101 with
**1 failed assertion**. These were not compile failures. A source hash check
confirmed the original production file was restored after the experiment;
no mutant is committed. The ordinary test for each row passed without mutation.

| Test name | Mutated/absent behaviour | Observed result |
|---|---|---|
| `review_dynamic_before_time_has_tf002` | Remove `finding(2, ...)` | Fails missing TF-002; also asserts no absolute time |
| `review_session_without_time_has_tf001` | Disable the `first_time_packet.is_none()` finalization branch | Fails missing TF-001 |
| `review_readable_invalid_requires_opt_in` | Replace the Invalid admission arm with unconditional true | Fails default exclusion; tests off and on, TF-003 retained |
| `review_unknown_year_does_not_invent_day366` | Expand unknown-year shift guard from 365 to366 days | Fails because an invented day366 is returned |
| `review_nonleap_day366_is_malformed` | Force `invalid_calendar=false` | Fails no-time/malformed expectation for day366/year2023, even with include-invalid enabled |
| `review_rtc_sync_error_is_tf024_warning_value_kept` | Remove `finding(24, ...)` | Fails missing TF-024; readable time must remain usable |
| `review_duplicate_packet_identity_is_rejected` | Disable the seen-ID rejection branch | Fails exact duplicate-ID error expectation |

After the reserved-SRC fix: **21 offline tests pass**. Reserved SRC is now
TF-006 warning with its value kept and raw source preserved; reserved FMT
remains unreadable and excluded. No TF row or default finding level changes.

## Coverage limits by requirement

Each pair below means the L2 requirement and its L3 child, with the same
numeric suffix; the complete parent chains remain in the requirements.
Tests are in `tests/offline_timeline.rs`, except the named compatibility
checks also updated in `tests/pipeline.rs`.

| L2 / L3 pair | E1 clauses covered and evidence | Deferred / not verified by E1 |
|---|---|---|
| COR-101 | Recording-order packet observations, bounded Format1 decoder input, eligible/rejected metadata, immutable final sessions/indexes; literal fixtures and wire-findings test | Format2/body/context adapters, complete joining-loop integration and report findings fields; no library file walking introduced |
| COR-102 | Same-session/channel nearest or preceding lookup, default no pre-first extrapolation, labelled explicit back-extrapolation; source examples and strategy/tie tests | Full interpolation/drift correction; midpoint preceding is a **proposed**, selectable tie default, pending owner confirmation |
| COR-103 | Offline caller-fixed choice without fallback, preference list, declared-external TIMEIN, valid-reference counts and ID tie; ranking/admission tests | **All stream clauses**: advance candidate knowledge, committed channel before data, counter-only undecidable selection, no future-count ranking/no revision/no switching on lost lock (E2) |
| COR-104 | Signed residual, threshold comparison and finding packet identities for compatible ordinary calendar readings; gap/jump test | Scale normalization/conversions and unresolved year context; coarser mixed-resolution threshold is a **proposed** selectable default, not newly owner-approved |
| COR-105 | High-water unwrapping, short modular wrap, backward-beyond-bound reset, distinct overlapping sessions and caller boundaries; boundary tests | No claim to infer unobservable multiple wraps/long gaps without caller context |
| COR-106 | TIMEIN/payload/readability admission, explicit invalid/undeclared inclusion, malformed exclusion, source None exclusion; literal SRC3 warning/value-kept and reserved-FMT exclusion; new invalid/calendar tests | Full Format2 TS adapters and conversion admission; reserved SRC must not be treated as unreadable format or as GPS |
| COR-107 | File-order source transitions with TF-011, default retention and selectable stop-on-loss; source-loss test | **Stream commitment/no rewriting**, plus preventing interpolation across jumps when interpolation is implemented (offline follow-up/E2) |
| COR-108 | Same-channel valid Format1 spacing, gap >1 s with default zero tolerance, separate from jump threshold; gap test | **Format2 raw-network-rate exception is not implemented here.** No Format2 adapter/exception basis is claimed; E2/full adapter must distinguish this before interpreting its rate |
| COR-109 | Exact1100 ms backward bound, high-water preservation, later-than-bound evidence with inferred reset; boundary/default tests | No automatic recording-edition inference; callers retain explicit boundary/policy responsibility |
| COR-110 | Explicit decoder/caller year-origin metadata is retained, and resolved known-year arithmetic is checked | **Automatic DMY→caller→RI4 year ranking, contradictory evidence, LY revalidation and full session year resolution** (offline follow-up); no G\\OD inference |
| COR-111 | No inverse-query implementation in E1 | **Inverse intervals and partitioning at gaps/jumps/year/scale changes** (offline follow-up) |
| COR-112 | First time packet tracked independently of eligibility, static exemption, dynamic-before-time TF-002, no-time-packet TF-001; new positive-finding tests and existing None-source test | Full file/decoder joining loop supplies trusted packet kind; E1 does not read a recording itself |
| COR-113 | Same-session/channel counts, interval/spacing/density and signed drift; duplicate/short/scale/context unavailable reasons; quality tests and compatibility drift regression | Rich report rendering, normalization/conversion/year-evidence resolution before cross-context measures; not a general uncertainty estimate |
| ANS-101 | Session/channel reason, reference packet/counter/time, signed distance, position, source/FMT/**exact ITS**, scale/origin, precision, raw wire/layout, supplied year origin, no-drift flag, full **implemented** policy value; example basis tests | **Stream channel commitment, hold/gap/caps/progress/fallback; interpolation endpoints/rounding; NTP era/raw fraction/sentinel/leap/table-currency basis; serialization; complete conversion corrections** (E2/later). E1 is not the full answer schema |
| ANS-105 | Counter-only result relative to session first packet, default no absolute before first reference, explicit offline back-extrapolation and unknown-year boundary guard; new guard test | Both stream-mode pre-first rules (E2), full year/scale/conversion context resolution |
| POL-101 | Offline policy carries implemented session/admission/ranking/reference/source-loss/gap/jump/late/severity settings and labels its defaults; snapshot test | **Year-ranking/complete boundary-validity combinations, secondary-time tolerance, centralized scale policy, stream mode/hold/caps/progress**, and full cross-layer policy. Decoder scale policy is separate and effective scale evidence is retained; not every contract5.6 setting is centralized |
| POL-102 | Version/late-bound/finding-ID validation, frozen builder policy and by-value answer snapshot, invalid identity/query errors; new duplicate-ID test | **Stable policy-ID registry, every setting/combination test, runtime updates, stream-mode transition drain/flush**. E1's version is a schema version, not a unique policy ID; drain/flush remains proposed for E2 |
| POL-103 | Complete implemented offline value copied in the Rust answer | **Versioned serialization/default-delta identity/table and complete cross-layer snapshot schema** (later); no serialization verification claimed |
| FND-103 | Bit5 produces TF-024 WARNING without discarding usable time; new literal flag test | Full report/CLI exposure of these findings |
| TAI-107 | Bounded wire reading retains second60 and emits TF-022 INFORMATION without normalization; core projection explicitly needs context | Leap arithmetic, NTP ambiguity windows, leap-table conversions and broader cross-scale/calendar reconstruction are not implemented by E1 |

## Source None and TF-003: explicit policy classification

106-24R1 Chapter11 §11.2.3.2, printed **11-24 / PDF32**, says verbatim:

> If the time data packet source is None, at least one time data packet is required IAW the previous note.

The SRC table on printed **11-25 / PDF33** says **“0xF = None”**.
The FMT table separately says **“0xF = NONE (time packet payload invalid)”**.
These fields are **not aliases**. The standard assigns neither this library's
TF identifiers nor an equivalence between SRC None and invalid FMT/TS.

Owner Q2 requires no absolute time from source None. E1 retains such a
packet as **present** (so it must not produce TF-001 merely for lacking a
usable reference), retains raw SRC None and `Rejection::NoneSource`, and
classifies the **unusable reference** under the existing TF-003 WARNING.
This is a documented **E1 admission-policy reuse** of the invalid-time warning,
not a claim the payload is malformed, FMT NONE, or TS invalid. TF-004 would
misdescribe a well-formed packet; TF-001 would contradict its presence.
Consumers can distinguish the source-None reason from payload-invalid reasons.
`channel_rank_admission_none_and_caller_no_fallback` asserts the TF-003 warning;
`first_time_presence_scale_boundaries_and_invalid_queries` verifies presence
without false TF-001/TF-002. **No TF row is changed.**

## Matrix interpretation and executed checks

The trace generator cannot express **partial clause coverage within a single
L2/L3 requirement**. It finds at least one test marker and can label a row
Implemented even while clauses in the table above remain deferred. Its
Partially Implemented rollup concerns children, not clause completion.
Therefore its status/percentage is an **artifact-link count**, not proof of
complete E1, COR, ANS or POL compliance. This limit is also in CHANGELOG.
Do not promote human Design/Review states from these marker counts.

This review adds mutation-tested behaviour checks, exact ITS and reserved-SRC
admission evidence. The matrix is regenerated and freshness checked. Full
local CI equivalent uses CARGO_BUILD_JOBS=2/RUST_TEST_THREADS=2, including
feature tests, MSRV1.85, clippy, fmt, rustdoc, CLI, WASM and cargo-deny; actual
head/CI results are reported in the PR. Source fixtures remain synthetic;
no real recordings, host clock, dependency installation or settings changes.
