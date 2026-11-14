//! Hand-derived IERS C72/IRIG106-24R1 11.2.3.3 (11-26/PDF34) fixtures.
use irig106_time::{findings::*, leap::*, network_time::*, *};
fn entry(at: u64, offset: i32) -> LeapSecondEntry {
    LeapSecondEntry {
        effective_unix: UnixSeconds::new(at),
        tai_utc_offset: TaiUtcOffset::new(offset),
    }
}
fn metadata(id: &str, start: u64, through: u64) -> LeapTableMetadata {
    LeapTableMetadata {
        id: id.into(),
        source: "caller test fixture".into(),
        supported_start: UnixSeconds::new(start),
        known_current_through: UnixSeconds::new(through),
    }
}
fn raw(seconds: u64, ns: u32) -> PtpTime {
    PtpTime {
        seconds: u32::try_from(seconds).unwrap(),
        nanoseconds: ns,
    }
}
fn location() -> FindingLocation {
    FindingLocation {
        session: Some(1),
        packet: Some(8),
        channel: Some(7),
        byte_offset: None,
    }
}
/// Requirements: L1-TAI-002, L2-TAI-102, L3-TAI-102
#[test]
fn all_builtin_transitions_match_the_independent_iers_list_and_currency() {
    let table = LeapSecondTable::builtin();
    let mut count = 0;
    for (expected, line) in table.entries().iter().zip(
        include_str!("fixtures/iers-bulletin-c72.txt")
            .lines()
            .filter(|l| !l.starts_with('#')),
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        let mjd: i64 = parts[0].split('.').next().unwrap().parse().unwrap();
        // MJD40587 = 1970-01-01, 86400 nominal UTC seconds/day.
        assert_eq!(
            expected.effective_unix.get(),
            ((mjd - 40587) * 86400) as u64
        );
        assert_eq!(
            expected.tai_utc_offset.get(),
            parts[4].parse::<i32>().unwrap()
        );
        count += 1;
    }
    assert_eq!(count, 28);
    assert_eq!(table.len(), 28);
    assert!(table.metadata().id.contains("C72"));
    assert!(table.metadata().source.contains("hpiers.obspm.fr"));
    assert_eq!(table.metadata().supported_start.get(), 63_072_000);
    // IERS file expires2027-06-28: conservative coverage includes all of June27,
    // through integer second1814140799 (inclusive of its nanosecond fractions).
    assert_eq!(table.metadata().known_current_through.get(), 1_814_140_799);
    assert_eq!(
        table.final_transition(),
        Some(UnixSeconds::new(1_483_228_800))
    );
}
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101, L1-TAI-003, L2-TAI-103, L3-TAI-103
#[test]
fn invalid_tables_are_rejected_without_sorting_or_repair() {
    let info = metadata("test", 63_072_000, 100_000_000);
    for entries in [
        vec![entry(63_072_000, 10), entry(63_072_000, 10)],
        vec![entry(63_072_000, 10), entry(63_072_000, 11)],
        vec![entry(78_796_800, 11), entry(63_072_000, 10)],
        vec![entry(63_072_000, 10), entry(78_796_800, 12)],
        vec![entry(63_072_000, 10), entry(63_072_001, 9)],
        vec![entry(100_000_001, 10)],
    ] {
        assert!(matches!(
            LeapSecondTable::try_new(entries, info.clone()),
            Err(TimeError::InvalidLeapTable { .. })
        ));
    }
    for bad in [
        metadata("", 63_072_000, 100_000_000),
        metadata("bad", 0, 100_000_000),
        metadata("bad", 63_072_000, 1),
    ] {
        assert!(LeapSecondTable::try_new(vec![], bad).is_err());
    }
}
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101, L1-PTP-004, L2-PTP-104, L3-PTP-104
/// Interpretations: RDG-010
#[test]
fn every_positive_transition_selects_on_tai_and_preserves_second60() {
    let table = LeapSecondTable::builtin();
    for pair in table.entries().windows(2) {
        let old = pair[0].tai_utc_offset;
        let new = pair[1].tai_utc_offset;
        let utc = pair[1].effective_unix.get();
        let tai = utc + new.get() as u64;
        for (t, ns, second, offset, unix) in [
            (tai - 2, 999_999_999, 59, old, utc - 1),
            (tai - 1, 0, 60, old, utc),
            (tai - 1, 999_999_999, 60, old, utc),
            (tai, 0, 0, new, utc),
        ] {
            let input = raw(t, ns);
            let converted = table.convert_ptp(input).unwrap();
            assert_eq!(converted.raw(), input);
            assert_eq!(converted.status(), PtpConversionStatus::Covered);
            let calendar = converted.utc().unwrap();
            assert_eq!((calendar.seconds, calendar.nanoseconds), (second, ns));
            assert_eq!(calendar.scale.stated(), TimeScale::Utc);
            assert_eq!(converted.basis().offset, Some(offset));
            assert_eq!(converted.unix_seconds().unwrap().get(), i128::from(unix));
            assert_eq!(converted.basis().leap_second, second == 60);
            assert_eq!(
                converted
                    .findings(location(), None)
                    .unwrap()
                    .iter()
                    .any(|f| f.code().number() == 22),
                second == 60
            );
        }
    }
    for (tai, year, doy) in [(78_796_810, 1972, 182), (1_483_228_836, 2016, 366)] {
        let r = table.convert_ptp(raw(tai, 0)).unwrap();
        let c = r.utc().unwrap();
        assert_eq!(
            (c.year, c.day_of_year, c.hours, c.minutes, c.seconds),
            (Some(year), doy, 23, 59, 60)
        );
    }
}
/// Requirements: L1-TAI-004, L2-TAI-104, L3-TAI-104, L1-PTP-004, L2-PTP-104, L3-PTP-104
#[test]
fn coverage_end_is_not_last_transition_and_stale_keeps_value_and_warning() {
    let table = LeapSecondTable::builtin();
    for (unix, ns, status) in [
        (1_800_000_000, 0, PtpConversionStatus::Covered),
        (1_814_140_799, 999_999_999, PtpConversionStatus::Covered),
        (1_814_140_800, 0, PtpConversionStatus::PossiblyOutOfDate),
        (1_893_456_000, 456, PtpConversionStatus::PossiblyOutOfDate),
    ] {
        let input = raw(unix + 37, ns);
        let r = table.convert_ptp(input).unwrap();
        assert_eq!(r.raw(), input);
        assert_eq!(r.status(), status);
        assert_eq!(r.unix_seconds().unwrap().get(), i128::from(unix));
        assert_eq!(r.utc().unwrap().nanoseconds, ns);
        assert_eq!(r.basis().offset, Some(TaiUtcOffset::new(37)));
        assert_eq!(r.basis().table, table.metadata().clone());
        let findings = r.findings(location(), None).unwrap();
        assert_eq!(
            findings.iter().any(|f| f.code().number() == 21),
            status == PtpConversionStatus::PossiblyOutOfDate
        );
        if status == PtpConversionStatus::PossiblyOutOfDate {
            let f = findings.iter().find(|f| f.code().number() == 21).unwrap();
            assert_eq!(f.severity(), Severity::Warning);
            assert_eq!(f.evidence().validity(), ValueValidity::Kept);
            assert_eq!(f.evidence().location(), location());
            assert!(
                f.evidence()
                    .raw_fields()
                    .iter()
                    .any(|f| f.name == "tai_seconds"
                        && f.observed == RawValue::Unsigned(input.seconds.into()))
            );
            assert!(f.evidence().raw_fields().iter().any(
                |f| f.name == "tai_nanoseconds" && f.observed == RawValue::Unsigned(ns.into())
            ));
            assert_eq!(
                r.findings(location(), Some(Severity::Information)).unwrap()[0].severity(),
                Severity::Information
            );
        }
    }
}
/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101, L1-TAI-004, L2-TAI-104, L3-TAI-104
#[test]
fn empty_and_pre1972_have_missing_context_not_a_default_offset() {
    for table in [LeapSecondTable::empty(), LeapSecondTable::builtin()] {
        for seconds in [0, 63_072_009] {
            let input = raw(seconds, 123);
            let r = table.convert_ptp(input).unwrap();
            assert_eq!(r.raw(), input);
            assert_eq!(r.status(), PtpConversionStatus::MissingOffset);
            assert_eq!(r.utc(), None);
            assert_eq!(r.unix_seconds(), None);
            assert_eq!(r.basis().offset, None);
            assert!(r.findings(location(), None).unwrap().is_empty());
        }
    }
    assert!(
        LeapSecondTable::builtin()
            .offset_at_unix(UnixSeconds::new(63_071_999))
            .is_err()
    );
    assert_eq!(
        raw(0, 0).to_signed_utc_seconds(TaiUtcOffset::new(10)).get(),
        -10
    );
    assert!(
        LeapSecondTable::builtin()
            .convert_ptp(raw(1_500_000_000, 1_000_000_000))
            .is_err()
    );
    assert!(LeapSecondTable::builtin().offset_for_f1(2017, 0).is_err());
    assert!(LeapSecondTable::builtin().offset_for_f1(1900, 1).is_err());
}
/// Requirements: L1-TAI-003, L2-TAI-103, L3-TAI-103
#[test]
fn extensions_and_replacements_are_validated_and_previous_basis_is_immutable() {
    let old = LeapSecondTable::builtin();
    let before = old.convert_ptp(raw(1_800_000_037, 0)).unwrap();
    let info = metadata("future-test", 63_072_000, 1_861_920_000);
    let extended = old
        .extended(
            vec![entry(1_483_228_800, 37), entry(1_830_297_600, 38)],
            info.clone(),
        )
        .unwrap();
    assert_eq!(extended.len(), 29);
    assert_eq!(old.len(), 28);
    assert_ne!(extended.metadata().id, before.basis().table.id);
    assert_eq!(
        before.basis().final_transition,
        Some(UnixSeconds::new(1_483_228_800))
    );
    assert!(
        old.extended(vec![entry(1_830_297_600, 39)], info.clone())
            .is_err()
    );
    assert!(
        old.extended(
            vec![entry(1_830_297_600, 38), entry(1_483_228_800, 37)],
            info.clone()
        )
        .is_err()
    );
    assert!(
        old.extended(vec![entry(1_483_228_800, 38)], info.clone())
            .is_err()
    );
    let mut changed = old.entries().to_vec();
    changed[26].effective_unix = UnixSeconds::new(1_451_606_400); // shift2015 transition to2016, same offsets: historical conflict
    assert!(old.replaced(changed, info.clone()).is_err());
    assert_eq!(
        old.replaced(extended.entries().to_vec(), info)
            .unwrap()
            .len(),
        29
    );
    // Negative leap is hypothetical, not an IERS historical claim: offset37->36.
    let negative = old
        .extended(
            vec![entry(1_830_297_600, 36)],
            metadata("negative-test", 63_072_000, 1_861_920_000),
        )
        .unwrap();
    for (tai, second, offset) in [(1_830_297_635, 58, 37), (1_830_297_636, 0, 36)] {
        let r = negative.convert_ptp(raw(tai, 0)).unwrap();
        assert_eq!(r.utc().unwrap().seconds, second);
        assert_eq!(r.basis().offset, Some(TaiUtcOffset::new(offset)));
        assert!(!r.basis().leap_second);
    }
}

/// Requirements: L1-TAI-001, L2-TAI-101, L3-TAI-101, L1-TAI-004, L2-TAI-104, L3-TAI-104
#[test]
fn narrowed_start_uses_tai_boundary_not_repeated_unix_coordinate() {
    let builtin = LeapSecondTable::builtin();
    let mut info = builtin.metadata().clone();
    info.id = "starts2017".into();
    info.supported_start = UnixSeconds::new(1_483_228_800);
    let t = LeapSecondTable::try_new(builtin.entries().to_vec(), info).unwrap();
    assert_eq!(
        t.convert_ptp(raw(1_483_228_836, 999_999_999))
            .unwrap()
            .status(),
        PtpConversionStatus::MissingOffset
    );
    assert_eq!(
        t.convert_ptp(raw(1_483_228_837, 0)).unwrap().status(),
        PtpConversionStatus::Covered
    );
    let mut info = builtin.metadata().clone();
    info.source.clear();
    assert!(LeapSecondTable::try_new(vec![], info).is_err());
}
