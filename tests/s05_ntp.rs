//! RFC5905 §6/Fig4 p14; RFC1305 E.8/Fig8 pp78-79; RDG-012.
use irig106_time::{findings::*, format1::CalendarReading, ntp::*, *};
fn raw(seconds: u32, fraction: u32) -> NtpTime {
    NtpTime { seconds, fraction }
}
fn declared(era: i64) -> NtpContext {
    let mut c = NtpContext::new(1);
    c.declared = Some(EraDeclaration {
        era,
        id: "caller-era".into(),
    });
    c
}
fn pivot(session: usize, seconds: i128, nanos: u32) -> NtpPivot {
    NtpPivot::approximate(
        session,
        "approximate-date",
        SignedUnixSeconds::new(seconds),
        nanos,
    )
    .unwrap()
}
fn location() -> FindingLocation {
    FindingLocation {
        session: Some(1),
        packet: Some(9),
        channel: Some(7),
        byte_offset: None,
    }
}
/// Requirements: L1-NTP-002, L2-NTP-102, L3-NTP-102, L1-NTP-004, L2-NTP-104, L3-NTP-104
#[test]
fn rfc_figure4_dates_and_post_leap_date_have_no_tai_correction() {
    let table = LeapSecondTable::builtin();
    // era*M+seconds-2208988800; 1999 day365, leap2036 Feb8=31+8=day39.
    for (era, s, unix, year, doy) in [
        (0, 2208988800, 0, 1970, 1),
        (0, 2272060800, 63072000, 1972, 1),
        (0, 3155587200, 946598400, 1999, 365),
        (1, 63104, 2086041600, 2036, 39),
        (0, 3692304000, 1483315200, 2017, 2),
        (0, 2208902400, -86400, 1969, 365),
    ] {
        let input = raw(s, 0);
        let c = declared(era);
        let r = convert_ntp(input, &c, Some(&table));
        assert_eq!(r.raw(), input);
        assert_eq!(r.status(), NtpConversionStatus::Mapped);
        assert_eq!(r.unix_seconds().unwrap().get(), unix);
        assert_eq!(r.basis().era, Some(era));
        assert_eq!(r.basis().era_basis, Some(EraBasis::Declared));
        let utc = r.utc().unwrap();
        assert_eq!(
            (
                utc.year,
                utc.day_of_year,
                utc.hours,
                utc.minutes,
                utc.seconds
            ),
            (Some(year), doy, 0, 0, 0)
        );
        assert_eq!(utc.scale.stated(), TimeScale::Utc);
        assert_eq!(r.basis().proleptic_utc, unix < 63_072_000);
        assert_eq!(
            convert_ntp(input, &c, None).unix_seconds(),
            r.unix_seconds()
        );
    }
    // Caller model baselines are not real IERS predictions; offsets cannot affect NTP.
    for offset in [36, 38] {
        let t = LeapSecondTable::try_new(
            vec![LeapSecondEntry {
                effective_unix: UnixSeconds::new(1483228800),
                tai_utc_offset: TaiUtcOffset::new(offset),
            }],
            leap::LeapTableMetadata {
                id: "caller-baseline".into(),
                source: "fixture".into(),
                supported_start: UnixSeconds::new(1483228800),
                known_current_through: UnixSeconds::new(1814140799),
            },
        )
        .unwrap();
        assert_eq!(
            convert_ntp(raw(3692304000, 0), &declared(0), Some(&t))
                .unix_seconds()
                .unwrap()
                .get(),
            1483315200
        );
    }
}
/// Requirements: L1-NTP-002, L2-NTP-102, L3-NTP-102
#[test]
fn labelled_default_and_declared_override_keep_all_input_evidence() {
    let input = raw(63104, 0);
    let c = NtpContext::new(1);
    let assumed = convert_ntp(input, &c, None);
    assert_eq!(assumed.basis().era, Some(0));
    assert_eq!(assumed.basis().era_basis, Some(EraBasis::AssumedZero));
    assert_eq!(assumed.utc().unwrap().year, Some(1900));
    assert!(assumed.as_pivot("circular").is_err());
    let mut c = declared(1);
    c.pivots = vec![pivot(2, -2208988800, 0), pivot(1, 2086041600, 0)];
    let r = convert_ntp(input, &c, None);
    assert_eq!(r.status(), NtpConversionStatus::Mapped);
    assert_eq!(r.basis().era, Some(1));
    assert_eq!(r.basis().era_basis, Some(EraBasis::Declared));
    assert_eq!(r.basis().context, c);
    assert_eq!(r.basis().rejected_pivot, None);
    assert_eq!(
        r.as_pivot("declared-over-ignored")
            .unwrap()
            .scale()
            .origin(),
        ScaleOrigin::Standard
    );
    c.pivots.clear();
    assert_eq!(r.basis().context.pivots.len(), 2);
}
/// Requirements: L1-NTP-002, L2-NTP-102, L3-NTP-102
#[test]
fn approximate_and_dated_same_session_pivots_cross_2036() {
    let date = CalendarReading {
        year: Some(2036),
        day_of_year: 39,
        hours: 0,
        minutes: 0,
        seconds: 0,
        nanoseconds: 0,
        scale: ScaleEvidence::assumed(TimeScale::Utc),
    };
    for p in [
        pivot(1, 2086041600, 0),
        NtpPivot::dated(1, "packet-year", 77, date).unwrap(),
    ] {
        let mut c = NtpContext::new(1);
        c.pivots.push(p.clone());
        for (s, f, era, unix) in [
            (63104, 0, 1, 2086041600),
            (u32::MAX, u32::MAX, 0, 2085978495),
            (0, 1, 1, 2085978496),
        ] {
            let r = convert_ntp(raw(s, f), &c, None);
            assert_eq!(r.status(), NtpConversionStatus::Mapped);
            assert_eq!(r.basis().era, Some(era));
            assert_eq!(r.basis().era_basis, Some(EraBasis::PivotDerived));
            assert_eq!(r.unix_seconds().unwrap().get(), unix);
            assert_eq!(r.basis().context.pivots[0], p);
        }
        assert_eq!(c.pivots[0].scale().origin(), ScaleOrigin::Policy);
    }
    let p = NtpPivot::dated(1, "packet-year", 77, date).unwrap();
    assert_eq!(p.origin(), PivotOrigin::DatedReference { packet: 77 });
    assert_eq!(p.seconds().get(), 2086041600);
    assert_eq!(p.calendar(), Some(date));
    // RFC epoch dates and one prior day verify both signed calendar branches.
    for (year, day, unix) in [(1969, 365, -86400), (1970, 1, 0), (1972, 1, 63072000)] {
        let d = CalendarReading {
            year: Some(year),
            day_of_year: day,
            ..date
        };
        let p = NtpPivot::dated(1, "calendar-date", 88, d).unwrap();
        assert_eq!(p.seconds().get(), unix);
        assert_eq!(p.calendar(), Some(d));
    }
}
/// Requirements: L1-NTP-002, L2-NTP-102, L3-NTP-102, L1-NTP-003, L2-NTP-103, L3-NTP-103
#[test]
fn half_era_edges_use_unrounded_fractions_and_conflicts_never_fallback() {
    const H: i128 = 2147483648;
    const E: i128 = 2208988800;
    for sign in [-1, 1] {
        let mut c = NtpContext::new(1);
        c.pivots.push(pivot(1, 1 + sign * H - E, 0));
        let r = convert_ntp(raw(1, 0), &c, None);
        assert_eq!(
            r.status(),
            NtpConversionStatus::NeedsContext(EraIssue::HalfEraTie)
        );
        assert_eq!(r.basis().rejected_pivot, Some(0));
        assert_eq!(r.unix_seconds(), None);
        // Raw fraction1 floors to0ns, but breaks the era tie in the exact direction.
        let r = convert_ntp(raw(1, 1), &c, None);
        assert_eq!(r.basis().era, Some(if sign == 1 { 0 } else { -1 }));
        c.pivots = vec![pivot(1, 1 + sign * H - E, 1)];
        assert_eq!(
            convert_ntp(raw(1, 0), &c, None).basis().era,
            Some(if sign == 1 { 1 } else { 0 })
        );
    }
    let mut c = NtpContext::new(1);
    c.pivots = vec![pivot(1, 0, 0), pivot(1, 4294967296, 0)];
    let r = convert_ntp(raw(2208988800, 0), &c, None);
    assert_eq!(
        r.status(),
        NtpConversionStatus::NeedsContext(EraIssue::ConflictingPivots)
    );
    assert_eq!(r.basis().rejected_pivot, Some(1));
    assert_eq!(r.unix_seconds(), None);
    assert!(r.findings(location(), None).unwrap().is_empty());
    c.pivots = vec![pivot(2, 0, 0)];
    assert_eq!(
        convert_ntp(raw(2208988800, 0), &c, None).status(),
        NtpConversionStatus::NeedsContext(EraIssue::CrossSession)
    );
}
/// Requirements: L1-NTP-003, L2-NTP-103, L3-NTP-103, L1-NTP-004, L2-NTP-104, L3-NTP-104, L1-RDG-002, L2-RDG-212, L3-RDG-212
/// Interpretations: RDG-012
#[test]
fn sentinel_precedes_era_and_rounding_but_nonzero_fraction_is_time() {
    for c in [NtpContext::new(0), declared(0), declared(i64::MAX)] {
        let input = raw(0, 0);
        let r = convert_ntp(input, &c, None);
        assert_eq!(r.raw(), input);
        assert_eq!(r.status(), NtpConversionStatus::UnknownUnsynchronized);
        assert_eq!(r.unix_seconds(), None);
        assert_eq!(r.utc(), None);
        assert_eq!(r.basis().era, None);
        assert!(r.as_pivot("zero-sentinel").is_err());
        let f = r.findings(location(), None).unwrap();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].code().number(), 29);
        assert_eq!(f[0].severity(), Severity::Warning);
        assert_eq!(f[0].evidence().validity(), ValueValidity::Unusable);
        assert_eq!(f[0].evidence().ntp().unwrap().raw, input);
        assert_eq!(f[0].evidence().ntp().unwrap().mapped_unix, None);
        assert_eq!(
            r.findings(location(), Some(Severity::Information)).unwrap()[0].severity(),
            Severity::Information
        );
    }
    for (fraction, ns) in [
        (0, 0),
        (1, 0),
        (4, 0),
        (5, 1),
        (0x80000000, 500000000),
        (u32::MAX, 999999999),
    ] {
        let input = raw(2208988800, fraction);
        let r = convert_ntp(input, &declared(0), None);
        assert_eq!(r.raw(), input);
        assert_eq!(r.nanoseconds(), ns);
        assert_eq!(r.unix_seconds().unwrap().get(), 0);
        assert_eq!(r.basis().rounding, NtpRounding::FloorNanoseconds);
        assert_eq!(r.basis().quantization, NanosDuration::new(1));
    }
    let r = convert_ntp(raw(0, 1), &declared(0), None);
    assert_eq!(r.status(), NtpConversionStatus::Mapped);
    assert_eq!(r.nanoseconds(), 0);
    assert_eq!(r.unix_seconds().unwrap().get(), -2208988800);
    assert!(r.findings(location(), None).unwrap().is_empty());
}
/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104, L1-TAI-007, L2-TAI-107, L3-TAI-107, L1-RDG-002, L2-RDG-212, L3-RDG-212
/// Interpretations: RDG-012, RDG-010
#[test]
fn covered_duplicate_coordinate_is_ambiguous_value_kept_not_a_read_leap() {
    let table = LeapSecondTable::builtin();
    // RFC1305 Fig8 numerical relation [B,B+1); use actual2016 insertion B1483228800.
    for (s, f, ambiguous) in [
        (3692217599, u32::MAX, false),
        (3692217600, 0, true),
        (3692217600, 1, true),
        (3692217600, 0x80000000, true),
        (3692217600, u32::MAX, true),
        (3692217601, 0, false),
    ] {
        let input = raw(s, f);
        let r = convert_ntp(input, &declared(0), Some(&table));
        assert_eq!(r.raw(), input);
        assert_eq!(r.unix_seconds().unwrap().get(), i128::from(s) - 2208988800);
        assert_eq!(
            r.basis().leap,
            if ambiguous {
                NtpLeapStatus::Ambiguous
            } else {
                NtpLeapStatus::UnambiguousCovered
            }
        );
        assert!(!r.utc().unwrap().is_leap_second());
        let findings = r.findings(location(), None).unwrap();
        assert_eq!(findings.iter().any(|f| f.code().number() == 30), ambiguous);
        assert!(!findings.iter().any(|f| f.code().number() == 22));
        if ambiguous {
            let ordinary = r.utc().unwrap();
            let leap = r.possible_leap().unwrap();
            assert_eq!(
                (ordinary.year, ordinary.day_of_year, ordinary.seconds),
                (Some(2017), 1, 0)
            );
            assert_eq!(
                (
                    leap.year,
                    leap.day_of_year,
                    leap.hours,
                    leap.minutes,
                    leap.seconds
                ),
                (Some(2016), 366, 23, 59, 60)
            );
            assert_eq!(ordinary.nanoseconds, leap.nanoseconds);
            assert_eq!(
                r.basis().ambiguity_transition.unwrap().effective_unix.get(),
                1483228800
            );
            let f = &findings[0];
            assert_eq!(f.code().number(), 30);
            assert_eq!(f.severity(), Severity::Warning);
            assert_eq!(f.evidence().validity(), ValueValidity::Kept);
            assert!(
                f.evidence()
                    .raw_fields()
                    .iter()
                    .any(|f| f.name == "nanoseconds_floor"
                        && f.observed == RawValue::Unsigned(r.nanoseconds().into()))
            );
            assert!(
                f.evidence()
                    .raw_fields()
                    .iter()
                    .any(|f| f.name == "ntp_era" && f.observed == RawValue::Signed(0))
            );
            assert_eq!(f.evidence().location(), location());
            assert_eq!(f.evidence().ntp().unwrap().raw, input);
            assert_eq!(
                f.evidence().ntp().unwrap().mapped_unix.unwrap().get(),
                1483228800
            );
            assert_eq!(
                r.findings(location(), Some(Severity::Information)).unwrap()[0].severity(),
                Severity::Information
            );
        } else {
            assert_eq!(r.possible_leap(), None);
        }
    }
}
/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104, L1-RDG-002, L2-RDG-212, L3-RDG-212
/// Interpretations: RDG-012
#[test]
fn missing_stale_and_partial_coverage_keep_utc_without_false_certainty() {
    let builtin = LeapSecondTable::builtin();
    let empty = LeapSecondTable::empty();
    for table in [None, Some(&empty)] {
        let r = convert_ntp(raw(3692304000, 0), &declared(0), table);
        assert_eq!(r.unix_seconds().unwrap().get(), 1483315200);
        assert_eq!(r.basis().leap, NtpLeapStatus::UncertainCoverage);
        assert_eq!(r.basis().table.as_ref(), table.map(|t| t.metadata()));
        assert!(r.findings(location(), None).unwrap().is_empty());
    }
    for (s, era, unix) in [
        (2208988800, 0, 0),
        (63104, 1, 2086041600),
        (4023129600, 0, 1814140800),
    ] {
        let r = convert_ntp(raw(s, 0), &declared(era), Some(&builtin));
        assert_eq!(r.unix_seconds().unwrap().get(), unix);
        assert!(r.utc().is_some());
        assert_eq!(r.basis().leap, NtpLeapStatus::UncertainCoverage);
        assert_eq!(r.basis().table.as_ref(), Some(builtin.metadata()));
        assert_eq!(r.basis().final_transition, builtin.final_transition());
        assert!(r.findings(location(), None).unwrap().is_empty());
    }
    // A hypothetical negative step is not the duplicated positive-insertion window.
    let mut future = builtin.metadata().clone();
    future.id = "negative-test".into();
    future.known_current_through = UnixSeconds::new(1861920000);
    let negative = builtin
        .extended(
            vec![LeapSecondEntry {
                effective_unix: UnixSeconds::new(1830297600),
                tai_utc_offset: TaiUtcOffset::new(36),
            }],
            future,
        )
        .unwrap();
    let r = convert_ntp(raw(4039286400, 0), &declared(0), Some(&negative));
    assert_eq!(r.basis().leap, NtpLeapStatus::UnambiguousCovered);
    assert_eq!(r.possible_leap(), None);
    assert!(r.findings(location(), None).unwrap().is_empty());
    let mut info = builtin.metadata().clone();
    info.supported_start = UnixSeconds::new(1483228800);
    let narrow = LeapSecondTable::try_new(builtin.entries().to_vec(), info).unwrap();
    let r = convert_ntp(raw(3692217600, 0), &declared(0), Some(&narrow));
    assert_eq!(r.basis().leap, NtpLeapStatus::UncertainCoverage);
    assert_eq!(r.possible_leap(), None);
    assert!(r.findings(location(), None).unwrap().is_empty());
}
/// Requirements: L1-NTP-002, L2-NTP-102, L3-NTP-102, L1-NTP-004, L2-NTP-104, L3-NTP-104
#[test]
fn invalid_evidence_and_range_limits_are_explicit_not_guessed() {
    let ordinary = CalendarReading {
        year: Some(2036),
        day_of_year: 39,
        hours: 0,
        minutes: 0,
        seconds: 0,
        nanoseconds: 0,
        scale: ScaleEvidence::standard(TimeScale::Utc),
    };
    for scale in [TimeScale::Tai, TimeScale::Gps, TimeScale::Unknown] {
        assert!(
            NtpPivot::dated(
                1,
                "bad-scale",
                1,
                CalendarReading {
                    scale: ScaleEvidence::standard(scale),
                    ..ordinary
                }
            )
            .is_err()
        );
    }
    assert!(
        NtpPivot::dated(
            1,
            "bad-calendar",
            1,
            CalendarReading {
                day_of_year: 0,
                ..ordinary
            }
        )
        .is_err()
    );
    assert!(NtpPivot::approximate(1, "", SignedUnixSeconds::new(0), 0).is_err());
    assert!(
        NtpPivot::dated(
            1,
            "no-year",
            1,
            CalendarReading {
                year: None,
                ..ordinary
            }
        )
        .is_err()
    );
    let leap_date = CalendarReading {
        year: Some(2016),
        day_of_year: 366,
        hours: 23,
        minutes: 59,
        seconds: 60,
        ..ordinary
    };
    let leap_pivot = NtpPivot::dated(1, "explicit-leap", 1, leap_date).unwrap();
    assert_eq!(leap_pivot.seconds().get(), 1483228800);
    assert_eq!(leap_pivot.calendar(), Some(leap_date));
    assert!(NtpPivot::approximate(1, "bad-nanos", SignedUnixSeconds::new(0), 1000000000).is_err());
    assert!(NtpPivot::approximate(0, "no-session", SignedUnixSeconds::new(0), 0).is_err());
    assert_eq!(
        convert_ntp(raw(1, 0), &NtpContext::new(0), None).status(),
        NtpConversionStatus::NeedsContext(EraIssue::InvalidSession)
    );
    let mut c = declared(0);
    c.declared.as_mut().unwrap().id.clear();
    assert_eq!(
        convert_ntp(raw(1, 0), &c, None).status(),
        NtpConversionStatus::NeedsContext(EraIssue::InvalidDeclaration)
    );
    c = NtpContext::new(1);
    c.pivots.push(pivot(1, i128::MAX, 0));
    let r = convert_ntp(raw(1, 0), &c, None);
    assert_eq!(
        r.status(),
        NtpConversionStatus::NeedsContext(EraIssue::QuantityRange)
    );
    assert_eq!(r.unix_seconds(), None);
    assert_eq!(r.raw(), raw(1, 0));
    let r = convert_ntp(raw(1, 0), &declared(i64::MAX), None);
    assert_eq!(
        r.unix_seconds().unwrap().get(),
        i128::from(i64::MAX) * 4294967296 + 1 - 2208988800
    );
    assert_eq!(r.utc(), None);
    assert!(r.calendar_error().is_some());
    assert_eq!(
        convert_ntp(raw(2208988800, 0), &declared(-1), None)
            .unix_seconds()
            .unwrap()
            .get(),
        -4294967296
    );
}
/// Requirements: L1-NTP-002, L2-NTP-102, L3-NTP-102, L1-NTP-003, L2-NTP-103, L3-NTP-103
#[test]
fn resolved_ntp_pivots_keep_raw_precision_and_scale_assumptions() {
    // An explicitly resolved source at half-era+one raw fraction tick. Its0ns
    // rounded view must not turn a unique next-era decision into a half-era tie.
    let parent = convert_ntp(raw(2147483649, 1), &declared(0), None);
    let p = parent.as_pivot("resolved-reference").unwrap();
    assert_eq!(p.fraction(), PivotFraction::NtpUnits(1));
    assert_eq!(
        p.origin(),
        PivotOrigin::ResolvedNtp {
            era: 0,
            kind: EraBasis::Declared
        }
    );
    let mut c = NtpContext::new(1);
    c.pivots.push(p);
    assert_eq!(convert_ntp(raw(1, 0), &c, None).basis().era, Some(1));
    let mut c = NtpContext::new(1);
    c.pivots.push(pivot(1, 2086041600, 0));
    let r = convert_ntp(raw(63104, 1), &c, None);
    assert_eq!(
        r.as_pivot("retained-assumption").unwrap().scale().origin(),
        ScaleOrigin::Policy
    );
}

/// Requirements: L1-NTP-004, L2-NTP-104, L3-NTP-104, L1-FND-001, L2-FND-101, L3-FND-101, L1-RDG-002, L2-RDG-212, L3-RDG-212
/// Interpretations: RDG-012
#[test]
fn tf030_nonzero_era_keeps_exact_evidence_and_unix_mapping() {
    // Synthetic caller model, NOT an IERS prediction: +1 at2037-01-01.
    // Gregorian days1970->2037:67*365+17 leap days=24472;
    // B=24472*86400=2114380800. RFC5905 §6/Fig4 era equation:
    // raw=B+2208988800-4294967296=28402304, era1. Fraction0x80000000=0.5s.
    let builtin = LeapSecondTable::builtin();
    let mut info = builtin.metadata().clone();
    info.id = "hypothetical-2037-insertion".into();
    info.source = "synthetic caller fixture, not IERS future data".into();
    info.known_current_through = UnixSeconds::new(2_145_916_800); // 2038-01-01
    let table = builtin
        .extended(
            vec![LeapSecondEntry {
                effective_unix: UnixSeconds::new(2_114_380_800),
                tai_utc_offset: TaiUtcOffset::new(38),
            }],
            info,
        )
        .unwrap();
    let input = raw(28_402_304, 0x80000000);
    let converted = convert_ntp(input, &declared(1), Some(&table));
    assert_eq!(converted.status(), NtpConversionStatus::Mapped);
    assert_eq!(converted.basis().era, Some(1));
    assert_eq!(converted.basis().leap, NtpLeapStatus::Ambiguous);
    assert_eq!(converted.unix_seconds().unwrap().get(), 2_114_380_800);
    let utc = converted.utc().unwrap();
    assert_eq!(
        (utc.year, utc.day_of_year, utc.nanoseconds),
        (Some(2037), 1, 500_000_000)
    );
    let findings = converted.findings(location(), None).unwrap();
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding.code().number(), 30);
    assert_eq!(finding.severity(), Severity::Warning);
    assert_eq!(finding.evidence().validity(), ValueValidity::Kept);
    assert_eq!(finding.evidence().ntp().unwrap().raw, input);
    assert_eq!(
        finding.evidence().ntp().unwrap().mapped_unix.unwrap().get(),
        2_114_380_800
    );
    let era = finding
        .evidence()
        .raw_fields()
        .iter()
        .find(|field| field.name == "ntp_era")
        .unwrap();
    assert_eq!(era.observed, RawValue::Signed(1));
}
