# S17a4 — native GPS and network inventory golden

Stacked on #80's metadata-only freeze (stable v1/revision2). No runtime, schema
shape, dependency or Review/Design change. This is a partial CLI-105/106 clause
slice; no whole-requirement markers or matrix coverage increase. Other S17a
stale-table/reset goldens and S17b timeline/CSV/correlate work remain OPEN.

Tests-only9deddaa adds a complete normalized JSON output golden for four channels.
It passes against the existing implementation: a numeric/projection baseline,
not a claimed failing-first feature fix. During fixture construction the expected
NeedsContext message spelling was corrected against error.rs ("time needs
context"); that fixture mismatch is not a production defect or failed-first proof.
No fixture is generated from captured CLI output or a production encoder.

Hand-derived payloads: 106-24R1 Figures11-12/13, FMT5/SRC1/DATE0 gives native GPS,
DOY100 12:00:00 =8596800000000000ns, year unknown. Figures11-15..17 give NTF0/1/2
and TS1. RFC5905 Figure4 gives1970 NTP seconds2208988800; fraction0x80000000 is0.5s.
Current CLI era-context wiring is incomplete: NTP inventory states UTC/native
protocol with a warning/no clock, not an invented absolute time or GPS label.
This golden does NOT claim RDG-012 CLI conversion completion. PTP UTC2017-01-02
Unix1483315200 +37 =1483315237 TAI, with123ns, maps to UTC day2, year2017 and
86400000000123ns; both PTP revisions retain native TAI and derived leap-table
clock basis. RTC0/1/2/3 seconds uses100ns ticks. Four36-byte aligned packets give
144 bytes; §11.2.1.1 checksums and Data Length are constructed without encoders.
Three admitted references, zero evictions/resets/jumps, one context warning.

Only runtime path/platform encoding and tool/build versions are normalized after
separate exact assertions. The golden pins the entire emitted key set, native
and converted scales, all protocol/status spellings, date/LY applicability,
raw words, clocks, findings and provenance. One expected test is added, with no
Requirements marker because it covers inventory clauses only.

The #79 nit names association_unestablished / association_conflicting /
association_evidenced at the design seam. Both unestablished and conflicting
remain unresolved for answers; evidenced still requires the correct governing
session/segment. Names are coordinator-supplied from tmats #9, not a claim of a
published API, a dependency, a Chapter9 relationship field, or implemented joining.
Current README/schema notes are synchronized with #80's freeze; historical #78
verification records remain historical.

One-at-a-time projection mutations all compile and fail this full golden: NTP
labelled GPS, swapped PTP2002 label, swapped PTP2008 label, GPS clock scale relabelled
UTC, and discarded nanoseconds. Source restored. The baseline golden passes.
Local checks are focused test/mutations, fmt, trace and diff only; platform/MSRV,
security and memory checks are delegated to PR CI. No complete CLI-105/106 claim.
