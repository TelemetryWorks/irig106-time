# PR46 review at1639e9c — i64 era and emitted-field completeness

Tests/record only. No production change or requirement/status change; no new
whole Requirements markers. Matrix remains192/288. Main858492f does not need
to be merged into46 for this revision; the reviewer's CLEAN-base instruction
is retained. Changes are carried into47 and48 with ordinary merges.

Two additional tests pass on the unchanged implementation:

- Negative era: a Word envelope declares era−1 and retains a resolved NTP pivot
  from that era. RFC5905 §6/Figure4: era width2^32s; raw seconds2,208,988,800
  resolves to Unix−4,294,967,296 in era−1. Assert the exact STRING "-1" on the
  declaration in WordContext, the pivot's ResolvedNtp origin, and the actual
  network conversion's context. A numeric −1 is not an acceptable substitute.
- Independent literal key sets pin the offline and Word answer envelopes,
  common AnswerView/AnswerValue, CounterBasis/clock, OfflineBasis in both its
  locations, PacketKey, OfflineAnswer, WordEnvelope, ExtractionOrigin,
  WordContext and NTP declaration/context. Empty/null fields must still exist.
  Exhaustive public AnswerValue/OfflineBasis patterns additionally make a new
  native field a compile-time review point even if omitted from serialization.
  Private-envelope key sets protect their emitted shape; this is not a new
  claim that one fixture checks every one of the67 serialization lists.

Each temporary reviewer mutation fails at runtime (no compile-error kills):

| Mutation | Failure |
|---|---|
| W9: serialize_i64 emits a JSON number | Declared era is Number(-1), expected String("-1"). The separate prewrapped pivot-era assertion also pins its contract. |
| W10: remove findings from AnswerValue serialize_struct! | Exact common-value key set is missing findings, including the empty array case. |
| W14: remove session_origin from OfflineBasis serialize_struct! | Exact offline-basis key set is missing session_origin. |

All mutations are restored and production files match1639e9c byte-for-byte.
The two tests were added to the existing two output tests (four total). Existing
adapter/shape tests and schema review boundaries retain their earlier scope.

Restored46: all18 local CI-equivalent checks pass, jobs2/threads2, --locked,
including feature suites, MSRV1.85, no_std/WASM, CLI, benches, all10 fuzz builds,
clippy/fmt/rustdoc, cargo-deny and trace freshness. Actionlint/diff checks pass.
