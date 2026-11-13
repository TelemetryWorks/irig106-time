# E2 bounded stream verification

Base: main `ce78185` (merged E1); split recorded before code in working notes.
One E2 core PR, about1,500 handwritten changed lines plus generated matrix.
No human Design/Review fields or requirement statements changed.

## Executed evidence and honest provenance

Tests-only commit `5354f9a`: the existing-API TF-024 evidence test executed on
main and failed (missing raw counter). The **nine initial stream tests could
not execute on main**: missing new API, a compilation failure, not nine failing
behavioural reproductions. They first passed after implementation. Two further
tests and further assertions arrived during implementation. Their exact-counter late-reference case
failed before its fix; the equal-counter changed-time case also failed before
jump detection was fixed. No numeric baseline is presented as a main failure.

`handbook_standard_and_contract_basis` uses RCC123-20 §5.6 printed5-44/PDF78
numbers; its envelope is hand-derived from Ch11 §11.2.1.1/Figures11-12/13,
not purported RCC packet bytes. Contract7.2's36 bytes are literal. Standard
mode reproduces15 ms and420 ms displacements and exact ExternalIrig ITS.
Synthetic interpolation independently computes1.000010/2=.500005 s for1 s
RTC, correction+5000 ns;302/3 ns tests floor rather than nearest rounding.
Endpoint precision remains10 ms, not the arithmetic result's resolution.

Final targeted result: **11 stream +22 offline tests pass**. All256 flag values
verify TF-024 presence/absence and raw RTC123, independently of timestamp decode.
A1000-data burst stays within2 packets/200 declared bytes/2 references.

## Controlled negative checks

For each mutation, execute only its named test with
`CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test --test stream_timeline --all-features TEST -- --exact`.
Each compiled, executed and failed (exit101); all26 mutants were restored,
with source hashes checked. No mutant is committed. To reproduce, suppress
one finding ID at `emit`/shared `finding`, or one reason at `flush`, per row.

| Mutation group | Test(s) that killed each individual mutant |
|---|---|
| Omit TF-001,002 | `early_data_none_and_no_time_findings` |
| Omit TF-003,004,006,008,024 | `admission_findings_and_raw_sync_counter` |
| Omit TF-005,022 | `reserved_leap_and_stop_on_loss_evidence` |
| Omit TF-011 | `commitment_inventory_loss_and_no_revision` |
| Omit TF-016 | `hold_gap_and_explicit_progress_boundaries` |
| Omit TF-017,018,019 | `discontinuities_and_scale_never_bridge` |
| Suppress PacketCap,ByteCap,ReferenceCap,Eof,ModeChange,SessionBoundary flush, individually | `every_resource_and_lifecycle_fallback_is_labelled` |
| Suppress Discontinuity,IncompatibleScale,UnusableReference flush, individually | `discontinuities_and_scale_never_bridge` |
| Disable timeout comparison `progress>d` | `hold_gap_and_explicit_progress_boundaries` |
| Disable exact-counter release | `late_endpoints_policy_transitions_and_oversized_data` |
| Change left-neighbour comparison `<=` to `<` (miss same-counter jump) | `discontinuities_and_scale_never_bridge` |

## Clause-level scope (each pair means L2 and L3)

| Requirement pair | Covered E2 clauses / evidence | Remaining limits |
|---|---|---|
| STRM-101 | Explicit progress/EOF, bounded neighbours/pending, incremental findings, no future-count ranking/no revisions; burst/configuration/commitment tests | Caller supplies decoded observations, complete inventory and strictly increasing identities; no file/network reader |
| STRM-201, POL-201 | Default Standard, immediate preceding same-channel reference; RCC/contract, future-only counter-only tests | Prototype streaming remains separate; old policy snapshots are not retroactively upgraded |
| STRM-202 | Same-session/channel bracket, checked i128, drift correction, both endpoints, exact/late endpoints, coarser precision, incompatible-context barrier | **Floor rounding remains a L3 proposal**; ordinary calendars only; leap-label arithmetic/network normalization deferred |
| STRM-203, COR-108 | Inclusive enqueue-progress+1 s+100 ms deadline, strictly-later timeout, stalled progress, EOF; gap>1 s still TF-016 inside1.1 s hold | Native RTC granularity is100 ns, so1 s+100 ns tests the first representable violation; Format2 adapters not implemented. Caller `raw_network_rate` labels policy-only gaps, not a1 Hz standard breach |
| STRM-204, ANS-105 | Immediate counter-only before usable preceding reference in both modes; TF-002 before first time packet, separate presence/eligibility, source None never reference | Format2 invalid-TS decoder adapter remains deferred; shared invalid input classification is tested. Source None's TF-003 policy reuse is explained in verification-e1.md |
| STRM-205 | Mandatory packet/declared-byte/reference caps, cap/timeout/EOF fallback without drops, oversized input, barriers at jump/reset/scale/unusable reference | Byte charges are supplied by caller; fixed metadata is additionally bounded by count caps. No real recording/load benchmark or sustained fuzz campaign claimed |
| STRM-206, POL-102 | Fresh increasing caller policy ID, full immutable snapshots, old-policy fallback before explicit replacement, fixed channel; transition/configuration tests | **Old-policy drain remains a labelled L3 proposal**. Changing admission/session settings clears retained references; new evidence required. Serialization/registry outside process deferred |
| COR-103 | Complete inventory: caller, external TIMEIN, lowest ID; incomplete inventory counter-only until explicit choice; no valid-count ranking | No TMATS parser integration; caller supplies declaration facts and inventory completeness |
| COR-107 | TF-011 transitions, fixed channel despite competing references/lost lock, stop-on-loss withholds; jumps flush before adding new reference | Cross-scale conversion is not inferred; reference context stays labelled |
| ANS-101 | Committed channel/reason, endpoints with full E1 wire/year/scale evidence, drift correction/rounding/precision, policy ID+value, progress/deadline, release reason and extrapolated flag | NTP era/table evidence, complete own-time/conversion corrections and serialization still deferred as in E1 |
| POL-101 | Separate mode/hold/gap/late/caps, inherited admission/source-loss/finding settings, raw-network-rate label | Centralized scale/year/secondary tolerance policy remains E1 follow-up; this is not the complete contract5.6 policy |
| FND-103 | Raw RTC on E1 and E2 findings, even with no timestamp; all256 flags | Full report/CLI rendering remains deferred |

Outputs are released by observation/release order, not promised globally sorted
by counter. Caps may evict older reference neighbours; missing retained context
returns labelled counter-only rather than fabricating a reference. No hidden
output/history/seen-ID queue is retained. Channel declaration inventory is
consumed only at construction; explicit session boundaries preserve the chosen
channel unless a separate new stream is constructed. No host clock is read.

The trace matrix only counts artifact markers; it cannot express partially
covered clauses within one linked requirement. Its Implemented rows and coverage
percentage are **not full compliance**. Full local CI equivalent uses two build
jobs, MSRV1.85/no_std+alloc, feature suites, CLI, WASM, clippy/docs/fmt, trace
freshness and cargo-deny; actual head and hosted CI results are reported in PR.
