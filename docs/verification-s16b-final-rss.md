# S16b final dense-F1 RSS gate

Base main c9508db (#74 merged). Partial L1-CLI-004 -> L2/L3-CLI-104 memory
clauses only; no whole-requirement marker or Review/Design changes. Matrix
stays 198 tested / 206 verified of 288. No Rust behavior/dependency changes.

## Workload and bounds

CI generates at least 384MiB, rounded to complete 28,224-byte cycles. A group
has one 36-byte F1 packet and nine 28-byte opaque non-time packets: 1/10 time
packets, not the old sparse 16KiB zero-data case. F1 Data Length10 excludes two
nonzero filler bytes; Ch11 §11.2.1.1 headers/checksums and Fig11-13 BCD DOY
words are derived directly. Header F1 version0x06 follows Table11-4/contract7;
opaque data version0x01 is not a claim about its PCM payload semantics.

Each 98-reference cycle includes one late packet, two consecutive late packets,
a >30s reset, a three-reference short reset, and two true +5s jumps. The cycle
boundary resets again. Calendar labels advance without an unknown-year rollover.
Every segment covers target RTC5s, forcing correlate's closed-answer spool.
Timeline uses explicit --limit1000; CSV uses --output to a file. No whole report
is loaded by the probe: bounded log prefix, streamed row counts and a CSV tail.
Exact warnings are pinned for each command, including withheld data, rather
than --fail-on never hiding malformed time input. Jump/segment/last-row counts
ensure that the work was not skipped. The sparse transport probe also stays.

All six child processes must have Linux wait4 peak RSS <65,536KiB. Cargo and
fixture generation are outside the measured command. The generator holds one
28KiB cycle, never the whole input. Commands run serially, each with a 180s
alarm; the CI step has a 20-minute cap. Input/report disk budget is checked
before generation. Owned temporary files are removed after the probe; the CSV
is deleted before the next command. No file is committed or generated locally
at full size. publish=false stays; publication is an S20 owner decision.

## Honest executed checks

Tests-first 4c3f1ad: one workflow assertion fails and two helper imports fail
because the gate/module do not exist. New small tests then pass: CRC/alignment,
1/10 ratio, literal day100/12:00 bytes, half-second late BCD, both +5s vectors,
reset counts and per-command warning totals. A two-cycle (56,448-byte) local
end-to-end smoke passes across six commands. It is NOT the release-scale RSS
result. Local fmt/trace/diff checks only beyond these focused tests; the full
384MiB workload and platform matrix run in GitHub CI.

Full-size CI evidence: pending. README warning remains until it passes.
