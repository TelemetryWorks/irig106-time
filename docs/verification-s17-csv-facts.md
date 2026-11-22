# S17 CSV packet facts — approved append-only contract

Base maine5a0bfa(#92 merged). The coordinator approved four APPENDED columns:
network_time_format,network_protocol,time_status,time_status_name. Names, order
and meanings of the original15 stay unchanged; CSV now has19 fields. Future
columns append only, never reorder/remove. Status names are exactly Valid,
NotValid and RSVD-n. These describe the PACKET, not the resolved clock's source;
non-F2/missing complete CSDW leaves all four empty. No new dependency or finding,
Review/Design change, JSON/schema revision or numeric correlation change.

## Tests-first / independent fixtures

Tests-onlycb6e012 has two runtime failures on main: width15 instead of19 and
missing appended fields in the full golden. Both pass after implementation.
[Golden](../apps/irig106-time-cli/tests/golden/s17_packet_facts.csv) pins the
header and every byte of seven rows. Figures11-12/13 supply FMT4/SRC1, unknown
year, DOY10012:00. Figures11-15/17 (printed11-27/PDF35, previously page-checked)
supply NTF1/2/reserved15 and TS0/1/reserved15. PTP2017-01-02:
Unix1483315200 +37 =1483315237 TAI; RTC1s=10000000 ticks. Later invalid/malformed
rows use that preceding valid reference's clock, not their own raw body999s;
packet protocol/status still describe their own CSDW. New fixtures use independent
primary-header checksums/Data Length per11.2.1.1, no encoder or captured output.

The golden includes F1, valid PTP, invalid TS, reserved NTF/TS, a two-byte CSDW,
a complete four-byte CSDW with no body, and PCM. The short CSDW remains empty
without consuming filler; the missing body retains its observed CSDW facts.
All256 NTF/TS pairs pin exact raw fields, all protocol/status names, column count
and the independent original15-field prefix. Zero selectors remain values,
not empty fields. Existing packet/stale/reset/adapter goldens append the approved
fields; calendar values and original positions remain unchanged.

Seven compiling focused mutants are killed: raw NTF0; wrong PTP2002 protocol;
raw TS1; wrong NotValid spelling; reserved status fixed2; omitted empty column;
accepting a one-byte CSDW. Each fails a runtime assertion; source restored.

## Bounds and clause status

CSV writes each field directly into the existing exclusive spool; no row list,
JSON projection or new per-file state. Atomic output/cleanup/permissions and
exit precedence are unchanged. Existing failed-writer and dense-F1 RSS gates
remain in CI. Local checks are fmt, focused CSV/report tests and mutants,
schema-sync/trace/diff checks only; no full local matrix or large workload.

Partial L1-CLI-005 -> L2/L3-CLI-105 protocol/status and L1-CLI-006 ->
L2/L3-CLI-106 column/absence clauses only. No whole Requirements marker;
198 tested/206 verified of288 unchanged. Remaining ordered work: F1 packet/query
scale+basis, policy/default identities, explicit NTP era, automated schema
validation, native session/quality and docs/migration. TMATS declarations remain
separately gated; this PR establishes no data/time-channel association. The
approved CSV contract alone does not close these broader requirements.

An implementation-side injected writer fails once at every appended-byte
boundary (including empty fields/newline), then recovers. Errors still propagate.
Two additional compiling mutants that discard the fact-write or empty-field
write error fail this unit test. Nine mutants total; source restored.
