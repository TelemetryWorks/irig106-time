# S03 — edition evidence resolver and qualified Format 1 fields

Baseline main `15ee379666442a86dab55c2d32781143462e2f5e` (S02 merged).
Scope: the eight S03 L2/L3 pairs in the saved plan. No dependency, CLI,
Design/Review field, TF-001..028 row or TF-032 default changes. TF-031 WARNING
is **proposed, pending owner confirmation**, implemented at coordinator direction;
it is not an owner-approved severity. Confirm it before the release audit.

## Tests-first provenance

- Tests-only `ed7a647`: TF-031 policy availability **failed by assertion on main**
  (the S01 catalog deliberately refused its then-pending level). The test was
  subsequently renamed to distinguish provisional availability from approval.
- Tests-only `662d5bb`: seven new resolver/qualified-reading tests could not
  compile on main: APIs did not exist. This is missing-API evidence, not seven
  executed failing tests. Existing explicit-layout source/FMT/mask tests already
  passed on main and are reused, not falsely claimed as new fixes.
- Tests-only `77b1fbf`: unresolved non-A/B/G ITS applicability **failed by
  assertion against the first uncommitted implementation**, not main. FMT15
  has different invalid-payload labels, but neither candidate is A/B/G; it must
  not acquire a possible-freewheeling ITS label. Fixed before the final suite.
  The stream-basis probe initially assumed time observations emit answers;
  it was corrected to observe a data query at the same RTC coordinate.
- `e37d85f` and later assertions strengthen raw alternatives, operand retention,
  override/no-finding and reserved-ITS coverage after implementation; no original
  failing-first claim. Final S03 suite: **eight tests**, plus existing catalog,
  version, wire and S01/S02/timeline regressions.

## Sources and literal fixtures

Reuses the reviewed page evidence in STANDARD-REVIEW T-6/T-9/T-14 and
Q-EDITION analysis; **no fresh page image was inspected in S03**.

- 106-04 Fig10-15, printed10-30/full PDF288, definitions10-31/289: EXT bit0,
  reserved3..1; FMT6..15 reserved. 106-05 Fig10-16a, 10-37/PDF41: SRC and
  FMT15 NONE. Literal nibble3 is old external-present plus reserved0x2, modern
  Reserved(3), or unresolved raw3; never GPS/None. All 16 source/FMT values
  and every bit of the three reserved masks are checked independently.
- 106-24R1 §11.2.1.1 e, 11-6/PDF14: "a value at or below the release version
  of the standard applied to the data types". Header01 alone is unresolved;
  it is allowed below an adopted24 declaration. Adopted04 plus known header0A
  contradicts it; unknownFF is retained, not numerically treated as a release.
- 106-04 setup §10.6.7.2.2/Fig10-34, 10-48/full PDF306 has no version field;
  106-05 Fig10-34b,10-55/PDF59: "Reserved. (Bits 31-0) are reserved."
  106-07 Fig10-50,10-72/PDF76 first names CH10VER07. Caller-qualified reserved
  setup words never become version codes. Shared code mappings remain separate;
  setup declarations alone do not select packet layout or create conflicts.
- G\106 describes the TMATS file (106-04 Table9-1,9-8/full PDF186), not
  necessarily the packet. Owner Q-EDITION requires caller opt-in. Override wins
  with all evidence retained, following TMATS ADR-0028 at41030d5: "No automatic
  conflict finding"; "differing declarations are shown side by side". A raw
  differing/unadopted declaration is metadata, not a contradiction by itself.
- RDG-007 analysis supplies ITS applicability; table rows cover04/05/07/09/11/
  13/15/17/19/20/22/23/24/24R1. The same zero word is not-reported before17,
  freewheeling only for applicable resolved17+ A/B/G, or explicitly ambiguous.
  Literal CSDW bytes01 80 00 00 mean SRC1/FMT0/ITS8: pre17 reserved-bit warning,
  applicable17+ reserved-value warning, unresolved alternatives otherwise.
- The hand body03 00 00 00 00 00 00 00 01 00 is FMT0/day1/midnight with
  source nibble3. Its common readable calendar is kept through both engines
  even when EXT/SRC is unresolved. Bases retain evidence; TF-031 retains actual
  header0A. No leap, era or automatic year reconstruction is introduced.

## Clause-level coverage

| L2/L3 pair | S03-covered clauses | Remaining / boundary |
|---|---|---|
| CSDW-101 | Four-byte/bounded Data Length reading, immutable resolver before qualification, raw LY/DATE via original CSDW, all field meanings and raw alternatives; common readable body retained. | Legacy explicitly supplied layout helpers remain compatibility APIs. CLI must adopt evidence-qualified entry points in its planned restructure. |
| CSDW-102 | All EXT/SRC nibbles, unresolved alternatives and placeholder Unresolved(raw) through references/bases; nibble3 never None/GPS; raw EXT reserved bits. | No TMATS parser/runtime dependency; caller supplies declarations. |
| CSDW-103 | All FMT values, legacy15 reserved/current15 None, unknown15 unresolved. Unsupported/invalid body produces no time. | Unknown15 adapter returns NeedsContext and caller retains its qualified reading, never a fabricated modern None label. |
| CSDW-106 | Every ITS value with A/B/G and other FMT across rule rows; explicit not-reported/not-applicable/unknown states; reserved ITS value warning with usable body kept. | No edition inference from a setup/header code alone. |
| CSDW-107 | All bits of three masks; selected intersection or unconditional common-mask violation; raw alternatives retained when unresolved; exact TF-005 operands, separate TF-006. | Alternative-only violations remain competing interpretations, not certain TF-005. Existing legacy layout APIs retain their documented strict validation entry point. |
| EDN-103 | Cited rule boundaries for EXT/SRC/ITS, Chapter10/11 and Format2 availability; independent edition row fixtures and missing/unknown evidence. | **Partial historical scheduling evidence:** no historical late-packet guarantee is invented from code-list names. Existing cited 1100ms policy remains unchanged; per-edition scheduling/legacy archive qualification remains a release evidence audit item, not a new release-scoped waiver. |
| EDN-201 | Every accepted resolver case, override precedence, opt-in G\106, known above-release contradiction, lower permitted hint, distinct setup/header domains, raw side-by-side evidence in both answer bases; provisional TF-031 only without override. | TF-031 level needs owner confirmation. Validated serialized schema/report adapters remain S12/S17; no host-clock/fallback selection. |
| RDG-207 | Written analysis, source inventory, historical/current/unknown ITS fixtures and reserved meanings. | Human Review stays pending. No claim of new page-image inspection or independent human reading review. |

No new release-scoped deferral. Matrix links cannot express partial clauses or
pending human review: **128→136/288** is link coverage, not full compliance.

## Targeted negative checks

Twenty isolated mutations compiled and failed by assertion: override discarded;
TMATS adopted without opt-in/replacing override; header01 guesses04; lower
header incorrectly conflicts; unknownFF incorrectly conflicts; contradiction
left resolved; TF-031 omitted/wrong level/header operand lost; unresolved
nibble3 guessed as SRC; legacy/unknown FMT15 promoted to None; legacy or unknown
ITS zero guessed freewheeling; EXT mask lost; unresolved mask selects legacy;
reserved ITS finding lost; CSDW reserved operand lost; unresolved edition drops
the body. All sources restored. Mutation failures are separate from main-first
evidence. Additional assertions after implementation are labelled above.

Full bounded CI equivalent: feature tests, MSRV1.85, no_std wasm ±serde,
clippy/fmt/docs, CLI build/clippy, benches build, locked all-fuzz-target check,
trace freshness and cargo-deny; worker cap2. No publishing or merge performed.
