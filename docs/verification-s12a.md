# S12a — shared policy and complete basis

Preparation starts from main38681fe. Before implementation, split proposal:
S12a1 centralizes validated immutable shared qualification settings/identity,
retains reference declaration/qualification snapshots, and wires the existing
cores without changing legacy defaults. S12a2 supplies the common replayable
answer view, typed counter duration and own/stamp/word envelopes. These are
review parts, not release deferrals; S11 inverse queries stays deferred by Q3.

Sources reused from reviewed requirements: ADR-0010/0011; contract5.3/5.6;
RCC123-20 §5.6 (5-44/PDF78), "clock time of a subsequent data packet";
106-24R1 §10.6.2 (10-26/PDF32), "minimum frequency of 1 hertz";
§10.6.1b/c, "packet generation times shall be equal to or less than 100
milliseconds" and "stream commit time equal to or less than 1000 ms".
Reader defaults remain: late1100ms, gap1s+0 tolerance, stream Standard,
Interpolate hold1s+100ms, caller progress/caps, C1 floor, C2 preceding tie,
C3 coarser resolution and C4 old-policy drain. These are cited reader policy,
not stronger guarantees on a recording. D6 words still require caller context;
D7 NTP and stale PTP use their accepted rules; D8 never inserts defaults;
D9 year default is packet/caller/RI4, with conflicts/raw evidence retained.

Design: optional shared qualification snapshot enables the consolidated path;
legacy per-function prepared inputs remain supported and explicitly untagged.
A shared instance is frozen/validated by ID. Reference inputs retain effective
and provided qualification evidence; no policy-history registry or new queue.
Mode-only identity changes keep eligible references with their old snapshots;
semantic qualification changes clear references only after old-policy drain.
