//! Constant-memory native quality for already-qualified, RTC-ordered references.
//! No admission, year inference, reordering, findings emission or file/clock I/O.
//! Counters/density/drift follow contract3.6/5.4 and OfflineTimeline::quality.
//! The standard supplies the free-running RTC (§11.2.1.1 i) and reference rate
//! (§11.2.3.2/3.3); statistics are reader analysis, not wire fields or gap findings.
use crate::{
    Result, TimeError,
    format1::CalendarReading,
    scale::TimeScale,
    timeline::{OfflineQuality, PacketKey, ReferenceValue, TimelineReference},
};

/// Native per-session/per-channel quality without retaining reference history.
///
/// Feed unique engine-qualified references sorted by (unwrapped ticks, packet).
/// Late-reference sorting and year/admission/normalization are caller prerequisites;
/// this accumulator never repairs or retroactively requalifies them. It retains
/// only first/last valid reference snapshots (including supplied provenance).
/// Caller-owned policy/table sizes determine snapshot size; there is no queue.
/// Source: contract3.6/5.4;106-24R1 §11.2.1.1 i and §11.2.3.3.
/// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
#[derive(Debug, Clone)]
pub struct OrderedQuality {
    session: usize,
    channel: u16,
    order: Option<PacketKey>,
    first: Option<TimelineReference>,
    last: Option<TimelineReference>,
    counts: OfflineQuality,
    duplicate: bool,
    normalized: bool,
    same_scale: bool,
    context: bool,
    missing_delta: bool,
    drift_sum: f64,
}
impl OrderedQuality {
    /// Bind a nonzero session and one channel; does not choose a time source.
    /// Source: 106-24R1 §11.2.1.1 i; contract3.6/5.4.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn new(session: usize, channel: u16) -> Result<Self> {
        if session == 0 {
            return Err(TimeError::InvalidTimelineInput("quality session zero"));
        }
        Ok(Self {
            session,
            channel,
            order: None,
            first: None,
            last: None,
            counts: OfflineQuality {
                valid: 0,
                rejected: 0,
                admitted: 0,
                interval_ns: None,
                min_spacing_ns: None,
                max_spacing_ns: None,
                references_per_second: None,
                drift_ppm: None,
                unavailable: Some("fewer than two valid references"),
            },
            duplicate: false,
            normalized: true,
            same_scale: true,
            context: false,
            missing_delta: false,
            drift_sum: 0.,
        })
    }
    /// Actual session/channel scope, not a data-channel association.
    /// Source: 106-24R1 §11.2.1.1 i; contract3.6.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn scope(&self) -> (usize, u16) {
        (self.session, self.channel)
    }
    /// Observe one prepared reference; wrong scope/order/overflow leaves state
    /// intact. Duplicate RTC with distinct packet IDs counts but disables drift.
    /// Valid means admitted Readable(_,true), exactly as native offline quality;
    /// readable invalids admitted by override do not become valid measurements.
    /// Source: 106-24R1 §11.2.1.1 i/§11.2.3.3; contract3.6/5.4.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn observe(&mut self, r: &TimelineReference) -> Result<()> {
        if r.key.session != self.session {
            return Err(TimeError::CrossSessionCounterDelta);
        }
        if r.channel != self.channel
            || self
                .order
                .is_some_and(|p| (r.key.ticks, r.key.packet) <= (p.ticks, p.packet))
        {
            return Err(TimeError::InvalidTimelineInput(
                "quality channel/order mismatch",
            ));
        }
        let mut counts = self.counts.clone();
        if r.admitted {
            counts.admitted = add(counts.admitted, 1)?;
        } else {
            counts.rejected = add(counts.rejected, 1)?;
        }
        let valid = r.admitted && matches!(r.input.value, ReferenceValue::Readable(_, true));
        let (mut duplicate, mut missing, mut sum) =
            (self.duplicate, self.missing_delta, self.drift_sum);
        if valid {
            counts.valid = add(counts.valid, 1)?;
            if let (Some(first), Some(last)) = (&self.first, &self.last) {
                counts.interval_ns = Some(interval(first.key, r.key)?);
                let gap = interval(last.key, r.key)?;
                counts.min_spacing_ns = Some(counts.min_spacing_ns.map_or(gap, |v| v.min(gap)));
                counts.max_spacing_ns = Some(counts.max_spacing_ns.map_or(gap, |v| v.max(gap)));
                if gap == 0 {
                    duplicate = true;
                } else if let Some(delta) = crate::timeline::absolute_delta(last, r) {
                    sum += 1e6 * (delta as f64 / gap as f64 - 1.);
                } else {
                    missing = true;
                }
            }
        }
        self.counts = counts;
        self.order = Some(r.key);
        self.duplicate = duplicate;
        self.missing_delta = missing;
        self.drift_sum = sum;
        if valid {
            let scale = r.input.scale.effective();
            self.same_scale &= scale != TimeScale::Unknown
                && self
                    .first
                    .as_ref()
                    .is_none_or(|f| f.input.scale.effective() == scale);
            self.normalized &= r.input.normalization.is_some();
            self.context |= r.input.normalization.is_some() || r.input.network.is_some();
            if self.first.is_none() {
                self.first = Some(r.clone());
            }
            self.last = Some(r.clone());
        }
        Ok(())
    }
    /// Count classified decode failures with no retained reference (native
    /// offline's extra TF-004 cases). Caller must not also submit those as refs.
    /// No finding severity is inferred; overflow leaves counts unchanged.
    /// Source: 106-24R1 Figs11-13..17; contract3.6/6.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn add_decode_rejections(&mut self, count: usize) -> Result<()> {
        self.counts.rejected = add(self.counts.rejected, count)?;
        Ok(())
    }
    /// All-input counts and measured valid-reference interval; absence is
    /// distinct from zero. Drift shares native scale/year/leap delta arithmetic.
    /// Source: 106-24R1 §11.2.1.1 i/§11.2.3.3; contract3.6/5.4.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn snapshot(&self) -> OfflineQuality {
        let mut q = self.counts.clone();
        if q.valid < 2 {
            return q;
        }
        if self.duplicate {
            q.min_spacing_ns = None;
            q.max_spacing_ns = None;
            q.unavailable = Some("duplicate RTC");
            return q;
        }
        q.references_per_second = q.interval_ns.map(|n| q.valid as f64 / (n as f64 / 1e9));
        if !self.normalized && !self.same_scale {
            q.unavailable = Some("unknown or incompatible scales");
        } else if self.missing_delta {
            q.unavailable = Some(if self.context {
                "scale normalization or leap context"
            } else {
                "calendar/year context"
            });
        } else {
            q.drift_ppm = Some(self.drift_sum / (q.valid - 1) as f64);
            q.unavailable = None;
        }
        q
    }
    /// First valid native reference by RTC order, with its original provenance.
    /// Source: 106-24R1 Figs11-13..17; contract3.6/ADR-0011.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn first_reference(&self) -> Option<&TimelineReference> {
        self.first.as_ref()
    }
    /// Last valid native reference by RTC order; not maximum calendar value.
    /// Source: 106-24R1 Figs11-13..17; contract3.6/ADR-0011.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn last_reference(&self) -> Option<&TimelineReference> {
        self.last.as_ref()
    }
    /// First/last valid calendars; unknown years/scales remain labelled partials.
    /// None means no valid references; Err means calendar context unavailable.
    /// These endpoints do not assert continuity or provide a physical duration.
    /// Network/table/era provenance stays on first_reference/last_reference.
    /// Source: 106-24R1 Figs11-13..17; RDG-009/010/012; contract3.6/ADR-0011.
    /// **Traces:** L3-COR-113 ← L2-COR-113 ← L1-COR-013
    pub fn calendar_endpoints(&self) -> Result<Option<(CalendarReading, CalendarReading)>> {
        match (&self.first, &self.last) {
            (Some(first), Some(last)) => Ok(Some((
                crate::reference::shifted_reference(first, 0)?,
                crate::reference::shifted_reference(last, 0)?,
            ))),
            _ => Ok(None),
        }
    }
}
fn add(n: usize, m: usize) -> Result<usize> {
    n.checked_add(m).ok_or(TimeError::QuantityOverflow {
        quantity: "quality count",
    })
}
fn interval(a: PacketKey, b: PacketKey) -> Result<i128> {
    b.ticks
        .checked_sub(a.ticks)
        .and_then(|n| n.checked_mul(100))
        .ok_or(TimeError::QuantityOverflow {
            quantity: "quality counter interval",
        })
}
