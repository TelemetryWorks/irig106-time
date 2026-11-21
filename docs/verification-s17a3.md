# S17a3 — draft JSON report foundation

Base27befa7 (PR77 merged). CLI-only serde_json already approved/pinned. No new
dependency or library source changes. No Review/Design edits. Partial
L1-CLI-006 -> L2/L3-CLI-106 only; coverage stays198 tested/206 verified of288.

## Reviewable split / no overclaim

This slice: strict --format text/json, common scan/Report for three inventory
commands, versioned draft schema/units/provenance, missing-year golden, exact
wide values and output4. Next slices: stale leap table, native GPS, all network
protocols, reset and inferred/default TFMT full JSON goldens; qualified joining
may extend the draft schema. Those clauses remain OPEN. The dedicated pre-freeze
schema review must follow that completion, not be mistaken for approval here.

Tests-first46cab07 reuses preserved assertions: two runtime failures at missing
--format; bad-format control already passes. Added projection/golden assertions
arrive with implementation, not claimed failing first. Missing-year vector is
literal Fig11-12/13 (11-23/PDF31,11-25/PDF33), §11.2.1.1 header checksumFC60;
Data Length10 excludes DD filler. Independent golden values: day10012:00:00,
clock8596800000000000ns, null year, assumed UTC/RDG-009, one admitted reference.
Only already-asserted runtime path/hex/version are normalized in the golden.

Focused JSON tests and previous12 text-report tests pass. Seven runtime mutants
killed: version, numeric wide count/file size/signed residual, lost clock basis,
lost year phase, swallowed writer error. Source restored. Schema version/unit constants and golden checks are not a general JSON Schema validator. i128/u128 extrema test
the helper, not nonexistent report fields. JSON retains each clock's basis and
raw/unresolved evidence without inventing SRC labels or UTC for RTC.

Local fmt/CLI Clippy/focused tests/mutants and small RSS-generator checks only.
CI carries the full matrix and existing32MiB gate. Direct JSON-mode dense probes
are preserved for the next golden/qualification slice. No local large file. Publishing staysfalse untilS20.

Draft rules: wide quantities/usize counts are decimal strings; small wire/date
fields and version are JSON integers. Null is unknown. Objects/IDs sort, samples
stay in observation order. Native path bytes are platform-labelled hex; counts
are not epochs. No JSON load API. After the dedicated freeze review, incompatible
shape/type/unit changes require a new schema version.
