# S15b — CLI root and I/O boundary safety

Depends on S15a/PR57. Main base57a46e1; the branch includes S15a so CI can run
against main. No complete CLI-restructure/exit-code conformance claim.

## Failure-first evidence and fixes

86715f4 adds four subprocess controls. All fail before implementation: malformed
hex RTC, u64::MAX threshold-ms and an unwritable CSV path panic with101;
missing input reports1 rather than the accepted input class3. They now pass:
usage/range errors2; input3; output4. These implement already accepted classes,
not new standard rules (IRIG106 is silent on CLI codes). S16 still owns the
complete parser precedence, naming, invalid-input and output/exit contract.

- CLI root forbids unsafe and denies all nine accepted Clippy operations.
  Remove its only unsafe block/file mmap and the memmap2 dependency; lockfile
  removes only memmap2/libc. File input becomes a safe owned byte snapshot.
- This deliberately changes memory/latency: file bytes are read into RAM up
  front rather than faulted from a mapping; no zero-copy claim remains.
  No invented file-size cap or new command-line policy. Streaming/capped CLI
  input remains part of the forthcoming CLI work, not a new release deferral.
- CSV creation/write/flush errors propagate as io::Result and class4; invalid
  hex and checked threshold multiplication become class2, not panic/default.
  Input failure now says cannot read and class3. Successful output unchanged.
- Only print_stdout/print_stderr are allowed on nine named CLI rendering/
  diagnostic functions in main.rs, and only the channel each uses. No parsing/
  scanning allowance, no panic/unwrap/expect exception in production. Test
  assertions allow unwrap only on the cfg(test) unit module. No crate allowance.
- Main/output helpers are the CLI's I/O boundary, separate from the library;
  rendering still uses standard print macros. This does not prove all hidden
  std allocation/printing failures are recoverable; full rendering is S16.

The policy script now checks both real roots: positive control, unsafe block,
child reallow and nine operations =24 expected results, all observed. Root
forbid applies even to unit tests; dependency unsafe is separately assessed,
not claimed removed. CI now executes CLI tests, not merely builds/lints them.
Locally only fmt, CLI all-target Clippy, four focused subprocess tests and
negative policy checks; full target/feature/deny matrix belongs to PR CI.

## Requirement clauses

L1-API-009 → L2/L3-API-109 roots and unsafe negative guard are now enforced
in both project crates; final source/dependency inventory remains S15c.
L1-API-012 → L2/L3-API-112 library policy from S15a plus CLI root policy and
narrow I/O allowances. L1-ERR-001 → L2/L3-ERR-101 library work remains partial;
these CLI controls are not evidence of every library parser being panic-free.
No whole markers;196/288. Traits/allocation/no-I/O audit continues in S15c.

## PR58 review — buffered write failure and main integration

Linux-only csv_flush_failure_is_an_output_error_not_silent_success writes CSV
to /dev/full and requires class4. It passes before any production change;
removing out.flush()? compiles, exits0 and fails the new assertion. Restored.
Ordinarily merged mainc809708, retaining all S15 CHANGELOG sections. CLI README
now warns whole-file input may return3 or be killed by the OS; no CLI publication
or release tag before S16 bounded streaming and larger-than-buffer verification.
