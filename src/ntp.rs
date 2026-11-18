//! RDG-012: NTP UTC reckoning, explicit era evidence and covered leap ambiguity.
//! No host clock, cumulative TAI-UTC correction, or invented NTP leap indicator.
//! Era precedence/default: owner2026-11-12/RDG-012; floor: owner2026-11-13.
use crate::{NanosDuration, NtpTime, Result, SignedUnixSeconds, TimeError, UnixSeconds};
use crate::{
    findings::*,
    format1::CalendarReading,
    leap::{LeapSecondEntry, LeapSecondTable, LeapTableMetadata},
    scale::{ScaleEvidence, ScaleOrigin, TimeScale},
};
use alloc::{string::String, vec::Vec};
const ERA: i128 = 1i128 << 32;
const HALF: i128 = 1i128 << 31;
const EPOCH: i128 = 2_208_988_800;
const NANO: i128 = 1_000_000_000;
/// Why an era was selected; a policy assumption is never independent evidence.
/// **Traces:** L3-NTP-102 ← L2-NTP-102 ← L1-NTP-002
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraBasis {
    /// Caller declared the era and supplied its identity.
    Declared,
    /// All supplied independent same-session pivots agree within strict half-era.
    PivotDerived,
    /// No pivot supplied: explicitly labelled era0 policy assumption.
    AssumedZero,
}
/// Caller era declaration, not an NTP wire field.
/// **Traces:** L3-NTP-102 ← L2-NTP-102 ← L1-NTP-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraDeclaration {
    /// Signed era; negative eras describe dates before1900.
    pub era: i64,
    /// Caller identity for this declaration; must be nonempty.
    pub id: String,
}
/// Preserve the precision of pivot evidence instead of rounding before selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PivotFraction {
    /// Caller/calendar fraction in nanoseconds, below1e9.
    Nanoseconds(u32),
    /// Resolved NTP fraction in exact2^-32second units.
    NtpUnits(u32),
}
/// Independent dated evidence origin; private pivot fields prevent relabelling
/// an assumed-era NTP result through the conversion factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PivotOrigin {
    /// Caller explicitly supplies an approximate UTC date.
    CallerApproximateDate,
    /// Independently dated absolute packet in the same session.
    DatedReference {
        /// Source packet identity, not a byte offset.
        packet: u64,
    },
    /// Previously resolved NTP with a declared/independently derived era.
    ResolvedNtp {
        /// Source era, not inferred again from the rounded output.
        era: i64,
        /// Original era evidence kind; AssumedZero is never admitted here.
        kind: EraBasis,
    },
}
/// Immutable pivot; constructors validate date/scale/precision/provenance.
/// **Traces:** L3-NTP-102 ← L2-NTP-102 ← L1-NTP-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtpPivot {
    session: usize,
    id: String,
    seconds: SignedUnixSeconds,
    fraction: PivotFraction,
    scale: ScaleEvidence,
    origin: PivotOrigin,
    calendar: Option<CalendarReading>,
}
impl NtpPivot {
    /// Explicit caller approximate date; its UTC interpretation is labelled policy.
    /// **Traces:** L3-NTP-102 ← L2-NTP-102 ← L1-NTP-002
    pub fn approximate(
        session: usize,
        id: &str,
        seconds: SignedUnixSeconds,
        nanoseconds: u32,
    ) -> Result<Self> {
        if session == 0 || id.is_empty() {
            return Err(TimeError::NeedsContext {
                context: "named NTP pivot session/identity",
            });
        }
        if nanoseconds >= 1_000_000_000 {
            return Err(TimeError::OutOfRange {
                field: "pivot_nanoseconds",
                value: nanoseconds,
                max: 999_999_999,
            });
        }
        Ok(Self {
            session,
            id: id.into(),
            seconds,
            fraction: PivotFraction::Nanoseconds(nanoseconds),
            scale: ScaleEvidence::assumed(TimeScale::Utc),
            origin: PivotOrigin::CallerApproximateDate,
            calendar: None,
        })
    }
    /// Require an explicit year and UTC scale; preserve the original civil label,
    /// stated/assumed scale and dated reading. An explicit second60 folds only
    /// its pivot coordinate to midnight while retaining the original label.
    /// This supplies era evidence, not an invented NTP leap indicator.
    /// This is an explicit caller assertion of independent packet provenance;
    /// use as_pivot for NTP-derived references to retain their evidence checks.
    pub fn dated(session: usize, id: &str, packet: u64, date: CalendarReading) -> Result<Self> {
        date.validate()?;
        let year = date.year.ok_or(TimeError::NeedsContext {
            context: "NTP pivot requires a dated year",
        })?;
        if date.scale.effective() != TimeScale::Utc {
            return Err(TimeError::NeedsContext {
                context: "NTP pivot requires UTC or explicit scale conversion",
            });
        }
        // Validated year0..9999 and fields bound this signed calendar arithmetic.
        let days: i128 = if year >= 1970 {
            (1970..year).map(days_in_year).sum()
        } else {
            -(year..1970).map(days_in_year).sum::<i128>()
        };
        let seconds = (days + i128::from(date.day_of_year) - 1) * 86400
            + i128::from(date.hours) * 3600
            + i128::from(date.minutes) * 60
            + i128::from(date.seconds);
        let mut out = Self::approximate(
            session,
            id,
            SignedUnixSeconds::new(seconds),
            date.nanoseconds,
        )?;
        out.scale = date.scale;
        out.origin = PivotOrigin::DatedReference { packet };
        out.calendar = Some(date);
        Ok(out)
    }
    /// Original source session.
    pub fn session(&self) -> usize {
        self.session
    }
    /// Original evidence identity.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Exact UTC whole-second coordinate.
    pub fn seconds(&self) -> SignedUnixSeconds {
        self.seconds
    }
    /// Original pivot fraction precision and units.
    pub fn fraction(&self) -> PivotFraction {
        self.fraction
    }
    /// Stated/assumed UTC interpretation retained from the source.
    pub fn scale(&self) -> ScaleEvidence {
        self.scale
    }
    /// Independent origin, never a hidden host-clock date.
    pub fn origin(&self) -> PivotOrigin {
        self.origin
    }
    /// Original dated fields when supplied/available.
    pub fn calendar(&self) -> Option<CalendarReading> {
        self.calendar
    }
}
fn days_in_year(year: u16) -> i128 {
    if crate::util::is_leap_year(year) {
        366
    } else {
        365
    }
}
/// Explicit per-call evidence; no hidden queue or automatic session discovery.
/// **Traces:** L3-NTP-102 ← L2-NTP-102 ← L1-NTP-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtpContext {
    /// Named nonzero session; independent references must name this same session.
    pub session: usize,
    /// Highest precedence; unused contradictory pivots are still retained.
    pub declared: Option<EraDeclaration>,
    /// Caller-bounded supplied evidence; conflicting/ambiguous inputs do not
    /// silently fall back to era0. An empty list enables labelled era0 policy.
    pub pivots: Vec<NtpPivot>,
}
impl NtpContext {
    /// Create an empty evidence set. Session validity is checked by conversion;
    /// even malformed context cannot override the all-zero NTP sentinel.
    pub fn new(session: usize) -> Self {
        Self {
            session,
            declared: None,
            pivots: Vec::new(),
        }
    }
}
/// Explicit unresolved era reason; no new permanent TF identifier is invented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraIssue {
    /// A session was unnamed.
    InvalidSession,
    /// A supplied declaration lacks identity.
    InvalidDeclaration,
    /// A dated reference belongs to another session.
    CrossSession,
    /// An exact strict-half-era boundary has two equally near candidates.
    HalfEraTie,
    /// Supplied pivots choose different eras.
    ConflictingPivots,
    /// Input arithmetic/era exceeds the supported signed quantity domain.
    QuantityRange,
}
/// Mapping status is separate from leap ambiguity and calendar range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtpConversionStatus {
    /// Signed Unix coordinate produced; calendar may separately exceed its range.
    Mapped,
    /// Both raw words zero: TF-029, no absolute time regardless of era evidence.
    UnknownUnsynchronized,
    /// Supplied evidence requires caller resolution; no policy fallback.
    NeedsContext(EraIssue),
}
/// NTP has no leap indicator in IRIG Format2; certainty depends on table coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtpLeapStatus {
    /// No mapped coordinate to assess.
    NoValue,
    /// Covered coordinate/preceding second with no positive insertion match.
    UnambiguousCovered,
    /// Covered [B,B+1) coordinate can mean leap label or following midnight.
    Ambiguous,
    /// No table/coverage: mapped UTC kept, absence of a match proves nothing.
    UncertainCoverage,
}
/// Accepted owner rounding2026-11-13; no carry into seconds.
/// **Traces:** L3-NTP-103 ← L2-NTP-103 ← L1-NTP-003
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtpRounding {
    /// Floor to integer nanoseconds; error is less than1ns.
    FloorNanoseconds,
}
/// Owned basis snapshot; raw evidence is never replaced by a rounded view.
/// **Traces:** L3-NTP-102 ← L2-NTP-102 ← L1-NTP-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtpConversionBasis {
    /// Original declaration and every pivot, including ignored/rejected evidence.
    pub context: NtpContext,
    /// Chosen signed era, absent for sentinel/unresolved evidence.
    pub era: Option<i64>,
    /// Declared / pivot-derived / policy-assumed, never host-clock-derived.
    pub era_basis: Option<EraBasis>,
    /// Supplied pivot requiring resolution, if applicable.
    pub rejected_pivot: Option<usize>,
    /// Raw fractions are retained in NtpConversion::raw.
    pub rounding: NtpRounding,
    /// Nanosecond output quantization, not the wire's2^-32second resolution.
    pub quantization: NanosDuration,
    /// RFC proleptic epoch convention, not a pre1972 physical UTC reconstruction.
    pub proleptic_utc: bool,
    /// Leap ambiguity/certainty, separate from existence of directly mapped UTC.
    pub leap: NtpLeapStatus,
    /// Supplied table identity/currency; None is not guessed as current.
    pub table: Option<LeapTableMetadata>,
    /// Final transition, independent of known-current-through.
    pub final_transition: Option<UnixSeconds>,
    /// Covered positive insertion supporting the alternative civil labels.
    pub ambiguity_transition: Option<LeapSecondEntry>,
}
/// Qualified NTP result; getters keep status, raw and mapping together.
/// **Traces:** L3-NTP-104 ← L2-NTP-104 ← L1-NTP-004
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtpConversion {
    raw: NtpTime,
    status: NtpConversionStatus,
    unix: Option<SignedUnixSeconds>,
    nanos: u32,
    utc: Option<CalendarReading>,
    possible_leap: Option<CalendarReading>,
    calendar_error: Option<TimeError>,
    basis: NtpConversionBasis,
}
fn pivot_era(raw: NtpTime, pivot: &NtpPivot) -> core::result::Result<i64, EraIssue> {
    let p = pivot
        .seconds
        .get()
        .checked_add(EPOCH)
        .ok_or(EraIssue::QuantityRange)?;
    let base = p
        .checked_sub(i128::from(raw.seconds))
        .ok_or(EraIssue::QuantityRange)?
        .div_euclid(ERA);
    let fraction = match pivot.fraction {
        PivotFraction::Nanoseconds(n) => i128::from(n) * ERA,
        PivotFraction::NtpUnits(f) => i128::from(f) * NANO,
    };
    let limit = HALF * ERA * NANO;
    let mut tie = false;
    for k in [base, base.checked_add(1).ok_or(EraIssue::QuantityRange)?] {
        let Ok(era) = i64::try_from(k) else { continue };
        // Subtract whole seconds before scaling: this near-candidate difference
        // is at most one era, avoiding overflow on very large dated evidence.
        let delta = (k * ERA + i128::from(raw.seconds))
            .checked_sub(p)
            .and_then(|d| d.checked_mul(ERA))
            .and_then(|d| d.checked_mul(NANO))
            .and_then(|d| d.checked_add(i128::from(raw.fraction) * NANO))
            .and_then(|d| d.checked_sub(fraction))
            .ok_or(EraIssue::QuantityRange)?;
        if delta > -limit && delta < limit {
            return Ok(era);
        }
        tie |= delta == -limit || delta == limit;
    }
    Err(if tie {
        EraIssue::HalfEraTie
    } else {
        EraIssue::QuantityRange
    })
}
fn resolve(
    raw: NtpTime,
    c: &NtpContext,
) -> core::result::Result<(i64, EraBasis), (EraIssue, Option<usize>)> {
    if c.session == 0 {
        return Err((EraIssue::InvalidSession, None));
    }
    if let Some(d) = &c.declared {
        return if d.id.is_empty() {
            Err((EraIssue::InvalidDeclaration, None))
        } else {
            Ok((d.era, EraBasis::Declared))
        };
    }
    let mut selected = None;
    for (i, p) in c.pivots.iter().enumerate() {
        if p.session != c.session {
            return Err((EraIssue::CrossSession, Some(i)));
        }
        let era = pivot_era(raw, p).map_err(|e| (e, Some(i)))?;
        if selected.is_some_and(|old| old != era) {
            return Err((EraIssue::ConflictingPivots, Some(i)));
        }
        selected = Some(era);
    }
    Ok(selected.map_or((0, EraBasis::AssumedZero), |era| {
        (era, EraBasis::PivotDerived)
    }))
}
/// Map era*M+seconds-2208988800 with no TAI-UTC adjustment. Both zero words
/// take precedence over context/rounding. Era0 is labelled only when no pivots
/// were supplied; invalid supplied evidence never silently becomes a century.
/// **Traces:** L3-NTP-104 ← L2-NTP-104 ← L1-NTP-004
/// **Traces:** L3-RDG-212 ← L2-RDG-212 ← L1-RDG-002
pub fn convert_ntp(
    raw: NtpTime,
    context: &NtpContext,
    table: Option<&LeapSecondTable>,
) -> NtpConversion {
    let mut out = NtpConversion {
        raw,
        status: NtpConversionStatus::UnknownUnsynchronized,
        unix: None,
        nanos: 0,
        utc: None,
        possible_leap: None,
        calendar_error: None,
        basis: NtpConversionBasis {
            context: context.clone(),
            era: None,
            era_basis: None,
            rejected_pivot: None,
            rounding: NtpRounding::FloorNanoseconds,
            quantization: NanosDuration::new(1),
            proleptic_utc: false,
            leap: NtpLeapStatus::NoValue,
            table: table.map(|t| t.metadata().clone()),
            final_transition: table.and_then(LeapSecondTable::final_transition),
            ambiguity_transition: None,
        },
    };
    if raw.seconds == 0 && raw.fraction == 0 {
        return out;
    }
    out.nanos = raw.fraction_as_nanos();
    let (era, kind) = match resolve(raw, context) {
        Ok(x) => x,
        Err((issue, index)) => {
            out.status = NtpConversionStatus::NeedsContext(issue);
            out.basis.rejected_pivot = index;
            return out;
        }
    };
    // i64 eras *2^32 fit i128; checked arithmetic documents the conversion domain.
    let Some(seconds) = i128::from(era)
        .checked_mul(ERA)
        .and_then(|s| s.checked_add(i128::from(raw.seconds)))
        .and_then(|s| s.checked_sub(EPOCH))
    else {
        out.status = NtpConversionStatus::NeedsContext(EraIssue::QuantityRange);
        return out;
    };
    let unix = SignedUnixSeconds::new(seconds);
    out.unix = Some(unix);
    out.status = NtpConversionStatus::Mapped;
    out.basis.era = Some(era);
    out.basis.era_basis = Some(kind);
    out.basis.proleptic_utc = seconds < 63_072_000;
    match crate::leap::calendar(unix, out.nanos, false) {
        Ok(c) => out.utc = Some(c),
        Err(e) => out.calendar_error = Some(e),
    };
    out.basis.leap = NtpLeapStatus::UncertainCoverage;
    if let Some(table) = table {
        // Both possible civil sides need coverage for certainty at a narrowed
        // start; direct NTP UTC remains available even when neither is covered.
        let previous = seconds.checked_sub(1).map(SignedUnixSeconds::new);
        if table.covers_unix(unix) && previous.is_some_and(|p| table.covers_unix(p)) {
            out.basis.leap = NtpLeapStatus::UnambiguousCovered;
            // The integer word plus any original fraction is precisely [B,B+1).
            // Never use a rounded/carrying nanosecond coordinate for this test.
            if let Some(pair) = table.entries().windows(2).find(|p| {
                p[1].effective_unix.to_signed() == unix
                    && i64::from(p[1].tai_utc_offset.get()) - i64::from(p[0].tai_utc_offset.get())
                        == 1
            }) {
                out.basis.leap = NtpLeapStatus::Ambiguous;
                out.basis.ambiguity_transition = Some(pair[1]);
                match crate::leap::calendar(unix, out.nanos, true) {
                    Ok(c) => out.possible_leap = Some(c),
                    Err(e) => out.calendar_error = Some(e),
                };
            }
        }
    }
    out
}
impl NtpConversion {
    /// Exact original seconds and2^-32second fraction, even for zero sentinel.
    pub fn raw(&self) -> NtpTime {
        self.raw
    }
    /// Mapped / unsynchronized / explicit era issue, not a severity decision.
    pub fn status(&self) -> NtpConversionStatus {
        self.status
    }
    /// Checked signed UTC coordinate; negative epochs never clamp to zero.
    pub fn unix_seconds(&self) -> Option<SignedUnixSeconds> {
        self.unix
    }
    /// Floor-rounded view of the raw fractional word; never a1e9 carry.
    pub fn nanoseconds(&self) -> u32 {
        self.nanos
    }
    /// Ordinary mapped civil UTC, not a claim a definite leap label was read.
    pub fn utc(&self) -> Option<CalendarReading> {
        self.utc
    }
    /// Alternative second60 only for covered insertion ambiguity.
    pub fn possible_leap(&self) -> Option<CalendarReading> {
        self.possible_leap
    }
    /// Signed mapping may exist outside the supported calendar year range.
    pub fn calendar_error(&self) -> Option<&TimeError> {
        self.calendar_error.as_ref()
    }
    /// Immutable full evidence/policy/table snapshot.
    pub fn basis(&self) -> &NtpConversionBasis {
        &self.basis
    }
    /// Export only independently resolved NTP as future pivot evidence. A prior
    /// era0 assumption cannot become an apparently independent dated reference.
    /// Preserve exact raw fraction and any scale assumption used by its pivot.
    pub fn as_pivot(&self, id: &str) -> Result<NtpPivot> {
        let (Some(era), Some(kind), Some(seconds)) =
            (self.basis.era, self.basis.era_basis, self.unix)
        else {
            return Err(TimeError::NeedsContext {
                context: "unresolved NTP is not independent dated evidence",
            });
        };
        if kind == EraBasis::AssumedZero {
            return Err(TimeError::NeedsContext {
                context: "assumed-era NTP cannot bootstrap a pivot",
            });
        }
        let mut out = NtpPivot::approximate(self.basis.context.session, id, seconds, 0)?;
        let assumed = kind == EraBasis::PivotDerived
            && self
                .basis
                .context
                .pivots
                .iter()
                .any(|p| p.scale.origin() == ScaleOrigin::Policy);
        out.scale = if assumed {
            ScaleEvidence::assumed(TimeScale::Utc)
        } else {
            ScaleEvidence::standard(TimeScale::Utc)
        };
        out.fraction = PivotFraction::NtpUnits(self.raw.fraction);
        out.origin = PivotOrigin::ResolvedNtp { era, kind };
        out.calendar = self.utc;
        Ok(out)
    }
    /// TF-029 unknown/no-value or TF-030 ambiguous/value-kept; neither is TF-022.
    /// Table uncertainty keeps UTC in the basis, not a spurious TAI correction
    /// or a TF-021 PTP offset warning. Caller override never changes validity.
    /// **Traces:** L3-NTP-104 ← L2-NTP-104 ← L1-NTP-004
    /// **Traces:** L3-TAI-107 ← L2-TAI-107 ← L1-TAI-007
    pub fn findings(
        &self,
        location: FindingLocation,
        level: Option<Severity>,
    ) -> Result<Vec<Finding>> {
        let f = if self.status == NtpConversionStatus::UnknownUnsynchronized {
            Some(Finding::ntp_zero(location, self.raw, level)?)
        } else if self.basis.leap == NtpLeapStatus::Ambiguous {
            let mapped = self.unix.and_then(SignedUnixSeconds::to_unsigned).ok_or(
                TimeError::QuantityOverflow {
                    quantity: "covered NTP ambiguity Unix seconds",
                },
            )?;
            Some(Finding::ntp_ambiguity(location, self.raw, mapped, level)?)
        } else {
            None
        };
        let mut out = Vec::new();
        if let Some(f) = f {
            let code = f.code();
            let severity = f.severity();
            let mut evidence = f.into_evidence().with_raw_field(RawField {
                name: "nanoseconds_floor",
                observed: RawValue::Unsigned(self.nanos.into()),
                expected: None,
            });
            if let Some(era) = self.basis.era {
                evidence = evidence.with_raw_field(RawField {
                    name: "ntp_era",
                    observed: RawValue::Signed(era.into()),
                    expected: None,
                });
            }
            out.push(Finding::new(code, Some(severity), evidence)?);
        }
        Ok(out)
    }
}
