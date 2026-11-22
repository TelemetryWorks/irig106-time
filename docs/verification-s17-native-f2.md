# S17 item 6 — native Format 2 summary quality

Base main 1e8646b (#101). Partial CLI-104/106, F2COR-101 and COR-113 clauses;
no whole-requirement markers. Coverage stays 198 tested / 206 verified of 288.

Format 2 enters the existing bounded session/late-order/quality pipeline. Its
immutable capture retains actual session, packet, raw RTC and header version.
New public reference::admit_format2_reference validates capture/key/context and
policy, then calls the same admission routine as both engines. It preserves
unwrapped keys, policy levels and raw-rate evidence without retaining history.
Shared-policy preparation requires the engines and is rejected, never ignored.
No header-flag/rate evaluation or full native answer provenance is claimed here.
Returned findings are not double-emitted: the existing CLI scanner remains the
finding emitter (including the governing-setup TF-008 rule). Native admission
keeps the explicit undeclared-candidate override and excludes invalid TS/NTF.

CLI NTP guard rejects nonzero raw time without --ntp-era; the zero sentinel stays
unknown independently of era, while fraction-only raw time works with era0.
Actual context is recreated per session; no host clock/pivots/implicit era0.
Native PTP stays TAI; NTP keeps UTC and per-endpoint leap ambiguity/coverage.
Ambiguous values remain readable but physical drift is unavailable. Limits and
spooling are unchanged, including the dense F1 RSS cap-exhaustion probe.
Sources: 106-24R1 §11.2.3.3/Figs 11-15..17; RDG-012/D7; RFC 5905 Figure 4.

Tests-first 565a75d has three runtime failures on absent F2 quality/context;
existing ten tests including the requested measured ppm assertion pass. edb7496
fails compilation on the absent standalone API. caa86ae fails on absent NTP
endpoint leap labels. Added policy/shared/wrapped-key guard cases arrived with
implementation and are not claimed failing-first. Literal 2017-01-02 UTC/TAI
(+37 s) points give zero mixed NTP/PTP/F1 drift; PTP endpoint is second 37 TAI.
Era 1 / offset 63,104 / half fraction gives 2036-02-08 +0.5 s in both native sessions.
Malformed/short bodies, TS 0/reserved TS, reserved NTF and sentinel are rejected.

Twenty compiling mutants fail: missing F2 hook, era guard/fraction/sentinel
rules, context session, era reset/source/value, three leap-label mappings, ppm
suffix; standalone capture session/packet/RTC, context, policy/shared guards,
raw-rate preservation and unwrapped key. The zero-session guard is redundant
with nonzero immutable captures plus equality; no independent mutant claim.
Sources restored. Focused tests/fmt/touched Clippy/trace/schema checks only;
full OS/MSRV/WASM/package/deny/fuzz/schema/RSS is PR CI. Session-year integration
is next; item 7 follows item 6. TMATS/data association remains gated.
