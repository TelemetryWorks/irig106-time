# S17 item3 — effective JSON policy/default identity

Base main7016804(#94 merged). Partial L1-CLI-006 -> L2/L3-CLI-106. Add
report.policy in stable schema v1 revision7, with no new dependency/library API,
Review/Design or finding-level change. Text/CSV, numeric correlation and reset
classification remain unchanged. No whole Requirements marker/coverage increase.

## Identity and actual scope

The CLI still calls the compatibility nearest-reference counter-offset APIs.
It must not claim native Standard/Interpolate mode or forge a library SharedPolicy
ID. Its IDs are telemetryworks/irigtime-cli-nearest-v1 and
telemetryworks/irigtime-cli-defaults-v1. Change the first with algorithm changes,
the second with fixed defaults; command overrides remain separately visible in
options. A future native adapter must identify what it actually runs. ADR-0010
requires policy/settings to be recorded; these IDs describe the current CLI,
not proof of complete native answer replay or qualified data-source joining.

The snapshot records no drift correction, actual main live-cache age from its
getter, shared1024 reference-cap constant, target query brackets (six candidates
per channel), strictly-greater30s immediate-reset threshold and increasing
three-reference late-run threshold. Constructor/isolated-pool window and reset
code now use shared constants, with unchanged values. The cap is per main cache
and per combined isolated pool, not a claim of1024 total objects in all report
state. Old target queries retain independent bounded brackets across the file.
These are accepted reader/resource rules, not RCC wire bounds (S16 decisions).

Leap metadata/count come from the scanner's actual installed table, not a fresh
builtin. ID includes source hash; source/coverage remain separate. Unix1972-01-01
=63072000; IERS C72 current-through1814140799 is inclusive of fractions;28
transitions do not mean coverage ends at the final2017 transition. This repeats
S04's verified metadata, with no new IERS fetch/page check. Per-clock stale
status remains separate from installed-table identity. No host clock or I/O.
All wide quantities are decimal strings. The field applies to the whole report,
is present without findings, and is additive under the frozen-v1 consumer rule.

## Evidence and honest tests-first

Tests-only5eb7f30: both integration and private-state tests fail on main at missing
policy metadata. An initial fixture incorrectly passed a jumps-only option to
summary; corrected before the tests-only commit, not a production defect.
The valid no-finding F1 recording uses literal106-24R1 Fig11-12/13 FMT4/SRC1,
DOY10012:00, independent11.2.1.1 checksum, packet36/Data Length10 excluding
filler. All five JSON views pin the entire policy and actual thresholds:
summary1000ms, jumps17ms, othersnull. Expected snapshots/golden edits are literal
from accepted defaults and existing S04 identity, never captured CLI output.

Private-state checks fill1050 references to verify the reported cap matches
actual retained1024, then substitute47s cache age and an empty leap table: the
report must reflect that actual state. Implementation-side checks pin equality
at30s versus31s, two versus three increasing late references, and the real six-
candidate duplicate bracket bound. These added checks are not claimed failed-
first feature tests; they protect the effective-settings claim.

Nine compiling mutants are killed: wrong defaults ID; preceding-mode label;
fixed30s instead of actual cache getter; actual cache cap1 while reporting1024;
fresh builtin instead of installed table metadata; wrong currency bound;
inclusive-reset comparison; two-reference run; claimed drift correction.
Source restored. Local fmt, focused tests/mutants and trace/schema-sync/diff
checks only; remaining checks and32MiB RSS gates use GitHub CI. Projection adds
only one fixed-size policy object/borrowed table metadata, no packet history.
198 tested/206 verified of288 unchanged. NTP era context, automated schema
validation, native sessions/quality and docs/migration remain items4..7;
TMATS declarations stay separately gated. No merge, tag or publication.


## #95 review — isolated-cache and sample-cap coupling

Tests-only37f345a adds an isolated-state comparison (passing at the reviewed
head) and a sample-cap test that fails there because the fields were absent.
The isolated implementation already uses the same REFERENCE_WINDOW_NS and
REFERENCE_CAP constants as the main cache; no resource value changes. The
new test fills1050 interleaved references, compares JSON with each real channel
cache's max-age getter, and compares the reported cap with actual retained
references AND pooled arrival entries after eviction. Both requested compiling
mutants (isolated20s, pool cap512) now fail that test at runtime.

Revision7 is still unreleased: add finding_sample_cap and jump_sample_cap there,
not a revision increment. Each reads its own sample constant; jump retention now
uses the dedicated JUMP_SAMPLE_LIMIT(20), while finding retention continues using
FINDING_SAMPLE_LIMIT(20). The test produces35 findings and31 literal+1s jumps
(Fig11-13 clock advances2s per1s RTC), and compares JSON with each actual retained
sample vector. Exact total counts and behavior stay unchanged.

A temporary build with distinct limits17(findings)/11(jumps) passes the coupling
test. Four compiling crosswire mutants then fail: either JSON cap reading the
other constant, jump retention reading the finding limit, or finding retention
reading the jump limit. Source/limits are restored. This avoids both defaults20
masking a wrong link. These probes plus the two isolated-cache mutants are six
additional killed mutants; no repeated full local matrix. Focused tests, fmt,
trace/schema-sync/diff locally; full checks and RSS gates are from new-head CI.
No Review/Design, library/dependency, numeric, text, CSV or coverage change.
