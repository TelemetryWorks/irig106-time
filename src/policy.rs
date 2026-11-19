//! Shared qualification snapshots (ADR0010/0011, contract5.3/5.6).
//! Defaults retain owner D5-D9 and C1-C4; no host clock or hidden registry.
use crate::{
    declarations::*,
    extracted::WordContext,
    ntp::*,
    own_time::*,
    reference::{NetworkReference, Normalization},
    scale::*,
    stream::StreamPolicy,
    timeline::*,
    years::YearContext,
    *,
};
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
/// Named year evidence classes. Network dates retain their own era evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YearSource {
    /// Same-session DMY, including the phase-advanced stable-first anchor.
    Packet,
    /// Explicit caller year.
    Caller,
    /// Governing recording date RI4, never G\OD.
    Recording,
}
/// Validated priority/subset; excluded candidates remain factual evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YearOrder {
    order: [YearSource; 3],
    len: u8,
}
impl Default for YearOrder {
    fn default() -> Self {
        Self {
            order: [
                YearSource::Packet,
                YearSource::Caller,
                YearSource::Recording,
            ],
            len: 3,
        }
    }
}
impl YearOrder {
    /// Reject duplicates. Empty disables inference; explicitly dated packets keep
    /// their calendar. Fixed caller-only inference is [YearSource::Caller].
    pub fn new(order: Vec<YearSource>) -> Result<Self> {
        if order.len() > 3
            || order
                .iter()
                .enumerate()
                .any(|(i, s)| order[..i].contains(s))
        {
            return Err(TimeError::InvalidTimelineInput("duplicate year source"));
        }
        let mut result = Self {
            len: order.len() as u8,
            ..Self::default()
        };
        for (i, s) in order.into_iter().enumerate() {
            result.order[i] = s;
        }
        Ok(result)
    }
    /// Exact inference priority/subset. Inline storage adds no per-wire allocation.
    pub fn sources(&self) -> &[YearSource] {
        &self.order[..usize::from(self.len)]
    }
    pub(crate) fn rank(&self, origin: YearOrigin) -> Option<usize> {
        self.sources().iter().position(|s| {
            *s == match origin {
                YearOrigin::Packet => YearSource::Packet,
                YearOrigin::RecordingDate => YearSource::Recording,
                _ => YearSource::Caller,
            }
        })
    }
}
/// Mutable construction data; freeze with SharedPolicy::new before use.
/// **Traces:** L3-POL-101 ← L2-POL-101 ← L1-POL-001
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SharedSettings {
    /// Default caller year for Format1 inference; never fills Chapter4 word context.
    pub caller_year: Option<u16>,
    /// D9 inference order for yearless values; dated packets keep their own year.
    pub year_order: YearOrder,
    /// Assumptions only: stated UTC/GPS formats are never relabelled.
    pub scales: ScalePolicy,
    /// Explicit own epochs, applicability and validated conversion table.
    pub own: OwnContext,
    /// None uses reference resolution (contract6.6); no zero placeholder.
    pub own_tolerance: Option<NanosDuration>,
    /// None preserves caller-prepared declaration labels; Some is authoritative
    /// plain setup data, including explicit Declarations::absent(). No defaults inserted.
    pub declarations: Option<Declarations>,
    /// D8 opt-in comparison default, not a declaration rewrite.
    pub use_tfmt_default: bool,
    /// Caller-bounded era/pivot contexts by session; absent entry is labelled era0.
    pub ntp: BTreeMap<usize, NtpContext>,
}
/// Immutable qualification snapshot, with caller-series identity and default version.
/// Full clock settings remain in OfflinePolicy/StreamPolicy beside this snapshot.
/// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedPolicy {
    id: u64,
    settings: SharedSettings,
}
impl SharedPolicy {
    /// Validate without repairing caller data. IDs must be unique in the caller's
    /// policy series; stream replacement additionally enforces monotonic IDs.
    pub fn new(id: u64, settings: SharedSettings) -> Result<Self> {
        if id == 0 || settings.caller_year.is_some_and(|y| y > 9999) {
            return Err(TimeError::InvalidTimelineInput(
                "shared policy identity/year",
            ));
        }
        for e in [&settings.own.ieee, &settings.own.ch4]
            .into_iter()
            .flatten()
        {
            e.origin.validate()?;
            if e.source.trim().is_empty()
                || e.counting == EpochCounting::Civil && e.origin.seconds == 60
            {
                return Err(TimeError::InvalidTimelineInput(
                    "unsupported epoch provenance/counting",
                ));
            }
        }
        if let crate::secondary_packet::TimeApplicability::CitedException { citation, .. } =
            &settings.own.counter_applicability
        {
            if citation.trim().is_empty() {
                return Err(TimeError::InvalidTimelineInput(
                    "empty applicability citation",
                ));
            }
        }
        for (session, context) in &settings.ntp {
            if *session == 0
                || *session != context.session
                || context
                    .declared
                    .as_ref()
                    .is_some_and(|d| d.id.trim().is_empty())
                || context.pivots.iter().any(|p| p.session() != *session)
            {
                return Err(TimeError::InvalidTimelineInput(
                    "NTP policy session/provenance",
                ));
            }
        }
        Ok(Self { id, settings })
    }
    /// Caller-series snapshot identity, not a clock counter or standard release.
    pub fn id(&self) -> u64 {
        self.id
    }
    /// Version of the cited policy defaults, separate from future JSON schema version.
    pub fn defaults_version(&self) -> u16 {
        1
    }
    /// Frozen settings; clone them to construct another identified snapshot.
    pub fn settings(&self) -> &SharedSettings {
        &self.settings
    }
    /// Attach shared qualification to an offline core without changing its defaults.
    pub fn offline(&self, mut core: OfflinePolicy) -> Result<OfflinePolicy> {
        core.shared = Some(Box::new(self.clone()));
        core.validate()?;
        Ok(core)
    }
    /// Attach to a stream snapshot of the same ID; never silently rewrite identity.
    pub fn stream(&self, mut core: StreamPolicy) -> Result<StreamPolicy> {
        if core.id != self.id {
            return Err(TimeError::InvalidTimelineInput(
                "shared/stream policy ID mismatch",
            ));
        }
        core.offline = self.offline(core.offline)?;
        core.validate()?;
        Ok(core)
    }
    /// Resolve the explicitly named session's era context, or RDG012 era0 policy.
    pub fn ntp_context(&self, session: usize) -> Result<NtpContext> {
        if session == 0 {
            return Err(TimeError::InvalidTimelineInput("NTP session is zero"));
        }
        Ok(self
            .settings
            .ntp
            .get(&session)
            .cloned()
            .unwrap_or_else(|| NtpContext::new(session)))
    }
    /// Share epochs/table/era defaults, but never infer a word's full day/year.
    pub fn word_context(&self, session: usize) -> Result<WordContext> {
        let mut c = WordContext::new(session);
        c.own = self.settings.own.clone();
        c.ntp = self.ntp_context(session)?;
        Ok(c)
    }
    /// Explicit D8 comparison policy using the clock's severity settings.
    pub fn declaration_policy(&self, core: &OfflinePolicy) -> DeclarationPolicy {
        DeclarationPolicy {
            use_tfmt_default: self.settings.use_tfmt_default,
            levels: core.finding_levels.clone(),
        }
    }
    pub(crate) fn same_rules(&self, other: &Self) -> bool {
        self.settings == other.settings
    }
    pub(crate) fn prepare(
        &self,
        mut packet: PacketObservation,
        key: PacketKey,
    ) -> Result<PacketObservation> {
        let prior = packet.reference.as_ref().map(|r| {
            (
                r.scale,
                r.year_basis.context(),
                r.normalization.clone(),
                r.network.clone(),
                r.qualification.as_ref().map(|q| q.policy.clone()),
            )
        });
        let declarations = if let Some(d) = &self.settings.declarations {
            let p = d.prepare(packet, key)?;
            packet = p.observation;
            Some(p.declarations)
        } else {
            None
        };
        if let Some(mut r) = packet.reference.take() {
            let (
                provided_scale,
                provided_year,
                provided_normalization,
                provided_network,
                provided_policy,
            ) = prior.ok_or(TimeError::InvalidTimelineInput(
                "qualification input missing",
            ))?;
            if r.network.is_some() {
                r = r.requalify_network(self, key, packet.declared_timein)?;
            } else {
                if let Some(format) = r.format {
                    r.scale = self.settings.scales.for_format1(format);
                    if let Some((_, w)) = r.wire.as_mut() {
                        if let Some(t) = w.time.as_mut() {
                            t.scale = r.scale;
                        }
                    }
                    if let Some(q) = r.qualified.as_mut() {
                        if let Some(t) = q.time.as_mut() {
                            t.scale = r.scale;
                        }
                    }
                }
                let mut y = r.year_basis.context();
                y.caller = self.settings.caller_year.or(y.caller);
                r = r.with_year_context(y)?;
                r.year_basis.set_order(self.settings.year_order.clone());
                r.normalization = None;
            }
            r.qualification = Some(Box::new(QualificationBasis {
                policy: self.clone(),
                declarations,
                provided_scale,
                provided_year,
                provided_normalization,
                provided_network,
                provided_policy,
            }));
            packet.reference = Some(r);
        }
        crate::reference::check_capture(&packet)?;
        Ok(packet)
    }
}
/// Actual shared qualification and replaced caller evidence, with no history chain.
/// **Traces:** L3-ANS-101 ← L2-ANS-101 ← L1-ANS-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationBasis {
    policy: SharedPolicy,
    declarations: Option<DeclarationBasis>,
    provided_scale: ScaleEvidence,
    provided_year: YearContext,
    provided_normalization: Option<Normalization>,
    provided_network: Option<Box<NetworkReference>>,
    provided_policy: Option<SharedPolicy>,
}
impl QualificationBasis {
    /// Effective shared qualification, retained even after later policy changes.
    pub fn policy(&self) -> &SharedPolicy {
        &self.policy
    }
    /// Actual governing raw setup, or caller-prepared labels when None.
    pub fn declarations(&self) -> Option<&DeclarationBasis> {
        self.declarations.as_ref()
    }
    /// Scale provided before applying centralized assumptions.
    pub fn provided_scale(&self) -> ScaleEvidence {
        self.provided_scale
    }
    /// Provided year inputs; RI4 application remains visible in effective YearBasis.
    pub fn provided_year_context(&self) -> YearContext {
        self.provided_year
    }
    /// Replaced normalization, if supplied, never falsely reported as applied.
    pub fn provided_normalization(&self) -> Option<&Normalization> {
        self.provided_normalization.as_ref()
    }
    /// Replaced network conversion and era/table context, if supplied.
    pub fn provided_network(&self) -> Option<&NetworkReference> {
        self.provided_network.as_deref()
    }
    /// Immediate provided snapshot only; no unbounded policy-history chain.
    pub fn provided_policy(&self) -> Option<&SharedPolicy> {
        self.provided_policy.as_ref()
    }
}
