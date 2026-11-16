# S12a — shared policy and complete basis

Preparation starts from main38681fe. Before implementation, split proposal:
S12a1 centralizes validated immutable shared qualification settings/identity,
retains reference declaration/qualification snapshots, and wires the existing
cores without changing legacy defaults. S12a2 supplies the common replayable
answer view, typed counter duration and own/stamp/word envelopes. These are
review parts, not release deferrals; S11 inverse queries stays deferred by Q3.

Sources reused from reviewed requirements: ADR-0010/0011; contract5.3/5.6;
RCC123-20 §5.6 (5-44/PDF78), "clock time of a subsequent data packet";
106-24R1 §10.6.2 (10-26/PDF32), "minimum frequency of 1 hertz";
§10.6.1b/c, "packet generation times shall be equal to or less than 100
milliseconds" and "stream commit time equal to or less than 1000 ms".
Reader defaults remain: late1100ms, gap1s+0 tolerance, stream Standard,
Interpolate hold1s+100ms, caller progress/caps, C1 floor, C2 preceding tie,
C3 coarser resolution and C4 old-policy drain. These are cited reader policy,
not stronger guarantees on a recording. D6 words still require caller context;
D7 NTP and stale PTP use their accepted rules; D8 never inserts defaults;
D9 year default is packet/caller/RI4, with conflicts/raw evidence retained.

Design: optional shared qualification snapshot enables the consolidated path;
legacy per-function prepared inputs remain supported and explicitly untagged.
A shared instance is frozen/validated by ID. Reference inputs retain effective
and provided qualification evidence; no policy-history registry or new queue.
Mode-only identity changes keep eligible references with their old snapshots;
semantic qualification changes clear references only after old-policy drain.

## S12a1 clause ledger

| Requirement pair (L2/L3) | This part covers | Still required in S12a2/later |
|---|---|---|
| ANS-101 | Effective shared ID/defaults version; exact governing setup; original scale/year/normalization/network conversion and immediate prior policy; selected session era/table alongside existing endpoint/rate/stream fallback basis. | Common replayable answer view across offline, stream, own/stamp/word; replay-only proof and full serialized evidence (S12b). |
| ANS-105 | Existing counter-only reasons preserved while qualification changes, missing context and unsupported dates stay explicit. | Typed counter duration in the common answer view; no whole-completion claim here. |
| POL-101 | Validated common year inference order/subset, scale assumptions, caller year, own epochs/table/tolerance, declarations/TFMT opt-in and per-session NTP context. Independently selectable gap and hold intervals beside existing late bound/caps/levels/channel settings. | Consolidated own/stamp/word answer view and complete public configuration/use examples; exhaustive setting-change inventory. |
| POL-102 | Immutable effective/provided qualification; matching stream/shared IDs; invalid change leaves pending answers intact; mode-only new ID keeps eligible old references; semantic changes drain then clear. | Complete common answer/replay proof and all-setting transitions. |

These pairs already have historical Implemented links in the matrix. This part
adds **no whole Requirements markers** and does not claim those links express
full clause coverage. Baseline and regenerated matrix remain **192/288** (delta0).
New Traces tags point to actual POL/ANS clauses; the table above limits the claim.
No requirement, Design/Review field, finding level or owner decision changed.

The new opt-in shared path freezes settings before observation. None keeps the
legacy explicit preparation path. Stream channel inventory is still supplied
explicitly to StreamTimeline::new; a time-varying setup is not silently treated
as a complete advance inventory. C4 mode-only replacement retains each old
reference's qualifier while new answers name the new query policy. Changed
qualification settings require fresh references after draining old answers.
Policy IDs are caller-series identities, not globally interned/unique IDs;
stream replacement enforces increasing IDs and matching shared/core IDs.
Caller-sized setup/table/pivot data is copied into snapshots, not stored in a
hidden global registry. Existing caller caps still bound stream queues.

Year order controls inference for **yearless** Format1 values, not replacement
of an explicit DMY date: packet/caller/RI4 is the accepted D9 default. A subset
or reordered list is explicit reader policy under ADR-0010; an empty list keeps
yearless values partial. Excluded/contradicting candidates remain evidence.
Chapter4 full day/year/origin remains caller context; no nearest-packet option.
Native/stated scales are preserved; a shared leap table normalizes qualified
Format1 dates and reconverts network observations under their actual session.
A legacy second60 input still is not admitted as an absolute anchor merely
because a table is supplied; its wire label and normalization remain evidence.

## Tests-first and fixtures

Tests-only commit **52c4103** precedes the implementation: four API tests fail
to compile on main38681fe (E0432 missing policy module / E0609 missing fields).
This is compile-level API evidence, **not four runtime failures on main**.
The remaining six tests and additional assertions were written against the
prototype; they are not retrospectively claimed failing first on main. A new
dated-DMY assertion failed on the prototype (2018 instead of stated2017) and
caught an inference-scope bug before this implementation was committed.
An initial stream fixture lacked a committed channel and correctly returned
counter-only; specifying channel7 corrected the fixture, not engine behaviour.

Ten integration tests use hand-written values: Fig11-13 (11-25/PDF33) supplies
SRC1/LY1/day1 or day36623:59:60; Fig11-14 supplies DMY2017-01-01. RFC5905 Fig4
supplies era1 offset63104 (little-endian80f60000), therefore2036-02-08/day39;
the original era0 result remains separately visible. Tiny gap20/200ns and
hold7+3ns with progress1 yield deadline11: no timeout at11, timeout at12.
These are selectable reader settings, not recording timing requirements.

## Targeted negative checks

Each of these **17** temporary source mutations produced a runtime assertion
failure in tests/s12a_policy.rs, and each source was restored afterward:

| Mutation | Failing evidence |
|---|---|
| Skip shared preparation in offline | Central year/scale, setup/era, empty inference and leap-context tests |
| Skip shared preparation in stream | Setup/era conversion assertion |
| Do not apply caller year | Central year, custom order and leap-context assertions |
| Do not apply scale | Native GPS/effective scale assertion |
| Do not requalify network era | Era1/date assertion |
| Erase provided network conversion | Original era0 assertion |
| Compare IDs as semantic settings (clear old refs) | Mode-change next answer loses its reference |
| Keep references on semantic setting change | Post-change answer incorrectly remains absolute |
| Ignore hold interval | Expected timeout at12 missing |
| Ignore gap interval | Expected TF-016 for20ns setting missing |
| Drop normalization attachment | Central and leap-labelled reference evidence assertions |
| Overwrite explicit packet year | Stated2017 date becomes2018 |
| Ignore empty year subset | Unknown year incorrectly becomes2024 |
| Drop own tolerance | Expected17ns tolerance absent |
| Accept mismatched era/session mapping | Invalid policy is accepted |
| Ignore stream shared scale before decode | Leap-label TF-022 missing / TF-004 emitted |
| Ignore offline shared scale before decode | Leap-labelled reference evidence lost |

No mutation changes finding levels or invents a new fallback. TF-016 factual
classification is independent of severity, as in the existing COR-108 paths.
Full CI results are recorded below after the final restored-source run.

## Restored-source validation

All **18/18** local CI-equivalent checks passed with CARGO_BUILD_JOBS=2 and
RUST_TEST_THREADS=2: four feature test combinations; three MSRV1.85 checks;
all-target/all-feature clippy; fmt; rustdoc with warnings denied; CLI build and
clippy; no_std WASM with/without serde; matrix check; benches; all ten fuzz
targets type-checked; cargo-deny. Cargo build/test/check/doc/clippy used --locked.
Actionlint and git diff --check also passed. This is local Linux evidence;
GitHub CI separately checks Linux, Windows and macOS at the reported PR head.

## Review at3939a7e: tests-only follow-up

Add five integration tests (15 total) and one private normalization unit test.
All six pass without production changes. They are partial clause evidence;
no whole Requirements markers are added and the matrix stays192/288.
The unit test can inspect the native coordinate without exposing a new API.

Each following temporary source mutation was applied independently, produced
an assertion failure, and was restored before the final checks (**10/10**):

| Reviewer mutant | Added assertion / failing test |
|---|---|
| P4: accept civil-counting epoch at second60 | review_civil_leap_epoch_and_cross_session_pivot_are_rejected: both IEEE and Chapter4 contexts reject a valid2016-36623:59:60 civil origin; continuous60 and civil59 controls accept. |
| P6: skip pivot-session check | Same test: map key1/context1 with a valid pivot from session2 rejects; same-session control accepts. |
| P15: omit native+1s for second60 | reference::review_s12a::leap_label_advances_native_coordinate_as_well_as_tai: native1483228800000000000ns, TAI1483228836000000000ns, both exactly1s after59. |
| P16: erase provided_policy | review_requalification_keeps_immediate_prior_policy_and_clears_stale_issue: sequential offline qualification10→11→12 retains10 then11, and previously returned snapshots stay unchanged. |
| P18: leave word context at default era | review_word_context_and_missing_era_entry_keep_the_named_session: exact session2 context including declared era1 retained in word context. |
| P19: fallback names session1 | Same test: absent entry3 returns context.session3, no declaration/pivots, conversion labelled AssumedZero/era0. |
| P19: fallback reuses context from another session | Same test, with populated session2/era1, fails instead of reusing its evidence. |
| P21: retain stale shared normalization_issue | review_requalification_keeps_immediate_prior_policy_and_clears_stale_issue: unknown-scale qualification first produces an issue, successful UTC requalification attaches normalization and clears it. |
| P24 offline: use raw rather than unwrapped preparation ticks | review_offline_shared_governing_range_uses_unwrapped_ticks: range starts at2^48, raw RTC1 follows2^48−2; reference key2^48+1 uses setup99/RI4 year2024. |
| P25: mode-change guard uses fixed1s | review_mode_change_checks_the_new_hold_interval_for_overflow: progressMAX−1100000000 fits old1s+100ms exactly, but new2s+100ms overflows; reject and retain old policy. No production bug found on this path. |

### Historical stream defect atb0f8831 — fixed in the follow-up below

The recommended P24 probe **fails on unmutated3939a7e** in both Standard and
Interpolate. Reproduction using the helpers in tests/s12a_policy.rs:

1. Shared declarations contain session1/setup99 with governing range
   [281474976710656,unbounded), channel7 TIMEIN, RI4 year2024.
2. Fix stream channel7; observe data packet1 at RTC281474976710654.
3. Observe Format1 packet2 at RTC1, using input() (LY1/day1).
4. Observe data packet3 at RTC1. Expected: absolute answer using setup99,
   year2024 and reference key ticks281474976710657. Actual: counter-only,
   no preceding reference, TF-008 and missing-year TF-020 on packet2.

The stream's initial SharedPolicy.prepare receives correct unwrapped ticks.
Its stream_observation adapter then creates a fresh one-packet TimelineBuilder
with the same shared policy: a second preparation uses raw ticks1/session1,
loses the governing range and rejects the reference. Offline P24 succeeds.
Both stream-mode probes were run separately and failed their absolute-answer
assertion; these are disclosed failing probes, not committed passing tests.

The same double preparation also overwrites **immediate prior policy in the
stream**: qualify input() offline under shared10, pass that input to stream11,
then inspect the released answer's preceding qualification. Expected prior10;
actual prior11. This independent probe failed at unmutated head too. P16's
new committed offline assertions kill the erasure mutant but do not claim
this stream defect is covered or fixed. No ignored regression hides either
failure. Production repair and permanent stream regressions need follow-up
authorization beyond this tests-only update; S12a2 claims remain unchanged.

After restoring every mutant, all18 local CI-equivalent checks passed again
(jobs2/threads2, --locked), including the15 integration tests and the new unit
test in all four feature combinations. Actionlint/diff checks pass; regenerated
trace matrix is unchanged192/288. These results exclude the explicitly failing
stream probes above and do not claim their defects are resolved.


## Authorized stream repair afterb0f8831

Tests-first commit **e1340a8** adds the permanent documented reproductions:
review_stream_standard_preserves_post_wrap_setup,
review_stream_interpolate_preserves_post_wrap_setup,
review_stream_standard_keeps_actual_immediate_prior_policy and
review_stream_interpolate_keeps_actual_immediate_prior_policy. All four fail
at runtime before the fix; the other15 policy tests pass. The strengthened
offline wrap control returns absolute2024 with setup99 at2^48+1. Each stream
prior-policy test also runs an offline10→11 control that retains prior10.
These are failing-before-fix reproductions of S12a1 defects, not failing-on-main
claims or evidence of S12a2 completion. No whole markers were added.

Minimal fix: the stream's private one-packet Format1 admission adapter clones
the clock settings and disables shared **requalification only in that local
adapter**. StreamTimeline still performs the single authoritative preparation
before admission with the actual session/unwrapped key. The original stream
policy and reference qualification remain untouched; the network admission
fast path is unchanged. This implements the existing ADR-0010/0011 snapshot
and provenance contract; no new design/owner decision is needed.

After the fix, all19 policy integration tests pass, including both mode pairs:
post-wrap references remain TIMEIN with setup99/RI4 year2024 and the exact
unwrapped reference key, and stream11 retains supplied prior10 rather than
inventing prior11. The caller's original snapshot is unchanged. The two
previously documented failing probes are now permanent passing regressions.
All10 review mutations were rerun against the fix and still cause runtime
assertion failures. Removing the local requalification suppression additionally
reproduces all four new failures. Sources were restored after every mutation.
No finding/severity/default changed; matrix remains192/288 (delta0).

Restored-source validation after repair: all18 local CI-equivalent checks pass
again with jobs2/threads2 and --locked (four feature test suites, MSRV1.85,
clippy/fmt/rustdoc, CLI, no_std/WASM, matrix, benches, all10 fuzz targets and
cargo-deny). Actionlint and diff checks pass. There are no excluded or ignored
probes for either of these defects; the four regressions run in the suite.

## F3 review at9a4af58 — actual stream preparation session

Add one session-boundary scenario with Standard/Interpolate test wrappers:
review_stream_standard_qualifies_with_actual_second_session and
review_stream_interpolate_qualifies_with_actual_second_session. Establish
session1 with a data packet at RTC55, then observe a Format1 reference at RTC0
with boundary=true (session2). Only session2 has a governing range: setup200,
channel7 TIMEIN, RI4 year2024. No caller year or session1 setup can mask a
wrong session lookup. A same-counter data query releases immediately in both
modes and must retain session2 in the query/reference/range, actual setup200
and its provenance, year2024 from RecordingDate, and the exact shared policy.
The boundary reference must not be reported as undeclared (TF-008).

Both tests pass on the unchanged production code. Temporarily replacing only
session in StreamTimeline's shared-preparation key with session:1 (F3) causes
**both tests to fail at runtime** on the no-TF-008 assertion; no session2 setup
is found. Restoring the source makes both pass. Production changes: none.
There are now21 policy integration tests plus the normalization unit test.
This is partial POL/ANS session provenance evidence, not a new whole marker;
regenerated matrix remains192/288 (delta0). No requirements or owner fields
changed.

Restored-source F3 validation: all18 local CI-equivalent checks pass with
jobs2/threads2 and --locked, including the21 policy tests and normalization
unit test across all four feature combinations; MSRV1.85, clippy/fmt/rustdoc,
CLI, no_std/WASM, matrix, benches, all10 fuzz targets and cargo-deny are green.
Actionlint and diff checks pass.
