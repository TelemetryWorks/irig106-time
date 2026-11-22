# 0.8.0 release prerequisites

Owner 2026-11-13, plan Q3/Q4; [ADR-0017](adr/0017-releases-after-the-rebuild.md)
and [ADR-0018](adr/0018-0-8-release-scope-and-policy-decisions.md).
This is a preparation checklist, not authorization to publish or configure CI.

## Additional blocker — owner 2026-11-21

[ADR-0021](adr/0021-tmats-release-and-bounded-time-view.md): wait for the **full
irig106-tmats 0.1 release and a bounded time view**. Record the published full
release/version and bounded-view evidence before clearing this blocker; a
placeholder/partial release is insufficient. A future TMATS dependency separately
requires owner approval after publication. No such dependency or publication is
authorized here. The time library remains TMATS-independent. Data-row association
without real evidence remains unresolved; see the [seam design](analysis/S17-time-view-consumer-seam.md).

## Build provenance

Release builds must set `IRIGTIME_SOURCE_REVISION` to the actual release commit.
The CLI's `GITHUB_SHA` fallback can identify a temporary PR merge commit and is
not a substitute for release identity. This is a build requirement only; no
release workflow, publication or tag is changed here.

## Workspace and lockstep versioning

The root is a virtual workspace, with no package. Its members are
`crates/irig106-time/` and `apps/irig106-time-cli/`; root `fuzz/` remains excluded
and independent, with its own lockfile. One root `Cargo.lock` covers the members.
[ADR-0022](adr/0022-virtual-workspace-layout.md) supersedes the earlier S16a layout.

The authoritative version is `[workspace.package] version`; both packages inherit
version, edition, rust-version, license and repository. The exact `=X.Y.Z` internal
library pin lives in `[workspace.dependencies]` beside
`path = "crates/irig106-time"`; the CLI declares `irig106-time.workspace = true`.
The CI metadata/manifest checks enforce matching versions and inherited edges.
Change the workspace version and this shared exact pin together, not a second
package-local dependency declaration.

Select the package explicitly for feature-specific checks and packaging:
`cargo test -p irig106-time --no-default-features`,
`cargo package -p irig106-time --list`, and
`cargo package -p irig106-time-cli --list`. Each package has its own README and
Apache-2.0 LICENSE. These listing checks are not publication or a dry-run release.

Both remain at 0.7.0 during this restructure; the approved release-preparation
PR will set 0.8.0 and the exact pin together. The standalone executable is now
`irigtime`, replacing `ch10time`; the package stays `irig106-time-cli`. Update
scripts to the new executable name. The 0.0.1 crates.io CLI placeholder reserves
the name; it is not this implementation or permission to publish. The CLI stays
`publish = false` until the owner’s S20 publication decision. The S16b
bounded-memory prerequisite has passed the [dense-F1 six-command RSS gate](verification-s16b-final-rss.md).
Passing that gate does not authorize publication or a release tag.

## Packaged-source verification

CI runs `python3 scripts/verify-packages.py`: Cargo packages and verifies BOTH
workspace members together, then compiles all-feature tests from isolated crate
archives. This catches test-only includes that normal package verification may
not build. The schema is canonical inside the CLI package, with a checked docs
snapshot; usage snippets are package-local fixtures checked against the root guide.
The disposable CLI test fixture uses the verified packaged library via a temporary
command-line patch because the in-development0.7 version differs from registry0.7.
No persistent patch or workspace lock change; release still publishes the matching
library first. See [asset verification](verification-package-assets.md).

## Release checklist

- Complete every remaining requirement except the current precisely scoped
  [release deferrals](analysis/plan-decisions-2026-11-13.md#release-scope).
  Do not treat a trace-matrix link as complete clause coverage. Offline
  drift-corrected query mode is after 0.8.0.
- `irig106-types` 0.1 registry adoption is complete; retain final lock/checksum
  and dependency-policy evidence. No git-to-registry migration remains.
- Release `irig106-time` and `irig106-time-cli` together as 0.8.0 (ADR-0007),
  with the CLI using the matching exact library version; make the approved
  version/lockfile changes in the release preparation PR.
- Current README/usage/examples and [migration](MIGRATING-0.8.md) target106-24R1.
  Reconcile any subsequent integration changes before release. Finalize
  CHANGELOG 0.8.0 with T-1..T-17 and their effects on old answers, API migrations,
  MSRV, policy decisions, unresolved readings and these release deferrals.
  Do not yank 0.1.0..0.7.0 (ADR-0017).
- Library serde/JSON removal under owner2026-11-17 [ADR-0019](adr/0019-json-belongs-to-the-cli.md)
  is complete. Keep that boundary, native AnswerView/replay and the types crate's
  independent serde feature; the Q3 library rehydration deferral is moot.
- The CLI schema received its single review and is stable v1/revision 13.
  Follow the reviewed additive/breaking rules for subsequent changes (Q5).
  Verify units, decimal-string 64/128-bit values and raw/stated/assumed provenance
  in emitted output. There is no library JSON loading or CLI input requirement.
- Keep the full GitHub CI matrix green. Under the compute rule, local checks
  stay focused (fmt/touched tests/Clippy); do not duplicate unchanged CI jobs.
  Release-task validation still needs requirements/matrix checks,
  final package-content inspection and
  `cargo publish --dry-run` for **both** packages, using their manifests.
  A dry run performs no publication; retain its evidence and resolve failures
  before requesting an authorized release. Do not bypass verification.
- Actual publication is **only** crates.io Trusted Publishing/OIDC from
  GitHub Actions. No stored publishing token and no local publish, including
  bootstrap exceptions. An authorized future release task must verify the
  crates.io trusted publisher and GitHub identity/workflow prerequisites;
  stop for owner action if they are not ready. This task changes no workflow,
  permissions, secrets, publisher settings, tags or releases.
- Ship no prebuilt binaries for 0.8.0. Conditional REL-106 SBOM/provenance
  obligations remain for future binary distribution.

IRIG 106 does not prescribe publishing authentication. This is the owner's
TelemetryWorks-wide policy. The Rust project's
[Trusted Publishing announcement](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
describes GitHub Actions OIDC and short-lived publication credentials;
[the January 2026 update](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/)
describes enforcement of Trusted Publishing only. A transient credential issued
by that exchange is distinct from a stored publishing token; no credential is
created or handled here.
