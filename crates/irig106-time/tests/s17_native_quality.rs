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
    q.add_decode_rejections(usize::MAX).unwrap();
    let before = q.snapshot();
    assert!(q.add_decode_rejections(1).is_err());
    assert_eq!(q.snapshot(), before);
    fn traits<T: Send + Sync + Clone + core::fmt::Debug>() {}
    traits::<OrderedQuality>();
}
