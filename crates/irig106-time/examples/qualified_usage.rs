#![deny(deprecated)]

fn main() -> irig106_time::Result<()> {
    // BEGIN wire
    use irig106_time::{
        TimeF1Layout,
        format1::decode_time_f1,
        scale::{ScalePolicy, TimeScale},
    };
    // 106-24R1 Figures11-12/13: UTC, DOY10012:30:25.340; Data Length10.
    // Layout is explicit caller evidence, not inferred from header version alone.
    let payload = [0x41, 0, 0, 0, 0x34, 0x25, 0x30, 0x12, 0, 1];
    let decoded = decode_time_f1(
        &payload,
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )?;
    let time = decoded.time.ok_or(irig106_time::TimeError::NeedsContext {
        context: "example body",
    })?;
    assert_eq!(time.scale.effective(), TimeScale::Utc);
    assert_eq!(time.year, None);
    assert_eq!(time.nanoseconds, 340_000_000);
    // END wire

    use irig106_time::{
        Rtc,
        answer::AnswerView,
        policy::{SharedPolicy, SharedSettings},
        timeline::{OfflinePolicy, PacketKind, PacketObservation, ReferenceInput, TimelineBuilder},
    };
    let shared = SharedPolicy::new(1, SharedSettings::default())?;
    // Explicit declaration-free exploration, not an invented TIMEIN declaration.
    let policy = shared.offline(OfflinePolicy {
        include_undeclared: true,
        ..Default::default()
    })?;
    let mut builder = TimelineBuilder::new(policy)?;
    builder.observe(
        PacketObservation {
            id: 0,
            channel: 7,
            rtc: Rtc::ZERO,
            kind: PacketKind::Time,
            declared_timein: false,
            declared_external: false,
            flags: 0,
            reference: Some(ReferenceInput::format1(
                &payload,
                10,
                TimeF1Layout::SourceWithIts17,
                ScalePolicy::default(),
            )?),
        },
        false,
    )?;
    let key = builder.observe(
        PacketObservation {
            id: 1,
            channel: 9,
            rtc: Rtc::from_raw(5_000_000),
            kind: PacketKind::Data,
            declared_timein: false,
            declared_external: false,
            flags: 0,
            reference: None,
        },
        false,
    )?;
    let timeline = builder.finish()?;
    let answer = timeline.lookup(key)?;
    let findings = timeline
        .findings()
        .iter()
        .map(|f| {
            irig106_time::findings::Finding::new(
                irig106_time::findings::FindingCode::from_number(f.code)?,
                Some(f.severity),
                f.evidence.clone(),
            )
        })
        .collect::<irig106_time::Result<Vec<_>>>()?;
    let view = AnswerView::offline(&answer, &findings)?;
    assert_eq!(view.value().findings.len(), findings.len());
    assert_eq!(
        view.value().calendar.map(|t| t.nanoseconds),
        Some(840_000_000)
    );
    assert_eq!(*view.value(), view.basis().replay()?.value().clone());
    assert_eq!(
        timeline.findings().iter().filter(|f| f.code == 20).count(),
        1
    );

    // Native header sessions are distinct from per-channel CLI reset heuristics.
    let mut sessions = irig106_time::session::SessionClock::new(&OfflinePolicy::default())?;
    sessions.observe(0, Rtc::ZERO, false)?;
    let update = sessions.observe(1, Rtc::from_raw(10_000_000), false)?;
    assert!(update.closed.is_none());
    assert_eq!(sessions.active().map(|s| s.high_ticks), Some(10_000_000));

    // Fig11-4 / Ch4 §4.7: no invented day/year/origin, even for zero words.
    let partial = irig106_time::ch4::Ch4Words {
        high: 0,
        low: 0,
        microseconds: 0,
    }
    .binary(None)?;
    assert!(partial.calendar().is_none());
    #[cfg(feature = "jiff")]
    {
        // Caller-supplied year for this civil conversion, not a decoded year.
        assert!(irig106_time::jiff_interop::reading_to_civil(time).is_err());
        let civil = irig106_time::jiff_interop::reading_to_civil(time.with_year(2025)?)?;
        assert_eq!(civil.value.year(), 2025);
        assert_eq!(civil.scale, time.scale);
    }
    Ok(())
}
#[test]
fn current_guide_qualified_api_examples() {
    main().unwrap();
}
