# S17 — bounded time-view consumer seam

**Status:** design proposal for the consumer adapter; no new API implemented.
Owner constraints are accepted in [ADR-0021](../adr/0021-tmats-release-and-bounded-time-view.md)
(2026-11-21). Original design baseline main `27befa7`; #78 is now merged.
This note changes no published TMATS API or external coordination plan.

## Release and dependency boundary

0.8.0 waits for BOTH a full irig106-tmats 0.1 release and a bounded time view.
A name placeholder, partial preview or unbounded document view does not clear
that gate. Adding the future dependency also requires separate owner approval
after publication; this note grants none. No TMATS dependency, parser, repository
edit or upstream release assertion is made here. The time library remains a
leaf depending on shared types, with caller-supplied plain declaration data
(ADR-0005/0006). A future approved CLI adapter owns TMATS parsing/I/O.

The precise upstream bounded-view API, published version and limits remain to be
inspected when available. Do not freeze speculative public Rust signatures now.
The following consumer contract can be tested with synthetic plain data without
requiring that upstream API or a TMATS dependency.

## Two separate facts: declarations and data-source association

A governing setup can describe channel 7 as TIMEIN and give its TTF/TFMT/TSRC.
That permits comparisons against observed time packets on channel 7. SHTF is
independently applicable to a declared channel with a secondary header. None of
these declarations proves that a data packet on channel 8 uses channel 7's time.

The owner-relayed TMATS review states: "Chapter 9 has no field linking a data
channel to its time channel". This is the supplied review constraint, not a new
page inspection or a fabricated standard quotation. No relationship may be
inferred from equal channel IDs, the sole TIMEIN channel, lowest ID, first/nearest
reference, a recorder-wide RTC, or default TFMT. Static selection of a reference
for a counter query (accepted stream Option A) remains a policy choice; it is
not evidence of a data-to-time-channel relationship.

Until real recording-specific evidence identifies the relationship and relevant
session/segment, report data-row association as unresolved with retained raw
counter/context. Keep existing conservative reset behaviour. Do not remove or
reinterpret library counter-query policies; do not invent a Chapter 9 link or a
new finding ID. An eventual association input must retain the actual evidence
and its origin separately from raw time-channel declarations.

### Association states (owner-relayed tmats #9 vocabulary)

Use three distinct states at the consumer seam:

- `association_unestablished`: no real relationship evidence is available;
  keep the association unresolved for answers.
- `association_conflicting`: retain the conflicting candidates/evidence;
  keep the association unresolved for answers, without silently selecting one.
- `association_evidenced`: real recording-specific evidence establishes the
  relationship; still qualify its governing session and segment before use.

These are the coordinator-supplied names from tmats #9, not a claim that a
particular published Rust API already exists. An evidenced association does
not authorize bridging sessions/resets or replacing counter-selection policy.
No dependency, inference rule or relationship is created by naming the states.

## Proposed adapter boundary

Use existing time-side data shapes, not upstream document objects:

| Consumer fact | Existing time-side shape / boundary |
| --- | --- |
| Session and unwrapped half-open coverage | PacketKey / GoverningRange; no re-unwrapping or crossing reset sessions |
| Governing setup identity and origin | SetupDeclarations / GoverningSetup: setup_packet, provenance, declared_version |
| Actual raw attributes and explicit absence | ChannelDeclaration: CDT, TTF, TFMT, TSRC, SHTF; preserve missing/unknown values |
| Year evidence | RI4 plus caller-extracted recording_year; never `G\OD` |
| Missing view vs range gap vs governing setup | DeclarationPresence::Absent / NoGoverningRange / Present |
| Packet/layout evidence | PacketFields plus S03 QualifiedCsdw and original flags |
| Comparison result and basis | compare_qualified / DetailedComparison; reduce findings into bounded report counts/samples |
| Data-to-time association | Separate three-state input above; NOT a ChannelDeclaration field or invented TMATS relationship |

A future provider advances over setup boundaries and supplies only the governing
slice needed for the current session/counter window. The CLI adapts that slice
into these plain shapes. Existing Declarations::present(Vec<SetupDeclarations>)
validates snapshots but does not impose a storage cap itself: never fill that
Vec with every setup in a many-GB recording and call it bounded. Construct a
bounded current/window view; retain only the provenance needed by live answers.

Proposed resource contract: caller-selected caps for active setup/channel data
and retained range history; no hidden queue or complete document/recording in
memory. A request outside available coverage stays unresolved with its reason;
it must not reuse the most recent setup silently. Handle late packets only
within retained coverage or through an explicitly bounded provider lookup.
Spool report history when necessary using the existing temp-file pattern.
Exact cap values and the adapter signature await the actual published view;
they are proposals, not new owner-approved defaults.

## TFMT integration without defaults inserted

For the governing TIMEIN channel, keep raw TTF/TFMT/TSRC and the setup/range basis.
Run the existing RDG001..006 comparisons only. Missing required TFMT is always
reported. Published default A participates only under explicit caller opt-in,
labelled default-derived; it never replaces the missing raw attribute or removes
its finding. I<->FMT 3 remains explicitly inferred (RDG003). Every raw letter/value
pair and the RCC 124-22 Chapter 7 example already have S09a evidence; the adapter
must preserve, not redo or broaden, those mappings.

Retain packet-layout qualification: unresolved and 106-04 EXT layouts do not
invent SRC labels or TSRC comparisons. Channel selection/admission and packet
comparison remain separate operations. An empty collection of setup ranges is NoGoverningRange, not a governing
setup. A covering setup with an empty channel list is Present and can report
TF008 for an undeclared time channel. Preserve the S09b applicability rule. Retaining a setup basis is not evidence of data-source association.

## Work that can proceed now / verification still planned

Independent S17 slices may complete protocol/scale/clock rendering, JSON/text
projection and golden coverage, finding/exit/writer tests, and bounded report
state over synthetic caller data. #78 schema fixes remain in their review cycle;
this note does not unfreeze or alter its reported head. Real-file TMATS extraction,
the final upstream adapter and its release integration stay blocked on the gate.
Inferred/default TFMT synthetic projection tests must be labelled as caller-data
tests, not proof that the CLI extracts real TMATS.

Future acceptance probes (not executed or marked covered here):

- Two TIMEIN channels with independent attributes/bases over successive governing
  ranges; missing setup, present-empty, gaps, unknown attributes, and cap overflow.
- Required TFMT missing with opt-in off/on; preserve the missing finding and raw
  absence, default-derived label, I inference, every letter/value mapping.
- A data channel with one or several TIMEIN channels still has unresolved
  association absent real evidence; repeat after resets and overlapping segments.
- Explicit real relationship evidence, once specified, is retained with its
  session/segment and never inferred merely from the global RTC or channel ID.
- Many setup changes and late packets over a small retained window: state stays
  bounded; evicted coverage returns unresolved, not a stale declaration.
- Same comparison facts/basis in text and JSON; writer failures and the existing
  dense memory gate remain effective. No new requirement coverage is claimed.
