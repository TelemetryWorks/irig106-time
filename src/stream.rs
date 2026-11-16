//! E2 stream core. No host clock, file walking, retained output or future-count
//! ranking. Format1/2 share offline admission and explicit TAI normalization.
//! Legacy `streaming` remains a separate compatibility API.
use crate::timeline::*;
use crate::{AbsoluteTime, Result, Rtc, TimeError, TimeF1Source, TimeSource};
use alloc::{collections::VecDeque, vec::Vec};
const MASK: i128 = (1i128 << 48) - 1;
/// Explicit stream strategy; Standard is the owner-accepted default.
/// **Traces:** L3-POL-201 ← L2-POL-201 ← L1-POL-001
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMode {
    /// RCC 123-20 §5.6 printed 5-44: preceding time plus RTC displacement.
    Standard,
    /// Drift-corrected bracket, bounded look-ahead; floor accepted C1 (2026-11-13).
    Interpolate,
}
/// Mandatory caller limits. Bytes account declared retained packet sizes;
/// packet/reference caps bound retained snapshots. Caller-supplied table/pivot
/// sizes and declared byte charges determine the size of each snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferCaps {
    /// Maximum pending data observations; must be positive.
    pub packets: usize,
    /// Maximum sum of pending declared byte sizes; must be positive.
    pub bytes: usize,
    /// Maximum retained committed-channel references; at least two.
    pub references: usize,
}
/// Immutable versioned stream policy copied into every released answer.
/// **Traces:** L3-POL-101 ← L2-POL-101 ← L1-POL-001
/// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
/// **Traces:** L3-POL-201 ← L2-POL-201 ← L1-POL-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamPolicy {
    /// Caller-assigned identity; nonzero, strictly increasing on replacement.
    pub id: u64,
    /// Accepted default Standard; Interpolate must be selected explicitly.
    pub mode: StreamMode,
    /// Shared admission/session/source-loss/gap/jump/finding settings.
    /// Offline strategy, preference and back-extrapolation must stay at defaults:
    /// stream commitment/preceding rules supersede offline lookup ranking.
    pub offline: OfflinePolicy,
    /// Added to the 1 s interval; default 100 ms, independently of gap/late.
    pub hold_tolerance_ns: u64,
    /// Look-ahead base interval, independent of reference gap/late; default1s.
    pub hold_interval_ns: u64,
    /// Declare raw NTP/PTP frame-rate recording for network references observed
    /// under this snapshot (§11.2.3.3). Per-reference declarations also apply.
    /// Both endpoints must qualify; Format1 is never exempt. Changing this flag
    /// does not retroactively reclassify references from an older policy.
    pub raw_network_rate: bool,
    /// Caller resource bounds, never implicit unbounded defaults.
    pub caps: BufferCaps,
}
impl StreamPolicy {
    /// ADR-0010 defaults: Standard (RCC123-20 §5.6), 1 s interval (106-24R1
    /// Ch10 §10.6.2), 100 ms hold tolerance (§10.6.1 b policy basis), zero gap
    /// tolerance. §10.6.1 c explains late commit; it does not guarantee successful look-ahead.
    pub fn new(id: u64, caps: BufferCaps) -> Self {
        Self {
            id,
            mode: StreamMode::Standard,
            offline: OfflinePolicy::default(),
            hold_tolerance_ns: 100_000_000,
            hold_interval_ns: 1_000_000_000,
            raw_network_rate: false,
            caps,
        }
    }
    /// Validate before ingestion or policy replacement; no state changes on error.
    pub fn validate(&self) -> Result<()> {
        self.offline.validate()?;
        if self.id == 0
            || self.caps.packets == 0
            || self.caps.bytes == 0
            || self.caps.references < 2
            || self
                .hold_tolerance_ns
                .checked_add(self.hold_interval_ns)
                .is_none()
            || self
                .offline
                .shared
                .as_ref()
                .is_some_and(|p| p.id() != self.id)
            || !self.offline.channel_preference.is_empty()
            || self.offline.back_extrapolate
            || self.offline.strategy != ReferenceStrategy::Nearest
        {
            return Err(TimeError::InvalidTimelineInput("stream policy/caps"));
        }
        Ok(())
    }
}
/// Complete advance channel inventory supplied by the caller; never inferred
/// from arrival counts. `None` inventory means selection is undecidable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamChannel {
    /// Channel identifier.
    pub id: u16,
    /// Declared R-x\CDT-n=TIMEIN.
    pub timein: bool,
    /// Declared R-x\TSRC-n=E.
    pub external: bool,
}
/// Option A commitment reason; no valid-count choice exists here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamChannelChoice {
    /// Explicit caller choice, without substitution.
    Caller,
    /// External TIMEIN, lowest ID tie.
    DeclaredExternal,
    /// Complete inventory's lowest eligible ID.
    LowestId,
}
/// Why an answer is released. All non-interpolated fallbacks carry this reason
/// and the old policy/progress/deadline; none silently claims drift correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseReason {
    /// Immediate preceding/reference-exact answer, no drift correction.
    Standard,
    /// Compatible bracket with owner-accepted floor rounding (C1, 2026-11-13).
    Interpolated,
    /// No usable preceding reference or undecidable channel.
    NoReference,
    /// Caller progress is strictly beyond the inclusive deadline.
    Timeout,
    /// Explicit end of input.
    Eof,
    /// Pending observation cap would be exceeded.
    PacketCap,
    /// Declared pending byte cap would be exceeded, including oversized input.
    ByteCap,
    /// Retained reference cap would be exceeded.
    ReferenceCap,
    /// Accepted C4 (2026-11-13): drain old policy before installing replacement.
    ModeChange,
    /// Explicit boundary or inferred counter reset.
    SessionBoundary,
    /// Detected clock jump: never interpolate across it.
    Discontinuity,
    /// Scale/year context does not establish a compatible bracket.
    IncompatibleScale,
    /// Opt-in stop-on-loss barrier, or an explicitly included invalid time
    /// used without interpolation. Rejected references do not cause a barrier.
    UnusableReference,
}
/// Stream-specific reproducible basis. Endpoint metadata retain source, scale,
/// declaration/overrides, precision and raw bounded decoder evidence.
/// **Traces:** L3-ANS-101 ← L2-ANS-101 ← L1-ANS-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamBasis {
    /// Query identity in unwrapped session coordinates.
    pub key: PacketKey,
    /// Immutable channel commitment and selection rule.
    pub channel: Option<(u16, StreamChannelChoice)>,
    /// Preceding reference, including admission exceptions.
    pub preceding: Option<TimelineReference>,
    /// Interpolation endpoint only; never present on fallback.
    pub following: Option<TimelineReference>,
    /// Full policy snapshot, with caller identity.
    pub policy: StreamPolicy,
    /// Monotonic caller progress at release, in ns; never a host-clock value.
    pub progress_ns: u64,
    /// Inclusive enqueue-progress + 1 s + tolerance deadline, if held.
    pub deadline_ns: Option<u64>,
    /// A fallback kept a preceding value rather than interpolating.
    pub extrapolated: bool,
    /// Explicit reason, including every resource/lifecycle fallback.
    pub release: ReleaseReason,
    /// Interpolated displacement minus uncorrected RTC displacement.
    pub drift_correction_ns: Option<i128>,
    /// Coarser endpoint resolution; arithmetic does not improve accuracy.
    pub precision_ns: Option<u64>,
    /// Owner-accepted rounding retained explicitly on interpolated answers.
    /// The legacy string label is unchanged until a separate behaviour migration.
    pub rounding: Option<&'static str>,
    /// Missing calendar/reference/selection context when counter-only.
    pub unresolved: Option<&'static str>,
}
/// Absolute-or-counter-only stream answer, never revised after release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamAnswer {
    /// Compatibility calendar; None for a qualified UTC second60.
    pub time: Option<AbsoluteTime>,
    /// Native scale-qualified calendar, preserving UTC leap labels.
    pub reading: Option<crate::format1::CalendarReading>,
    /// Counter displacement from this session's first observation, ns.
    pub counter_ns: i128,
    /// Reference, mode, policy, progress and fallback evidence.
    pub basis: StreamBasis,
}
/// Incremental return value; outputs/findings are caller-owned, not queued.
#[derive(Debug, Default)]
pub struct StreamBatch {
    /// Newly released answers only.
    pub answers: Vec<StreamAnswer>,
    /// Newly observed findings only.
    pub findings: Vec<TimelineFinding>,
}
/// Current bounded retained storage; output batches are not retained.
#[derive(Debug, Clone, Copy)]
pub struct BufferUsage {
    /// Held observations.
    pub packets: usize,
    /// Sum of caller-declared held byte sizes.
    pub bytes: usize,
    /// Committed-channel neighbours.
    pub references: usize,
}
struct Pending {
    key: PacketKey,
    counter_ns: i128,
    bytes: usize,
    deadline: Option<u64>,
    preceding: Option<TimelineReference>,
}
/// Bounded stream state with one committed channel per session. Caller packet
/// identities must strictly increase (e.g. recording offsets); this detects
/// duplicates without an unbounded seen-ID set. Explicit progress is independent
/// of RTC order; short late packets never move the high-water counter backward.
/// **Traces:** L3-STRM-101 ← L2-STRM-101 ← L1-STRM-001
/// **Traces:** L3-COR-103 ← L2-COR-103 ← L1-COR-003
/// **Traces:** L3-COR-107 ← L2-COR-107 ← L1-COR-007
pub struct StreamTimeline {
    policy: StreamPolicy,
    channel: Option<(u16, StreamChannelChoice)>,
    refs: Vec<TimelineReference>,
    // References older than this closed clock-segment boundary cannot reopen it.
    segment_start: Option<i128>,
    pending: VecDeque<Pending>,
    bytes: usize,
    progress: u64,
    last_id: Option<u64>,
    first: Option<PacketKey>,
    high: i128,
    session: usize,
    time_seen: bool,
    source: Option<TimeF1Source>,
    external_seen: bool,
    finished: bool,
    years: crate::years::YearClock,
}
impl StreamTimeline {
    /// Commit from caller choice or complete inventory, before any data.
    pub fn new(policy: StreamPolicy, inventory: Option<&[StreamChannel]>) -> Result<Self> {
        policy.validate()?;
        let channel = policy
            .offline
            .channel
            .map(|ch| (ch, StreamChannelChoice::Caller))
            .or_else(|| {
                let candidate = inventory?
                    .iter()
                    .filter(|c| c.timein)
                    .min_by_key(|c| (!c.external, c.id))?;
                Some((
                    candidate.id,
                    if candidate.external {
                        StreamChannelChoice::DeclaredExternal
                    } else {
                        StreamChannelChoice::LowestId
                    },
                ))
            });
        Ok(Self {
            policy,
            channel,
            refs: Vec::new(),
            segment_start: None,
            pending: VecDeque::new(),
            bytes: 0,
            progress: 0,
            last_id: None,
            first: None,
            high: 0,
            session: 0,
            time_seen: false,
            source: None,
            external_seen: false,
            finished: false,
            years: crate::years::YearClock::default(),
        })
    }
    /// Frozen effective policy. Replacement requires a fresh caller ID.
    pub fn policy(&self) -> &StreamPolicy {
        &self.policy
    }
    /// Committed channel; None stays counter-only until explicit caller choice.
    pub fn channel(&self) -> Option<(u16, StreamChannelChoice)> {
        self.channel
    }
    /// Actual pending/reference usage, excluding returned caller-owned outputs.
    pub fn usage(&self) -> BufferUsage {
        BufferUsage {
            packets: self.pending.len(),
            bytes: self.bytes,
            references: self.refs.len(),
        }
    }
    fn check_progress(&self, progress: u64) -> Result<()> {
        if self.finished
            || progress < self.progress
            || progress
                .checked_add(self.policy.hold_interval_ns)
                .and_then(|p| p.checked_add(self.policy.hold_tolerance_ns))
                .is_none()
        {
            return Err(TimeError::InvalidTimelineInput(
                "closed stream or invalid caller progress",
            ));
        }
        Ok(())
    }
    fn emit(
        &self,
        b: &mut StreamBatch,
        code: u8,
        key: PacketKey,
        channel: Option<u16>,
        related: Option<u64>,
        delta: Option<i128>,
    ) -> Result<()> {
        b.findings.push(TimelineFinding::observed(
            code,
            self.policy.offline.finding_levels.get(&code).copied(),
            key,
            channel,
            related,
            delta,
        )?);
        Ok(())
    }
    /// **Traces:** L3-STRM-202 ← L2-STRM-202 ← L1-STRM-001
    /// **Traces:** L3-F2COR-102 ← L2-F2COR-102 ← L1-F2COR-002
    fn answer(
        &self,
        p: Pending,
        reason: ReleaseReason,
        next: Option<TimelineReference>,
    ) -> StreamAnswer {
        let mut reading = None;
        let mut correction = None;
        let mut precision = p.preceding.as_ref().map(|r| r.input.precision_ns);
        let mut unresolved = Some("no usable preceding reference or undecidable committed channel");
        if let Some(r) = &p.preceding {
            if let ReferenceValue::Readable(..) = r.input.value {
                let offset = (p.key.ticks - r.key.ticks) * 100;
                let displacement = if let Some(n) = &next {
                    precision = Some(r.input.precision_ns.max(n.input.precision_ns));
                    let interpolated = crate::timeline::absolute_delta(r, n)
                        .and_then(|d| (p.key.ticks - r.key.ticks).checked_mul(d))
                        .and_then(|d| d.checked_div_euclid(n.key.ticks - r.key.ticks));
                    correction = interpolated.and_then(|d| d.checked_sub(offset));
                    interpolated
                } else {
                    Some(offset)
                };
                reading = displacement.and_then(|d| crate::reference::shifted_reference(r, d).ok());
                unresolved = if reading.is_some() {
                    None
                } else {
                    Some("calendar/arithmetic context unavailable")
                };
            }
        }
        let time = reading.and_then(|r| crate::reference::legacy_calendar(r).ok());
        StreamAnswer {
            time,
            reading,
            counter_ns: p.counter_ns,
            basis: StreamBasis {
                key: p.key,
                channel: self.channel,
                preceding: p.preceding,
                following: next,
                policy: self.policy.clone(),
                progress_ns: self.progress,
                deadline_ns: p.deadline,
                extrapolated: reading.is_some()
                    && !matches!(
                        reason,
                        ReleaseReason::Standard | ReleaseReason::Interpolated
                    ),
                release: reason,
                drift_correction_ns: correction,
                precision_ns: precision,
                rounding: if reason == ReleaseReason::Interpolated {
                    Some("floor (L3 proposal)")
                } else {
                    None
                },
                unresolved,
            },
        }
    }
    fn flush(&mut self, b: &mut StreamBatch, reason: ReleaseReason) {
        while let Some(p) = self.pending.pop_front() {
            self.bytes -= p.bytes;
            b.answers.push(self.answer(p, reason, None));
        }
    }
    /// Caller progress alone drives timeouts; an exact deadline remains held.
    /// **Traces:** L3-STRM-203 ← L2-STRM-203 ← L1-STRM-001
    pub fn advance(&mut self, progress: u64) -> Result<StreamBatch> {
        self.check_progress(progress)?;
        self.progress = progress;
        let mut b = StreamBatch::default();
        while self
            .pending
            .front()
            .is_some_and(|p| p.deadline.is_some_and(|d| progress > d))
        {
            let p = self
                .pending
                .pop_front()
                .ok_or(TimeError::InvalidTimelineInput("pending queue"))?;
            self.bytes -= p.bytes;
            b.answers.push(self.answer(p, ReleaseReason::Timeout, None));
        }
        Ok(b)
    }
    /// Observe metadata/decoded input with explicit monotonic progress and byte
    /// charge. No data payload is retained; decoded time/context snapshots are.
    /// New sessions drain old pending answers.
    /// **Traces:** L3-STRM-201 ← L2-STRM-201 ← L1-STRM-001
    /// **Traces:** L3-STRM-202 ← L2-STRM-202 ← L1-STRM-001
    /// **Traces:** L3-STRM-204 ← L2-STRM-204 ← L1-STRM-001
    /// **Traces:** L3-STRM-205 ← L2-STRM-205 ← L1-STRM-001
    /// **Traces:** L3-ANS-105 ← L2-ANS-105 ← L1-ANS-005
    pub fn observe(
        &mut self,
        mut packet: PacketObservation,
        progress: u64,
        bytes: usize,
        boundary: bool,
    ) -> Result<StreamBatch> {
        self.check_progress(progress)?;
        crate::reference::check_capture(&packet)?;
        if bytes == 0
            || self.last_id.is_some_and(|id| packet.id <= id)
            || (packet.kind != PacketKind::Time && packet.reference.is_some())
        {
            return Err(TimeError::InvalidTimelineInput(
                "byte charge, identity order or non-time reference",
            ));
        }
        let raw = i128::from(packet.rtc.as_raw());
        let delta = if self.first.is_some() && !boundary {
            crate::rtc::SessionRtc {
                session: self.session,
                rtc: Rtc::from_raw((self.high & MASK) as u64),
            }
            .delta_to(
                crate::rtc::SessionRtc {
                    session: self.session,
                    rtc: packet.rtc,
                },
                crate::rtc::CounterOrder::Nearest,
            )?
            .get()
        } else {
            0
        };
        let late = self.first.is_some()
            && delta < 0
            && -delta * 100 > i128::from(self.policy.offline.late_ns);
        let reset = late && self.policy.offline.infer_sessions;
        let ticks = if self.first.is_none() || boundary || reset {
            raw
        } else {
            self.high
                .checked_add(delta)
                .ok_or(TimeError::InvalidTimelineInput("counter overflow"))?
        };
        let session = if self.first.is_none() || boundary || reset {
            self.session
                .checked_add(1)
                .ok_or(TimeError::QuantityOverflow {
                    quantity: "stream session identity",
                })?
        } else {
            self.session
        };
        if let Some(policy) = &self.policy.offline.shared {
            let key = PacketKey {
                session,
                ticks,
                packet: packet.id,
            };
            packet = policy.prepare(packet, key)?;
        }
        if let Some(n) = packet.reference.as_ref().and_then(|r| r.network.as_ref()) {
            if n.reference().observation().capture().session != session {
                return Err(TimeError::InvalidTimelineInput(
                    "network capture session differs from stream",
                ));
            }
        }
        // Reject ambiguous year gaps/explanations before progress/timeouts or packet
        // identity are committed. Preserve held answers and allow an explicit-boundary retry.
        if self.first.is_some()
            && !boundary
            && !reset
            && packet
                .reference
                .as_ref()
                .is_some_and(|r| r.network.is_none())
        {
            let key = PacketKey {
                session,
                ticks,
                packet: packet.id,
            };
            let (reference, _) =
                crate::timeline::stream_observation(packet.clone(), &self.policy.offline, key)?;
            if let Some(mut r) = reference {
                self.years.clone().stamp(&mut r, &self.policy.offline)?;
            }
        }
        let mut b = self.advance(progress)?;
        if self.first.is_none() || boundary || reset {
            self.flush(&mut b, ReleaseReason::SessionBoundary);
            self.end_session(&mut b)?;
            self.session = session;
            self.high = ticks;
            self.time_seen = false;
            self.segment_start = None;
            self.years = crate::years::YearClock::default();
            self.refs.clear();
            self.source = None;
            self.external_seen = false;
            self.first = Some(PacketKey {
                session: self.session,
                ticks,
                packet: packet.id,
            });
        }
        let key = PacketKey {
            session: self.session,
            ticks,
            packet: packet.id,
        };
        if late && !boundary {
            self.emit(
                &mut b,
                19,
                key,
                Some(packet.channel),
                None,
                Some(delta * 100),
            )?;
            if reset {
                self.emit(
                    &mut b,
                    18,
                    key,
                    Some(packet.channel),
                    None,
                    Some(delta * 100),
                )?;
            }
        }
        self.high = self.high.max(ticks);
        self.last_id = Some(packet.id);
        let kind = packet.kind;
        let ch = packet.channel;
        let mut packet = packet;
        if let Some(n) = packet.reference.as_mut().and_then(|r| r.network.as_mut()) {
            n.record_rate_policy(self.policy.id, self.policy.raw_network_rate);
        }
        let (mut reference, findings) =
            crate::timeline::stream_observation(packet, &self.policy.offline, key)?;
        b.findings.extend(findings);
        if let Some(r) = reference.as_mut() {
            self.years.stamp(r, &self.policy.offline)?;
            b.findings
                .extend(self.years.resolve(r, &self.policy.offline)?);
            for prior in &mut self.refs {
                b.findings
                    .extend(self.years.resolve(prior, &self.policy.offline)?);
            }
            self.refs
                .retain(|r| r.admitted && matches!(r.input.value, ReferenceValue::Readable(..)));
            for pending in &mut self.pending {
                if let Some(prior) = pending.preceding.as_mut() {
                    for f in self.years.resolve(prior, &self.policy.offline)? {
                        if !b.findings.contains(&f) {
                            b.findings.push(f);
                        }
                    }
                }
                if pending.preceding.as_ref().is_some_and(|r| {
                    !r.admitted || !matches!(r.input.value, ReferenceValue::Readable(..))
                }) {
                    pending.preceding = self
                        .refs
                        .iter()
                        .filter(|r| r.key.ticks <= pending.key.ticks)
                        .max_by_key(|r| {
                            (
                                self.policy.mode == StreamMode::Interpolate
                                    && matches!(r.input.value, ReferenceValue::Readable(_, true)),
                                r.key.ticks,
                                r.key.packet,
                            )
                        })
                        .cloned();
                }
            }
        }
        if kind == PacketKind::Time {
            self.time_seen = true;
            if self.channel.is_some_and(|(id, _)| id == ch) {
                self.add_reference(reference, key, &mut b)?;
            }
        } else if kind == PacketKind::Data {
            if !self.time_seen {
                self.emit(&mut b, 2, key, Some(ch), None, None)?;
            }
            let preceding = self
                .refs
                .iter()
                .filter(|r| r.key.ticks <= ticks)
                .max_by_key(|r| {
                    (
                        self.policy.mode == StreamMode::Interpolate
                            && matches!(r.input.value, ReferenceValue::Readable(_, true)),
                        r.key.ticks,
                        r.key.packet,
                    )
                })
                .cloned();
            let first = self
                .first
                .ok_or(TimeError::InvalidTimelineInput("session origin"))?;
            let mut p = Pending {
                key,
                counter_ns: (ticks - first.ticks) * 100,
                bytes,
                deadline: None,
                preceding,
            };
            if p.preceding.is_none() {
                b.answers
                    .push(self.answer(p, ReleaseReason::NoReference, None));
            } else if self.policy.mode == StreamMode::Standard
                || p.preceding.as_ref().is_some_and(|r| r.key.ticks == ticks)
            {
                b.answers
                    .push(self.answer(p, ReleaseReason::Standard, None));
            } else if p
                .preceding
                .as_ref()
                .is_some_and(|r| matches!(r.input.value, ReferenceValue::Readable(_, false)))
            {
                b.answers
                    .push(self.answer(p, ReleaseReason::UnusableReference, None));
            } else {
                p.deadline =
                    Some(progress + self.policy.hold_interval_ns + self.policy.hold_tolerance_ns);
                if self.pending.len() == self.policy.caps.packets {
                    self.flush(&mut b, ReleaseReason::PacketCap);
                }
                if bytes > self.policy.caps.bytes || self.bytes > self.policy.caps.bytes - bytes {
                    self.flush(&mut b, ReleaseReason::ByteCap);
                }
                if bytes > self.policy.caps.bytes {
                    b.answers.push(self.answer(p, ReleaseReason::ByteCap, None));
                } else {
                    self.bytes += bytes;
                    self.pending.push_back(p);
                    self.resolve(&mut b);
                }
            }
        }
        Ok(b)
    }
    /// **Traces:** L3-STRM-201 ← L2-STRM-201 ← L1-STRM-001
    /// **Traces:** L3-STRM-204 ← L2-STRM-204 ← L1-STRM-001
    /// **Traces:** L3-STRM-205 ← L2-STRM-205 ← L1-STRM-001
    /// **Traces:** L3-COR-104 ← L2-COR-104 ← L1-COR-004
    /// **Traces:** L3-COR-108 ← L2-COR-108 ← L1-COR-008
    fn add_reference(
        &mut self,
        reference: Option<TimelineReference>,
        key: PacketKey,
        b: &mut StreamBatch,
    ) -> Result<()> {
        // Missing/unreadable input already has its finding; it does not
        // invalidate an earlier eligible reference or drain held queries.
        let Some(r) = reference else { return Ok(()) };
        if self.segment_start.is_some_and(|start| r.key.ticks < start) {
            return Ok(());
        }
        if self.source.is_some() && r.input.source.is_some() && self.source != r.input.source {
            self.emit(
                b,
                11,
                key,
                Some(r.channel),
                self.refs.last().map(|r| r.key.packet),
                None,
            )?;
        }
        if r.input.source.is_some() {
            self.source = r.input.source;
        }
        self.external_seen |= matches!(
            r.input.source,
            Some(TimeF1Source::Source(TimeSource::External))
        );
        let stopped = self.policy.offline.stop_on_loss
            && self.external_seen
            && matches!(
                r.input.source,
                Some(TimeF1Source::Source(TimeSource::Internal))
            );
        if stopped {
            self.flush(b, ReleaseReason::UnusableReference);
            self.refs.clear();
            return Ok(());
        }
        if !r.admitted || !matches!(r.input.value, ReferenceValue::Readable(..)) {
            return Ok(());
        }
        let mut barrier = None;
        // Compare both immediate neighbours for late reference insertion.
        let left = self
            .refs
            .iter()
            .filter(|p| {
                p.key.ticks <= r.key.ticks
                    && matches!(p.input.value, ReferenceValue::Readable(_, true))
            })
            .max_by_key(|p| p.key.ticks);
        let right = self
            .refs
            .iter()
            .filter(|p| {
                p.key.ticks > r.key.ticks
                    && matches!(p.input.value, ReferenceValue::Readable(_, true))
            })
            .min_by_key(|p| p.key.ticks);
        for (a, z) in left
            .map(|a| (a, &r))
            .into_iter()
            .chain(right.map(|z| (&r, z)))
            .filter(|(a, z)| {
                matches!(a.input.value, ReferenceValue::Readable(_, true))
                    && matches!(z.input.value, ReferenceValue::Readable(_, true))
            })
        {
            let elapsed = (z.key.ticks - a.key.ticks) * 100;
            let related = Some(if a.key.packet == r.key.packet {
                z.key.packet
            } else {
                a.key.packet
            });
            if elapsed
                > i128::from(self.policy.offline.gap_interval_ns)
                    + i128::from(self.policy.offline.gap_tolerance_ns)
            {
                self.emit(b, 16, key, Some(r.channel), related, Some(elapsed))?;
                if let Some(f) = b.findings.last_mut() {
                    crate::reference::qualify_gap(f, a, z);
                }
            }
            match crate::timeline::absolute_delta(a, z) {
                Some(d) => {
                    let residual = d - elapsed;
                    let threshold = self
                        .policy
                        .offline
                        .jump_threshold_ns
                        .unwrap_or(a.input.precision_ns.max(z.input.precision_ns));
                    if residual.abs() > i128::from(threshold) {
                        self.emit(b, 17, key, Some(r.channel), related, Some(residual))?;
                        barrier = Some((ReleaseReason::Discontinuity, z.key.ticks));
                    }
                }
                None => barrier = Some((ReleaseReason::IncompatibleScale, z.key.ticks)),
            }
        }
        if let Some((reason, cut)) = barrier {
            self.flush(b, reason);
            let start = self.segment_start.map_or(cut, |prior| prior.max(cut));
            self.segment_start = Some(start);
            // With a late incoming endpoint, keep the already-known later side.
            self.refs
                .retain(|p| p.key.ticks >= start && p.key.ticks > r.key.ticks);
            if r.key.ticks < start {
                return Ok(());
            }
        }
        if self.refs.len() == self.policy.caps.references {
            self.flush(b, ReleaseReason::ReferenceCap);
            self.refs.remove(0);
        }
        self.refs.push(r);
        self.refs.sort_by_key(|r| (r.key.ticks, r.key.packet));
        self.resolve(b);
        Ok(())
    }
    fn resolve(&mut self, b: &mut StreamBatch) {
        let mut remaining = VecDeque::new();
        while let Some(mut p) = self.pending.pop_front() {
            let a = self
                .refs
                .iter()
                .filter(|r| {
                    r.key.ticks <= p.key.ticks
                        && matches!(r.input.value, ReferenceValue::Readable(_, true))
                })
                .max_by_key(|r| (r.key.ticks, r.key.packet));
            let z = self
                .refs
                .iter()
                .filter(|r| {
                    r.key.ticks > p.key.ticks
                        && matches!(r.input.value, ReferenceValue::Readable(_, true))
                })
                .min_by_key(|r| (r.key.ticks, r.key.packet));
            if let Some(a) = a {
                p.preceding = Some(a.clone());
            }
            if a.is_some_and(|a| a.key.ticks == p.key.ticks) {
                self.bytes -= p.bytes;
                b.answers
                    .push(self.answer(p, ReleaseReason::Standard, None));
                continue;
            }
            if let (Some(a), Some(z)) = (a, z) {
                if crate::timeline::absolute_delta(a, z).is_some() {
                    self.bytes -= p.bytes;
                    b.answers
                        .push(self.answer(p, ReleaseReason::Interpolated, Some(z.clone())));
                    continue;
                }
            }
            remaining.push_back(p);
        }
        self.pending = remaining;
    }
    fn end_session(&self, b: &mut StreamBatch) -> Result<()> {
        if !self.time_seen {
            if let Some(key) = self.first {
                self.emit(b, 1, key, None, None, None)?;
            }
        }
        Ok(())
    }
    /// Bounded Format1 decode with original TF004 cause and streaming context.
    /// **Traces:** L3-ERR-102 ← L2-ERR-102 ← L1-ERR-002
    pub fn observe_format1(
        &mut self,
        packet: PacketObservation,
        input: crate::format1::Format1Input<'_>,
        progress: u64,
        bytes: usize,
        boundary: bool,
    ) -> Result<StreamBatch> {
        let id = packet.id;
        let input = crate::format1::Format1Input {
            scales: self
                .policy
                .offline
                .shared
                .as_ref()
                .map_or(input.scales, |s| s.settings().scales),
            ..input
        };
        let (packet, error) = crate::timeline::decode_observation(packet, input)?;
        let mut batch = self.observe(packet, progress, bytes, boundary)?;
        if let Some(error) = error {
            crate::timeline::retain_decode_error(
                &mut batch.findings,
                id,
                error,
                input.reserved_body_bits().ok(),
                self.policy.offline.finding_levels.get(&5).copied(),
            )?;
        }
        Ok(batch)
    }
    /// Explicit EOF releases as extrapolated (or counter-only), then closes.
    /// **Traces:** L3-STRM-203 ← L2-STRM-203 ← L1-STRM-001
    pub fn finish(&mut self, progress: u64) -> Result<StreamBatch> {
        let mut b = self.advance(progress)?;
        self.flush(&mut b, ReleaseReason::Eof);
        self.end_session(&mut b)?;
        self.refs.clear();
        self.finished = true;
        Ok(b)
    }
    /// Owner-accepted C4 old-snapshot drain on replacement. Never changes an
    /// already committed channel or issued answer. Admission-setting changes
    /// clear old references, requiring fresh evidence under the new snapshot.
    /// **Traces:** L3-STRM-206 ← L2-STRM-206 ← L1-STRM-001
    /// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
    pub fn set_policy(&mut self, policy: StreamPolicy, progress: u64) -> Result<StreamBatch> {
        policy.validate()?;
        self.check_progress(progress)?;
        if progress
            .checked_add(policy.hold_interval_ns)
            .and_then(|p| p.checked_add(policy.hold_tolerance_ns))
            .is_none()
            || policy.id <= self.policy.id
            || self
                .channel
                .is_some_and(|(ch, _)| policy.offline.channel.is_some_and(|p| p != ch))
        {
            return Err(TimeError::InvalidTimelineInput(
                "policy identity or committed channel",
            ));
        }
        let mut b = self.advance(progress)?;
        self.flush(&mut b, ReleaseReason::ModeChange);
        if !self.policy.offline.same_reference_rules(&policy.offline) {
            self.refs.clear();
        }
        if self.channel.is_none() {
            self.channel = policy
                .offline
                .channel
                .map(|ch| (ch, StreamChannelChoice::Caller));
        }
        self.policy = policy;
        while self.refs.len() > self.policy.caps.references {
            self.refs.remove(0);
        }
        Ok(b)
    }
    /// Resolve an undecidable channel explicitly; old answers remain unchanged.
    pub fn choose_channel(
        &mut self,
        channel: u16,
        policy_id: u64,
        progress: u64,
    ) -> Result<StreamBatch> {
        let mut p = self.policy.clone();
        p.id = policy_id;
        p.offline.channel = Some(channel);
        self.set_policy(p, progress)
    }
}
