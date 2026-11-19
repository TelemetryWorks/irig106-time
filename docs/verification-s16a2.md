# S16a2 — host-callable CLI and injected output

Base main ac41df3. ADR-0007/0008; no time-library changes.
run accepts &[S] where S: AsRef<OsStr>, without argv[0]/host time, and returns ExitClass; only the
thin src/bin/irigtime.rs obtains stdio handles and exits. All six command/help
renderers use injected writers, propagating writes/flushes. CLI stays std-only.
To bound the review, Cargo temporarily targets existing src/main.rs as the CLI
library (autobins=false). S16a3 can rename it to lib.rs without rewriting a
second main at that path. All three roots retain forbid/deny policy probes.

Tests first a9db084: three host tests cannot compile before the CLI library
exists (E0432), not three reproduced runtime failures. After extraction: host
help/version matches the standalone version, write/flush failure returns4,
unknown command returns2, and failed input retains3 when diagnostics also fail.
Three runtime mutants are killed: output error mapped to success, omitted host
stdout flush, and diagnostic failure replacing input failure. All six S15 safety
regressions (Linux /dev/full included) and the two moved CLI BCD unit tests pass.
Local fmt, touched-crate Clippy, scanner/workspace tests and trace check pass;
full platform/feature/MSRV/WASM/deny/fuzz matrix is delegated to PR CI.

Partial chains only: L1-CLI-001 -> L2/L3-CLI-101 now has reusable library/thin
main but planned module boundaries remain; L1-CLI-002 -> L2/L3-CLI-102 has host
and writer controls, with per-command comparisons deferred. L1-CLI-003 ->
L2/L3-CLI-103 strict args and L1-CLI-007 -> L2/L3-CLI-107 full finding threshold,
usage-before-input, invalid-input and all-five-code precedence remain S16a3.
No whole markers added or tests lost; tested196/288, verified202/288 unchanged.
S16b bounded memory and qualified joining/reporting remain separate. README
memory/publication gate and publish=false remain. No Review/Design field changes.

## S16a2 review fixes — host input and native arguments

bf39b00 first reproduces the 24-byte Format 2/flag0x04 file panic at runtime,
caught by the host test's catch_unwind before its expected exit3 assertion.
5831ac6 adds Unix native-argv tests before the production change: both native
input/output filenames work in host and binary; non-UTF-8 command/option names
return2, including an option on a missing input before it can be opened.
The coordinator's pre-freeze decision is now in CLI-102 and ADR-0007.
run accepts strings or OsString/OsStr-compatible values without lossy path
conversion. Main collects args_os. Only display labels may replace invalid
Unicode; file access always receives the original Path. Other tmats work is
outside this repository and was not touched.

### Complete CLI packet slice/index inventory

| Site | Bound or fix |
| --- | --- |
| PktHeader::parse fixed byte indexes 0..21 | Existing len>=24 guard proves all fixed header fields readable. |
| scan remaining/packet selection | Checked remaining/packet slices. Length below the declared header size or not divisible by4 fails3 (§11.2.1.4 filler rule); Data Length past the declared body fails3. A plausible aligned final packet extending past EOF, or a partial final header, is a reported cut-tail finding. Recovered sync is a finding; no recoverable sync remains fatal3. |
| scan progress and counters | Each accepted packet is at least24 bytes, bounded by the remaining slice; offset+packet.len cannot overflow or exceed file length. Sync retry offset+1 is bounded by that same remaining-header proof. Counts are bounded by file length/24 in this single-pass private scanner. |
| PktHeader::payload | Checked get(data_offset..) then get(..Data Length); no addition of hostile lengths. u32 fits usize on supported32/64-bit targets. Trailer bytes cannot satisfy a short declared body. |
| process_time_packet CSDW and BCD | get(..4) proves indexes0..3. Constant body offset4 plus minimum6/8-byte BCD guards proves the remaining body slice; library decoder sees only declared data. |
| process_time_f2_packet | Replace unchecked data_start.. slice with checked payload. Structural header/body bounds are checked before decode; contained time-decode/context/correlation issues are reported findings, not fatal input errors. |
| find_next_sync fixed checksum accesses | get(start..) then windows(24); header[..22] is safe, chunks_exact(2) proves each word index0/1, window proves indexes22/23. checked_add returns the recovered offset. Existing word-sum candidate convention is retained, not a new standard checksum claim. |
| Other packet-path values/formatters | Arrays passed to RTC/CSDW are fixed after the bounds above; map entries use entry/get/iteration, resolved records use iteration, fmt_comma iterates bytes (i<len), no unchecked packet-derived indexes remain. |

The legacy flag interpretation (including0x04) and qualification/report model
remain S16b work; this audit repairs bounds, not those semantics. Empty input
retains the S15 interim behavior so CSV flush/write guards still run. Whole-file
allocation and the README memory/publication gate remain unchanged.

### Focused evidence

Nine packet/native-argv tests, four host tests, six S15 guards and the two CLI
BCD tests pass. Cases cover short headers, both time types with/without the
legacy secondary offset, undersized/max packet lengths, declared body bounds
with otherwise valid BCD/PTP bytes and trailer filler, and native filesystem
input/output. No new whole-requirement markers: coverage remains unchanged.
The success-path stderr-flush assertion kills its previously surviving mutant.
Additional runtime mutants killed: ignore Data Length, open a lossy path, and
use UTF-8 env::args in main. All mutations are restored before final checks.
Local fmt, focused tests/mutants, touched-crate Clippy and trace check only;
remaining checks use PR CI. Full strict grammar/threshold exits remain S16a3.

## Review follow-up — findings do not abort the report

The previous review fix incorrectly promoted every accumulated scanner message
to input3. Remove that blanket conversion: NTP needing era/basis, other contained
time-decode/correlation errors, reserved BCD bits and recovered sync loss stay
reported findings with success0 until S16a3 wires the threshold. Summary retains
its Errors section; the other five commands report messages on injected stderr,
leaving CSV parseable. An output failure while reporting still returns4.
The existing no-reference correlate behavior remains1; no new threshold is
invented here, and the mixed-reference tests retain two usable F1 references.

Coordinator decision: a truncated FINAL packet or partial header at EOF is a
finding, with the full preceding analysis retained. Validate minimum/aligned
packet length and Data Length against the declared body first; impossible
structure is still3. If a structurally plausible aligned packet extends beyond
remaining file bytes, stop with a tail finding, without reading or counting the
incomplete packet. No guessed maximum packet size is introduced. Zero length,
missing declared header, Data Length overrun and unrecoverable sync stay3.

Tests-first c5160a5: NTP-between-F1, recovered sync and reserved-bit tests each
fail at runtime on the prior head; the tail test also fails when replayed against
7433ba9's exact source. 9ec6746 updates old assertions to the coordinator's new
classification, rather than retaining fatal expectations for content/cut tails.
All four new scenarios exercise all six commands. Assert success AND a message,
complete-prefix packet/reference counts, report sections and CSV row counts.
The structural and catch_unwind panic regressions remain3. Both independent
silent-tail mutants (omit header finding / omit packet finding) fail at runtime.
All mutated/replayed source is restored. Local focused tests, fmt, touched CLI
Clippy and trace check only; full matrix delegated to CI. Coverage unchanged.
