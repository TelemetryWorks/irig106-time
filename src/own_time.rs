//! Explicit own-time interpretation after S10a1 trust validation.
//! Ch11 §11.2.1.1g/1.2a, Figs11-4..6 (11-9/10, PDF17/18), contract6.6.
//! Neither an IEEE-shaped word nor a relative ERTC establishes a Unix epoch.
use crate::{
    ch4::{Ch4Value, Ch4Words},
    findings::*,
    format1::CalendarReading,
    leap::{LeapSecondTable, PtpConversionBasis, PtpConversionStatus},
    scale::*,
    secondary::SecondaryHeaderTime,
    secondary_packet::*,
    stream::StreamAnswer,
    timeline::*,
    *,
};
use alloc::{collections::BTreeMap, string::String, vec::Vec};
const SECOND: i128 = 1_000_000_000;
fn missing(context: &'static str) -> TimeError {
    TimeError::NeedsContext { context }
}
fn overflow() -> TimeError {
    TimeError::QuantityOverflow {
        quantity: "own-time arithmetic",
    }
}
/// Caller-defined meaning of elapsed epoch seconds; no default interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EpochCounting {
    /// Continuous SI time, including UTC insertions when crossing them.
    Continuous,
    /// Nominal86400-second civil days; a repeated insertion-window coordinate
    /// cannot uniquely identify a physical instant and remains unresolved.
    Civil,
}
/// Explicit dated epoch, scale, counting convention and nonempty caller citation.
/// Scale is labelled Policy on the own result even if the origin was stated UTC.
/// **Traces:** L3-ABS-103 ← L2-ABS-103 ← L1-ABS-003
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochContext {
    /// Dated origin, not a nearby-packet calendar guess.
    pub origin: CalendarReading,
    /// Explicit interpretation of elapsed seconds.
    pub counting: EpochCounting,
    /// Nonempty caller provenance, not treated as verified standard wording.
    pub source: String,
}
/// Immutable snapshot copied into the result; no host clock or hidden epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnContext {
    /// IEEE time-word epoch/scale; absence leaves the decoded words partial.
    pub ieee: Option<EpochContext>,
    /// Chapter4 binary origin; no binary/BCD day split or nearest-packet option.
    pub ch4: Option<EpochContext>,
    /// Validated table, including identity and coverage. Default is the pinned
    /// S04 IERS snapshot, not a network fetch or a claim of current coverage.
    pub table: LeapSecondTable,
    /// Instant described by the counter answer; comparison requires an exact
    /// match to the header's first-bit or cited-exception applicability.
    pub counter_applicability: TimeApplicability,
}
impl Default for OwnContext {
    fn default() -> Self {
        Self {
            ieee: None,
            ch4: None,
            table: LeapSecondTable::builtin(),
            counter_applicability: TimeApplicability::FirstDataBit,
        }
    }
}
/// Comparison policy independent of the original timeline's retained policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OwnPolicy {
    /// None uses the counter reference resolution (coarser bracket endpoint
    /// for interpolated stream answers), contract6.6. Strictly greater emits14.
    pub tolerance: Option<NanosDuration>,
    /// Catalog severity overrides, never trust/epoch/scale overrides.
    pub levels: BTreeMap<u8, Severity>,
}
/// Borrowed, unchanged released answer; no session/channel/reference reselection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterAnswer<'a> {
    /// Offline nearest/selected-reference answer with its original basis.
    Offline(&'a OfflineAnswer),
    /// Stream answer, including mode/fallback/drift/old-policy drain basis.
    Stream(&'a StreamAnswer),
}
impl<'a> CounterAnswer<'a> {
    /// Original offline answer if applicable.
    pub fn offline(self) -> Option<&'a OfflineAnswer> {
        if let Self::Offline(a) = self {
            Some(a)
        } else {
            None
        }
    }
    /// Original stream answer if applicable.
    pub fn stream(self) -> Option<&'a StreamAnswer> {
        if let Self::Stream(a) = self {
            Some(a)
        } else {
            None
        }
    }
    /// Released packet identity.
    pub fn key(self) -> PacketKey {
        match self {
            Self::Offline(a) => a.basis.key,
            Self::Stream(a) => a.basis.key,
        }
    }
    fn reference(self) -> Option<&'a TimelineReference> {
        match self {
            Self::Offline(a) => a.basis.reference.as_ref(),
            Self::Stream(a) => a.basis.preceding.as_ref(),
        }
    }
    pub(crate) fn reading(self) -> Option<CalendarReading> {
        match self {
            Self::Offline(a) => a.reading,
            Self::Stream(a) => a.reading,
        }
    }
    fn precision(self) -> Option<u64> {
        match self {
            Self::Offline(a) => a.basis.reference.as_ref().map(|r| r.input.precision_ns),
            Self::Stream(a) => a.basis.precision_ns,
        }
    }
    pub(crate) fn normalized(self, table: &LeapSecondTable) -> Result<i128> {
        let r = self.reference().ok_or(missing("counter reference"))?;
        if r.key.session != self.key().session {
            return Err(missing("counter reference session"));
        }
        // An ambiguous/unconverted network value cannot become a unique instant
        // merely by normalizing its retained display calendar a second time.
        if r.input.network.is_some() && r.input.normalization.is_none() {
            return Err(missing("unique network counter instant"));
        }
        normalize(self.reading().ok_or(missing("counter calendar"))?, table)
    }
    pub(crate) fn ertc_shift(self, ns: i128) -> Result<i128> {
        if let Self::Stream(a) = self {
            if let (Some(p), Some(f)) = (&a.basis.preceding, &a.basis.following) {
                if p.key.session != f.key.session || p.key.session != self.key().session {
                    return Err(missing("ERTC bracket session"));
                }
                let span = f
                    .key
                    .ticks
                    .checked_sub(p.key.ticks)
                    .and_then(|n| n.checked_mul(100))
                    .filter(|n| *n > 0)
                    .ok_or(missing("ERTC bracket span"))?;
                let absolute = absolute_delta(p, f).ok_or(missing("ERTC bracket scale"))?;
                // Difference of floors, not floor of the difference: preserves
                // the exact released interpolation and its sub-tick phase.
                let base = self
                    .key()
                    .ticks
                    .checked_sub(p.key.ticks)
                    .and_then(|n| n.checked_mul(100))
                    .ok_or_else(overflow)?;
                let at = base
                    .checked_add(ns)
                    .and_then(|n| n.checked_mul(absolute))
                    .ok_or_else(overflow)?
                    .div_euclid(span);
                let before = base
                    .checked_mul(absolute)
                    .ok_or_else(overflow)?
                    .div_euclid(span);
                return at.checked_sub(before).ok_or_else(overflow);
            }
        }
        Ok(ns)
    }
}
/// Interpreted own instant; raw words remain in the separate inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnPoint {
    tai: SignedNanos,
    native: Option<CalendarReading>,
    utc: OwnUtc,
}
impl OwnPoint {
    /// Signed continuous TAI nanoseconds since1970, checked without saturation.
    pub fn tai_ns(&self) -> SignedNanos {
        self.tai
    }
    /// Calendar in the caller-assigned native scale; absent if missing UTC offset.
    pub fn native(&self) -> Option<CalendarReading> {
        self.native
    }
    /// S04 conversion basis, including raw derived TAI, actual offset and currency.
    /// Its TAI operand is derived from the header/context, not a Format2 packet.
    pub fn utc_conversion(&self) -> &OwnUtc {
        &self.utc
    }
}
/// UTC projection of a signed own instant, retaining D7 stale-offset basis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnUtc {
    reading: Option<CalendarReading>,
    status: PtpConversionStatus,
    basis: PtpConversionBasis,
}
impl OwnUtc {
    /// UTC including second60; None when no applicable offset is known.
    pub fn reading(&self) -> Option<CalendarReading> {
        self.reading
    }
    /// Covered, last-known possibly stale, or missing offset.
    pub fn status(&self) -> PtpConversionStatus {
        self.status
    }
    /// Immutable table identity/coverage and actual selected offset/transition.
    pub fn basis(&self) -> &PtpConversionBasis {
        &self.basis
    }
}
fn utc_projection(tai: i128, table: &LeapSecondTable) -> Result<OwnUtc> {
    let mut out = OwnUtc {
        reading: None,
        status: PtpConversionStatus::MissingOffset,
        basis: PtpConversionBasis {
            table: table.metadata().clone(),
            final_transition: table.final_transition(),
            offset: None,
            selected_transition: None,
            leap_second: false,
        },
    };
    let Ok(seconds) = u64::try_from(tai.div_euclid(SECOND)) else {
        return Ok(out);
    };
    let seconds = TaiSeconds::new(seconds);
    let Ok(offset) = table.offset_at_tai(seconds) else {
        return Ok(out);
    };
    let unix = seconds.to_signed_unix(offset);
    let boundary = |e: &crate::leap::LeapSecondEntry| {
        i128::from(e.effective_unix.get()) + i128::from(e.tai_utc_offset.get())
    };
    let selected = table
        .entries()
        .iter()
        .rposition(|e| boundary(e) <= i128::from(seconds.get()));
    let leap = selected
        .and_then(|i| table.entries().get(i + 1))
        .is_some_and(|e| {
            i64::from(e.tai_utc_offset.get()) - i64::from(offset.get()) == 1
                && i128::from(seconds.get()) + 1 == boundary(e)
        });
    out.reading = Some(crate::leap::calendar(
        unix,
        tai.rem_euclid(SECOND) as u32,
        leap,
    )?);
    if let Some(c) = out.reading.as_mut() {
        c.scale = ScaleEvidence::assumed(TimeScale::Utc);
    }
    out.status = if table.covers_unix(unix) {
        PtpConversionStatus::Covered
    } else {
        PtpConversionStatus::PossiblyOutOfDate
    };
    out.basis.offset = Some(offset);
    out.basis.selected_transition = selected.map(|i| table.entries()[i]);
    out.basis.leap_second = leap;
    Ok(out)
}
/// Own and independent counter result with immutable context, policy and findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnAnswer<'a> {
    counter: CounterAnswer<'a>,
    header: SecondaryInspection,
    context: OwnContext,
    policy: OwnPolicy,
    own: Option<OwnPoint>,
    delta: Option<SignedNanos>,
    tolerance: Option<NanosDuration>,
    issue: Option<TimeError>,
    findings: Vec<Finding>,
}
impl<'a> OwnAnswer<'a> {
    /// Exact borrowed counter result, including its original table/policy basis.
    pub fn counter(&self) -> CounterAnswer<'a> {
        self.counter
    }
    /// Checksum trust, raw relative values and independent reserved/RTC evidence.
    pub fn header(&self) -> &SecondaryInspection {
        &self.header
    }
    /// Epoch, scale/counting provenance and table actually used by this adapter.
    pub fn context(&self) -> &OwnContext {
        &self.context
    }
    /// Comparison tolerance/override snapshot.
    pub fn policy(&self) -> &OwnPolicy {
        &self.policy
    }
    /// Absolute own instant only when the needed epoch/session context exists.
    pub fn own(&self) -> Option<&OwnPoint> {
        self.own.as_ref()
    }
    /// Signed own minus counter, or None for unresolved/incompatible context.
    pub fn delta_ns(&self) -> Option<SignedNanos> {
        self.delta
    }
    /// Effective comparison tolerance, if a reference resolution/override exists.
    pub fn tolerance(&self) -> Option<NanosDuration> {
        self.tolerance
    }
    /// Missing context or conversion failure; never substituted by a zero delta.
    pub fn issue(&self) -> Option<&TimeError> {
        self.issue.as_ref()
    }
    /// Header findings plus conversion/comparison findings; originals unchanged.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}
/// Encode only the eight-byte IEEE time word, not a complete secondary/stamp API.
/// Ch11 Fig11-5: LSLW nanoseconds first, MSLW seconds second; unlike Format2.
/// **Traces:** L3-ABS-103 ← L2-ABS-103 ← L1-ABS-003
pub fn encode_ieee_time(t: Ieee1588Time) -> Result<[u8; 8]> {
    if t.nanoseconds >= 1_000_000_000 {
        return Err(TimeError::OutOfRange {
            field: "ieee_nanoseconds",
            value: t.nanoseconds,
            max: 999_999_999,
        });
    }
    let mut out = [0; 8];
    out[..4].copy_from_slice(&t.nanoseconds.to_le_bytes());
    out[4..].copy_from_slice(&t.seconds.to_le_bytes());
    Ok(out)
}
// Calendar coordinate is explicitly caller-assigned, never a wire epoch claim.
fn nominal(c: CalendarReading) -> Result<i128> {
    c.validate()?;
    let year = c.year.ok_or(missing("dated own calendar"))?;
    let days = |y: u16| -> i128 {
        (0..y)
            .map(|y| {
                if crate::util::is_leap_year(y) {
                    366
                } else {
                    365
                }
            })
            .sum()
    };
    let date = days(year) - days(1970) + i128::from(c.day_of_year) - 1;
    Ok(((date * 86400
        + i128::from(c.hours) * 3600
        + i128::from(c.minutes) * 60
        + i128::from(c.seconds))
        * SECOND)
        + i128::from(c.nanoseconds))
}
pub(crate) fn normalize(c: CalendarReading, table: &LeapSecondTable) -> Result<i128> {
    let ns = nominal(c)?;
    let offset = match c.scale.effective() {
        TimeScale::Tai => 0,
        TimeScale::Gps => 19,
        TimeScale::Unknown => return Err(missing("own/counter scale")),
        TimeScale::Utc => {
            let sec = ns.div_euclid(SECOND) - i128::from(c.seconds == 60);
            let sec = u64::try_from(sec).map_err(|_| missing("UTC before table support"))?;
            table.offset_at_unix(UnixSeconds::new(sec))?.get()
        }
    };
    let tai = ns + i128::from(offset) * SECOND;
    if c.scale.effective() == TimeScale::Utc {
        let p = utc_projection(tai, table)?;
        let mut roundtrip = p.reading.ok_or(missing("UTC offset"))?;
        roundtrip.scale = c.scale;
        if roundtrip != c {
            return Err(missing("calendar is not a table-supported UTC instant"));
        }
    }
    Ok(tai)
}
pub(crate) fn point(tai: i128, scale: TimeScale, table: &LeapSecondTable) -> Result<OwnPoint> {
    let utc = utc_projection(tai, table)?;
    let mut native = match scale {
        TimeScale::Utc => utc.reading,
        TimeScale::Tai | TimeScale::Gps => {
            let ns = tai
                - if scale == TimeScale::Gps {
                    19 * SECOND
                } else {
                    0
                };
            Some(crate::leap::calendar(
                SignedUnixSeconds::new(ns.div_euclid(SECOND)),
                ns.rem_euclid(SECOND) as u32,
                false,
            )?)
        }
        TimeScale::Unknown => return Err(missing("own scale")),
    };
    if let Some(c) = native.as_mut() {
        c.scale = ScaleEvidence::assumed(scale);
    }
    Ok(OwnPoint {
        tai: SignedNanos::new(tai),
        native,
        utc,
    })
}
pub(crate) fn from_epoch(
    elapsed: i128,
    epoch: Option<&EpochContext>,
    table: &LeapSecondTable,
) -> Result<OwnPoint> {
    let epoch = epoch.ok_or(missing("explicit own-time epoch and scale"))?;
    if epoch.source.trim().is_empty() {
        return Err(missing("epoch source citation"));
    }
    let tai = match epoch.counting {
        EpochCounting::Continuous => normalize(epoch.origin, table)?
            .checked_add(elapsed)
            .ok_or_else(overflow)?,
        EpochCounting::Civil => {
            if epoch.origin.seconds == 60 {
                return Err(missing(
                    "civil epoch cannot select repeated leap coordinate",
                ));
            }
            let n = nominal(epoch.origin)?
                .checked_add(elapsed)
                .ok_or_else(overflow)?;
            let mut c = crate::leap::calendar(
                SignedUnixSeconds::new(n.div_euclid(SECOND)),
                n.rem_euclid(SECOND) as u32,
                false,
            )?;
            c.scale = epoch.origin.scale;
            if c.scale.effective() == TimeScale::Utc
                && table.entries().windows(2).any(|w| {
                    w[1].tai_utc_offset.get() - w[0].tai_utc_offset.get() == 1
                        && i128::from(w[1].effective_unix.get()) == n.div_euclid(SECOND)
                })
            {
                return Err(missing("civil UTC insertion coordinate is ambiguous"));
            }
            normalize(c, table)?
        }
    };
    point(tai, epoch.origin.scale.effective(), table)
}
/// Interpret trusted own time beside an already released offline/stream answer.
/// ERTC uses that named session's calibration and its released interpolation
/// slope. No reference search, epoch guess, revision, or fallback to bad headers.
/// Comparison normalizes the ACTUAL released counter calendar with this explicit
/// table; its original table/policy remains accessible in `counter()`.
/// **Traces:** L3-ANS-102 ← L2-ANS-102 ← L1-ANS-002
/// **Traces:** L3-SEC-102 ← L2-SEC-102 ← L1-SEC-002
/// **Traces:** L3-SEC-103 ← L2-SEC-103 ← L1-SEC-003
pub fn answer_own<'a>(
    counter: CounterAnswer<'a>,
    packet: SecondaryPacket<'_>,
    context: &OwnContext,
    policy: &OwnPolicy,
) -> Result<OwnAnswer<'a>> {
    if packet.key != counter.key()
        || packet.rtc != Rtc::from_raw((packet.key.ticks & ((1i128 << 48) - 1)) as u64)
    {
        return Err(TimeError::InvalidTimelineInput(
            "own/counter packet/session/RTC mismatch",
        ));
    }
    let location = FindingLocation {
        session: Some(packet.key.session),
        packet: Some(packet.key.packet),
        channel: Some(packet.channel),
        byte_offset: packet.byte_offset,
    };
    let rtc = packet.rtc;
    let header = inspect_secondary(
        packet,
        &SecondaryPolicy {
            levels: policy.levels.clone(),
        },
    )?;
    let convert = || -> Result<OwnPoint> {
        match header.time().ok_or(missing("trusted enabled own time"))? {
            SecondaryHeaderTime::Ieee1588(t) => from_epoch(
                i128::from(t.seconds) * SECOND + i128::from(t.nanoseconds),
                context.ieee.as_ref(),
                &context.table,
            ),
            SecondaryHeaderTime::Ch4(t) => {
                let Ch4Value::BinaryElapsed(n) = Ch4Words::from(t).binary(None)?.value() else {
                    return Err(missing("Chapter4 binary elapsed"));
                };
                from_epoch(i128::from(n.get()), context.ch4.as_ref(), &context.table)
            }
            SecondaryHeaderTime::Ertc(_) => {
                let phase = header.rtc_ertc().ok_or(missing("RTC/ERTC phase"))?;
                let shift = phase
                    .delta_ticks()?
                    .get()
                    .checked_mul(-100)
                    .and_then(|n| n.checked_add(i128::from(phase.remainder_ns().get())))
                    .ok_or_else(overflow)?;
                let tai = counter
                    .normalized(&context.table)?
                    .checked_add(counter.ertc_shift(shift)?)
                    .ok_or_else(overflow)?;
                point(
                    tai,
                    counter
                        .reading()
                        .ok_or(missing("ERTC session scale"))?
                        .scale
                        .effective(),
                    &context.table,
                )
            }
        }
    };
    let (own, issue) = match convert() {
        Ok(p) => (Some(p), None),
        Err(e) => (None, Some(e)),
    };
    let mut out = OwnAnswer {
        counter,
        findings: header.findings().to_vec(),
        header,
        context: context.clone(),
        policy: policy.clone(),
        own,
        delta: None,
        tolerance: policy
            .tolerance
            .or_else(|| counter.precision().map(NanosDuration::new)),
        issue,
    };
    if let Some(own) = &out.own {
        for code in [21, 22] {
            if (code == 21 && own.utc.status == PtpConversionStatus::PossiblyOutOfDate)
                || (code == 22 && own.utc.basis.leap_second)
            {
                let mut e = FindingEvidence::new(location, ValueValidity::Kept).with_raw_rtc(rtc);
                for (name, value) in [
                    ("own_tai_ns", own.tai.get()),
                    (
                        "known_current_through_unix_seconds",
                        i128::from(context.table.metadata().known_current_through.get()),
                    ),
                ] {
                    e = e.with_raw_field(RawField {
                        name,
                        observed: RawValue::Signed(value),
                        expected: None,
                    });
                }
                if let Some(offset) = own.utc.basis.offset {
                    e = e.with_raw_field(RawField {
                        name: "tai_minus_utc_seconds",
                        observed: RawValue::Signed(offset.get().into()),
                        expected: None,
                    });
                }
                out.findings.push(Finding::new(
                    FindingCode::from_number(code)?,
                    policy.levels.get(&code).copied(),
                    e,
                )?);
            }
        }
        let comparison = || -> Result<i128> {
            if out.header.applicability() != &context.counter_applicability {
                return Err(missing("same own/counter applicability"));
            }
            own.tai
                .get()
                .checked_sub(counter.normalized(&context.table)?)
                .ok_or_else(overflow)
        };
        match comparison() {
            Err(e) => out.issue = Some(e),
            Ok(delta) => {
                out.delta = Some(SignedNanos::new(delta));
                if let Some(tolerance) = out.tolerance {
                    if delta.unsigned_abs() > u128::from(tolerance.get()) {
                        let mut e =
                            FindingEvidence::new(location, ValueValidity::Kept).with_raw_rtc(rtc);
                        for (name, value) in [
                            ("own_minus_counter_ns", delta),
                            ("tolerance_ns", i128::from(tolerance.get())),
                            ("own_tai_ns", own.tai.get()),
                            ("counter_tai_ns", own.tai.get() - delta),
                        ] {
                            e = e.with_raw_field(RawField {
                                name,
                                observed: RawValue::Signed(value),
                                expected: None,
                            });
                        }
                        if let Some(r) = counter.reference() {
                            e = e.with_related(FindingLocation {
                                session: Some(r.key.session),
                                packet: Some(r.key.packet),
                                channel: Some(r.channel),
                                byte_offset: None,
                            });
                        }
                        out.findings.push(Finding::new(
                            FindingCode::from_number(14)?,
                            policy.levels.get(&14).copied(),
                            e,
                        )?);
                    }
                } else {
                    out.issue = Some(missing("counter reference resolution or tolerance"));
                }
            }
        }
    }
    Ok(out)
}
