# S16a2 — host-callable CLI and injected output

Base main ac41df3. ADR-0007/0008; no time-library or reader-algorithm changes.
run accepts &[&str] without argv[0]/host time and returns ExitClass; only the
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
