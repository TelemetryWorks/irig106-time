//! Session year evidence: Fig11-13/14 (11-25/PDF33), §11.2.3.2a LY,
//! Ch9 Table9-4 R-x\RI4; contract5.3 / owner D9. No G\OD or host clock.
use crate::{Result, TimeError, findings::*, timeline::*};
use alloc::vec::Vec;
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
}
impl YearBasis {
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
    /// Observed Dec31/Jan1 transitions relative to session start.
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
/// Constant-size session clock/first-DMY anchor, independent of stream caps.
#[derive(Default)]
pub(crate) struct YearClock {
    last: Option<(i128, u16)>,
    phase: i32,
    anchor: Option<Anchor>,
    latest: Option<Anchor>,
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
impl YearClock {
    pub(crate) fn stamp(&mut self, r: &mut TimelineReference) -> Result<()> {
        if r.input.network.is_some() {
            return Ok(());
        }
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
        let Some((day, year)) = calendar(r) else {
            return Ok(());
        };
        let mut phase = self.phase;
        // A validated leap-label body can supply a DMY year without being a
        // usable ordinary time reference. Other rejection reasons still block it.
        let readable_year = matches!(r.input.value, ReferenceValue::NeedsContext)
            && body_year.is_some()
            && r.reasons
                .iter()
                .all(|reason| matches!(reason, Rejection::NeedsContext));
        if r.admitted || readable_year {
            if let Some((ticks, last)) = self.last {
                if r.key.ticks >= ticks {
                    if last >= 365 && day == 1 {
                        self.phase =
                            self.phase
                                .checked_add(1)
                                .ok_or(TimeError::QuantityOverflow {
                                    quantity: "session year rollovers",
                                })?;
                    }
                    phase = self.phase;
                    self.last = Some((r.key.ticks, day));
                } else if last == 1 && day >= 365 {
                    phase -= 1;
                }
            } else {
                self.last = Some((r.key.ticks, day));
            }
            if r.input.year_basis.original_origin == Some(YearOrigin::Packet)
                && matches!(
                    r.input.value,
                    ReferenceValue::Readable(_, true) | ReferenceValue::NeedsContext
                )
            {
                if let Some(year) = year {
                    let anchor = Anchor {
                        year,
                        phase,
                        packet: r.key.packet,
                    };
                    self.anchor.get_or_insert(anchor);
                    self.latest = Some(anchor);
                }
            }
        }
        r.input.year_basis.phase = phase;
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
        // Same-priority choice PROPOSAL: first eligible DMY anchors yearless
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
                add(a.year, phase - a.phase, YearOrigin::Packet, Some(a.packet))?;
            }
        }
        if let Some(a) = self.latest {
            if self.anchor.is_none_or(|first| first.packet != a.packet)
                && !(origin == YearOrigin::Packet && raw_year.is_some() && a.packet == r.key.packet)
            {
                add(a.year, phase - a.phase, YearOrigin::Packet, Some(a.packet))?;
            }
        }
        if origin != YearOrigin::Packet {
            if let Some(y) = raw_year {
                add(y, 0, origin, None)?;
            }
        }
        if let Some(y) = r.input.year_basis.context.caller {
            add(y, phase, YearOrigin::Caller, None)?;
        }
        if let Some(y) = r.input.year_basis.context.recording {
            add(y, phase, YearOrigin::RecordingDate, None)?;
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
        r.input.year_origin = selected.map_or(YearOrigin::Unknown, |c| c.origin);
        let normalization_rebased = raw_year != selected_year && r.input.normalization.is_some();
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
        clock.stamp(&mut refs[i])?;
    }
    let mut findings = Vec::new();
    for i in order {
        findings.extend(clock.resolve(&mut refs[i], policy)?);
    }
    Ok(findings)
}
