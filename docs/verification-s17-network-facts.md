# S17 follow-up1 — packet/query protocol and status

Base main2610636(#91 merged). Partial L1-CLI-005 -> L2/L3-CLI-105;
no whole Requirements marker, Review/Design or requirement-statement change.
CSV is unchanged pending coordinator approval of the proposed18-column contract.
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


CSV proposal (not implemented or approved here): append network_time_format,
network_protocol,time_status to the existing15 columns; raw NTF/TS integers,
closed protocol names as JSON, empty for non-F2/missing CSDW. Await coordinator
approval of the exact18-column list in the PR before changing the CSV contract.
