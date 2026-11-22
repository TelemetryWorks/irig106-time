# S17 item 6b2 — ordered native quality/calendar core and capped text

Base main ee2c3a1 (#99). Partial L1-COR-013 → L2/L3-COR-113 and CLI-104/106
infrastructure. Summary native quality/calendar integration is STILL OPEN; this
core requires already-qualified references in RTC order. No retrospective year
qualification, admission, late sorting, source choice or data-channel association
is silently substituted. TMATS remains gated; no Review/Design/marker changes.

OrderedQuality retains first/last valid native snapshots, never reference history.
Owned policy/table sizes are caller input; this is constant in reference count,
not an allocation-free wire reader. Scope is one nonzero session/channel; wrong
scope, duplicate keys, reversed order and checked overflow leave state intact.
Valid=admitted Readable(_,true), like OfflineTimeline::quality; invalids admitted
by override remain separate. Extra decode rejections count only failures without
a retained reference. RTC interval/spacing/density and mean adjacent-pair drift
reuse native absolute_delta, including scale/year/normalization/leap context.
Source: contract 3.6/5.4; §11.2.1.1 i free-running RTC; §11.2.3.2/3 rate rules.
Statistics are reader analysis, not rate findings. Duplicates/missing context
produce unavailable reasons, never fabricated zero drift. Mixed scales require
both normalized endpoints. Calendar endpoints are first/last valid by RTC, not
minimum/maximum civil dates or a physical duration. Unknown year/scale stays
partial; original raw/stated/assumed/provided evidence remains on retained refs.

Tests-only 7358ecb fails compilation on the absent API, not a runtime bug. Four
focused tests now pass. Literal Figures 11-12/14 DMY 2017-01-02 times 0/1.01/3.03 s
at RTC 0/1/3 s yield valid 3, interval 3 s, spacing 1..2 s, density 1 and +10000 ppm.
UTC/GPS/RTC/assumed-UTC agree with native offline quality; unknown-year rollover
stays partial. Mixed UTC/GPS without normalization is unavailable; normalized
TAI 37→20.01 over 1 s gives -17_990_000 ppm while preserving native endpoint scales.
15 compiling core mutants fail: units, inverted drift, validity/admission,
scale/normalization/context guards, duplicates, endpoints, session/channel/order,
counter overflow and density. Source restored. Native controller is classified
in S15's exact inventory, with meaningful Clone/Debug/Send/Sync checks.

Tests-only fdfaa02 fails text 25 vs cap 20. The CLI now shows 20 plus exact known
count/omitted line; JSON 25 remains complete. Three more mutants fail: cap 21,
wrong omitted count and lost JSON active span. This display cap changes no
analysis settings, schema 9 or JSON inventory. Structural two-snapshot inspection
establishes the core bound; current CLI RSS does not yet exercise this new core.
Local focused tests/Clippy/fmt/trace/diff only; full platform/MSRV/WASM/package/
deny/fuzz/RSS is PR CI. Coverage stays 198 tested / 206 verified of 288. Next is
bounded qualification/late-order integration and native summary quality/calendar
projection; item 7 and full published TMATS 0.1/bounded-view approval stay open.
