# S16b final dense-F1 RSS gate

Base main c9508db (#74 merged). Partial L1-CLI-004 -> L2/L3-CLI-104 memory
clauses only; no whole-requirement marker or Review/Design changes. Matrix
stays 198 tested / 206 verified of 288. No production Rust behavior/dependency changes.

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

All six commands in both phases must have GNU time CLI peak RSS <32,768KiB. Cargo and
fixture generation are outside the measured command. The generator holds one
28KiB cycle, never the whole input. Commands run serially, each with a 180s
alarm; the CI step has a 20-minute cap. Input/report disk budget is checked
before generation. Owned temporary files are removed after the probe; the CSV
is deleted before the next command. No file is committed or generated locally
at full size. publish=false stays; publication is an S20 owner decision.

## Honest executed checks

Tests-first 4c3f1ad: one workflow assertion fails and two helper imports fail
because the gate/module do not exist. New small tests then pass: checksum/alignment,
1/10 ratio, literal day100/12:00 bytes, half-second late BCD, both +5s vectors,
reset counts and per-command warning totals. A two-cycle (56,448-byte) local
end-to-end smoke passes across six commands. It is NOT the release-scale RSS
result. Local fmt/trace/diff checks only beyond these focused tests; the full
384MiB workload and platform matrix run in GitHub CI.

## Review correction: isolated RSS and a discriminating ceiling

The initial wait4 values in runs 44845372998 / 44846090901 (14,688–14,876KiB)
were harness-inclusive upper bounds, not true CLI peaks: the Python fork's
high-water mark survives exec. The 64MiB ceiling also admitted the reviewer's
compact all-reference Vec (44.6MiB). Those results do NOT establish the stronger
gate. Replace wait4 with GNU /usr/bin/time -f %M: its small exec image forks the
CLI, so Python's high-water mark cannot dominate the measurement. Require
<32MiB, no fallback. Timeout kills the owned process group (launcher and CLI).

Keep the original reset/late workload unchanged. Add a second, reset-free
115,200,000-byte phase: 400,000 F1 references / 4,000,000 packets on four time
channels, 100Hz per channel, 1000s. Each channel supplies 3000 refs per 30s;
the shared 1024 cap fills before age eviction, without resets clearing it.
One F1 / nine data packets remains; expected findings/jumps are zero. Run all
six commands on both phases, with timeline --limit1000 and CSV to a file.
A few KiB of generation state is retained; input phases are generated/deleted
serially. The 32MiB RSS cap applies independently to all twelve invocations.

Tests-first 1e5154e: the new reset-free import and isolated-measurement assertion
fail before implementation. Five small tests pass afterward. Wire tests pin
per-channel monotonic RTCs, channel count, valid checksums and literal 0.01s BCD.
No new whole-requirement marker. No large workload was run locally.

## Corrected executed evidence

[CI run 44847361574](https://github.com/TelemetryWorks/irig106-time/actions/runs/44847361574)
at b8fc36b7b1d3c06e1c2b137faf8ec7a114773451 passed the full matrix and both
phases. These are GNU time CLI peaks, not harness-inclusive upper bounds:

| Command | Reset phase KiB | Reset-free phase KiB | Limit KiB |
| --- | ---: | ---: | ---: |
| summary | 2,968 | 3,368 | <32,768 |
| channels | 2,984 | 3,272 | <32,768 |
| jumps | 2,984 | 3,272 | <32,768 |
| timeline --limit 1000 | 3,224 | 3,380 | <32,768 |
| csv --output report.csv | 3,140 | 3,392 | <32,768 |
| correlate (RTC 5s) | 3,012 | 3,344 | <32,768 |

Reset phase: 402,671,808 bytes, 13,981,660 packets / 1,398,166 F1,
42,800 resets, 42,801 late observations, 28,534 real jumps. Reset-free phase:
115,200,000 bytes, 4,000,000 packets / 400,000 F1, no findings or jumps.
The steady summary/correlate assertions require all four time channels.
The first revised CI run 44847118115 passed reset measurements, then failed a
harness assertion expecting “Total: 0” instead of the CLI's “None detected”;
correcting that assertion changed no production behavior.

### One-off negative control (not part of regular CI)

[Run 44847154990](https://github.com/TelemetryWorks/irig106-time/actions/runs/44847154990)
at throwaway commit 4e1fee371d21a48e924de4ade6a1b4996b259c9c injected a
scanner-owned Vec<[u8;32]>, retaining one copy of the raw F1 payload padded to
32 bytes on every observation, with black_box keeping it live. It retained all
1,398,166 references without changing normal bounded-cache answers. Summary's
exit/finding/output checks passed, then its true CLI peak **46,692KiB** failed
the **32,768KiB** RSS assertion. This equivalent compact all-reference leak
would pass the former 64MiB ceiling. The gate stopped at the first rejection;
no repeated full mutant campaign or mutation in regular CI. The throwaway
branch was deleted; neither mutant nor one-off workflow is in this PR.

Local checks: five focused Python tests, fmt and diff checks. Large workloads
and the negative control ran only in GitHub CI. The README memory warning is
removed on this corrected evidence; publish=false stays until S20. Coverage
remains 198 tested / 206 verified of 288; S17 qualified joining stays separate.
