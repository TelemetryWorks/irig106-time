//! Owned answer replay: ADR-0010/0011, contract3.1/3.5/5.6/7.3.
//! Captures selected evidence, never a live engine or hidden reference registry.
//! Versioned output uses snapshot::AnswerOutput; checked loads replay report inputs.
//!
//! ```
//! use irig106_time::{answer::AnswerView, timeline::OfflineAnswer, Result};
//! fn freeze(answer: &OfflineAnswer) -> Result<AnswerView> {
//!     let view = AnswerView::offline(answer, &[])?;
//!     assert_eq!(view.value(), view.basis().replay()?.value());
//!     Ok(view)
//! }
//! ```
use crate::{
    ch4::Ch4Reading, extracted::*, findings::Finding, format1::CalendarReading,
    format2::NetworkConversion, own_time::*, secondary_packet::*, stream::*, timeline::*, *,
};
use alloc::{boxed::Box, vec::Vec};
fn invalid() -> TimeError {
    TimeError::InvalidTimelineInput("inconsistent answer replay basis")
}
/// Full core policy, selected references and release/selection reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ClockBasis {
    /// Offline selection; no new search during replay.
    Offline(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        Box<OfflineBasis>,
    ),
    /// Released stream selection, including original mode/progress/fallback.
    Stream(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        Box<StreamBasis>,
    ),
}
/// Owned selected counter basis with explicitly caller-supplied findings.
/// **Traces:** L3-ANS-101 ← L2-ANS-101 ← L1-ANS-001
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterBasis {
    clock: ClockBasis,
    findings: Vec<Finding>,
}
impl CounterBasis {
    /// Frozen original core basis; qualifiers retain stated/provided/applied context.
    pub fn clock(&self) -> &ClockBasis {
        &self.clock
    }
    /// Caller-selected evidence, not an assertion that no other finding occurred.
    pub fn supplied_findings(&self) -> &[Finding] {
        &self.findings
    }
    fn capture(c: CounterAnswer<'_>, findings: &[Finding]) -> Result<Self> {
        let (clock, original) = match c {
            CounterAnswer::Offline(a) => (
                ClockBasis::Offline(Box::new(a.basis.clone())),
                ResolvedCounter::offline(a.clone()),
            ),
            CounterAnswer::Stream(a) => (
                ClockBasis::Stream(Box::new(a.basis.clone())),
                ResolvedCounter::stream(a.clone()),
            ),
        };
        let b = Self {
            clock,
            findings: findings.to_vec(),
        };
        if b.resolve()? != original {
            return Err(invalid());
        }
        Ok(b)
    }
    fn resolve(&self) -> Result<ResolvedCounter> {
        match &self.clock {
            ClockBasis::Offline(b) => {
                b.policy.validate()?;
                if b.drift_corrected {
                    return Err(invalid());
                }
                let counter = duration(b.session_origin, b.key)?;
                let reading = if let Some(r) = &b.reference {
                    check_reference(r, b.key, b.channel.map(|c| c.0))?;
                    let delta = duration(r.key, b.key)?.get();
                    if b.distance_ns != Some(delta) {
                        return Err(invalid());
                    }
                    crate::reference::shifted_reference(r, delta).ok()
                } else {
                    if b.distance_ns.is_some() || b.position != ReferencePosition::CounterOnly {
                        return Err(invalid());
                    }
                    None
                };
                if reading.is_some() != b.unresolved.is_none() {
                    return Err(invalid());
                }
                Ok(ResolvedCounter::offline(OfflineAnswer {
                    time: reading.and_then(|c| crate::reference::legacy_calendar(c).ok()),
                    reading,
                    counter_ns: counter.get(),
                    basis: (**b).clone(),
                }))
            }
            ClockBasis::Stream(b) => {
                b.policy.validate()?;
                let counter = duration(b.session_origin, b.key)?;
                if b.following.is_some() != (b.release == ReleaseReason::Interpolated) {
                    return Err(invalid());
                }
                let mut correction = None;
                let reading = if let Some(r) = &b.preceding {
                    check_reference(r, b.key, b.channel.map(|c| c.0))?;
                    let offset = duration(r.key, b.key)?.get();
                    if offset < 0 {
                        return Err(invalid());
                    }
                    let delta = if let Some(n) = &b.following {
                        check_reference(n, b.key, b.channel.map(|c| c.0))?;
                        if b.release != ReleaseReason::Interpolated
                            || b.policy.mode != StreamMode::Interpolate
                            || n.key.ticks <= r.key.ticks
                            || b.key.ticks > n.key.ticks
                        {
                            return Err(invalid());
                        }
                        let d = crate::timeline::absolute_delta(r, n)
                            .and_then(|d| b.key.ticks.checked_sub(r.key.ticks)?.checked_mul(d))
                            .and_then(|d| {
                                n.key
                                    .ticks
                                    .checked_sub(r.key.ticks)
                                    .and_then(|width| d.checked_div_euclid(width))
                            });
                        correction = d.and_then(|d| d.checked_sub(offset));
                        d
                    } else {
                        if b.release == ReleaseReason::Interpolated {
                            return Err(invalid());
                        }
                        Some(offset)
                    };
                    delta.and_then(|d| crate::reference::shifted_reference(r, d).ok())
                } else {
                    if b.following.is_some() {
                        return Err(invalid());
                    }
                    None
                };
                if correction != b.drift_correction_ns
                    || reading.is_some() != b.unresolved.is_none()
                {
                    return Err(invalid());
                }
                Ok(ResolvedCounter::stream(StreamAnswer {
                    time: reading.and_then(|c| crate::reference::legacy_calendar(c).ok()),
                    reading,
                    counter_ns: counter.get(),
                    basis: (**b).clone(),
                }))
            }
        }
    }
}
fn duration(origin: PacketKey, key: PacketKey) -> Result<SignedNanos> {
    origin
        .delta_to(key)?
        .get()
        .checked_mul(100)
        .map(SignedNanos::new)
        .ok_or(TimeError::QuantityOverflow {
            quantity: "answer counter duration",
        })
}
fn check_reference(r: &TimelineReference, key: PacketKey, channel: Option<u16>) -> Result<()> {
    if r.key.session != key.session
        || !r.admitted
        || r.input.precision_ns == 0
        || Some(r.channel) != channel
    {
        return Err(invalid());
    }
    Ok(())
}
/// Own-time envelope keeps independent counter, raw trust and exact adapter inputs.
#[derive(Debug, Clone)]
pub struct OwnEnvelope {
    counter: CounterBasis,
    header: SecondaryInspection,
    context: OwnContext,
    policy: OwnPolicy,
}
impl OwnEnvelope {
    /// Selected independent counter evidence.
    pub fn counter(&self) -> &CounterBasis {
        &self.counter
    }
    /// Original header and checksum/consistency findings, including byte location.
    pub fn header(&self) -> &SecondaryInspection {
        &self.header
    }
    /// Exact epoch/scale/table/applicability inputs.
    pub fn context(&self) -> &OwnContext {
        &self.context
    }
    /// Exact tolerance and severity overrides.
    pub fn policy(&self) -> &OwnPolicy {
        &self.policy
    }
}
/// Stamp envelope retains extraction, raw flags/bytes/order and actual query basis.
#[derive(Debug, Clone)]
pub struct StampEnvelope {
    inspection: StampInspection,
    counter: Option<CounterBasis>,
    context: OwnContext,
}
impl StampEnvelope {
    /// Original resolved inspection, including all findings; replay re-inspects raw inputs.
    pub fn inspection(&self) -> &StampInspection {
        &self.inspection
    }
    /// Actual released query, never an enclosing-packet reselection.
    pub fn counter(&self) -> Option<&CounterBasis> {
        self.counter.as_ref()
    }
    /// Exact epoch/table input.
    pub fn context(&self) -> &OwnContext {
        &self.context
    }
}
/// Word envelope retains raw components, matching binding and session era/table.
#[derive(Debug, Clone)]
pub struct WordEnvelope {
    origin: ExtractionOrigin,
    word: TimeWord,
    context: WordContext,
    policy: SecondaryPolicy,
}
impl WordEnvelope {
    /// Original extraction metadata.
    pub fn origin(&self) -> &ExtractionOrigin {
        &self.origin
    }
    /// Complete raw word kind/components.
    pub fn word(&self) -> TimeWord {
        self.word
    }
    /// Exact caller context, including unresolved alternatives.
    pub fn context(&self) -> &WordContext {
        &self.context
    }
    /// Actual severity settings.
    pub fn policy(&self) -> &SecondaryPolicy {
        &self.policy
    }
}
/// Immutable inputs sufficient to reproduce the answer without an engine.
/// **Traces:** L3-ANS-101 ← L2-ANS-101 ← L1-ANS-001
/// **Traces:** L3-POL-102 ← L2-POL-102 ← L1-POL-002
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum AnswerBasis {
    /// Selected offline/released stream answer.
    Counter(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        Box<CounterBasis>,
    ),
    /// Secondary own and independent counter answers.
    Own(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        Box<OwnEnvelope>,
    ),
    /// Extracted intra-packet stamp answer.
    Stamp(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        Box<StampEnvelope>,
    ),
    /// Caller-extracted time word answer.
    Word(
        #[cfg_attr(
            feature = "serde",
            serde(serialize_with = "crate::snapshot::wide::serialize")
        )]
        Box<WordEnvelope>,
    ),
}
/// Common result projection. Values are output, never trusted replay inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerValue {
    /// Native scale/calendar, including unknown year or second60; never implicit UTC.
    pub calendar: Option<CalendarReading>,
    /// Signed session displacement; None for an absolute/component-only word.
    pub counter_duration: Option<SignedNanos>,
    /// Independent core answer; own time never replaces this evidence.
    pub counter: Option<ResolvedCounter>,
    /// Physical own/stamp/word instant if fully qualified.
    pub point: Option<OwnPoint>,
    /// Own minus counter, when comparable.
    pub own_minus_counter: Option<SignedNanos>,
    /// Applied discrepancy tolerance, not inferred accuracy.
    pub tolerance: Option<NanosDuration>,
    /// Explicit missing/invalid context, never a zero-filled time.
    pub issue: Option<TimeError>,
    /// Adapter findings or explicitly supplied counter findings.
    pub findings: Vec<Finding>,
    /// Full network conversion, including raw/era/pivot/coverage/ambiguity evidence.
    pub network: Option<NetworkConversion>,
    /// Partial Chapter4 interpretation, retained even without calendar context.
    pub chapter4: Option<Ch4Reading>,
}
impl AnswerValue {
    fn empty() -> Self {
        Self {
            calendar: None,
            counter_duration: None,
            counter: None,
            point: None,
            own_minus_counter: None,
            tolerance: None,
            issue: None,
            findings: Vec::new(),
            network: None,
            chapter4: None,
        }
    }
    fn counter(c: ResolvedCounter, findings: Vec<Finding>) -> Self {
        let mut v = Self::empty();
        let (calendar, duration, reason) = match &c {
            ResolvedCounter::Offline(a) => (a.reading, a.counter_duration(), a.basis.unresolved),
            ResolvedCounter::Stream(a) => (a.reading, a.counter_duration(), a.basis.unresolved),
        };
        v.calendar = calendar;
        v.counter_duration = Some(duration);
        v.counter = Some(c);
        v.findings = findings;
        v.issue = reason.map(|context| TimeError::NeedsContext { context });
        v
    }
}
/// Owned replayable answer view; no live timeline, callback or borrowed raw bytes.
#[derive(Debug, Clone)]
pub struct AnswerView {
    basis: AnswerBasis,
    value: AnswerValue,
}
impl AnswerView {
    /// Validate and capture a native offline result. Supplied findings remain caller-selected.
    pub fn offline(a: &OfflineAnswer, findings: &[Finding]) -> Result<Self> {
        AnswerBasis::Counter(Box::new(CounterBasis::capture(
            CounterAnswer::Offline(a),
            findings,
        )?))
        .replay()
    }
    /// Validate and capture an already released stream result without revising selection.
    pub fn stream(a: &StreamAnswer, findings: &[Finding]) -> Result<Self> {
        AnswerBasis::Counter(Box::new(CounterBasis::capture(
            CounterAnswer::Stream(a),
            findings,
        )?))
        .replay()
    }
    /// Capture all secondary conversion inputs plus its independent core answer.
    pub fn own(a: &OwnAnswer<'_>) -> Result<Self> {
        AnswerBasis::Own(Box::new(OwnEnvelope {
            counter: CounterBasis::capture(a.counter(), &[])?,
            header: a.header().clone(),
            context: a.context().clone(),
            policy: a.policy().clone(),
        }))
        .replay()
    }
    /// Capture raw stamp and exact released query; never requery a recording.
    pub fn stamp(a: &StampAnswer) -> Result<Self> {
        AnswerBasis::Stamp(Box::new(StampEnvelope {
            inspection: a.inspection().clone(),
            counter: a
                .counter()
                .map(|c| CounterBasis::capture(c.as_counter(), &[]))
                .transpose()?,
            context: a.context().clone(),
        }))
        .replay()
    }
    /// Capture raw word, component bindings and actual context/severity settings.
    pub fn word(a: &WordAnswer) -> Result<Self> {
        AnswerBasis::Word(Box::new(WordEnvelope {
            origin: a.origin().clone(),
            word: a.word(),
            context: a.context().clone(),
            policy: a.policy().clone(),
        }))
        .replay()
    }
    /// Attach caller-selected core findings without relabelling adapter findings.
    /// They stay in the nested counter basis; primary adapter findings stay separate.
    pub fn with_counter_findings(&self, findings: &[Finding]) -> Result<Self> {
        let mut basis = self.basis.clone();
        let counter = match &mut basis {
            AnswerBasis::Counter(b) => b.as_mut(),
            AnswerBasis::Own(b) => &mut b.counter,
            AnswerBasis::Stamp(b) => b.counter.as_mut().ok_or(invalid())?,
            AnswerBasis::Word(_) => return Err(invalid()),
        };
        counter.findings = findings.to_vec();
        basis.replay()
    }
    /// Frozen replay inputs, separate from the computed output.
    pub fn basis(&self) -> &AnswerBasis {
        &self.basis
    }
    /// Common native/partial/counter-only projection and independent evidence.
    pub fn value(&self) -> &AnswerValue {
        &self.value
    }
}
impl AnswerBasis {
    /// Recompute from selected references/raw inputs and frozen settings only.
    /// **Traces:** L3-ANS-101 ← L2-ANS-101 ← L1-ANS-001
    /// **Traces:** L3-ANS-105 ← L2-ANS-105 ← L1-ANS-005
    pub fn replay(&self) -> Result<AnswerView> {
        let value = match self {
            Self::Counter(b) => AnswerValue::counter(b.resolve()?, b.findings.clone()),
            Self::Own(b) => {
                let c = b.counter.resolve()?;
                let h = &b.header;
                let a = answer_own(
                    c.as_counter(),
                    SecondaryPacket {
                        key: h.key(),
                        channel: h.channel(),
                        data_type: h.data_type(),
                        rtc: Rtc::from_raw((h.key().ticks & ((1i128 << 48) - 1)) as u64),
                        flags: h.flags(),
                        bytes: h.raw_bytes(),
                        byte_offset: h.byte_offset(),
                        applicability: h.applicability().clone(),
                    },
                    &b.context,
                    &b.policy,
                )?;
                let mut v = AnswerValue::empty();
                v.calendar = a.own().and_then(OwnPoint::native);
                v.counter_duration = Some(match &c {
                    ResolvedCounter::Offline(a) => a.counter_duration(),
                    ResolvedCounter::Stream(a) => a.counter_duration(),
                });
                v.point = a.own().cloned();
                v.own_minus_counter = a.delta_ns();
                v.tolerance = a.tolerance();
                v.issue = a.issue().cloned();
                v.findings = a.findings().to_vec();
                v.counter = Some(c);
                v
            }
            Self::Stamp(b) => {
                let i = &b.inspection;
                let fresh = inspect_stamp(
                    StampInput {
                        origin: i.origin().clone(),
                        query_id: i.query_id(),
                        flags: i.flags(),
                        bytes: i.raw(),
                        order: i.order(),
                    },
                    i.policy(),
                )?;
                let a = fresh.resolve(&b.context, |_| {
                    b.counter
                        .as_ref()
                        .ok_or(TimeError::NeedsContext {
                            context: "captured stamp counter",
                        })?
                        .resolve()
                })?;
                let mut v = AnswerValue::empty();
                v.calendar = a.calendar();
                v.counter_duration = a.counter_offset();
                v.counter = a.counter().cloned();
                v.point = a.point().cloned();
                v.issue = a.issue().cloned();
                v.findings = a.inspection().findings().to_vec();
                v
            }
            Self::Word(b) => {
                let a = answer_word(b.origin.clone(), b.word, &b.context, &b.policy)?;
                let mut v = AnswerValue::empty();
                v.calendar = a.calendar();
                v.point = a.point().cloned();
                v.issue = a.issue().cloned();
                v.findings = a.findings().to_vec();
                v.network = a.network().cloned();
                v.chapter4 = a.chapter4().cloned();
                v
            }
        };
        Ok(AnswerView {
            basis: self.clone(),
            value,
        })
    }
}

#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(CounterBasis { clock, findings });
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(OwnEnvelope {
    counter,
    header,
    context,
    policy
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(StampEnvelope {
    inspection,
    counter,
    context
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(WordEnvelope {
    origin,
    word,
    context,
    policy
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(AnswerValue {
    calendar,
    counter_duration,
    counter,
    point,
    own_minus_counter,
    tolerance,
    issue,
    findings,
    network,
    chapter4
});
#[cfg(feature = "serde")]
crate::snapshot::wide::serialize_struct!(AnswerView { basis, value });
