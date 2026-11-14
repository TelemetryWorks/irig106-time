//! Validated integer-offset UTC model, exact only from1972; no implicit fetch.
use crate::{
    AbsoluteTime, PtpTime, Result, SignedNanos, SignedUnixSeconds, TaiSeconds, TaiUtcOffset,
    TimeError, UnixSeconds,
};
use crate::{
    findings::*,
    format1::CalendarReading,
    scale::{ScaleEvidence, TimeScale},
};
use alloc::{string::String, vec::Vec};
const START: u64 = 63_072_000;
const MAX_CALENDAR: u64 = 253_402_300_799;
const BUILTIN: &[(u64, i32)] = &[
    (63072000, 10),
    (78796800, 11),
    (94694400, 12),
    (126230400, 13),
    (157766400, 14),
    (189302400, 15),
    (220924800, 16),
    (252460800, 17),
    (283996800, 18),
    (315532800, 19),
    (362793600, 20),
    (394329600, 21),
    (425865600, 22),
    (489024000, 23),
    (567993600, 24),
    (631152000, 25),
    (662688000, 26),
    (709948800, 27),
    (741484800, 28),
    (773020800, 29),
    (820454400, 30),
    (867715200, 31),
    (915148800, 32),
    (1136073600, 33),
    (1230768000, 34),
    (1341100800, 35),
    (1435708800, 36),
    (1483228800, 37),
];
/// One typed offset effective at a UTC boundary; first entry is a baseline.
/// **Traces:** L3-TAI-101 ← L2-TAI-101 ← L1-TAI-001
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LeapSecondEntry {
    /// Unix UTC instant at which the offset is effective.
    pub effective_unix: UnixSeconds,
    /// TAI-minus-UTC seconds, not an unsigned magnitude.
    pub tai_utc_offset: TaiUtcOffset,
}
/// Identity/version and independently declared UTC coverage, not final transition.
/// **Traces:** L3-TAI-104 ← L2-TAI-104 ← L1-TAI-004
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeapTableMetadata {
    /// Caller versioned identity, or the pinned IERS snapshot identity.
    pub id: String,
    /// Source/authority supplied with the data; no trust inferred from a URL.
    pub source: String,
    /// First supported UTC second, inclusive; never before1972.
    pub supported_start: UnixSeconds,
    /// Last covered UTC second, including ALL its nanosecond fractions.
    /// Coverage ends at the following second. Independent of final transition.
    pub known_current_through: UnixSeconds,
}
/// Validated transitions and coverage, immutable except checked add/install.
/// **Traces:** L3-TAI-101 ← L2-TAI-101 ← L1-TAI-001
/// **Traces:** L3-TAI-103 ← L2-TAI-103 ← L1-TAI-003
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "TableWire"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeapSecondTable {
    entries: Vec<LeapSecondEntry>,
    metadata: LeapTableMetadata,
}
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct TableWire {
    entries: Vec<LeapSecondEntry>,
    metadata: LeapTableMetadata,
}
#[cfg(feature = "serde")]
impl TryFrom<TableWire> for LeapSecondTable {
    type Error = TimeError;
    fn try_from(w: TableWire) -> Result<Self> {
        Self::try_new(w.entries, w.metadata)
    }
}
fn invalid(reason: &'static str) -> TimeError {
    TimeError::InvalidLeapTable { reason }
}
// u64 UTC + i32 offset fits i128, including a negative offset: no cast/saturation.
fn tai_boundary(e: LeapSecondEntry) -> i128 {
    i128::from(e.effective_unix.get()) + i128::from(e.tai_utc_offset.get())
}
impl LeapSecondTable {
    /// IERS C72 (2026-07-06), entire28-row list, checked in S04/release evidence.
    /// Expiry2027-06-28 is conservatively interpreted at00:00 UTC; coverage is
    /// through2027-06-27 23:59:59 including fractions, not merely through2017.
    /// **Traces:** L3-TAI-102 ← L2-TAI-102 ← L1-TAI-002
    pub fn builtin() -> Self {
        let entries = BUILTIN
            .iter()
            .map(|&(at, offset)| LeapSecondEntry {
                effective_unix: UnixSeconds::new(at),
                tai_utc_offset: TaiUtcOffset::new(offset),
            })
            .collect();
        let metadata=LeapTableMetadata {
            id:"IERS-C72-2026-07-06/6cb6f5d4b819f2e568e25db4b0b26d89dedf031fdffb18bc94d40f4e94e268d7".into(),
            source:"https://hpiers.obspm.fr/iers/bul/bulc/Leap_Second.dat (Bulletin C72)".into(),
            supported_start:UnixSeconds::new(START),known_current_through:UnixSeconds::new(1_814_140_799),
        };
        Self { entries, metadata }
    }
    /// Empty has no known offset, including before/after its declared interval.
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            metadata: LeapTableMetadata {
                id: "empty".into(),
                source: "caller".into(),
                supported_start: UnixSeconds::new(START),
                known_current_through: UnixSeconds::new(START),
            },
        }
    }
    /// Validate without sorting, deduplicating or repairing caller data.
    /// Source credibility is caller/release-reviewed; replacements also compare
    /// known history against the installed table. Future +/-1 steps are allowed.
    pub fn try_new(entries: Vec<LeapSecondEntry>, metadata: LeapTableMetadata) -> Result<Self> {
        if metadata.id.is_empty()
            || metadata.source.is_empty()
            || metadata.supported_start.get() < START
            || metadata.known_current_through < metadata.supported_start
            || metadata.known_current_through.get() > MAX_CALENDAR
        {
            return Err(invalid("invalid identity/source/coverage"));
        }
        for (i, e) in entries.iter().enumerate() {
            if e.effective_unix.get() < START || e.effective_unix > metadata.known_current_through {
                return Err(invalid("transition outside supported integer UTC model"));
            }
            if i == 0 {
                if e.effective_unix > metadata.supported_start {
                    return Err(invalid("no baseline at supported start"));
                }
            } else {
                let prev = entries[i - 1];
                let delta =
                    i64::from(e.tai_utc_offset.get()) - i64::from(prev.tai_utc_offset.get());
                if e.effective_unix <= prev.effective_unix
                    || tai_boundary(*e) <= tai_boundary(prev)
                    || !matches!(delta, -1 | 1)
                {
                    return Err(invalid("unordered/duplicate transition or non-unit step"));
                }
                let c = calendar(e.effective_unix.to_signed(), 0, false)?;
                let july =
                    if crate::util::is_leap_year(c.year.ok_or(invalid("missing transition year"))?)
                    {
                        183
                    } else {
                        182
                    };
                if c.hours != 0
                    || c.minutes != 0
                    || c.seconds != 0
                    || (c.day_of_year != 1 && c.day_of_year != july)
                {
                    return Err(invalid("step is not an allowed June/December UTC boundary"));
                }
            }
        }
        Ok(Self { entries, metadata })
    }
    /// Immutable transitions; the baseline is not itself a leap insertion.
    pub fn entries(&self) -> &[LeapSecondEntry] {
        &self.entries
    }
    /// Declared identity and independently bounded coverage.
    pub fn metadata(&self) -> &LeapTableMetadata {
        &self.metadata
    }
    /// Last offset transition, not the last known-current instant.
    pub fn final_transition(&self) -> Option<UnixSeconds> {
        self.entries.last().map(|e| e.effective_unix)
    }
    /// Count including baseline.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// No offset available.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// True only inside the declared coverage and with an applicable baseline.
    pub fn covers_unix(&self, time: SignedUnixSeconds) -> bool {
        !self.is_empty()
            && time >= self.metadata.supported_start.to_signed()
            && time <= self.metadata.known_current_through.to_signed()
    }
    /// Offset-only helper; after coverage still returns last known offset.
    /// Use convert_ptp for currency/basis/findings; pre-start/empty need context.
    pub fn offset_at_unix(&self, time: UnixSeconds) -> Result<TaiUtcOffset> {
        if time < self.metadata.supported_start {
            return Err(TimeError::NeedsContext {
                context: "before supported leap table start",
            });
        }
        self.entries
            .partition_point(|e| e.effective_unix <= time)
            .checked_sub(1)
            .map(|i| self.entries[i].tai_utc_offset)
            .ok_or(TimeError::NeedsContext {
                context: "no known applicable TAI-UTC offset",
            })
    }
    fn selected_index(&self, time: TaiSeconds) -> Result<usize> {
        let i = self
            .entries
            .partition_point(|e| tai_boundary(*e) <= i128::from(time.get()))
            .checked_sub(1)
            .ok_or(TimeError::NeedsContext {
                context: "no known applicable TAI-UTC offset",
            })?;
        let start_offset = self.offset_at_unix(self.metadata.supported_start)?;
        let start_tai =
            i128::from(self.metadata.supported_start.get()) + i128::from(start_offset.get());
        if i128::from(time.get()) < start_tai {
            return Err(TimeError::NeedsContext {
                context: "before supported leap table start",
            });
        }
        Ok(i)
    }
    /// Latest applicable TAI-side boundary; never approximate with offset37.
    pub fn offset_at_tai(&self, time: TaiSeconds) -> Result<TaiUtcOffset> {
        self.selected_index(time)
            .map(|i| self.entries[i].tai_utc_offset)
    }
    /// New validated snapshot; overlapping known history must agree throughout.
    pub fn replaced(
        &self,
        entries: Vec<LeapSecondEntry>,
        metadata: LeapTableMetadata,
    ) -> Result<Self> {
        let candidate = Self::try_new(entries, metadata)?;
        let start = self
            .metadata
            .supported_start
            .max(candidate.metadata.supported_start);
        let end = self
            .metadata
            .known_current_through
            .min(candidate.metadata.known_current_through);
        for at in core::iter::once(start)
            .chain(
                self.entries
                    .iter()
                    .chain(candidate.entries.iter())
                    .map(|e| e.effective_unix),
            )
            .filter(|at| *at >= start && *at <= end)
        {
            if let (Ok(a), Ok(b)) = (self.offset_at_unix(at), candidate.offset_at_unix(at)) {
                if a != b {
                    return Err(invalid("replacement contradicts known historical offsets"));
                }
            }
        }
        Ok(candidate)
    }
    /// Merge an ordered extension; equal-date/equal-offset overlap is idempotent.
    /// Conflicts, reordered input and historical rewrites fail before installation.
    pub fn extended(
        &self,
        extra: Vec<LeapSecondEntry>,
        metadata: LeapTableMetadata,
    ) -> Result<Self> {
        if metadata.supported_start != self.metadata.supported_start
            || metadata.known_current_through < self.metadata.known_current_through
        {
            return Err(invalid("extension narrows known coverage"));
        }
        if extra
            .windows(2)
            .any(|w| w[0].effective_unix >= w[1].effective_unix)
        {
            return Err(invalid("extension is not strictly ordered"));
        }
        let mut merged = self.entries.clone();
        for e in extra {
            match merged.binary_search_by_key(&e.effective_unix, |x| x.effective_unix) {
                Ok(i) if merged[i] != e => return Err(invalid("conflicting overlapping offset")),
                Ok(_) => (),
                Err(i) => merged.insert(i, e),
            }
        }
        self.replaced(merged, metadata)
    }
    /// Compatibility mutator, checked and atomic; does not silently extend currency.
    /// Use extended with new explicit source/coverage for dates beyond coverage.
    pub fn add(&mut self, entry: LeapSecondEntry) -> Result<()> {
        let new = self.extended(alloc::vec![entry], self.metadata.clone())?;
        *self = new;
        Ok(())
    }
    /// Validated ordinary F1 date lookup; no day0 repair or pre1972 extrapolation.
    pub fn offset_for_f1(&self, year: u16, day_of_year: u16) -> Result<TaiUtcOffset> {
        let t = AbsoluteTime::new(day_of_year, 0, 0, 0, 0)?.with_year(Some(year))?;
        if year < 1972 {
            return Err(TimeError::NeedsContext {
                context: "integer UTC offsets exact only from1972",
            });
        }
        let days: u64 = (1970..year)
            .map(|y| {
                if crate::util::is_leap_year(y) {
                    366
                } else {
                    365
                }
            })
            .sum();
        let secs = (days + u64::from(t.day_of_year()) - 1)
            .checked_mul(86400)
            .ok_or(TimeError::QuantityOverflow {
                quantity: "F1 Unix seconds",
            })?;
        self.offset_at_unix(UnixSeconds::new(secs))
    }
    /// Proximity to an actual offset change, not the first baseline or proof of
    /// an NTP insertion-ambiguity window (that converter is S05).
    pub fn is_near_leap_second(&self, time: UnixSeconds, window_secs: u64) -> bool {
        self.entries
            .iter()
            .skip(1)
            .any(|e| crate::util::abs_diff_u64(time.get(), e.effective_unix.get()) <= window_secs)
    }
    /// Preserve raw TAI and leap labels, checked UTC and full currency/basis.
    /// D7: stale uses last known offset + TF-021; missing offset produces no UTC.
    /// **Traces:** L3-PTP-104 ← L2-PTP-104 ← L1-PTP-004
    /// **Traces:** L3-TAI-104 ← L2-TAI-104 ← L1-TAI-004
    pub fn convert_ptp(&self, raw: PtpTime) -> Result<PtpConversion> {
        if raw.nanoseconds >= 1_000_000_000 {
            return Err(TimeError::OutOfRange {
                field: "ptp_nanoseconds",
                value: raw.nanoseconds,
                max: 999_999_999,
            });
        }
        let mut out = PtpConversion {
            raw,
            utc: None,
            unix_seconds: None,
            status: PtpConversionStatus::MissingOffset,
            basis: PtpConversionBasis {
                table: self.metadata.clone(),
                final_transition: self.final_transition(),
                offset: None,
                selected_transition: None,
                leap_second: false,
            },
        };
        let Ok(i) = self.selected_index(raw.tai_seconds()) else {
            return Ok(out);
        };
        let offset = self.entries[i].tai_utc_offset;
        let unix = raw.tai_seconds().to_signed_unix(offset);
        let leap = self.entries.get(i + 1).is_some_and(|next| {
            i64::from(next.tai_utc_offset.get()) - i64::from(offset.get()) == 1
                && i128::from(raw.seconds) + 1 == tai_boundary(*next)
        });
        out.utc = Some(calendar(unix, raw.nanoseconds, leap)?);
        out.unix_seconds = Some(unix);
        out.status = if self.covers_unix(unix) {
            PtpConversionStatus::Covered
        } else {
            PtpConversionStatus::PossiblyOutOfDate
        };
        out.basis.offset = Some(offset);
        out.basis.selected_transition = Some(self.entries[i]);
        out.basis.leap_second = leap;
        Ok(out)
    }
}
impl Default for LeapSecondTable {
    fn default() -> Self {
        Self::builtin()
    }
}
fn calendar(unix: SignedUnixSeconds, nanoseconds: u32, leap: bool) -> Result<CalendarReading> {
    let coordinate = unix.get() - i128::from(leap);
    let ns = coordinate
        .checked_mul(1_000_000_000)
        .and_then(|s| s.checked_add(i128::from(nanoseconds)))
        .ok_or(TimeError::QuantityOverflow {
            quantity: "UTC calendar nanoseconds",
        })?;
    let t = AbsoluteTime::new(1, 0, 0, 0, 0)?
        .with_year(Some(1970))?
        .checked_shift(SignedNanos::new(ns))?;
    let result = CalendarReading {
        year: t.year(),
        day_of_year: t.day_of_year(),
        hours: t.hours(),
        minutes: t.minutes(),
        seconds: if leap { 60 } else { t.seconds() },
        nanoseconds: t.nanoseconds(),
        scale: ScaleEvidence::standard(TimeScale::Utc),
    };
    result.validate()?;
    Ok(result)
}
/// Currency is independent of presence/absence of an absolute value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtpConversionStatus {
    /// Within independently declared table coverage.
    Covered,
    /// UTC retained with last known offset; TF-021 WARNING.
    PossiblyOutOfDate,
    /// Empty/pre-start: raw TAI retained, no applicable known offset/UTC.
    MissingOffset,
}
/// Snapshot retained on each answer, not a live mutable table reference.
/// **Traces:** L3-PTP-104 ← L2-PTP-104 ← L1-PTP-004
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtpConversionBasis {
    /// Versioned source identity and coverage at conversion time.
    pub table: LeapTableMetadata,
    /// Last table transition, not coverage expiry.
    pub final_transition: Option<UnixSeconds>,
    /// Actually applied signed TAI-minus-UTC seconds.
    pub offset: Option<TaiUtcOffset>,
    /// Transition supplying that offset, not necessarily the final transition.
    pub selected_transition: Option<LeapSecondEntry>,
    /// Distinct inserted UTC second; Unix coordinate repeats midnight's value.
    pub leap_second: bool,
}
/// Checked conversion result; immutable getters keep raw/basis/status together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtpConversion {
    raw: PtpTime,
    utc: Option<CalendarReading>,
    unix_seconds: Option<SignedUnixSeconds>,
    status: PtpConversionStatus,
    basis: PtpConversionBasis,
}
impl PtpConversion {
    /// Exact original TAI seconds/nanoseconds.
    pub fn raw(&self) -> PtpTime {
        self.raw
    }
    /// Labelled UTC, including second60; None only for missing applicable offset.
    pub fn utc(&self) -> Option<CalendarReading> {
        self.utc
    }
    /// Unix-style UTC whole-second coordinate; at an inserted second repeats
    /// the following midnight coordinate. Consult UTC second60/basis explicitly.
    pub fn unix_seconds(&self) -> Option<SignedUnixSeconds> {
        self.unix_seconds
    }
    /// Covered, possibly-out-of-date with value, or missing context without value.
    pub fn status(&self) -> PtpConversionStatus {
        self.status
    }
    /// Actual source, offset and coverage used.
    pub fn basis(&self) -> &PtpConversionBasis {
        &self.basis
    }
    /// TF-021 stale value-kept and TF-022 read leap label, with exact raw operands.
    /// Override affects TF-021 severity only, never currency/value or TF-022.
    /// **Traces:** L3-TAI-104 ← L2-TAI-104 ← L1-TAI-004
    pub fn findings(
        &self,
        location: FindingLocation,
        level: Option<Severity>,
    ) -> Result<Vec<Finding>> {
        let mut out = Vec::new();
        for code in [21, 22] {
            if (code == 21 && self.status == PtpConversionStatus::PossiblyOutOfDate)
                || (code == 22 && self.basis.leap_second)
            {
                let mut evidence = FindingEvidence::new(location, ValueValidity::Kept);
                for (name, observed) in [
                    ("tai_seconds", RawValue::Unsigned(self.raw.seconds.into())),
                    (
                        "tai_nanoseconds",
                        RawValue::Unsigned(self.raw.nanoseconds.into()),
                    ),
                    (
                        "known_current_through_unix_seconds",
                        RawValue::Unsigned(self.basis.table.known_current_through.get().into()),
                    ),
                ] {
                    evidence = evidence.with_raw_field(RawField {
                        name,
                        observed,
                        expected: None,
                    });
                }
                if let Some(offset) = self.basis.offset {
                    evidence = evidence.with_raw_field(RawField {
                        name: "tai_minus_utc_seconds",
                        observed: RawValue::Signed(offset.get().into()),
                        expected: None,
                    });
                }
                out.push(Finding::new(
                    FindingCode::from_number(code)?,
                    if code == 21 { level } else { None },
                    evidence,
                )?);
            }
        }
        Ok(out)
    }
}
