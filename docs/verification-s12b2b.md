# S12b2b — exact answers and checked report replay

Base main5dfbdf1. Pre-code split into stacked review units: b1 exact answer
output + remaining adapter audit; b2 checked replay inputs/report/answer loads
and consolidated schema. This is not a release deferral: Q5 is complete only
with both, and the schema remains a candidate for the owner's one review.

## Clause ledger

| L2/L3 pair | b1 | b2 / remaining |
|---|---|---|
| API-107 | Exact serialization of the entire AnswerView graph, including shared quantities/map keys; native adapter lengths/RTC/ERTC/PTP fields fixed. | Full report roundtrip and schema inventory. |
| API-110 | Versioned answer-output with units, recomputation before capture, supported native leaf guards; no complex basis/engine Deserialize added. | Typed complete replay inputs, compare regenerated outputs on load, all adapter envelopes and schema review preparation. |
| POL-103 | Every answer retains full selected policy/context and prior qualification; all wide fields are strings. | Replay restores the actual policy/mode/progress inputs, including previous-policy drains. |

No whole Requirements markers added. Regenerated matrix192/288 (delta0).
Public engine-state Deserialize remains absent under Q3; reporting is not an
unchecked reconstruction of queues/indexes/anchors. No Review/Design changes.

## b1 evidence and migration

Tests-first d520015: five adapter tests failed at runtime on main5dfbdf1
(number tokens instead of strings). The new answer-output test failed to
compile; that first attempt also had incorrect packet/query setup, later
corrected. No claim that this was a clean missing-API-only failure. Added
loader negatives and qualified-graph assertions passed on the implementation.
Final b1 dedicated tests: six adapter + two answer-output tests.

Four temporary numeric-emission mutations (consumed counts, PTP seconds, RTC,
ERTC) each fail at runtime; no compile errors counted, mutations restored.
Native RTC loads enforce48-bit width; ERTC unsigned64; consumed counts are
platform-sized; decimal spellings/ranges use S12b1's checked Decimal parser.
PTP seconds is actually u32 in the implementation/Fig11-17 (11-28/PDF36), not
wide: it remains u32 but is quoted as requested in the audit. Values above
u32-max reject. Fraction/time conversion behavior is unchanged.

Breaking native Serde fields: PtpTime.seconds; Format1/2Reading.consumed;
IntraPacketTime.Rtc/Ertc and SecondaryHeaderTime.Ertc now emit strings.
PtpTime, RTC and ERTC supported loads accept canonical strings only. Raw derived
Format1/2Reading Deserialize is removed: a logical decoded reading must be
regenerated from bounded raw inputs, not loaded with unchecked consumed/body
metadata. Compile-fail docs enforce this. b2 supplies that validated report
path; no extra release deferral is claimed.

Exact field serialization uses one fixed token serializer so recursive
qualification evidence does not instantiate Serializer<Serializer<...>>.
All wide primitives/shared values are emitted as strings; narrow fields stay
numeric. Named numeric map keys become JSON member strings. No dependency was
added. The token tree is an internal protocol value, not an exposed unchecked
engine loader; finite input/output buffers remain caller-owned.

b1 verification: full CI equivalent recorded after completion below. No new
page-image checks: existing reviewed Fig11-17 wire width and RFC5905 Figure4
1970 coordinate are reused. JSON representation is owner Q5/ADR0010/0011,
not an IRIG106 mandate. First part alone does NOT complete Q5 or freeze schema.

b1 restored source: all18 local CI-equivalent checks pass, jobs2/threads2,
--locked, MSRV1.85, CLI, no_std/WASM±serde, benches, all10 fuzz builds, deny,
fmt/clippy/rustdoc and matrix. Source parsing/conversion/defaults unchanged.
