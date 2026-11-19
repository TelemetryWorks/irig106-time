# S16a3a — strict argument preflight and conventional library target

Base main f43970d. S16a3 is split to stay within the review budget: this PR
closes args/preflight and renames src/main.rs to src/lib.rs; the separately
prepared threshold/five-exit suite is S16a3b, not claimed here. No fail-on flag
is accepted until it is wired. Content/EOF-tail/padding findings retain #63's
behavior; only its structural failures return3. Reader and memory gate unchanged.

## Args inspection

L1-CLI-003 -> L2/L3-CLI-103.

ADR-0008: public args::parse performs no I/O, environment access or process
exit; std collections, native strings and paths only. Manifest inspection:
no parser dependency, only the existing exact time-library pin. All syntax is
resolved before dispatch reaches fs::read. No lossy filesystem path conversion.

Six commands, help/version (global or command --help), exact positional counts,
unknown/inapplicable/duplicate options and missing values are checked. Values
are separate tokens, not --option=value. -- makes all remaining tokens literal.
threshold-ms is unsigned ASCII decimal in0..=18446744073709, checked in ns;
its retained default is1000ms. limit is positive decimal fitting usize, default100.
RTC is unsigned hex, optional0x/0X, at most48 bits; no masking of excess bits.
Native non-UTF-8 paths remain supported; command/option/numeric syntax rejects
invalid Unicode with2. This is CLI grammar/policy, not an IRIG wire mandate.

85afcfc is tests first: malformed threshold on a missing input returns3 on
main instead of2. The strict usage table now passes before input access.
Two later positive/control tests cover all commands/options, numeric boundaries,
help, missing arguments and a real dash-leading/space-containing filename via
-- in a child process (without changing host cwd). These controls were added
with the implementation, not claimed as runtime failures on main. Five runtime
mutants killed: missing threshold overflow check, zero limit accepted, duplicate
option accepted, unchecked48-bit RTC and ignored terminator. All source restored.

The existing BCD/host/panic/flush/native-argv/recovered-sync/tail/padding tests
pass unchanged. Policy/scanner roots now include CLI lib.rs and thin binary;
trace roots and marker paths are unchanged, with no lost tests. Local fmt,
focused tests/mutants, touched CLI Clippy and policy/scanner checks only; the
remaining matrix runs in PR CI. No Review/Design field changes.

Only CLI-103 inspection closes: tested196/288 unchanged; verified204/288 (+2).
CLI-101/102 module/report/interface clauses and CLI-107's selected threshold,
all five exit classes and their precedence remain explicitly for later slices.
S16a3b work is saved separately; no complete exit-contract claim in this PR.
