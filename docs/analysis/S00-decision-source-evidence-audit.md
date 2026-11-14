# S00 — decision/source and evidence audit

> **Later decision, owner 2026-11-13:** plan Q1–Q6 and C1–C4 are accepted
> with TF-031/032 levels pending S03/S08 owner confirmation. The original
> evidence and then-pending labels below are historical, not new results.
> See [the decision record](../analysis/plan-decisions-2026-11-13.md); release deferrals do not imply coverage.
> **Subsequent owner decision 2026-11-14:** TF-031 WARNING accepted (body kept,
> EXT/SRC qualification withheld); only TF-032 remains level pending (S08).
> The 2026-11-13 holds above are historical; see the owner-decision audit.

2026-11-13. Baseline: main `2b996243ec14dd2b5a795902df804e0a79f78692`
(E2 PR #12 merged). This is the coordinator-authorised **S00 only**, from
`coordination-work/irig106-time-0.8.0-plan.md` saved by the coordinator.
That local coordination file is not part of this checkout. This audit does
not approve that plan's other slices or release deferrals.

## Boundary and evidence vocabulary

- Documentation only: no Rust, tests, manifests, dependency, CI configuration,
  credentials or policy settings change. No new findings or conversions are
  implemented. No merge, publication or tag is authorised.
- Historical runtime results below are attributed to their records/PRs and
  baselines. They were **not re-executed by S00**. A source inspection is not
  a runtime result. A missing-API compilation failure is not an executed
  failing regression. A later mutation is not proof a test failed before the
  original implementation.
- Existing page checks are reused with their recorded pages and limits;
  **no page image was newly rendered/inspected in S00**. This audit cannot
  replace the register's all-edition analysis or two-person human review.
- No **Review** field or owner-set **Design** field is changed. Pending plan
  Q1–Q6, Q3 deferrals and C1–C4 remain pending; see the hold table below.

## Confirmed source/decision corrections

### PTP words: correct the stale covered-table-only obligation

Trace: **L1-WORD-002 → L2-WORD-102 → L3-WORD-102**. The old L3 sentence
required a covered table for UTC, contradicting its own planned verification,
L2/L3-PTP-104, L2/L3-TAI-104, RDG-012 and the accepted owner correction.

The already cited standard says verbatim:

> The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds.

106-24R1 Chapter 11 §11.2.3.3, printed **11-26 / PDF 34**; existing page
inspection is recorded in [slice 2](../verification-slice-two.md#page-inspection).
This establishes the scale/epoch, not a library policy for stale tables.
**Owner 2026-11-13** confirms contract 6.8/6.9: after table coverage ends,
use the last known offset, retain raw TAI, label UTC possibly out of date
and raise TF-021 (warning). If no known applicable offset exists, retain
TAI with missing context rather than inventing an offset, as L3-TAI-101/104
already require. This does not reintroduce the withdrawn no-UTC-after-coverage
rule. No pending plan question is needed for this correction.

The correction changes L3-WORD-102 and appends the existing accepted PTP
source to both WORD-102 Source lines. All earlier source quotes, full versus
component-only word distinctions and NTP era rules remain. Conversion code
and its tests remain future work; a source correction is not that work.

### Finding catalog: include only the already accepted TF-028 addition

Traces: **L1-FND-001 → L2-FND-101 → L3-FND-101**, and
**L1-FND-002 → L2-FND-102 → L3-FND-102**. L2-FND-101 stopped at TF-027;
the related L3-FND-102 dispatch list also omitted the accepted TF-028.
The catalog and list now include that existing decision only.

Existing flag sources, verbatim (106-24R1 §11.2.1.1 g, printed
**11-8/11-9 / PDF 16/17**, recorded in the
[Q-FLAGS page/source analysis](Q-FLAGS-2026-11-12.md)):

- "Bits 3-2: Indicate the packet secondary header time format."
- "11 = Reserved"
- "1 = Packet secondary header time (bit 7 must be 1)."

Those words define flags, **not finding identifiers or severity**.
**Owner 2026-11-13, Q-FLAGS-ERROR** already accepted TF-028
"Secondary-derived time has an undefined format" at ERROR, architecture §8.
The finding table and TF-001..TF-027 stay byte-for-byte unchanged. This is
not approval of plan Q1's additional finding-ID/mapping requests, nor a new
runtime emitter. No L1 statement needs alteration: its stable-findings and
network-word obligations already allow these accepted refinements.

## T-1 through T-17 inventory

All original finding text remains in [STANDARD-REVIEW](../STANDARD-REVIEW.md).
Its prototype code references and dated page notes are historical, not a
live module-completion report. The table below separates their current
bounded scope from still-missing clauses. Test names identify inspectable
source; pass/fail claims require the chronology following this table.

| Finding | Classification and source | Recorded evidence / representative test | Scope present at baseline; remaining limit |
|---|---|---|---|
| T-1 — wrong flag bits | Code; Ch11 §11.2.1.1 g / §11.2.1.3 b | W1 runtime failures; later Q-FLAGS runtime failures. `tests/rebuild_wire.rs::t1_all_flag_combinations_are_checked`, `qflags_literal_rtc_reserved_format_keeps_value`, `qflags_literal_error_variants`. | Dispatch is bits 6/7/3–2, with independent RTC padding/reserved-value/own-secondary diagnostics. Later all-256 diagnostic oracle/exact assertions passed the pre-change implementation; not extra failing-baseline claims. Unified stamp-answer/finding attachment remains later work. |
| T-2 — NTF/TS CSDW bits | Code; Ch11 Fig11-15 / §11.2.3.3 | W2 runtime failures; `tests/slice2_wire.rs::t2_ntf_is_high_nibble_not_valid_status`; `slice2_readings.rs::t2_each_ntf_and_status_remain_independent` was in the compile-first new-API suite. | NTF7–4, TS3–0 and reserved31–8 raw reading is corrected. Format2 recording admission and attached TF-003/005/006 evidence remain incomplete. |
| T-3 — PTP body length/order | Code; Ch11 Fig11-17 (106-17 Fig11-16) | W2 runtime failures; `tests/slice2_wire.rs::t3_ptp_has_two_u32_words`, `t3_ptp_never_reads_following_filler`. | Correct bounded8-byte seconds/nanos body and encoder; TAI→UTC table/currency conversion is separate pending work. |
| T-4 — Format2 edition | Code/documentation; 106-17 Table11-4, Ch11 body figures; 106-15 Table10-10 | Version helper corrected in PR #6. W2 explicitly reports it already correct on its baseline; `tests/slice2_readings.rs::t4_explicit_availability_never_infers_unknown_edition` could not execute until the new API existed. | Availability from106-17 is page-confirmed. No dedicated executed T-4 prototype-failure record is established by those later passing/compile-first cases. Recording edition resolution/TF-026 attachment remain pending. |
| T-5 — ERTC100× scale | Shared code/adoption; Ch11 §11.2.1.1 g | W1 runtime failure; `tests/rebuild_wire.rs::t5_shared_ertc_tick_is_one_nanosecond`. | Correct1 ns/tick adopted from types48f1834. Relative ERTC/session correlation and same-packet RTC/ERTC finding integration are separate pending clauses. Shared package publication is not established by adopting the git revision. |
| T-6 — EXT/SRC and SRC3 | Code plus edition reading; 106-04 Fig10-15, 106-05 Fig10-16a, Ch11 §11.2.3.2 | W1 SRC3/source2 runtime failures; explicit EXT layout suite compile-first. `t6_source_three_is_reserved`, `t6_source_two_is_rmm`, `tests/rebuild_layout.rs::t6_legacy_ext_is_not_src`. E1 review later exposed reserved-SRC admission failure. | Shared names/raw EXT versus SRC corrected; reserved SRC remains a warning and does not discard readable time. Effective-edition resolver/unknown alternatives remain pending. Header0x01 alone is not proof of EXT. |
| T-7 — FMT NONE/ITS/masks | Code and edition-qualified fields; Ch11 Fig11-12 / §11.2.3.2 | W1 FMT15 runtime failure; `t7_none_format_is_distinct`. ITS/layout tests initially compile-failed: `tests/rebuild_layout.rs::t7_its_requires_explicit_applicable_layout`, `t7_reserved_masks_are_layout_specific`. | Explicit-layout FMT/ITS/masks exist. Unknown-edition qualification and the RDG-007 analysis/resolver remain incomplete; all of ITS was not shown failing at runtime before the original fix. |
| T-8 — fixed event meanings | Code ownership/migration; Ch11 §11.2.7.3 / Fig11-38, ADR-0009 | W2 page/implementation inspection: deprecation and decode pointers, not a fabricated runtime-failing event-removal test. | Deprecated API remains. Actual removal from time and downstream decode handoff remain later work; no external decode implementation was inspected by S00. |
| T-9 — version0x0F | Shared/local code mapping; Ch11 Fig11-34 / §11.2.7.2, distinct header list | W1 setup15 runtime failure; `tests/rebuild_wire.rs::t9_setup_fifteen_is_unknown`. Separate shared mapping suite compile-first; `tests/rebuild_layout.rs::t9_separate_shared_code_lists`. | Shared code lists adopted; 0x0F unknown. Code-list names do not themselves determine a recording's edition; resolver applicability remains pending. |
| T-10 — Chapter4 binary layout/count/calendar | Code plus DERIVED/unresolved reading; Ch11 Fig11-4 and Ch4 §4.7/Figs4-3/4-4 | W1 raw word-order runtime failure; `tests/rebuild_wire.rs::t10_secondary_ch4_has_usec_low_high_words`. Later top-low-bit/reserved/usec cases and recorded visual page checks supplement it. | Types48f1834 layout/count adopted. Full32-bit binary count is recorded as DERIVED; historical BCD characters/bits wording unresolved. Explicit-origin/calendar/partial API and removal of old17-bit conversion remain pending. A page check is not a passing calendar reconstruction. |
| T-11 — Format2 caption says Format1 | **Caption-only**; Ch11 Table11-17, as recorded in STANDARD-REVIEW | Documentary finding; body-layout fixtures under T-2/T-3 protect the actual bytes. No caption-specific runtime regression is claimed or meaningful. | Do not implement a different wire format from the mistaken caption or count this as a newly fixed parser bug. |
| T-12 — body/Data Length | Code; Ch11 Figs11-13/14 / §11.2.1.1 d | W1 runtime failures; `t12_day_body_is_six_bytes`, `t12_calendar_body_is_eight_bytes`; reviewer CLI exact-buffer failures. W2 bounded new-reader tests were partly compile-first. | Correct6/8-byte bodies and bounded F1/F2 readers; CLI consumer minimum lengths corrected. A complete CLI framing/joining loop is not established by these consumer tests. |
| T-13 — handbook edition | **Documentation-only**; RCC123-09 §6.6 printed6-36 versus RCC123-20 §5.6 printed5-44 | Rewritten L1-COR sources distinguish them. E1/E2 RCC worked example is independent numeric/basis evidence; the numeric examples passed main before E1 changes. | No code-specific failing T-13 test is claimed. The full README/usage publication refresh remains a later slice; handbook guidance is not silently relabelled standard text. |
| T-14 — ITS applicability has no own data-type version | Reading/source qualification; Ch11 §11.2.3.2 / Table11-4; RDG-007 | Explicit-layout ITS test was compile-first in W1; source and page notes document pre17 reserved fields. No missing-era resolver runtime failure is manufactured. | Explicit caller layout gates ITS; missing-edition zero ambiguity and effective-edition resolver/analysis remain pending. Design acceptance does not fill those gaps or complete human review. |
| T-15 — secondary checksum reading | Code plus **suspect** RDG-008; Ch11 §11.2.1.2 c, RCC123-20 B-31 | W2 runtime byte-only/word-only failures; `tests/slice2_wire.rs::rdg008_byte_only_checksum_is_valid`, `rdg008_word_only_checksum_is_not_trusted`; new both/neither API compile-first. | Per-header byte trust plus both sums exists. Recording-level aggregation/own-time attachment and local vendor evidence remain pending. Synthetic tests do not settle suspect status. |
| T-16 — leap second rejected | Code/reading; RCC200-16 §3.6/A.2, Ch11 Fig11-13, RDG-010 | W2 new calendar/scale API compile-failed first; `tests/slice2_readings.rs::t16_leap_second_retained_only_at_permitted_utc_dates`, `t16_calendar_known_year_and_wrong_time_boundaries` later passed. | Explicit qualified UTC second60 is retained. This is not an executed prototype-failing new-API test or proof of full leap arithmetic, chrono preservation, NTP ambiguity or leap-table conversion. |
| T-17 — missing scale | Code/reading; Ch11 §11.2.3.2/3, RCC200-16 A.2, RDG-009/012 | W2 new scale API compile-failed first; `t17_f2_stated_scales_do_not_depend_on_counter_value`, `t17_all_fmt_scales_and_policy_provenance` later passed. E1/E2 retain ordinary reference scale evidence. | Raw stated/assumed scales exist; native GPS stays GPS. Mixed-format normalized arithmetic and RDG-012 era/PTP currency conversion remain pending. No blanket all-scale conversion correctness claim. |

### Historical tests-first chronology and qualifications

This chronology is an inspection of existing evidence, not new execution or a
retroactive repair of an absent failing test. Records are scoped to their own
heads; counts are not current all-suite totals.

| Record / baseline / tests-only commit | Executed-before evidence recorded | Qualification |
|---|---|---|
| **W1:** [first slice](../verification-first-slice.md), base `be33a72`, tests-only `5692427e632b1ebb876452b3ce3d7f896ff9791f` | `rebuild_wire`:0 passed/11 failed; `rebuild_layout`: compilation failed,5 cases did not run. | Later assertions expanded those cases. Reviewer tests-only `5067b94` recorded five reserved-warning/CLI failures before corrections. The initial strict reserved-word assertion was later corrected; do not retain it as accepted warning semantics. |
| **Q-FLAGS:** [PR #9 evidence](https://github.com/TelemetryWorks/irig106-time/pull/9), base `644b8bc`, tests-only `53a049d760c31556eaa81033d96e71d264d4952c` |14 passed/3 failed: exhaustive dispatch, literal RTC/reserved format and distinct secondary error. | The Debug-based existing API allowed runtime failures; additional separate-validator API tests arrived with implementation. PR #10's independent4×4 oracle and exact literal assertions passed its baseline20 tests before mapping-document changes; they are oracle strengthening, not extra failing regressions. Q-FLAGS-ERROR was subsequently decided as TF-028; the PR #9 TODO is historical. |
| **W2:** [slice 2](../verification-slice-two.md), base `30898a7`, tests-only `afa411b` | `slice2_wire`:0 passed/9 failed; `slice2_readings`: compilation failed,8 cases did not run. | Source/page checks and new API tests are not executed prototype failures. T-4 already had the helper fix at that baseline. The NTP refusal tests pin NeedsContext, not the later accepted RDG-012 conversion. |
| **E1:** [offline record](../verification-e1.md), base `3c1cf2e`, tests-only `ca146936d6d1b6692358defc58aa667e97eaef3a` | **Only2 of the eventual13 original tests failed first** (late default and drift); two numeric worked-example baselines passed main. | Nine tests arrived with implementation. Review tests-only `8cac04a79f875c5a262e591e6c6a66135ed34951` recorded20 passed/1 failed for SRC3 admission; seven additional behaviour tests passed baseline but each failed under its targeted mutation. |
| **E2:** [stream record](../verification-e2.md), base `ce78185`, tests-only `5354f9a` | TF-024 raw-counter evidence test executed failing; nine initial stream tests could not execute because the API was missing. | Two further stream test functions/additional assertions arrived during implementation; exact-counter endpoint and same-counter jump cases failed before their fixes. Review tests-only `200217b` recorded15 passed/2 failed; four channel/identity cases passed baseline but failed their mutations. All34 final mutations were caught and source restored; this is not34 original prototype failures. |

The contract section7 fixture is its literal36-byte packet. RCC123-20 §5.6
supplies numeric reference/query values, not a complete wire packet; its test
envelope is hand-derived from Chapter11 layouts. Passing those numeric
baselines on main is not proof the later complete basis API failed first.
Round trips supplement these independent fixtures, not replace them.

**Open evidence deficit:** L1-TST-001 → L2-TST-101 → L3-TST-101 is not
fully satisfied merely by this inventory. The compile-first/new-API and
later-assertion cases above remain explicitly distinguished. Caption-only
T-11 and documentation-only T-13 do not concern a code fix in the same sense
as a faulty reader. T-4's missing original runtime proof is not supplied by
a later already-correct version-helper test. Future regression or mutation
work must name its actual baseline; no historical result is invented here.

## Reading-analysis and human-review gate

Trace: **L1-RDG-002 → L2-RDG-102 → L3-RDG-102**. Current matrix metadata
reports **8/12 entries without a written analysis**, **7/12 without a tagged
test**, and zero missing development marks. These are link/checklist counts,
not semantic or two-person-review certification. L1-RDG-002 already displays
Implemented (I) from its existing script Evidence link; that does not prove
the missing analyses/edges complete, and its RDG-102 children remain Draft.
The register records all
12 Review fields pending, RDG-008 Design suspect and the others accepted;
those exact fields remain unchanged. A written-analysis link does not by
itself prove every touched edition/edge or local vendor check complete.

The table is a **proposed follow-up queue for the coordinator to assign**,
not permission to start another slice or contact/recruit reviewers. Before
extending a reading, its assigned work must re-read the touched archived
editions, list edges and derive edition fixtures under the existing
Development convention. Independent human checks remain under the register
conventions; this audit assigns no people or new review gate. Existing code with an absent
analysis is a recorded deficit, not retroactive fulfilment or a reason for
S00 to alter runtime behaviour.

| Register entries | Existing evidence at baseline | Follow-up / hold |
|---|---|---|
| RDG-001..004 | Analysis not yet written; no interpretation-tagged tests. RDG-002 page/default decision notes and RDG-003 owner inference exist, not a completed analysis/test suite. | Proposed S09a: both vocabularies, every TFMT letter/FMT pair and RCC124-22 Ch7 example; preserve opt-in default and inference. No comparison code in S00. |
| RDG-005/006 | Analysis not yet written; no interpretation-tagged tests. | Proposed S09b: source informational changes, SHTF mapping and Chapter9/11 wording distinction. No new mapping allowed outside accepted RDG-001..006. |
| RDG-007 | Explicit-layout ITS tests tagged; analysis not yet written. | Proposed S03: edition/ITS evidence analysis, unknown-edition cases and fixtures. Page notes alone are insufficient. |
| RDG-008 | Written per-header dual-checksum analysis and tagged tests in slice2. | Proposed S10a: recording aggregation/own-time evidence; local vendor checks remain local. Keep suspect Design and pending human Review. |
| RDG-009/010 | Written scale/leap reading analyses and tagged tests in slice2; E2 additional tagged cases. | Proposed conversion/adapter work and S18 audit: extend analysis/tests before adding normalization/leap arithmetic; do not extrapolate raw-reader proof. |
| RDG-011 | Per-session first-time test tagged; analysis not yet written. | Proposed S18 analysis/fixture closure and source audit. E1/E2 findings do not substitute for the missing written analysis. |
| RDG-012 | Written all-edition/RFC analysis; no interpretation-tagged conversion tests. Existing NTP tests still refuse conversion. | Proposed S05 after the coordinator's go; plan Q1 finding IDs/mappings pending. No RDG-012 conversion starts here. |

The human rule is the register's existing requirement: author and different
reviewer, both people; AI drafting/tooling counts as neither. The coordinator
must arrange and record that process before claiming human review complete.
PR approval/tool success alone does not change any register Review field.

## Pending plan questions — no decisions taken

These are the labels in the coordinator-saved **0.8 plan**, not the older
roadmap Q1–Q9 already answered in November. Recommendations in that plan
remain recommendations; S00 chooses no option and approves no deferral.

| Pending item | Boundary/source to preserve | S00 action |
|---|---|---|
| Plan Q1 — additional finding IDs/levels or reuse mappings | NTP zero: RFC5905 §6 p14; leap ambiguity: RFC1305 AppendixE.8 pp78–79; edition/year conflict evidence: accepted resolver/year requirements. IDs/levels are library policy, not standard assignments. | Inventory the requests only. No numbers, levels, aliases or producers selected; accepted TF-028 is not this pending request. |
| Plan Q2 — inverse interval boundaries | Ch11 RTC units and contract4.9/5.5 do not select query inclusivity. | No half-open interval convention adopted. |
| Plan Q3 — release-scope deferrals | L1 scope assigns release timing to ROADMAP; the saved plan is not an owner waiver. | No inverse-query, encoder or deserialization requirement is deferred by S00. |
| Plan Q4 — binary distribution | ADR-0007/0017 and REL-106; IRIG106 does not choose distribution mode. | No crates-only/binary-artifact release choice, publication or attestation policy change. |
| Plan Q5 — exact-integer JSON schema | API-107/110, CLI-106; IRIG wire widths do not prescribe JSON representation. | No decimal-string schema or new serialization dependency chosen. |
| Plan Q6 — simultaneous input/output error precedence | Accepted exit classes0–4 and usage-before-input remain; Q-CLI-EXIT does not settle concurrent input/output failure precedence. | No first-fatal or other new ordering selected. |
| Plan C1 — interpolation floor | L3-STRM-202 proposal, already labelled in E2. | Remains a proposal; **NTP fraction floor is separately already accepted**. |
| Plan C2 — nearest midpoint tie | L3-COR-102 proposed preceding tie. | No promotion to owner-accepted default. |
| Plan C3 — mixed-resolution jump threshold | L3-COR-104 coarser-resolution proposal. | No promotion to owner-accepted default. |
| Plan C4 — mode-change drain | L3-POL-102 / STRM-206 old-policy flushing proposal. | No promotion to owner-accepted transition. |

Historical Chapter4 characters/bits ambiguity also remains reported under
the already accepted explicit-context rule. S00 does not resolve it.
No external RFC archiving, cross-repository source work or release action
is authorised by this queue.

## Changed requirement IDs and proof limits

| Changed IDs | Parent chain / exact change |
|---|---|
| L2-WORD-102; L3-WORD-102 | L1-WORD-002 → L2-WORD-102 → L3-WORD-102. Existing PTP source appended; L3 stale-table rule corrected. No change to component distinctions or NTP sources. |
| L2-FND-101; L3-FND-101 | L1-FND-001 → L2-FND-101 → L3-FND-101. Catalog through accepted TF-028 and exact existing flag/owner source; evidence/severity invariants unchanged. |
| L3-FND-102 | L1-FND-002 → unchanged L2-FND-102 → L3-FND-102. Add only accepted undefined-secondary TF-028 to the existing dispatch list and its source. No accepted row/level changed. |
| L2-RDG-102; L3-RDG-102 | L1-RDG-002 lineage unchanged. Link this audit under planned verification, explicitly not satisfaction evidence. |
| L2-TST-101; L3-TST-101 | L1-TST-001 lineage unchanged. Link the inventory and historical gaps without promoting proof status. |

L1, all parent relationships, verification methods, IDs/counts, all earlier
Source quotations, every Design/Review field and architecture's finding rows
are preserved. No `Evidence` field is added to mark an incomplete obligation
verified. Matrix regeneration/freshness is checked; coverage remains
**106/288 (36.8%)**, counts **124/144/144**, zero orphan/unknown IDs.
Its linked status cannot express partial clauses; S00 does not claim the
RDG-102/TST-101 requirements fulfilled or complete standard compliance.

## Checks executed for this documentation PR

Local documentation checks: regenerate the matrix and run `--check`;
verify relative Markdown file links, all17 finding rows and cited test names,
changed parent chains/ID counts, preservation of previous Source quotations,
protected Design/Review fields and TF-001..TF-028 rows, unchanged non-document
paths, and `git diff --check`. The PR reports the hosted CI result at its
actual head. No Rust suite, compilation, mutation campaign, build, page render,
dependency install or publish dry-run is claimed as newly run locally by S00.
