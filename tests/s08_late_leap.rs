//! Partial S08 clauses: Fig11-13/14 (11-25/PDF33), RDG-010,
//! contract5.3 and owner2026-11-15 YEAR-PHASE rule. Hand-derived words.
use irig106_time::{leap::LeapSecondTable, stream::*, timeline::*, years::*, *};
const S: u64 = 10_000_000;
fn wire(dmy: bool, midnight: bool, sec: u8, caller: u16, normalize: bool) -> ReferenceInput {
    // BCD seconds58/59/60 live in word0's high byte; HHMM2359.
    // DOY366=0366; DMY MMDD=1231/0101, YYYY=2016/2017.
    let second = (sec / 10) << 4 | (sec % 10);
    let mut bytes = if midnight {
        vec![
            1,
            if dmy { 2 } else { 0 },
            0,
            0,
            0,
            second,
            0,
            0,
            1,
            if dmy { 1 } else { 0 },
        ]
    } else {
        vec![
            1,
            if dmy { 3 } else { 1 },
            0,
            0,
            0,
            second,
            0x59,
            0x23,
            if dmy { 0x31 } else { 0x66 },
            if dmy { 0x12 } else { 3 },
        ]
    };
    if dmy {
        bytes.extend(if midnight { [0x17, 0x20] } else { [0x16, 0x20] });
    }
    let mut r = ReferenceInput::format1(
        &bytes,
        bytes.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(YearContext {
        caller: Some(caller),
        recording: None,
    })
    .unwrap();
    if normalize {
        if let ReferenceValue::Readable(t, valid) = r.value {
            // The explicit caller date needed by normalization; never normalize
            // a second60 label into AbsoluteTime or silently ignore an error.
            r.value = ReferenceValue::Readable(
                t.with_year(Some(if midnight { 2017 } else { 2016 }))
                    .unwrap(),
                valid,
            );
            if !dmy {
                r.year_origin = YearOrigin::Caller;
            }
            r = r.normalize_to_tai(LeapSecondTable::builtin()).unwrap();
        }
    }
    r
}
fn packet(id: u64, ticks: u64, r: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 1,
        rtc: Rtc::from_raw(ticks),
        kind: if r.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: false,
        flags: 0,
        reference: r,
    }
}
fn policy(mode: StreamMode) -> StreamPolicy {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 4,
            bytes: 400,
            references: 3,
        },
    );
    p.mode = mode;
    p.offline.channel = Some(1);
    p
}
fn late(leap: bool, mode: Option<StreamMode>) {
    let first = packet(1, S, Some(wire(false, true, 0, 2017, false)));
    let last = packet(
        2,
        0,
        Some(wire(false, false, if leap { 60 } else { 59 }, 2017, false)),
    );
    if let Some(mode) = mode {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        s.observe(first, 0, 100, false).unwrap();
        let batch = s.observe(last, 0, 100, false).unwrap();
        assert!(
            !batch
                .findings
                .iter()
                .any(|f| matches!(f.code, 4 | 7 | 17 | 32))
        );
        if !leap {
            let answer = s
                .observe(packet(3, 0, None), 0, 100, false)
                .unwrap()
                .answers
                .remove(0);
            assert_eq!(answer.time.unwrap().year(), Some(2016));
            assert_eq!(answer.time.unwrap().day_of_year(), 366);
        }
    } else {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(first, false).unwrap();
        let key = b.observe(last, false).unwrap();
        let t = b.finish().unwrap();
        let r = &t.sessions()[0].references[1];
        assert_eq!(r.input.year_basis.selected().unwrap().year, 2016);
        assert_eq!(r.input.year_basis.rollovers(), -1);
        assert!(
            !t.findings()
                .iter()
                .any(|f| matches!(f.code, 4 | 7 | 17 | 32))
        );
        if leap {
            assert_eq!(r.input.value, ReferenceValue::NeedsContext);
        } else {
            let answer = t.lookup(key).unwrap().time.unwrap();
            assert_eq!((answer.year(), answer.day_of_year()), (Some(2016), 366));
            assert!(r.admitted);
        }
    }
}
fn span(dmy: bool, normalize: bool, mode: Option<StreamMode>) {
    span_with_correction(dmy, normalize, mode, 0);
}
fn span_with_correction(dmy: bool, normalize: bool, mode: Option<StreamMode>, correction: u8) {
    let packets = [
        packet(1, 0, Some(wire(dmy, false, 59, 2016, normalize))),
        packet(2, S, Some(wire(dmy, false, 60, 2016, normalize))),
        packet(3, 2 * S, Some(wire(dmy, true, correction, 2016, normalize))),
        packet(
            4,
            3 * S,
            Some(wire(dmy, true, correction + 1, 2016, normalize)),
        ),
    ];
    if let Some(mode) = mode {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        let mut findings = Vec::new();
        for p in packets {
            findings.extend(s.observe(p, 0, 100, false).unwrap().findings);
        }
        let jumps: Vec<_> = findings
            .iter()
            .filter(|f| f.code == 17)
            .map(|f| (f.packet, f.delta_ns))
            .collect();
        if correction == 0 {
            assert!(jumps.is_empty(), "{jumps:?}");
        } else {
            assert!(!jumps.is_empty());
            assert!(
                jumps
                    .iter()
                    .all(|(_, d)| *d == Some(i128::from(correction) * 1_000_000_000)),
                "{jumps:?}"
            );
        }
        assert_eq!(
            s.observe(packet(5, 3 * S, None), 0, 100, false)
                .unwrap()
                .answers[0]
                .time
                .unwrap()
                .year(),
            Some(2017)
        );
    } else {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in packets {
            b.observe(p, false).unwrap();
        }
        let t = b.finish().unwrap();
        let jumps: Vec<_> = t
            .findings()
            .iter()
            .filter(|f| f.code == 17)
            .map(|f| (f.packet, f.delta_ns))
            .collect();
        if correction == 0 {
            assert!(jumps.is_empty(), "{jumps:?}");
        } else {
            assert!(!jumps.is_empty());
            assert!(
                jumps
                    .iter()
                    .all(|(_, d)| *d == Some(i128::from(correction) * 1_000_000_000)),
                "{jumps:?}"
            );
        }
        assert!(
            t.quality(t.sessions()[0].id, 1)
                .unwrap()
                .drift_ppm
                .is_some_and(|p| (p - f64::from(correction) * 250_000.).abs() < 0.0001)
        );
        assert_eq!(
            t.sessions()[0].references[0]
                .input
                .year_basis
                .observed_leap_adjustment()
                .get(),
            0
        );
        assert_eq!(
            t.sessions()[0].references[2]
                .input
                .year_basis
                .observed_leap_adjustment()
                .get(),
            1_000_000_000
        );
        assert!(
            matches!(t.sessions()[0].references[2].input.value,ReferenceValue::Readable(t,_) if t.year()==Some(2017))
        );
        assert_eq!(
            t.sessions()[0].references[3]
                .input
                .year_basis
                .observed_leap_adjustment()
                .get(),
            1_000_000_000
        );
    }
}
macro_rules! scenarios {
    ($name:ident,$body:ident,$a:expr,$b:expr) => {
        mod $name {
            use super::*;
            #[test]
            fn offline() {
                $body($a, $b, None);
            }
            #[test]
            fn standard() {
                $body($a, $b, Some(StreamMode::Standard));
            }
            #[test]
            fn interpolate() {
                $body($a, $b, Some(StreamMode::Interpolate));
            }
        }
    };
}
// Late helper has one parameter fewer; its three paths are explicit.
#[test]
fn q14_offline() {
    late(false, None);
}
#[test]
fn q14_standard() {
    late(false, Some(StreamMode::Standard));
}
#[test]
fn q14_interpolate() {
    late(false, Some(StreamMode::Interpolate));
}
#[test]
fn q13_offline() {
    late(true, None);
}
#[test]
fn q13_standard() {
    late(true, Some(StreamMode::Standard));
}
#[test]
fn q13_interpolate() {
    late(true, Some(StreamMode::Interpolate));
}
scenarios!(q5_doy, span, false, false);
scenarios!(q5n_doy_normalized, span, false, true);
scenarios!(q15n_dmy_normalized, span, true, true);

// Post-fix controls: do not hide a real clock correction or double count a table.
fn corrected(dmy: bool, normalize: bool, mode: Option<StreamMode>) {
    span_with_correction(dmy, normalize, mode, 5);
}
scenarios!(q5_real_five_second_correction, corrected, false, false);
scenarios!(q15_dmy_native, span, true, false);

// Partial RDG-010/COR-110 clauses; no whole-requirement marker. Fig11-13
// (11-25/PDF33): day365=0x0365,182=0x0182, seconds59/60=0x5900/0x6000.
fn doy(day: u16, sec: u8, midnight: bool, year: Option<u16>) -> ReferenceInput {
    let bcd = |n: u16| (((n / 10) << 4) | (n % 10)) as u8;
    let bytes = [
        1,
        if year.is_some_and(|y| y % 4 == 0) || year.is_none() {
            1
        } else {
            0
        },
        0,
        0,
        0,
        bcd(sec.into()),
        if midnight { 0 } else { 0x59 },
        if midnight { 0 } else { 0x23 },
        bcd(day % 100),
        (day / 100) as u8,
    ];
    ReferenceInput::format1(
        &bytes,
        bytes.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
    .with_year_context(YearContext {
        caller: year,
        recording: None,
    })
    .unwrap()
}
fn forbidden_label(year: u16, day: u16, mode: Option<StreamMode>) {
    let packets = [
        packet(1, 0, Some(doy(day, 59, false, Some(year)))),
        packet(2, S, Some(doy(day, 60, false, Some(year)))),
        packet(3, 2 * S, Some(doy(day + 1, 0, true, Some(year)))),
    ];
    let findings = if let Some(mode) = mode {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        let mut findings = Vec::new();
        for p in packets {
            findings.extend(s.observe(p, 0, 100, false).unwrap().findings);
        }
        findings
    } else {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in packets {
            b.observe(p, false).unwrap();
        }
        let t = b.finish().unwrap();
        let refs = &t.sessions()[0].references;
        assert_eq!(refs[1].input.value, ReferenceValue::Malformed);
        assert!(!refs[1].admitted);
        assert!(
            refs.iter()
                .all(|r| r.input.year_basis.observed_leap_adjustment().get() == 0)
        );
        // Only readable endpoints remain: calendar 1s minus RTC2s = -1s;
        // drift = -1s/2s * 1e6 = -500,000 ppm.
        assert_eq!(
            t.quality(t.sessions()[0].id, 1).unwrap().drift_ppm,
            Some(-500_000.)
        );
        t.findings().to_vec()
    };
    let malformed = findings
        .iter()
        .find(|f| f.packet == 2 && f.code == 4)
        .expect("TF-004 on forbidden leap label");
    assert_eq!(
        malformed.evidence.validity(),
        findings::ValueValidity::Unusable
    );
    assert_eq!(malformed.severity, findings::Severity::Error);
    let jumps: Vec<_> = findings.iter().filter(|f| f.code == 17).collect();
    assert!(!jumps.is_empty(), "real -1s discrepancy must remain");
    assert!(
        jumps.iter().all(|f| f.delta_ns == Some(-1_000_000_000)),
        "{jumps:?}"
    );
}
scenarios!(
    i5_forbidden_common_day_in_leap_year,
    forbidden_label,
    2016,
    365
);
scenarios!(
    i6_forbidden_leap_day_in_common_year,
    forbidden_label,
    2017,
    182
);

#[test]
fn n2_observed_leap_never_crosses_year_evidence_segment() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    // Segment0: real 2016 Dec31 leap. Segment1 begins with an explanation
    // residual of183 days (>182.5): 2017 Jan1 -> DOY365 at+181 days.
    // DMY2017 Dec31 supplies a new-segment anchor, without caller context.
    let body = [1, 2, 0, 0, 0, 0, 0, 0, 0x31, 0x12, 0x17, 0x20];
    let dated = ReferenceInput::format1(
        &body,
        body.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    for p in [
        packet(1, 0, Some(wire(false, false, 59, 2016, false))),
        packet(2, S, Some(wire(false, false, 60, 2016, false))),
        packet(3, 2 * S, Some(wire(false, true, 0, 2016, false))),
        packet(4, (2 + 100 * 86400) * S, None),
        packet(
            5,
            (2 + 181 * 86400) * S,
            Some(doy(365, 0, true, Some(2016))),
        ),
        packet(6, (3 + 181 * 86400) * S, Some(dated)),
    ] {
        b.observe(p, false).unwrap();
    }
    let t = b.finish().unwrap();
    let refs = &t.sessions()[0].references;
    assert_eq!(
        refs[2].input.year_basis.observed_leap_adjustment().get(),
        1_000_000_000
    );
    assert_eq!(refs[3].input.year_basis.observed_leap_adjustment().get(), 0);
    assert!(
        t.findings()
            .iter()
            .any(|f| f.code == 17 && f.packet == 5 && f.delta_ns == Some(183 * 86_400_000_000_000))
    );
    assert!(!t.findings().iter().any(|f| f.code == 17
        && f.packet == 5
        && f.delta_ns == Some(183 * 86_400_000_000_000 - 1_000_000_000)));
}

#[test]
fn yearless_labels_keep_all_four_potential_dates() {
    for day in [181, 182, 365, 366] {
        for mode in [StreamMode::Standard, StreamMode::Interpolate] {
            let mut s = StreamTimeline::new(policy(mode), None).unwrap();
            let out = s
                .observe(packet(1, 0, Some(doy(day, 60, false, None))), 0, 100, false)
                .unwrap();
            assert!(!out.findings.iter().any(|f| f.code == 4), "day{day}");
        }
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        b.observe(packet(1, 0, Some(doy(day, 60, false, None))), false)
            .unwrap();
        let t = b.finish().unwrap();
        assert!(!t.findings().iter().any(|f| f.code == 4), "day{day}");
    }
}

#[test]
fn future_dmy_year_rejects_forbidden_label_without_credit() {
    let body = [1, 3, 0, 0, 0, 0, 0, 0, 0x31, 0x12, 0x16, 0x20]; //2016Dec31 midnight
    let dated = ReferenceInput::format1(
        &body,
        body.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for p in [
        packet(1, 0, Some(doy(365, 59, false, None))),
        packet(2, S, Some(doy(365, 60, false, None))),
        packet(3, 2 * S, Some(dated)),
    ] {
        b.observe(p, false).unwrap();
    }
    let t = b.finish().unwrap();
    assert_eq!(
        t.sessions()[0].references[1].input.value,
        ReferenceValue::Malformed
    );
    assert!(
        t.sessions()[0].references.iter().all(|r| r
            .input
            .year_basis
            .observed_leap_adjustment()
            .get()
            == 0)
    );
    assert!(
        t.findings()
            .iter()
            .any(|f| f.packet == 3 && f.code == 17 && f.delta_ns == Some(-1_000_000_000))
    );
}

fn anchor_qualified_label(_: bool, _: bool, mode: Option<StreamMode>) {
    // Fig11-14 DMY2016Dec30 23:59:59; following DOY has no caller year.
    let body = [1, 3, 0, 0, 0, 0x59, 0x59, 0x23, 0x30, 0x12, 0x16, 0x20];
    let anchor = ReferenceInput::format1(
        &body,
        body.len(),
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let packets = [
        packet(1, 0, Some(anchor)),
        packet(2, S, Some(doy(365, 60, false, None))),
        packet(3, 2 * S, Some(doy(366, 0, true, None))),
    ];
    let findings = if let Some(mode) = mode {
        let mut s = StreamTimeline::new(policy(mode), None).unwrap();
        packets
            .into_iter()
            .flat_map(|p| s.observe(p, 0, 100, false).unwrap().findings)
            .collect::<Vec<_>>()
    } else {
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        for p in packets {
            b.observe(p, false).unwrap();
        }
        let t = b.finish().unwrap();
        assert_eq!(
            t.sessions()[0].references[1].input.value,
            ReferenceValue::Malformed
        );
        assert!(
            t.sessions()[0].references.iter().all(|r| r
                .input
                .year_basis
                .observed_leap_adjustment()
                .get()
                == 0)
        );
        t.findings().to_vec()
    };
    assert!(findings.iter().any(|f| f.packet == 2 && f.code == 4));
    assert!(
        findings
            .iter()
            .any(|f| f.packet == 3 && f.code == 17 && f.delta_ns == Some(-1_000_000_000))
    );
}
scenarios!(
    anchor_qualifies_doy_label,
    anchor_qualified_label,
    false,
    false
);
