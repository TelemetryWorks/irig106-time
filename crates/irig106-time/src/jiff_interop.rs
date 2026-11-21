//! Checked civil interoperability (owner 2026-11-17, ADR-0020).
//!
//! Enable optional `jiff` (off by default). [`jiff::civil::DateTime`] matches
//! naive date/time fields, not an instant or timezone. [`Scaled`] retains the
//! supplied scale/provenance; no UTC/TAI/GPS conversion, epoch inference, clock
//! access or timezone lookup occurs. Convert with explicit table context first
//! if UTC is needed, retaining the original conversion basis separately.
//! Second 60 cannot fit Jiff civil time and is explicitly refused (RDG-010).
//! Unknown year/scale is never replaced with 1970/UTC. Supported years are 0..=9999.
//!
//! ```
//! use irig106_time::{AbsoluteTime, jiff_interop::{absolute_to_civil, civil_to_calendar},
//!     scale::{Scaled, ScaleEvidence, TimeScale}};
//! let input = Scaled { value: AbsoluteTime::new(100,12,30,25,340_000_000)?
//!     .with_year(Some(2025))?, scale: ScaleEvidence::standard(TimeScale::Utc) };
//! let civil = absolute_to_civil(input)?;
//! assert_eq!(civil.value.month(),4);
//! assert_eq!(civil_to_calendar(civil)?.value.day_of_year(),100);
//! # Ok::<(), irig106_time::TimeError>(())
//! ```
use crate::{
    AbsoluteTime, CalendarTime, Result, TimeError,
    format1::CalendarReading,
    scale::{ScaleEvidence, Scaled, TimeScale},
};
use jiff::civil::{Date, DateTime};

fn known_scale(scale: ScaleEvidence) -> Result<()> {
    if scale.effective() == TimeScale::Unknown {
        return Err(TimeError::NeedsContext {
            context: "jiff civil scale",
        });
    }
    Ok(())
}

/// Preserve a validated calendar reading's fields and scale in civil form.
/// Require a year and known scale; refuse second 60 rather than clamp/normalize.
/// **Traces:** L3-API-111 ← L2-API-111 ← L1-API-011
pub fn reading_to_civil(value: CalendarReading) -> Result<Scaled<DateTime>> {
    value.validate()?;
    known_scale(value.scale)?;
    let year = value.year.ok_or(TimeError::NeedsContext {
        context: "jiff civil year",
    })?;
    if value.seconds == 60 {
        return Err(TimeError::NeedsContext {
            context: "jiff civil cannot represent second 60",
        });
    }
    let date = Date::new(year as i16, 1, 1)
        .and_then(|d| d.with().day_of_year(value.day_of_year as i16).build())
        .map_err(|_| TimeError::NeedsContext {
            context: "jiff civil date representation",
        })?;
    let civil = DateTime::new(
        year as i16,
        date.month(),
        date.day(),
        value.hours as i8,
        value.minutes as i8,
        value.seconds as i8,
        value.nanoseconds as i32,
    )
    .map_err(|_| TimeError::NeedsContext {
        context: "jiff civil time representation",
    })?;
    Ok(Scaled {
        value: civil,
        scale: value.scale,
    })
}

/// Convert a legacy DOY value only with explicit year and scale evidence.
/// No 1970 fallback or implicit UTC; ordinary values have no second 60 label.
/// **Traces:** L3-API-111 ← L2-API-111 ← L1-API-011
pub fn absolute_to_civil(value: Scaled<AbsoluteTime>) -> Result<Scaled<DateTime>> {
    let t = value.value;
    reading_to_civil(CalendarReading {
        year: t.year(),
        day_of_year: t.day_of_year(),
        hours: t.hours(),
        minutes: t.minutes(),
        seconds: t.seconds(),
        nanoseconds: t.nanoseconds(),
        scale: value.scale,
    })
}

/// Convert the validated calendar value while retaining explicit scale evidence.
/// **Traces:** L3-API-111 ← L2-API-111 ← L1-API-011
pub fn calendar_to_civil(value: Scaled<CalendarTime>) -> Result<Scaled<DateTime>> {
    absolute_to_civil(Scaled {
        value: value.value.into_absolute_time(),
        scale: value.scale,
    })
}

/// Convert civil fields back to a validated calendar with declared scale.
/// Reject negative Jiff years (the native calendar domain is 0..=9999).
/// Jiff civil values cannot contain second 60; no leap-label clamping occurs.
/// **Traces:** L3-API-111 ← L2-API-111 ← L1-API-011
pub fn civil_to_calendar(value: Scaled<DateTime>) -> Result<Scaled<CalendarTime>> {
    known_scale(value.scale)?;
    let t = value.value;
    let year =
        u16::try_from(t.year()).map_err(|_| TimeError::CivilYearOutOfRange { year: t.year() })?;
    let calendar = CalendarTime::from_parts(
        year,
        t.month() as u8,
        t.day() as u8,
        t.day_of_year() as u16,
        t.hour() as u8,
        t.minute() as u8,
        t.second() as u8,
        t.subsec_nanosecond() as u32,
    )?;
    Ok(Scaled {
        value: calendar,
        scale: value.scale,
    })
}
