#![cfg(feature = "jiff")]
//! ADR0020 / Jiff1.0.1 civil::DateTime (Gregorian, nanoseconds, no second60).
//! Hand-derived DOYs: common Apr10=31+28+31+10=100; leap Jul18=182+18=200.
use irig106_time::{
    AbsoluteTime, CalendarTime, TimeError,
    format1::CalendarReading,
    jiff_interop::*,
    scale::{ScaleEvidence, Scaled, TimeScale},
};
use jiff::civil::DateTime;
fn utc<T>(value: T) -> Scaled<T> {
    Scaled {
        value,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    }
}
fn reading(year: Option<u16>, day: u16) -> CalendarReading {
    CalendarReading {
        year,
        day_of_year: day,
        hours: 12,
        minutes: 30,
        seconds: 25,
        nanoseconds: 340_000_000,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    }
}
/// Clause-level API-011/111 coverage is recorded in verification-jiff.md.
#[test]
fn absolute_to_jiff_ports_absolute_to_chrono() {
    let t = AbsoluteTime::new(100, 12, 30, 25, 340_000_000)
        .unwrap()
        .with_year(Some(2025))
        .unwrap();
    let dt = absolute_to_civil(utc(t)).unwrap();
    assert_eq!(
        dt,
        utc(DateTime::new(2025, 4, 10, 12, 30, 25, 340_000_000).unwrap())
    );
}
#[test]
fn calendar_to_jiff_ports_calendar_to_chrono() {
    let t = CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap();
    assert_eq!(
        calendar_to_civil(utc(t)).unwrap(),
        utc(DateTime::new(2025, 4, 10, 12, 30, 25, 340_000_000).unwrap())
    );
}
#[test]
fn jiff_to_calendar_ports_chrono_to_calendar_time() {
    let t = civil_to_calendar(utc(
        DateTime::new(2025, 4, 10, 12, 30, 25, 340_000_000).unwrap()
    ))
    .unwrap();
    assert_eq!(
        t,
        utc(CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap())
    );
}
#[test]
fn round_trip_ports_round_trip_via_calendar_time() {
    let t = utc(CalendarTime::from_parts(2024, 7, 18, 200, 8, 15, 45, 123_456_789).unwrap());
    assert_eq!(civil_to_calendar(calendar_to_civil(t).unwrap()).unwrap(), t);
}
#[test]
fn no_year_refuses_instead_of_legacy_default1970() {
    let t = AbsoluteTime::new(50, 6, 0, 0, 0).unwrap();
    assert!(matches!(
        absolute_to_civil(utc(t)),
        Err(TimeError::NeedsContext {
            context: "jiff civil year"
        })
    ));
    assert!(matches!(
        reading_to_civil(reading(None, 50)),
        Err(TimeError::NeedsContext {
            context: "jiff civil year"
        })
    ));
}
#[test]
fn calendar_to_absolute_ports_calendar_to_absolute_drops_month_day() {
    let c = civil_to_calendar(utc(DateTime::new(2025, 4, 10, 12, 0, 0, 0).unwrap())).unwrap();
    assert_eq!(
        c.value.into_absolute_time(),
        AbsoluteTime::new(100, 12, 0, 0, 0)
            .unwrap()
            .with_year(Some(2025))
            .unwrap()
    );
}
#[test]
fn doy_boundaries_and_year_limits_are_checked() {
    // Gregorian century rule: 1900 common, 2000 leap; astronomical year0 leap.
    for (year, day, month, dom) in [
        (0, 1, 1, 1),
        (0, 366, 12, 31),
        (1900, 59, 2, 28),
        (1900, 60, 3, 1),
        (2000, 60, 2, 29),
        (2000, 61, 3, 1),
        (2023, 365, 12, 31),
        (9999, 365, 12, 31),
    ] {
        let r = reading(Some(year), day);
        let c = reading_to_civil(r).unwrap();
        assert_eq!(
            c.value,
            DateTime::new(year as i16, month, dom, 12, 30, 25, 340_000_000).unwrap()
        );
        let back = civil_to_calendar(c).unwrap();
        assert_eq!(back.value.day_of_year(), day);
        assert_eq!(back.value.year(), Some(year));
    }
    for year in [-9999, -1] {
        assert_eq!(
            civil_to_calendar(utc(DateTime::new(year, 1, 1, 0, 0, 0, 0).unwrap())),
            Err(TimeError::CivilYearOutOfRange { year })
        );
    }
    let mut r = reading(Some(2023), 366);
    assert!(matches!(
        reading_to_civil(r),
        Err(TimeError::OutOfRange {
            field: "day_of_year",
            ..
        })
    ));
    r.year = Some(10_000);
    r.day_of_year = 1;
    assert!(matches!(
        reading_to_civil(r),
        Err(TimeError::OutOfRange { field: "year", .. })
    ));
}
#[test]
fn unknown_scale_refused_in_every_entry_point() {
    let mut r = reading(Some(2025), 100);
    r.scale = ScaleEvidence::unknown();
    assert!(matches!(
        reading_to_civil(r),
        Err(TimeError::NeedsContext {
            context: "jiff civil scale"
        })
    ));
    let c = CalendarTime::from_parts(2025, 4, 10, 100, 12, 30, 25, 340_000_000).unwrap();
    for err in [
        absolute_to_civil(Scaled {
            value: c.into_absolute_time(),
            scale: r.scale,
        })
        .unwrap_err(),
        calendar_to_civil(Scaled {
            value: c,
            scale: r.scale,
        })
        .unwrap_err(),
        civil_to_calendar(Scaled {
            value: DateTime::new(2025, 4, 10, 0, 0, 0, 0).unwrap(),
            scale: r.scale,
        })
        .unwrap_err(),
    ] {
        assert_eq!(
            err,
            TimeError::NeedsContext {
                context: "jiff civil scale"
            }
        );
    }
}
/// Interpretations: RDG-009
#[test]
fn scales_and_assumption_provenance_are_not_relabelled() {
    for scale in [
        ScaleEvidence::standard(TimeScale::Tai),
        ScaleEvidence::standard(TimeScale::Gps),
        ScaleEvidence::assumed(TimeScale::Utc),
    ] {
        let mut r = reading(Some(2025), 100);
        r.scale = scale;
        let c = reading_to_civil(r).unwrap();
        assert_eq!(c.scale, scale);
        let back = civil_to_calendar(c).unwrap();
        assert_eq!(back.scale, scale);
        assert_eq!(back.value.seconds(), 25);
        assert_eq!(
            absolute_to_civil(Scaled {
                value: back.value.into_absolute_time(),
                scale
            })
            .unwrap(),
            c
        );
        assert_eq!(calendar_to_civil(back).unwrap(), c);
    }
}
/// Interpretations: RDG-010
#[test]
fn explicit_second60_rejected_never_clamped_or_defaulted() {
    let mut r = reading(Some(2016), 366);
    r.hours = 23;
    r.minutes = 59;
    r.seconds = 60;
    assert_eq!(
        reading_to_civil(r),
        Err(TimeError::NeedsContext {
            context: "jiff civil cannot represent second 60"
        })
    );
    // Malformed public input still reports the native validation error.
    r.day_of_year = 365;
    assert!(matches!(
        reading_to_civil(r),
        Err(TimeError::OutOfRange {
            field: "seconds",
            ..
        })
    ));
}
#[test]
fn malformed_fields_reject_before_target_construction() {
    let base = reading(Some(2025), 100);
    for (field, value) in [
        ("day_of_year", 0),
        ("hours", 24),
        ("minutes", 60),
        ("seconds", 61),
        ("nanoseconds", 1_000_000_000),
    ] {
        let mut r = base;
        match field {
            "day_of_year" => r.day_of_year = value as u16,
            "hours" => r.hours = value as u8,
            "minutes" => r.minutes = value as u8,
            "seconds" => r.seconds = value as u8,
            _ => r.nanoseconds = value,
        }
        assert!(
            matches!(reading_to_civil(r),Err(TimeError::OutOfRange {field:f,..}) if f==field),
            "{field}"
        );
    }
}
#[test]
fn explicit_table_conversion_keeps_stale_basis_outside_naive_projection() {
    use irig106_time::{LeapSecondTable, PtpTime, leap::PtpConversionStatus};
    let table = LeapSecondTable::builtin();
    for (seconds, status) in [
        (1_483_228_837, PtpConversionStatus::Covered),
        (u32::MAX, PtpConversionStatus::PossiblyOutOfDate),
    ] {
        let result = table
            .convert_ptp(PtpTime {
                seconds,
                nanoseconds: 123_456_789,
            })
            .unwrap();
        assert_eq!(result.status(), status);
        let r = result.utc().unwrap();
        let c = reading_to_civil(r).unwrap();
        assert_eq!(c.scale, r.scale);
        assert_eq!(
            civil_to_calendar(c).unwrap().value.day_of_year(),
            r.day_of_year
        );
        assert_eq!(result.status(), status); // Civil adapter does not replace table/basis.
    }
}

#[test]
fn clock_and_subsecond_extrema_are_exact() {
    for (h, m, s, ns) in [(0, 0, 0, 0), (23, 59, 59, 999_999_999)] {
        let r = CalendarReading {
            hours: h,
            minutes: m,
            seconds: s,
            nanoseconds: ns,
            ..reading(Some(2000), 60)
        };
        let c = reading_to_civil(r).unwrap();
        assert_eq!(
            c.value,
            DateTime::new(2000, 2, 29, h as i8, m as i8, s as i8, ns as i32).unwrap()
        );
        let back = civil_to_calendar(c).unwrap();
        assert_eq!(
            (
                back.value.hours(),
                back.value.minutes(),
                back.value.seconds(),
                back.value.nanoseconds()
            ),
            (h, m, s, ns)
        );
    }
    for (year, day) in [(2016, 182), (2016, 366), (2017, 181), (2017, 365)] {
        let r = CalendarReading {
            hours: 23,
            minutes: 59,
            seconds: 60,
            ..reading(Some(year), day)
        };
        assert_eq!(
            reading_to_civil(r),
            Err(TimeError::NeedsContext {
                context: "jiff civil cannot represent second 60"
            })
        );
    }
    assert_eq!(
        TimeError::CivilYearOutOfRange { year: -1 }.to_string(),
        "civil year -1 is outside 0..=9999"
    );
}
