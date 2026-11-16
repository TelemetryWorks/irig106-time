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

## Executed evidence and interface boundary

Tests-only bf878d7 preceded production and fails E0432: extracted API absent.
Its three tests are compile-time API evidence, not runtime parser regressions.
Four further tests and expanded assertions are explicitly post-prototype.
The resolver constructor later boxed owned snapshots and gained an explicit
query_id: multiple messages may share one physical packet origin while the
stream core requires distinct observation IDs. No original packet ID is
silently rewritten. Query preparation/scheduling is caller-controlled; a
released counter at another key/session is rejected, never shifted as a proxy.
Offline callbacks use lookup at the derived key; stream fixtures feed the
extracted queries through Standard/Interpolate with existing caps/progress.
Resolver operational errors propagate; counter-only replies remain partial
with raw values, nominal counter duration and the original unresolved basis.

Seven focused tests pass, including the literal all256 truth table, defaults
and severity overrides, all four stamp forms, absent context/reference, short
buffers, RTC wrap, separate reserved fields, actual per-packet dispatch and
all six network-word kinds. Component binding needs matching target and same
session; both origins remain in the snapshot. No missing component becomes0.
Native PTP stays TAI; UTC is a separate S04 conversion with table/currency.
NTP uses S05 declared/pivot/assumed eras, floor fractions, zero and ambiguity.
BCD keeps S07's characters/bits disagreement even with full caller context.
No unrelated header counter supplies a word's day/year/era/epoch.

The stream midpoint fixture is1s RTC/1.000010s absolute ->0.500005s.
A second hand-derived case uses1s/1.000000011s: RTC9090909 ticks maps
floor(909090900*1.000000011)=909090909ns; ERTC909090999ns maps909091009ns.
Its99ns remainder therefore moves the released physical coordinate by100ns;
nominal counter duration still ends in99ns. These are caller-supplied exact
reference-value fixtures, not claims that a10ms BCD wire body encodes11ns.
The two messages use distinct core query IDs2/3 but physical packet origin2.

32 distinct isolated mutants compiled and failed runtime assertions, restored
before normal verification: suppress4/5/6/15/21/22/24; omit simultaneous028;
discard warnings; unbounded stamp prefix; ignore RTC padding; ignore modular
displacement; wrong ERTC units; skip sub-tick drift; ignore resolver identity;
IEEE epoch fallback for stamp/word binary; ignore binding target/session;
mislabel native PTP UTC; omit NTP/PTP converted findings; drop both override
paths; ignore absent PTP offset; accept ns1e9; zero-fill components; mark15
Kept; conflate query/physical IDs; lose nominal remainder; wrong channel; discard
a usable RTC stamp solely because secondary-only028 has ERROR severity.
The first pass left prefix and sub-tick drift mutants alive; the tail fixture
and11ns-per-second example above killed them. Further offset/channel assertions
killed two initial survivors in the final query-evidence pass. No compile-error
or equivalent mutant is counted as killed.

## Clause-level verification

| L2/L3 suffix | Evidence and remaining boundaries |
|---|---|
| IPT107 | All256 flags, literal0c/8c/4c/cc/40, short input, low48/padding, independent15/6/28 and24, original flags/locations, error levels cannot synthesize time; existing whole marker gains integrated findings tests |
| ANS103 | Whole marker: all stamp forms, per-query session/time-channel basis, no reference/context, invalid flags, wrap, per-packet format. Separate stream tests cover both modes, actual stamp rather than header query, two messages, interpolation and retained release basis |
| ANS104 | Whole marker: binary/BCD/network values, full/component cases, missing and explicit context, origin and selected interpretation, no implicit counter-derived date |
| WORD102 | Whole marker: all six kinds, explicit binding/missing components, declared/pivot/assumed NTP, zero and leap ambiguity, native TAI, stale-offset UTC/21 and empty table |

Matrix186 ->192/288 (+6, ANS103/104 and WORD102 pairs). IPT107 already had
whole coverage and gains integrated diagnostics; no new status for partial
S12 common-schema, serde, CLI, body extraction or complete encoder work.
No Review/Design fields or finding levels change. No new owner question.
Source/test code is approximately1500 lines; the additional record and generated
matrix describe scope and prevent whole-requirement claims for later adapters.

Final local verification: all 18 locked CI-equivalent checks pass with jobs/
threads 2 and incremental off: feature suites, MSRV 1.85, no_std/WASM, docs,
CLI, benches, all 10 fuzz targets, clippy, format, trace and cargo-deny.
No fuzz campaign or local recording check is claimed.
