# S10b — stamps and caller-extracted words

Starts from main d076f27. Scope: L2/L3-IPT-107, ANS-103/104 and WORD-102.
Source quotations reused from the governing requirements (no fresh page scan):
106-24R1 §11.2.1.3b (11-11/PDF19), "48-bit RTC format (plus 16 high-order
zero bits)"; §11.2.1.1g (11-9/PDF17), "bit 7 must be 1", "11 = Reserved".
The full wording and flag decisions are in analysis/Q-FLAGS-2026-11-12.md.
Ch4 §4.7 (4-11/PDF15) specifies word weights, not a calendar origin. Ch9
Table9-11 C-d\NTM names PTP/PTPS/PTPNS/NTP/NTPS/NTPF; WORD-102 and RDG-012
supply full-network conversion semantics, including D7 stale-offset fallback.

Design before code: inspection collects simultaneous flag findings, bounds
reads to eight bytes, and derives a named-session query for relative stamps
using explicit caller wrap ordering. Resolution consumes a counter answer AT
THAT QUERY (offline lookup or caller-released stream query), not an enclosing
packet answer shifted across unknown references/jumps. Retain the query answer
and original extraction origin separately. ERTC sub-tick remainder uses the
released bracket slope; absolute stamps use S10a2 caller epochs, no fake header
or fabricated checksum. Stream scheduling/caps/fallback stay with its core.
Words are already extracted by the caller; no PCM/bus/event walker is added.
Component binding is explicit, targeted to an extraction origin, same session,
and retains both origins; missing/mismatched components remain partial.

Hand-derived fixtures planned: all256 flags with literal16-cell truth table;
0x0c/8c/4c/cc/40; lengths0..7; padding and Ch4 reserved words; raw RTC wrap;
ERTC500000099ns -> query5000000ticks +99ns; bracket1s RTC vs1.000010s
absolute -> midpoint0.500005s. IEEE ns-first, Ch4 low100=1s, BCD from S07.
NTP2208988800=1970, fraction80000000=0.5s; zero sentinel; declared/pivot/
assumed eras and insertion ambiguity. PTP1483228837TAI=2017-01-01UTC;
2028 uses last offset37/TF021; empty table keeps TAI without UTC.
Tests-first evidence and clause/marker limits are recorded below after execution.
