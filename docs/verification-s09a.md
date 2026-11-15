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
| DECL-101 | All plain attributes/version/RI4/provenance, multiple channels, half-open counter ranges, session isolation, original spelling, no payload/counter reset | Closed for caller-supplied facts; upstream parsing remains external |
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
