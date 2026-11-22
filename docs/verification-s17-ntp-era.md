# S17 item4 — explicit caller NTP era

Base maindba6a1c(#95). Partial CLI-103/106/107 and RDG-012 adapter evidence,
no whole Requirements marker/Review/Design or dependency/library change.
The coordinator decision is explicit --ntp-era N, labelled caller-declared era;
without it nonzero NTP stays needs-context, no automatic pivot/era0/host clock.
The parenthetical2036–2104 example is a half-era interval, not a full era:
RFC5905 §6/RDG-012/library arithmetic uses2^32 seconds (~136years), so era1
continues to2172-03-15 12:56:32. This clarification does not implement pivots.

Strict OsString parsing accepts optional-minus decimal i64 values, including0;
rejects malformed/duplicate/missing/out-of-i64-range input before opening files.
The declaration applies to the whole input, independent of channel/reset.
Checked ntp::convert_ntp receives a declared context and no pivots. Its local
nonzero scope token is not an inferred native recorder session or published
session ID. No declaration reaches the API's era0 fallback for nonzero readings.
The sentinel bypasses era resolution as RDG-012 requires; raw zero produces
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
era basis values. Labels survive live metadata, target brackets, own reset rows
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
