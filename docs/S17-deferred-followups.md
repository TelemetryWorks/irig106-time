# S17 follow-ups retained for the coordinator

These are explicit remaining items, not newly approved post-0.8 release deferrals.

- **Decode once:** review measured 1M-reference time increasing from 7.8s to
  18.1s with two decodes per replay pass. Optimize separately, preserving raw
  provenance, admission, rejected-label rebuilds, equality with offline and the
  32MiB gate. These timings are reviewer evidence, not a new local benchmark.
- **Packet-local TF-007 omission:** main's CLI can omit LY/year or LY/day findings
  present in packet-local facts. #104 intentionally pins main-compatible counts
  and emits newly derived findings only; that is not a standards-completeness
  claim. Fix the baseline scanner/native finding ownership in a reviewed slice.
- **Single-pass survivor:** the reviewer reported a surviving mutation that
  forces a single replay pass. Add a discriminating fixture or document a proved
  equivalence before simplifying. It is not proof that rejected-label rebuilds
  are unnecessary; retain the current protocol. No fresh mutation run is claimed
  by this documentation-only note.
