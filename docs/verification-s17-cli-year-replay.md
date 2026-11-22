# S17 item 6 — disk-backed native summary years

Base 9674598 (#103). Partial CLI-104/106, COR-110/113 clauses; no whole
requirement markers. Coverage remains198 tested / 206 verified of 288. The shared
summary report, not legacy query/source selection, gains retrospective DMY years.
TMATS/caller-year inputs and data/time-channel association remain gated.

Private 62-byte records retain key/session, high-water progress, kind/version,
Data-Length-bounded 16-byte time prefix and a monotone rejection flag. One raw
spool per active native session is deleted by CsvSpool cleanup. Each pass uses
original packet preparation plus only rejected leap flags; no resolved non-leap
state is fed back. Bounded native cursor/resolver objects close year-context gaps
without collecting a resolver list. Only final stable statistics/findings survive;
flags change 0->1, so at most N labels can force rebuilds (worst-case repeated disk
passes, not unbounded RAM). Raw disk grows 62N bytes, plus existing span spool.
Quality still caps 64 channels / 1024 pending references with prefix-only exhaustion.
No library I/O/serde, JSON loading, new dependency or public API.

Sources: 106-24R1 Figures 11-13/11-14, contract §5.3, D9/RDG-010 and owner-adopted
YEAR-PHASE rule 2026-11-15. Resolved ordinary F1 endpoints use the existing checked
TAI normalization before quality; second 60 retains native context-only admission.
Only newly derived year TF-004/007/032 and year-context TF-020 facts are added to the
scanner's bounded samples/exact counts; baseline decode and jump/rate analysis
are not repeated. No new severity. Plain evidence-free partial years retain the
existing summary behavior rather than adding one TF-020 per packet.
Revision13 adds year_evidence and session-dmy scope, not data-source qualification.

Tests-first 61ecd41 has 3 runtime failures (scope, missing backdated prefix year,
missing new malformed finding). Later leap/gap/provenance/count/error fixtures
arrived with implementation; no fail-first claim. The 181-day gap fixture needs
intermediate opaque headers to make RTC ordering unambiguous; an initial direct
181-day raw step correctly stopped at the native ordering bound. A proposed
unsampled-error fixture initially selected a legitimate previous leap year;
December31 anchors pin the intended common-year malformed case instead.

End-to-end fixtures cover rollover/future DMY,1,321 references before final year
qualification, separate native sessions, year-context gap, real 2016 insertion
versus forbidden day 365 second 60, contradictory candidates and deterministic reruns.
Exact 30-warning/20-sample counts plus an unsampled error pin exit precedence.
Private flag persistence, record-length/size errors and cleanup use existing
fallible spool APIs. Ten compiling mutations fail: resolution, normalization, rejected flag write/use,
year findings, live late draining, closed-gap resolution, unsampled counts/level
and per-pass sample cap. All sources restored. The extra count/error/corruption
checks arrived with implementation, not tests-first.
Focused CLI tests/Clippy, fmt and trace/schema checks locally; full OS/MSRV/WASM,
packaging/deny/fuzz/schema and dense RSS gates belong to PR CI. Item 7 remains next.

CI caught a projection-unit fixture that constructed NativeSessions directly
without the scanner's new finish step. The fixture now finalizes analysis before
rendering; its assertion that row arrays remain deferred until emission is kept.

Tests-first f665cdc exposed a new-wrapper error-class bug: future DMY year0000
makes a preceding yearless DOY need year-1. That library NeedsContext was mapped
to output exit4. The wrapper now counts it as rejected context with TF-020 at
its approved INFORMATION level, preserving the rest of the report; no malformed
value or operational failure is invented. Other unexpected invariant/I/O errors
remain fallible. The year-range regression passes after this correction.
