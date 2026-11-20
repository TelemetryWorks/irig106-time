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
Coordinator 2026-11-20 clarifies that findings are not publication failure:
file and stdout reports publish on exits 0 AND 1; only exits 2–4 withhold file
publication. This supersedes the initial exit-1 rollback interpretation.
Input/output failures cannot be disabled. Usage precedes input open;
spool creation/writes may then fail output 4 before later packet analysis. The
first fatal operational failure wins, including temporary-storage errors.

File flow: exclusive/private temp, bounded buffered row writes, analysis,
spool flush, diagnostics, both injected host flushes, threshold, close and
rename. No fallible write/flush follows successful rename. Reject non-regular
file destinations (including devices/symlinks); use stdout for streams. Unix
report temps stay private while written. Before rename, copy existing target
permissions or normal new-file defaults (Unix 0666 masked by umask; Windows
defaults). New inode: other hard links keep old contents. Copying std Permissions
preserves Unix mode/Windows readonly state, not ownership/other inode metadata.
Every normal error return
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
The original six integration/five unit controls included exit 1 rollback;
that case is now withdrawn. Original five mutants included publishing on
findings, which is now accepted behaviour. Other original evidence remains
historical, not a claim that those mutants were all rerun for this delta.
Windows/macOS run transaction targets; Ubuntu retains the full CLI suite.
Sources restored. Local fmt, focused/touched CLI tests and Clippy only; full
matrix/RSS via PR CI. Linux /dev/full writer injection covers direct-write and buffered-flush failures
with cleanup; actual filesystem exhaustion is not forced.

CI 44526050932 caught a race in the tiny broken-pipe fixture (child could finish
before parent closed the reader); use 32768 packets so output exceeds pipe
capacity. The broad macOS CLI step also exposed a pre-existing invalid-byte
filename fixture rejected by APFS. Scope the new cross-platform step to the
CSV transaction targets; Ubuntu retains the full CLI suite. No production fix
or unrelated filesystem-test change. The revised pipe fixture passes locally.

## PR #68 coordinator delta (2026-11-20)

Tests-first 008559a: findings publication, existing-mode transfer and new-file
umask policy fail at runtime before the fix (three failures, eight controls).
Current focused Linux suite passes: 11 CLI unit tests plus 12 transaction
integration tests. Additional collision/privacy/nonce tests arrived with the
implementation; no failing-first claim for them. No whole requirement markers.

- Candidate injection in the private factory deterministically plants the next
  regular-file and symlink names; each remains untouched and a fresh name wins.
  Production names use RandomState suffix + pid/counter, no new dependency.
- Private temp mode checked under child umask 022 (host 077 previously masked a
  weak 0666 mode); published new mode checked under child umask 027 -> 0640;
  existing target mode 0604 is copied. Parent process umask is never mutated.
- New-file defaults come from an EMPTY, exclusive 0666 permission probe in the
  same directory, immediately cleaned up. std has no safe umask getter; no
  unsafe/global umask manipulation. Actual report bytes stay private until
  permission transfer after analysis and host flushes, before final rename.
- Symlink and /dev/null targets give 4 unchanged. Non-regular checks run both
  before staging and before publication. Guard mutations run ONLY against an
  owned symlink fixture, never against a device. Missing output directory plus
  readable-but-structurally-bad input gives 4: temp creation fails before framing.
- Eight runtime mutants killed: create vs create_new (file and symlink cases),
  weak temp mode, missing existing-mode copy, private new-file default, missing
  non-regular guards, constant suffix, and withholding findings publication.
  Sources restored; local fmt/targeted tests/Clippy and trace check pass.

README and cli_commands now document CSV's interleaved staging/analysis order.
Dense-F1 RSS and bounded findings/reference/jump/quality state remain later
slices; interim warning and publication gate stay. No Review field changes.
