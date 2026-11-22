# S17 item6b1 — summary native counter spans, not reference quality

Base mainedd6c3b (#98). Partial L1-CLI-004/006 → L2/L3-CLI-104/106. Use native
SessionClock for every summary header. Recording-wide sessions are separate from
per-channel CLI reset segments; legacy query/correlation is unchanged. Native
quality, calendar spans and TMATS admission/association remain OPEN. No whole
Requirements markers, Review/Design changes or coverage increase (198/206 of288).

Source: contract5.4/ADR-0010; §11.2.1.1 i free-running recorder RTC. Native1100ms
is reader policy, separate from CLI30s/run resets; settings come from getters.
First/last follow observation order; low/high are extrema; checked(high-low)*100
is counter ns. Text/JSON stream one private spool row plus the active cursor.
Temp/writer failures give4. Other commands do not construct native state (null).
Exact half-wrap ambiguity stops only native analysis at a labelled prefix.
Proposed WARNING/code null matches correlation needs-context; no TF-018 fact is
invented. One finding retains channel/packet; status/reason disclose the limit.

Tests-first c33d344:3 runtime failures for absent sections/fields and writer4.
Now3 tests cover2 native sessions vs1 channel segment, data-header inclusion,
3000ns forward wrap, ambiguous-prefix/default-warning-never classes and writer4.
Literal Fig11-12/13/§11.2.1.1 wire/checksum fixtures and inventory golden header
coordinates are independent of production encoders/captured output. A private
test pins lazy closed+active emission. Seven compiling mutants fail: units,
last coordinate, closed row, stop guard, finding, writer propagation and eager
collection; sources restored. Conditional schema fragments are distinguished
from full producer objects by the shape test; full-schema CI also validates.
Revision9 is additive. Local focused tests/Clippy/fmt/trace/schema-sync/diff and
30-view validation pass; full platform/package/deny/fuzz/RSS use PR CI. Item6
quality/absolute spans, item7 and full published TMATS0.1/bounded-view approval
remain release work; native counter spans alone do not close CLI-104 or item6.

CI45008572011 passed platform, package, schema and CLI tests, then exposed the
RSS harness's8KiB prefix assumption: native span rows precede summary findings/
jump totals. Scan the log line by line for these assertions; keep32MiB and exact
expected counts unchanged. A small footer-after8KiB/no-slurp regression fails
before the helper exists and passes after; no local large workload is rerun.
