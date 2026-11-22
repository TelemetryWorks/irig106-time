# S17 full-adapter goldens and clause audit

Base main f40b286 (#90 merged); stable schema v1/revision5. This is a
verification/documentation slice: no production, schema, dependency or policy
change. No Review/Design field changes. S17's independent rendering work is
implemented, but **CLI-104/105/106 are not fully closed**. The audit below separates
implemented clauses from remaining release blockers; a golden of a limitation
is not evidence that the missing feature works.

## New executable adapter goldens

Tests-only1ef93c5 adds three passing baseline tests. They do not reproduce a new
production failure and are not claimed failed-first feature fixes. Initial
fixture corrections were schema vocabulary (`assumed, RDG-009`, `standard`) and
0x09's PCM-1 label, not production defects. No captured output or production
encoder supplies expectations. The compact [case table](../apps/irig106-time-cli/tests/golden/s17_adapter_cases.json)
and [test](../apps/irig106-time-cli/tests/s17_adapter_goldens.rs) are partial
CLI-104/105/106 evidence, without whole Requirements markers.

- Fifteen cases: all six F1 formats; valid NTP; both valid PTP revisions; stale
  PTP; invalid TS for all three NTFs; reserved TS; reserved NTF. Every case traverses
  all six text commands plus the five supported JSON commands. CSV has its own
  exact15-field golden, not a JSON wrapper. Each emitted inventory pins native
  protocol/scale/status and converted first/last clocks; packet/query projections
  pin the entire clock, null versus zero, basis and text. Renderer stderr parity
  is checked; summary intentionally renders its diagnostics on stdout.
- Chapter11 Figures11-12/13 give FMT0..5, SRC1, DATE0 and BCD DOY10012:00:00,
  unknown year. Source pages retained from S17a4: printed11-23/11-25
  (PDF31/33); Figures11-15..17 printed11-27(PDF35). No new page inspection
  is claimed in this test-only audit. Its day1-relative count is(99*86400+12*3600)*1e9 =8596800000000000ns.
  IRIG-A/B/G assume UTC/RDG-009; RTC is unknown; UTC and GPS are stated FMT.
- Figures11-15..17 give NTF0/1/2, TS0/1/3 and reservedNTF3. RFC5905 Figure4 gives
  NTP1970 seconds2208988800 and fraction0x80000000=0.5s. The CLI currently reports
  missing era/basis and no absolute NTP value; this is **not** RDG-012 completion.
- PTP2017-01-02: Unix1483315200 +37 =1483315237 TAI;2030-01-02:
  Unix1893542400 +37 =1893542437 TAI.123ns survives. C72 coverage makes only
  the latter stale; UTC remains usable with TF-021 and the stale basis/suffix.
- A full reset adapter fixture uses FMT4 DOY10012:00:00/40 at RTC0/40s, then
  12:01:01/10 at RTC1/10s: one immediate reset, two ranges, no false jump.
  Inventory segment arrays match in all four relevant JSON views. Timeline/CSV
  pin all five rows including an unassociated counter-only PCM row. RTC5s has
  labelled segment answers12:00:05 and12:01:05 with an ambiguity warning; text
  and JSON agree, any-channel answer is withheld. Header sums/Data Length/filler
  and100ns RTC are independently constructed per11.2.1.1.
- Injected failed stdout gives exit4 for all six text and five JSON paths, even
  when findings exist. Existing atomic file-spool/error-path tests remain.

Five isolated compiling mutations are killed by the new tests: GPS inventory
scale relabelled UTC; network status dropped; CSV RTC ns changed to raw ticks;
segment index forced1; stale JSON basis changed to covered. Each fails a runtime
assertion; source restored. Focused tests, fmt/diff and trace checks run locally;
GitHub CI supplies platform/MSRV/security/package/fuzz and32MiB RSS evidence.

## Clause ledger (L1 -> L2 -> L3)

Rows apply to both L2/L3 unless a level is distinguished. Evidence is cumulative;
older verification files remain dated slice records, not current closure claims.

| Lineage / clause | Evidence now | Remaining / disposition |
| --- | --- | --- |
| CLI-004 -> CLI-104: six commands, one Data-Length-bounded scan/report | S16b records; S17a1,b1..b5; new cross-view cases | Implemented for current caller-unqualified CLI |
| CLI-104: buffered input; checksums/resync; bit7 secondary; complete-packet/EOF split | S16b1/2a and their structural/content regressions | Implemented; new goldens don't repeat the reader audit |
| CLI-104: spool, atomic replace on0/1, cleanup on2..4, permissions/nonregular/read-only guards | S16b2b1/2, Windows/Unix tests | Implemented |
| CLI-104: bounded samples/exact counts, reset run rule/baselines, ambiguity, isolation, honest queries | S16b2b2..3c2; S17a3,b3,b4; reset golden above | Implemented; unassociated data remains conservative after reset |
| CLI-104: rollover and real jumps, readable clocks | S17a1 reviewer deltas (common/leap/unknown year and real boundary jumps) | Implemented for legacy report policy; not a new session/year inference claim |
| L3-CLI-104: summary sessions/spans/quality; channels declarations/observations; qualified basis in every command | Partial counters/gaps/reset segments/inventory, no independent renderer conversion | Full native session/quality/declaration joining and basis still open; heuristic reset segments are not complete qualified sessions |
| CLI-104: no fixed event meanings; unresolved data association | S14 removal/handoff; rows carry association_unestablished; S17 consumer seam | Implemented refusal to invent association; real evidenced joining is gated |
| CLI-005 -> CLI-105: correct native network labels/status/scale, no fabricated GPS | S17a1,a4 and new15-case inventory goldens in all JSON views/text inventory | Inventory implemented; CSV/timeline/correlate protocol/status distinctions not complete |
| L3-CLI-105: protocol/version distinctions in text, CSV, JSON | JSON inventory retains NTF/TS/raw; packet/query clocks retain PTP conversion basis | CSV has only Time-F2, no protocol/status/scale fields; timeline/query do not carry complete protocol facts. **Not TMATS-gated** |
| CLI-006 -> CLI-106: shared Rust report, text/CSV/JSON, reviewed stable schema | S17a3/schema review, freeze#80, b1..b5; revision5/source+docs sync | Implemented format/projection clauses; no csv-in-JSON requirement |
| CLI-106: units, decimal wide values, null/zero, deterministic ASCII/provenance | S17a3,b2,b4 and projection/extrema/path tests; CLI schema | Implemented current fields. No128-bit field exists; future ones must remain decimal strings |
| CLI-106: quotes/commas/CR/LF, absent calendar, packet/query/reset parity, writer exits | S17b1..b5 plus new cross-renderer/reset goldens | Implemented for current limited report model |
| L3-CLI-106: schema validation and one review before freeze | Reviewer validated2020-12 metaschema/output in#78; revision rules/sync/key tests thereafter | Full schema-validator regression on latest views is still a final validation task; shape checks alone aren't claimed full validation |
| L3-CLI-106: policy/defaults identity, complete raw/stated/assumed answer provenance | Tool/input/options; raw F1nibble/F2word; channel assumptions/PTP currency; stable Rust AnswerView exists | Native answer-policy/reference provenance not fully projected. F1 packet/query scale is unknown despite inventory GPS/assumedUTC. NTP conversion has no CLI era-context wiring. **Not wholly TMATS-gated** |
| CLI-104/106: governing ranges; inferred/default TFMT in reports | S09a/b library mapping tests; S17 seam only | CLI setup extraction, bounded view, comparison/report propagation and TFMT goldens await full published TMATS0.1 plus owner approval |
| CLI-007 -> CLI-107:0..4, threshold, precedence/host parity | S16a3b and S16b operational regressions; new failed-writer tests | Existing whole-chain coverage unchanged; no new threshold policy |

No new whole requirement closes. CLI-104/105/106 retain Draft in the generated
matrix: **198/288 tested(68.8%),206/288 verified(71.5%)**, delta0/0. The matrix
cannot encode the partial clauses above. Do not mark them Implemented merely
because the new test reaches every renderer. Statements/sources are unchanged;
only stale Planned-verification evidence pointers are refreshed.

## S18-S20 handoff / release blockers

The original coordinator plan is external to this checkout; this ledger does
not claim to inspect it, rename its phases or amend owner decisions. For the
coordinator's S18-S20 sequencing, carry these concrete remaining obligations:

1. **Integration, before closure:** consume full published TMATS0.1's bounded
   time view only after owner dependency approval(ADR-0021). Preserve governing
   ranges, no hidden defaults, required-TFMT finding even with opted-in A,
   default-derived/inferred provenance and RDG001..006-only comparisons. The
   seam names association_unestablished/conflicting/evidenced; conflicting is
   unresolved. Chapter9 provides no data/time-channel link. Real evidence is
   required, not channel-ID/RTC coincidence. Neither release nor dependency
   gate is cleared by this PR; no upstream release status was queried.
2. **Independent integration gaps:** finish native policy/reference basis and
   NTP-context projection; preserve GPS and all network protocols/statuses in
   packet/query/CSV adapters. Recommend a reviewed CSV-column contract update
   and bounded adapters over existing typed Rust answers, not guessed labels.
   These must not be hidden behind the TMATS gate or called approved deferrals.
3. **Documentation/migration and verification:** refresh current README/usage/
   examples to106-24R1 and actual APIs/options; consolidate0.8 migration/export
   audit and T1..T17 effects; audit remaining requirements and clause records;
   schema-validate all current views; retain bounded CI, fuzz/mutations and dense
   reset/steady+100k-row RSS evidence. Existing CI covers much of this, not final
   release acceptance of future integration changes.
4. **Release preparation/owner decision:** REL-101..105/ADR-0017 checklist;
   set lockstep0.8.0/exact pin, final package verification/dry-run evidence for
   both crates, actual release source revision and reviewed CHANGELOG. Types0.1
   adoption is already done, not a still-missing git-to-registry migration.
   Keep CLI publish=false until owner S20 approval; actual publication only
   Trusted Publishing/OIDC, no local publish/token/tag/merge here.

Accepted post0.8 deferrals remain inverse queries COR-111, a NEW complete-header
encoder ENC-103 and prebuilt distribution REL-106; offline drift-corrected
queries remain later. Complex library JSON rehydration is moot after ADR-0019,
not an obligation to restore serde. Existing encoder corrections are complete
(S13); library Jiff/serde-removal/layout moves are already done. The open adapter
clauses above are release blockers, not additional unilateral deferrals.
