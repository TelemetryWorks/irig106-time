# S17 follow-up1 — packet/query protocol and status

Base main2610636(#91 merged). Partial L1-CLI-005 -> L2/L3-CLI-105;
no whole Requirements marker, Review/Design or requirement-statement change.
CSV is unchanged; the coordinator-approved19-column append-only contract
follows separately after #92 merges.
Later assigned F1 basis, policy identity, NTP era, CI schema validation, native
session/quality and documentation slices are not claimed here.

Tests-first03c1519 has three runtime failures on main: missing packet facts,
missing selected-reference query facts, and missing PTP packet observation when
its numeric clock came from IRIG. Implementation-side expansion checks all256
NTF/TS combinations and CSDW/body truncation without filler reads; it is not a separate failed-first claim. Fixtures derive
from106-24R1 Fig11-15(printed11-27/PDF35): bits7..4 NTF0NTP/1PTP2002/2PTP2008,
bits3..0 TS0not-valid/1valid/othersreserved. Fig11-17 PTP2017-01-02:
Unix1483315200 +37 =1483315237 TAI. Fig11-12/13 supplies DOY10012:00 IRIG-B.
Primary headers use independent11.2.1.1 word sums and100ns RTCs. No production
encoder or captured output supplies expected facts; existing numeric values stay.

Revision6 adds nullable `network_time` to packet rows, any-channel query values
and channel/segment answers. Its four keys are raw NTF, closed protocol, raw TS
and closed status. Every defined key stays present. In a packet row these are
**that packet's** CSDW, even when malformed/reserved/context-missing; null means
non-F2 or fewer than4 CSDW bytes. This is separate from its resolved clock.
In a query they describe the **reference actually selected**, not the channel's
latest observation. Null means no selected F2 reference/answer. Rejected TS or
unresolved NTP cannot invent an answer or inherit another channel's facts.
Text appends the same protocol and raw/decoded TS, including counter-only rows.

Numeric selection/basis is unchanged. Target brackets retain one byte of facts
in each existing bounded candidate; both basis and facts use the same selected
candidate. Reset records retain it in the existing bounded private spool. No
new history, per-file queue, library I/O, TMATS dependency or source association.
A40->1 reset fixture queriesRTC5s: segment1 selectsPTP2002, segment2PTP2008;
any-channel is withheld. Equal-RTC IRIG/PTP controls distinguish packet facts
from the clock source. F1/data null fields and previous goldens remain pinned.

Six compiling focused mutants are killed: NTP labelledGPS; raw TS forced0;
packet facts discarded; latest bracket chosen instead of nearest; private-spool
facts dropped; text TS forced0. Sources restored. Local fmt, focused tests/mutants,
schema-sync/trace/diff checks only; CI carries platform/package/security and RSS.
Matrix remains198 tested/206 verified of288, with no whole coverage gain.


The initial18-column CSV proposal is superseded by the coordinator-approved
19-column contract: append network_time_format,network_protocol,time_status,
time_status_name (Valid/NotValid/RSVD-n). Future columns append only, never
reorder/remove; empty for non-F2/missing CSDW; packet facts, not clock source.
Implementation remains a separate PR after #92 merges.

## #92 wording review

Tests-only18040ec: three runtime failures before the fix; existing JSON/source
selection control passes. The reviewer's wire fixture is ch9 PTP2008 TS1 atRTC0,
ch10 PTP2002 TS1 atRTC1, then ch9 PTP2008 TS0 atRTC2. Ch10's UTC00:01:40
plus1s supplies row3's00:01:41 clock; ch9 would give00:00:02 from its prior
reference, and its invalid body would give00:08:20. The row must end with
`[packet: IEEE1588-2008 TS=0 not-valid]`. The any-channel query atRTC5 instead
uses00:01:44 and `[reference: IEEE1588-2002 TS=1 valid]`.

The change only prefixes existing text annotations at their three render sites:
timeline packet rows, any-channel correlate and channel/segment answers. JSON,
numeric selection, schema revision6, CSV and findings are unchanged. The256-case
text expectations and reset-spool labels now include the appropriate prefix.
No whole requirement marker, Review/Design change or coverage increase.

Three additional compiling scope-swap mutants fail: timeline labelled reference,
any-channel correlate labelled packet, and channel/segment correlate labelled
packet. Source restored; the22 focused network/stale/query tests pass.
