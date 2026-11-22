# S17 item2 — F1 reference scale/basis through rows and answers

Base maina554c97(#93). Partial L1-CLI-006 -> L2/L3-CLI-106; no whole
Requirements marker, Review/Design, dependency or library change. CSV's19
columns and JSON's existing field/type/unit/enum contracts are unchanged.
Stable schema v1/revision6 already permits these F1 values; description updates
clarify their actual propagation, not a new field or enum requiring a revision.
Item3's future policy/default fields will require an additive revision.

## Accepted basis, not a new scale policy

RDG-009 says IRIG-A/B/G are UTC by assumption, labelled "assumed"; FMT3 is
unknown unless the caller names its scale. The existing f1_basis() already
applied that rule to channel/segment inventory. §11.2.3.2 a states:
"0x4 = Universal Coordinated Time (UTC) time from GPS" and
"0x5 = Native GPS Time" (quotes retained in INTERPRETATIONS.md). Figures11-12/13,
printed11-23/25(PDF31/33), supply the literal FMT0..5/SRC1/DATE0 fixtures;
no new standard page inspection is claimed. Expected DOY10012:00:.12 has
(99*86400+12*3600)*1e9+120000000 =8596800120000000ns. There is no year.

The defect was a PTP-only filter between the observed reference and packet/query
projection/private spooling. Extend the same bounded metadata path's canonical
labels to F1: UTC / assumed, RDG-009 for0/1/2, unknown / unresolved for3,
UTC / stated FMT for4 and GPS / stated FMT for5. Text adds the matching explicit
suffix, including scale unknown. Both DOY and DMY decode paths already meet
at observe_reference after parsing; numeric parsing/arithmetic is untouched.
No TAI/UTC shift is applied to native GPS, no year/origin invented. A numeric
reference choice is not evidence of a data-channel/time-channel relationship.

## Tests-first and selection evidence

Tests-onlya63564d has three runtime failures on main. All three pass after the
fix. Six-format tests pin time-packet and subsequent data-row clocks, any/per-
channel answers, text suffixes, unknown year, second and nanosecond values.
Mixed F1 ties in both packet orders show a UTC packet's clock using the selected
lower-ID GPS reference: the scale stays GPS, not the packet's UTC. A later IRIG
reference on the same channel must not relabel an old GPS query target.

The40->1 reset fixture alternates FMT labels deliberately to distinguish each
basis. It pins reset-own row labels and selected-reference GPS/assumed-UTC
labels through the closed-segment spool, while any-channel remains withheld.
This verifies metadata transport under the existing legacy selection policy;
it does not claim new mixed-scale session/barrier semantics. Numeric policy,
selection, timing, finding levels and no-revision rules do not change.

Nine compiling mutations fail the new focused tests: discard assumed UTC,
stated UTC, GPS or unknown labels; use a decoded packet's latest own-channel
basis instead of the selected reference; keep the private loader PTP-only;
keep target brackets PTP-only; keep reset-own labels PTP-only; relabel GPS as
UTC in JSON. Source restored. Existing duplicate/tie/eviction metadata tests
continue to pass; they now expect the explicit unknown label rather than None.
Existing packet/query/reset goldens change only their F1 labels; CSV goldens
and numeric counts remain identical. No captured output supplies expectations.

Local checks: fmt, focused report tests/mutations and trace/schema-sync/diff.
Platform/package/security/fuzz and32MiB RSS evidence is from PR CI. Source uses
existing static labels and bounded metadata/spools; no added recording history.
Coverage remains198 tested/206 verified of288, delta0/0. Policy/default identity,
NTP era context, automated schema validation, native sessions/quality and final
docs/migration remain ordered items3..7. TMATS declarations remain gated.
