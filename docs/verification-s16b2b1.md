# S16b2b1 — CSV staging and atomic publication (partial CLI-104)

Base main c114171. Split for review: this PR removes CSV's packet-row vector
and implements approved transactional spooling. Next: bounded finding samples
with exact counts; streaming reference/jump/quality state; dense-F1 RSS gate.
README warning and publish=false remain until that gate and qualified joining
are complete. No library I/O, dependency, unsafe, publication or tag.

## Decisions and operational order

Coordinator: CSV files spool in the SAME directory, then atomic rename on
success; stdout spools in system temp and copies only after analysis. "On any
failure, delete the temp file and leave any existing destination untouched."
File publication therefore occurs only on exit 0, not findings 1. Stdout still
renders a completed report returning 1. fail-on never permits file export with
findings; input/output failures cannot be disabled. Usage precedes input open;
spool creation/writes may then fail output 4 before later packet analysis. The
first fatal operational failure wins, including temporary-storage errors.

File flow: exclusive/private temp, bounded buffered row writes, analysis,
spool flush, diagnostics, both injected host flushes, threshold, close and
rename. No fallible write/flush follows successful rename. Reject non-regular
file destinations (including devices/symlinks); use stdout for streams. Unix
replacement files retain the private 0600 temp mode. Every normal error return
keeps cleanup ownership; successful rename releases the old temp name.
Drop closes before unlink, including Windows. OS-denied unlink cannot be
promised successful; cleanup is attempted without replacing the original error.

## Clause evidence and honest limits

L1-CLI-004 -> L2/L3-CLI-104: CSV row staging only, no retained row vector.
L1-CLI-007 -> L2/L3-CLI-107: transaction/first-fatal/flush clauses only.
No whole-requirement markers: totals remain 198 tested / 206 verified of 288.
Existing CLI-107 aggregate still passes. No Review/Design field changed.
Reference and finding collections are STILL unbounded; the existing non-time
RSS workload is insufficient to lift the release gate. Disk use grows with
CSV output. Qualified time/setup/report rebuild is not claimed here.

Tests-first c3e3f75: three runtime failures, two controls pass on main. Added
later: rename/copy failure cleanup, post-publication name ownership, row-sink
retention and broken stdout-pipe controls (not claimed failing-first).
Six integration tests + four focused unit controls pass: exits 1–4 rollback,
output/diagnostic flush failure, valid replacement/hard-link preservation,
missing stdout temp directory, isolated stdout temp success/input failure,
rename failure, broken-copy/pipe cleanup, same-directory placement and no
row retention. Windows/macOS now run CLI transaction tests in existing OS jobs; Ubuntu
already does. Five runtime mutants killed: omitted cleanup; publish on
findings; publish before host flushes; wrong temp directory; retain CSV rows.
Sources restored. Local fmt, focused/touched CLI tests and Clippy only; full
matrix/RSS via PR CI. Physical temp-disk exhaustion is not injected locally.

CI 44526050932 caught a race in the tiny broken-pipe fixture (child could finish
before parent closed the reader); use 32768 packets so output exceeds pipe
capacity. The broad macOS CLI step also exposed a pre-existing invalid-byte
filename fixture rejected by APFS. Scope the new cross-platform step to the
CSV transaction targets; Ubuntu retains the full CLI suite. No production fix
or unrelated filesystem-test change. The revised pipe fixture passes locally.
