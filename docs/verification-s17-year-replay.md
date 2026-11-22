# S17 item 6 — bounded year replay foundation and stale drift

Base 4479dfa (#102). Partial COR-110/113 and CLI-104/106 clauses only; no new
whole-requirement markers. Coverage remains 198 tested / 206 verified of 288.

The public years cursor stamps in packet observation order using the existing
YEAR-PHASE rule and offline gap handling. A frozen resolver keeps final first/
latest DMY evidence separately from phase anchors, as the native offline engine
does. Opaque stamps retain session/segment/origin/policy scope. Replays restart
at the actual segment origin; callers retain rejected leap labels and repeat
until no NEW forbidden labels are found. Policy validation, scope/order/range
and checked counter-span guards reject invalid use without advancing the cursor.
State is bounded in recording length (policy-owned collections retain caller
size); references/history remain caller-owned. No I/O, serde or opaque loading.
Sources: 106-24R1 Figures 11-13/11-14, contract §5.3, D9 and RDG-010;
owner-adopted YEAR-PHASE reader policy 2026-11-15, not a new standard mandate.

CLI disk-backed retrospective year replay is NOT wired by this PR. Existing
`year_inference: packet-only` remains honest; that integration is the next slice,
then item 7 docs/migration. TMATS and data/time-channel association remain gated.

Native quality now retains whether an actual contributing normalized pair used
a stale UTC offset, including interior references. Available drift gets a text
suffix and revision-12 JSON boolean. Pure TAI arithmetic and unavailable drift
are false, even if a separate UTC display is stale. No finding level/count change.
Hand fixture: 2035-01-02 Unix 2,051,308,800, NTP 4,260,297,600,
TAI 2,051,308,837 using last known +37 s; mixed interior F1/NTP with TAI endpoints.
Undeclared references retain false declared_timein and Rejection::Undeclared;
text rejection diagnostics are asserted explicitly.

Tests-first 88dc1e4 fails compilation on the absent replay API and runtime on
the missing stale boolean; requested small provenance/text assertions pass base.
Additional scope/gap/overflow/late-reference oracle cases arrived with the API.
One new-wrapper defect was caught during implementation: freezing phase anchors
lost DMY evidence skipped by phase tracking; a failing oracle test prompted the
separate evidence collector. Native engine year arithmetic was not changed.

23 compiling mutants fail: undeclared flag/reason, missing rejection text, stale
tracking/last-pair overwrite/value guard/pure-TAI/text suffix, closed segment,
replay gap/segment/origin, policy validation, session/order, resolver policy/domain/
range, counter overflow/headroom, rebuilt-label signal, future anchors and phase
anchors substituted for final DMY evidence. Sources restored after each probe.
Focused tests, fmt, touched Clippy and trace/schema checks locally; OS/MSRV/WASM,
package/deny/fuzz/schema/RSS checks run in PR CI. No full local matrix.
