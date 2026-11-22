# S17 completion ledger — after #104 / schema revision 13

Base 7c1ca0d; supersedes the [dated #91 audit](https://github.com/TelemetryWorks/irig106-time/blob/7c1ca0de669b585a90374d71b62b4b74be984e06/docs/verification-s17-completion.md).
This docs/examples slice changes no runtime policy, Review/Design field, version
or publication setting. Earlier records remain dated evidence. Inspection is
against current source, manifests, help/parser and verification records; no new
standard-page inspection or upstream release-status check is claimed.

## Closed S17 clauses (not whole-requirement closure)

| Lineage / clause | Current evidence and disposition |
| --- | --- |
| L1-CLI-004 → L2/L3-CLI-104: buffered scan, Data Length, checksum/resync, final-tail split | S16b records and tests; one bounded reader/report loop; complete-stream requirement retained |
| CLI-104: bounded state and atomic CSV | S16b spool/permissions/error/reset tests and dense 32 MiB gates, including 100k JSON rows; private year spool in #104 |
| CLI-104: native sessions/spans/quality | #98–104: native SessionClock, F1/F2 candidate quality, TAI normalization, stale-drift flag, disk-backed DMY replay and year evidence. Native sessions are distinct from CLI reset segments |
| L1-CLI-005 → L2/L3-CLI-105: protocol/status/scale | #92 packet/reference facts, #93 19-column CSV, #94 F1 scale/basis, protocol/stale/reset/full-adapter goldens. Native GPS never relabelled UTC; NTP/PTP revisions retained |
| L1-CLI-006 → L2/L3-CLI-106: shared views and machine formats | Summary/channels/jumps/timeline/correlate JSON and text share the report; CSV is a separate documented machine format, not JSON-wrapped CSV. Rows/answers stream from private spools |
| CLI-106: schema/version/units/provenance | One schema review and freeze, additive revisions through13; ASCII JSON, exact wide decimal strings, per-command findings, policy/window/cap/reset/leap-table identity, native year source/candidates |
| CLI-106: NTP era context and automated validation | #96 explicit --ntp-era, no CLI guessing; #97 pinned/hash-locked isolated jsonschema tooling validates all 30 live views, with option/fixture guards |
| CLI-106: current docs/examples/migration | This slice: current106-24R1 guide, executed qualified/network/Jiff examples, package README, migration table, consolidated Unreleased notes. Prior guide/slice notes are linked at an immutable commit |
| L1-CLI-007 → L2/L3-CLI-107: exit/host/writer behavior | S16 exit/threshold/transaction tests retained; context is a finding, not output failure (#104 year-range regression); writer/flush failures remain4 |

The #104 review compared 45,000 cases with offline (reviewer evidence, not a new
run here). Its filter tests preserve baseline packet-local findings and pin
newly derived TF-007, exact year-evidence text and corrupt-flag rejection.
[Deferred follow-ups](S17-deferred-followups.md) remain explicit: decode-once,
packet-local TF-007 omission and the single-pass mutation survivor.

## Qualified-joining audit completed; closure remains gated

The [2026-11-22 clause audit](verification-s17-qualified-joining.md) is the current
closure disposition for CLI-104/105/106. **Draft — closure gated on TMATS
(ADR-0021)**; no whole-requirement credit is added. It separates closed clauses
from TMATS-dependent consumption and independent native adapter/basis integration.
Summary session-year replay is closed by #104; it is not an open implementation
item. S18-S20 are not started; wait for the coordinator after the TMATS gate.

- **TMATS gate (ADR-0021):** full published TMATS 0.1 and bounded time view, then
  explicit owner dependency approval. No time-library→TMATS dependency. CLI
  setup extraction, governing ranges, declared-vs-observed channels, RDG001..006
  comparison propagation and inferred/opt-in-default TFMT report goldens await it.
- **Association:** Chapter9 has no data/time-channel link. Keep
  association_unestablished / association_conflicting unresolved; only real
  relationship evidence can yield association_evidenced. No channel-ID/RTC
  coincidence rule, especially for data rows after resets.
- **Native answer integration:** current reports still use the documented CLI
  compatibility query policy. Full qualified AnswerView/reference/adapter basis
  and governing-source joining remain incomplete, as the new clause audit records;
  rendering goldens do not establish that all native engine clauses are wired.
  Do not silently classify every independent integration obligation as TMATS-gated.
- **Release:** final remaining-requirement audit, approved deferrals, lockstep
  version/exact pin, package/dry-run/source-revision/CHANGELOG acceptance and
  owner S20 publication decision. Types0.1 adoption is done; library JSON
  rehydration is moot, not a restoration task. Trusted Publishing/OIDC only.

## Coverage and executed checks

No new whole-requirement markers: CLI-104/105/106 remain Draft in the generated
matrix, which cannot encode these partial clauses. **198/288 tested (68.8%);
206/288 verified (71.5%); delta0/0.** L1 count124, L2/L3 count144 each.
New Rust examples and snippet synchronization are executed locally with/without
Jiff; network fixtures retain their existing runtime assertions. Shell commands
are inspected against parser/help, not run against a fabricated flight file.
Fmt/trace and focused documentation guards run locally. PR CI supplies the full
matrix, package/schema/fuzz/deny/WASM and memory evidence. No release is cut.

Current documentation checks: qualified example/snippet and existing network
fixtures passed locally; the qualified example also ran with Jiff. Its initial
finding-count assertion assumed one fact, but the explicit undeclared override
also reports a fact; the example now preserves every supplied finding rather
than dropping one. No runtime implementation was changed to fit documentation.
Historical usage and development notes remain at the linked immutable commit; the published
0.1–0.7 changelog sections are byte-for-byte unchanged. No tests-first runtime
failure claim is made for this docs-only slice.


PR105 migration-review delta: added explicit u64→NanosDuration before/after
calls, typed/fallible leap/network signatures and executed migration snippets.
The silent DEFAULT_OOO_WINDOW_NS 2s→1.1s change is prominent, with contract §5.4 /
ADR-0010 basis; new/Default still store None. No runtime semantics were changed.
The Python/Git source API gate pins v0.7.0's peeled commit, prints the complete
local-declaration diff in CI and requires a matching migration acknowledgement.
Supporting private types/constants are included conservatively; external API
expansion/cfg reachability and behavior still require human/compiler review.
Six unit probes cover declarations, constants, aliases, literals, attributes,
macros, cfg/test filtering, stale markers and CI wiring. Live negative probes
changed the duration constant and added a public constant: both failed the gate;
restoring sources passed. No Rust build/mutation campaign is claimed for these
source-only probes. The migration examples run with deny(deprecated); the old
0.7-only snippet is labelled ignore. Baseline sources are read, never built, so
old chrono/serde dependencies are not installed. No new tools/shipped dependencies.
