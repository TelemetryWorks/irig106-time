---
status: accepted
date: 2026-09-27
decision-makers: Joey
---

# One workspace in lockstep: the library and `irig106-time-cli`, binary `irigtime`, mounted as `irig106 time`

## Context and Problem Statement

`irig106-time-cli/` today has its own manifest, is not a workspace member,
is `publish = false`, and is one 996-line `src/main.rs`, so nothing in it can
be reused (ROADMAP P6-10). The owner wants its commands in the ecosystem's
`irig106-cli`: "We need a irig106-time-cli sub-crate as well like we have on
tmats. This will be used in the irig106-cli project in future development"
(2026-09-26).

## Decision Drivers

* No duplicated command code between `irigtime` and `irig106 time`
* One version for the library and its CLI, so the CLI always reports what
  the library it ships with does
* A standalone binary name that does not clash with the shell

## Considered Options

* **One workspace, two crates in lockstep; the CLI a library and a binary**
* A CLI kept private, re-implemented in `irig106-cli`
* The commands folded into the library behind a feature

## Decision Outcome

Chosen option: **one workspace, two crates in lockstep**, as
`irig106-tmats` did (its ADR-0011):

* **`irig106-time`**, the library, and **`irig106-time-cli`**, published
  together with one version and one tag; the CLI pins the library with
  `=X.Y.Z`.
* **`irig106-time-cli` is a library and a binary.** The library holds
  argument parsing, input (the packet reader until `irig106-core`),
  commands, the report model, and renderers, with a `run` entry point; the
  binary is a thin `main`.
* **The binary is `irigtime`** — the owner's decision, 2026-09-26: "give the
  standalone binary a distinct name (for example irigtime) while irig106-cli
  still mounts it as irig106 time", and 2026-09-27: "CLI sub-crate should be
  called irig106-time-cli and the command in the built binary should be
  irigtime and when used in irig106-cli it will be `time` sub-command". A
  standalone `time` would clash with the shell keyword and `/usr/bin/time`.
* **`irig106-cli` mounts it as `irig106 time …`**, through `run` or the
  individual commands and renderers.

### Current layout — owner 2026-11-21, ADR-0022

[ADR-0022](0022-virtual-workspace-layout.md) supersedes the S16a root-library /
sibling-CLI layout choice. Use a virtual workspace modelled on
[Praevis-Quant/praevis-thetadata](https://github.com/Praevis-Quant/praevis-thetadata/tree/0ee33ca512664e2f3feffa219b4028e1ca0bdc1e):
`crates/irig106-time/` is the library; `apps/irig106-time-cli/` is the testing and
simple-example CLI. Names, the binary, runner and lockstep release contract stay
the same. The root has no package; both crates inherit `[workspace.package]`
metadata and the CLI inherits its exact internal pin from
`[workspace.dependencies]`. Root `fuzz/` stays an independent excluded workspace.
Cargo benches and examples stay with their owning library package.

The earlier S16a instruction used a root library plus `irig106-time-cli/`, and
prohibited a `crates/` move; that layout instruction is historical, not current.
The host-callable runner still takes arguments and injected stdout/stderr,
returns an exit class and never exits the host; only the thin binary main exits.

### Consequences

* Good: one implementation of each time command.
* Good: `irigtime` and `irig106 time` cannot drift apart.
* Bad: every CLI release is a library release and the reverse.
* The command set is `summary`, `channels`, `jumps`, `timeline`, `csv`,
  `correlate`; network time is reported as itself, not as GPS (P6-10).

Coordinator S16a2 review clarification, before exit-contract freeze: use
OsString-compatible arguments and args_os in main. Native non-UTF-8 paths work;
non-UTF-8 command/option names return usage2 without a panic. This applies to
the shared runner and standalone binary, not only one invocation form.
