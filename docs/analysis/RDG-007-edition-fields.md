# RDG-007 — qualified Format 1 fields (S03)

This analysis reuses STANDARD-REVIEW T-6/T-9/T-14 and the reviewed sources in
[Q-EDITION](Q-EDITION-2026-11-13.md). No new page-image inspection is claimed.
Design and Review fields in the register are unchanged; human review is pending.

106-05/07/13/15 Chapter 10 says "Reserved. Bits 31-12 are reserved";
106-17+ §11.2.3.2 a defines ITS "when FMT is IRIG-A, B, or G" and
"0000 = IRIG TCG freewheeling (no or loss of time source)". Time Format1's
current data-type version remains 0x06 (Table11-4), so this header cannot
establish ITS applicability. Figure11-34 gives "0x0C = RCC 106-17" in the
separate setup domain. These quotes are preserved in RDG-007/T-14; current
Figure11-12 is printed11-24/PDF32. Historical EXT/SRC and version evidence
page numbers are in Q-EDITION (04 Fig10-15, 05 Fig10-16a, 07 Fig10-50).

Apply the accepted Q-EDITION policy before interpreting fields: override wins;
opted-in G\106 requires no contradictory packet evidence; neither a raw setup
code nor header0x01 alone selects a layout. Setup declarations remain distinct
metadata. An unadopted TMATS-file edition or differing declarations alone do
not create an automatic conflict. No fictional 04/05 setup code is introduced.

Resolved pre17 ITS is NotReported, including zero; resolved17+ A/B/G reports
all 16 numeric meanings, including reserved8..15. Other formats report
NotApplicable. Unresolved A/B/G preserves zero as FreewheelingOrNotReported,
nonzero as Unresolved(raw); non-A/B/G cannot acquire ITS just from nonzero bits.
Raw intersections for all three reserved masks remain available. Only bits
reserved in every candidate interpretation support an unconditional TF-005
when layout is unresolved; possible-only violations remain labelled alternatives.
Legacy EXT is never relabelled as modern SRC; nibble3 is never GPS or None.

TF-031 is emitted only for the no-override contradictory adopted packet edition.
WARNING is **proposed, pending owner confirmation**, implemented at coordinator
direction: common readable time fields stay usable, EXT/SRC qualification is
withheld. Override wins even against a newer header; evidence stays side by side
without an automatic conflict finding. This neither changes TF-001..028 nor
assigns a default to TF-032. See verification-s03 for clause limits and evidence.
