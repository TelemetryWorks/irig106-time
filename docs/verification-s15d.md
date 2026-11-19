# S15d — recording/adaptor traits and completed library boundary inspection

Depends on S15a/PR57 and S15c/PR59. CLI root/ingestion is independent PR58.
This record closes the library I-inspections below; it does not turn partial
runtime tests into whole ERR-101 coverage or claim PR58's code is in this head.

## Trait contract

Inventory:186 current public struct/enum definitions across src, including the
two internal-file values re-exported through declarations.179 are public value
shapes,7 are mutable controllers (listed below). The external integration test
compiles Debug/Clone/PartialEq bounds for all179 and Copy/Eq/Hash for95 fixed
local shapes, plus the shared root re-exports. Generic views are instantiated
with permitted value parameters; derives retain their conditional type bounds.
Jiff's feature-gated Scaled<DateTime> is checked by the S15c test.

Controllers: TimeCorrelator, StreamingTimeCorrelator, TimelineBuilder,
OfflineTimeline, StreamTimeline, DeclarationAudit, SecondaryAudit. These manage
mutable lifecycle/index/queue state, not replayable public-value snapshots.
Do not force Copy on owning buffers or equate controller identity with answers.
The inventory test reads this repository's source in test code only, compares
every public struct/enum against the checked value set and explicit controller
set, and requires classification when a new definition appears. It is not a
Rust parser for arbitrary generated/conditional source; there is none in this
production boundary. Shared re-exports and aliases use explicit trait controls.

Add missing structural derives only: answer/adapter snapshots become Clone/
PartialEq/Eq, fixed selectors/metadata become Hash, TimeJump/RtcReset become
Copy. Equality includes raw/context/basis/findings, not just the time reading.
TimeQuality/OfflineQuality contain f64: PartialEq only, no invented Eq/Hash.
CounterAnswer borrows non-Hash answer snapshots: Copy/Eq, no forced Hash.
TimeError contains shared TypesError without Copy/Hash; do not duplicate or
change that upstream type. All owned Vec/String/Box/BTreeMap snapshots remain
non-Copy. No serialization, load/replay policy or arithmetic change introduced.

Tests first:bb1f602 precedes production derive changes. The corrected inventory
replayed on pre-fix sources produces73 missing-bound diagnostics (not73 types),
without unrelated compiler errors. Exploratory internal-file paths were fixed
to their actual public re-exports before publishing this tests-only commit.
Both focused trait tests and all-target/all-feature Clippy pass after fixes.

## Allocation boundary

Fixed layer inspection: [S15c call graph and bounds](verification-s15c.md).
CSDW, BCD, bounded Format1/2 readers/writers, RTC quantities, secondary/IPTS,
Chapter4 words, ordinary calendar math and Jiff civil conversion use fixed
arrays/scalars/borrowed slices, fixed error payloads and no allocating calls.
No parser length chooses a heap allocation. Copy/drop tests protect the shape;
source-call inspection, not Copy alone, proves absence of temporary allocation.

Alloc layer is explicit: qualified declaration/edition evidence, leap tables,
NTP conversion basis, findings/adapters, policy/answer snapshots, correlators,
timelines and streams. A qualified single answer may allocate. Offline input
length owns its state/work budget; streams cap retained packet/ref counts and
caller-declared bytes. Those are not hard RSS limits on all cloned metadata or
allocator overhead. No recoverable-global-OOM or allocation-free recording claim.
Feature inspection: std only forwards types/std and gates the same core Error
trait; no std-only public value signatures. Optional Jiff adds only civil
adapters. CI covers default/no-default/Jiff/no-default+Jiff,1.85 and WASM;
no new feature-dependent value layout/signature is introduced here.

## Recording bounds and overflow inspection

| Modules | Termination / checked state boundary |
|---|---|
| declarations / edition / declaration checks | Finite supplied ranges/attribute lists; validates ordering/identity; BTree maps and bounded standard mappings; no setup/network/file discovery |
| leap / ntp | Finite validated entry lists; binary/linear searches bounded by entries; era pivot candidates fixed and supplied pivots finite; i128 checked coordinates, calendar domain checks; zero sentinel and missing contexts retained |
| reference / years | Raw/native versus normalized scale guarded; checked shifts; year phase considers finite hypotheses. Offline requalification strictly increases invalid-label count on a continuing pass, at mostR+1 passes forR refs; no claim of linear complexity |
| timeline | Finite observations/sessions/indexes; duplicate/session guards; checked tick/duration arithmetic and explicit partials. Offline intentionally owns input-sized memory |
| stream | Every draining loop removes an item; expiry uses caller progress, not a clock. Ref-cap loop shortens refs; held counts/declared bytes checked, fixed-channel segment barriers preserved; output batches caller-owned, never a hidden output queue |
| own_time / secondary_packet / extracted / answer | Fixed wire reads followed by explicit allocating evidence; checked session/epoch/era/table and duration bounds, capacity/identity checks; replay validates frozen basis, never searches/fetches ambient state |
| findings / policy | Private validated IDs/indexes, explicit configured severities; validated intervals/caps and checked deadlines. No fabricated fallback severity or host default |
| legacy correlation / streaming / quality | Finite sorted reference collections,48-bit RTC*100 fitsu64; ordinary calendar bounds; guarded density/drift divisions. S15a removes the two guarded quality unwraps; legacy results remain explicitly unqualified |

No unbounded fixed-value parser loop, unchecked public-input arithmetic defect
or source panic operation was found beyond the cleanup already proposed in
PR57/58. This is source inspection plus existing regression/property coverage,
not an exhaustive fuzz campaign or proof about allocator/foreign/caller code.
CLI remains a separate S16 release obligation: prototype secondary-header bit,
Data Length/packet walking and complete usage/input/output precedence still need
that rebuild. PR58's safe snapshot has whole-file RAM/latency cost; it does not
solve streaming CLI input. Rendering uses standard print macros at named CLI
I/O boundaries; full write-failure handling is S16, not a library exemption.

## Library I/O

All production modules were searched for std imports/I/O names/foreign blocks
and dynamic code includes; only test-only std import remains. Library namespace
probe accepts safe core code and rejects clock/env/file/network even with
feature=std. Library root cannot reallow unsafe (PR57); CLI equivalent is independent PR58. Dependency unsafe
is outside that claim. Complete relevant call sets: types byte/quantity/error
operations; optional Jiff civil constructors/builders/getters only. Both normal
edges are unchanged; no core/tmats/decode/CLI/I/O dependency edge. Leap data are
literal or caller supplied, never fetched. Caller callbacks/format sinks and
host allocator behaviour are caller infrastructure, not hidden library input.
New dependencies/imports/callback behaviour require reinspection.

## Trace and execution limits

L2/L3-API-101/102/106 get Inspection evidence links to this complete inventory/
call-graph/feature audit. No new whole Requirements marker: tested196/288;
verified202/288 after the six evidenced I requirements. ERR-101 stays clause
level (runtime tests alone do not prove no allocation/loop defects); API-109/112
root evidence remains in PR57/58 records pending integration, not falsely
attributed to this head's still-prototype CLI. No Review/Design field change; only evidence-driven matrix rollup changes.
Local work is fmt, focused trait tests and Clippy only; PR CI owns full matrix.
CI accepts S15 stack bases so this final module-sized PR targets PR59 directly.

Negative controls after implementation: a temporary otherwise well-derived
public SafetyInventoryProbe compiles but fails the runtime inventory comparison;
removing BufferUsage Hash fails compilation specifically E0277/Hash. Both
mutants restored. No code/syntax/tool failure is counted as a successful kill.
