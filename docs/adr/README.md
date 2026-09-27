# Architecture Decision Records

MADR-format records of the decisions that shape `irig106-time` and
`irig106-time-cli`, in the form of `irig106-tmats`'s `docs/adr/`. Each record
states the problem, the options considered, the choice, and its
consequences. A decision is changed by a new record that supersedes the old
one; records are not rewritten.

**Status:** `accepted` — decided by the project owner; `proposed` — part of
the architecture proposal (`docs/architecture.md`) awaiting the owner's
review. The contract these decisions serve is `docs/TIME-IN-CHAPTER-10.md`.
All seventeen records were accepted by 2026-09-27 (0011 to 0014, 0016, and
0017 on that day, at the owner's review of step 1).

| ADR | Decision | Status |
|-----|----------|--------|
| [0001](0001-rebuild-rather-than-patch-the-prototype.md) | Rebuild the crate rather than patch the prototype; the prototype kept at `prototype-0` | accepted |
| [0002](0002-documentation-first.md) | Documentation first: contract, decisions, architecture, requirements, tests, code | accepted |
| [0003](0003-edition-2024-and-msrv-1-85.md) | Edition 2024 and Rust 1.85 across the ecosystem | accepted |
| [0004](0004-baseline-and-editions.md) | Baseline 106-24R1; every rule cites the archived standard; edition differences named | accepted |
| [0005](0005-leaf-library-depending-only-on-irig106-types.md) | A leaf library: depends only on `irig106-types`; `irig106-decode` depends on it | accepted |
| [0006](0006-library-performs-no-io.md) | The library performs no I/O | accepted |
| [0007](0007-lockstep-workspace-and-irig106-time-cli.md) | One workspace in lockstep: the library and `irig106-time-cli`, binary `irigtime`, mounted as `irig106 time` | accepted |
| [0008](0008-hand-rolled-cli-argument-parsing.md) | Hand-rolled argument parsing for `irig106-time-cli` | accepted |
| [0009](0009-recording-events-move-to-irig106-decode.md) | Recording events move to `irig106-decode`; this crate keeps their time tags | accepted |
| [0010](0010-selectable-time-policy.md) | The rules of time over a recording are a selectable policy, recorded with every answer | accepted |
| [0011](0011-answers-carry-their-basis.md) | Every absolute time carries its basis | accepted |
| [0012](0012-reading-register-for-tmats-and-packets.md) | Readings between TMATS and the packets are a reviewed register | accepted |
| [0013](0013-findings-not-repairs.md) | Degraded time is reported as findings with stable identifiers; never repaired silently | accepted |
| [0014](0014-no-std-with-alloc.md) | `no_std` with `alloc`; `std` a default feature; WebAssembly built in CI | accepted |
| [0015](0015-shared-time-values-in-irig106-types.md) | Shared time values live in `irig106-types` and are fixed there first | accepted |
| [0016](0016-tests-from-the-standard.md) | Tests from the standard: a failing test per finding; fixtures from its figures; real data local only | accepted |
| [0017](0017-releases-after-the-rebuild.md) | The rebuilt crate is released as 0.8.0; users of 0.1.0 to 0.7.0 are told what was wrong | accepted |

New records take the next number and use the same front matter
(`status`, `date`, `decision-makers`).
