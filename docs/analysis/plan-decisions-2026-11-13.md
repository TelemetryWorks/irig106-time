# 0.8.0 plan — accepted decisions and source audit

Owner decision, **2026-11-13**: "accept your recommendations", for
`coordination-work/irig106-time-0.8.0-plan.md` saved by the coordinator.
That local coordination artifact is not in this checkout. The coordinator's
PR F assignment supplies the decisions below and explicitly reserves the
TF-031/032 **levels** for later owner confirmation. This record does not
reinterpret the older roadmap Q1–Q9. Baseline main: `4a5e752` (S00 merged).
[ADR-0018](../adr/0018-0-8-release-scope-and-policy-decisions.md) records the
accepted policy; earlier ADRs and historical verification remain intact.

## Q1 — permanent finding identities

TF-001..028 rows remain byte-for-byte unchanged. The new catalog entries are:

| ID | Approved title | Level | Value effect / boundary |
|---|---|---|---|
| TF-029 | NTP time is zero (unknown or unsynchronized) | WARNING | Both seconds and fraction zero: no absolute value, raw kept. |
| TF-030 | NTP time falls in a leap-second ambiguity window | WARNING | Covered duplicate insertion coordinate: ordinary mapped value kept, labelled leap-ambiguous. Distinct from TF-022 explicit second 60. |
| TF-031 | Packet-layout edition evidence is contradictory | level pending owner (S03) | Existing no-override conflict case only. No severity chosen here. |
| TF-032 | Year evidence is contradictory | level pending owner (S08) | Preserve contradictory year candidates. No severity chosen here. |

Verbatim sources (reusing the cited evidence, not claiming a new page check):

- RFC 5905 §6, **p.14**: "a value of zero is a special case representing
  unknown or unsynchronized time". Source: [RFC 5905](https://www.rfc-editor.org/rfc/rfc5905.html#section-6).
- RFC 1305 Appendix E.8, **p.78**: "the NTP timescale is in effect reset to
  UTC as each timecode is received"; "knowledge of all previous leap seconds
  is lost." Figure 8, **p.79**, repeats a coordinate for insertion. Source:
  [RFC 1305](https://www.rfc-editor.org/rfc/rfc1305.html); RDG-012's existing
  [analysis](RDG-012-ntp-utc-era.md) defines the [B,B+1) window and table coverage.
  RDG-010's explicit leap-label rule and TF-022 remain unchanged.
- 106-24R1 §11.2.1.1 e, **printed 11-6 / PDF 14**: "a value at or below the
  release version of the standard applied to the data types". 106-04 Table
  9-1, **printed 9-8 / full PDF 186**, G\106: "VERSION OF IRIG 106 STANDARD
  USED TO GENERATE THIS TMATS FILE". The latter describes TMATS, not necessarily
  packet layout. [Q-EDITION evidence](Q-EDITION-2026-11-13.md) and TMATS
  ADR-0028 at `41030d54dfbd8a539c2810d023dd0670bfd3dc1f` retain override precedence:
  an explicit override wins, contradictory evidence remains side by side;
  differing declarations alone and an allowed lower header version are not
  automatic conflicts. TF-031 does not add such a conflict.
- 106-24R1 Figures 11-13/14, **printed 11-25 / PDF 33**, distinguish
  the DMY year field (Figure 11-14) from the day-only body (Figure 11-13).
  Contract **5.3** supplies the reader's year-evidence precedence. The
  standard does not specify a TF identifier or contradiction severity;
  TF-032 preserves evidence without changing the year policy. Existing
  L2/L3-COR-110 Sources and all their quotes are retained.

These standards/RFCs describe evidence and time semantics, not our TF numbers
or levels. The owner approves those product choices. Implementing slices must
propose TF-031/032 levels for confirmation; an absent level is not permission
to silently use WARNING/ERROR. Runtime producers and the existing 1..28
validation range are unchanged by this documentation-only PR.

## Q2 — inverse query semantics

Absolute ranges are **[start, end)**. Return **every** matching half-open RTC
tick interval, retaining quantization and basis; a point query is separate.
Never bridge gaps or jumps, and retain unresolved boundaries. The standard
provides the RTC scale, not an inverse-query API: §11.2.1.1 i,
"a free-running 10-MHz binary counter". Contract 4.9/5.5 and L1-COR-011 are
the product intent; half-open boundaries are the accepted owner policy, not
an IRIG mandate. This feature remains deferred by Q3.

## Release scope

Q3 approves exactly four release-scoped deferrals:

| Requirement (L2 and L3) | Deferred portion | Still required before 0.8.0 |
|---|---|---|
| COR-111 | Inverse queries | Retain the requirement and Q2 semantics; no coverage claim. |
| ENC-103 | New complete secondary/stamp encoder API | Existing exposed/used encoders and correct byte-sum behaviour. |
| API-110 | Complex timeline-state rehydration through deserialization only | Disable/omit unsupported complex Deserialize; existing public-value loads validate; complete versioned policy/answer/timeline outputs and Q5 schema. |
| REL-106 (conditional) | Prebuilt binary distribution | No binaries shipped; retain future binary SBOM/provenance obligations. |

Requirements are not deleted or marked covered. **Every other requirement
remains a 0.8.0 blocker.** Offline drift-corrected query mode is after 0.8.0;
this is distinct from stream interpolation and does not expand the four
requirement deferrals. This is owner release scope, not a standard exemption.

## Q4 — publication and distribution

0.8.0 ships `irig106-time` and `irig106-time-cli` to **crates.io only**, in
lockstep (ADR-0007/0017). No prebuilt binaries. All TelemetryWorks publication
uses **crates.io Trusted Publishing/OIDC from GitHub Actions only**: no stored
publishing token and no local publish. IRIG 106 is silent on publishing.
See [release prerequisites](../RELEASING.md) for official Rust sources and
remaining preparation; no workflow, credentials, settings or publication is
performed or authorized by recording the policy.

## Q5 — snapshot/report JSON

Use a versioned schema with units; **64/128-bit quantities are decimal strings**.
Validate supported loads and retain raw/stated/assumed provenance. Review the
schema once before freeze. Only the Q3 complex timeline rehydration subset
is deferred; unsupported complex Deserialize is omitted/disabled for 0.8.0.
ADR-0010/0011 require reproducible policy and basis; L1-POL-003, API-007/010
and CLI-006 are retained parents. IRIG 106 defines wire units, not JSON/schema
syntax; this representation is accepted owner policy. No schema is falsely
claimed frozen or implemented here.

## Q6 — CLI simultaneous failures

Classes remain 0 success; 1 threshold findings; 2 usage; 3 input; 4 output.
Usage is checked **before input**. The **first fatal operational failure**
determines 3 or 4; later failures become diagnostics, not replacements for
that fatal classification. Keep the pinned `irig106-tmats` draft source and
ADR-0008 hand-rolled parser. IRIG 106 is silent on tool exit codes and their
precedence. Planned restructure tests inject both failure orders, failed
output while reporting input failure, and usage rejection before file access.
No current CLI runtime change is claimed.

## C1–C4 — accepted defaults

| Choice | Accepted default | Source boundary |
|---|---|---|
| C1 | Stream interpolation floors the checked rational displacement. | Owner 2026-11-13; §10.6.2 "minimum frequency of 1 hertz" and RCC 123-20 §5.6 do not specify rounding. |
| C2 | Nearest-reference midpoint tie chooses preceding. | Owner 2026-11-13; RCC 123-09 §6.6 "a time packet that occurs near the current data packet" does not define ties. |
| C3 | Mixed-resolution jump threshold defaults to the coarser endpoint resolution. | Owner 2026-11-13; contract 5.4; caller may override. Not a standard-prescribed tolerance. |
| C4 | A mode change drains under the old policy before installing the new one. | Owner 2026-11-13; ADR-0010/0011; retain old policy/fallback basis, no revision. No standard defines our API transition. |

These decisions remove their pending-owner labels from normative docs and Rust
**doc comments** only. Behaviour, tests and output remain unchanged. In
particular, the existing runtime basis string `floor (L3 proposal)` is retained
verbatim, including its tests. Replacing that output is a separate behaviour
migration; the historical string no longer represents an unresolved decision.

## Requirement and evidence audit

The following table lists every changed requirement, its unchanged parent and
its new Source/Release-scope addition. Original Source text/quotes, verification
methods, identifiers and parent links remain. L1 is unchanged. No Design or
Review fields change. Matrix regeneration does not promote evidence or certify
partial clauses. Existing S00/E1/E2 records retain their historical evidence,
with a dated link to this later acceptance; no historical tests are re-run or
represented as newly failing first.

<!-- requirement audit -->
| Changed ID | Unchanged parent | Source addition / scope |
|---|---|---|
| L2-NTP-104 | L1-NTP-004 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-TAI-107 | L1-TAI-007 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-ENC-103 | L1-ENC-003 | Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes. |
| L2-EDN-201 | L1-EDN-002 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-COR-102 | L1-COR-002 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L2-COR-104 | L1-COR-004 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L2-COR-110 | L1-COR-010 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-COR-111 | L1-COR-011 | Owner 2026-11-13: "accept your recommendations" for the 0.8.0 plan; [0.8 plan decisions](plan-decisions-2026-11-13.md). This is reader/release policy, not an added IRIG mandate. |
| L2-POL-101 | L1-POL-001 | Owner 2026-11-13 accepts plan C1–C4; [decision/source record](plan-decisions-2026-11-13.md). |
| L2-POL-102 | L1-POL-002 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L2-POL-103 | L1-POL-003 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L2-POL-201 | L1-POL-001 | Owner 2026-11-13 accepts plan C1–C4; [decision/source record](plan-decisions-2026-11-13.md). |
| L2-FND-101 | L1-FND-001 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-FND-102 | L1-FND-002 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-RDG-212 | L1-RDG-002 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L2-STRM-202 | L1-STRM-001 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L2-STRM-206 | L1-STRM-001 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L2-API-107 | L1-API-007 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L2-API-110 | L1-API-010 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L2-CLI-106 | L1-CLI-006 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L2-CLI-107 | L1-CLI-007 | Owner 2026-11-13, plan Q6: usage before input; first fatal operational failure determines code 3 (input) or 4 (output), later failures are diagnostics. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 is silent on process exit codes and their precedence. |
| L2-REL-101 | L1-REL-001 | Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes. |
| L2-REL-106 | L1-REL-006 | Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes. |
| L3-NTP-104 | L2-NTP-104 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-TAI-107 | L2-TAI-107 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-ENC-103 | L2-ENC-103 | Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes. |
| L3-EDN-201 | L2-EDN-201 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-COR-102 | L2-COR-102 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L3-COR-104 | L2-COR-104 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L3-COR-110 | L2-COR-110 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-COR-111 | L2-COR-111 | Owner 2026-11-13: "accept your recommendations" for the 0.8.0 plan; [0.8 plan decisions](plan-decisions-2026-11-13.md). This is reader/release policy, not an added IRIG mandate. |
| L3-POL-101 | L2-POL-101 | Owner 2026-11-13 accepts plan C1–C4; [decision/source record](plan-decisions-2026-11-13.md). |
| L3-POL-102 | L2-POL-102 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L3-POL-103 | L2-POL-103 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L3-POL-201 | L2-POL-201 | Owner 2026-11-13 accepts plan C1–C4; [decision/source record](plan-decisions-2026-11-13.md). |
| L3-FND-101 | L2-FND-101 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-FND-102 | L2-FND-102 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-RDG-212 | L2-RDG-212 | Owner 2026-11-13, plan Q1: permanent TF-029/030 WARNING with the stated value effects; TF-031/032 IDs/titles approved, levels pending owner in S03/S08. [0.8 plan decisions](plan-decisions-2026-11-13.md). |
| L3-STRM-202 | L2-STRM-202 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L3-STRM-206 | L2-STRM-206 | Owner 2026-11-13 accepts plan C1–C4: floor interpolation, preceding midpoint tie, coarser mixed-resolution jump threshold and old-policy drain. [0.8 plan decisions](plan-decisions-2026-11-13.md). These are selectable reader-policy decisions; original standard/handbook sources remain below or above. |
| L3-API-107 | L2-API-107 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L3-API-110 | L2-API-110 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L3-CLI-106 | L2-CLI-106 | Owner 2026-11-13, plan Q5: reviewed versioned snapshot/report JSON schema with units, 64/128-bit quantities as decimal strings, validation on load and raw/stated/assumed provenance. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 does not prescribe JSON representation. |
| L3-CLI-107 | L2-CLI-107 | Owner 2026-11-13, plan Q6: usage before input; first fatal operational failure determines code 3 (input) or 4 (output), later failures are diagnostics. [0.8 plan decisions](plan-decisions-2026-11-13.md). IRIG 106 is silent on process exit codes and their precedence. |
| L3-REL-101 | L2-REL-101 | Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes. |
| L3-REL-106 | L2-REL-106 | Owner 2026-11-13, plan Q3/Q4: exact release-scoped deferrals and crates.io-only 0.8.0; Trusted Publishing/OIDC from GitHub Actions only, no stored publishing token or local publish. [0.8 plan decisions](plan-decisions-2026-11-13.md). This policy does not authorise publication or settings/workflow changes. |

This table describes changed obligations, not implemented coverage. The matrix
remains a link checklist; partial clauses and release deferrals are recorded
in requirements and verification records. S01 needs a separate go after PR F
merges and is not started here.
