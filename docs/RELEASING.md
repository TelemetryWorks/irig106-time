# 0.8.0 release prerequisites

Owner 2026-11-13, plan Q3/Q4; [ADR-0017](adr/0017-releases-after-the-rebuild.md)
and [ADR-0018](adr/0018-0-8-release-scope-and-policy-decisions.md).
This is a preparation checklist, not authorization to publish or configure CI.

- Complete every remaining requirement except the four precisely scoped
  [release deferrals](analysis/plan-decisions-2026-11-13.md#release-scope).
  Do not treat a trace-matrix link as complete clause coverage. Offline
  drift-corrected query mode is after 0.8.0.
- Publish `irig106-types` through its own authorized release process before
  replacing this repository's git dependency with a crates.io version. No
  dependency change is made by this documentation PR.
- Release `irig106-time` and `irig106-time-cli` together as 0.8.0 (ADR-0007),
  with the CLI using the matching exact library version; make the approved
  version/lockfile changes in the release preparation PR.
- Refresh README, usage and migration documentation to 106-24R1. Finalize
  CHANGELOG 0.8.0 with T-1..T-17 and their effects on old answers, API migrations,
  MSRV, policy decisions, unresolved readings and these release deferrals.
  Do not yank 0.1.0..0.7.0 (ADR-0017).
- Review the versioned JSON schema once before freeze. Verify units,
  decimal-string 64/128-bit values, supported-load validation and provenance.
  Unsupported complex timeline Deserialize must be disabled/omitted.
- Run the full CI equivalent with bounded resources (`CARGO_BUILD_JOBS=2`),
  requirements/matrix checks, final package-content inspection and
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
