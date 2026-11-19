# S16b — bounded transport, joining and reports

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
