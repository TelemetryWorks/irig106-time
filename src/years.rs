//! Session year evidence: Fig11-13/14 (11-25/PDF33), §11.2.3.2a LY,
//! Ch9 Table9-4 R-x\RI4; contract5.3 / owner D9. No G\OD or host clock.
use crate::{Result, TimeError, findings::*, timeline::*};
use alloc::{collections::BTreeMap, vec::Vec};
/// Caller-extracted governing context for this reference. Years refer to the
/// session's first observed calendar year; detected rollovers advance them.
/// No TMATS defaults or configuration origination dates are consumed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct YearContext {
    /// Explicit caller year, after same-session DMY in priority.
    pub caller: Option<u16>,
    /// Governing R-x\RI4 recording year, lowest priority; not G\OD.
    pub recording: Option<u16>,
}
impl YearContext {
    fn validate(self) -> Result<()> {
        if self
            .caller
            .into_iter()
            .chain(self.recording)
            .any(|y| y > 9999)
        {
            return Err(TimeError::InvalidTimelineInput("year context exceeds9999"));
        }
        Ok(())
    }
}
/// One candidate at this reference's rollover position; original year retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearCandidate {
    /// Original year from packet/caller/RI4, before rollover adjustment.
    pub supplied: u16,
    /// Candidate year at the affected reference.
    pub year: u16,
    /// Source, never G\OD or LY-derived.
    pub origin: YearOrigin,
    /// DMY packet identity, where applicable.
    pub packet: Option<u64>,
}
/// Immutable per-reference evidence, copied into answer basis at release.
/// **Traces:** L3-COR-110 ← L2-COR-110 ← L1-COR-010
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct YearBasis {
    context: YearContext,
    candidates: Vec<YearCandidate>,
    original: Option<ReferenceValue>,
    original_origin: Option<YearOrigin>,
    phase: i32,
    resolved: bool,
    normalization_issue: Option<TimeError>,
    segment: usize,
    gap: Option<i128>,
    year_issue: Option<TimeError>,
}
impl YearBasis {
    // Relative calendar differences can be signed even without an absolute year,
    // but never across an unresolved year gap or a classified year transition.
    pub(crate) fn same_calendar_year(&self, other: &Self) -> bool {
        self.resolved
            && other.resolved
            && self.segment == other.segment
            && self.phase == other.phase
    }
    /// Exact governing caller/RI4 inputs, including contradictory ones.
    pub fn context(&self) -> YearContext {
        self.context
    }
    /// Highest priority candidate; None means partial/unknown year.
    pub fn selected(&self) -> Option<YearCandidate> {
        self.candidates.first().copied()
    }
    /// All applicable candidates in priority order, not silently overwritten.
    pub fn candidates(&self) -> &[YearCandidate] {
        &self.candidates
    }
    /// Original decoded value before inferred-year attachment.
    pub fn original_value(&self) -> Option<ReferenceValue> {
        self.original
    }
    /// A supplied normalization table could not qualify the selected year.
    /// Native capture value remains available; displaced physical answers need context.
    pub fn normalization_issue(&self) -> Option<&TimeError> {
        self.normalization_issue.as_ref()
    }
    /// Missing year context after a gap, independent of the decoded clock value.
    pub fn year_issue(&self) -> Option<&TimeError> {
        self.year_issue.as_ref()
    }
    /// Year-evidence continuity segment within this recording session.
    pub fn continuity_segment(&self) -> usize {
        self.segment
    }
    /// Unwrapped RTC ticks at the gap that broke this segment's year continuity.
    pub fn gap_ticks(&self) -> Option<crate::SignedRtcTicks> {
        self.gap.map(crate::SignedRtcTicks::new)
    }
    /// RTC-consistent year transitions relative to the first observed calendar.
    pub fn rollovers(&self) -> i32 {
        self.phase
    }
}
impl ReferenceInput {
    /// Attach exact governing context; each reference can carry a new RI4 snapshot.
    /// Same-session DMY wins; no date is inferred until an engine resolves years.
    /// **Traces:** L3-COR-110 ← L2-COR-110 ← L1-COR-010
    pub fn with_year_context(mut self, context: YearContext) -> Result<Self> {
        context.validate()?;
        self.year_basis.context = context;
        Ok(self)
    }
}
#[derive(Debug, Clone, Copy)]
struct Anchor {
    year: u16,
    phase: i32,
    packet: u64,
}
#[derive(Debug, Clone, Copy)]
struct Point {
    ticks: i128,
    day: u16,
    position: i128,
    year: Option<u16>,
    precision: u64,
    leap: bool,
}
const DAY_NS: i128 = 86_400_000_000_000;
/// Constant-size session clock/first-DMY anchor, independent of stream caps.
#[derive(Default)]
pub(crate) struct YearClock {
    last: Option<Point>,
    phase: i32,
    anchor: Option<Anchor>,
    latest: Option<Anchor>,
    segment: usize,
    gap: Option<i128>,
}
fn calendar(r: &TimelineReference) -> Option<(u16, Option<u16>)> {
    match r.input.year_basis.original.unwrap_or(r.input.value) {
        ReferenceValue::Readable(t, _) => Some((t.day_of_year(), t.year())),
        _ => r
            .input
            .wire
            .and_then(|(_, w)| w.time)
            .or_else(|| r.input.qualified.as_ref().and_then(|q| q.time))
            .map(|t| (t.day_of_year, t.year)),
    }
}
// These facts qualify for phase tracking, independently of ordinary leap-time
// admission. Non-admitted invalid/None/malformed observations remain skippable.
fn year_evidence(r: &TimelineReference) -> bool {
    if r.input.network.is_some() || calendar(r).is_none() {
        return false;
    }
    let body_year = r
        .input
        .wire
        .and_then(|(_, w)| w.time)
        .or_else(|| r.input.qualified.as_ref().and_then(|q| q.time))
        .and_then(|t| t.year);
    r.admitted
        || (matches!(r.input.value, ReferenceValue::NeedsContext)
            && body_year.is_some()
            && r.reasons
                .iter()
                .all(|reason| matches!(reason, Rejection::NeedsContext)))
}
impl YearClock {
    // A gap of365days can hide an entire ordinal-day cycle (or two year
    // boundaries near year end). Conservatively request context instead of
    // guessing how many years passed. This is a reader guard, not an IRIG rate.
    pub(crate) fn long_gap(&self, ticks: i128) -> Result<bool> {
        let Some(last) = self.last else {
            return Ok(false);
        };
        let elapsed = ticks
            .checked_sub(last.ticks)
            .ok_or(TimeError::QuantityOverflow {
                quantity: "year evidence counter gap",
            })?;
        Ok(elapsed.unsigned_abs() >= 365 * 86_400 * 10_000_000u128)
    }
    pub(crate) fn check_gap(&self, r: &TimelineReference) -> Result<()> {
        if year_evidence(r) && self.long_gap(r.key.ticks)? {
            return Err(TimeError::NeedsContext {
                context: "year evidence gap of365days or more; supply a new session/date context",
            });
        }
        Ok(())
    }
    // Call in observation order, separately from counter order used for phase.
    fn remember_anchor(&mut self, r: &TimelineReference) {
        if year_evidence(r)
            && r.input.year_basis.original_origin == Some(YearOrigin::Packet)
            && matches!(
                r.input.value,
                ReferenceValue::Readable(_, true) | ReferenceValue::NeedsContext
            )
        {
            if let Some((_, Some(year))) = calendar(r) {
                let anchor = Anchor {
                    year,
                    phase: r.input.year_basis.phase,
                    packet: r.key.packet,
                };
                self.anchor.get_or_insert(anchor);
                self.latest = Some(anchor);
            }
        }
    }
    fn point(&self, r: &TimelineReference, phase: i32) -> Option<Point> {
        let (day, raw_year) = calendar(r)?;
        let (position, leap) = match r.input.year_basis.original.unwrap_or(r.input.value) {
            ReferenceValue::Readable(t, _) => (i128::from(t.as_total_ns()), false),
            _ => {
                let t = r
                    .input
                    .wire
                    .and_then(|(_, w)| w.time)
                    .or_else(|| r.input.qualified.as_ref().and_then(|q| q.time))?;
                (
                    (i128::from(t.day_of_year) - 1) * DAY_NS
                        + i128::from(t.hours) * 3_600_000_000_000
                        + i128::from(t.minutes) * 60_000_000_000
                        + i128::from(t.seconds) * 1_000_000_000
                        + i128::from(t.nanoseconds),
                    t.is_leap_second(),
                )
            }
        };
        let adjust = |year: u16, delta: i32| {
            i32::from(year)
                .checked_add(delta)
                .filter(|y| (0..=9999).contains(y))
                .map(|y| y as u16)
        };
        let origin = r
            .input
            .year_basis
            .original_origin
            .unwrap_or(r.input.year_origin);
        let year = if origin == YearOrigin::Packet {
            raw_year
        } else {
            self.anchor
                .and_then(|a| {
                    phase
                        .checked_sub(a.phase)
                        .and_then(|delta| adjust(a.year, delta))
                })
                .or_else(|| raw_year.filter(|_| origin != YearOrigin::RecordingDate))
                .or_else(|| {
                    if self.segment == 0 {
                        r.input
                            .year_basis
                            .context
                            .caller
                            .and_then(|y| adjust(y, phase))
                    } else {
                        None
                    }
                })
                .or(raw_year)
                .or_else(|| {
                    if self.segment == 0 {
                        r.input
                            .year_basis
                            .context
                            .recording
                            .and_then(|y| adjust(y, phase))
                    } else {
                        None
                    }
                })
        };
        Some(Point {
            ticks: r.key.ticks,
            day,
            position,
            year,
            precision: r.input.precision_ns,
            leap,
        })
    }
    // Calendar consistency does not itself establish a year change: strict day
    // order and positive RTC remain separate guards (also for same-day late data).
    fn consistent(earlier: Point, later: Point, elapsed: i128, policy: &OfflinePolicy) -> bool {
        let tolerance = i128::from(
            policy
                .jump_threshold_ns
                .unwrap_or(earlier.precision.max(later.precision)),
        );
        [365u16, 366].into_iter().any(|days| {
            if earlier.day > days
                || earlier.year.is_some_and(|y| {
                    days != if crate::util::is_leap_year(y) {
                        366
                    } else {
                        365
                    }
                })
            {
                return false;
            }
            let mut span = later.position - earlier.position;
            if span < 0 {
                span += i128::from(days) * DAY_NS;
            }
            if earlier.leap && !later.leap {
                span += 1_000_000_000;
            }
            span >= 0 && (span - elapsed).unsigned_abs() <= tolerance as u128
        })
    }
    pub(crate) fn stamp(
        &mut self,
        r: &mut TimelineReference,
        policy: &OfflinePolicy,
    ) -> Result<()> {
        self.check_gap(r)?;
        if r.input.network.is_some() {
            return Ok(());
        }
        r.input.year_basis.segment = self.segment;
        r.input.year_basis.gap = self.gap;
        r.input.year_basis.original.get_or_insert(r.input.value);
        let body_year = r
            .input
            .wire
            .and_then(|(_, w)| w.time)
            .or_else(|| r.input.qualified.as_ref().and_then(|q| q.time))
            .and_then(|t| t.year);
        let original_origin = if body_year.is_some() {
            YearOrigin::Packet
        } else {
            r.input.year_origin
        };
        r.input
            .year_basis
            .original_origin
            .get_or_insert(original_origin);
        let Some(mut point) = self.point(r, self.phase) else {
            return Ok(());
        };
        let mut phase = self.phase;
        if year_evidence(r) {
            if let Some(last) = self.last {
                let ticks = last.ticks;
                if r.key.ticks >= ticks {
                    if r.key.ticks > ticks
                        && point.day < last.day
                        && Self::consistent(last, point, (r.key.ticks - ticks) * 100, policy)
                    {
                        self.phase =
                            self.phase
                                .checked_add(1)
                                .ok_or(TimeError::QuantityOverflow {
                                    quantity: "session year rollovers",
                                })?;
                    }
                    phase = self.phase;
                    point = self.point(r, phase).unwrap_or(point);
                    self.last = Some(point);
                } else if point.day > last.day {
                    let previous = self.point(r, phase - 1).unwrap_or(point);
                    if Self::consistent(previous, last, (ticks - r.key.ticks) * 100, policy) {
                        phase -= 1;
                    }
                }
            } else {
                self.last = Some(point);
            }
        }
        r.input.year_basis.phase = phase;
        self.remember_anchor(r);
        Ok(())
    }
    /// Resolve against evidence available now, preserving raw wire/year inputs.
    /// **Traces:** L3-COR-110 ← L2-COR-110 ← L1-COR-010
    /// **Traces:** L3-CSDW-104 ← L2-CSDW-104 ← L1-CSDW-004
    pub(crate) fn resolve(
        &self,
        r: &mut TimelineReference,
        policy: &OfflinePolicy,
    ) -> Result<Vec<TimelineFinding>> {
        if r.input.network.is_some() {
            return Ok(Vec::new());
        }
        let old = r.input.year_basis.clone();
        let Some((day, raw_year)) = calendar(r) else {
            return Ok(Vec::new());
        };
        let mut candidates = Vec::new();
        let phase = r.input.year_basis.phase;
        let mut add =
            |supplied: u16, adjust: i32, origin: YearOrigin, packet: Option<u64>| -> Result<()> {
                let year = i32::from(supplied)
                    .checked_add(adjust)
                    .filter(|y| (0..=9999).contains(y))
                    .ok_or(TimeError::NeedsContext {
                        context: "year rollover outside0..9999",
                    })? as u16;
                candidates.push(YearCandidate {
                    supplied,
                    year,
                    origin,
                    packet,
                });
                Ok(())
            };
        let origin = r
            .input
            .year_basis
            .original_origin
            .unwrap_or(r.input.year_origin);
        // Same-priority choice PROPOSAL: first observed eligible DMY anchors yearless
        // references; pending owner confirmation (verification-s08).
        // A DMY packet retains its own explicit calendar. Other references use
        // the first eligible DMY anchor; all disagreements remain candidates.
        if origin == YearOrigin::Packet {
            if let Some(y) = raw_year {
                add(y, 0, YearOrigin::Packet, Some(r.key.packet))?;
            }
        }
        if let Some(a) = self.anchor {
            if !(origin == YearOrigin::Packet && raw_year.is_some() && a.packet == r.key.packet) {
                add(
                    a.year,
                    phase
                        .checked_sub(a.phase)
                        .ok_or(TimeError::QuantityOverflow {
                            quantity: "year evidence phase difference",
                        })?,
                    YearOrigin::Packet,
                    Some(a.packet),
                )?;
            }
        }
        if let Some(a) = self.latest {
            if self.anchor.is_none_or(|first| first.packet != a.packet)
                && !(origin == YearOrigin::Packet && raw_year.is_some() && a.packet == r.key.packet)
            {
                add(
                    a.year,
                    phase
                        .checked_sub(a.phase)
                        .ok_or(TimeError::QuantityOverflow {
                            quantity: "year evidence phase difference",
                        })?,
                    YearOrigin::Packet,
                    Some(a.packet),
                )?;
            }
        }
        if origin != YearOrigin::Packet {
            if let Some(y) = raw_year {
                add(y, 0, origin, None)?;
            }
        }
        // A session-origin year cannot count unknown cycles across a long gap.
        // Keep the original context, but only a dated value/anchor in this segment
        // can restore an absolute year. LY never fills the missing context.
        if r.input.year_basis.segment == 0 {
            if let Some(y) = r.input.year_basis.context.caller {
                add(y, phase, YearOrigin::Caller, None)?;
            }
            if let Some(y) = r.input.year_basis.context.recording {
                add(y, phase, YearOrigin::RecordingDate, None)?;
            }
        }
        // An already dated legacy input retains its source's priority; an RI4
        // projection must not outrank explicit caller context merely by insertion.
        candidates.sort_by_key(|c| match c.origin {
            YearOrigin::Packet => 0,
            YearOrigin::RecordingDate => 2,
            _ => 1,
        });
        r.input.year_basis.candidates = candidates;
        r.input.year_basis.resolved = true;
        let selected = r.input.year_basis.selected();
        let selected_year = selected.map(|c| c.year);
        r.input.year_basis.year_issue =
            if selected_year.is_none() && r.input.year_basis.gap.is_some() {
                Some(TimeError::NeedsContext {
                    context: "year unknown after a long reference gap",
                })
            } else {
                None
            };
        r.input.year_origin = selected.map_or(YearOrigin::Unknown, |c| c.origin);
        let normalization_rebased = raw_year != selected_year && r.input.normalization.is_some();
        if selected_year.is_none() && r.input.normalization.is_some() {
            r.input.normalization = None;
            r.input.year_basis.normalization_issue = Some(TimeError::NeedsContext {
                context: "normalization year unknown after a reference gap",
            });
        }
        let malformed = selected_year.is_some_and(|y| day == 366 && !crate::util::is_leap_year(y));
        if let Some(ReferenceValue::Readable(t, valid)) = r.input.year_basis.original {
            match t.with_year(selected_year) {
                Ok(t) => {
                    r.input.value = ReferenceValue::Readable(t, valid);
                    if let Some(n) = r.input.normalization.as_ref().filter(|_| {
                        Some(t)
                            != match r.input.year_basis.original {
                                Some(ReferenceValue::Readable(t, _)) => Some(t),
                                _ => None,
                            }
                    }) {
                        // Never retain normalization computed for a superseded year.
                        match crate::reference::Normalization::new(
                            t,
                            r.input.scale,
                            n.table().clone(),
                        ) {
                            Ok(n) => {
                                r.input.normalization = Some(n);
                                r.input.year_basis.normalization_issue = None;
                            }
                            Err(error) => {
                                r.input.normalization = None;
                                r.input.year_basis.normalization_issue = Some(error);
                            }
                        }
                    }
                }
                Err(_) => {
                    r.input.value = ReferenceValue::Malformed;
                    r.admitted = false;
                    if !r.reasons.contains(&Rejection::Malformed) {
                        r.reasons.push(Rejection::Malformed);
                    }
                }
            }
        }
        if old == r.input.year_basis {
            return Ok(Vec::new());
        }
        let ly = r.input.raw_csdw.map(|raw| raw & 0x100 != 0);
        let mismatch = ly.is_some_and(|ly| {
            day == 366 && !ly || selected_year.is_some_and(|y| ly != crate::util::is_leap_year(y))
        });
        let conflict = selected.is_some_and(|winner| {
            r.input
                .year_basis
                .candidates
                .iter()
                .any(|c| c.year != winner.year)
        });
        let mut out = Vec::new();
        if normalization_rebased {
            if let Some(f) =
                crate::reference::normalization_warning(&r.input, r.key, r.channel, policy)?
            {
                out.push(f);
            }
        }
        for (code, needed) in [
            (32, conflict),
            (7, mismatch),
            (4, malformed),
            (20, selected.is_none()),
        ] {
            if !needed {
                continue;
            }
            let mut f = TimelineFinding::observed(
                code,
                policy.finding_levels.get(&code).copied(),
                r.key,
                Some(r.channel),
                selected
                    .and_then(|c| c.packet)
                    .filter(|p| *p != r.key.packet),
                None,
            )?;
            let mut evidence = FindingEvidence::new(
                f.evidence.location(),
                if malformed {
                    ValueValidity::Unusable
                } else {
                    ValueValidity::Kept
                },
            )
            .with_raw_rtc(crate::Rtc::from_raw(
                (r.key.ticks & ((1i128 << 48) - 1)) as u64,
            ));
            for c in &r.input.year_basis.candidates {
                let name = match c.origin {
                    YearOrigin::Packet => "year.packet",
                    YearOrigin::RecordingDate => "year.ri4",
                    _ => "year.caller",
                };
                if let Some(packet) = c.packet.filter(|p| *p != r.key.packet) {
                    let location = evidence.location();
                    evidence = evidence.with_related(FindingLocation {
                        packet: Some(packet),
                        ..location
                    });
                }
                evidence = evidence.with_raw_field(RawField {
                    name,
                    observed: RawValue::Unsigned(c.year.into()),
                    expected: selected_year.map(|y| RawValue::Unsigned(y.into())),
                });
            }
            if let Some(y) = selected_year {
                evidence = evidence.with_raw_field(RawField {
                    name: "year.selected",
                    observed: RawValue::Unsigned(y.into()),
                    expected: None,
                });
            }
            if let Some(gap) = r.input.year_basis.gap {
                evidence = evidence.with_raw_field(RawField {
                    name: "year.gap_ticks",
                    observed: RawValue::Signed(gap),
                    expected: None,
                });
                for (name, value) in [
                    ("year.session_caller", r.input.year_basis.context.caller),
                    ("year.session_ri4", r.input.year_basis.context.recording),
                ] {
                    if let Some(value) = value {
                        evidence = evidence.with_raw_field(RawField {
                            name,
                            observed: RawValue::Unsigned(value.into()),
                            expected: None,
                        });
                    }
                }
            }
            if let Some(ly) = ly {
                evidence = evidence.with_raw_field(RawField {
                    name: "ly",
                    observed: RawValue::Unsigned(ly.into()),
                    expected: None,
                });
            }
            f.evidence = evidence;
            out.push(f);
        }
        Ok(out)
    }
}
pub(crate) fn resolve_session(
    refs: &mut [TimelineReference],
    policy: &OfflinePolicy,
) -> Result<Vec<TimelineFinding>> {
    let mut order: Vec<_> = (0..refs.len()).collect();
    order.sort_by_key(|&i| (refs[i].key.ticks, refs[i].key.packet));
    let mut clock = YearClock::default();
    for &i in &order {
        // Offline keeps the timeline usable. Break only year continuity, not the
        // recording session or the decoded clock, including intervening unusable
        // calendar observations that cannot restore the missing year.
        if refs[i].input.network.is_none()
            && calendar(&refs[i]).is_some()
            && clock.long_gap(refs[i].key.ticks)?
        {
            let gap = clock
                .last
                .and_then(|last| refs[i].key.ticks.checked_sub(last.ticks));
            clock = YearClock {
                segment: clock
                    .segment
                    .checked_add(1)
                    .ok_or(TimeError::QuantityOverflow {
                        quantity: "year continuity segments",
                    })?,
                gap,
                ..YearClock::default()
            };
        }
        clock.stamp(&mut refs[i], policy)?;
    }
    let mut groups = BTreeMap::<usize, Vec<usize>>::new();
    for (i, r) in refs
        .iter()
        .enumerate()
        .filter(|(_, r)| r.input.network.is_none())
    {
        groups
            .entry(r.input.year_basis.segment)
            .or_default()
            .push(i);
    }
    let mut findings = Vec::new();
    for (segment, ids) in groups {
        let origin = ids
            .iter()
            .find(|&&i| year_evidence(&refs[i]))
            .map_or(0, |&i| refs[i].input.year_basis.phase);
        let mut resolver = YearClock {
            segment,
            ..YearClock::default()
        };
        for &i in &ids {
            refs[i].input.year_basis.phase =
                refs[i].input.year_basis.phase.checked_sub(origin).ok_or(
                    TimeError::QuantityOverflow {
                        quantity: "year evidence phase origin",
                    },
                )?;
            resolver.remember_anchor(&refs[i]);
        }
        for i in ids {
            findings.extend(resolver.resolve(&mut refs[i], policy)?);
        }
    }
    Ok(findings)
}
