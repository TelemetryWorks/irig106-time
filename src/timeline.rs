//! Offline E1 timeline core. Inputs arrive in recording order; no file walking.
//! Ordinary decoded Format 1 times retain scale evidence. Automatic year
//! inference, inverse queries, drift correction and streaming are follow-ups.
extern crate alloc;
use crate::findings::{
    Finding, FindingCode, FindingEvidence, FindingLocation, RawField, RawValue, ValueValidity,
};
use crate::rtc::{CounterOrder, SessionRtc, SignedRtcTicks};
use crate::scale::{ScaleEvidence, ScalePolicy, TimeScale};
use crate::{
    AbsoluteTime, IrigTimeSource, Result, Rtc, TimeError, TimeF1Layout, TimeF1Source, TimeFormat,
    TimeSource,
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};

const MASK: i128 = (1i128 << 48) - 1;
const HALF: i128 = 1i128 << 47;
const DAY: i128 = 86_400_000_000_000;

/// Offline selection; equal-distance nearest ties use preceding (explicit policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceStrategy {
    /// Contract 5.2: minimum absolute counter distance.
    Nearest,
    /// RCC 123-20 §5.6: latest reference not after the query.
    Preceding,
}
/// Why a channel wins contract 5.2's ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelChoice {
    /// Caller-fixed channel; never silently substitute another.
    Caller,
    /// Caller preference list, before standard default ranking.
    Preference,
    /// Governing declarations prefer external TIMEIN.
    DeclaredExternal,
    /// Most valid references, then lowest channel ID.
    ValidCountThenId,
}
/// Complete snapshot of the settings implemented by this offline core.
/// Defaults: contract 5.2/5.4/5.6, ADR-0010; Ch10 §10.6.1 b/c supplies
/// 100 ms generation + 1000 ms commit; §10.6.2 supplies the 1 Hz gap basis.
/// This does not configure the prototype streaming API.
/// **Traces:** L3-POL-101 ← L2-POL-101 ← L1-POL-001
/// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflinePolicy {
    /// Snapshot schema version, currently 1.
    pub version: u16,
    /// Backward displacement allowed without an inferred reset, default 1100 ms.
    pub late_ns: u64,
    /// Infer resets from recording order; false means caller boundaries only.
    pub infer_sessions: bool,
    /// Fixed channel, with no fallback if unavailable.
    pub channel: Option<u16>,
    /// Optional caller preference order; otherwise contract 5.2 ranking.
    pub channel_preference: Vec<u16>,
    /// Admit readable undeclared channels, marking the reference's reason.
    pub include_undeclared: bool,
    /// Admit readable invalid payloads, marking the reference's reason.
    pub include_invalid: bool,
    /// Reference lookup strategy; default nearest.
    pub strategy: ReferenceStrategy,
    /// L3-COR-102 owner-accepted C2 (2026-11-13): preceding by default;
    /// false chooses following. The snapshot exposes this policy choice.
    pub preceding_on_tie: bool,
    /// Allow labelled offline back-extrapolation; default false.
    pub back_extrapolate: bool,
    /// Keep using internal references after external lock is lost by default.
    pub stop_on_loss: bool,
    /// Extra tolerance beyond the 1 s gap interval; default zero.
    pub gap_tolerance_ns: u64,
    /// Explicit jump threshold; None uses the coarser endpoint precision.
    /// The coarser mixed-resolution default is owner-accepted C3 (2026-11-13);
    /// callers can set an explicit threshold.
    pub jump_threshold_ns: Option<u64>,
    /// Optional severity overrides, indexed by stable TF number.
    pub finding_levels: BTreeMap<u8, Severity>,
}
impl Default for OfflinePolicy {
    fn default() -> Self {
        Self {
            version: 1,
            late_ns: 1_100_000_000,
            infer_sessions: true,
            channel: None,
            channel_preference: Vec::new(),
            include_undeclared: false,
            include_invalid: false,
            strategy: ReferenceStrategy::Nearest,
            preceding_on_tie: true,
            back_extrapolate: false,
            stop_on_loss: false,
            gap_tolerance_ns: 0,
            jump_threshold_ns: None,
            finding_levels: BTreeMap::new(),
        }
    }
}
impl OfflinePolicy {
    /// Validate arithmetic bounds and the snapshot version before ingestion.
    /// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || i128::from(self.late_ns) >= HALF * 100
            || self.finding_levels.keys().any(|id| {
                FindingCode::from_number(*id)
                    .ok()
                    .and_then(FindingCode::default_severity)
                    .is_none()
            })
        {
            return Err(TimeError::InvalidTimelineInput(
                "policy version, late bound or finding ID",
            ));
        }
        Ok(())
    }
}
/// Compatibility path for the common stable finding severity vocabulary.
pub use crate::findings::Severity;
/// A stable finding plus packet and related-reference evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineFinding {
    /// TF identifier as its numeric suffix (e.g. 18 is TF-018).
    pub code: u8,
    /// Default or caller-overridden level; does not change admission.
    pub severity: Severity,
    /// Session identity, one based.
    pub session: usize,
    /// Packet where this finding is observed.
    pub packet: u64,
    /// Affected channel if applicable.
    pub channel: Option<u16>,
    /// Previous/reference packet if applicable.
    pub related: Option<u64>,
    /// Raw packet counter, including TF-024 when no timestamp is readable.
    /// **Traces:** L3-FND-103 ← L2-FND-103 ← L1-FND-003
    pub raw_rtc: Option<Rtc>,
    /// Signed counter displacement or jump residual, in ns.
    pub delta_ns: Option<i128>,
    /// Reusable factual evidence; legacy fields above remain compatibility views.
    /// **Traces:** L3-FND-101 ← L2-FND-101 ← L1-FND-001
    pub evidence: FindingEvidence,
}
impl TimelineFinding {
    pub(crate) fn observed(
        code: u8,
        level: Option<Severity>,
        key: PacketKey,
        channel: Option<u16>,
        related: Option<u64>,
        delta_ns: Option<i128>,
    ) -> Result<Self> {
        let location = FindingLocation {
            session: Some(key.session),
            packet: Some(key.packet),
            channel,
            byte_offset: None,
        };
        // Only the malformed-input emission establishes unusability here.
        // Other findings alone do not decide admission (e.g. reserved FMT vs SRC).
        let mut evidence = FindingEvidence::new(
            location,
            if code == 4 {
                ValueValidity::Unusable
            } else {
                ValueValidity::Undetermined
            },
        );
        evidence = evidence.with_raw_rtc(Rtc::from_raw((key.ticks & MASK) as u64));
        if let Some(packet) = related {
            evidence = evidence.with_related(FindingLocation {
                packet: Some(packet),
                ..location
            });
        }
        if let Some(delta) = delta_ns {
            evidence = evidence.with_raw_field(RawField {
                name: "delta_ns",
                observed: RawValue::Signed(delta),
                expected: None,
            });
        }
        let finding = Finding::new(FindingCode::from_number(code)?, level, evidence)?;
        let severity = finding.severity();
        let evidence = finding.into_evidence();
        Ok(Self {
            code,
            severity,
            session: key.session,
            packet: key.packet,
            channel,
            related,
            raw_rtc: evidence.raw_rtc(),
            delta_ns,
            evidence,
        })
    }
}
/// Caller labels; static setup packets may precede the first time packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketKind {
    /// Static Computer-Generated Format 1.
    StaticSetup,
    /// A time packet, even when its time is invalid/unreadable/None.
    Time,
    /// Other dynamic data.
    Data,
}
/// Year provenance supplied by a reviewed decoder/caller; no inference here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearOrigin {
    /// DMY packet provides year.
    Packet,
    /// Caller explicitly supplied year.
    Caller,
    /// Recording date RI4; never G\OD.
    RecordingDate,
    /// Year not established.
    Unknown,
}
/// Bounded decoder result, separate from reference admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceValue {
    /// An ordinary calendar value and its payload-valid status.
    Readable(AbsoluteTime, bool),
    /// Payload is explicitly invalid; no readable value.
    Invalid,
    /// Malformed data cannot be admitted under any policy.
    Malformed,
    /// A leap/calendar conversion needs context the core cannot supply.
    NeedsContext,
}
/// Decoder-supplied evidence; public construction is an explicit caller boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceInput {
    /// Decoded value or retained failure classification.
    pub value: ReferenceValue,
    /// Scale and stated/assumed provenance; never relabel GPS as UTC.
    pub scale: ScaleEvidence,
    /// Edition-qualified source, including legacy EXT without inventing SRC.
    pub source: TimeF1Source,
    /// Format 1 format, retained even when invalid/reserved.
    pub format: TimeFormat,
    /// Applicable IRIG time-source evidence.
    pub its: Option<IrigTimeSource>,
    /// Actual format resolution, in ns (not arithmetic output resolution).
    pub precision_ns: u64,
    /// Original bounded wire reading and explicit layout, including reserved
    /// bits and leap labels which cannot be projected into this core calendar.
    pub wire: Option<(TimeF1Layout, crate::format1::Format1Reading)>,
    /// S03 qualified reading; retains unresolved alternatives through answer basis.
    pub qualified: Option<crate::format1::QualifiedFormat1Reading>,
    /// Original channel-specific word where available.
    pub raw_csdw: Option<u32>,
    /// Basis of a resolved year; unknown remains unknown.
    pub year_origin: YearOrigin,
}
impl ReferenceInput {
    /// Adapt an evidence-qualified reading without inventing SRC or ITS.
    /// Unresolved FMT15 cannot become a modern None label: retain the original
    /// reading and request context instead. Ordinary body values remain usable.
    /// **Traces:** L3-EDN-201 ← L2-EDN-201 ← L1-EDN-002
    pub fn qualified_format1(reading: crate::format1::QualifiedFormat1Reading) -> Result<Self> {
        let format = reading.csdw.format().ok_or(TimeError::NeedsContext {
            context: "unresolved Format1 FMT15 meaning",
        })?;
        let source = reading.csdw.source().unwrap_or(TimeF1Source::Unresolved(
            (reading.csdw.raw().as_raw() & 15) as u8,
        ));
        let scale = reading.time.map_or(ScaleEvidence::unknown(), |t| t.scale);
        let its = match reading.csdw.its() {
            crate::csdw::ItsQualification::Reported(value) => Some(value),
            _ => None,
        };
        let (value, year_origin) = match reading.time {
            Some(t) if !t.is_leap_second() => (
                ReferenceValue::Readable(
                    AbsoluteTime::new(t.day_of_year, t.hours, t.minutes, t.seconds, t.nanoseconds)?
                        .with_year(t.year)?,
                    true,
                ),
                if t.year.is_some() {
                    YearOrigin::Packet
                } else {
                    YearOrigin::Unknown
                },
            ),
            Some(_) => (ReferenceValue::NeedsContext, YearOrigin::Unknown),
            None => (
                if matches!(format, TimeFormat::Reserved(_)) {
                    ReferenceValue::NeedsContext
                } else {
                    ReferenceValue::Invalid
                },
                YearOrigin::Unknown,
            ),
        };
        Ok(Self {
            value,
            scale,
            source,
            format,
            its,
            precision_ns: 10_000_000,
            raw_csdw: Some(reading.csdw.raw().as_raw()),
            year_origin,
            wire: None,
            qualified: Some(reading),
        })
    }
    /// Decode only within caller Data Length, with explicitly selected layout.
    /// Leap labels are retained as NeedsContext, not normalized by this core.
    /// **Traces:** L3-COR-101 ← L2-COR-101 ← L1-COR-001
    pub fn format1(
        payload: &[u8],
        data_length: usize,
        layout: TimeF1Layout,
        scales: ScalePolicy,
    ) -> Result<Self> {
        let reading = crate::format1::decode_time_f1(payload, data_length, layout, scales)?;
        let format = reading.csdw.time_format_for_layout(layout);
        let scale = scales.for_format1(format);
        let (value, year_origin) = match reading.time {
            Some(t) if !t.is_leap_second() => (
                ReferenceValue::Readable(
                    AbsoluteTime::new(t.day_of_year, t.hours, t.minutes, t.seconds, t.nanoseconds)?
                        .with_year(t.year)?,
                    true,
                ),
                if t.year.is_some() {
                    YearOrigin::Packet
                } else {
                    YearOrigin::Unknown
                },
            ),
            Some(_) => (ReferenceValue::NeedsContext, YearOrigin::Unknown),
            None => (
                if matches!(format, TimeFormat::Reserved(_)) {
                    ReferenceValue::NeedsContext
                } else {
                    ReferenceValue::Invalid
                },
                YearOrigin::Unknown,
            ),
        };
        Ok(Self {
            value,
            scale,
            source: reading.csdw.time_source_for_layout(layout),
            format,
            its: reading.csdw.irig_time_source_for_layout(layout),
            precision_ns: 10_000_000,
            wire: Some((layout, reading)),
            qualified: None,
            raw_csdw: Some(reading.csdw.as_raw()),
            year_origin,
        })
    }
}
/// Packet identity and governing declaration facts; ingestion is in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketObservation {
    /// Unique caller identity, e.g. file byte offset.
    pub id: u64,
    /// Channel ID.
    pub channel: u16,
    /// Header counter.
    pub rtc: Rtc,
    /// Static/time/data classification.
    pub kind: PacketKind,
    /// Governing setup declares this channel TIMEIN.
    pub declared_timein: bool,
    /// Governing setup declares TSRC=E; separate from dynamic source.
    pub declared_external: bool,
    /// Header flags for independent RTC sync evidence.
    pub flags: u8,
    /// Bounded decoder result for time packets; None means malformed if Time.
    pub reference: Option<ReferenceInput>,
}
/// Explicit query identity, disambiguating overlapping raw RTC sessions/wraps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketKey {
    /// Session, one based.
    pub session: usize,
    /// Unwrapped counter ticks within that session's coordinate system.
    pub ticks: i128,
    /// Caller packet identity.
    pub packet: u64,
}
impl PacketKey {
    /// Checked signed unwrapped ticks within the same named session.
    /// No modular guessing occurs after unwrapping.
    /// **Traces:** L3-RTC-103 ← L2-RTC-103 ← L1-RTC-003
    /// **Traces:** L3-RTC-104 ← L2-RTC-104 ← L1-RTC-004
    pub fn delta_to(self, later: Self) -> Result<SignedRtcTicks> {
        crate::rtc::same_session(self.session, later.session)?;
        later
            .ticks
            .checked_sub(self.ticks)
            .map(SignedRtcTicks::new)
            .ok_or(TimeError::QuantityOverflow {
                quantity: "unwrapped RTC subtraction",
            })
    }
}
/// Reference admission reasons remain visible even when policy admits a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// No governing TIMEIN declaration.
    Undeclared,
    /// Readable or unreadable invalid payload.
    Invalid,
    /// Source None never produces a reference. E1 classifies this unusable
    /// reference as TF-003 warning; it is not an assertion that SRC None
    /// equals FMT NONE. See verification-e1.md for the policy/source basis.
    NoneSource,
    /// Unsupported reserved format; reserved SRC is only a TF-006 warning.
    Reserved,
    /// Malformed/unreadable value or zero precision.
    Malformed,
    /// Explicit conversion context is absent.
    NeedsContext,
}
/// A retained eligible or rejected reference with packet/declaration evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineReference {
    /// Counter and packet identity.
    pub key: PacketKey,
    /// Channel of this observation.
    pub channel: u16,
    /// Governing declaration at this packet.
    pub declared_timein: bool,
    /// Static external declaration at this packet.
    pub declared_external: bool,
    /// Original decoded value and metadata.
    pub input: ReferenceInput,
    /// Whether this reference is admitted by this policy.
    pub admitted: bool,
    /// Rejection/override reasons; malformed data is never admitted.
    pub reasons: Vec<Rejection>,
}
/// Immutable session boundaries and retained reference evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineSession {
    /// Session identity.
    pub id: usize,
    /// First packet's unwrapped counter.
    pub first_ticks: i128,
    /// High-water counter, never lowered by late packets.
    pub high_ticks: i128,
    /// All time references, including rejected ones, in file order.
    pub references: Vec<TimelineReference>,
    /// First time-packet identity, independent of eligibility.
    pub first_time_packet: Option<u64>,
    first_packet: u64,
}
/// Build once from recording-order packets; explicit boundaries take priority.
/// **Traces:** L3-COR-101 ← L2-COR-101 ← L1-COR-001
pub struct TimelineBuilder {
    policy: OfflinePolicy,
    sessions: Vec<OfflineSession>,
    findings: Vec<TimelineFinding>,
    seen: BTreeSet<u64>,
}
impl TimelineBuilder {
    /// Freeze a validated policy for the entire builder.
    pub fn new(policy: OfflinePolicy) -> Result<Self> {
        policy.validate()?;
        Ok(Self {
            policy,
            sessions: Vec::new(),
            findings: Vec::new(),
            seen: BTreeSet::new(),
        })
    }
    /// Observe one packet; returns its explicit session/unwrapped query key.
    /// Context-free sequence arithmetic: short forward wrap is not a reset.
    /// **Traces:** L3-COR-105 ← L2-COR-105 ← L1-COR-005
    /// **Traces:** L3-COR-106 ← L2-COR-106 ← L1-COR-006
    /// **Traces:** L3-COR-107 ← L2-COR-107 ← L1-COR-007
    /// **Traces:** L3-COR-109 ← L2-COR-109 ← L1-COR-009
    /// **Traces:** L3-COR-112 ← L2-COR-112 ← L1-COR-012
    pub fn observe(&mut self, packet: PacketObservation, boundary: bool) -> Result<PacketKey> {
        if self.seen.contains(&packet.id) {
            return Err(TimeError::InvalidTimelineInput("duplicate packet identity"));
        }
        if packet.kind != PacketKind::Time && packet.reference.is_some() {
            return Err(TimeError::InvalidTimelineInput(
                "reference on non-time packet",
            ));
        }
        let raw = i128::from(packet.rtc.as_raw());
        let mut ticks = raw;
        let mut late = None;
        let mut reset = false;
        if let Some(s) = self.sessions.last().filter(|_| !boundary) {
            let delta = SessionRtc {
                session: s.id,
                rtc: Rtc::from_raw((s.high_ticks & MASK) as u64),
            }
            .delta_to(
                SessionRtc {
                    session: s.id,
                    rtc: packet.rtc,
                },
                CounterOrder::Nearest,
            )?
            .get();
            ticks = s
                .high_ticks
                .checked_add(delta)
                .ok_or(TimeError::QuantityOverflow {
                    quantity: "unwrapped RTC ticks",
                })?;
            if delta < 0 && -delta * 100 > i128::from(self.policy.late_ns) {
                late = Some(delta * 100);
                reset = self.policy.infer_sessions;
            }
        }
        self.seen.insert(packet.id);
        if self.sessions.is_empty() || boundary || reset {
            ticks = raw;
            self.sessions.push(OfflineSession {
                id: self.sessions.len() + 1,
                first_ticks: ticks,
                high_ticks: ticks,
                references: Vec::new(),
                first_time_packet: None,
                first_packet: packet.id,
            });
        }
        let index = self.sessions.len() - 1;
        let key = PacketKey {
            session: index + 1,
            ticks,
            packet: packet.id,
        };
        if !boundary {
            if let Some(delta) = late {
                self.finding(19, key, Some(packet.channel), None, Some(delta))?;
                if reset {
                    self.finding(18, key, Some(packet.channel), None, Some(delta))?;
                }
            }
        }
        if packet.flags & 0x20 != 0 {
            self.finding(24, key, Some(packet.channel), None, None)?;
        }
        if packet.kind == PacketKind::Data && self.sessions[index].first_time_packet.is_none() {
            self.finding(2, key, Some(packet.channel), None, None)?;
        }
        self.sessions[index].high_ticks = self.sessions[index].high_ticks.max(ticks);
        if packet.kind == PacketKind::Time {
            self.sessions[index]
                .first_time_packet
                .get_or_insert(packet.id);
            if let Some(input) = packet.reference {
                if let Some((layout, wire)) = input.wire {
                    if wire.csdw.validate_reserved(layout).is_err()
                        || wire.reserved_body_bits.iter().any(|bits| *bits != 0)
                    {
                        self.finding(5, key, Some(packet.channel), None, None)?;
                        if let Some(finding) = self.findings.last_mut() {
                            finding.evidence = body_reserved_evidence(
                                finding.evidence.clone(),
                                wire.reserved_body_bits,
                            );
                        }
                    }
                    if wire.time.is_some_and(|t| t.is_leap_second()) {
                        self.finding(22, key, Some(packet.channel), None, None)?;
                    }
                }
                if let Some(q) = &input.qualified {
                    if q.csdw.resolution().contradiction() {
                        self.finding(31, key, Some(packet.channel), None, None)?;
                        if let Some(f) = self.findings.last_mut() {
                            if let Some(raw) = q.csdw.resolution().evidence().data_type_version {
                                f.evidence = f.evidence.clone().with_raw_field(RawField {
                                    name: "data_type_version",
                                    observed: RawValue::Unsigned(raw.into()),
                                    expected: None,
                                });
                            }
                        }
                    }
                    if q.csdw.reserved_bits() != 0 || q.reserved_body_bits.iter().any(|b| *b != 0) {
                        self.finding(5, key, Some(packet.channel), None, None)?;
                        if let Some(f) = self.findings.last_mut() {
                            f.evidence =
                                body_reserved_evidence(f.evidence.clone(), q.reserved_body_bits)
                                    .with_raw_field(RawField {
                                        name: "csdw.reserved",
                                        observed: RawValue::Unsigned(q.csdw.reserved_bits().into()),
                                        expected: Some(RawValue::Unsigned(0)),
                                    });
                        }
                    }
                    if matches!(
                        q.csdw.its(),
                        crate::csdw::ItsQualification::Reported(IrigTimeSource::Reserved(_))
                    ) {
                        self.finding(6, key, Some(packet.channel), None, None)?;
                    }
                    if q.time.is_some_and(|t| t.is_leap_second()) {
                        self.finding(22, key, Some(packet.channel), None, None)?;
                    }
                }
                let mut reasons = Vec::new();
                if !packet.declared_timein {
                    reasons.push(Rejection::Undeclared);
                    self.finding(8, key, Some(packet.channel), None, None)?;
                }
                if matches!(input.source, TimeF1Source::Source(TimeSource::None)) {
                    reasons.push(Rejection::NoneSource);
                }
                if matches!(input.source, TimeF1Source::Source(TimeSource::Reserved(_))) {
                    // SRC is a reserved VALUE, not an unreadable payload.
                    // TF-006 warning keeps the admitted time and raw source.
                    self.finding(6, key, Some(packet.channel), None, None)?;
                }
                if matches!(input.format, TimeFormat::Reserved(_)) {
                    reasons.push(Rejection::Reserved);
                }
                match input.value {
                    ReferenceValue::Readable(_, false) | ReferenceValue::Invalid => {
                        reasons.push(Rejection::Invalid)
                    }
                    ReferenceValue::Malformed => reasons.push(Rejection::Malformed),
                    ReferenceValue::NeedsContext => reasons.push(Rejection::NeedsContext),
                    _ => (),
                }
                let invalid_calendar = matches!(input.value, ReferenceValue::Readable(t, _) if t.year().is_some_and(|y| i128::from(t.day_of_year()) > year_days(y)));
                if matches!(input.format, TimeFormat::None)
                    && !reasons.contains(&Rejection::Invalid)
                {
                    reasons.push(Rejection::Invalid);
                }
                if input.precision_ns == 0 || invalid_calendar {
                    reasons.push(Rejection::Malformed);
                }
                for reason in &reasons {
                    let code = match reason {
                        // §11.2.3.2 requires a time packet even for source None:
                        // count its presence, but owner Q2 forbids using its time.
                        // TF-003 is E1's unusable-reference warning classification;
                        // do not call it malformed or claim no time packet existed.
                        Rejection::Invalid | Rejection::NoneSource => Some(3),
                        Rejection::Reserved => Some(6),
                        Rejection::Malformed => Some(4),
                        _ => None,
                    };
                    if let Some(code) = code {
                        self.finding(code, key, Some(packet.channel), None, None)?;
                    }
                }
                let admitted = matches!(input.value, ReferenceValue::Readable(..))
                    && !matches!(input.format, TimeFormat::None | TimeFormat::Reserved(_))
                    && reasons.iter().all(|r| match r {
                        Rejection::Undeclared => self.policy.include_undeclared,
                        Rejection::Invalid => self.policy.include_invalid,
                        _ => false,
                    });
                if let Some(prev) = self.sessions[index]
                    .references
                    .iter()
                    .rev()
                    .find(|r| r.channel == packet.channel)
                {
                    if prev.input.source != input.source {
                        self.finding(11, key, Some(packet.channel), Some(prev.key.packet), None)?;
                    }
                }
                self.sessions[index].references.push(TimelineReference {
                    key,
                    channel: packet.channel,
                    declared_timein: packet.declared_timein,
                    declared_external: packet.declared_external,
                    input,
                    admitted,
                    reasons,
                });
            } else {
                self.finding(4, key, Some(packet.channel), None, None)?;
            }
        }
        Ok(key)
    }
    fn finding(
        &mut self,
        code: u8,
        key: PacketKey,
        channel: Option<u16>,
        related: Option<u64>,
        delta_ns: Option<i128>,
    ) -> Result<()> {
        self.findings.push(TimelineFinding::observed(
            code,
            self.policy.finding_levels.get(&code).copied(),
            key,
            channel,
            related,
            delta_ns,
        )?);
        Ok(())
    }
    /// Ingest a bounded Format1 packet, preserving a parse error in TF-004.
    /// Decoder failure affects this value, never an earlier eligible reference.
    /// **Traces:** L3-ERR-102 ← L2-ERR-102 ← L1-ERR-002
    pub fn observe_format1(
        &mut self,
        packet: PacketObservation,
        input: crate::format1::Format1Input<'_>,
        boundary: bool,
    ) -> Result<PacketKey> {
        let id = packet.id;
        let (packet, error) = decode_observation(packet, input)?;
        let key = self.observe(packet, boundary)?;
        if let Some(error) = error {
            retain_decode_error(
                &mut self.findings,
                id,
                error,
                input.reserved_body_bits().ok(),
                self.policy.finding_levels.get(&5).copied(),
            )?;
        }
        Ok(key)
    }
    /// Finalize immutable sessions and per-channel sorted reference indexes.
    /// Gap/jump checks use only valid admitted same-scale references.
    /// **Traces:** L3-COR-104 ← L2-COR-104 ← L1-COR-004
    /// **Traces:** L3-COR-108 ← L2-COR-108 ← L1-COR-008
    pub fn finish(mut self) -> Result<OfflineTimeline> {
        let mut indexes = BTreeMap::<(usize, u16), Vec<usize>>::new();
        let mut events = Vec::new();
        for s in &self.sessions {
            if s.first_time_packet.is_none() {
                events.push((
                    1,
                    PacketKey {
                        session: s.id,
                        ticks: s.first_ticks,
                        packet: s.first_packet,
                    },
                    None,
                    None,
                    None,
                ));
            }
            for (i, r) in s.references.iter().enumerate().filter(|(_, r)| r.admitted) {
                indexes.entry((s.id, r.channel)).or_default().push(i);
            }
        }
        for (&(session, channel), ids) in &mut indexes {
            let refs = &self.sessions[session - 1].references;
            ids.sort_by_key(|&i| (refs[i].key.ticks, refs[i].key.packet));
            let valid: Vec<_> = ids
                .iter()
                .map(|&i| &refs[i])
                .filter(|r| matches!(r.input.value, ReferenceValue::Readable(_, true)))
                .collect();
            for pair in valid.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let elapsed = (b.key.ticks - a.key.ticks) * 100;
                if elapsed > 1_000_000_000 + i128::from(self.policy.gap_tolerance_ns) {
                    events.push((16, b.key, Some(channel), Some(a.key.packet), Some(elapsed)));
                }
                if let Some(delta) = absolute_delta(a, b) {
                    let residual = delta - elapsed;
                    let threshold = self
                        .policy
                        .jump_threshold_ns
                        .unwrap_or(a.input.precision_ns.max(b.input.precision_ns));
                    if residual.abs() > i128::from(threshold) {
                        events.push((17, b.key, Some(channel), Some(a.key.packet), Some(residual)));
                    }
                }
            }
        }
        for (code, key, ch, related, delta) in events {
            self.finding(code, key, ch, related, delta)?;
        }
        Ok(OfflineTimeline {
            policy: self.policy,
            sessions: self.sessions,
            findings: self.findings,
            indexes,
        })
    }
}
/// Position relative to admitted same-channel references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferencePosition {
    /// No admitted/selected reference, with a separately recorded reason.
    CounterOnly,
    /// Explicitly enabled offline back-extrapolation.
    Before,
    /// Exact counter match.
    Exact,
    /// Within the reference range.
    Between,
    /// Beyond the final reference.
    After,
}
/// Reproducible offline answer basis; references retain all decoder metadata.
/// **Traces:** L3-ANS-101 ← L2-ANS-101 ← L1-ANS-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineBasis {
    /// Session/counter/packet query identity.
    pub key: PacketKey,
    /// Chosen time channel and ranking rule.
    pub channel: Option<(u16, ChannelChoice)>,
    /// Contributing reference, including override/rejection evidence.
    pub reference: Option<TimelineReference>,
    /// Signed query-minus-reference counter distance, in ns.
    pub distance_ns: Option<i128>,
    /// Bracket/extrapolation position.
    pub position: ReferencePosition,
    /// Why absolute time is absent, if so.
    pub unresolved: Option<&'static str>,
    /// Full immutable implemented-policy snapshot.
    pub policy: OfflinePolicy,
    /// This core never silently applies drift correction.
    pub drift_corrected: bool,
}
/// An absolute-or-counter-only answer; year/scale come from its reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineAnswer {
    /// Ordinary calendar reading in the reference's scale, never silently UTC.
    pub time: Option<AbsoluteTime>,
    /// Counter offset from this session's first packet, in ns.
    pub counter_ns: i128,
    /// Source/format/ITS/scale/year/precision/reference and policy evidence.
    pub basis: OfflineBasis,
}
/// Immutable offline timeline. Query keys always name a session explicitly.
pub struct OfflineTimeline {
    policy: OfflinePolicy,
    sessions: Vec<OfflineSession>,
    findings: Vec<TimelineFinding>,
    indexes: BTreeMap<(usize, u16), Vec<usize>>,
}
impl OfflineTimeline {
    /// Immutable session evidence, in file order.
    pub fn sessions(&self) -> &[OfflineSession] {
        &self.sessions
    }
    /// Stable finding IDs/default levels plus recorded evidence.
    pub fn findings(&self) -> &[TimelineFinding] {
        &self.findings
    }
    /// Policy snapshot used for all results.
    pub fn policy(&self) -> &OfflinePolicy {
        &self.policy
    }
    fn selected(&self, s: &OfflineSession) -> Option<(u16, ChannelChoice)> {
        if let Some(ch) = self.policy.channel {
            return Some((ch, ChannelChoice::Caller));
        }
        for &ch in &self.policy.channel_preference {
            if self.indexes.contains_key(&(s.id, ch)) {
                return Some((ch, ChannelChoice::Preference));
            }
        }
        let mut candidates = BTreeMap::<u16, (bool, usize)>::new();
        for r in s.references.iter().filter(|r| r.admitted) {
            let e = candidates.entry(r.channel).or_default();
            e.0 |= r.declared_timein && r.declared_external;
            e.1 += usize::from(matches!(r.input.value, ReferenceValue::Readable(_, true)));
        }
        candidates
            .into_iter()
            .min_by_key(|&(ch, (external, count))| (!external, core::cmp::Reverse(count), ch))
            .map(|(ch, (external, _))| {
                (
                    ch,
                    if external {
                        ChannelChoice::DeclaredExternal
                    } else {
                        ChannelChoice::ValidCountThenId
                    },
                )
            })
    }
    /// Nearest/preceding within one chosen channel; no cross-session fallback.
    /// **Traces:** L3-COR-102 ← L2-COR-102 ← L1-COR-002
    /// **Traces:** L3-COR-103 ← L2-COR-103 ← L1-COR-003
    /// **Traces:** L3-COR-106 ← L2-COR-106 ← L1-COR-006
    /// **Traces:** L3-ANS-105 ← L2-ANS-105 ← L1-ANS-005
    pub fn lookup(&self, key: PacketKey) -> Result<OfflineAnswer> {
        let s = self
            .sessions
            .iter()
            .find(|s| s.id == key.session)
            .ok_or(TimeError::InvalidTimelineInput("unknown session"))?;
        let mut answer = OfflineAnswer {
            time: None,
            counter_ns: distance_ns(key.ticks, s.first_ticks)?,
            basis: OfflineBasis {
                key,
                channel: self.selected(s),
                reference: None,
                distance_ns: None,
                position: ReferencePosition::CounterOnly,
                unresolved: Some("no admitted reference on selected channel"),
                policy: self.policy.clone(),
                drift_corrected: false,
            },
        };
        let Some((ch, _)) = answer.basis.channel else {
            return Ok(answer);
        };
        let Some(ids) = self.indexes.get(&(s.id, ch)) else {
            return Ok(answer);
        };
        let refs = &s.references;
        let split = ids.partition_point(|&i| refs[i].key.ticks <= key.ticks);
        if split == 0 && !self.policy.back_extrapolate {
            answer.basis.unresolved = Some("before first admitted reference");
            return Ok(answer);
        }
        let selected = match (split, self.policy.strategy) {
            (0, _) => ids[0],
            (n, _) if n == ids.len() => ids[n - 1],
            (n, ReferenceStrategy::Preceding) => ids[n - 1],
            (n, ReferenceStrategy::Nearest) => {
                let before = distance_ns(key.ticks, refs[ids[n - 1]].key.ticks)?;
                let after = distance_ns(refs[ids[n]].key.ticks, key.ticks)?;
                if before < after || (before == after && self.policy.preceding_on_tie) {
                    ids[n - 1]
                } else {
                    ids[n]
                }
            }
        };
        let r = &refs[selected];
        if self.policy.stop_on_loss && lost_lock_at(refs, ch, key.ticks) {
            answer.basis.unresolved = Some("stop-on-loss policy");
            return Ok(answer);
        }
        let delta = distance_ns(key.ticks, r.key.ticks)?;
        answer.basis.reference = Some(r.clone());
        answer.basis.distance_ns = Some(delta);
        answer.basis.position = if delta == 0 {
            ReferencePosition::Exact
        } else if split == 0 {
            ReferencePosition::Before
        } else if split == ids.len() {
            ReferencePosition::After
        } else {
            ReferencePosition::Between
        };
        let ReferenceValue::Readable(time, _) = r.input.value else {
            return Ok(answer);
        };
        match shift(time, delta) {
            Ok(t) => {
                answer.time = Some(t);
                answer.basis.unresolved = None;
            }
            Err(_) => answer.basis.unresolved = Some("calendar/year boundary needs context"),
        }
        Ok(answer)
    }
    /// Same-session, same-channel valid-reference quality; unavailable reasons
    /// are explicit. Drift is 1e6 * (absolute_delta / RTC_duration - 1).
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn quality(&self, session: usize, channel: u16) -> Result<OfflineQuality> {
        let s = self
            .sessions
            .iter()
            .find(|s| s.id == session)
            .ok_or(TimeError::InvalidTimelineInput("unknown session"))?;
        let all: Vec<_> = s
            .references
            .iter()
            .filter(|r| r.channel == channel)
            .collect();
        let mut valid: Vec<_> = all
            .iter()
            .copied()
            .filter(|r| r.admitted && matches!(r.input.value, ReferenceValue::Readable(_, true)))
            .collect();
        valid.sort_by_key(|r| (r.key.ticks, r.key.packet));
        let mut q = OfflineQuality {
            valid: valid.len(),
            rejected: all.iter().filter(|r| !r.admitted).count()
                + self
                    .findings
                    .iter()
                    .filter(|f| {
                        f.session == session
                            && f.channel == Some(channel)
                            && f.code == 4
                            && !all.iter().any(|r| r.key.packet == f.packet)
                    })
                    .count(),
            admitted: all.iter().filter(|r| r.admitted).count(),
            interval_ns: None,
            min_spacing_ns: None,
            max_spacing_ns: None,
            references_per_second: None,
            drift_ppm: None,
            unavailable: Some("fewer than two valid references"),
        };
        if valid.len() < 2 {
            return Ok(q);
        }
        let span = (valid[valid.len() - 1].key.ticks - valid[0].key.ticks) * 100;
        q.interval_ns = Some(span);
        if valid.windows(2).any(|p| p[0].key.ticks == p[1].key.ticks) {
            q.unavailable = Some("duplicate RTC");
            return Ok(q);
        }
        q.min_spacing_ns = valid
            .windows(2)
            .map(|p| (p[1].key.ticks - p[0].key.ticks) * 100)
            .min();
        q.max_spacing_ns = valid
            .windows(2)
            .map(|p| (p[1].key.ticks - p[0].key.ticks) * 100)
            .max();
        q.references_per_second = Some(valid.len() as f64 / (span as f64 / 1e9));
        if valid.iter().any(|r| {
            r.input.scale.effective() == TimeScale::Unknown
                || r.input.scale.effective() != valid[0].input.scale.effective()
        }) {
            q.unavailable = Some("unknown or incompatible scales");
            return Ok(q);
        }
        let mut sum = 0.;
        for p in valid.windows(2) {
            let Some(abs) = absolute_delta(p[0], p[1]) else {
                q.unavailable = Some("calendar/year context");
                return Ok(q);
            };
            sum += 1e6 * (abs as f64 / ((p[1].key.ticks - p[0].key.ticks) * 100) as f64 - 1.);
        }
        q.drift_ppm = Some(sum / (valid.len() - 1) as f64);
        q.unavailable = None;
        Ok(q)
    }
}
/// Per-channel quality with the measured interval and explicit missing context.
#[derive(Debug, Clone, PartialEq)]
pub struct OfflineQuality {
    /// Valid admitted reference count.
    pub valid: usize,
    /// Rejected reference count.
    pub rejected: usize,
    /// Admitted count, including explicitly admitted invalid references.
    pub admitted: usize,
    /// Measurement interval in counter ns.
    pub interval_ns: Option<i128>,
    /// Minimum same-channel spacing.
    pub min_spacing_ns: Option<i128>,
    /// Maximum same-channel spacing.
    pub max_spacing_ns: Option<i128>,
    /// Valid count divided by measured interval in seconds.
    pub references_per_second: Option<f64>,
    /// Mean adjacent-pair absolute-clock drift, ppm.
    pub drift_ppm: Option<f64>,
    /// Reason a requested measure (particularly drift) is unavailable.
    pub unavailable: Option<&'static str>,
}
fn lost_lock_at(refs: &[TimelineReference], channel: u16, ticks: i128) -> bool {
    let current = refs
        .iter()
        .filter(|r| r.channel == channel && r.key.ticks <= ticks)
        .max_by_key(|r| (r.key.ticks, r.key.packet));
    let Some(current) = current else { return false };
    matches!(
        current.input.source,
        TimeF1Source::Source(TimeSource::Internal)
    ) && refs.iter().any(|r| {
        r.channel == channel
            && r.key.ticks < current.key.ticks
            && matches!(r.input.source, TimeF1Source::Source(TimeSource::External))
    })
}
fn year_days(year: u16) -> i128 {
    if crate::util::is_leap_year(year) {
        366
    } else {
        365
    }
}
pub(crate) fn absolute_delta(a: &TimelineReference, b: &TimelineReference) -> Option<i128> {
    if a.input.scale.effective() == TimeScale::Unknown
        || a.input.scale.effective() != b.input.scale.effective()
    {
        return None;
    }
    let (ReferenceValue::Readable(a, _), ReferenceValue::Readable(b, _)) =
        (a.input.value, b.input.value)
    else {
        return None;
    };
    match (a.year(), b.year()) {
        (Some(y0), Some(y1)) => {
            let days = |y: u16| -> i128 { (0..y).map(year_days).sum() };
            Some(
                (days(y1) - days(y0)) * DAY + i128::from(b.as_total_ns())
                    - i128::from(a.as_total_ns()),
            )
        }
        (None, None) if b.day_of_year() >= a.day_of_year() => {
            Some(i128::from(b.as_total_ns()) - i128::from(a.as_total_ns()))
        }
        _ => None,
    }
}
pub(crate) fn shift(time: AbsoluteTime, delta: i128) -> Result<AbsoluteTime> {
    time.checked_shift(crate::SignedNanos::new(delta))
}

fn distance_ns(a: i128, b: i128) -> Result<i128> {
    a.checked_sub(b)
        .and_then(|n| n.checked_mul(100))
        .ok_or(TimeError::InvalidTimelineInput(
            "counter arithmetic overflow",
        ))
}

// Both engines share decoder-error retention rather than inventing source/format
// labels when a data word/body is missing or malformed.
pub(crate) fn decode_observation(
    mut packet: PacketObservation,
    input: crate::format1::Format1Input<'_>,
) -> Result<(PacketObservation, Option<TimeError>)> {
    if packet.kind != PacketKind::Time || packet.reference.is_some() {
        return Err(TimeError::InvalidTimelineInput(
            "Format1 ingestion needs an undecoded time packet",
        ));
    }
    match ReferenceInput::format1(input.payload, input.data_length, input.layout, input.scales) {
        Ok(reference) => {
            packet.reference = Some(reference);
            Ok((packet, None))
        }
        Err(error) => Ok((packet, Some(error))),
    }
}
pub(crate) fn retain_decode_error(
    findings: &mut Vec<TimelineFinding>,
    id: u64,
    error: TimeError,
    masks: Option<[u16; 4]>,
    level: Option<Severity>,
) -> Result<()> {
    let f = findings
        .iter_mut()
        .rev()
        .find(|f| f.packet == id && f.code == 4)
        .ok_or(TimeError::InvalidTimelineInput(
            "missing malformed-input finding",
        ))?;
    f.evidence = f.evidence.clone().with_decode_error(error);
    if let Some(bits) = masks.filter(|bits| bits.iter().any(|b| *b != 0)) {
        // A supplementary finding stores only raw RTC, not an unwrapped query.
        let raw = f.raw_rtc.ok_or(TimeError::InvalidTimelineInput(
            "missing raw decoder counter",
        ))?;
        let mut warning = TimelineFinding::observed(
            5,
            level,
            PacketKey {
                session: f.session,
                packet: f.packet,
                ticks: i128::from(raw.as_raw()),
            },
            f.channel,
            None,
            None,
        )?;
        warning.evidence = body_reserved_evidence(warning.evidence, bits);
        findings.push(warning);
    }
    Ok(())
}
fn body_reserved_evidence(mut evidence: FindingEvidence, bits: [u16; 4]) -> FindingEvidence {
    let names = [
        "format1.word0.reserved",
        "format1.word1.reserved",
        "format1.word2.reserved",
        "format1.word3.reserved",
    ];
    for (word, mask) in bits.into_iter().enumerate() {
        if mask != 0 {
            evidence = evidence.with_raw_field(RawField {
                name: names[word],
                observed: RawValue::Unsigned(mask.into()),
                expected: Some(RawValue::Unsigned(0)),
            });
        }
    }
    evidence
}

// Bounded one-packet admission adapter shared by E1/E2. Never accumulates
// recording history; stream owns session/source/presence and gap state.
pub(crate) fn stream_observation(
    packet: PacketObservation,
    policy: &OfflinePolicy,
    key: PacketKey,
) -> Result<(Option<TimelineReference>, Vec<TimelineFinding>)> {
    let mut b = TimelineBuilder::new(policy.clone())?;
    b.observe(packet, false)?;
    let mut reference = b.sessions.pop().and_then(|mut s| s.references.pop());
    if let Some(r) = &mut reference {
        r.key = key;
    }
    b.findings.retain(|f| f.code != 2);
    for f in &mut b.findings {
        f.session = key.session;
        f.packet = key.packet;
    }
    Ok((reference, b.findings))
}
