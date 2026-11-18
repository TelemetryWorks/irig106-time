//! Checked reports replay explicit inputs into fresh, temporary library engines.
//! No internal indexes/queues/anchors are deserialized or returned to a caller.
use super::*;
use crate::{
    answer::*, ch4::*, extracted::*, format1::*, format2::*, own_time::*, policy::*, scale::*,
    secondary_packet::*, stream::*, timeline::*,
};
use alloc::{string::String, vec::Vec};

/// Replay resource ceilings for our Q5 JSON protocol, not an IRIG 106 format.
/// Document declarations can only lower a loader's independent caller ceilings.
/// Readers must also bound JSON bytes before parsing (including policy graphs).
/// Defaults: 1024 packets, 4096 steps, 1024 answers, 4 MiB raw bytes,
/// 32 quality queries and 32768 packet/query pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayLimits {
    /// Maximum observed packets, including references and data.
    pub packets: Decimal<usize>,
    /// Maximum requested offline quality measurements.
    pub quality_queries: Decimal<usize>,
    /// Maximum packets times quality queries (checked before replay).
    /// Conservative bound on the current per-query reference/session scans.
    pub quality_work: Decimal<usize>,
    #[doc = "Maximum count of observations, progress events, queries and measurements."]
    pub steps: Decimal<usize>,
    #[doc = "Maximum count of emitted answer views."]
    pub answers: Decimal<usize>,
    #[doc = "Maximum total retained raw payload/header bytes; bound JSON bytes separately."]
    pub raw_bytes: Decimal<usize>,
}
impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            packets: Decimal::new(1024),
            steps: Decimal::new(4096),
            answers: Decimal::new(1024),
            raw_bytes: Decimal::new(4 * 1024 * 1024),
            quality_queries: Decimal::new(32),
            quality_work: Decimal::new(32768),
        }
    }
}
impl ReplayLimits {
    fn within(&self, ceiling: &Self) -> bool {
        self.packets.get() <= ceiling.packets.get()
            && self.steps.get() <= ceiling.steps.get()
            && self.answers.get() <= ceiling.answers.get()
            && self.raw_bytes.get() <= ceiling.raw_bytes.get()
            && self.quality_queries.get() <= ceiling.quality_queries.get()
            && self.quality_work.get() <= ceiling.quality_work.get()
    }
}
/// An explicit unwrapped query/extraction coordinate; never inferred on load.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayKey {
    #[doc = "Explicit nonzero session identity (decimal count)."]
    pub session: Decimal<usize>,
    #[doc = "Signed unwrapped RTC ticks (decimal,100ns per tick)."]
    pub ticks: Decimal<i128>,
    #[doc = "Original packet identity or recorded packet facts."]
    pub packet: Decimal<u64>,
}
impl ReplayKey {
    fn native(&self) -> Result<PacketKey> {
        if self.session.get() == 0 {
            return Err(invalid());
        }
        Ok(PacketKey {
            session: self.session.get(),
            ticks: self.ticks.get(),
            packet: self.packet.get(),
        })
    }
}
/// Raw payload or explicit caller calendar. Invalid wire data is replayed too.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReferenceSpec {
    /// None replay operation or source variant.
    None,
    /// Format1 replay operation or source variant.
    Format1 {
        #[doc = "Original bytes, including any bytes outside declared Data Length."]
        payload: Vec<u8>,
        #[doc = "Declared Data Length in bytes, decimal platform count."]
        data_length: Decimal<usize>,
        #[doc = "Explicit Format1 EXT/SRC layout; no edition inference."]
        layout: TimeF1Layout,
        #[doc = "Caller-provided scale assumptions, prior to central qualification."]
        scales: ScalePolicy,
    },
    /// Bounded edition-evidence qualification through the existing native adapter.
    QualifiedFormat1 {
        /// Original bounded payload, including any adjacent filler.
        payload: Vec<u8>,
        /// Data Length in decimal bytes.
        data_length: Decimal<usize>,
        /// Original override/TMATS/setup/header declarations, never inferred.
        evidence: crate::edition::EditionEvidence,
        /// Explicit scale assumptions before central qualification.
        scales: ScalePolicy,
    },
    /// Format2 replay operation or source variant.
    Format2 {
        #[doc = "Original bytes, including any bytes outside declared Data Length."]
        payload: Vec<u8>,
        #[doc = "Declared Data Length in bytes, decimal platform count."]
        data_length: Decimal<usize>,
        #[doc = "Explicit nonzero session identity (decimal count)."]
        session: Decimal<usize>,
        #[doc = "Actual optional byte offset, decimal unsigned64."]
        #[serde(deserialize_with = "super::required")]
        byte_offset: Option<Decimal<u64>>,
        #[doc = "Explicit recording edition, if supplied."]
        #[serde(deserialize_with = "super::required")]
        edition: Option<crate::edition::PacketEdition>,
        #[doc = "Original header data type version evidence."]
        data_type_version: u8,
        #[doc = "Complete policy snapshot containing shared context and full leap table."]
        context: PolicySnapshot,
        #[doc = "Explicit raw-network-rate declaration."]
        raw_network_rate: bool,
    },
    /// Calendar replay operation or source variant.
    Calendar {
        #[doc = "Validated calendar with raw/stated/assumed scale."]
        value: CalendarSnapshot,
        #[doc = "Original payload validity."]
        valid: bool,
        #[doc = "Optional explicitly qualified Format1 source."]
        #[serde(deserialize_with = "super::required")]
        source: Option<TimeF1Source>,
        #[doc = "Optional Format1 format evidence."]
        #[serde(deserialize_with = "super::required")]
        format: Option<TimeFormat>,
        #[doc = "Optional IRIG time-source evidence."]
        #[serde(deserialize_with = "super::required")]
        its: Option<IrigTimeSource>,
        #[doc = "Stated resolution in decimal nanoseconds."]
        precision_ns: Decimal<u64>,
        #[doc = "Optional full leap-table snapshot for explicit TAI normalization."]
        #[serde(deserialize_with = "super::required")]
        normalize: Option<LeapTableSnapshot>,
    },
}
/// Original caller packet facts, before centralized qualification/admission.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayPacket {
    #[doc = "Caller packet identity, decimal unsigned64."]
    pub id: Decimal<u64>,
    #[doc = "Unmasked48-bit RTC ticks, decimal unsigned64."]
    pub rtc: Decimal<u64>,
    #[doc = "Packet or time-channel identifier."]
    pub channel: u16,
    #[doc = "Explicit packet class; no classification inferred from body presence."]
    pub kind: PacketKind,
    #[doc = "Original governing TIMEIN declaration."]
    pub declared_timein: bool,
    #[doc = "Original governing external-time declaration."]
    pub declared_external: bool,
    #[doc = "Original packet flag byte."]
    pub flags: u8,
    /// Prior explicit qualifications, in order; retained as provided-policy evidence.
    pub prepare: Vec<Preparation>,
    /// Explicit caller/recording year candidates supplied before central policy.
    #[serde(deserialize_with = "super::required")]
    pub year_context: Option<crate::years::YearContext>,
    #[doc = "Bounded raw input or explicit caller calendar."]
    pub body: ReferenceSpec,
}
impl ReplayPacket {
    fn native(&self) -> Result<PacketObservation> {
        let mut p = PacketObservation {
            id: self.id.get(),
            rtc: crate::rtc::checked_rtc(self.rtc.get())?,
            channel: self.channel,
            kind: self.kind,
            declared_timein: self.declared_timein,
            declared_external: self.declared_external,
            flags: self.flags,
            reference: None,
        };
        p.reference = match &self.body {
            ReferenceSpec::None | ReferenceSpec::Format1 { .. } => None,
            ReferenceSpec::QualifiedFormat1 {
                payload,
                data_length,
                evidence,
                scales,
            } => Some(ReferenceInput::qualified_format1(
                decode_qualified_time_f1(payload, data_length.get(), evidence.clone(), *scales)?,
            )?),
            ReferenceSpec::Format2 {
                payload,
                data_length,
                session,
                byte_offset,
                edition,
                data_type_version,
                context,
                raw_network_rate,
            } => {
                let s = shared(context)?;
                let obs = Format2Input {
                    payload,
                    data_length: data_length.get(),
                    capture: ReferenceCapture {
                        session: session.get(),
                        packet: p.id,
                        channel: p.channel,
                        rtc: p.rtc,
                        byte_offset: byte_offset.map(Decimal::get),
                    },
                    declared_edition: *edition,
                    data_type_version: *data_type_version,
                    declared_timein: p.declared_timein,
                }
                .decode()?;
                Some(ReferenceInput::format2(
                    obs,
                    s.ntp_context(session.get())?,
                    s.settings().own.table.clone(),
                    *raw_network_rate,
                )?)
            }
            ReferenceSpec::Calendar {
                value,
                valid,
                source,
                format,
                its,
                precision_ns,
                normalize,
            } => {
                let c = value.value();
                let mut input = ReferenceInput {
                    value: ReferenceValue::Readable(crate::reference::legacy_calendar(c)?, *valid),
                    scale: c.scale,
                    source: *source,
                    format: *format,
                    its: *its,
                    precision_ns: precision_ns.get(),
                    wire: None,
                    qualified: None,
                    raw_csdw: None,
                    year_origin: if c.year.is_some() {
                        YearOrigin::Caller
                    } else {
                        YearOrigin::Unknown
                    },
                    year_basis: crate::years::YearBasis::default(),
                    network: None,
                    normalization: None,
                    qualification: None,
                };
                if let Some(table) = normalize {
                    input = input.normalize_to_tai(table.value().clone())?;
                }
                Some(input)
            }
        };
        if let Some(context) = self.year_context {
            if let Some(input) = self.raw_f1() {
                p.reference = Some(ReferenceInput::format1(
                    input.payload,
                    input.data_length,
                    input.layout,
                    input.scales,
                )?);
            }
            p.reference = Some(
                p.reference
                    .take()
                    .ok_or_else(invalid)?
                    .with_year_context(context)?,
            );
        }
        if !self.prepare.is_empty() {
            if p.reference.is_none() {
                if let Some(input) = self.raw_f1() {
                    p = crate::timeline::decode_observation(p, input)?.0;
                }
            }
            for step in &self.prepare {
                let key = step.key.native()?;
                if key.packet != p.id {
                    return Err(invalid());
                }
                p = shared(&step.policy)?.prepare(p, key)?;
            }
        }
        Ok(p)
    }
    fn format1(&self) -> Option<Format1Input<'_>> {
        if self.prepare.is_empty() && self.year_context.is_none() {
            self.raw_f1()
        } else {
            None
        }
    }
    fn raw_f1(&self) -> Option<Format1Input<'_>> {
        if let ReferenceSpec::Format1 {
            payload,
            data_length,
            layout,
            scales,
        } = &self.body
        {
            Some(Format1Input {
                payload,
                data_length: data_length.get(),
                layout: *layout,
                scales: *scales,
            })
        } else {
            None
        }
    }
    fn raw_bytes(&self) -> usize {
        match &self.body {
            ReferenceSpec::Format1 { payload, .. }
            | ReferenceSpec::QualifiedFormat1 { payload, .. }
            | ReferenceSpec::Format2 { payload, .. } => payload.len(),
            _ => 0,
        }
    }
}
/// Observation order and explicit boundary; sorting is the engine's job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedPacket {
    #[doc = "Original packet identity or recorded packet facts."]
    pub packet: ReplayPacket,
    #[doc = "Explicit new-session boundary."]
    pub boundary: bool,
}
/// Explicit caller progress, bytes, mode changes and final EOF, never host time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StreamStep {
    /// Observe replay operation or source variant.
    Observe {
        #[doc = "Original observation facts."]
        record: RecordedPacket,
        #[doc = "Explicit caller progress in decimal nanoseconds."]
        progress: Decimal<u64>,
        #[doc = "Caller charge for held packet bytes, decimal platform count."]
        bytes: Decimal<usize>,
    },
    /// Advance replay operation or source variant.
    Advance {
        #[doc = "Explicit caller progress in decimal nanoseconds."]
        progress: Decimal<u64>,
    },
    /// Policy replay operation or source variant.
    Policy {
        #[doc = "Complete versioned policy applied at this event."]
        policy: PolicySnapshot,
        #[doc = "Explicit caller progress in decimal nanoseconds."]
        progress: Decimal<u64>,
    },
    /// Finish replay operation or source variant.
    Finish {
        #[doc = "Explicit caller progress in decimal nanoseconds."]
        progress: Decimal<u64>,
    },
}
/// Quality measurements use the same native channel/session selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityQuery {
    #[doc = "Explicit nonzero session identity (decimal count)."]
    pub session: Decimal<usize>,
    #[doc = "Packet or time-channel identifier."]
    pub channel: u16,
}
/// Full clock inputs; a stream must explicitly end with Finish.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ClockRecipe {
    /// Offline replay operation or source variant.
    Offline {
        #[doc = "Complete versioned policy applied at this event."]
        policy: PolicySnapshot,
        #[doc = "Recorded offline observations in file order."]
        packets: Vec<RecordedPacket>,
        #[doc = "Explicit offline channel/session measurements requested."]
        quality: Vec<QualityQuery>,
    },
    /// Stream replay operation or source variant.
    Stream {
        #[doc = "Complete versioned policy applied at this event."]
        policy: PolicySnapshot,
        #[doc = "Advance channel inventory; null means undecidable."]
        #[serde(deserialize_with = "super::required")]
        inventory: Option<Vec<StreamChannel>>,
        #[doc = "Maximum count of observations, progress events, queries and measurements."]
        steps: Vec<StreamStep>,
    },
}
/// Explicit extraction location, including actual byte offset and caller name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayOrigin {
    #[doc = "Explicit query or extraction coordinate."]
    pub key: ReplayKey,
    #[doc = "Packet or time-channel identifier."]
    pub channel: u16,
    #[doc = "Actual optional byte offset, decimal unsigned64."]
    #[serde(deserialize_with = "super::required")]
    pub byte_offset: Option<Decimal<u64>>,
    #[doc = "Nonempty caller extraction name."]
    pub field: String,
}
impl ReplayOrigin {
    fn native(&self) -> Result<ExtractionOrigin> {
        if self.field.trim().is_empty() {
            return Err(invalid());
        }
        Ok(ExtractionOrigin {
            key: self.key.native()?,
            channel: self.channel,
            byte_offset: self.byte_offset.map(Decimal::get),
            field: self.field.clone(),
        })
    }
}
/// Matching components are bound by explicit identity, not nearest-word guesses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayBinding {
    #[doc = "Extraction being completed by a matching component."]
    pub target: ReplayOrigin,
    #[doc = "Companion extraction identity."]
    pub other: ReplayOrigin,
    #[doc = "Original extracted full/component word."]
    pub word: NetworkWord,
}
/// Adapter requests refer only to existing, already produced counter answers.
/// Context policies must contain SharedPolicy, including the full conversion table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReportRequest {
    /// Counter replay operation or source variant.
    Counter {
        #[doc = "Explicit query or extraction coordinate."]
        key: ReplayKey,
    },
    /// Own replay operation or source variant.
    Own {
        #[doc = "Index of a preceding produced counter answer, decimal count."]
        counter: Decimal<usize>,
        #[doc = "Original extraction metadata, not a nearby-packet guess."]
        origin: ReplayOrigin,
        #[doc = "Original header data type for applicability checks."]
        data_type: u8,
        #[doc = "Unmasked48-bit RTC ticks, decimal unsigned64."]
        rtc: Decimal<u64>,
        #[doc = "Original packet flag byte."]
        flags: u8,
        #[doc = "Original extracted/header bytes."]
        raw: Vec<u8>,
        #[doc = "First-bit or explicitly cited data-type exception."]
        applicability: TimeApplicability,
        #[doc = "Complete policy snapshot containing shared context and full leap table."]
        context: PolicySnapshot,
    },
    /// Stamp replay operation or source variant.
    Stamp {
        #[doc = "Index of a preceding produced counter answer, decimal count."]
        #[serde(deserialize_with = "super::required")]
        counter: Option<Decimal<usize>>,
        #[doc = "Original extraction metadata, not a nearby-packet guess."]
        origin: ReplayOrigin,
        #[doc = "Caller stamp query identity, decimal unsigned64."]
        query_id: Decimal<u64>,
        #[doc = "Original packet flag byte."]
        flags: u8,
        #[doc = "Original extracted/header bytes."]
        raw: Vec<u8>,
        #[doc = "Stored RTC ordering, including non-Nearest policies."]
        order: crate::rtc::CounterOrder,
        #[doc = "Complete policy snapshot containing shared context and full leap table."]
        context: PolicySnapshot,
    },
    /// Word replay operation or source variant.
    Word {
        #[doc = "Original extraction metadata, not a nearby-packet guess."]
        origin: ReplayOrigin,
        #[doc = "Original extracted full/component word."]
        word: TimeWord,
        #[doc = "Complete policy snapshot containing shared context and full leap table."]
        context: PolicySnapshot,
        #[doc = "Explicit BCD year/day/interpretation context."]
        bcd: BcdContext,
        #[doc = "Explicit BCD scale assumption, never implicit UTC."]
        bcd_scale: TimeScale,
        #[doc = "Optional explicit complementary-word binding."]
        #[serde(deserialize_with = "super::required")]
        matching: Option<ReplayBinding>,
        /// Explicit word context, including a mismatched/raw session for diagnosis.
        /// Null selects the context policy's entry for the extraction session.
        #[serde(deserialize_with = "super::required")]
        ntp_context: Option<crate::ntp::NtpContext>,
    },
}
/// Immutable snapshots validate this public construction data before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayPlan {
    #[doc = "Declared resource ceilings; never authority to raise caller load limits."]
    pub limits: ReplayLimits,
    #[doc = "Complete clock replay recipe."]
    pub clock: ClockRecipe,
    #[doc = "Adapter/query requests in evaluation order."]
    pub requests: Vec<ReportRequest>,
}
fn shared(p: &PolicySnapshot) -> Result<&SharedPolicy> {
    core_policy(p)
        .shared
        .as_deref()
        .ok_or(TimeError::NeedsContext {
            context: "report explicit shared context",
        })
}
fn core_policy(p: &PolicySnapshot) -> &OfflinePolicy {
    match p.value() {
        PolicyValue::Offline(p) => p,
        PolicyValue::Stream(p) => &p.offline,
    }
}
fn offline_policy(p: &PolicySnapshot) -> Result<OfflinePolicy> {
    match p.value() {
        PolicyValue::Offline(p) => Ok(p.clone()),
        _ => Err(invalid()),
    }
}
fn stream_policy(p: &PolicySnapshot) -> Result<StreamPolicy> {
    match p.value() {
        PolicyValue::Stream(p) => Ok(p.clone()),
        _ => Err(invalid()),
    }
}
#[derive(Debug, Clone)]
struct Results {
    answers: Vec<AnswerOutput>,
    findings: Vec<TimelineFinding>,
    quality: Vec<OfflineQuality>,
    sessions: Option<Vec<OfflineSession>>,
}
super::wide::serialize_struct!(Results {
    answers,
    findings,
    quality,
    sessions
});
impl ReplayPlan {
    fn validate_size(&self) -> Result<()> {
        self.check_size(&self.limits)
    }
    fn check_size(&self, ceiling: &ReplayLimits) -> Result<()> {
        let (packets, queries, stream_answers) = match &self.clock {
            ClockRecipe::Offline {
                packets, quality, ..
            } => (packets.len(), quality.len(), 0),
            ClockRecipe::Stream { steps, .. } => {
                let mut packets = 0usize;
                let mut answers = 0usize;
                for step in steps {
                    if let StreamStep::Observe { record, .. } = step {
                        packets = packets.checked_add(1).ok_or_else(invalid)?;
                        if record.packet.kind == PacketKind::Data {
                            answers = answers.checked_add(1).ok_or_else(invalid)?;
                        }
                    }
                }
                (packets, 0, answers)
            }
        };
        let answers = stream_answers
            .checked_add(self.requests.len())
            .ok_or_else(invalid)?;
        let work = packets.checked_mul(queries).ok_or_else(invalid)?;
        let (steps, raw) = match &self.clock {
            ClockRecipe::Offline {
                packets, quality, ..
            } => (
                packets.len().checked_add(quality.len()),
                packets
                    .iter()
                    .map(|p| p.packet.raw_bytes())
                    .try_fold(0usize, usize::checked_add),
            ),
            ClockRecipe::Stream { steps, .. } => (
                Some(steps.len()),
                steps
                    .iter()
                    .map(|s| {
                        if let StreamStep::Observe { record, .. } = s {
                            record.packet.raw_bytes()
                        } else {
                            0
                        }
                    })
                    .try_fold(0usize, usize::checked_add),
            ),
        };
        let preparation_steps = match &self.clock {
            ClockRecipe::Offline { packets, .. } => packets
                .iter()
                .map(|p| p.packet.prepare.len())
                .try_fold(0usize, usize::checked_add),
            ClockRecipe::Stream { steps, .. } => steps
                .iter()
                .map(|s| {
                    if let StreamStep::Observe { record, .. } = s {
                        record.packet.prepare.len()
                    } else {
                        0
                    }
                })
                .try_fold(0usize, usize::checked_add),
        }
        .ok_or_else(invalid)?;
        let steps = steps.and_then(|n| n.checked_add(preparation_steps));
        let raw = self
            .requests
            .iter()
            .map(|r| match r {
                ReportRequest::Own { raw, .. } | ReportRequest::Stamp { raw, .. } => raw.len(),
                _ => 0,
            })
            .try_fold(raw.ok_or_else(invalid)?, usize::checked_add)
            .ok_or_else(invalid)?;
        let steps = steps
            .and_then(|n| n.checked_add(self.requests.len()))
            .ok_or_else(invalid)?;
        if self.limits.steps.get() == 0
            || self.limits.answers.get() == 0
            || self.limits.raw_bytes.get() == 0
            || packets > ceiling.packets.get()
            || queries > ceiling.quality_queries.get()
            || work > ceiling.quality_work.get()
            || answers > ceiling.answers.get()
            || steps > ceiling.steps.get()
            || raw > ceiling.raw_bytes.get()
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn execute(&self) -> Result<Results> {
        self.validate_size()?;
        let mut r = Results {
            answers: Vec::new(),
            findings: Vec::new(),
            quality: Vec::new(),
            sessions: None,
        };
        let mut offline = None;
        match &self.clock {
            ClockRecipe::Offline {
                policy,
                packets,
                quality,
            } => {
                let mut engine = TimelineBuilder::new(offline_policy(policy)?)?;
                for record in packets {
                    let p = &record.packet;
                    if let Some(input) = p.format1() {
                        engine.observe_format1(p.native()?, input, record.boundary)?;
                    } else {
                        engine.observe(p.native()?, record.boundary)?;
                    }
                }
                let t = engine.finish()?;
                r.sessions = Some(t.sessions().to_vec());
                r.findings = t.findings().to_vec();
                for q in quality {
                    r.quality.push(t.quality(q.session.get(), q.channel)?);
                }
                offline = Some(t);
            }
            ClockRecipe::Stream {
                policy,
                inventory,
                steps,
            } => {
                if !matches!(steps.last(), Some(StreamStep::Finish { .. })) {
                    return Err(invalid());
                }
                let mut engine = StreamTimeline::new(stream_policy(policy)?, inventory.as_deref())?;
                for (i, step) in steps.iter().enumerate() {
                    let batch = match step {
                        StreamStep::Observe {
                            record,
                            progress,
                            bytes,
                        } => {
                            let p = &record.packet;
                            if let Some(input) = p.format1() {
                                engine.observe_format1(
                                    p.native()?,
                                    input,
                                    progress.get(),
                                    bytes.get(),
                                    record.boundary,
                                )?
                            } else {
                                engine.observe(
                                    p.native()?,
                                    progress.get(),
                                    bytes.get(),
                                    record.boundary,
                                )?
                            }
                        }
                        StreamStep::Advance { progress } => engine.advance(progress.get())?,
                        StreamStep::Policy { policy, progress } => {
                            engine.set_policy(stream_policy(policy)?, progress.get())?
                        }
                        StreamStep::Finish { progress } => {
                            if i + 1 != steps.len() {
                                return Err(invalid());
                            }
                            engine.finish(progress.get())?
                        }
                    };
                    if r.answers
                        .len()
                        .checked_add(batch.answers.len())
                        .is_none_or(|n| n > self.limits.answers.get())
                    {
                        return Err(invalid());
                    }
                    for a in batch.answers {
                        r.answers
                            .push(AnswerOutput::new(AnswerView::stream(&a, &[])?)?);
                    }
                    r.findings.extend(batch.findings);
                }
            }
        }
        for request in &self.requests {
            if r.answers.len() >= self.limits.answers.get() {
                return Err(invalid());
            }
            let view = match request {
                ReportRequest::Counter { key } => {
                    let k = key.native()?;
                    if let Some(t) = &offline {
                        AnswerView::offline(&t.lookup(k)?, &[])?
                    } else {
                        r.answers
                            .iter()
                            .find(|a| {
                                a.view().value().counter.as_ref().is_some_and(|c| match c {
                                    ResolvedCounter::Offline(a) => a.basis.key == k,
                                    ResolvedCounter::Stream(a) => a.basis.key == k,
                                })
                            })
                            .ok_or_else(invalid)?
                            .view()
                            .clone()
                    }
                }
                ReportRequest::Own {
                    counter,
                    origin,
                    data_type,
                    rtc,
                    flags,
                    raw,
                    applicability,
                    context,
                } => {
                    let c = counter_at(&r, counter.get())?;
                    let o = origin.native()?;
                    let a = answer_own(
                        c.as_counter(),
                        SecondaryPacket {
                            key: o.key,
                            channel: o.channel,
                            data_type: *data_type,
                            rtc: crate::rtc::checked_rtc(rtc.get())?,
                            flags: *flags,
                            bytes: raw,
                            byte_offset: o.byte_offset,
                            applicability: applicability.clone(),
                        },
                        &shared(context)?.settings().own,
                        &core_policy(context).own_policy(),
                    )?;
                    AnswerView::own(&a)?
                }
                ReportRequest::Stamp {
                    counter,
                    origin,
                    query_id,
                    flags,
                    raw,
                    order,
                    context,
                } => {
                    if let Some(index) = counter {
                        counter_at(&r, index.get())?;
                    }
                    let policy = SecondaryPolicy {
                        levels: core_policy(context).finding_levels.clone(),
                    };
                    let i = inspect_stamp(
                        StampInput {
                            origin: origin.native()?,
                            query_id: query_id.get(),
                            flags: *flags,
                            bytes: raw,
                            order: *order,
                        },
                        &policy,
                    )?;
                    let a = i.resolve(&shared(context)?.settings().own, |_| {
                        counter_at(&r, counter.ok_or_else(invalid)?.get()).cloned()
                    })?;
                    AnswerView::stamp(&a)?
                }
                ReportRequest::Word {
                    origin,
                    word,
                    context,
                    bcd,
                    bcd_scale,
                    matching,
                    ntp_context,
                } => {
                    let s = shared(context)?;
                    let o = origin.native()?;
                    let ctx = WordContext {
                        own: s.settings().own.clone(),
                        bcd: *bcd,
                        bcd_scale: *bcd_scale,
                        ntp: match ntp_context {
                            Some(ctx) => ctx.clone(),
                            None => s.ntp_context(o.key.session)?,
                        },
                        matching: matching
                            .as_ref()
                            .map(|b| {
                                Ok::<_, TimeError>(ComponentBinding {
                                    target: b.target.native()?,
                                    other: b.other.native()?,
                                    word: b.word,
                                })
                            })
                            .transpose()?,
                    };
                    let a = answer_word(
                        o,
                        *word,
                        &ctx,
                        &SecondaryPolicy {
                            levels: core_policy(context).finding_levels.clone(),
                        },
                    )?;
                    AnswerView::word(&a)?
                }
            };
            r.answers.push(AnswerOutput::new(view)?);
        }
        Ok(r)
    }
}
fn counter_at(r: &Results, index: usize) -> Result<&ResolvedCounter> {
    r.answers
        .get(index)
        .and_then(|a| a.view().value().counter.as_ref())
        .ok_or_else(invalid)
}
/// Complete versioned replay report: explicit input recipe plus checked outputs.
/// **Traces:** L3-API-110 ← L2-API-110 ← L1-API-010
/// **Traces:** L3-POL-103 ← L2-POL-103 ← L1-POL-003
#[derive(Debug, Clone)]
pub struct ReportSnapshot {
    #[doc = "plan replay parameter."]
    plan: ReplayPlan,
    #[doc = "results replay parameter."]
    results: Results,
}
impl ReportSnapshot {
    /// Load with independent caller ceilings, preflighting declarations and
    /// actual inputs/cached answer counts before any replay work. Bound JSON
    /// bytes before invoking the deserializer. Plain Deserialize uses defaults.
    pub fn load<'de, D: Deserializer<'de>>(
        d: D,
        limits: &ReplayLimits,
    ) -> core::result::Result<Self, D::Error> {
        Self::from_wire(object(d)?, limits)
    }
    fn from_wire<E: de::Error>(
        w: ReportWire,
        limits: &ReplayLimits,
    ) -> core::result::Result<Self, E> {
        header(&w.schema, w.version, "irig106-time/report").map_err(E::custom)?;
        if w.defaults_version != 1 || w.units != report_units() {
            return Err(E::custom(invalid()));
        }
        if !w.plan.limits.within(limits)
            || w.plan.check_size(limits).is_err()
            || w.plan.validate_size().is_err()
            || !w.cached_sizes_within(limits)
        {
            return Err(E::custom("replay resource limits exceeded"));
        }
        let report = Self::new(w.plan).map_err(E::custom)?;
        if super::data::capture(&report.results).map_err(E::custom)? != w.outputs {
            return Err(E::custom("report outputs do not match replay"));
        }
        Ok(report)
    }
    /// Execute bounded inputs using the real library paths; no live state escapes.
    pub fn new(plan: ReplayPlan) -> Result<Self> {
        let results = plan.execute()?;
        Ok(Self { plan, results })
    }
    /// Full source inputs, including policy/table/default version and progress.
    pub fn plan(&self) -> &ReplayPlan {
        &self.plan
    }
    /// Recomputed full owned answer views, in release/request order.
    pub fn answers(&self) -> &[AnswerOutput] {
        &self.results.answers
    }
    /// All engine findings, even when not attached to a selected answer.
    pub fn findings(&self) -> &[TimelineFinding] {
        &self.results.findings
    }
    /// Requested offline measurements, with interval and unavailable reasons.
    pub fn quality(&self) -> &[OfflineQuality] {
        &self.results.quality
    }
    /// Validated offline session/reference records, not a live timeline/index.
    pub fn sessions(&self) -> Option<&[OfflineSession]> {
        self.results.sessions.as_deref()
    }

    /// Replay independently from the immutable source inputs.
    pub fn replay(&self) -> Result<Self> {
        Self::new(self.plan.clone())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportWire {
    schema: String,
    version: u16,
    defaults_version: u16,
    #[serde(deserialize_with = "super::unit_map")]
    units: BTreeMap<String, String>,
    plan: ReplayPlan,
    outputs: super::data::Node,
}
impl ReportWire {
    fn cached_sizes_within(&self, limits: &ReplayLimits) -> bool {
        let super::data::Node::Map(fields) = &self.outputs else {
            return false;
        };
        [
            ("answers", limits.answers.get()),
            ("quality", limits.quality_queries.get()),
        ]
        .iter()
        .all(|(name, cap)| {
            matches!(
                fields.iter().find(|(key, _)| key == name),
                Some((_, super::data::Node::Seq(v))) if v.len() <= *cap
            )
        })
    }
}
impl Serialize for ReportSnapshot {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        ReportWire {
            schema: "irig106-time/report".into(),
            version: 1,
            defaults_version: 1,
            units: report_units(),
            plan: self.plan.clone(),
            outputs: super::data::capture(&self.results).map_err(serde::ser::Error::custom)?,
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for ReportSnapshot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        Self::load(d, &ReplayLimits::default())
    }
}
/// Self-contained individual answer snapshot, with complete replay evidence.
#[derive(Debug, Clone)]
pub struct AnswerSnapshot {
    #[doc = "report replay parameter."]
    report: ReportSnapshot,
    #[doc = "index replay parameter."]
    index: usize,
}
impl AnswerSnapshot {
    /// Load the nested report with the caller's ceilings, before replay.
    /// Bound JSON bytes before parsing. Plain Deserialize uses defaults.
    pub fn load<'de, D: Deserializer<'de>>(
        d: D,
        limits: &ReplayLimits,
    ) -> core::result::Result<Self, D::Error> {
        let w: DefaultEnvelope<SelectedWire> = object(d)?;
        header(&w.schema, w.version, "irig106-time/answer").map_err(de::Error::custom)?;
        if w.defaults_version != 1 || w.units != report_units() {
            return Err(de::Error::custom(invalid()));
        }
        Self::new(
            ReportSnapshot::from_wire(w.value.report, limits)?,
            w.value.index.get(),
        )
        .map_err(de::Error::custom)
    }
    /// Validate the selected answer, retaining the full bounded input report.
    pub fn new(report: ReportSnapshot, index: usize) -> Result<Self> {
        if index >= report.answers().len() {
            return Err(invalid());
        }
        Ok(Self { report, index })
    }
    /// Selected common answer with the full raw/stated/assumed basis.
    pub fn view(&self) -> &AnswerView {
        self.report.answers()[self.index].view()
    }
    /// Full replay report shared by this snapshot's selected answer.
    pub fn report(&self) -> &ReportSnapshot {
        &self.report
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectedWire {
    report: ReportWire,
    index: Decimal<usize>,
}
#[derive(Serialize)]
struct Selected {
    report: ReportSnapshot,
    index: Decimal<usize>,
}
impl Serialize for AnswerSnapshot {
    fn serialize<S: Serializer>(&self, s: S) -> core::result::Result<S::Ok, S::Error> {
        DefaultEnvelope {
            schema: "irig106-time/answer".into(),
            version: 1,
            defaults_version: 1,
            units: report_units(),
            value: Selected {
                report: self.report.clone(),
                index: Decimal::new(self.index),
            },
        }
        .serialize(s)
    }
}
impl<'de> Deserialize<'de> for AnswerSnapshot {
    fn deserialize<D: Deserializer<'de>>(d: D) -> core::result::Result<Self, D::Error> {
        Self::load(d, &ReplayLimits::default())
    }
}

/// A prior qualifier and the actual coordinate used to select its declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preparation {
    /// Complete prior policy, never just a bare ID.
    pub policy: PolicySnapshot,
    /// Exact source qualification coordinate (session/ticks/packet).
    pub key: ReplayKey,
}
