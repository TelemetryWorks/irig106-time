# Snapshot JSON v1 — complete candidate for one owner review

**Status: review candidate, NOT FROZEN.** Owner Q5 (2026-11-13) governs this
protocol; IRIG106 does not prescribe JSON. S12b1, S12b2a and S12b2b implement
exact values, full policy, answer/report output and checked replay loads.
Review the entire family once before freeze; no owner approval is recorded here.
Sources: ADR-0010/0011; L2/L3 POL-103, API-107/110; owner Q3/Q5 and C1-C4.

## Review card: root objects and supported loads

Every root requires its exact `schema`, numeric `version:1` and specified units.
Policy-dependent roots additionally require numeric `defaults_version:1`.
Unknown/missing/duplicate fields, unknown versions and wrong/extra/missing units
reject on supported loads. These are JSON objects, not positional root tuples.

| Public type / schema prefix `irig106-time/` | Other fields | Load rule |
|---|---|---|
| ScalarSnapshot / `scalar` | kind, unit, value | Checked canonical string/range/kind/unit; no defaults_version. |
| CalendarSnapshot / `calendar` | units, value | Gregorian/partial/scale/RDG010 validation; no defaults_version. |
| LeapTableSnapshot / `leap-table` | units, value={entries,metadata} | Original validated table constructor; no defaults_version. |
| PolicySnapshot / `policy` | defaults_version, units, value={Offline or Stream} | All native policy/config/table/range/pivot validators. |
| AnswerOutput / `answer-output` | defaults_version, units, value={basis,value} | Output only. A bare serialized basis is not trusted as a loadable answer. |
| ReportSnapshot / `report` | defaults_version, units, plan, outputs={answers,findings,quality,sessions} | Caller-limit preflight, then replay typed plan; reject any cached-output mismatch. |
| AnswerSnapshot / `answer` | defaults_version, units, value={report,index} | Same caller-limit preflight for nested report plus an in-range decimal answer index. |

Fields in the inventory below are exhaustive for the library value/config
shapes emitted by these roots. `Option` emits null when absent, vectors emit
arrays, structs emit objects. Rust enums use their exact case-sensitive names:
unit variants are strings; payload variants are one-key objects (positional
payloads are arrays). No enum name is an implicit edition/UTC/year assumption.
Private indexes, live queues and timeline/stream engine state are not encoded.

**Compatibility:** all policy defaults are materialized or explicitly selected,
not looked up by ID. Compare cached diagnostics as well as numeric facts: the
current replay contract rejects any mismatch, including changed diagnostic text.
A later behavior/schema change therefore needs compatible versioned migration
or a new version, not silent cache repair. This is validation, not authentication:
changing inputs and outputs consistently describes a different valid report.

## Integer/metric and unit rules

In time-owned snapshots/reports/adapters, all64/128-bit integer quantities,
shared signed types and platform counts are
canonical decimal **strings**, even below2^53: `0`, positive digits without
leading zero, or minus plus nonzero digits. No `+`, `-0`, spaces, exponent,
fraction, numeric token or overflow. Decimal parsing checks destination width;
platform counts exceeding the receiving usize reject (including32-bit/WASM).
RTC must fit48bits. ERTC is unsigned64 at1ns/tick. No saturation or masking.
PTP seconds is u32 in Fig11-17; it is quoted by the requested adapter audit and
still rejects values above u32-max. NTP's raw words remain u32 JSON numbers.
Other8/16/32-bit fields stay numbers. Unwrapped reexports from irig106-types
retain that crate's Serde contract and are NOT this wire format; use
ScalarSnapshot or the time-owned field wrappers, not bare shared Serialize. Numeric map keys are decimal JSON member
strings; duplicate keys reject. Counts/IDs never pass through floats.

Quality density/drift are **finite approximate f64 numbers**, not exact integer
quantities; unavailable measurements are null with a reason, never zero-filled.
All time/counter arithmetic retains checked integer quantities. NaN/Inf reject.

| Unit/role | Fields/types |
|---|---|
| ns | `*_ns`, nanoseconds, NanosDuration/SignedNanos, ERTC, counter duration, tolerances/intervals/deadlines/progress/late/hold/gap; nanoseconds_floor |
| TAI ns since1970 | OwnPoint.tai, OwnUtc.tai, Normalization.tai_ns and derived_tai_ns |
| Native-scale ns | Normalization.native_ns; interpret with retained scale/epoch, never implicit UTC |
| RTC ticks (100ns) | Rtc/SignedRtcTicks, PacketKey/ReplayKey.ticks, governing range start/end, session first/high ticks, RtcErtcCheck.expected/delta/distance, named `*rtc_ticks*`/`*delta_ticks*`/year.gap_ticks |
| ns remainder0..99 | RtcErtcCheck.remainder |
| Unix-style UTC seconds since1970 | UnixSeconds/SignedUnixSeconds, NtpPivot.seconds, mapped_unix, effective_unix, supported_start, known_current_through; no cumulative leap count |
| TAI seconds since1970 | TaiSeconds, PtpTime.seconds; PTP nanoseconds component has ns units |
| NTP seconds modulo2^32 since1900 /2^-32s | NtpTime.seconds / fraction; PivotFraction discriminant distinguishes Nanoseconds from NtpUnits |
| Seconds offset | TaiUtcOffset, applied_seconds/applied_offset, tai_minus_native_seconds, tai_minus_utc_seconds |
| Era count | EraDeclaration/ResolvedNtp.era, ntp_era; signed count of2^32-second eras |
| ms | DayFormatTime/DmyFormatTime.milliseconds (10ms resolution) |
| Gregorian fields | year/supplied/caller/recording/year candidates: year; day_of_year/full_day/day_of_month: day; hours:h, minutes:min, seconds:s, nanoseconds:ns; unknown year is null |
| Count / identity | packet/query/setup/policy IDs, session/segment/index, counts, phase (year-phase displacement); tuple year.packet_year=(year,packet ID) |
| Bytes | byte_offset, Data Length/consumed, byte caps, raw byte budget, BufferTooShort.expected/actual |
| Codes / bit patterns | type/version/FMT/SRC/ITS/flags/LY/word/reserved/checksum fields; raw words retain original weighting/layout context |
| ppm / reference/s | OfflineQuality.drift_ppm / references_per_second |
| Contextual raw fields | RawField.name defines the observed/expected quantity: suffix `_ns`/nanoseconds, `_ticks`/rtc_ticks, `_seconds`, `_unix_seconds`, year.selected, ntp_era; remaining named flags/csdw/checksum/version/source fields are raw codes/bitmasks. `standard_minimum_1hz_applies` is0/1. OutOfRange.value/max inherit the named field's units. |

YearBasis.phase_residual is `(related packet ID,residual ns,threshold ns,inclusive
edge bool)`. ReferenceInput.wire is `(explicit layout,Format1Reading)`.
FindingCode is `(numeric approved TF suffix,permanent title)`; all32 severities
are owner-approved, with overrides recorded separately. RawValue.Signed/
Unsigned use exact128-bit strings; their unit is the enclosing named field.
TypesError inside TimeError.Types is retained as external diagnostic display
text; the source bytes and typed recipe permit regeneration, not opaque loading.

The answer/report unit map is exactly:
`bytes=byte; calendar=year/day/h/min/s/ns; counter=rtc_ticks;
density=reference/s; drift=ppm; duration=ns; identity=count;
ntp_fraction=2^-32 s; tai=tai_ns; unix=unix_s`.
The inventory's type/field roles above disambiguate each component/epoch.

## Pure values and policy wire details

Scalar fields: schema/version/kind/unit/value. Kinds/units:
rtc/rtc_ticks; ertc/ns; nanos/ns; signed_nanos/ns; signed_ticks/rtc_ticks;
unix/unix_s; signed_unix/unix_s; tai/tai_s; u64/i64/u128/i128/count.
Calendar units are exactly year/year,day_of_year/day,hours/h,minutes/min,
seconds/s,nanoseconds/ns. The value contains year/day/clock/nanoseconds and
scale={stated,effective,origin}. Standard requires equal known scales; Policy
requires unknown stated and known effective; Unspecified requires both unknown.
Native GPS stays GPS. Leap labels/partial year follow RDG010, not inferred dates.

Leap units are exactly effective_unix/unix_s,tai_utc_offset/s,
supported_start/unix_s,known_current_through/unix_s. Entries contain
{effective_unix,tai_utc_offset}; metadata contains {id,source,supported_start,
known_current_through}. Order/history/steps/coverage validate; nothing is fetched.

Policy units: buffer_bytes/byte,counter/rtc_ticks,duration/ns,identity/count,
ntp_fraction/2^-32 s,unix/unix_s. Offline contains every OfflinePolicy field in
the inventory. Stream contains id/mode/offline/hold_tolerance_ns/hold_interval_ns/
raw_network_rate/caps. Caps={packets,bytes,references}; overrides are unique
[TF suffix,severity] pairs. Shared is null or contains id/caller_year/year_order/
scales/ieee/ch4/table/counter_applicability/own_tolerance/declarations/
use_tfmt_default/ntp. Epochs={origin:CalendarSnapshot,counting,source}; full
LeapTableSnapshot retained. Named sources and Civil second60 constraints validate.

Policy declaration variants: CallerPrepared, Absent, Present(array). Each setup
has session/start/end/setup_packet/provenance/declared_version/ri4/
recording_year/channels. Channel attributes remain raw, missing TFMT stays null;
no default is inserted. Setup ranges validate and canonically sort. Policy ntp
is a unique key/session/declared/pivots entry list; declared={era,id}. NtpPivot
fields are in the inventory; origin variants are CallerApproximateDate,
DatedReference{packet}, ResolvedNtp{era,kind}. Validate units/provenance/session/
era/calendar and reject zero sentinel or AssumedZero as independent evidence.

## Checked replay, supported load boundaries and resource rules

This is our Q5 JSON protocol, not an IRIG106 format. Owner Option A
(2026-11-17): the CALLER sets load limits, never the document. ReplayPlan
contains declared ceilings, a clock recipe and ordered requests. On load,
`ReportSnapshot::load(deserializer, &ReplayLimits)` and
`AnswerSnapshot::load(deserializer, &ReplayLimits)` require every declared
ceiling and every actual input count to fit independent caller ceilings,
BEFORE replay. Nested answer reports use the same caller limits. Plain serde
Deserialize uses `ReplayLimits::default()`. Trusted in-process `new(plan)`
checks plan ceilings but is not an external-input loader.

All six ReplayLimits fields are required canonical decimal usize strings:

| Field | Unit/count rule | Default caller ceiling |
|---|---|---:|
| packets | Observed packets, including unusable/data/reference observations |1024|
| steps | Observations + preparations + progress/policy/Finish events + requests + quality queries |4096|
| answers | Offline requests; stream Data observations + requests; cached answers also capped |1024|
| raw_bytes | Format1/QualifiedFormat1/Format2 payloads + Own/Stamp raw buffers, bytes |4194304 (4MiB)|
| quality_queries | Requested offline measurements; cached quality entries also capped |32|
| quality_work | Checked packets × quality_queries; conservative scan-work budget, pair count |32768|

The packet ceiling also bounds per-query native work; the pair budget bounds
repeated reference/session scans. Arithmetic overflow rejects. A document may
lower any ceiling but cannot grant resources. A caller may explicitly authorize
larger ceilings; no document-supplied default or higher value is authoritative.
Invalid/excessive cached counts reject before replay, then full outputs must
still match regenerated results. JSON/policy-graph bytes MUST be bounded by the
reader before parsing: the wire representation is allocated before preflight,
so these APIs do not claim to prevent arbitrary input allocation themselves.
Cached values permit at most128 nested containers. No OOM-recovery claim.

Offline replays recorded packets then queries/adapters; stream replays
observations, explicit progress, policy changes and final Finish. No host clock,
hidden era, year, epoch, channel, TFMT default, table registry or new
interpolation policy.

Format1 uses its bounded decoder; QualifiedFormat1 additionally retains full
original edition evidence through the native resolver/adapter. Format2 retains
explicit session/edition/header hint/context/rate declaration. Calendar is an
explicit caller calendar, not a raw-wire claim; normalization is explicit.
Prior preparation policies and provided year candidates remain in the basis.
Requests select earlier counter answers by checked index; stream queries select
already released keys. Own/Stamp/Word carry raw extraction, order, caller epochs,
BCD context, matching components, era/table and tolerance/override policy.

On load the canonical answers/findings/quality/offline sessions are regenerated
and compared **in full**; unknown, duplicate, contradictory, changed or dangling
cached fields reject. Arrays preserve observation/request/byte order. Map member
order is irrelevant; policy maps/ranges retain their existing canonical ordering.
An answer index must exist. Live engines are temporary and discarded; session
records are values, not resumable state. The public snapshot values expose only
immutable views. Snapshot/report load does not install indexes/queues/anchors.

Native CalendarReading/ScaleEvidence/CalendarTime/PTP/leap/year-context loads
use checked validators. ScalarSnapshot and RTC/ERTC field adapters enforce
canonical decimal width; decoded Ch4/IEEE header/stamp adapters enforce valid
subseconds. Raw Ch4Words retain all16-bit patterns for diagnosis. Existing native calendar leaves can accept Serde
positional input; versioned roots and cached report maps are objects. Native
optional BCD/evidence members can denote unknown by omission/null; replay DTO
optional fields require explicit null. Raw derived Format1/2Reading and generic
Scaled<T> have no unchecked direct Deserialize; typed value/report replay is
the supported route. Scaled<T> output also wraps nested/shared wide integers.
The six legacy aggregate Serde surfaces remain removed (S12b2a). No other work
is deferred under Q3: only unsupported live complex-state rehydration is omitted.

## Owner review checklist — approve once, then freeze separately

Review root/type/variant names, nullable versus missing rules, all unit/epoch
spellings, tuple ordering, defaults-version separation, exact integer and finite
metric representation, replay resource ceilings, completed-stream scope,
diagnostic-text comparison and migration/version policy together. Recommended:
approve this complete candidate after review, with any requested revisions
applied before a separate explicit freeze. This file records neither approval
nor freeze. No Review/Design fields have been changed.

## Complete emitted value/config field inventory

Types below are structural components, not additional versioned roots. All
field/variant names and positional payload order are listed. Units follow the
exhaustive role table above; nested types inherit their own rows. An em-dash
unit means text/bool/enum/structural reference, not a hidden time unit. Shared
primitive widths/units are defined in the scalar table; Ch4BinaryTime contains
high_order/low_order16-bit words and microseconds, Ieee1588Time contains seconds/
nanoseconds with caller epoch, and Irig106Version/TimeSource/TimeFormat/
IrigTimeSource/SecHdrTimeFormat are shared named/code enums. Raw bytes always
retain the original wire layout. YearOrder emits its active ordered source list.

| Type | Fields / variants (Rust widths identify numeric domains) |
|---|---|
| `AdmissionPolicy` | `include_invalid: bool; include_undeclared: bool; finding_levels: BTreeMap<u8, Severity>` |
| `AnswerBasis` | `Counter( Box<CounterBasis>, ), Own( Box<OwnEnvelope>, ), Stamp( Box<StampEnvelope>, ), Word( Box<WordEnvelope>, ),` |
| `AnswerValue` | `calendar: Option<CalendarReading>; counter_duration: Option<SignedNanos>; counter: Option<ResolvedCounter>; point: Option<OwnPoint>; own_minus_counter: Option<SignedNanos>; tolerance: Option<NanosDuration>; issue: Option<TimeError>; findings: Vec<Finding>; network: Option<NetworkConversion>; chapter4: Option<Ch4Reading>` |
| `AnswerView` | `basis: AnswerBasis; value: AnswerValue` |
| `BcdContext` | `year: Option<u16>; full_day: Option<u16>; interpretation: Option<BcdDayInterpretation>; edition: Option<PacketEdition>` |
| `BcdDayInterpretation` | `LowThreePackedBcdBits,` |
| `BcdDayWording` | `Characters, Bits, EditionNotDeclared,` |
| `BufferCaps` | `packets: usize; bytes: usize; references: usize` |
| `CalendarReading` | `year: Option<u16>; day_of_year: u16; hours: u8; minutes: u8; seconds: u8; nanoseconds: u32; scale: ScaleEvidence` |
| `CalendarTime` | `time: AbsoluteTime; month: u8; day_of_month: u8` |
| `Ch4Context` | `Binary { origin: Option<AbsoluteTime>, }, Bcd( BcdContext, ),` |
| `Ch4Reading` | `raw: Ch4Words; context: Ch4Context; value: Ch4Value; calendar: Option<AbsoluteTime>; missing: MissingCh4Context; wording: Option<BcdDayWording>; reserved: u16` |
| `Ch4Value` | `BinaryElapsed( NanosDuration, ), BcdTimeOfDay { day_fragment: u8, hours: u8, minutes: u8, seconds: u8, nanoseconds: u32, },` |
| `Ch4Words` | `high: u16; low: u16; microseconds: u16` |
| `ChannelChoice` | `Caller, Preference, DeclaredExternal, ValidCountThenId,` |
| `ChannelDeclaration` | `id: u16; cdt: Option<String>; ttf: Option<String>; tfmt: Option<String>; tsrc: Option<String>; shtf: Option<String>` |
| `ClockBasis` | `Offline( Box<OfflineBasis>, ), Stream( Box<StreamBasis>, ),` |
| `ClockRecipe` | `Offline { policy: PolicySnapshot, packets: Vec<RecordedPacket>, quality: Vec<QualityQuery>, }, Stream { policy: PolicySnapshot, inventory: Option<Vec<StreamChannel>>, steps: Vec<StreamStep>, },` |
| `ComponentBinding` | `target: ExtractionOrigin; other: ExtractionOrigin; word: NetworkWord` |
| `CounterBasis` | `clock: ClockBasis; findings: Vec<Finding>` |
| `CounterOrder` | `Nearest, ForwardWithinOneWrap, BackwardWithinOneWrap,` |
| `DeclarationBasis` | `presence: DeclarationPresence; governing: Option<GoverningSetup>; channel: Option<ChannelDeclaration>` |
| `DeclarationPresence` | `Absent, Present, NoGoverningRange,` |
| `Declarations` | `snapshots: Option<Vec<SetupDeclarations>>` |
| `EditionBasis` | `Override, DeclaredAssumedFromTmats, Unresolved,` |
| `EditionEvidence` | `override_edition: Option<PacketEdition>; tmats_edition: Option<String>; use_tmats: bool; setup: SetupVersionEvidence; data_type_version: Option<u8>` |
| `EditionResolution` | `edition: Option<PacketEdition>; basis: EditionBasis; evidence: EditionEvidence; contradiction: bool` |
| `EditionRules` | `layout: TimeF1Layout; chapter: u8; max_data_type_version: u8; format2: bool; source: &'static str` |
| `EpochContext` | `origin: CalendarReading; counting: EpochCounting; source: String` |
| `EpochCounting` | `Continuous, Civil,` |
| `EraBasis` | `Declared, PivotDerived, AssumedZero,` |
| `EraDeclaration` | `era: i64; id: String` |
| `EraIssue` | `InvalidSession, InvalidDeclaration, CrossSession, HalfEraTie, ConflictingPivots, QuantityRange,` |
| `ExtractionOrigin` | `key: PacketKey; channel: u16; byte_offset: Option<u64>; field: String` |
| `Finding` | `code: FindingCode; severity: Severity; evidence: FindingEvidence` |
| `FindingCode` | `(u8, &'static str)` |
| `FindingEvidence` | `location: FindingLocation; related: Vec<FindingLocation>; raw_rtc: Option<Rtc>; raw_fields: Vec<RawField>; validity: ValueValidity; ntp: Option<NtpEvidence>; decode_error: Option<TimeError>` |
| `FindingLocation` | `session: Option<usize>; packet: Option<u64>; channel: Option<u16>; byte_offset: Option<u64>` |
| `Format1Reading` | `csdw: TimeF1Csdw; time: Option<CalendarReading>; reserved_body_bits: [u16; 4]; consumed: usize` |
| `Format2Availability` | `Unavailable, Available, Unknown,` |
| `Format2Observation` | `capture: ReferenceCapture; declared_edition: Option<PacketEdition>; data_type_version: u8; declared_timein: bool; csdw: TimeF2Csdw; raw_body: [u8; 8]; reading: Option<Format2Reading>; decode_error: Option<TimeError>` |
| `Format2Reading` | `csdw: TimeF2Csdw; time: Option<Scaled<NetworkTime>>; consumed: usize` |
| `Format2Reference` | `observation: Format2Observation; conversion: NetworkConversion; admission: NetworkAdmission; policy: AdmissionPolicy; findings: Vec<Finding>` |
| `GoverningRange` | `session: usize; start: SignedRtcTicks; end: Option<SignedRtcTicks>` |
| `GoverningSetup` | `range: GoverningRange; setup_packet: u64; provenance: String; declared_version: Option<String>; ri4: Option<String>; recording_year: Option<u16>` |
| `IntraPacketTime` | `Rtc( Rtc), Ch4( Ch4BinaryTime, ), Ieee1588( Ieee1588Time, ), Ertc( Ertc),` |
| `IntraPacketTimeFormat` | `Rtc48, Ch4Binary, Ieee1588, Ertc64, InvalidFlags(u8),` |
| `ItsQualification` | `Reported( IrigTimeSource, ), NotReported, NotApplicable, FreewheelingOrNotReported, Unresolved( u8, ),` |
| `LeapSecondEntry` | `effective_unix: UnixSeconds; tai_utc_offset: TaiUtcOffset` |
| `LeapSecondTable` | `entries: Vec<LeapSecondEntry>; metadata: LeapTableMetadata` |
| `LeapTableMetadata` | `id: String; source: String; supported_start: UnixSeconds; known_current_through: UnixSeconds` |
| `MissingCh4Context` | `origin: bool; year: bool; full_day: bool; interpretation: bool` |
| `NetworkAdmission` | `eligible: bool; invalid: bool; undeclared: bool` |
| `NetworkConversion` | `Ntp( NtpConversion, ), Ptp( PtpConversion, ), UnsupportedFormat, Malformed,` |
| `NetworkReference` | `reference: Format2Reference; context: NtpContext; table: LeapSecondTable; raw_network_rate: bool; stream_rate_policy: Option<(u64, bool)>; normalization_issue: Option<TimeError>` |
| `NetworkTime` | `Ntp(NtpTime), Ptp(PtpTime),` |
| `NetworkTimeFormat` | `NtpV3, Ptp2002, Ptp2008, Reserved(u8),` |
| `NetworkTimeProtocol` | `Ntp, Ptp, Reserved(u8),` |
| `NetworkWord` | `Ptp( PtpTime, ), PtpSeconds( u32, ), PtpNanoseconds( u32, ), Ntp( NtpTime, ), NtpSeconds( u32, ), NtpFraction( u32, ),` |
| `Normalization` | `tai_ns: i128; native_ns: i128; scale: ScaleEvidence; applied_seconds: i32; possibly_out_of_date: bool; table: LeapSecondTable` |
| `NtpContext` | `session: usize; declared: Option<EraDeclaration>; pivots: Vec<NtpPivot>` |
| `NtpConversion` | `raw: NtpTime; status: NtpConversionStatus; unix: Option<SignedUnixSeconds>; nanos: u32; utc: Option<CalendarReading>; possible_leap: Option<CalendarReading>; calendar_error: Option<TimeError>; basis: NtpConversionBasis` |
| `NtpConversionBasis` | `context: NtpContext; era: Option<i64>; era_basis: Option<EraBasis>; rejected_pivot: Option<usize>; rounding: NtpRounding; quantization: NanosDuration; proleptic_utc: bool; leap: NtpLeapStatus; table: Option<LeapTableMetadata>; final_transition: Option<UnixSeconds>; ambiguity_transition: Option<LeapSecondEntry>` |
| `NtpConversionStatus` | `Mapped, UnknownUnsynchronized, NeedsContext( EraIssue, ),` |
| `NtpEvidence` | `raw: NtpTime; mapped_unix: Option<UnixSeconds>` |
| `NtpLeapStatus` | `NoValue, UnambiguousCovered, Ambiguous, UncertainCoverage,` |
| `NtpRounding` | `FloorNanoseconds,` |
| `NtpTime` | `seconds: u32; fraction: u32` |
| `OfflineAnswer` | `time: Option<AbsoluteTime>; reading: Option<crate::format1::CalendarReading>; counter_ns: i128; basis: OfflineBasis` |
| `OfflineBasis` | `session_origin: PacketKey; key: PacketKey; channel: Option<(u16, ChannelChoice)>; reference: Option<TimelineReference>; distance_ns: Option<i128>; position: ReferencePosition; unresolved: Option<&'static str>; policy: OfflinePolicy; drift_corrected: bool` |
| `OfflinePolicy` | `shared: Option<alloc::boxed::Box<crate::policy::SharedPolicy>>; gap_interval_ns: u64; version: u16; late_ns: u64; infer_sessions: bool; channel: Option<u16>; channel_preference: Vec<u16>; include_undeclared: bool; include_invalid: bool; strategy: ReferenceStrategy; preceding_on_tie: bool; back_extrapolate: bool; stop_on_loss: bool; gap_tolerance_ns: u64; jump_threshold_ns: Option<u64>; finding_levels: BTreeMap<u8, Severity>` |
| `OfflineQuality` | `valid: usize; rejected: usize; admitted: usize; interval_ns: Option<i128>; min_spacing_ns: Option<i128>; max_spacing_ns: Option<i128>; references_per_second: Option<f64>; drift_ppm: Option<f64>; unavailable: Option<&'static str>` |
| `OfflineSession` | `id: usize; first_ticks: i128; high_ticks: i128; references: Vec<TimelineReference>; first_time_packet: Option<u64>; first_packet: u64` |
| `OwnContext` | `ieee: Option<EpochContext>; ch4: Option<EpochContext>; table: LeapSecondTable; counter_applicability: TimeApplicability` |
| `OwnEnvelope` | `counter: CounterBasis; header: SecondaryInspection; context: OwnContext; policy: OwnPolicy` |
| `OwnPoint` | `tai: SignedNanos; native: Option<CalendarReading>; utc: OwnUtc` |
| `OwnPolicy` | `tolerance: Option<NanosDuration>; levels: BTreeMap<u8, Severity>` |
| `OwnUtc` | `reading: Option<CalendarReading>; status: PtpConversionStatus; basis: PtpConversionBasis` |
| `PacketEdition` | `V04, V05, V07, V09, V11, V13, V15, V17, V19, V20, V22, V23, V24, V24R1,` |
| `PacketKey` | `session: usize; ticks: i128; packet: u64` |
| `PacketKind` | `StaticSetup, Time, Data,` |
| `PacketStandard` | `Ch10, Ch11, Unknown,` |
| `PivotFraction` | `Nanoseconds(u32), NtpUnits(u32),` |
| `Preparation` | `policy: PolicySnapshot; key: ReplayKey` |
| `PtpConversion` | `raw: PtpTime; utc: Option<CalendarReading>; unix_seconds: Option<SignedUnixSeconds>; status: PtpConversionStatus; basis: PtpConversionBasis` |
| `PtpConversionBasis` | `table: LeapTableMetadata; final_transition: Option<UnixSeconds>; offset: Option<TaiUtcOffset>; selected_transition: Option<LeapSecondEntry>; leap_second: bool` |
| `PtpConversionStatus` | `Covered, PossiblyOutOfDate, MissingOffset,` |
| `PtpTime` | `seconds: u32; nanoseconds: u32` |
| `QualificationBasis` | `policy: SharedPolicy; declarations: Option<DeclarationBasis>; provided_scale: ScaleEvidence; provided_year: YearContext; provided_normalization: Option<Normalization>; provided_network: Option<Box<NetworkReference>>; provided_policy: Option<SharedPolicy>` |
| `QualifiedCsdw` | `raw: TimeF1Csdw; resolution: crate::edition::EditionResolution` |
| `QualifiedFormat1Reading` | `csdw: crate::csdw::QualifiedCsdw; time: Option<CalendarReading>; reserved_body_bits: [u16; 4]; consumed: usize` |
| `QualityQuery` | `session: Decimal<usize>; channel: u16` |
| `RawField` | `name: &'static str; observed: RawValue; expected: Option<RawValue>` |
| `RawValue` | `Unsigned( u128, ), Signed( i128, ),` |
| `RecordedPacket` | `packet: ReplayPacket; boundary: bool` |
| `RecordingEventType` | `Started, Stopped, Overrun, IndexPoint(u8), Reserved(u8),` |
| `ReferenceCapture` | `session: usize; packet: u64; channel: u16; rtc: Rtc; byte_offset: Option<u64>` |
| `ReferenceInput` | `value: ReferenceValue; scale: ScaleEvidence; source: Option<TimeF1Source>; format: Option<TimeFormat>; its: Option<IrigTimeSource>; precision_ns: u64; wire: Option<(TimeF1Layout, crate::format1::Format1Reading)>; qualified: Option<crate::format1::QualifiedFormat1Reading>; raw_csdw: Option<u32>; year_origin: YearOrigin; year_basis: crate::years::YearBasis; network: Option<alloc::boxed::Box<crate::reference::NetworkReference>>; normalization: Option<crate::reference::Normalization>; qualification: Option<alloc::boxed::Box<crate::policy::QualificationBasis>>` |
| `ReferencePosition` | `CounterOnly, Before, Exact, Between, After,` |
| `ReferenceSpec` | `None, Format1 { payload: Vec<u8>, data_length: Decimal<usize>, layout: TimeF1Layout, scales: ScalePolicy, }, QualifiedFormat1 { payload: Vec<u8>, data_length: Decimal<usize>, evidence: crate::edition::EditionEvidence, scales: ScalePolicy, }, Format2 { payload: Vec<u8>, data_length: Decimal<usize>, session: Decimal<usize>, byte_offset: Option<Decimal<u64>>, edition: Option<crate::edition::PacketEdition>, data_type_version: u8, context: PolicySnapshot, raw_network_rate: bool, }, Calendar { value: CalendarSnapshot, valid: bool, source: Option<TimeF1Source>, format: Option<TimeFormat>, its: Option<IrigTimeSource>, precision_ns: Decimal<u64>, normalize: Option<LeapTableSnapshot>, },` |
| `ReferenceStrategy` | `Nearest, Preceding,` |
| `ReferenceValue` | `Readable( AbsoluteTime, bool, ), Invalid, Malformed, NeedsContext,` |
| `Rejection` | `Undeclared, Invalid, NoneSource, Reserved, Malformed, NeedsContext,` |
| `ReleaseReason` | `Standard, Interpolated, NoReference, Timeout, Eof, PacketCap, ByteCap, ReferenceCap, ModeChange, SessionBoundary, Discontinuity, IncompatibleScale, UnusableReference,` |
| `ReplayBinding` | `target: ReplayOrigin; other: ReplayOrigin; word: NetworkWord` |
| `ReplayKey` | `session: Decimal<usize>; ticks: Decimal<i128>; packet: Decimal<u64>` |
| `ReplayLimits` | `packets: Decimal<usize>; quality_queries: Decimal<usize>; quality_work: Decimal<usize>; steps: Decimal<usize>; answers: Decimal<usize>; raw_bytes: Decimal<usize>` |
| `ReplayOrigin` | `key: ReplayKey; channel: u16; byte_offset: Option<Decimal<u64>>; field: String` |
| `ReplayPacket` | `id: Decimal<u64>; rtc: Decimal<u64>; channel: u16; kind: PacketKind; declared_timein: bool; declared_external: bool; flags: u8; prepare: Vec<Preparation>; year_context: Option<crate::years::YearContext>; body: ReferenceSpec` |
| `ReplayPlan` | `limits: ReplayLimits; clock: ClockRecipe; requests: Vec<ReportRequest>` |
| `ReportRequest` | `Counter { key: ReplayKey, }, Own { counter: Decimal<usize>, origin: ReplayOrigin, data_type: u8, rtc: Decimal<u64>, flags: u8, raw: Vec<u8>, applicability: TimeApplicability, context: PolicySnapshot, }, Stamp { counter: Option<Decimal<usize>>, origin: ReplayOrigin, query_id: Decimal<u64>, flags: u8, raw: Vec<u8>, order: crate::rtc::CounterOrder, context: PolicySnapshot, }, Word { origin: ReplayOrigin, word: TimeWord, context: PolicySnapshot, bcd: BcdContext, bcd_scale: TimeScale, matching: Option<ReplayBinding>, ntp_context: Option<crate::ntp::NtpContext>, },` |
| `ResolvedCounter` | `Offline( Box<OfflineAnswer>, ), Stream( Box<StreamAnswer>, ),` |
| `RtcErtcCheck` | `expected: Rtc; remainder: u8; delta: Result<SignedRtcTicks>; distance: u64` |
| `ScaleEvidence` | `stated: TimeScale; effective: TimeScale; origin: ScaleOrigin` |
| `ScaleOrigin` | `Standard, Policy, Unspecified,` |
| `ScalePolicy` | `irig: TimeScale; rtc: TimeScale` |
| `Scaled` | `value: T; scale: ScaleEvidence` |
| `SecondaryChecksum` | `stored: u16; byte_sum: u16; word_sum: u16` |
| `SecondaryHeaderTime` | `Ch4( Ch4BinaryTime, ), Ieee1588( Ieee1588Time, ), Ertc( Ertc),` |
| `SecondaryInspection` | `data_type: u8; byte_offset: Option<u64>; key: PacketKey; channel: u16; flags: u8; applicability: TimeApplicability; raw: Vec<u8>; format: Option<SecHdrTimeFormat>; checksum: Option<SecondaryChecksum>; time: Option<SecondaryHeaderTime>; consistency: Option<RtcErtcCheck>; errors: Vec<TimeError>; findings: Vec<Finding>` |
| `SecondaryPolicy` | `levels: BTreeMap<u8, Severity>` |
| `SessionRtc` | `session: usize; rtc: Rtc` |
| `SetupDeclarations` | `range: GoverningRange; setup_packet: u64; provenance: String; declared_version: Option<String>; ri4: Option<String>; recording_year: Option<u16>; channels: Vec<ChannelDeclaration>` |
| `SetupVersionEvidence` | `Unavailable, Reserved( u32, ), Rccver( u8, ),` |
| `Severity` | `Information, Warning, Error,` |
| `SharedPolicy` | `id: u64; settings: SharedSettings` |
| `SharedSettings` | `caller_year: Option<u16>; year_order: YearOrder; scales: ScalePolicy; own: OwnContext; own_tolerance: Option<NanosDuration>; declarations: Option<Declarations>; use_tfmt_default: bool; ntp: BTreeMap<usize, NtpContext>` |
| `StampEnvelope` | `inspection: StampInspection; counter: Option<CounterBasis>; context: OwnContext` |
| `StampInspection` | `origin: ExtractionOrigin; flags: u8; query_id: u64; raw: Vec<u8>; order: CounterOrder; time: Option<IntraPacketTime>; query: Option<PacketKey>; remainder: u64; issue: Option<TimeError>; findings: Vec<Finding>; policy: SecondaryPolicy` |
| `StreamAnswer` | `time: Option<AbsoluteTime>; reading: Option<crate::format1::CalendarReading>; counter_ns: i128; basis: StreamBasis` |
| `StreamBasis` | `session_origin: PacketKey; key: PacketKey; channel: Option<(u16, StreamChannelChoice)>; preceding: Option<TimelineReference>; following: Option<TimelineReference>; policy: StreamPolicy; progress_ns: u64; deadline_ns: Option<u64>; extrapolated: bool; release: ReleaseReason; drift_correction_ns: Option<i128>; precision_ns: Option<u64>; rounding: Option<&'static str>; unresolved: Option<&'static str>` |
| `StreamChannel` | `id: u16; timein: bool; external: bool` |
| `StreamChannelChoice` | `Caller, DeclaredExternal, LowestId,` |
| `StreamMode` | `Standard, Interpolate,` |
| `StreamPolicy` | `id: u64; mode: StreamMode; offline: OfflinePolicy; hold_tolerance_ns: u64; hold_interval_ns: u64; raw_network_rate: bool; caps: BufferCaps` |
| `StreamStep` | `Observe { record: RecordedPacket, progress: Decimal<u64>, bytes: Decimal<usize>, }, Advance { progress: Decimal<u64>, }, Policy { policy: PolicySnapshot, progress: Decimal<u64>, }, Finish { progress: Decimal<u64>, },` |
| `TimeApplicability` | `FirstDataBit, CitedException { data_type: u8, citation: String, },` |
| `TimeError` | `InvalidLeapTable { reason: &'static str, }, CounterWidth { value: u64, }, AmbiguousCounterDelta, CrossSessionCounterDelta, QuantityOverflow { quantity: &'static str, }, InvalidBcdDigit { nibble: u8, position: &'static str, word: u8, digit: u8, bit_offset: u8, raw_word: u16, }, ReservedBitSet { position: &'static str, }, OutOfRange { field: &'static str, value: u32, max: u32, }, ChecksumMismatch { stored: u16, computed: u16, }, InvalidPacketFlags { flags: u8, }, ReservedPacketTimeFormat { flags: u8, }, UndefinedSecondaryTimeFormat { flags: u8, }, InvalidTimelineInput( &'static str, ), NeedsContext { context: &'static str, }, InvalidTimeStatus { status: u8, }, NoReferencePoint, BufferTooShort { expected: usize, actual: usize, }, Types( irig106_types::TypesError, ),` |
| `TimeF1Csdw` | `(u32)` |
| `TimeF1Layout` | `Legacy04, Source05, SourceWithIts17,` |
| `TimeF1Source` | `Unresolved(u8), ExternalTime { present: bool, reserved_bits: u8, }, Source(TimeSource),` |
| `TimeF2Csdw` | `(u32)` |
| `TimeScale` | `Utc, Tai, Gps, Unknown,` |
| `TimeStatus` | `NotValid, Valid, Reserved(u8),` |
| `TimeWord` | `Binary( Ch4Words, ), Bcd( Ch4Words, ), Network( NetworkWord, ),` |
| `TimelineFinding` | `code: u8; severity: Severity; session: usize; packet: u64; channel: Option<u16>; related: Option<u64>; raw_rtc: Option<Rtc>; delta_ns: Option<i128>; evidence: FindingEvidence` |
| `TimelineReference` | `key: PacketKey; channel: u16; declared_timein: bool; declared_external: bool; input: ReferenceInput; admitted: bool; reasons: Vec<Rejection>` |
| `ValueValidity` | `Kept, Unusable, Undetermined,` |
| `WordContext` | `own: OwnContext; bcd: BcdContext; bcd_scale: TimeScale; ntp: NtpContext; matching: Option<ComponentBinding>` |
| `WordEnvelope` | `origin: ExtractionOrigin; word: TimeWord; context: WordContext; policy: SecondaryPolicy` |
| `YearBasis` | `context: YearContext; order: crate::policy::YearOrder; packet_year: Option<(u16, u64)>; candidates: Vec<YearCandidate>; original: Option<ReferenceValue>; original_origin: Option<YearOrigin>; phase: i32; observed_leap_ns: i128; resolved: bool; normalization_issue: Option<TimeError>; segment: usize; gap: Option<i128>; year_issue: Option<TimeError>; phase_residual: Option<(u64, i128, u64, bool)>` |
| `YearCandidate` | `supplied: u16; year: u16; origin: YearOrigin; packet: Option<u64>` |
| `YearContext` | `caller: Option<u16>; recording: Option<u16>` |
| `YearOrigin` | `NetworkEvidence, Packet, Caller, RecordingDate, Unknown,` |
| `YearSource` | `Packet, Caller, Recording,` |
| `AbsoluteTime` (custom wire) | `day_of_year:u16; hours:u8; minutes:u8; seconds:u8; nanoseconds:u32; year:Option<u16>` |
| `DayFormatTime` (custom wire) | `milliseconds:u16; seconds:u8; minutes:u8; hours:u8; day_of_year:u16` |
| `DmyFormatTime` (custom wire) | `milliseconds:u16; seconds:u8; minutes:u8; hours:u8; day:u8; month:u8; year:u16` |
| `YearOrder` (custom wire) | `active ordered Vec<YearSource>, max3 distinct; no unused trailing slots` |
| `NtpPivot` (custom wire) | `session:decimal usize; id:String; seconds:decimal SignedUnixSeconds; fraction:PivotFraction; scale:ScaleEvidence; origin:PivotOrigin; calendar:Option<CalendarReading>` |
| `PivotOrigin` (NtpPivot wire) | `CallerApproximateDate; DatedReference{packet:decimal u64}; ResolvedNtp{era:decimal i64,kind:EraBasis}` |

The public literal/caller context is evidence, not authenticated source truth.
Raw invalid inputs can be reported with diagnostics; decoded/absolute values
cannot be loaded past their constructors. A report is reconstructed by native
operations, not by accepting cached decoded structs. QualifiedFormat1 delegates
the current public adapter (including its NeedsContext errors); it does not
invent a supported interpretation for an unresolved format. New fields such as
provided year context are checked before use; old contexts and raw declarations
are retained side by side. All observations are supplied explicitly; a bare
arbitrary external AnswerView without its replay inputs is exportable but is
not a self-contained checked load.

For Word requests, ntp_context=null selects the context policy's entry for the
extraction session. An explicit NtpContext preserves the original raw context,
including a mismatch/zero session, rather than silently replacing it. Raw
context validity and the zero-sentinel exception are evaluated by the native
word converter before any absolute value is produced. This raw input shape is
not a claim of valid independent era evidence; checked policy contexts and
immutable NtpPivot constructors retain their stricter rules.
