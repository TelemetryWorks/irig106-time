# S16b — bounded transport, joining and reports

S16b1 stage record; subsequent command integration: [S16b2a](verification-s16b2a.md).

Base main 212a962. Reviewable split (about 600 changed lines per PR):
1. S16b1 (this slice): bounded raw transport, small-buffer file test and CI
   transport RSS probe. No command integration yet.
2. S16b2: approved F1/F2/own-time qualification and setup-range joining,
   dedicated reserved-NTF error, all commands using one validated loop.
3. S16b3 (split further if needed): shared report/renderers, end-to-end memory
   and mixed-recording exit/provenance checks, remove interim warning only
   after the command release gate is demonstrated.

## Clause ledger

| Lineage | Covered here | Remaining |
| --- | --- | --- |
| L1-CLI-004 -> L2/L3-CLI-104 | Buffered raw input, caller buffer/capture caps, validated lengths, data-only prefix, EOF/recovery events | Every command's shared joining/report model, approved conversion, TF-001, multiple setup ranges, sessions/quality/basis, full CLI RSS |
| L1-CLI-007 -> L2/L3-CLI-107 | Raw distinctions preserve the inputs needed for existing severity/exit policy | Integrate events with fail-on, analyse before render, keep existing regression suite |

No whole-requirement Requirements marker is added. Trace totals remain
198/288 tested and 206/288 verified. No Review or Design field is changed.
The CLI STILL calls its old whole-file scanner; README's interim warning and
publish=false stay. This is not closure of the bounded-memory release gate.
No library I/O, dependency, mmap, unsafe, CLI publish or tag is introduced.

## Transport contract and evidence

106-24R1 §11.2.1.1 supplies header layout, secondary-presence bit 7, lengths
and primary word checksum. Test header bytes/checksums are hand-packed from
those fields, independent of the reader. No time value is decoded in this
slice. A callback requests a bounded data prefix; it is clamped to Data Length,
then checked against the caller cap. Uncaptured data, filler and packet-data
checksum bytes are drained through the fixed buffer; they are never exposed
as data. Prefix completeness must be checked before a full-body decoder.

Packet Length never sizes an allocation. Live reader state is a buffer,
a 24-byte window, a bounded capture and fixed metadata. It retains no packet
history. Returned packets belong to the caller. Resync is a sliding fixed-size
checksum-qualified window. Real read errors propagate; framing/cap errors
terminate the reader. Header bit 2 does NOT imply secondary-header presence.

Coordinator EOF exception retained: short final header/packet is a warning;
unsynced padding after an established stream is information; no established
stream with no recoverable sync is fatal input. Invalid aligned/minimum length
or Data Length past the body is fatal. A declared packet crossing EOF is a
truncated final packet: bytes swallowed within its claimed body are lost but
counted/reported, not rescanned. Severity rendering is not wired here yet.

Tests-first b6237d5 fails to compile on main: missing input API, NOT a runtime
failure. Three focused tests pass after implementation; the ignored RSS worker
runs explicitly in CI. The small-buffer test processes an actual 3200-byte file
through a 32-byte buffer with three-byte short reads and excludes filler.
Additional impossible-length fixtures arrived with implementation. Five
one-at-a-time runtime mutants fail the named focused tests: omit Data Length
clamp; use bit 2 for secondary; remove capture cap; allow no-stream as padding;
remove Data Length bound. Sources restored and focused tests rerun green.

CI generates 384 MiB of valid 16 KiB frames (never committed), invokes the
ignored worker with capture=0, and measures that worker's Linux wait4 peak RSS
only, excluding Cargo/compiler/generator. Limit: 64 MiB; counts/offsets must
match the file. RSS and scope are logged and added to the job summary. This is
transport-only; no claim about the old CLI's RSS or report-state growth.
Locally: fmt, focused reader tests/mutants and touched-crate Clippy only.

## Review follow-up: standard framing, intentional fixes

Coordinator decision (2026-11-19): follow 106-24R1 Ch11 §11.2.1.1.
Every packet header requires a valid primary checksum, including at the
expected offset. A bad checksum is sync loss: resync and report a finding
with the recovered byte count. No valid header anywhere means not Chapter 10
input (exit 3). Bit 7 means a secondary header follows. Ignoring checksums and
using bit 2 for presence were bugs in the old scanner; these are intentional
behaviour fixes, not compatibility options. This slice exposes recovered_bytes
as the finding evidence; S16b2 still owns command grading/report integration
and file-level classification (including a file with no complete valid header).
The established-stream cut-final/tail exception remains as documented above.

S16b2 MUST regenerate s16_packet_bounds.rs::packet and other legacy packet
fixtures with correct primary checksums and bit 7 for secondary presence.
Keep bits 3-2 as the independent time-format field. Recompute checksums after
mutating structural fields when testing those fields, so checksum rejection
cannot hide the intended assertion. Preserve content-finding vs structural
intent; do not turn short time bodies or established-stream EOF tails into
fatal input simply because their old fixture checksum/presence flag was wrong.

Two new tests pass without production changes: a bad-checksum frame at the
expected offset between valid frames is skipped, with exactly 28 recovered
bytes and the following frame's data; the isolated bad frame is InvalidData.
The second fixture declares length 26, Data Length 0, and a correct checksum
(EB25 + 001A + 0900 = F43F): alignment alone yields InvalidData.
One-at-a-time runtime mutants ignoring checksum / alignment respectively
fail those exact tests. Sources restored; focused suite: 5 pass, 1 CI worker
ignored locally. No new whole-requirement marker or coverage increase.
