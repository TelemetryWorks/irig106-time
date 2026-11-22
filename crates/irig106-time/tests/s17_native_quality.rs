//! Native quality/calendar infrastructure clauses only; no whole marker.
use irig106_time::{native_quality::OrderedQuality, scale::*, timeline::*, *};
fn timeline(word: u8) -> OfflineTimeline {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    // Fig11-12/14: DATE1,2017-01-02,00:00:00 /01.01 /03.03.
    // Native RTC0/1/3s, so each absolute/RTC slope is1.01 =>+10000ppm.
    for (id, ticks, fraction, seconds) in
        [(0, 0, 0, 0), (1, 10_000_000, 1, 1), (2, 30_000_000, 3, 3)]
    {
        let bytes = [word, 2, 0, 0, fraction, seconds, 0, 0, 2, 1, 0x17, 0x20];
        b.observe(
            PacketObservation {
                id,
                channel: 7,
                rtc: Rtc::from_raw(ticks),
                kind: PacketKind::Time,
                declared_timein: true,
                declared_external: true,
                flags: 0,
                reference: Some(
                    ReferenceInput::format1(
                        &bytes,
                        12,
                        TimeF1Layout::SourceWithIts17,
                        ScalePolicy::default(),
                    )
                    .unwrap(),
                ),
            },
            false,
        )
        .unwrap();
    }
    b.finish().unwrap()
}
#[test]
fn ordered_quality_matches_native_metrics_and_keeps_calendar_scale() {
    for word in [0x41, 0x51, 0x31, 0x01] {
        // UTC/GPS/RTC/assumed-UTC IRIG-B.
        let t = timeline(word);
        let refs = &t.sessions()[0].references;
        let mut q = OrderedQuality::new(1, 7).unwrap();
        assert_eq!(q.scope(), (1, 7));
        assert!(q.calendar_endpoints().unwrap().is_none());
        for r in refs {
            q.observe(r).unwrap();
        }
        let got = q.snapshot();
        assert_eq!(got, t.quality(1, 7).unwrap());
        assert_eq!((got.valid, got.admitted, got.rejected), (3, 3, 0));
        assert_eq!(got.interval_ns, Some(3_000_000_000));
        assert_eq!(got.min_spacing_ns, Some(1_000_000_000));
        assert_eq!(got.max_spacing_ns, Some(2_000_000_000));
        assert_eq!(got.references_per_second, Some(1.));
        if word == 0x31 {
            assert_eq!(got.unavailable, Some("unknown or incompatible scales"));
        } else {
            assert!((got.drift_ppm.unwrap() - 10_000.).abs() < 1e-6);
        }
        let (first, last) = q.calendar_endpoints().unwrap().unwrap();
        assert_eq!(
            (first.year, first.day_of_year, first.seconds),
            (Some(2017), 2, 0)
        );
        assert_eq!((last.seconds, last.nanoseconds), (3, 30_000_000));
        assert_eq!(first.scale, refs[0].input.scale);
        assert_eq!(last.scale, refs[2].input.scale);
        assert_eq!(q.first_reference().unwrap(), &refs[0]);
        assert_eq!(q.last_reference().unwrap(), &refs[2]);
    }
}
#[test]
fn admission_counts_duplicates_and_missing_context_are_explicit() {
    let t = timeline(0x41);
    let refs = &t.sessions()[0].references;
    let mut q = OrderedQuality::new(1, 7).unwrap();
    q.observe(&refs[0]).unwrap();
    let mut invalid = refs[1].clone();
    if let ReferenceValue::Readable(time, _) = invalid.input.value {
        invalid.input.value = ReferenceValue::Readable(time, false);
    }
    q.observe(&invalid).unwrap(); // Explicitly admitted invalid, not valid.
    let mut rejected = refs[2].clone();
    rejected.admitted = false;
    q.observe(&rejected).unwrap();
    q.add_decode_rejections(1).unwrap();
    assert_eq!(
        (
            q.snapshot().valid,
            q.snapshot().admitted,
            q.snapshot().rejected
        ),
        (1, 2, 2)
    );
    assert_eq!(
        q.snapshot().unavailable,
        Some("fewer than two valid references")
    );
    assert_eq!(q.first_reference(), q.last_reference());
    let mut dup = refs[1].clone();
    dup.key.ticks = 0;
    let mut q = OrderedQuality::new(1, 7).unwrap();
    q.observe(&refs[0]).unwrap();
    q.observe(&dup).unwrap();
    assert_eq!(q.snapshot().interval_ns, Some(0));
    assert_eq!(q.snapshot().unavailable, Some("duplicate RTC"));
    assert!(q.snapshot().min_spacing_ns.is_none());
    assert!(q.snapshot().drift_ppm.is_none());
    let mut normalized = refs[0].clone();
    normalized.input = normalized
        .input
        .normalize_to_tai(LeapSecondTable::builtin())
        .unwrap();
    let mut q = OrderedQuality::new(1, 7).unwrap();
    q.observe(&normalized).unwrap();
    q.observe(&refs[1]).unwrap();
    assert_eq!(
        q.snapshot().unavailable,
        Some("scale normalization or leap context")
    );
    assert!(q.snapshot().drift_ppm.is_none());
}
#[test]
fn scope_order_overflow_and_decode_rejection_errors_do_not_change_state() {
    let t = timeline(0x41);
    let refs = &t.sessions()[0].references;
    assert!(OrderedQuality::new(0, 7).is_err());
    let mut q = OrderedQuality::new(1, 7).unwrap();
    q.observe(&refs[1]).unwrap();
    let before = q.snapshot();
    let mut wrong_channel = refs[2].clone();
    wrong_channel.channel = 8;
    let mut wrong_session = refs[2].clone();
    wrong_session.key.session = 2;
    for r in [
        refs[0].clone(),
        refs[1].clone(),
        wrong_channel,
        wrong_session,
    ] {
        assert!(q.observe(&r).is_err());
        assert_eq!(q.snapshot(), before);
    }
    assert_eq!(q.last_reference().unwrap(), &refs[1]);
    q.observe(&refs[2]).unwrap(); // Rejected observations did not consume order.
    assert_eq!(q.last_reference().unwrap(), &refs[2]);
    q.add_decode_rejections(usize::MAX).unwrap();
    let before = q.snapshot();
    assert!(q.add_decode_rejections(1).is_err());
    assert_eq!(q.snapshot(), before);
    let mut lo = refs[0].clone();
    lo.key.ticks = i128::MIN;
    let mut hi = refs[1].clone();
    hi.key.ticks = i128::MAX;
    let mut overflow = OrderedQuality::new(1, 7).unwrap();
    overflow.observe(&lo).unwrap();
    let before = overflow.snapshot();
    assert!(overflow.observe(&hi).is_err());
    assert_eq!(overflow.snapshot(), before);
    // A failed far-future reference must not consume the ordering key.
    let mut between = refs[1].clone();
    between.key.ticks = i128::MIN + 1;
    overflow.observe(&between).unwrap();
    assert_eq!(overflow.last_reference().unwrap(), &between);
    // Subtraction fits; multiplication by 100 alone must be checked.
    lo.key.ticks = 0;
    hi.key.ticks = i128::MAX / 50;
    let mut multiply = OrderedQuality::new(1, 7).unwrap();
    multiply.observe(&lo).unwrap();
    let before = multiply.snapshot();
    assert!(multiply.observe(&hi).is_err());
    assert_eq!(multiply.snapshot(), before);

    fn traits<T: Send + Sync + Clone + core::fmt::Debug>() {}
    traits::<OrderedQuality>();
}

#[test]
fn unknown_year_rollover_and_mixed_scales_never_invent_a_drift() {
    // Fig11-13 DOY36523:59:59 ->00100:00:00, no year; RTC advances1s.
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    for (id, bytes) in [
        [0x41, 0, 0, 0, 0, 0x59, 0x59, 0x23, 0x65, 3],
        [0x41, 0, 0, 0, 0, 0, 0, 0, 1, 0],
    ]
    .into_iter()
    .enumerate()
    {
        b.observe(
            PacketObservation {
                id: id as u64,
                channel: 7,
                rtc: Rtc::from_raw(id as u64 * 10_000_000),
                kind: PacketKind::Time,
                declared_timein: true,
                declared_external: false,
                flags: 0,
                reference: Some(
                    ReferenceInput::format1(
                        &bytes,
                        10,
                        TimeF1Layout::SourceWithIts17,
                        ScalePolicy::default(),
                    )
                    .unwrap(),
                ),
            },
            false,
        )
        .unwrap();
    }
    let t = b.finish().unwrap();
    let mut q = OrderedQuality::new(1, 7).unwrap();
    for r in &t.sessions()[0].references {
        q.observe(r).unwrap();
    }
    assert_eq!(q.snapshot(), t.quality(1, 7).unwrap());
    assert_eq!(q.snapshot().unavailable, Some("calendar/year context"));
    let (a, b) = q.calendar_endpoints().unwrap().unwrap();
    assert_eq!(
        (a.year, b.year, a.day_of_year, b.day_of_year),
        (None, None, 365, 1)
    );
    let utc = timeline(0x41);
    let gps = timeline(0x51);
    let mut q = OrderedQuality::new(1, 7).unwrap();
    q.observe(&utc.sessions()[0].references[0]).unwrap();
    q.observe(&gps.sessions()[0].references[1]).unwrap();
    assert_eq!(
        q.snapshot().unavailable,
        Some("unknown or incompatible scales")
    );
    assert!(q.snapshot().drift_ppm.is_none());
    let (a, b) = q.calendar_endpoints().unwrap().unwrap();
    assert_eq!(
        (a.scale.effective(), b.scale.effective()),
        (TimeScale::Utc, TimeScale::Gps)
    );
    // Normalized mixed scales ARE comparable. UTC0 ->GPS1.01 in2017:
    // TAI37 ->TAI20.01 is -16.99s over1s RTC =>-17_990_000ppm.
    let mut a = utc.sessions()[0].references[0].clone();
    let mut b = gps.sessions()[0].references[1].clone();
    a.input = a
        .input
        .normalize_to_tai(LeapSecondTable::builtin())
        .unwrap();
    b.input = b
        .input
        .normalize_to_tai(LeapSecondTable::builtin())
        .unwrap();
    let mut q = OrderedQuality::new(1, 7).unwrap();
    q.observe(&a).unwrap();
    q.observe(&b).unwrap();
    assert_eq!(q.snapshot().unavailable, None);
    assert!((q.snapshot().drift_ppm.unwrap() + 17_990_000.).abs() < 1e-6);
    let (first, last) = q.calendar_endpoints().unwrap().unwrap();
    assert_eq!(
        (first.scale.effective(), last.scale.effective()),
        (TimeScale::Utc, TimeScale::Gps)
    );
}
