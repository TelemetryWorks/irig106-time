# S14a — event boundary, dependency and Jiff exit audit

Base74fc668 (S13 complete,196/288). S14 splits into event removal/audit here,
then legacy shortcut retirement in S14b; no cross-repository changes.

## Tests first and removals

40b9187 adds three compile-fail import checks before removal: module and both
root event types. All three FAIL because each import still compiles, not
because of an unrelated compiler error. Initial invocation combined --doc
with --lib and was rejected by Cargo; corrected --doc filter gives the three
expected failures.16591f9's caller-extracted tag control already passes on main;
not claimed as a failure-first test. After removal all focused checks pass.
Removed143-line module,95-line fixed-meaning unit suite and pipeline fixture;
no proper L1/L2/L3 requirement markers were attached to those obsolete tests.

| Removed export | Migration |
|---|---|
| recording_event module; root/module RecordingEventType; from_index, may_cause_time_gap, Display | Setup-defined12-bit event fields/descriptions belong in decode; never infer meanings from numbers |
| root/module RecordingEvent; new, has_reference_time; event_type/channel_id/rtc/absolute_time fields | Caller retains decoded event metadata; extracted::inspect_stamp / resolve handle time with explicit context |

No fixed event semantics or event-specific automatic gap/reference admission
replacement is introduced. Generic adapters, native replay and types stay.
T-8 handoff: [coordinator note](handoff-recording-events-to-decode.md).

## Clause-level evidence (no whole-requirement marker added)

| Requirement | Inspected/executed here | Limits |
|---|---|---|
| L1-API-005 → L2/L3-API-105 | Cargo.toml normal edges: irig106-types only; optional jiff; no core/tmats/decode/write/index/CLI/serde edge or forwarding. Executed normal cargo-tree no-default and no-default+jiff; std adds only types/std, jiff only dep:jiff, so these cover normal edge combinations | Dev/CLI dependencies are outside library runtime edges; CI verifies build targets |
| L1-API-011 → L2/L3-API-111 | Existing13 Jiff tests rerun with feature: known year/scale fields; unknown year/scale refusal; UTC/TAI/GPS/assumed provenance retained; eligible second60 refused and malformed labels rejected; covered/stale table conversion explicit | Already implemented in PR51; no conversion/API/dependency change or redundant tests |
| Same optional/MSRV/no_std/WASM clauses | Manifest defaults exclude jiff, dependency default features false; PR51's verification-jiff.md retains actual target/toolchain evidence, current CI checks1.85/WASM+jiff | Runtime tests do not prove target/MSRV clauses; no whole marker |
| Event time tags / ADR-0009 | s14_event_boundary retains raw tags, origin/offset and RTC42 for caller event labels0/1/2/3/4095; removed-export doctests | No event body decoder or semantics tested |

Matrix remains196/288; obsolete GAP-08 text was not valid requirement coverage.
Only fmt, focused doctests/Jiff/tag tests, dependency inspection and trace check
run locally; full checks belong to PR CI under owner compute rule2026-11-18.
