# S16a — workspace, reusable CLI and exit boundaries

Base: main c82656f (S15 complete). This is S16a1, not completion of S16a.
The owner requires PRs of about 600 changed lines or fewer. Planned split:

1. **S16a1:** root/sibling workspace, lockstep metadata/lock, binary rename,
   and the S15 scanner nit. This record describes that slice only.
2. **S16a2:** extract the CLI library, thin main and injected writer paths;
   keep one implementation, with no process::exit in the host-callable runner.
3. **S16a3:** hand-rolled args, usage-before-input and exit classes 0..4 with
   finding threshold, injected failures and host/standalone controls.

The bounded-memory reader belongs to S16b. The interim whole-file input and
README memory warning are unchanged. No CLI publication or release tag until
bounded streaming and its larger-than-buffer test are present; publish=false
remains as a mechanical gate. No unsafe, mmap, parser dependency or publication.

## Layout and migration evidence

Read only TelemetryWorks/irig106-tmats's public Cargo.toml and
[docs/RELEASING.md](https://github.com/TelemetryWorks/irig106-tmats/blob/18b34e66b276cfc4869b06bcda42ed50612e4286/docs/RELEASING.md#lockstep-versioning)
for its shipped root-package/sibling layout and exact dependency pattern.
There was no access to another local checkout or any edit to that repository.

Root Cargo.toml now owns workspace.package.version (still 0.7.0); both packages
inherit it. The CLI pins =0.7.0. The root lock adds only the CLI package; the
redundant CLI lock is removed, fuzz's independent lock is retained. An explicit
empty workspace in fuzz/Cargo.toml anchors its independent package; without
that boundary Cargo rejected fuzz metadata after introducing the parent
workspace, despite the root exclusion. No crates/
move or release bump. Existing CI paths and all four trace roots are retained.
The binary is irigtime, with matching help/version/banner; the package name
stays irig106-time-cli. README and project-structure examples use the new name
and root target path; scripts must migrate from ch10time. This slice does not
change command parsing, findings, operational exit handling or input allocation.

## Tests first and focused verification

- 8141a1d: the two workspace tests fail before implementation (one missing
  workspace key error and one metadata member assertion). Four subcases of
  the new nested-attribute scanner test fail before the guard change.
- 6070acf: standalone identity test fails at runtime: ch10time 0.7.0 instead of
  irigtime 0.7.0. After the rename it checks exact version and help identity.
- 2021446: after adding the parent workspace, an additional fuzz-isolation
  metadata test reproduces Cargo's workspace-membership error. Adding fuzz's
  own empty workspace fixes it without changing the fuzz lockfile/dependencies.
- All six focused CLI s15_safety tests pass, including the Linux /dev/full
  exit-4 guard. All three workspace tests and all 19 scanner tests pass. Manifest
  metadata is checked locked; no dependency build is performed by that test.
- The scanner now rejects any unmasked dollar token in production attributes,
  including cfg_attr(all(), $l(...)) and allow($n). Tests cover library and
  CLI locations and permitted cfg(test) templates/dollar text in literals.
  Standalone metadata-only negative probes demonstrate both forms compiling
  under Clippy -D warnings but being rejected by the scanner. No probe runs.
- Local checks are limited to fmt, these focused tests/probes, the existing
  policy/I/O probes and the trace check. Remaining platform/feature/MSRV/WASM/
  Clippy/deny/fuzz/CLI checks are delegated to PR CI under the compute rule.

## Clause coverage (no new whole-requirement markers)

| Requirement chain | S16a1 evidence | Remaining |
| --- | --- | --- |
| L1-CLI-001 -> L2/L3-CLI-101 | Root/sibling layout, lockstep version, exact pin, binary identity | Reusable library/module structure and thin main, embedded comparison |
| L1-CLI-002 -> L2/L3-CLI-102 | None added | Host-callable run and injected interfaces, no host exit |
| L1-CLI-003 -> L2/L3-CLI-103 | No parser dependency added | args module, syntax/validation/help behavior |
| L1-CLI-007 -> L2/L3-CLI-107 | Existing S15 operational guards retained | Five classes, threshold, complete precedence and injected/host checks |

No requirement is newly marked complete; matrix coverage remains tested196/288
and verified202/288. Existing test names/markers stay discoverable; the trace
matrix does not lose any tests. No Design or Review field is changed.
