# S17 item4 — explicit caller NTP era

Base maindba6a1c(#95). Partial CLI-103/106/107 and RDG-012 adapter evidence,
no whole Requirements marker/Review/Design or dependency/library change.
The coordinator decision is explicit --ntp-era N, labelled caller-declared era;
without it nonzero NTP stays needs-context, no automatic pivot/era0/host clock.
The corrected era1 end is2172, not2104 (coordinator-confirmed2026-11-22):
RFC5905 §6/RDG-012/library arithmetic uses2^32 seconds (~136years), so era1
continues to2172-03-15 12:56:32. This clarification does not implement pivots.

Strict OsString parsing accepts optional-minus decimal eras -14..=59, including0;
rejects malformed/duplicate/missing/out-of-range input before opening files.
Derivation: proleptic Gregorian year0..10000 contains leap centuries divisible by400.
From1900, year0 starts -693961days = -59958230400s; year10000 starts
2958464days =255611289600s. An era's span is [era*2^32,(era+1)*2^32).
Floor-dividing the calendar interval's first and last integer seconds gives
-14 and59 (boundary raw seconds171311744 and2208219135 respectively).
Every overlapping era is accepted; an individual out-of-calendar reading still
needs context. -15/60 and i64 extremes fail usage before input access.
The declaration applies to the whole input, independent of channel/reset.
Checked ntp::convert_ntp receives a declared context and no pivots. Its local
nonzero scope token is not an inferred native recorder session or published
session ID. No declaration reaches the API's era0 fallback for nonzero readings.
Coordinator-confirmed2026-11-22: only seconds AND fraction both zero are the
sentinel (owner-approved RDG-0122026-11-13). Era0+seconds0+nonzero fraction
legitimately dates1900-01-01; without an era even this fraction-only reading
needs context. The sentinel bypasses era resolution as RDG-012 requires; raw zero produces
TF-029 WARNING/no new reference even without the option. Existing eligible
counter references are not cleared by an unusable observation.

Conversion retains UTC without cumulative TAI-UTC, fraction floor and known
calendar year. Covered ambiguity keeps value with TF-030 WARNING and a per-
clock leap-ambiguous label; missing leap coverage keeps value explicitly uncertain
(no spurious PTP TF-021). Pre1972 dates use the RFC proleptic convention, not
physical UTC/leap reconstruction. Unsupported civil years return needs-context,
never clamp/panic or label the readable wire body malformed. Era affects no F1/
PTP conversion. CSV keeps its19 columns; native packet facts stay unchanged.

Schema v1 revision8 adds options.ntp_era as decimal string/null and three caller-
era basis values. The revision8 policy also records ntp_era_source as
caller-declared/none and ntp_era_inference as disabled, tied to the same option.
Labels survive live metadata, target brackets, own reset rows
and private segment spools; the old numeric-selection algorithm stays unchanged.

Tests-only6122ae1: three runtime failures (unrecognized option), while the no-
era/dated-F1 control passes. Five focused tests now cover six text/five JSON
views, raw status/NTF rejection, reset spooling, range/usage safety and no guessing.
Fixtures cite106-24R1 Fig11-15/16(11-27/PDF35), header11.2.1.1 and RFC5905 Fig4:
era0 2208988800->1970;2272060800->1972;3155587200->1999-12-31;
era1 63104->2036-02-08. Post2016 raw3692304000 maps UTC2017-01-02, not+37s.
Half fraction is500000000ns;FFFFFFFF floors999999999ns. Fraction-only raw1 is
not the zero sentinel even if it floors0ns. Final era1 second maps2172 day75.
Additional reset/status/boundary cases arrived with implementation, not falsely
claimed failed-first. No captured output/production encoder creates expectations.

Nine compiling mutants fail: ignored option; auto era0; declared era forced0;
fraction dropped; TF-029/030 codes lost; uncertain labelled covered; NTP basis
lost from retention; leap ambiguity labelled certain. Source restored. Local
fmt/focused tests/mutants/trace/schema-sync/diff only; platform/package/security
and32MiB RSS are CI evidence. 198 tested/206 verified of288 unchanged.
Items5..7 and the separate full-TMATS release/approval gate remain pending.

## Review delta

Tests-only cf75d3e has two runtime failures: -15 returned input rather than usage,
and policy lacked ntp_era_source. Four earlier controls pass. The new fraction-only
no-era and era0 distinction plus covered plain-basis row/query/reset-spool
assertions are in the same regression, with no whole requirement claims.

Six compiling review mutants fail: no-era guard ignores fraction-only input;
plain caller-era JSON mapping changed; NTP_DECLARED removed from known();
era-range guard removed; source enum reversed; inference enabled. All fail at
runtime, restored afterward. Endpoint fixtures date year0 day1 and9999 day365
using the above derived raw seconds; outside eras fail before file I/O.
