# S17 item 6 — Format 1 summary quality/calendar integration

Base main 2713c18 (#100). Partial CLI-104/106 and COR-113 clauses only;
no whole-requirement markers. Coverage stays 198 tested / 206 verified of 288.

Summary calls native Data-Length-bounded qualified Format 1 decoding with
unresolved edition evidence, never a header-version guess. One-packet native
TimelineBuilder applies calendar/admission checks, with explicit
include_undeclared=true for candidate analysis only (not TIMEIN evidence).
It does not change query policy or invent data-channel association. Native
normalization is checked when context exists; native endpoint scales remain.
Packet years only: no retrospective first-DMY/session-year promotion is claimed.
Format 2 is explicitly outside this slice; full item 6 remains open before item 7.
Sources: 106-24R1 Figs 11-12..14, §11.2.1.1 i; RDG-009/010; contract 3.6/5.4.
Statistics are reader analysis, not additional standard rate findings.

Quality is per native session/channel, independent of CLI reset segments.
The 1.1 s native late bound sorts pending references by (ticks, packet) before
OrderedQuality. There are at most 64 channels, 1,024 pending references and two
endpoint snapshots per accumulator. Exhaustion freezes a labelled prefix, with
no false claim that its counts cover the whole session. Closed span records
are private-spooled (read bound 128 KiB); current bounded channel metadata is
projected once, not accumulated over recording length. Full-CLI RSS is CI evidence.

Schema v1 revision 10 adds reference_quality to each native span. Exact decimal
counts/ns, density in references/s and mean drift in ppm reuse native quality.
Calendar endpoints are RTC-ordered valid references, not physical durations or
calendar extrema. Unknown year and unavailable context stay labelled. Text and
JSON project the same records; JSON retains all sessions, text retains its cap20.

Tests-first ab47b4b: the new late-F1 integration test fails at runtime on absent
quality. Four DOY100 12:00 observations at RTC0/1/3/2s give valid4, interval3s,
spacing1s, drift -1,000,000ppm (clock frozen), partial year, stated UTC endpoints.
Existing two-session tests now pin independent valid counts2/1. Unit fixtures
pin a DMY2017 year, malformed decode rejection, channel and pending caps.
Those additional guard tests arrived with implementation, not failing first.

The two reviewer-requested overflow regressions pass unchanged main: retry an
intermediate key after overflow, plus lo0/hi i128::MAX/50 (subtraction fits but
multiplication does not). Both kill their compiling mutants: eager order commit
and wrapping *100. Integration mutants remove sorting, remove late holding or
lose the DMY year; each compiles and fails its focused test. Sources restored.
Local fmt/touched Clippy/focused tests/trace/schema sync/diff only; full matrix,
package, deny, fuzz, schema validation and RSS are delegated to PR CI.

## PR #101 review delta

At reviewed b4914cb the four new integration fixtures already pass; no core
behavior was changed. Tests-only 8f9a2c5 adds: the same channel after a native
session boundary (own count and day101 endpoint); four closed-channel rows whose
serialized span exceeds 2 KiB; 1,100 live references (complete, exact valid count);
and UTC 2016-12-31 23:59:59 -> 2017-01-01 00:00:00 at RTC +2 s (36->37 offset,
zero drift in TAI). All four paths are decoder-independent Figures 11-12..14 fixtures.
The text assertion initially fails on unavailable-unit wording; text now prints
bare unavailable and retains both Unavailable:/Quality prefix only: explanations.

Six compiling mutants each fail the relevant test: omit session quality reset
(second span incomplete), omit live draining (cap instead of valid 1100), restore
2KiB record ceiling (closed-span read fails), omit TAI normalization (-500000 ppm
instead of 0), and remove either explanatory text label. Sources restored. The
summary target has 10 passing tests; trace coverage and partial-clause scope stay.

Tests-only a7d1e3c first fails on the missing warmup metadata. The steady RSS
workload now prepends 1,025 same-RTC F1 references (one per 10 packets), so the
shared reference cap fills and the native pending cap is exceeded before any
RTC advance. CI asserts both outcomes in the actual summary, then runs the long
reset-free four-channel phase. Seven small Python tests pass; a 298,656-byte CLI
probe confirms cap 1024, prefix-only native quality and zero scanner findings.
No large workload was run locally; full RSS evidence is the PR CI run.
