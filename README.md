# irig106-time workspace

IRIG 106-24R1 time parsing, qualified conversion and correlation, with the `irigtime` inspection CLI.
The rebuild targets 0.8.0; published 0.1–0.7 versions have known errors. See the
[library README](crates/irig106-time/README.md) and [standard review](docs/STANDARD-REVIEW.md).

| Package | Location | Purpose |
| --- | --- | --- |
| `irig106-time` | [crates/irig106-time](crates/irig106-time) | `no_std` + `alloc` library; optional std/Jiff |
| `irig106-time-cli` | [apps/irig106-time-cli](apps/irig106-time-cli) | Testing and simple examples; library runner and `irigtime` binary |

The root is a virtual Cargo workspace, with no package. Both packages inherit
one version from `[workspace.package]`; the CLI inherits the exact library pin
from `[workspace.dependencies]`. One root lockfile covers the members. Root
`fuzz/` remains an independent excluded workspace with its own lockfile; Cargo
benchmarks and examples live with the library. [ADR-0022](docs/adr/0022-virtual-workspace-layout.md)
records the owner's 2026-11-21 layout decision.

```sh
cargo test -p irig106-time --all-features
cargo test -p irig106-time-cli --locked
cargo run -p irig106-time-cli --bin irigtime -- --help
cargo bench -p irig106-time --bench correlation_bench
```

See [contributing](CONTRIBUTING.md), [CLI usage](apps/irig106-time-cli/README.md),
[architecture](docs/architecture.md), [trace matrix](docs/TRACE-MATRIX.md), and
[release prerequisites](docs/RELEASING.md). Library JSON/serde is removed for
0.8.0; report JSON belongs to the CLI. No library I/O or TMATS dependency. See the [current usage guide](docs/usage.md),
[0.8 migration](docs/MIGRATING-0.8.md), [S17 completion ledger](docs/verification-s17-completion.md)
and [explicit follow-ups](docs/S17-deferred-followups.md).

**Release gate:** 0.8.0 waits for the full `irig106-tmats` 0.1 release and a
bounded time view, with separate owner approval before any TMATS dependency.
The CLI remains `publish = false`; no release/tag/publication is authorized by
this documentation update. Apache-2.0, with a [license](LICENSE) in each package.
