# Owner decisions of 2026-11-12/13 — requirement and source audit

Documentation-only update based on origin/main
`b839c4343cd8bddc7509ce0462cc86923a3f2737` (merged PR7).
Before checkout changes, origin was verified as
`https://github.com/TelemetryWorks/irig106-time.git`; initial HEAD was
`1c61ff7381de491e3eee8539331968633b2dd70b`. The new work branch starts from
that current origin/main. No source/configuration/dependency changes,
installation, merge, publication or archive modification is part of this PR.

The detailed Source-line audit below is the historical PR8 snapshot at
`48001e3c60faef4b74daef4f49b5adc13340f1c8`; later requirements supersede it.

## Decisions and their source limits

| Decision | Accepted result | Verbatim support / policy distinction |
|---|---|---|
| Q-DEFAULT | Standard/preceding reference is stream default; explicit Interpolate remains | RCC123-20 §5.6, printed5-44/PDF78: “clock time of a subsequent data packet”; ADR-0010 requires cited selectable defaults. The default is owner policy. |
| Q-STREAM-BOUND | Hold1 s+100 ms, gap tolerance0 ns independently; explicit caller progress/caps, no hidden queue; cap/timeout/EOF releases extrapolated from preceding time, or counter-only without one | Ch10 §10.6.2: “minimum frequency of 1 hertz”; §10.6.1b: “packet generation times shall be equal to or less than 100 milliseconds”; §10.6.1c: “stream commit time equal to or less than 1000 ms”, all printed10-26/PDF32. These support the chosen tolerance and explain late-written packets, not a reader algorithm mandated by IRIG. |
| Q-STREAM-CHANNEL | Option A commits before data: caller-fixed, else declared external TIMEIN, else lowest ID; no future counts, revisions or lost-lock switch. Undecidable -> counter-only until caller choice | RCC123-20 §5.6: “it is incumbent on the programmer or data analyst to determine and select the best source of time”; Ch9 Table9-4, printed9-50/PDF56: “TIMEIN” / “IRIG Time Input”, printed9-72/PDF78: “E” / “External”; Ch11 §11.2.3.2 note, printed11-26/PDF34: “the time source shall indicate Internal (0x0)”. Ranking is owner policy, static declaration and dynamic source are separate. |
| Q-CH4 | Explicit binary origin or BCD year/full day; otherwise labelled partial; no invented day1/year/origin and no nearest-packet option now. Characters/bits wording remains unresolved | Ch4 §4.7, printed4-11/PDF15: “resolution of 655.36 seconds”, “10 milliseconds”, “three least significant bits of the BCD Julian date”, “at a rate necessary to resolve time ambiguities”. RCC123-20 B-43/PDF149 `enI106_Ch4Binary2IrigTime` applies weights, no origin. Missing-origin policy is the owner's boundary, not a new inferred standard origin. |
| Q-NTP | RDG-012 accepted: UTC-following reckoning, evidenced/assumed eras, zero sentinel, leap ambiguity/currency; PTP UTC with last-known-offset fallback/TF-021 outside coverage | Full verbatim sources, seven-edition page checks, RFC numbers/sections/pages, hand arithmetic and planned test cases in [RDG-012 analysis](RDG-012-ntp-utc-era.md). Floor rounding accepted, owner 2026-11-13. |
| Q-FLAGS | Refine: source=1 requires a present header and defined format; source=0/format=11 retains RTC with TF-006 warning, separately reporting an unreadable secondary time if present | [Verbatim flag/RTC sources and all-edition page checks](Q-FLAGS-2026-11-12.md); owner 2026-11-12, Q-FLAGS. The finding severity/value-retention rules are owner policy. |
| Q-CLI-EXIT | Align with TMATS: 0 success, 1 threshold findings, 2 usage, 3 unreadable/invalid Chapter 10/time input, 4 output-write failure; never renumber | [Pinned repo/path/commit, verbatim draft and planned tests](Q-CLI-EXIT-2026-11-12.md). Owner 2026-11-12; shared ecosystem convention, IRIG 106 is silent and no standard source exists. |
| Q-EDITION | Accepted owner 2026-11-13: explicit declaration/override **wins**, basis **override**, contradictory opted-in G\106/header evidence retained side by side; without override, caller opt-in G\106 packet edition → **declared, assumed from TMATS** only when uncontradicted by packet evidence, else unresolved/raw bits with evidence shown (and a finding for a no-override conflict), no SRC/None invention. TMATS ADR-0028 vocabulary; no fallback selects EXT. [Sources and planned fixtures](Q-EDITION-2026-11-13.md). |

The handbook, Ch10/Ch4/Ch9 and all seven Ch11 edition pages were rendered
with pdftoppm and inspected visually. Existing T-10 documentation supplies
the earlier Chapter4 characters/bits page provenance; that wording is not
silently resolved by accepting the caller-context boundary.

## Why L1 changed, and why NTP L1 did not

Six L1 statements needed correction: L1-COR-002's blanket nearest
default is explicitly offline; L1-COR-003's blanket offline
count ranking conflicts with a committed stream channel; L1-STRM-001's
blanket same-answer promise conflicts with the distinct stream modes;
L1-WORD-001's unconditional complete calendar result conflicts with required
partial output without reconstruction context; L1-IPT-007 now retains RTC stamps
with reserved secondary-format bits under the owner Q-FLAGS refinement;
L1-CLI-007 adds output-write failure as the fifth shared ecosystem exit class.
No L1 IDs were added/retired.

L1-NTP-001..004 do not require a caller-declared era or cumulative offset;
the quoted “includes leap seconds” statement is refined by the RFC reading
at L2/L3. They remain unchanged. L1-TAI-004's generic table-currency rule is
also retained: PTP outside coverage uses the last known offset, with UTC
labelled possibly out of date and TF-021 warning, per contract 6.8/6.9 and
owner confirmation 2026-11-13. The contrary no-UTC instruction came from the
coordinator brief and is withdrawn; it was not an owner decision. NTP direct
UTC mapping remains independent of cumulative offsets, with uncertainty evidence.

## Review corrections, owner 2026-11-13

Changed requirement IDs in this review round (relative to 091ae3e):

`L1-IPT-007`, `L1-WORD-001`, `L1-COR-002`, `L1-COR-003`, `L2-ABS-102`, `L2-IPT-107`, `L2-NTP-103`, `L2-PTP-104`, `L2-TAI-104`, `L2-WORD-101`, `L2-WORD-102`, `L2-EDN-201`, `L2-COR-103`, `L2-COR-107`, `L2-COR-108`, `L2-ANS-101`, `L2-RDG-212`, `L3-ABS-102`, `L3-IPT-107`, `L3-NTP-103`, `L3-PTP-104`, `L3-TAI-104`, `L3-WORD-101`, `L3-WORD-102`, `L3-ENC-103`, `L3-EDN-201`, `L3-COR-103`, `L3-COR-107`, `L3-COR-108`, `L3-ANS-101`, `L3-RDG-212`.

- PTP last-known-offset/TF-021 fallback restored; coordinator-brief refusal withdrawn, never attributed to the owner. Contract 6.8/6.9 and L1-TAI-004 remain authoritative.
- Every Source line edited by this PR was compared with origin/main for lost quotations. Restored original quotations, alongside the new sources, in: `L1-WORD-001`, `L1-COR-002`, `L1-COR-003`, `L2-ABS-102`, `L2-TAI-104`, `L2-WORD-101`, `L2-WORD-102`, `L2-COR-103`, `L2-COR-107`, `L2-COR-108`, `L2-ANS-101`, `L3-ABS-102`, `L3-TAI-104`, `L3-WORD-101`, `L3-WORD-102`, `L3-COR-103`, `L3-COR-107`, `L3-COR-108`, `L3-ANS-101`. No original quotation remains dropped; confirmed layout prose replaces obsolete page-check TODO text.
- **Architecture revert:** `architecture.md` TF-005 and TF-006 rows restored byte-for-byte to origin/main, including warning defaults and the flags reserved-value listing. This reverts PR edits to both finding IDs; no new finding ID or level is introduced. RTC with format 11 retains its value with TF-006, not TF-005. Undefined secondary-derived time has no fitting existing numeric error finding; Q-FLAGS-ERROR is explicitly open.
- Q-EDITION accepted with TMATS ADR-0028 basis vocabulary and conflict/raw-value fixtures; removed from the nine-question open table. At that PR8 review point Q-TFMT-DEFAULT was open; it is now accepted (owner 2026-11-13), opt-in labelled default.
- NTP fraction floor/round toward zero accepted; removed from open proposals. Interpolation rounding and RFC archiving remain proposals.

## Final Q-EDITION review clarification

Relative to 4625cf5, changed IDs are **L2-EDN-201** and **L3-EDN-201**.
Explicit override wins (basis override), retaining contradictory opted-in
TMATS/header evidence side by side. The earlier brief's unresolved-conflict
rule applies only without an override; opted-in G\106 then requires no
contradictory packet evidence. This restores the owner-accepted precedence
and pinned TMATS ADR-0028 basis convention. Figure 10-15 is on printed
10-30 / full PDF 288; EXT definitions are on 10-31 / 289.

## Changed requirement IDs, parents and exact new Source lines

Each pair below lists both IDs and their immediate parents. L1 is the
repository's top requirement level (there is no invented L0 parent); its
owner/product basis and verbatim primary source are retained in Source.
Line references are to the accompanying requirements files in this PR.

<!-- REQUIREMENT-SOURCE-AUDIT -->

| Changed ID(s) and parent(s) | Source location(s) | Exact new **Source** line |
|---|---|---|
| L1-IPT-007 ← top-level owner/product requirement<br>L2-IPT-107 ← L1-IPT-007<br>L3-IPT-107 ← L2-IPT-107 | `docs/L1_Requirements.md:191`<br>`docs/L2_Requirements.md:241`<br>`docs/L3_Requirements.md:189` | "0 = Packet header 48-bit RTC."; "1 = Packet secondary header time (bit 7 must be 1)."; "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved" (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17); "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b, printed 11-11 / PDF 19). Value retention is owner 2026-11-12, Q-FLAGS; TF-006 reserved-value warning and unchanged TF-005/TF-006 levels are confirmed owner 2026-11-13; page images and older equivalent wording: `docs/analysis/Q-FLAGS-2026-11-12.md`. |
| L1-WORD-001 ← top-level owner/product requirement<br>L2-WORD-101 ← L1-WORD-001<br>L3-WORD-101 ← L2-WORD-101 | `docs/L1_Requirements.md:541`<br>`docs/L2_Requirements.md:727`<br>`docs/L3_Requirements.md:597` | "resolution of 655.36 seconds", "10 milliseconds", "three least significant bits of the BCD Julian date", "at a rate necessary to resolve time ambiguities" (106-24R1 Chapter 4 §4.7, printed 4-11 / PDF 15); RCC 123-20 `enI106_Ch4Binary2IrigTime`, printed B-43 / PDF 149, applies word weights without an origin. Explicit caller context and partial results are owner policy (2026-11-12); historical characters/bits wording remains unresolved (T-10). Retained earlier source: "High and low order time words shall be binary or binary coded decimal (BCD) weighted, and microsecond words shall be binary weighted"; resolutions of 1 µs (to 9999), 10 ms, and 655.36 s binary or one minute BCD; "For BCD, the days field shall contain the three least significant bits of the BCD Julian date" (Chapter 4 §4.7); `C-d\PTM`, `C-d\BTM` (Chapter 9); contract 3.8. Layout confirmed 2026-11-12, T-10. |
| L1-COR-002 ← top-level owner/product requirement | `docs/L1_Requirements.md:645` | "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78, supports the preceding-reference stream example); ADR-0010 (cited selectable defaults). The offline nearest-reference default remains contract 5.2 rule 5 / L1-POL-001 product policy; owner 2026-11-12 Q-DEFAULT selects standard for streaming, not a new offline algorithm. Retained earlier source: "It is better to use the clock and relative time values from a time packet that occurs near the current data packet as the data file is decoded since there is some drift in the RTC during a recording session" (RCC 123-09 §6.6; T-13); contract 5.2 rule 5; a policy setting (L1-POL-001). Prototype identifier kept. |
| L1-COR-003 ← top-level owner/product requirement<br>L2-COR-103 ← L1-COR-003<br>L3-COR-103 ← L2-COR-103 | `docs/L1_Requirements.md:653`<br>`docs/L2_Requirements.md:891`<br>`docs/L3_Requirements.md:745` | "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. Retained earlier source: "When multiple time channels are available, it is incumbent on the programmer or data analyst to determine and select the best source of time for a particular data set" (RCC 123-20 §5.6); "It is usually most correct to select one time channel only and to use this channel exclusively" (RCC 123-09 §6.6); contract 5.2 rule 4; a policy setting. Prototype identifier kept. |
| L1-STRM-001 ← top-level owner/product requirement<br>L2-POL-101 ← L1-POL-001<br>L3-POL-101 ← L2-POL-101 | `docs/L1_Requirements.md:885`<br>`docs/L2_Requirements.md:1087`<br>`docs/L3_Requirements.md:909` | "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. |
| L1-CLI-007 ← top-level owner/product requirement<br>L2-CLI-107 ← L1-CLI-007<br>L3-CLI-107 ← L2-CLI-107 | `docs/L1_Requirements.md:1116`<br>`docs/L2_Requirements.md:1739`<br>`docs/L3_Requirements.md:1455` | As `irig106-tmats` L1-CLI-006 (the existing L1 source); TelemetryWorks/irig106-tmats, `docs/CLI.md` §4, lines 88–91, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`: "`0` success; `1` the check failed (`verify` mismatch, `validate` findings at or above the failure threshold); `2` usage error; `3` input could not be read or is not TMATS/Chapter 10; `4` output could not be written. To be fixed before 0.1 and never renumbered." Owner 2026-11-12, Q-CLI-EXIT = align with irig106-tmats; findings/severities ADR-0013, hand-rolled parser ADR-0008. IRIG 106 is silent on tool exit codes: no standard source exists. This is a shared ecosystem convention so `irig106 tmats` and `irig106 time` agree; code 3 specializes the domain to Chapter 10/time input. See `docs/analysis/Q-CLI-EXIT-2026-11-12.md`. |
| L2-ABS-102 ← L1-ABS-002<br>L3-ABS-102 ← L2-ABS-102 | `docs/L2_Requirements.md:179`<br>`docs/L3_Requirements.md:137` | "resolution of 655.36 seconds", "10 milliseconds", "three least significant bits of the BCD Julian date", "at a rate necessary to resolve time ambiguities" (106-24R1 Chapter 4 §4.7, printed 4-11 / PDF 15); RCC 123-20 `enI106_Ch4Binary2IrigTime`, printed B-43 / PDF 149, applies word weights without an origin. Explicit caller context and partial results are owner policy (2026-11-12); historical characters/bits wording remains unresolved (T-10). Figure 11-4, printed 11-10 / PDF 18; raw layout already confirmed in T-10 and adopted from types 48f1834. Retained earlier source: "00 = Chapter 4 binary weighted 48-bit time format. The two lsbs of the 64-bit packet secondary header time and IPTS shall be zero-filled" (§11.2.1.1 g); Figure 11-4; Chapter 4 §4.7. Layout confirmed 2026-11-12, T-10. Prototype identifier kept. |
| L2-NTP-102 ← L1-NTP-002<br>L3-NTP-102 ← L2-NTP-102 | `docs/L2_Requirements.md:527`<br>`docs/L3_Requirements.md:429` | "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. "Eras cannot be produced by NTP directly" and "they can be derived from external means" (RFC 5905 §6, p.12); "if the client is set within 68 years of the server" (§6, p.15); "0 h 1 January 1900 UTC" (§6, p.13). Evidence precedence and labelled era-0 fallback are owner policy (2026-11-12), specified in RDG-012. |
| L2-NTP-103 ← L1-NTP-003<br>L3-NTP-103 ← L2-NTP-103 | `docs/L2_Requirements.md:539`<br>`docs/L3_Requirements.md:439` | "32-bit fraction field" (RFC 5905 §6, p.13); "Time Unsigned Seconds Fractions" (106-24R1 Figure 11-16, printed 11-27 / PDF 35). Floor (round toward zero for the unsigned fraction) is accepted owner policy, 2026-11-13, not standard wording (RDG-012). |
| L2-NTP-104 ← L1-NTP-004<br>L3-NTP-104 ← L2-NTP-104 | `docs/L2_Requirements.md:551`<br>`docs/L3_Requirements.md:449` | "2,208,988,800" (RFC 5905 §6 Figure 4, p.14, 1 Jan 1970); "a value of zero is a special case representing unknown or unsynchronized time" (§6, p.14); "knowledge of all previous leap seconds is lost" (RFC 1305 Appendix E.8, p.78); Figure 8 (p.79) repeats the same NTP number for UTC second 60 and midnight. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. |
| L2-PTP-104 ← L1-PTP-004<br>L3-PTP-104 ← L2-PTP-104 | `docs/L2_Requirements.md:589`<br>`docs/L3_Requirements.md:481` | "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012. |
| L2-TAI-104 ← L1-TAI-004<br>L3-TAI-104 ← L2-TAI-104 | `docs/L2_Requirements.md:677`<br>`docs/L3_Requirements.md:555` | "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Retained earlier source: "The leap-second table does not reach the date … the conversion is given with the last known offset and labelled as possibly out of date" (contract 6.8). |
| L2-TAI-107 ← L1-TAI-007<br>L3-TAI-107 ← L2-TAI-107 | `docs/L2_Requirements.md:713`<br>`docs/L3_Requirements.md:585` | "time changes are made on December 31 and on June 30 at 2400 hours" (RCC 200-16 Appendix A.2, printed A-1 / PDF 56); "Time Unsigned Seconds", "Time Unsigned Seconds Fractions" (106-24R1 Figure 11-16, printed 11-27 / PDF 35); RFC 1305 Appendix E.8 Figure 8, p.79. RDG-010 remains accepted; RDG-012 refines the NTP case. |
| L2-WORD-102 ← L1-WORD-002<br>L3-WORD-102 ← L2-WORD-102 | `docs/L2_Requirements.md:739`<br>`docs/L3_Requirements.md:617` | "PTP Time (64-bit)", "NTP Time (64-bit)" (106-24R1 Chapter 9 Table 9-11, `C-d\NTM`); "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. Retained earlier source: `C-d\NTM`: "PTP Time (64-bit)", "PTP Seconds (32-bit)", "PTP Nanoseconds (32-bit)", "NTP Time (64-bit)", "NTP Seconds (32-bit)", "NTP Fractions (32-bit)" (Chapter 9, Table 9-11); contract 3.8. |
| L2-EDN-201 ← L1-EDN-002<br>L3-EDN-201 ← L2-EDN-201 | `docs/L2_Requirements.md:827`<br>`docs/L3_Requirements.md:691` | "EXT" (106-04 §10.6.3.3 Figure 10-15, printed 10-30 / full PDF 288; EXT definitions printed 10-31 / full PDF 289); "SRC" (106-05 Figure 10-16a, printed 10-37 / PDF 41). "a value at or below the release version of the standard applied to the data types" (106-24R1 §11.2.1.1 e, printed 11-6 / PDF 14). No setup version: 106-04 §10.6.7.2.2/Figure 10-34, printed 10-48 / full PDF 306; "Reserved. (Bits 31-0) are reserved." (106-05 Figure 10-34b, printed 10-55 / PDF 59); "CH10VER" (106-07 Figure 10-50, printed 10-72 / PDF 76, only 0x07 assigned). G\106 is "VERSION OF IRIG 106 STANDARD USED TO GENERATE THIS TMATS FILE" (106-04 Table 9-1, printed 9-8 / full PDF 186). Basis vocabulary: "override", "declared", "fallback" (TelemetryWorks/irig106-tmats `docs/adr/0028-edition-declarations-and-validation-basis.md`, commit `41030d54dfbd8a539c2810d023dd0670bfd3dc1f`). Owner 2026-11-13, Q-EDITION: explicit caller override wins with contradictory evidence retained side by side; without override use opt-in TMATS only when uncontradicted, else unresolved. The earlier conflicting-evidence brief applied to the no-override case, as clarified in review. ADR-0028: "No automatic conflict finding"; "differing declarations are shown side by side" (same pinned commit, lines 59–63; caller-override precedence context line 11). STANDARD-REVIEW T-6; 106-24R1 Table 9-2 printed 9-10 / PDF 16 (index 9-9 / PDF 15); analysis: `docs/analysis/Q-EDITION-2026-11-13.md`. |
| L2-COR-107 ← L1-COR-007<br>L3-COR-107 ← L2-COR-107 | `docs/L2_Requirements.md:939`<br>`docs/L3_Requirements.md:785` | "the time source shall indicate Internal (0x0)" (106-24R1 §11.2.3.2 note, printed 11-26 / PDF 34). "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. Retained earlier source: "If the time source is external (0x1) and lock on the external source is lost then the time source shall indicate Internal (0x0). Once lock on the external time source is regained, time source shall once again indicate external (0x1)" (§11.2.3.2 a); contract 5.2 rule 6. |
| L2-COR-108 ← L1-COR-008<br>L3-COR-108 ← L2-COR-108 | `docs/L2_Requirements.md:951`<br>`docs/L3_Requirements.md:795` | "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "unless it is recorded at the raw network rate of the NTP or PTP frames" (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Retained earlier source: "the time packet will be generated at a minimum frequency of 1 hertz" (§11.2.3.2; Chapter 10 §10.6.2); the Format 2 exception "unless it is recorded at the raw network rate" (§11.2.3.3); contract 5.4. |
| L2-ANS-101 ← L1-ANS-001<br>L3-ANS-101 ← L2-ANS-101 | `docs/L2_Requirements.md:1025`<br>`docs/L3_Requirements.md:857` | "Eras cannot be produced by NTP directly" and "they can be derived from external means" (RFC 5905 §6, p.12); "if the client is set within 68 years of the server" (§6, p.15); "0 h 1 January 1900 UTC" (§6, p.13). Evidence precedence and labelled era-0 fallback are owner policy (2026-11-12), specified in RDG-012. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. Retained earlier source: Contract 3.1 (with the precision of a reference: "consistent with the resolution with the time packet body format (10 milliseconds [ms] as measured by the 10-MHz RTC)", §11.2.3.2), 3.2, 3.5, 7.3; ADR-0011. |
| L2-POL-201 ← L1-POL-001<br>L2-STRM-201 ← L1-STRM-001<br>L3-POL-201 ← L2-POL-201<br>L3-STRM-201 ← L2-STRM-201 | `docs/L2_Requirements.md:1123`<br>`docs/L2_Requirements.md:1383`<br>`docs/L3_Requirements.md:939`<br>`docs/L3_Requirements.md:1157` | "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. |
| L2-RDG-212 ← L1-RDG-002<br>L3-RDG-212 ← L2-RDG-212 | `docs/L2_Requirements.md:1357`<br>`docs/L3_Requirements.md:1135` | "2,208,988,800" (RFC 5905 §6 Figure 4, p.14, 1 Jan 1970); "a value of zero is a special case representing unknown or unsynchronized time" (§6, p.14); "knowledge of all previous leap seconds is lost" (RFC 1305 Appendix E.8, p.78); Figure 8 (p.79) repeats the same NTP number for UTC second 60 and midnight. "The NTP is referenced in UTC time with an epoch of January 1, 1900. The NTP time includes leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34); "the NTP timescale is in effect reset to UTC as each timecode is received" (RFC 1305 Appendix E.8, p.78). RDG-012, owner 2026-11-12; analysis: `docs/analysis/RDG-012-ntp-utc-era.md`. "The PTP is referenced in International Atomic Time with an epoch of January 1, 1970. The PTP time does not include leap seconds." (106-24R1 §11.2.3.3, printed 11-26 / PDF 34). Owner 2026-11-13 confirms contract 6.8/6.9 and L1-TAI-004: outside leap-table coverage use the last known offset, label the UTC conversion possibly out of date, and raise TF-021 (warning); keep TAI and table/offset provenance. The contrary no-UTC rule came from the coordinator brief and was withdrawn, not an owner decision; RDG-012. |
| L2-STRM-101 ← L1-STRM-001<br>L3-STRM-101 ← L2-STRM-101 | `docs/L2_Requirements.md:1371`<br>`docs/L3_Requirements.md:1147` | "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. |
| L2-STRM-202 ← L1-STRM-001<br>L2-STRM-205 ← L1-STRM-001<br>L3-STRM-202 ← L2-STRM-202<br>L3-STRM-205 ← L2-STRM-205 | `docs/L2_Requirements.md:1395`<br>`docs/L2_Requirements.md:1431`<br>`docs/L3_Requirements.md:1167`<br>`docs/L3_Requirements.md:1197` | "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. |
| L2-STRM-203 ← L1-STRM-001<br>L3-STRM-203 ← L2-STRM-203 | `docs/L2_Requirements.md:1407`<br>`docs/L3_Requirements.md:1177` | "minimum frequency of 1 hertz" (106-24R1 Chapter 10 §10.6.2); "packet generation times shall be equal to or less than 100 milliseconds" (§10.6.1 b); "stream commit time equal to or less than 1000 ms" (§10.6.1 c), all printed 10-26 / PDF 32. The 100 ms hold tolerance, zero gap tolerance, explicit progress, caps and release rules are owner-selected policy (2026-11-12), not timing guarantees imposed on a reader. |
| L2-STRM-206 ← L1-STRM-001<br>L3-STRM-206 ← L2-STRM-206 | `docs/L2_Requirements.md:1443`<br>`docs/L3_Requirements.md:1207` | "it is incumbent on the programmer or data analyst to determine and select the best source of time" (RCC 123-20 §5.6, printed 5-44 / PDF 78); "TIMEIN" / "IRIG Time Input" (`R-x\CDT-n`, 106-24R1 Chapter 9 Table 9-4, printed 9-50 / PDF 56), "E" / "External" (`R-x\TSRC-n`, printed 9-72 / PDF 78). Option A ranking/commitment is owner policy (2026-11-12); the standard names attributes, not that ranking. "clock time of a subsequent data packet" (RCC 123-20 §5.6, printed 5-44 / PDF 78); ADR-0010 (cited, selectable defaults). Owner decision 2026-11-12; policy/interpretation, not an added standard mandate. |
| L3-ENC-103 ← L2-ENC-103 | `docs/L3_Requirements.md:649` | "0 = Packet header 48-bit RTC."; "1 = Packet secondary header time (bit 7 must be 1)."; "Bits 3-2: Indicate the packet secondary header time format."; "11 = Reserved" (106-24R1 §11.2.1.1 g, printed 11-8/11-9, PDF 16/17); "48-bit RTC format (plus 16 high-order zero bits)" (§11.2.1.3 b, printed 11-11 / PDF 19). Value retention is owner 2026-11-12, Q-FLAGS; TF-006 reserved-value warning and unchanged TF-005/TF-006 levels are confirmed owner 2026-11-13; page images and older equivalent wording: `docs/analysis/Q-FLAGS-2026-11-12.md`. Secondary-header output layout: §11.2.1.2; stamp layout: §11.2.1.3 b. |

## Open proposals and checks

- **NTP floor rounding is accepted**, owner 2026-11-13; raw fraction retained,
  boundaries 0/500000000/999999999 ns. Interpolation rounding/ties and
  mode-change flushing remain proposed.
- **RFC archive:** propose publisher RFC1305/5905 artifacts with provenance,
  hashes and pagination notes in rcc-106-standards; not performed. The RFCs
  remain externally cited primary sources.
- Q-TFMT-DEFAULT is accepted (owner 2026-11-13): opt-in labelled default,
  always retain the missing-required finding. Chapter4 characters/bits wording remains unresolved and
  reported; accepting explicit context did not settle that textual ambiguity.

Executed local checks: trace matrix regenerated and `--check` passed;
all changed Source quotations matched primary text after whitespace-only
line-wrap normalization (retained contract ellipsis checked as its two quoted
segments, without inventing omitted wording); counts/parent chains/L1 ancestors checked; prior
register Review and Design fields compared unchanged, with only new
RDG-012 accepted/pending added. The changes are confined to docs/*.md and
docs/analysis/*.md. Full Rust tests/builds were not rerun locally for this
documentation-only change; GitHub CI status is reported in the PR.

TRACE-MATRIX now has124 L1 /144 L2 /144 L3; the extra pair is
L2/L3-RDG-212. Existing58 marker links over288 countable requirements (20.1%)
are not evidence that the newly revised conversion semantics are implemented.
RDG-012 has written analysis but no tests yet; its test cases are planned.
The existing slice-2 NeedsContext assertions stay unchanged until a future
separately authorized implementation slice replaces them with failing tests
for these requirements. No Design/Review promotion or semantic-completeness
claim is inferred from the generated matrix.

Q-FLAGS code work is conditional on this docs PR merging, in a separate
tests-first PR; all 256 flags and five literal cases are planned, not executed.
The current Rust implementation still rejects the newly accepted RTC cases.

Q-CLI-EXIT is accepted and removed from the open table. Its implementation
and per-code tests (including output failure and ADR-0008 parser usage)
belong to the CLI restructure step, with Requirements/Traces markers.

**Accepted Q-FLAGS-ERROR, owner 2026-11-13:** TF-028 "Secondary-derived time has an undefined format" is ERROR and maps from UndefinedSecondaryTimeFormat. It reports selected or present secondary time with format 11; RTC stamps remain usable. TF-006 remains the separate reserved-VALUE warning, TF-005 reserved-BITS warning, and TF-015 missing-required-header error. With 0x4C, TF-015/TF-028/TF-006 evidence can coexist. TF-001..TF-027 are unchanged; the former numeric finding-assignment open item is resolved.

## Final owner decisions: PR D

Owner 2026-11-13 resolves Q-TFMT-DEFAULT as opt-in labelled default and
Q-FLAGS-ERROR as TF-028 ERROR. Changed requirement IDs are
L2/L3-RDG-202, L2/L3-FND-104 and L2/L3-IPT-107; every parent remains unchanged.
Their updated Source/supplement quotes the primary definitions and pins
TMATS ADR-0023. The RDG-002 register change is confined to Behaviour;
its earlier question records the prior discussion, now explicitly resolved
by that Behaviour decision. Design/Review fields are unchanged.

No TFMT comparison implementation or findings-report emitter is introduced.
PR9 already decodes RTC with independent warnings/errors; the error variant
now documents its accepted TF-028 mapping. Independent truth-table and exact
error-variant tests strengthen that existing behaviour.

### PR D review: preserve the existing requirement limits

L2/L3-FND-104 retain **only RDG-001..006** comparisons, the I inference
warning (I ↔ 0x3 labelled inferred, owner Q1), and external->internal
loss-of-lock information (TF-011), alongside the new TFMT default cases.
All six requirements edited by PR D were compared with main. The audit also
restored RDG-202's explicit Format 1 scope, separately recorded published
default (including with comparison disabled), and visual-check citation.
IPT-107's remaining changes are the accepted TF-028 mapping and strengthened
verification; none of its existing flag/padding/length cases was removed.
TF-028's Contract cell follows the existing style; its owner date remains
in this decision record. TF-001..TF-027 and human Design/Review fields
are unchanged.
