# S15c — fixed-reader traits, allocation and library I/O boundary

Depends on S15a/PR57; separate from CLI PR58. Recording/adaptor trait inventory
and state audit are split into S15d. No whole requirement marker added here.

## Evidence order

8ca9c24 adds compile probes/tests first. The production prefix under feature=std
admits the clock probe; the test fails because it compiled (no I/O executed).
Trait compilation exposes twelve missing Eq/Hash/Copy bounds on fixed values;
two additional failures came from a draft attempt to treat OfflineTimeline as
a Debug/Clone value: removed that controller assertion, not a value requirement.
Unused scale glob removed. Add only structurally valid derives in the fixed
modules, including dependencies of the Chapter4 contexts. No shared types copied
or changed; TimeError remains non-Copy/non-Hash because TypesError provides
neither trait. Error payload fields are fixed scalars/static strings nonetheless.

Remove production extern crate std; keep it cfg(test) only. Implement the same
feature-gated Error trait via core::error::Error (std reexports that trait).
No default/no-default error-trait or public-signature change is intended.
CI verifies1.85 and feature/target builds; do not substitute metadata for them.

## Fixed entry-point call graph / bounds (inspection)

| Entry points | Allocation / loop and error boundary |
|---|---|
| rtc checked quantities, CSDW getters/writers; version/packet-standard maps | Pure fixed math/matches; counters48-bit, ERTC1ns; checked signed conversions, named session guards; explicit modular arithmetic only |
| BCD6/8-byte readers, decode_time_f1/decode_time_f2 and encoders | Fixed array/nibble slots, slices bounded before index, Data Length excludes trailer; decimal/range/precision errors; CSDW enum/layout guards; year0..9999 native/3999 wire |
| secondary parse/checksum, intra_packet parse/validation | Fixed8/12-byte checks, at most10 checksum bytes; wrapping16-bit checksum is intentional, reserved findings separate; microseconds0..9999/PTP ns<1e9; no flags repaired |
| Ch4Words binary/bcd | Full32-bit binary count*10ms plus usec fitsu64; explicit origin/year/full-day/wording; missing context stays partial; no allocation or packet search |
| AbsoluteTime/CalendarTime arithmetic | Bounded checked displacement; known year checked at0..9999, loops advance a year and terminate; unknown-year crossing refuses; no inferred origin or host year |
| Deprecated PTP scalar calendar helper | Private year walk called only with u32 seconds and i32 caller offset: at most6,442,450,943 unsigned Unix seconds (year2174); no arbitrary u64 year walk through public input. Deprecated NTP helpers always refuse |
| Jiff civil adapters | Existing checked year/scale/leap/range paths; default features disabled, no timezone/clock APIs. Pure Date/DateTime fields/builders, no implicit table or UTC |

None of these fixed operations calls Vec/String/Box/format allocation. Their
error cases also build fixed payloads. Copy/Eq/Hash compile assertions and
needs_drop checks protect value shapes, NOT temporary allocations inside a
function; the no-allocation claim rests on the inspected closed call paths.

## Alloc layer (deliberate, not allocation-free)

Qualified edition/declaration/Format1 wrappers, NTP/leap-table provenance,
Format2 admission, findings, own/stamp/word adapters, policies, answer replay,
quality and timelines/streams own Vec/String/Box/BTreeMap/VecDeque. Even one
qualified answer may allocate for evidence. Fixed decode_time_f1 differs from
alloc-backed decode_qualified_time_f1 in the same module: layer is an entry-point
contract, not a claim that every export of a source file is allocation-free.
Recording storage is caller-managed/capped as required; global allocator OOM
is not claimed recoverable. Detailed recording-state bounds/traits are S15d.

## No-I/O proof boundary (inspection + negative compilation)

Project production source has no std namespace import, env/file/network/clock
call, foreign extern block, dynamic code include or unsafe escape. Only test
code can import std. Root forbid from S15a rejects foreign unsafe escape.
The new CI script copies the actual production prefix and compiles with the
std feature enabled: safe control succeeds; clock/env/file/network references
fail specifically E0433/std unavailable. Probes are metadata-only, never run.

Normal runtime edges remain only irig106-types0.1 and optional Jiff1.0.1 with
default features disabled. Inspected called types operations are checked numeric
wrappers/byte readers and error traits; Jiff calls are civil constructors,
field getters/builders. No clock/fetch/host fallback path. This is a closed-call
source/dependency audit, not a claim all dependency internals contain no unsafe,
that alloc never invokes the host allocator, or that caller callbacks cannot
perform caller-owned I/O. New dependency/import/callback changes require review.

Clause coverage: L2/L3-API-101 fixed values; API-102 fixed versus owned layer;
API-106 production namespace/call/dependency boundary; ERR-101 fixed bounds.
No claim all recording/adaptor public-value traits are complete yet. API-109/112
root work comes from PR57/58; no blanket exemptions added. Matrix196/288.
Locally fmt, focused trait tests, no-I/O compilation probes and library
all-target/all-feature Clippy; full feature/target/CLI/fuzz/deny checks in CI.
