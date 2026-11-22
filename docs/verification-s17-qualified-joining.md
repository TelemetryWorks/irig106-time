# S17 qualified-joining and answer-basis closure audit

Inspection baseline: main `6f84e9c` (#105 merged), 2026-11-22. Scope is
L1-CLI-004/005/006 -> L2/L3-CLI-104/105/106. This is a source/evidence audit,
not a new runtime test, standard-page check, upstream release-status check or
approval to implement anything. Existing reviewed tests and records below are
evidence; they were not rerun for this docs-only PR.

## Closure decision

**CLI-104/105/106: Draft — closure gated on TMATS (ADR-0021).** This applies
to the whole-requirement closure checkpoint, not to every individual clause.
No new whole-requirement test marker, verification credit or requirement change
is justified by this inspection. The closed clauses below retain their evidence.
In particular, CLI-105's literal protocol/status/scale obligations are evidenced
for the current observed-only views; there is no intrinsic TMATS dependency in
those spellings. Keeping its whole-requirement checkpoint Draft does not mean
NTF/TS rendering is missing, or add a declaration requirement to CLI-105.

The audit closes the *uncertainty in the previous ledger*: session-year
qualification in summary is integrated by #104, not still an implementation
TODO. It does not close full qualified packet/query answer provenance. No
additional whole requirement can be marked Implemented/Verified now. Coverage
remains **198/288 tested; 206/288 verified**. The generated matrix cannot express
partial clause completion; these tables are the clause-level disposition.

## Closed clauses and evidence

Paths under `apps/irig106-time-cli/` below are the CLI package, abbreviated `CLI/`.
Each row names clauses, not all of its parent requirement.

| Lineage | Closed clause | Existing evidence |
| --- | --- | --- |
| L2/L3-CLI-104 | Six commands share the bounded validated packet walk; Data Length excludes filler; checksum/resync, bit 7, no-complete-packet exit 3, truncated/padded final tail findings | [S16 reader/join record](verification-s16b2a.md); CLI/tests/s16_reader.rs, s16_packet_bounds.rs, s16_join.rs |
| CLI-104 | Atomic same-directory file CSV / system-temp stdout spool; findings publish; operational rollback; private temps, permissions, exclusive random names, non-regular/read-only rejection and cleanup | [CSV transaction](verification-s16b2b1.md), [bounded findings/read-only recheck](verification-s16b2b2.md); CLI/tests/s16_csv_atomic.rs |
| CLI-104 | Exact severity counts with bounded samples/N-more; bounded references, online jump/gap state, streamed rows and segment spools | CLI/src/main_tests.rs: finding_samples_are_bounded_and_levels_remain_exact, csv_row_sink_does_not_retain_packet_rows, online_reference_jump_and_gap_state_is_bounded_and_exact; [dense RSS record](verification-s16b-final-rss.md) and scripts/check-dense-rss.py |
| CLI-104 | Immediate >30 s reset and three-increasing-late-reference reset; one/two late packets remain late; reset baseline/cache isolation, overlap warnings/labelled segment answers | [reset/isolation record](verification-s16b2b3c2.md); CLI/tests/s17_correlate_report.rs: reset_query_keeps_both_segment_answers_and_other_channel_exact, correlate_json_reset_golden_keeps_each_segment_and_unreset_channel |
| CLI-104 | Continuous day365/366 -> day1 rollover with actual year length; real boundary jumps retained; year-relative text; no reference-less inventory query results | [report/rollover record](verification-s17a1.md); CLI/tests/s17_report_text.rs and s17_correlate_report.rs |
| CLI-104 | Summary recording-wide native sessions, counter spans, F1/F2 candidate quality and calendar endpoints, normalized drift and stale flag; distinct from per-channel CLI reset segments | [native summary quality](verification-s17-summary-quality.md), [F2 quality](verification-s17-native-f2.md), [CLI year replay](verification-s17-cli-year-replay.md); CLI/src/native_sessions.rs, native_quality.rs, year_replay.rs; CLI/tests/s17_summary_sessions.rs |
| CLI-104 | Summary same-session DMY year priority/rollover/contradictions and disk-backed replay; unavailable/prefix-only/context states remain explicit | [#104 regressions and review evidence](verification-s17-cli-year-replay.md), [library replay foundation](verification-s17-year-replay.md); this replaces the old ledger's open session-year note, not the counter-query engine |
| L2/L3-CLI-105 | NTF0=NTP, 1=IEEE1588-2002, 2=IEEE1588-2008; reserved selectors retained, TS valid/not-valid/reserved, never network-as-GPS; stated/assumed/unknown scale labels | [network facts](verification-s17-network-facts.md), [inventory golden](verification-s17a4.md), [adapter goldens](verification-s17-completion.md); CLI/src/network_facts.rs; CLI/tests/s17_adapter_goldens.rs, s17_inventory_golden.rs, s17_network_facts.rs |
| CLI-105 | Packet facts versus selected-reference facts are distinct in timeline/query text and JSON; CSV appends four packet-fact columns including named status, with empty non-F2/missing-CSDW values | CLI/tests/s17_network_facts.rs: packet_observation_is_distinct_from_the_query_reference, invalid_packet_label_is_explicit_when_clock_comes_from_other_protocol, csv_raw_and_named_status_are_exact_for_all_nibbles; [19-column contract](verification-s17-csv-facts.md) |
| L2/L3-CLI-106 | Text/JSON share Rust reports, timeline/CSV share packet rows, correlate uses one query report; text unresolved differs from zero; CSV quoting/CR/LF/absence, units and append-only columns | [packet views](verification-s17b1.md), [streamed JSON rows](verification-s17b2.md), [query report](verification-s17b3.md), [query JSON](verification-s17b4.md); CLI/tests/s17_packet_rows.rs, s17_timeline_json.rs, s17_correlate_report.rs |
| CLI-106 | CLI-owned stable versioned JSON, one schema review/freeze, additive revisions to 13; units, ASCII, decimal-string wide integers, input/tool/options provenance, deterministic ordering and schema validation | [schema review](verification-s17a3.md), docs/cli-report-schema.md; CLI/schema/irigtime-report-v1.schema.json; CLI/src/report_json.rs projection/decimal-extrema tests; scripts/test_report_schema.py and validate_cli_reports.py (30 live views) |
| CLI-106 | Actual compatibility policy/default IDs, 30 s/1024 caps, reset thresholds, sample caps, leap-table identity; explicit --ntp-era, no CLI pivot inference | [policy identity](verification-s17-policy-identity.md), [era boundary](verification-s17-ntp-era.md); CLI/tests/s17_policy_identity.rs, s17_ntp_era.rs |
| CLI-106 | Per-value F1 scale/stated-or-assumed basis and PTP covered/stale conversion basis follow the numerically selected reference through rows, query answers and reset spools | [F1 basis](verification-s17-f1-basis.md), [stale PTP review](verification-s17b5.md); CLI/tests/s17_f1_basis.rs, s17_stale_ptp.rs: decoded_packet_row_basis_follows_selected_reference_not_latest_channel |
| CLI-104/106 | Observation-only safety boundary: no matching-ID/RTC declaration association invented; post-reset data answers withheld; native candidate quality admits undeclared channels explicitly | CLI/src/lib.rs correlate/resolve, report_rows.rs (association_unestablished), native_quality.rs (include_undeclared=true); this closes conservative behavior without evidence, not future evidence-aware joining |

These are the exact current scopes: packet/query clocks use compatibility
nearest-reference selection and its metadata mirror; native session/year/quality
results are separate summary data. A summary calendar endpoint is not proof that
all six commands use a native replayable answer. CSV intentionally has no basis
columns; stale/basis detail is available in findings and JSON, as documented.

## Open qualified clauses: gate versus implementation

| Clauses / requirement lineage | What is missing | Dependency and required evidence after permission |
| --- | --- | --- |
| CLI-104 channels declarations/observations and shared governing join; CLI-106 raw/stated/assumed declaration provenance | Read bounded TMATS time view and setup packet ranges; retain setup/version evidence, TIMEIN/TSRC/TTF/TFMT/SHTF and `R-x\RI4`; apply correct governing range per packet/session | TMATS-gated. Full published 0.1 + bounded view + explicit owner dependency approval; test range changes, absent setup and conflicting declarations, not only one setup at file start |
| CLI-104 basis/findings; CLI-106 declaration comparisons and labelled policy defaults | Wire existing library RDG-001..006 comparison/admission evidence into CLI report findings and clocks; inferred I<->FMT3, missing required TFMT, opt-in labelled A, lost-lock information and edition-qualified EXT/SRC | TMATS-gated at CLI consumption, not missing library comparison code. Library declaration_checks.rs and declarations.rs are already available; qualified CLI goldens must cover governing setup locations and keep all accepted TF levels |
| CLI-104 packet/query source selection; CLI-106 complete relationship provenance | Real data-channel/time-channel association and correct reset segment; preserve association_unestablished / association_conflicting / association_evidenced | TMATS integration gate, but TMATS alone is insufficient: Chapter 9 has no linking field. Both unestablished and conflicting remain unresolved; evidenced needs actual relationship evidence and matching session/segment. No channel-ID/RTC coincidence inference |
| CLI-104 every command's answer basis (ADR-0011); CLI-106 full raw/stated/assumed provenance | Native AnswerView/reference identity, channel-choice reason, session/counter/reference/distance/position, source/FMT/ITS, year basis and policy/fallback threaded into packet/query answers | Integration incomplete, **not inherently dependent on TMATS for every field**. Full governing-source closure depends on rows above. Existing compatibility clocks plus policy IDs/basis strings do not supply the entire native envelope |
| CLI-104 joining own-header/caller-extracted time answers with counter answers and findings | Secondary/ERTC consistency and stamp/word adapter context through qualified reports, without fixed recording-event meanings | Library adapters are available, CLI integration is not demonstrated. Own-header parsing/context is not intrinsically TMATS-dependent; data-body extraction/word definitions need decoder/caller evidence. Do not invent body layouts or treat library tests as CLI end-to-end coverage |
| CLI-106 full-provenance planned verification and CLI-105 qualified-path regression checkpoint | Same closed protocol/rendering/schema rules applied to final qualified answers, including setup-derived year/default/inference, conflicts, unknown context and reset association | Integration validation after the gate; existing observed-only protocol tests do not prove the future join. CLI-105 itself adds no new TMATS semantic obligation |

The last three rows are explicit integration work, not claims that installing
TMATS automatically completes the CLI or that every native-basis field needs
TMATS. No new post-0.8 deferral is made. No non-TMATS implementation is started
by this audit: the coordinator's 2026-11-22 instruction is to stop and wait.

## Stop point and checks

ADR-0021 is the governing release/dependency gate. No TMATS dependency is added
and no upstream release is presumed. Wait for the coordinator after TMATS 0.1;
S18-S20 require an explicit go-ahead. No release, version bump or publication.

Local checks for this audit: regenerate/check the trace matrix, validate local
Markdown links, verify requirement statements/sources and Review/Design fields
are unchanged, and diff hygiene. No runtime tests/builds/mutations are warranted
for this documentation-only change. PR CI provides its normal checks; earlier
recorded mutations, RSS and reviewer probes remain historical evidence.
