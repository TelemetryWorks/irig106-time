# S09a — governing declarations and RDG-001..004

Base main `58b1cf0070779c0e2ca0805b954d17cc0d1bedf9`. PR28 is frozen and
not part of this branch unless merged into main. Scope follows the 0.8 plan's
S09a: plain setup facts/ranges and TTF/TFMT comparisons. No time-to-tmats
dependency, parser, default insertion, TSRC/SHTF comparison or CLI change.

## Written reading analysis (before implementation commit)

Source evidence is the existing register/page review, not a fresh PDF rendering.
106-24R1 Table9-4, printed9-50/9-72; required/default definitions9-5/9-6;
Ch11 Table11-4 and §11.2.3.2/3. RCC124-22 Ch7, printed7-3/PDF149. Historical
edition comparisons beyond caller-qualified facts remain an RDG-102 evidence
gap; no blanket all-edition or two-person-review claim.

| Reading | Verbatim source fragments / derivation / edges |
|---|---|
| RDG-001 | Table9-4: "0 Reserved", "1 Time data", "2 Network time"; Table11-4: "0x11 Time Data, Format 1", "0x12 Time Data, Format 2 Network Time". Pair by name, retain missing/unknown TTF, report mismatch/reserved declaration with TF-010. Packet type remains authoritative. |
| RDG-002 | Table9-4: "A IRIG-A 1xy", "B IRIG-B 1xy", "G IRIG-G 1xy", "N Native GPS time", "U UTC time from GPS", "X None". §11.2.3.2a identifies FMT1/0/2/5/4/F respectively. All matching pairs and all other nibbles must be tested; reserved observed packet values also retain TF-006. |
| RDG-003 | Table9-4: "I Internal"; §11.2.3.2a: "0x3 = Real-Time Clock". RCC124-22: `R-1\TTF-1:1; R-1\TFMT-1:I; R-1\TSRC-1:I;`, "Time format is internal", "Time source is internal". No stated equivalence: owner Q1 accepts I↔3 as inference. Forward AND reverse mapping retain that label. Matching I stays labelled; a mismatch is TF-010 WARNING by default. No unconditional new warning identity is invented for a match. SRC0/1 is retained but not compared until S09b. |
| RDG-004 | Table9-4: "0 Network Time Protocol Version 3 RFC-1305", "1 IEEE Std 1588-2002", "2 IEEE Std 1588-2008"; §11.2.3.3a NTF0/1/2. Apply numeric vocabulary only after TTF2 context; wrong/missing context is visible, never a letter/NTP fallback. |

Required/default source: TFMT row "R/R Ch 10 Status: R", "Default: A";
§9.5.1a: "A value of “R” requires that the attribute be specified in the TMATS
file whenever the attribute is allowed." The note on9-6 says the attribute
"must be included in the TMATS file if all other conditions apply even if it
has a default." D8/owner2026-11-13: missing required TFMT always emits TF-010
with MissingRequired reason; default policy performs no comparison. Opt-in
uses effective A, explicitly default-derived, retaining raw absence and that
finding. A mismatch may coexist. Published A is recorded even when unused.
Raw setup snapshots are immutable; there is no write-back path.

The table's A is not a numeric Format2 designation. Opt-in A in that context
is retained as default-derived but unmappable, with a diagnostic and no invented
NTP mapping. This source ambiguity is not silently resolved. Likewise caller
values are already structured: the library does not trim, case-fold, parse
TMATS syntax, infer an edition or normalize unknown spellings. Caller-qualified
packet fields preserve packet authority; unresolved field meanings use None.

## Governing boundary and factual findings

Half-open ranges use session and typed unwrapped RTC ticks, supplied by the
caller configuration timeline (contract2.3C/4.5, ADR-0005). No wrapping or
absolute-time reconstruction occurs here. Overlaps/duplicate channels are
rejected; exact boundaries select the new setup without resetting counters.
Version, RI4 and caller provenance are raw strings; RI4 year is optional parsed
caller data, never G\OD. Applying a new range clears stale RI4 context while
preserving explicit caller year and original packet payload.

Absent declarations, a present-empty setup and a gap in supplied ranges remain
distinct. prepare refreshes only governing admission flags and RI4 facts. Existing
core defaults exclude undeclared references; include_undeclared can admit them
with undeclared reference basis. PreparedObservation carries the full declaration
basis, as does every comparison finding. Each diagnostic retains its RDG,
raw/effective/default/inferred values, observed/expected bits and setup identity.
TF-010 is also the time-domain missing/unknown declaration diagnostic; generic
TMATS required-field validation is not implemented. Validity is Undetermined:
comparison severity never admits or discards decoded time.

## Clause ledger

| L2/L3 suffix | S09a evidence | Remaining |
|---|---|---|
| DECL-101 | All plain attributes/version/RI4/provenance, two TIMEIN channels with independent TTF/TFMT and per-channel basis/comparisons/preparation, half-open counter ranges, session isolation, original spelling, no payload/counter reset; RI4 versus G\OD structural boundary (review follow-up below) | Closed for caller-supplied facts; upstream TMATS extraction is external, not claimed tested |
| DECL-102 | Explicit absence/present-empty/gap, default exclusion and opt-in undeclared operation in offline and both stream modes; prepared/report basis | Universal propagation into legacy core answers and parser findings is S10b/S12; no whole-requirement marker |
| RDG-101 | One closed comparison module, only RDG001..004, inference provenance, numeric context gate | RDG005/006 added in S09b; no whole-requirement claim |
| RDG-201..204 | All current vocabularies, missing/unknown/reserved, every mismatch, bidirectional seven-letter table, RCC example, default provenance | Closed for caller-qualified comparisons; edition adapters/all-edition evidence remain separate |
| FND-104 | TTF/TFMT differences, inferred I mismatch warning, required/default cases | TSRC/SHTF and lock-loss information are S09b; no marker |

## Tests-first and executed checks

Tests-only `e099d37` had a fixture delimiter error, not behavioral evidence.
Corrected tests-only `2065f6d` fails on main with E0432: declarations API absent.
No runtime failing-first claim for this new API. Seven final tests run exhaustive
literal tables: 2×256 TTF/type comparisons, 7×16 Format1 comparisons, all reverse
values, 3×16 numeric NTF comparisons, raw unknowns, defaults, ranges, provenance
and offline/Standard/Interpolate admission. Additional assertions after the API
prototype are not retroactively claimed failing-first. No real recording/fuzz
campaign or fresh page checks. Mutation and full CI results follow execution.


Mutation checks:23 independent variants compiled and failed runtime assertions:
all seven letter/value pairs, half-open end, session isolation, TTF1 mapping,
silent/default-on-explicit application, missing finding suppressed by default,
raw absence lost, inferred flag hidden, numeric context ignored, mapping leaked
outside context, severity override ignored, reserved packet finding dropped,
absent mode remaining declared, external flag ignoring TIMEIN, stale RI4 year
and related setup identity replaced by the observed packet. Each starts from
saved source; original source restored at completion. No compile-error kills.
No source-global negative test is claimed to prove future unrelated code.

Whole markers are limited to DECL-101 and RDG-201..204, whose full statements
are exercised by the comprehensive tests. DECL-102, RDG-101 and FND-104 remain
clause-level, as above. Register Test/Analysis links are updated for001..004;
Design and Review fields are byte-for-byte unchanged. All18 bounded local CI-equivalent checks pass (jobs/threads2), including
MSRV1.85, no_std WASM±serde, CLI, benches, locked fuzz compile and cargo-deny.
Actionlint/diff/trace pass. Matrix160→170/288 (+10: five L2/L3 chains).
Hosted CI is verified at the reported head.


## PR29 review follow-up — DECL-101 and surviving mutants

Ordinary merge of current main5eeca7d (published types0.1.0) before changes;
merge82ebd1c retains both CHANGELOG entries. Frozen PR28 is untouched. No
production source, requirement statement/planned-verification item, dependency
policy or Design/Review changes are introduced by this follow-up.

The original DECL-101 test's second channel had no CDT, so the claim of testing
multiple TIMEIN channels was not supported. Its comprehensive test now calls
an additional fixture with TIMEIN7:TTF1/TFMTB/TSRCE/SHTF0 and
TIMEIN8:TTF2/TFMT2/TSRCI/SHTF2 in ONE setup. Both matching and swapped wire
data types retain the correct independent TTF/TFMT and complete per-channel
basis; preparation independently retains channel facts and governing RI4.
The original two-range/exact-boundary/session/absence/spelling tests remain.
Thus the full DECL-101 marker remains on that expanded comprehensive test.

### Recording date versus TMATS origination date — structural coverage

SetupDeclarations accepts only the named RI4 string and caller-extracted
recording_year as date inputs; it has no G\OD input, generic attribute map,
TMATS parser or date-extraction fallback. prepare copies only recording_year
into YearContext.recording. Consequently G\OD cannot be selected by this API.
The fixture uses RI4=2016-12-31/year2016, while opaque provenance explicitly
mentions G\OD=2025-01-01. It asserts the RI4/year2016 survives in both channel
bases and prepared year contexts, then clears RI4/year and asserts absence
without fallback despite that provenance. Provenance is not parsed or scanned.
This is structural separation plus copy/no-fallback testing, NOT a claim of
parsing a real TMATS file containing both attributes. Upstream callers must
supply the correct named fields; that TMATS extraction stays external. The
existing planned item is retained with this explicit interpretation, without
loosening either requirement statement.

### Executed review evidence

Nine tests pass on the merged implementation (seven existing tests expanded,
two new test functions). These are new assertions on correct code, not new
failing-first defect claims. Five isolated source mutations each compiled and
failed runtime assertions; saved production source was restored after each:

| Reviewer mutant | New assertion that fails |
|---|---|
| D21 reserved range6..15 ->6..=15 | exhaustive seven-letter ×16 Format1 comparisons assert NO TF-006 for FMT0..5/F, TF-006 for6..E; NONE/F now fails |
| D12 drop applicability from default_derived | missing TFMT on non-TIMEIN with opt-in must have default_derived=false and effective=None |
| D13 publish A on non-applicable channel | same fixture requires published_default=None |
| D24 omit applicability from reserved-observed finding | non-TIMEIN, observed FMT/NTF0..15, must never emit TF-006 |
| D7 omit recording-year range check |10000/u16::MAX rejected,0/9999 accepted |

Non-TIMEIN fixtures cover missing CDT, PCM and lowercase timein, opt-in enabled,
packet types0x11/0x12 and all16 format nibbles. Production files are unchanged
from the pre-mutation merged implementation. No compile-failure mutant counts.
Matrix stays170/288 (delta0 against reviewed PR29, +10 against current main),
with five whole L2/L3 chains. DECL-102/RDG-101/FND-104 remain partial as above.

All18 bounded local checks pass with locked Cargo resolution (jobs/threads2):
feature suites, MSRV1.85, no_std WASM±serde, CLI build/clippy, benches, all-fuzz
compile, fmt/docs/trace and cargo-deny. Actionlint and diff check pass. Hosted
CI is verified at the reported head. No new source/page review or fuzz campaign.
