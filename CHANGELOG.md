# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ⚠️ S04 validated leap tables and PTP→UTC

- BREAKING: LeapSecondEntry fields are UnixSeconds/TaiUtcOffset. Table add,
  offset_at_unix/offset_at_tai/offset_for_f1 return Result; propagate failures.
  Empty/pre-start no longer guess37/first offset. InvalidLeapTable reports
  rejected data; add is atomic and does not silently extend coverage.
- Table serialization gains metadata; deserialization validates through try_new.
  Use try_new/extended/replaced with explicit identity/source/start/currency.
  Conflicting history, duplicate/reordered entries and non-unit steps fail.
- convert_ptp preserves raw TAI, second60, UTC/basis and coverage labels.
  Stale uses actual last offset with TF-021; no offset means missing context.
  Lookup uses TAI intervals, not the modern37 approximation. Exact only1972+.
  Builtin28 transitions verified against IERS C72; expiry and2017 final change
  are distinct. Canonical findings retain operands and TF-022 leap information.
- Legacy correlators now propagate missing context/refuse leap-label projection;
  CLI indirectly inherits these errors. Their bare calendar still lacks currency
  basis; Format2 adapter/report adoption remains planned, not claimed complete.
  Explicit-offset scalar helpers do not infer pre1972 rate/offset history.
- Verification-s04 records12 focused tests, honest failing-first evidence and
  32 killed mutations; matrix links do not prove every clause/release check.

### ⚠️ S03 edition-qualified Format 1

- Add caller-first edition evidence resolution: explicit override wins; G\106
  selects packet layout only by opt-in and without contradictory known packet
  evidence. Header/setup version mappings alone never select legacy EXT.
  Original declarations and hints stay side by side in qualified answer bases.
- Add qualified SRC/EXT/FMT/ITS and all reserved-mask alternatives. Unknown
  nibble3 has no GPS/None label; legacy FMT15 stays reserved, modern15 None,
  unresolved15 needs context. Common readable body values remain usable.
- BREAKING: TimeF1Source gains Unresolved(raw); exhaustive matches must handle
  it. ReferenceInput literals gain qualified (use None at the existing explicit
  caller-layout boundary, or qualified_format1 for retained resolver evidence).
  Existing explicit-layout helpers/CLI output paths are not automatically changed.
- TF-031 now emits WARNING only for the accepted no-override contradiction
  case: **owner-accepted WARNING (2026-11-14)**, time body kept and EXT/SRC
  qualification withheld. Catalog/policy can configure this approved level; TF-032 still
  has no default. Reserved ITS yields TF-006; reserved bits yield TF-005.
  No TF-001..028 row or Design/Review status changes.
- Eight S03 tests, 20 assertion-killed mutations, 128→136/288 matrix links.
  verification-s03 distinguishes one main-first assertion failure, missing-API
  compile evidence, later assertions and partial historical scheduling evidence.

### ⚠️ S02 calendar and raw Format 1 closure

- BREAKING: `AbsoluteTime::add_nanos` / `sub_nanos` now return Result.
  Propagate errors: known years advance with Gregorian lengths; unknown-year
  boundaries require context instead of assuming 365/366 days. `with_year`
  rejects day366 in a common year. These are nominal unscaled calendar shifts,
  not leap-table elapsed UTC arithmetic. Compatibility correlators propagate
  the errors; benchmark/test callers explicitly handle the Result.
- BREAKING: `InvalidBcdDigit` gains word/digit/bit-offset/raw-word fields and
  more precise Display text. Match with `..` or migrate constructed errors.
  Valid decimal six in minute/second tens now produces a total-range error
  instead of a false invalid-digit error in legacy body parsers.
- Add checked labelled CalendarReading shifts: preserve scale/second60,
  refuse missing leap/scale context. Add bounded Format1Input and offline/stream
  ingestion retaining original typed TF-004 causes and TF-005 word/mask operands,
  including simultaneous malformed-decimal and zero-bit findings. DATE selection
  and filler exclusion are retained. Raw LY does not replace Gregorian year.
- Strict legacy body-only parsers remain unscaled and reject reserved bits and
  second60; canonical Format1 decoding preserves warnings and eligible leap
  labels. No CLI output path is switched in this slice. See verification-s02:
  15 focused tests, honest failing-first history, 17 initial plus 18 review
  assertion-killed mutations;
  matrix links 116→128/288 express links, not complete clause compliance.

### ⚠️ S01b shared-types adoption

- Fuzz follow-up: migrate the missed correlation duration constant and older
  BCD harness getter/checked-conversion calls, without changing library BCD
  behaviour or turning expected errors into fuzz crashes. Add a bounded,
  compile-only all-fuzz-target CI job and the matching local CI command;
  no fuzz campaign or instrumented runtime coverage is claimed.

- Advance irig106-types to `7bfe47fc903efea342f3efba1ad0f527dd059a0d` (types
  PR #8); root/CLI/fuzz lockfiles follow it. No other dependency version changes.
  Re-export shared SignedNanos/SignedRtcTicks/SignedUnixSeconds; local signed
  definitions are removed. Shared inherent duration conversions return Option;
  use `rtc::signed_ticks_to_nanos`, `signed_nanos_magnitude` and
  `signed_nanos_to_unsigned` when a Result/QuantityOverflow boundary is needed.
- PtpTime::to_utc_seconds now returns Result<u64>; propagate with `?`.
  `to_signed_utc_seconds` preserves negative Unix seconds. Calendar/offline/
  stream adapters reject an unrepresentable unsigned result before insertion
  instead of silently clamping to 1970. Both former to_unix callers use signed
  arithmetic; no new leap-table policy or pre-1970 calendar support is claimed.
- NanosDuration::from_secs/from_millis/from_micros now return Option, and
  TaiSeconds::to_unix is checked; use to_signed_unix for negative results.
  After the fuzz follow-up, callers/examples including fuzz harnesses are
  migrated; all ten fuzz targets were type-checked with the locked cargo check.
  The CLI fixed-threshold
  expression remains exactly 1000000000 ns. A pre-1970 PTP result now follows
  its existing correlation-error path instead of displaying fabricated 1970 time.
  Bench/test literal fixtures handle Option explicitly. deny.toml is unchanged.
- API-104's shared-type gaps are closed in `docs/verification-s01.md`, with
  adoption evidence and remaining global API/conversion limits retained.

### Added

- S01 checked `rtc` quantities: `checked_rtc`, named-session `SessionRtc`,
  explicit ordering context, signed ticks/nanoseconds and checked shared
  `NanosDuration` conversion. No absolute epoch or multi-wrap count is inferred.
- Stable `findings` catalog/evidence shared by the offline/stream engines:
  optional actual byte offset, related identities, exact raw fields/counter,
  factual value effect independent of severity. TF-029/030 evidence retains
  raw NTP words and sentinel-no-value / ambiguity-value-kept effects; conversion
  and leap-window detection remain S05. TF-031 is owner-accepted WARNING
  (2026-11-14), configurable/emittable in S03; only TF-032 remains pending
  and cannot be emitted by a default or explicit override.
- S01 verification: 15 focused tests, unit/epoch/privacy compile-fail controls and
  12 initial + 10 reviewer assertion-killed mutations; see `docs/verification-s01.md` for honest
  failing-first history and clause-level limits. Matrix links rise 106→116/288;
  container tests and partial API/producer links do not prove full compliance.

### ⚠️ S01 migration notes

- `TimelineBuilder::finish()` now returns `Result<OfflineTimeline>`; propagate
  construction errors with `?`. `Finding`/`FindingEvidence` fields are private;
  use checked construction, getters and consuming evidence builders.
- `TimelineFinding` adds required `evidence`; external struct literals must
  provide it. Existing scalar views and `timeline::Severity` remain available;
  the latter now re-exports the common `findings::Severity` type.
- Offline/stream exact half-wrap RTC displacements now return
  `AmbiguousCounterDelta` instead of silently selecting a direction/reset.
  Supply explicit ordering to `SessionRtc::delta_to`, or an actual session
  boundary to ingestion; rejected ambiguous input does not consume packet ID.
  `PacketKey::delta_to` checks session and signed subtraction overflow.
- New non-exhaustive `TimeError` variants report counter width, ambiguity,
  cross-session subtraction and quantity overflow. Legacy `Rtc::from_raw`
  still explicitly masks; use `checked_rtc` to reject high bits. Signed
  conversion refuses overflow/negative-to-unsigned rather than truncating.
- Policies accept approved TF-029/030/031 severity overrides; TF-031 WARNING
  is owner-accepted (2026-11-14). Only TF-032 and unknown IDs remain rejected. Shared catalog defaults follow architecture §8;
  level selection never repairs malformed input. No CLI/schema/dependency or
  published version change is made. The Q-CH4 warning now cites the accepted
  2026-11-12 caller-context decision; the unsafe prototype conversion is not fixed here.


- Owner 2026-11-13 plan decisions (ADR-0018): permanent TF-029 zero-NTP and
  TF-030 NTP ambiguity WARNING findings. TF-031 packet-layout conflict is
  owner-accepted WARNING (2026-11-14, time body kept/EXT-SRC qualification
  withheld); only TF-032 year-conflict level remains pending owner (S08).
  Catalog/requirements only; no new producer is implemented by this docs PR.
- 0.8.0 release scope: defer only COR-111 inverse queries (half-open semantics
  accepted), the new complete ENC-103 secondary/stamp API, complex API-110
  timeline rehydration, and conditional REL-106 prebuilt distribution. These
  remain requirements without coverage claims. Existing encoders/byte sum,
  supported validated loads and complete versioned outputs remain blockers;
  all other requirements remain blockers. Offline drift-corrected query mode
  is after 0.8.0. Trace links cannot express partially covered clauses.
- Accepted versioned snapshot/report JSON with units, decimal-string 64/128-bit
  quantities, validated loads and raw/stated/assumed provenance; schema review
  before freeze. Accepted CLI failure precedence: usage before input, first
  fatal operational failure determines 3/4, later failures diagnostic. These
  are future implementation obligations, not runtime changes in this PR.
- Release policy: both crates at 0.8.0, crates.io only, GitHub Actions Trusted
  Publishing/OIDC only, no stored publishing token or local publish; no prebuilt
  binaries. No workflow, settings, credentials or release action changed.
- C1–C4 acceptance updates documentation/comments only. The legacy runtime basis
  string `floor (L3 proposal)` is deliberately unchanged, including its tests;
  replacing that output requires a separate behaviour migration.

- E2 `stream::StreamTimeline`: Standard preceding-reference default and explicit
  Interpolate, caller progress, fixed advance channel commitment, mandatory
  packet/byte/reference caps, incremental findings and immutable answer basis.
  Hold defaults to1 s+100 ms, gap tolerance to0 ns; every resource/timeout/EOF
  fallback is labelled extrapolated or counter-only. No session/jump/scale bridge.
  Floor interpolation and old-policy mode-change flushing are owner-accepted
  defaults (2026-11-13, C1/C4).
  See `docs/verification-e2.md`: ordinary Format1 projections only; matrix links
  cannot express partial clauses. Network adapters/year inference/serialization
  remain deferred; legacy `streaming` is a separate compatibility API.

- E1 offline core in `timeline`: recording-order session/reset/wrap/late
  handling, caller-fixed or ranked time channels, bounded Format 1 decoding,
  retained reference admission evidence, nearest/preceding lookup, counter-only
  results, complete implemented-policy snapshots, source/format/ITS/scale/year
  basis, and per-session/channel quality. TF severities follow architecture §8.
  Retain the bounded wire reading/layout: TF-005 reserved warnings keep the
  value, and TF-022 leap labels remain available even when projection needs
  context. Existing correlator/streaming entry points remain compatibility APIs; use
  the new builder for policy/session-aware answers. No automatic year inference,
  inverse lookup, drift correction, Format 2 conversion, leap-label arithmetic,
  snapshot serialization or streaming rebuild is claimed by this first part.
  Unsupported calendar boundaries return a labelled counter-only answer.

- E1 reserved SRC values now retain readable valid time with TF-006 warning;
  reserved FMT remains unreadable and excluded. SRC 3 is not GPS.
- E1 verification is **partial at clause level**. The generated trace matrix
  counts test markers and cannot express partially verified clauses within a
  linked L2/L3 requirement; an Implemented row does not mean full compliance.
  See `docs/verification-e1.md` for covered/deferred clauses and mutation evidence.
- The preceding midpoint tie and coarser mixed-resolution jump threshold are
  owner-accepted selectable defaults (2026-11-13, C2/C3). Historical E1 evidence
  is unchanged; this documentation update changes no arithmetic.

### ⚠️ E1 migration notes

- `TimeCorrelator::DEFAULT_OOO_WINDOW{,_NS}` is now **1100 ms**, the accepted
  1000 ms commit +100 ms generation policy, instead of 2 s. `new()` remains the
  explicit legacy unbounded correlator; `TimelineBuilder` uses the bounded
  default. Set an explicit window/policy to retain a different tolerance.
- `compute_quality` and `TimeCorrelator::drift_ppm` now use
  `1e6 * (absolute_delta / RTC_duration - 1)` consistently with L3-COR-113.
  Positive means absolute time advances faster; a 1.001 s RTC interval over
  1 s absolute is -999.000999 ppm. Previous RTC-fast-positive values used
  the opposite sign and a different denominator. Update sign-based checks.
- Add `TimeError::InvalidTimelineInput` for bad policy/identity/query bounds;
  the error enum remains non_exhaustive. No CLI source/output-format changes.

### Changed

- E2 skips non-admitted/missing time references without draining pending data or
  clearing still-eligible predecessors. SRC None/invalid/malformed observations
  keep their findings; both stream modes can continue from earlier valid time.
  Invalid payloads are never interpolation endpoints, even under opt-in admission.
  Only the accepted opt-in stop-on-loss setting adds its source-loss barrier.
- Correct E2 coverage: `raw_network_rate` is inert snapshot metadata and does not
  qualify TF-016. COR-108's raw-network exception remains deferred with Format2.

- Owner decisions: absent required TFMT is reported and skipped in comparison
  by default; only caller opt-in permits default A, labelled default-derived,
  with the missing finding preserved. Requirements only: declaration comparison
  implementation is deferred; no defaults are inserted into TMATS.
- Document `UndefinedSecondaryTimeFormat` as TF-028 ERROR. No error variant,
  parser result or Display string changes; a findings enum/report emitter is
  not implemented yet. Keep RTC values and the independent TF-006 warning.
- Strengthen flag validation with an independent 4x4 truth table over all256
  flags and exact literal error-variant assertions.

### ⚠️ BREAKING CHANGES

- `timeline::TimelineFinding` adds `raw_rtc: Option<Rtc>` (including TF-024
  without a decoded timestamp). Supply the field in struct literals; use `..`
  in destructuring if unneeded. No CLI/output-format changes.

- **Edition 2024 and MSRV 1.85** (was edition 2021 and 1.60), the same across the IRIG 106 ecosystem (owner, 2026-09-27; `docs/adr/0003`). CI checks 1.85; formatting follows the 2024 style edition.

- **Shared types moved to `irig106-types`** (P6-01) — `Rtc`, `Ertc`, `Ch4BinaryTime`, `Ieee1588Time`, `TimeSource`, `TimeFormat`, `DateFormat`, and `SecHdrTimeFormat` are now defined in the new [`irig106-types`](https://github.com/TelemetryWorks/irig106-types) crate and re-exported from their existing `irig106_time` paths, so imports keep working.

  **Migration guide:**
  - `Ch4BinaryTime::to_absolute()` is now provided by the `Ch4BinaryTimeExt` trait. `use irig106_time::*;` or `use irig106_time::Ch4BinaryTimeExt;` brings it into scope.
  - `Ieee1588Time::from_le_bytes`, `Ertc::from_le_bytes`, and `Ch4BinaryTime::from_secondary_bytes` / `from_intra_packet_bytes` now return `irig106_types::TypesError`. `TimeError` implements `From<TypesError>`, so `?` inside functions returning `irig106_time::Result` is unchanged.

- **Shared version API replaces the local `Irig106Version`** (T-9) — existing
  `irig106_time::version::Irig106Version` imports now name the shared enum.
  `Pre07` and `V23` are gone; handle `V04`, `V05` and `Unknown(raw)` explicitly
  in matches and migrate persisted/serde variant names where applicable.
  `from_version_field` is removed: use `Irig106Version::from_setup_record(code)`
  (or `version::detect_version(csdw)`) for an applicable setup record, and
  `Irig106Version::from_data_type_version(byte)` (or
  `version::detect_data_type_version(byte)`) for a header. These are different
  code lists; setup 0 and 0x0F now stay `Unknown`, not `Pre07`/`V23`. There is no
  setup version code at the 106-04/05 boundary, and header 0x01 alone does not
  establish the recording edition. Removed `is_pre_ordering_guarantee` has no
  automatic replacement: provide an explicit caller edition/ordering policy.
  Replace `version.supports_format_2()` with `version::supports_format_2(&version)`;
  this now enables known V17/V19/V22 and returns false for unknown versions.

- **Edition-qualified Format 1 CSDW** (T-6/T-7) — removed
  `Irig106Version::has_gps_time_source` and
  `TimeF1Csdw::time_source_versioned`. SRC 3 is reserved, never GPS or legacy
  None. Use `time_source_for_layout(TimeF1Layout::...)` and match `TimeF1Source`:
  explicit `Legacy04` returns EXT bit 0 plus reserved bits 3–1, while
  `Source05`/`SourceWithIts17` return SRC. For legacy FMT use
  `time_format_for_layout`; FMT 15 is reserved in 106-04 and None from 106-05.
  Existing `time_source()`/`time_format()` accessors mean modern 106-05+ only.
  New `payload_is_valid`, `irig_time_source_for_layout`, and `validate_reserved`
  expose validity, applicable ITS, and reserved diagnostics independently;
  reserved validation must not discard the decoded value. Layout is a required
  caller declaration on these APIs, not an inference from the header version.

- **Correct BCD body sizes** (T-12) — `DayFormatTime::to_le_bytes()` returns
  `[u8; 6]` instead of `[u8; 8]`; `DmyFormatTime::to_le_bytes()` returns `[u8; 8]`
  instead of `[u8; 10]`. Their parsers require at least 6/8 bytes respectively
  and inspect exactly those bytes, excluding CSDW, trailer filler and checksums.
  Resize caller buffers and length checks; do not append a phantom reserved
  word. Encoded bodies are two bytes shorter. The CLI now accepts exact 6/8-byte
  consumer bodies instead of reporting buffer-too-short errors, so those
  packets can contribute time channels and correlation references to output.

- **Flag decoding and exhaustive matches** (T-1) —
  `IntraPacketTimeFormat::from_packet_flags` now uses bit 7 for secondary
  presence, bit 6 for stamp source, and bits 3–2 for format; the former bit-2
  presence interpretation was wrong. Bit 6 clear selects RTC; inconsistent
  secondary-source flags yield `InvalidFlags(raw)`. Under accepted Q-FLAGS,
  reserved format 11 invalidates only secondary-derived stamps; RTC stamps
  still decode (see migration below). `parse_intra_packet_time` returns
  `TimeError::InvalidPacketFlags { flags }` for a missing required header
  instead of falling back to RTC. Add match arms for `IntraPacketTimeFormat::InvalidFlags` and
  `PacketStandard::Unknown`: these two enums are **not** `non_exhaustive`, so
  existing exhaustive matches break. `PacketStandard::from_version(Unknown(_))`
  now returns `Unknown`, not `Ch11`; both `is_ch10()` and `is_ch11()` return
  false, and Display emits `Unknown`. `TimeError` remains `non_exhaustive`.

- **Q-FLAGS reserved-value refinement** — flags `0x0C`/`0x8C` now select
  `Rtc48` and decode the stamp instead of returning `InvalidPacketFlags`.
  Call `intra_packet::validate_reserved_time_format(flags)` separately: its
  `TimeError::ReservedPacketTimeFormat { flags }` is a **TF-006 warning**,
  with the RTC value kept; it is not TF-005 reserved padding. Call
  `validate_secondary_time_format(flags)` separately for selected/present
  secondary time: `0x8C` is unreadable there despite its usable RTC stamp;
  `0x4C` reports undefined format independently of its missing-header error.
  For `0xCC`, stamp parsing now returns the distinct
  `TimeError::UndefinedSecondaryTimeFormat { flags }` rather than
  `InvalidPacketFlags`; no time is returned. `0x4C` and `0x40` retain the
  missing-header error (`InvalidPacketFlags`, TF-015), with raw flags and
  separate reserved-format validation preserving simultaneous evidence.
  Update error handling for the new variants and invoke separate validation
  when reporting findings. `TimeError` remains `non_exhaustive`.
  Owner 2026-11-13 maps `UndefinedSecondaryTimeFormat` to **TF-028 ERROR**;
  TF-001..TF-027 and the TF-006 warning level remain unchanged.
  No CLI behaviour/output changes in this slice.

- **Corrected shared types at `48f1834`** — replace `TimeSource::InternalRtc`
  with `InternalFromRmm` (SRC 2) and `TimeSource::Gps` with `Reserved(3)`;
  GPS remains a `TimeFormat`, not a source. Match the new `TimeFormat::None`
  for modern FMT 0xF (previously Reserved), including serialized variant changes.
  `IrigTimeSource` exposes ITS. `Ertc::to_nanos()` now uses **1 ns/tick** rather
  than 100 ns/tick; remove caller compensation for the old scale.
  `Ch4BinaryTime::from_secondary_bytes`/`from_intra_packet_bytes` now read
  `[reserved, microseconds, low, high]` u16 LE words instead of
  `[unused, high, low, microseconds]`, preserving the full 32-bit binary count,
  including low bit 15. Update manually packed fixtures and any workaround.
  The time crate's secondary/intra-packet paths now reject microseconds above
  9999 as `OutOfRange` (TF-004). Reserved words/padding remain usable values
  with separate TF-005 diagnostics: call `intra_packet::validate_rtc48_reserved`,
  `secondary::validate_ch4_reserved`, and `secondary::validate_secondary_reserved`
  as applicable, without propagating a reserved-warning result as a parse error.
  These validators return `BufferTooShort` for incomplete buffers. The prototype
  `Ch4BinaryTimeExt::to_absolute()` calendar conversion still has the incorrect
  17-bit split; it is documented as untrusted pending Q-CH4, not fixed here.
  CLI labels change only as required by corrected enums: source 2 is
  `Internal from RMM` (was `Internal RTC`), source 3 is `Reserved` (was `GPS`),
  and modern FMT 0xF is `None` (was `Reserved`); native GPS FMT 5 stays `GPS`.

- **Unit and epoch newtypes replace raw integers** (P6-01, team review #3) — `UnixSeconds`, `TaiSeconds`, `TaiUtcOffset`, and `NanosDuration` (from `irig106-types`, re-exported at the crate root).

  | Before | After |
  |--------|-------|
  | `TimeCorrelator::with_ooo_window(Option<u64>)` | `with_ooo_window(Option<NanosDuration>)` |
  | `detect_time_jump(ch, 1_000_000_000)` | `detect_time_jump(ch, NanosDuration::new(1_000_000_000))` |
  | `StreamingTimeCorrelator::new(60_000_000_000)` | `StreamingTimeCorrelator::new(NanosDuration::new(60_000_000_000))` |
  | `PtpTime::to_utc_seconds(37)` / `to_absolute(37)` | `to_utc_seconds(TaiUtcOffset::new(37))` / `to_absolute(TaiUtcOffset::new(37))` |
  | `LeapSecondTable::offset_at_unix(u64) -> i32` | `offset_at_unix(UnixSeconds) -> TaiUtcOffset` |
  | `LeapSecondTable::offset_at_tai(u64) -> i32` | `offset_at_tai(TaiSeconds) -> TaiUtcOffset` |
  | `LeapSecondTable::offset_for_f1(..) -> i32` | `offset_for_f1(..) -> TaiUtcOffset` |
  | `LeapSecondTable::is_near_leap_second(u64, u64)` | `is_near_leap_second(UnixSeconds, u64)` |
  | `DEFAULT_TAI_UTC_OFFSET: i32` | `DEFAULT_TAI_UTC_OFFSET: TaiUtcOffset` |

  `offset_at_*` now return `TaiUtcOffset` so they compose directly with `to_utc_seconds` / `to_absolute`; use `.get()` for the raw `i32`.

- **Slice 2 Format 2 wire correction** (T-2/T-3) — NTF is bits 7–4, TS is
  bits 3–0, reserved bits are 31–8. `time_protocol()` still returns broad
  NTP/PTP for compatibility; use `time_format()` to distinguish NTPv3,
  IEEE 1588-2002 and IEEE 1588-2008. `time_status()` preserves invalid and
  reserved statuses. `parse_time_f2_payload()` now rejects TS other than 1
  (`InvalidTimeStatus`) and reserved NTF; use `decode_time_f2(payload,
  data_length)` to retain raw invalid-status readings separately. Reserved
  bits remain independent warnings, not reasons to discard values.
  `PtpTime.seconds` changes **u64 -> u32**, and its wire parser/encoder changes
  **10 -> 8 bytes** (`[u8; 8]` output), seconds at [0..4), nanos at [4..8).
  Update struct literals, buffers, property generators and manually packed
  fixtures; remove 48-bit packing and never consume filler as PTP data.

- **NTP conversions explicitly unresolved** (Q-NTP) — `to_unix_seconds()`
  changes `Option<u64> -> Result<u64>` and `to_nanos_since_ntp_epoch()` changes
  `u64 -> Result<u64>`. These and `to_absolute()` return
  `TimeError::NeedsContext` until era/UTC/leap semantics are reviewed.
  No implicit era zero or present-era guess remains. Handle this refusal;
  use `raw_fixed_point_nanos()` for a modular numeric view with no inferred
  era. NTP reference admission through the unchanged correlator now propagates
  the refusal. CLI source is unchanged, but NTP packets can therefore report
  an error instead of a fabricated absolute reference; invalid TS is also
  rejected through the corrected parser.

- **Secondary checksums follow the byte reading** (RDG-008, still suspect) —
  `validate_secondary_checksum()` and `parse_secondary_header()` now require
  the sum of bytes [0..10), not five words. Word-only matches no longer admit
  trusted own time. `inspect_secondary_checksum()` exposes stored, byte and
  word sums and both validity flags, including word-only (TF-027/TF-012),
  without discarding evidence. Migrate fixtures/checksum generators; do not
  switch readings silently to keep a word-only recording accepted.

- **Scale-qualified wire readings and leap seconds** (T-16/T-17) — new
  `format1::decode_time_f1` and `network_time::decode_time_f2` take explicit
  Data Length and retain scale/provenance. Native GPS stays GPS; IRIG's default
  UTC assignment is labelled policy; RTC is unknown unless caller-assumed.
  `CalendarReading` preserves eligible UTC 23:59:60, including known-year
  checks. Its legacy projection refuses leap labels and non-UTC scales rather
  than normalizing/relabeling them. Legacy body-only BCD/calendar APIs remain
  unqualified and strict; migrate to the new reading APIs for leap/scale support.
  `TimeScaleTagged` reports Unknown for unqualified calendar/counter values;
  NTP and PTP have the standard-stated UTC/TAI labels. No correlation/stream
  scale integration or leap arithmetic is claimed in this slice.

- **Recording events deprecated, retained** (T-8, ADR-0009) —
  `RecordingEvent` and `RecordingEventType` now carry deprecation warnings.
  Their fixed event meanings are unsupported by §11.2.7.3; body decoding
  belongs in `irig106-decode`, with setup definitions from `irig106-tmats`.
  No replacement availability is claimed and no public API is removed here.

### Added

- `Ch4BinaryTimeExt` trait (see above).
- `PtpTime::tai_seconds() -> TaiSeconds`.
- `TimeCorrelator::DEFAULT_OOO_WINDOW: NanosDuration` (`DEFAULT_OOO_WINDOW_NS` is kept for comparison with `ooo_window_ns()`).
- `TimeError::Types(TypesError)` for `irig106-types` errors without a more specific equivalent.
- `Ertc::from_raw` and public `TimeSource::from_raw` / `TimeFormat::from_raw` (via `irig106-types`).

### Fixed

- Clippy `collapsible_match` in `AbsoluteTime::sub_nanos_year_rollover` (new lint in current stable).

## [v0.7.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.7.0) - 2026-03-29

### ⚠️ BREAKING CHANGES

- **`AbsoluteTime` restructured with `u64` internal representation** (P4-04) — Fields are now accessed via methods instead of direct field access. The internal representation is a single `u64` (nanoseconds since start of day 1), making `add_nanos`/`sub_nanos` single arithmetic operations on the common path.

  **Migration guide:**
  - Field reads: `t.hours` → `t.hours()`, `t.day_of_year` → `t.day_of_year()`, `t.nanoseconds` → `t.nanoseconds()`, etc.
  - Optional fields: `t.year` → `t.year()`
  - Field mutations: `t.year = Some(2025)` → `AbsoluteTime::new(...).unwrap().with_year(Some(2025)).unwrap()`
  - Struct literals: `AbsoluteTime { day_of_year: 100, hours: 12, ... }` → `AbsoluteTime::new(100, 12, ...)`

- **`AbsoluteTime` no longer holds calendar metadata** — The `month`, `day_of_month`, `set_month()`, `set_day_of_month()`, and `with_date()` APIs have been removed. Calendar date information is now carried by the new `CalendarTime` type, which enforces that year, month, and day are all present and valid at construction time. This eliminates the class of bugs where partial calendar state (e.g., month set without day) could be constructed.

  **Migration guide:**
  - `t.month()` / `t.day_of_month()` → use `CalendarTime` instead of `AbsoluteTime`
  - `t.set_month(Some(3))` / `t.set_day_of_month(Some(15))` → `CalendarTime::new(t, 3, 15)?`
  - `t.with_date(2025, 3, 15)?` → `CalendarTime::from_parts(2025, 3, 15, doy, h, m, s, ns)?`
  - `DmyFormatTime::to_absolute()` → `DmyFormatTime::to_calendar_time()` (returns `CalendarTime`)
  - `From<NaiveDateTime> for AbsoluteTime` → `From<NaiveDateTime> for CalendarTime`
  - `CalendarTime` implements `Deref<Target=AbsoluteTime>`, so all `AbsoluteTime` methods work directly on `CalendarTime` values
  - To convert back: `let abs: AbsoluteTime = calendar_time.into();`

### Added

- **`CalendarTime` type** (L3-ABS-006) — Calendar-aware absolute time with validated year, month, and day-of-month. Wraps `AbsoluteTime` with `Deref<Target=AbsoluteTime>`. Constructed via `CalendarTime::new(abs_time, month, day)` or `CalendarTime::from_parts(year, month, day, doy, h, m, s, ns)`. Enforces at the type level that calendar metadata is either fully present and valid, or absent entirely.
- **`DmyFormatTime::to_calendar_time()`** — Replaces `to_absolute()` for DMY BCD format. Returns `CalendarTime` with validated year/month/day.
- **`From<CalendarTime> for chrono::NaiveDateTime`** — Direct conversion using validated month/day fields.
- **`From<chrono::NaiveDateTime> for CalendarTime`** — chrono always provides full date, so this returns `CalendarTime` (was `AbsoluteTime`).
- **`AbsoluteTime::as_total_ns()`** — Exposes the raw internal nanosecond count for efficient comparison and serialization.
- **`AbsoluteTime::with_year()`** — Consuming builder method that enriches a DOY timestamp with year metadata (0–9999). Returns `Result<Self>` for chaining: `AbsoluteTime::new(...).unwrap().with_year(Some(2025)).unwrap()`. Year is metadata, not a calendar invariant — see `CalendarTime` for full date validation.
- **UDP framing documentation** (P5-03) — New `docs/udp_framing.md` documenting that UDP Transfer Format 1 and 2 headers carry no time fields, and how to use `StreamingTimeCorrelator` for live UDP streams.
- **WASM build verification** (P6-06) — CI now verifies `wasm32-unknown-unknown` compilation with `--no-default-features` and `--no-default-features --features serde`.
- **`util` module** (P6-08) — Crate-internal `is_leap_year` and `abs_diff_u64` helpers that replace `u16::is_multiple_of` (Rust 1.87) and `u64::abs_diff` (Rust 1.60) respectively. Each carries a targeted `#[allow(clippy::...)]` and full MSRV dependency documentation. 18 unit tests covering leap year rules (common, century, quad-century, edge cases, IRIG 106 era years) and abs_diff properties (symmetry, extremes, leap second timestamps).
- **11 new integration tests** in `tests/pipeline.rs`: leap year rollover through `sub_nanos` (leap, non-leap, century, quad-century), BCD DMY Feb 29 on leap/non-leap years, `is_near_leap_second` via `abs_diff_u64` (exact boundary, within window, outside, symmetry, far future).

### Changed

- **`AbsoluteTime` internal representation** (P4-04) — Single `u64` replaces 5 numeric fields. `add_nanos` is now a single `u64` addition (was 4-level carry chain). `sub_nanos` year rollover is cleaner arithmetic. `total_nanos_of_day()` is a single modulo operation.
- **Custom serde for `AbsoluteTime`** — Serializes to the same expanded JSON shape as v0.6 (`day_of_year`, `hours`, `minutes`, `seconds`, `nanoseconds`, `year`, `month`, `day_of_month`) for backward compatibility. Deserializes and recomposes the `u64` internally.
- **GitHub Actions CI** — Added `wasm32-unknown-unknown` build job. Separated stable (full `cargo test`) from MSRV 1.60 (`cargo check` only — dev-dependencies require newer Rust). Tests 4 feature combos on stable + WASM + MSRV check.
- **`Cargo.toml`** — Version bumped to 0.7.0. MSRV lowered from 1.87 to 1.60.
- **CLI (irig106-time-cli)** — Version bumped to 0.7.0. All `AbsoluteTime` field accesses migrated to methods.
- **MSRV lowered from 1.87 → 1.60** (P6-08) — Replaced `u16::is_multiple_of` (Rust 1.87) with `util::is_leap_year` and `u64::abs_diff` (Rust 1.60) with `util::abs_diff_u64`. The MSRV is now constrained by `dep:` namespaced features in `Cargo.toml` (Rust 1.60). Each helper carries a targeted clippy `#[allow]` and full documentation of the API it replaces, the Rust version it avoids, and when it can be upgraded.
- **API audit** — Added `Hash` to 25 public types, enabling use as `HashMap` keys and `HashSet` members. Added `Copy` to `TimeJump` and `RtcReset`. `AbsoluteTime` intentionally omits `Ord` because derived `PartialEq` compares all fields including optional year metadata; callers needing within-year ordering should use `as_total_ns()`. `TimeQuality` omits `Hash`/`Eq` (contains `f64`).
- **Documentation** — All code examples in `usage.md` updated for method-based access. Version refs `0.6`→`0.7` across all docs.
- All consumer modules migrated: `bcd.rs`, `chrono_interop.rs`, `network_time.rs`, `correlation.rs`, `quality.rs`, `streaming.rs`, and all test files.
- Total test count: **269** (184 unit + 68 integration + 17 property).

## [v0.6.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.6.0) - 2026-03-29

### Added

- **`streaming` module** (P5-02) — `StreamingTimeCorrelator` with sliding-window eviction for live UDP stream processing. Per-channel `BTreeMap` index with O(log n) lookup. Configurable `max_age_ns` window with automatic eviction of stale references. Methods: `add_reference`, `add_reference_f2`, `correlate`, `evict`, `len`, `is_empty`, `total_evicted`, `max_age_ns`, `latest_rtc`, `channel_ids`, `channel_len`. `StreamingRef` struct for reference points.
- **`quality` module** (P5-04) — `TimeQuality` struct and `compute_quality(refs)` function. Metrics: `total_refs`, `channel_count`, `refs_per_channel`, `max_rtc_gap_ns`, `min_rtc_gap_ns`, `ref_density_per_sec`, `drift_ppm_per_channel`, `rtc_span_ns`.
- **`packet_standard` module** (P5-01) — `PacketStandard::Ch10` / `Ch11` enum for IRIG 106-17 chapter provenance. `from_version()`, `is_ch10()`, `is_ch11()`, `Display`.
- **`recording_event` module** (GAP-08) — `RecordingEventType` enum (Started, Stopped, Overrun, IndexPoint, Reserved) and `RecordingEvent` struct with `has_reference_time()` and `may_cause_time_gap()`. Parses Data Type 0x02 event packets for time context.
- **`chrono_interop` module** (GAP-06) — Feature-gated `From<AbsoluteTime> for chrono::NaiveDateTime` and reverse. Enable with `features = ["chrono"]`.
- **F1 leap second handling** (GAP-04) — `LeapSecondTable::offset_for_f1(year, doy)` for Format 1 time sources (IRIG-B, GPS). `is_near_leap_second(unix_seconds, window_secs)` for flagging packets near leap second boundaries.
- **34 new unit tests** across 5 new test files: `packet_standard_tests.rs` (5), `streaming_tests.rs` (10), `quality_tests.rs` (6), `recording_event_tests.rs` (9), `chrono_interop.rs` (4).
- **7 new integration tests** in `tests/pipeline.rs`: PacketStandard from version, streaming correlator basic pipeline, streaming eviction, quality metrics, F1 leap second offset, near leap second boundary, recording event pipeline.

### Changed

- **`Cargo.toml`** — Version bumped to 0.6.0. Added `chrono` optional dependency and feature gate.
- **CLI (irig106-time-cli)** — Version bumped to 0.6.0.
- **GitHub Actions CI** — Added `--features serde` and `--features chrono` individual test runs to catch cross-feature coupling. Pipeline now tests 4 feature combinations: `--all-features`, `--no-default-features`, `--features serde`, `--features chrono`.
- **Crate docs updated** — `lib.rs` feature list now includes streaming correlation, quality metrics, packet standard, recording events, and chrono interop.
- **Re-exports** — `PacketStandard`, `StreamingTimeCorrelator`, `StreamingRef`, `TimeQuality`, `compute_quality`, `RecordingEvent`, `RecordingEventType` added to crate root.
- **P5-05 (async API)** permanently deferred — async runtime choice belongs to the application layer, not a `#![no_std]` parsing library.
- **Rust formatting** — Ran `cargo fmt` across all new modules.
- Test matrix totals: **244** checks under default features (166 unit + 57 integration + 17 property + 4 doc) and **249** with `--all-features` (170 unit + 57 integration + 17 property + 5 doc).

### Fixed

- **`RtcReset.index` now reports global index** — v0.5.0 regression where `detect_rtc_resets` returned per-channel indices instead of global `references()` indices. Fixed via `global_index_of()` helper that matches on all fields.
- **`TimeJump.index` correct for duplicate RTCs** — v0.5.0 regression where `.position()` matched only on `rtc + channel_id`, always returning the first match for duplicate RTC values. Same `global_index_of()` fix.
- **README/lib.rs serde claim** — Now correctly says "except `TimeError`" to match usage.md and CHANGELOG.
- **Clippy fixes** — Removed unused `Rtc` import in `quality.rs`. Added `alloc::format` and `alloc::vec` imports in `no_std` test modules. Replaced manual abs diff with `abs_diff()` in `is_near_leap_second`.
- **chrono doc test** — Replaced `ignore` fence with real ```` ```rust ```` doc test that compiles under the `chrono` feature gate.

## [v0.5.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.5.0) - 2026-03-29

### Added

- **Channel-indexed correlation** (P4-01) — Per-channel `BTreeMap<u16, Vec<ReferencePoint>>` index alongside the flat RTC-sorted vec. `nearest_for_channel` is now O(log n) binary search instead of O(n) linear scan. Methods upgraded: `correlate(…, Some(ch))`, `detect_time_jump`, `drift_ppm`, `detect_rtc_resets`.
- **Per-channel accessors** (P4-02) — `TimeCorrelator::channel_references(channel_id)` returns a sorted slice for a single channel. `channel_ids()` returns all active channel IDs.
- **BCD lookup table** (P4-03) — `extract_bcd_digit` now validates nibbles via a 16-entry const `BCD_LUT` instead of a branch, eliminating conditional logic in the hot path.
- **Criterion benchmarks** (P4-06) — `benches/correlation_bench.rs` with statistical benchmarks for `correlate_any`, `correlate_by_channel`, `detect_time_jump`, `drift_ppm`, and `add_reference` at 10/100/1000/3600 reference point scales.
- **`serde` feature gate** (GAP-02) — Optional `Serialize`/`Deserialize` derives on all public data types (except `TimeError` which contains `&'static str` fields). Enable with `features = ["serde"]`.
- **7 new integration tests** in `tests/pipeline.rs`: channel_references accessor, channel_ids, channel-indexed correlate consistency, large-set correlate, sub_nanos year boundary crossing, same-day no-year-change, and no-year-info graceful wrap.

### Changed

- **`Cargo.toml`** — Version bumped to 0.5.0. Added `serde` optional dependency and feature gate. Added `criterion` to dev-dependencies with `correlation_bench` bench target.
- **CLI (irig106-time-cli)** — Version bumped to 0.5.0.
- **Test structure** — Extracted `version.rs` inline tests to `src/version_tests.rs` to match the crate's test-per-module convention.
- **Crate docs updated** — `lib.rs` feature list now includes channel-indexed O(log n) lookup and serde support.
- Total test count: **203** (136 unit + 50 integration + 17 property).

### Fixed

- **`AbsoluteTime::sub_nanos` year rollover** (GAP-05) — Subtracting past day 1 now correctly decrements the year (accounting for leap years) instead of wrapping to day 366. When no year info is available, assumes 365 days.

## [v0.4.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.4.0) - 2026-03-28

### Added

- **`version` module** (P2-03) — `Irig106Version` enum with variants `Pre07` through `V23` plus `Unknown(u8)`. `detect_version(tmats_csdw)` extracts the version from bits \[7:0\] of the TMATS CSDW. Helper methods: `is_pre_ordering_guarantee()`, `supports_format_2()`, `has_gps_time_source()`.
- **Version-aware CSDW parsing** (P2-04, P2-01) — `TimeF1Csdw::time_source_versioned(version)` disambiguates time source value 3, which was "None" in 106-04 but "GPS" from 106-05 onward. Pre-07 files return `Reserved(3)` to signal ambiguity.
- **Configurable out-of-order window** (P2-02) — `TimeCorrelator::with_ooo_window(ooo_window_ns)` constructor. Pre-105 files may need unbounded OOO tolerance; post-105 defaults to 2 seconds via `DEFAULT_OOO_WINDOW_NS`. Accessor: `ooo_window_ns()`.
- **RTC reset detection** (GAP-07) — `TimeCorrelator::detect_rtc_resets(channel_id)` identifies counter resets (as opposed to 48-bit wraps) by checking whether RTC went backward while absolute time continued forward. Returns `Vec<RtcReset>` with before/after RTC and absolute time values.
- **`to_le_bytes()` encoding** (GAP-11) — Wire-format encoding for all parseable types, enabling packet construction for `irig106-write`:
  - `Rtc::to_le_bytes()` → `[u8; 6]`
  - `TimeF1Csdw::to_le_bytes()` → `[u8; 4]`
  - `TimeF2Csdw::to_le_bytes()` → `[u8; 4]`
  - `NtpTime::to_le_bytes()` → `[u8; 8]`
  - `PtpTime::to_le_bytes()` → `[u8; 10]`
  - `DayFormatTime::to_le_bytes()` → `[u8; 8]`
  - `DmyFormatTime::to_le_bytes()` → `[u8; 10]`
- **19 new integration tests** in `tests/pipeline.rs`: version detection, version-aware CSDW, OOO window, RTC reset detection (basic, no false positive, channel isolation), and `to_le_bytes` round-trip for all 7 types.
- **4 new property tests** in `tests/properties.rs`: encode round-trip for RTC, CSDW, NTP, and PTP.

### Changed

- **GitHub Actions** — Updated `actions/checkout` from `v4` to `v6` across the CI workflow.
- **Rust formatting** — Ran `cargo fmt` across the repository to normalize code style.
- **Crate docs updated** — `lib.rs` feature list now includes version detection, RTC reset detection, and encoding.
- **Re-exports** — `RtcReset`, `detect_version`, and `Irig106Version` added to crate root.
- **`Cargo.toml`** — Version bumped to 0.4.0.
- **CLI (irig106-time-cli)** — Version bumped to 0.4.0.
- Total test count: **196** (136 unit + 43 integration + 17 property).

### Fixed

- **`detect_rtc_resets` sorted by absolute time** — References are stored sorted by RTC for lookup efficiency. After a reset, the low RTC value was inserted before pre-reset values, masking the discontinuity. Now sorts filtered channel references by absolute time to restore temporal order.
- **Removed unnecessary `as u16` casts** in `DayFormatTime::to_le_bytes()` and `DmyFormatTime::to_le_bytes()` — fields already typed `u16` (`milliseconds`, `day_of_year`, `year`) no longer carry redundant casts.

### Known Issues

- **Legacy support is specification-based only.** Version-aware CSDW parsing and OOO tolerance are implemented per the IRIG 106-04/05 specifications and validated with synthetic tests, but have not been tested against real Ch10 files from legacy recorders (Ampex DCRsi, L-3 MARS, Acra KAM-500). Tracked as P1-07 and P2-05.
- **Ch4 Binary Weighted Time bit layout is unverified against multi-vendor samples.** The current decode assumes a single bit-field interpretation. Real-world validation against multiple recorder vendors is needed. Tracked as GAP-03.

## [v0.3.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.3.0) - 2026-03-28

### Added

- **GitHub Actions CI/CD** (P1-03) — Test on stable + MSRV 1.87, clippy, rustfmt, rustdoc, and CLI build/lint.
- **`impl Display for AbsoluteTime`** (GAP-01) — Formats as `YYYY-MM-DD HH:MM:SS.mmm.uuu` when year/month/day are available, or `Day DDD HH:MM:SS.mmm.uuu` otherwise.
- **`TimeCorrelator::drift_ppm(channel_id)`** (GAP-10) — Estimates RTC clock drift in parts-per-million against absolute time references. Returns average drift across consecutive same-channel reference pairs.
- **Calendar validation in DMY BCD decoding** (GAP-09) — `DmyFormatTime::from_le_bytes` now rejects invalid day-for-month combinations (e.g., Feb 30, Jun 31) via a `days_in_month()` helper that accounts for leap years.
- **CLI `Proto` column** (P1-09/GAP-12) — `channels` command table now shows NTP/PTP protocol identity. Summary view appends `Proto: NTP` or `Proto: PTP` for Format 2 channels. `TimeChannelInfo` carries a `network_protocol` field.
- **`proptest` property-based tests** (P1-08) — `proptest = "1"` added to `[dev-dependencies]`. `tests/properties.rs` rewritten with `proptest!` macros covering RTC masking, round-trip, elapsed bounds, absolute time add/sub/monotonicity/Display, IEEE-1588 consistency, CSDW stability, NTP fraction bounds, and PTP monotonicity.
- **10 new integration tests** in `tests/pipeline.rs`: Display formatting (DOY, DMY, zero-padding), drift_ppm (zero drift, fast RTC, channel isolation, insufficient refs), calendar validation, and year overflow guard.

### Changed

- **`#[deny(missing_docs)]`** (P1-04) — Added to crate root. All `pub mod` declarations in `lib.rs` now carry `///` doc comments.
- **Crate docs updated** (GAP-13) — `lib.rs` module-level documentation now includes network time (NTP/PTP) and drift estimation in the feature list. Requirement traceability paths updated to lowercase filenames.
- **`unix_seconds_to_ymd_pub`** (GAP-15) — Visibility reduced from `pub` to `pub(crate)`.
- **Year overflow guard** (GAP-16) — `unix_seconds_to_ymd` now uses `saturating_add` and breaks at `u16::MAX` to prevent panic on malformed timestamps far in the future.
- **`Cargo.toml`** — Version bumped to 0.3.0. `rust-version = "1.87"` MSRV declared.
- **CLI (irig106-time-cli)** — Version bumped to 0.3.0.
- Total test count: **163** (126 unit + 24 integration + 13 property).

### Fixed

- Escaped bit-range bracket notation (`[3:2]`, `[3:0]`, etc.) in rustdoc comments across `absolute.rs`, `csdw.rs`, `secondary.rs`, `intra_packet.rs`, and `network_time.rs` to resolve 9 intra-doc link warnings.
- Resolved `prop_assert!` macro conflict with nested `format!` in property tests.
- Removed unused imports and variables in `properties.rs` and `pipeline.rs`.

## [v0.2.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.2.0) - 2026-03-28

### Added

- **`network_time` module** — Time Data Format 2 (0x12, Network Time) support, introduced in IRIG 106-17 (CR93).
  - `TimeF2Csdw` — Format 2 channel-specific data word parser with `NetworkTimeProtocol` enum (NTP/PTP).
  - `NtpTime` — NTP timestamp (32-bit seconds since 1900-01-01 UTC + 32-bit fractional seconds). Includes `fraction_as_nanos()` (2⁻³² → ns conversion), `to_unix_seconds()`, `to_nanos_since_ntp_epoch()`, and `to_absolute()`.
  - `PtpTime` — PTP/IEEE 1588 timestamp (48-bit seconds since 1970-01-01 TAI + 32-bit nanoseconds). Includes `to_utc_seconds(tai_offset)`, `to_nanos_since_tai_epoch()`, and `to_absolute(tai_offset)`.
  - `NetworkTime` enum — Discriminated union of `Ntp(NtpTime)` and `Ptp(PtpTime)`.
  - `parse_time_f2_payload()` — Full payload dispatcher: CSDW → NTP or PTP time data.
  - `LeapSecondTable` — Built-in table with all 28 TAI-UTC leap seconds from 1972-01-01 (offset 10) through 2017-01-01 (offset 37). Binary search lookup via `offset_at_unix()` and `offset_at_tai()`. Runtime extensible via `add()`.
  - `NTP_UNIX_EPOCH_OFFSET` constant (2,208,988,800 seconds).
  - `DEFAULT_TAI_UTC_OFFSET` constant (37 seconds, since 2017).
- **`TimeCorrelator::add_reference_f2()`** — Accept Format 2 (NTP/PTP) time packets as correlation reference points, with automatic leap-second offset application for PTP sources.
- **22 new unit tests** in `network_time_tests.rs` covering F2 CSDW parsing, NTP parse/convert/epoch math, PTP parse/TAI-UTC conversion, leap second table lookup, and full payload dispatch.
- **5 new integration tests** in `tests/pipeline.rs`: full NTP pipeline, full PTP pipeline, mixed F1+F2 correlation, NTP sub-millisecond precision, and leap second table historical accuracy.
- **2 new fuzz targets**: `fuzz_ntp` and `fuzz_ptp` covering NTP and PTP parsing entry points.
- **7 new benchmarks**: `ntp_from_le_bytes` (1.1 ns), `ntp_to_absolute` (115 ns), `ptp_from_le_bytes` (1.6 ns), `ptp_to_absolute` (119 ns), `leap_table_lookup` (6.6 ns), `f2_ntp_payload_parse` (2.7 ns).
- **L1 Requirements updated** — 16 new L1 requirements for Format 2 (F2CSDW, NTP, PTP, F2COR, TAI) merged into `L1_Requirements.md`, traced to Ch11 §11.2.3.3. Total: 53 L1 requirements.
- **L2/L3 Requirements updated** — Format 2 functional decomposition and design specifications merged into `L2_Requirements.md` (§3.10–3.14) and `L3_Requirements.md` (§3.12–3.13) with extended traceability matrices.
- **CLI (irig106-time-cli)** — `ch10time` now recognizes and processes Type 0x12 Network Time packets in all commands (summary, channels, jumps, timeline, csv, correlate). Uses built-in leap second table for PTP→UTC conversion.

### Changed

- Re-exports at crate root now include all Format 2 types: `TimeF2Csdw`, `NetworkTimeProtocol`, `NtpTime`, `PtpTime`, `NetworkTime`, `LeapSecondTable`, `LeapSecondEntry`, `NTP_UNIX_EPOCH_OFFSET`, `DEFAULT_TAI_UTC_OFFSET`.
- Total test count: **151** (126 unit + 15 integration + 10 property).
- Total fuzz targets: **10** (8 original + 2 new).
- Total benchmarks: **30** (23 original + 7 new).
- CLI `process_time_f2_packet` refactored to use `add_reference_f2()` instead of manually duplicating NTP/PTP→AbsoluteTime conversion logic.
- F2 L1 requirements merged from standalone addendum into canonical `L1_Requirements.md` (§3.10–3.14).
- F2 L2/L3 requirements merged from standalone addendum into canonical `L2_Requirements.md` (§3.10–3.14) and `L3_Requirements.md` (§3.12–3.13). Removed `L2L3_REQUIREMENTS_F2_ADDENDUM.md`.
- `usage.md` updated with 3 new Format 2 sections (§13 Processing Network Time Packets, §14 Working with the Leap Second Table, §15 Correlating with F1 + F2 Sources), updated imports and version references throughout.

### Fixed

- Resolved clippy `manual_is_multiple_of` in `network_time.rs` leap year check.
- Resolved clippy `manual_range_contains` in `network_time_tests.rs` and `pipeline.rs` NTP fractional precision assertions.
- Resolved clippy `unused_imports` in `pipeline.rs` F2 integration tests.
- Resolved clippy `unused_must_use` in `time_benchmarks.rs` for `ntp_to_absolute` and `ptp_to_absolute` benchmarks.
- Resolved clippy `needless_borrow`, `for_kv_map`, `redundant_closure`, `single_match`, and `manual_is_multiple_of` in CLI.

### Known Issues

- CLI `channels` command displays NTP sources as "External / UTC" and PTP sources as "External / GPS" — the specific network protocol identity is not shown. Tracked as GAP-12, planned for v0.3.0 (P1-09).

## [v0.1.0](https://github.com/TelemetryWorks/irig106-time/releases/tag/v0.1.0) - 2026-03-26

### Added

- **`rtc` module** — 48-bit Relative Time Counter newtype (`Rtc`) with `from_le_bytes`, `from_raw`, `elapsed_ticks`, `elapsed_nanos`, `to_nanos`, and wrap-around safe arithmetic.
- **`absolute` module** — `AbsoluteTime` with nanosecond precision, optional DMY calendar date, `add_nanos`/`sub_nanos` with carry logic. `Ch4BinaryTime` (Chapter 4 Binary Weighted Time), `Ieee1588Time`, and `Ertc` (64-bit Extended RTC).
- **`csdw` module** — `TimeF1Csdw` parser for Time Data Format 1 (0x11) channel-specific data word. `TimeSource`, `TimeFormat`, and `DateFormat` enums with bitfield extraction.
- **`bcd` module** — `DayFormatTime` (8-byte DOY BCD) and `DmyFormatTime` (10-byte DMY BCD) decoders with full BCD nibble validation, reserved-bit checking, and range validation.
- **`secondary` module** — Secondary header checksum validation (`validate_secondary_checksum`) and time parsing for Ch4, IEEE-1588, and ERTC formats via `parse_secondary_header`.
- **`intra_packet` module** — `IntraPacketTime` enum with 4 variants (Rtc, Ch4, Ieee1588, Ertc) and `parse_intra_packet_time` format dispatcher.
- **`correlation` module** — `TimeCorrelator` with sorted-insert reference points, nearest-point correlation with optional channel filtering, and `detect_time_jump` for GPS lock discontinuity detection.
- **`error` module** — `TimeError` non-exhaustive enum with 6 variants: `InvalidBcdDigit`, `ReservedBitSet`, `OutOfRange`, `ChecksumMismatch`, `NoReferencePoint`, `BufferTooShort`. `Display` impl and feature-gated `std::error::Error`.
- **`#![no_std]` support** — Crate compiles with only `core` and `alloc`. Optional `std` feature for `Error` impl.
- **Zero required dependencies** in `[dependencies]`.
- **Zero `unsafe` blocks** in the entire crate.
- **`#[inline]` annotations** on all hot-path functions.
- **104 unit tests** across 8 modules (tests in separate files alongside source).
- **10 integration tests** in `tests/pipeline.rs` covering full CSDW-to-correlation flows, multi-channel scenarios, and GPS lock jump detection.
- **10 property-based tests** in `tests/properties.rs` (10,000 iterations each, zero external dependencies).
- **23 benchmarks** in `benches/time_benchmarks.rs` (zero-dependency, `std::time::Instant`).
- **8 fuzz targets** in `fuzz/fuzz_targets/` covering all parsing entry points.
- **Documentation**: L1/L2/L3 requirements (37/78/65), architecture doc, security doc, test index, project structure, shared types migration plan, roadmap, and rationale for separate repo.
- **13-slide technical presentation** (PPTX).
