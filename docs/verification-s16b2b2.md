# S16b2b2 — bounded finding samples and writable CSV targets

Base main acd6fde. Next reviewable slice: first 20 generated finding messages,
exact per-level counts and a final omitted-count line. 20 reuses the old summary
sample limit, now shared by all commands. Every severity affects --fail-on,
not just sampled severities. Derived jump/query findings enter the counts;
jumps are counted per occurrence, not as one aggregate warning. Existing TF
levels and the legacy summary Errors(N) heading stay; N is now the full total.

## Clause coverage / bounds

L1-CLI-004 -> L2/L3-CLI-104: diagnostic sample/report-memory clause only.
L1-CLI-007 -> L2/L3-CLI-107: unsampled severities and operational-error clauses.
No whole-requirement test markers; coverage remains 198 tested / 206 verified
of 288. No Review/Design changes. Correlation references and detect_time_jump's
vector remain unbounded; streaming jump/quality work and dense-F1 RSS follow.
README warning/publish=false remain. No library I/O, dependency, unsafe or tag.

u64 counts do not saturate. A complete frame takes at least 24 bytes, and the
reader checks u64 byte offsets. Current generation emits at most 3 findings
per packet (sync, decode, derived jump) plus terminal/query facts, below the
u64 count limit. Messages use fixed templates/numeric wire fields, not retained
raw payloads. Sampling is global encounter order; 20 strings do not grow with
file length. Full diagnostic counts still include messages whose text is omitted.

## Read-only decision from PR #68 review

Reject existing read-only targets (including 444) even for a privileged caller.
For other regular files, ask the OS for effective-user/ACL write access with a
non-truncating write-only open. Check before staging and again before rename;
never modify old bytes or permissions just to publish. Non-regular rejection
stays. This preserves expected write checks instead of bypassing them via
writable-directory rename. No change to exit 0/1 publication for writable files.

## Executed evidence

Tests-first 3712f40: two runtime failures on main: 38 retained samples instead
of 20, and readonly 444 output returned 0 instead of 4. After implementation:
- Direct notes prove exact 1 error/30 warning/7 info counts, first 20 order and
  omitted 18; late error is not sampled but raises highest severity.
- All six commands see literal Fig11-14 frames with 1003 reserved-bit warnings,
  one late malformed-digit error and one info tail: total 1005, omitted 985,
  bounded diagnostics, exit 1 at error and 0 with never. Header checksums remain
  valid: only body bits/digits change. One valid reference avoids context noise.
- Summary/jumps report two actual jumps as two warnings; correlate counts both
  TF-001 and its unresolved-query context warning. These tests arrived with the
  implementation, not claimed failing on main.
- Unix mode 444 regression preserves old contents, inode and permissions and
  returns 4 with no temp leak. Existing CSV permission/cleanup tests still pass.
- Eight runtime mutants killed: unbounded samples; sampled-only counts;
  sampled-only severities; missing omitted count; last instead of first samples;
  readonly rename bypass; summary/jumps each collapsing jump counts. Sources
  restored, focused suite rerun green. Local fmt/CLI tests/mutants/Clippy and
  trace check only; large RSS and remaining matrix are delegated to PR CI.

## PR #69 delta review — readonly recheck and probe hardening

81c9f43 adds the requested tests before hardening: a Unix target is staged,
then chmod 444, then commit must fail preserving bytes/inode/mode and removing
the spool. The existing implementation passes; bypassing commit's permission
recheck fails at runtime. A cfg(windows) host regression marks the destination
readonly and requires output 4 with unchanged content/attribute/no temp. It is
not claimed run on Linux; the owned fixture flag is reset only after observing
the result, for cleanup.

Linux/macOS write probes now use O_NONBLOCK|O_NOFOLLOW without truncation;
Unix compares (dev,ino) from lstat to the opened handle and rejects mismatches.
Other Unix targets retain the identity comparison. Only supported matrix
platforms are claimed nonblocking here. Constants verified read-only against
[Linux UAPI](https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/fcntl.h)
and arch/{arm,arm64,powerpc,m68k,mips,sparc}/include/uapi/asm/fcntl.h overrides,
and [Darwin](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/fcntl.h).
The ARM nofollow override was caught by the local aarch64 symlink regression;
no assumption that generic flags work on every Linux architecture.

Deterministic tests swap the lstat-observed regular file for another inode,
a symlink to the saved original, or a FIFO. Four runtime mutants killed:
commit check skipped; identity check ignored; NOFOLLOW omitted; NONBLOCK
omitted. The FIFO omission is contained in an owned child, killed after a
bounded 5s wait, with parent-owned directory cleanup. No live device mutation.
Sources restored: 11 spool units and 13 transaction integrations pass locally;
Windows test awaits CI. These are partial CLI-104/107 clauses only, no whole
marker/coverage increase or Review/Design change. Local fmt, focused tests/
mutants, Clippy and trace check; remaining matrix is CI. No new dependency,
unsafe code or library I/O. The release gate still stays closed.
