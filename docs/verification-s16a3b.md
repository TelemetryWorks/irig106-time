# S16a3b — shared finding threshold and complete exit contract

Base main e19ed51. Public TMATS audit pinned18b34e66b276cfc4869b06bcda42ed50612e4286:
[CLI source](https://github.com/TelemetryWorks/irig106-tmats/blob/18b34e66b276cfc4869b06bcda42ed50612e4286/irig106-tmats-cli/src/main.rs) is a help/version scaffold, [docs/CLI.md](https://github.com/TelemetryWorks/irig106-tmats/blob/18b34e66b276cfc4869b06bcda42ed50612e4286/docs/CLI.md) gives only draft0..4, and
L2/L3 are placeholders. No threshold name/default exists there. The coordinator
therefore establishes this convention here for later TMATS adoption. Only the
public source was read; no other repository was edited.

`--fail-on <level>` uses case-sensitive lowercase `error`, `warning`, `info`,
or `never`, as a separate argument. Default: `error`. Exit1 occurs if any
finding is at or above the selected level (`error > warning > info`); `never`
disables only exit1. Accept the option on every command, including help/version,
where a valid option has no effect. Unknown values and duplicates are usage2.

Precedence is usage2 first, then the first fatal input3/output4, then findings1,
then success0. Validate all usage before opening input; finish output and both
flushes before selecting a finding-only result. Diagnostics after fatal errors
are attempted where possible and cannot replace that first fatal class. Here
input/analysis completes before rendering, so a later input failure after an
output failure cannot occur; S16b's reader must preserve the same precedence.

## Full current severity table

| Current CLI finding | Level | Basis |
| --- | --- | --- |
| Recovered sync loss | warning | Coordinator reader policy |
| Truncated final packet or header | warning | Coordinator EOF policy |
| Trailing unsynced bytes after a stream | info | Coordinator EOF-padding policy |
| Malformed/short time data, invalid BCD digits, date/field ranges | error | TF-004 class |
| Reserved BCD bits | warning | Owner-approved TF-005; exception to the proposed generic decode-error grade |
| Invalid TS / reserved TS or NTF value | warning | Owner-approved TF-003 / TF-006 |
| Correlation needs context (NTP era or no known applicable leap offset) | warning | Coordinator context policy |
| No usable correlation reference | warning | CLI context finding |
| Reported time jump | warning | Owner-approved TF-017 |
| Other unusable correlation/query result | error | Affected answer cannot be produced |

**Flagged disagreement:** reserved BCD bits remain TF-005 WARNING, not the
proposed generic decode-error ERROR. Recommend preserving that owner-approved
level, as required by the coordinator's first rule. Architecture TF rows and
Review/Design fields are untouched. No new TF identity is invented. Full native
finding generation/value retention/report provenance remains S16b; this slice
only grades the messages currently emitted by the legacy CLI scanner.

## Evidence and clause limits

1464c25 is tests first: five runtime failures on main (including the aggregate),
two controls pass. Additional rank/parity/level controls were added with the
implementation, not claimed as failing on main. The cross-command malformed
case caught a new grading-path defect: correlate replaced Error with its later
no-reference Warning. Its regression failed0 vs1 before the maximum-grade fix.

There is a named test per numeric code, plus a whole-contract aggregate reusing
those checks. It covers inclusive exact thresholds, changed thresholds, never,
unknown/case/duplicate values, usage before nonexistent input, input failure
with a failed diagnostic writer, first output failure with later diagnostic
attempts, both flushes, file-output failure over findings and host/standalone
class/output parity for0..4. All commands accept fail-on; help/version validate
it with no effect. Both noted parser edge cases stay unchanged. Existing#63
panic/native-argv/sync/cut-tail/padding controls remain; short-body boundary
checks opt into never to isolate classification from the new error threshold.

L1-CLI-007 -> L2/L3-CLI-107 is linked only by the aggregate covering that whole
exit statement; per-case tests carry no whole markers. CLI-101/102 full module,
input-interface and report clauses remain for S16b. No claim of full TF catalog
coverage. Only focused tests/mutants/fmt/CLI Clippy locally; remaining matrix
uses PR CI. Memory/publication gate remains; no dependency/version/publish/tag.

Eleven focused runtime mutants are killed: exclusive threshold; never enabled;
missing stdout flush; missing stderr flush; fatal class overwritten; omitted
post-fatal diagnostic; padding raised to warning; reserved bits raised to error;
needs-context raised to error; no-reference warning lowering an existing error;
and default changed to warning. Mutations are restored before final checks.
Coverage: tested198/288 (+2), verified206/288 (+2), through CLI-107 only. The
existing CLI-103 inspection and every prior test remain; no FND overclaim.
