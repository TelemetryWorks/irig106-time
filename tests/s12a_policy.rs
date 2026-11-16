//! S12a1 clause evidence: ADR0010/0011, contract5.3/5.6, Ch10 §10.6.1/2.
//! No whole-requirement marker while common answer/replay remains S12a2.
use irig106_time::{declarations::*, policy::*, scale::*, stream::*, timeline::*, *};
fn input() -> ReferenceInput {
    ReferenceInput::format1(
        &[1, 1, 0, 0, 0, 0, 0, 0, 1, 0],
        10,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
}
fn packet(id: u64, ticks: u64, r: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(ticks),
        kind: if r.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference: r,
    }
}
fn shared(id: u64) -> SharedPolicy {
    let s = SharedSettings {
        caller_year: Some(2024),
        ..SharedSettings::default()
    };
    SharedPolicy::new(id, s).unwrap()
}
#[test]
fn central_qualification_retains_identity_year_scale_and_provided_evidence() {
    let mut s = shared(41).settings().clone();
    s.scales.irig = TimeScale::Gps;
    s.own_tolerance = Some(NanosDuration::new(17));
    let p = SharedPolicy::new(41, s).unwrap();
    let core = p.offline(OfflinePolicy::default()).unwrap();
    assert_eq!(core.own_policy().tolerance, Some(NanosDuration::new(17)));
    let mut b = TimelineBuilder::new(core).unwrap();
    b.observe(packet(1, 0, Some(input())), false).unwrap();
    let key = b.observe(packet(2, 1_000_000, None), false).unwrap();
    let a = b.finish().unwrap().lookup(key).unwrap();
    assert_eq!(a.reading.unwrap().year, Some(2024));
    assert_eq!(
        a.reading.unwrap().scale,
        ScaleEvidence::assumed(TimeScale::Gps)
    );
    let r = a.basis.reference.unwrap();
    let q = r.input.qualification.unwrap();
    assert_eq!(q.policy().id(), 41);
    assert_eq!(q.provided_scale(), ScaleEvidence::assumed(TimeScale::Utc));
    assert_eq!(q.policy().defaults_version(), 1);
    assert_eq!(a.basis.policy.shared.unwrap().id(), 41);
    assert_eq!(
        r.input.year_basis.selected().unwrap().origin,
        YearOrigin::Caller
    );
    assert!(r.input.normalization.is_some());
    let w = p.word_context(1).unwrap();
    assert!(w.bcd.year.is_none());
    assert!(w.own.ch4.is_none());
}
#[test]
fn independent_hold_gap_late_and_invalid_policy_settings() {
    let mut o = OfflinePolicy::default();
    o.gap_interval_ns = 20;
    o.gap_tolerance_ns = 3;
    let mut p = shared(1)
        .stream(StreamPolicy::new(
            1,
            BufferCaps {
                packets: 2,
                bytes: 100,
                references: 3,
            },
        ))
        .unwrap();
    p.offline.gap_interval_ns = o.gap_interval_ns;
    p.offline.gap_tolerance_ns = o.gap_tolerance_ns;
    assert_eq!(p.hold_interval_ns, 1_000_000_000);
    assert_eq!(p.hold_tolerance_ns, 100_000_000);
    assert_eq!(p.offline.late_ns, 1_100_000_000);
    assert_eq!(p.mode, StreamMode::Standard);
    p.hold_interval_ns = u64::MAX;
    assert!(p.validate().is_err());
    o.gap_interval_ns = u64::MAX;
    assert!(o.validate().is_err());
    assert!(SharedPolicy::new(0, SharedSettings::default()).is_err());
    let s = SharedSettings {
        caller_year: Some(10000),
        ..SharedSettings::default()
    };
    assert!(SharedPolicy::new(1, s).is_err());
    assert!(YearOrder::new(vec![YearSource::Caller, YearSource::Caller]).is_err());
}
#[test]
fn mode_change_drains_old_snapshot_and_keeps_eligible_reference_qualification() {
    let caps = BufferCaps {
        packets: 2,
        bytes: 100,
        references: 3,
    };
    let p1 = shared(1);
    let mut clock = p1.stream(StreamPolicy::new(1, caps)).unwrap();
    clock.mode = StreamMode::Interpolate;
    let mut s = StreamTimeline::new(clock, None).unwrap();
    s.observe(packet(1, 0, Some(input())), 0, 20, false)
        .unwrap();
    assert!(
        s.observe(packet(2, 5_000_000, None), 1, 20, false)
            .unwrap()
            .answers
            .is_empty()
    );
    let p2 = shared(2);
    let b = s
        .set_policy(p2.stream(StreamPolicy::new(2, caps)).unwrap(), 9)
        .unwrap();
    assert_eq!(b.answers.len(), 1);
    let a = &b.answers[0];
    assert_eq!(a.basis.policy.id, 1);
    assert_eq!(a.basis.policy.offline.shared.as_ref().unwrap().id(), 1);
    assert_eq!(a.basis.release, ReleaseReason::ModeChange);
    assert!(a.basis.extrapolated);
    let next = s
        .observe(packet(3, 6_000_000, None), 10, 20, false)
        .unwrap();
    let n = &next.answers[0];
    assert!(n.reading.is_some());
    assert_eq!(n.basis.policy.id, 2);
    assert_eq!(
        n.basis
            .preceding
            .as_ref()
            .unwrap()
            .input
            .qualification
            .as_ref()
            .unwrap()
            .policy()
            .id(),
        1
    );
    assert_eq!(a.basis.policy.id, 1);
}
#[test]
fn caller_first_year_order_is_explicit_and_keeps_contradicting_packet_year() {
    let r = ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let s = SharedSettings {
        caller_year: Some(2018),
        year_order: YearOrder::new(vec![
            YearSource::Caller,
            YearSource::Packet,
            YearSource::Recording,
        ])
        .unwrap(),
        ..SharedSettings::default()
    };
    let mut b = TimelineBuilder::new(
        SharedPolicy::new(1, s)
            .unwrap()
            .offline(OfflinePolicy::default())
            .unwrap(),
    )
    .unwrap();
    let k = b.observe(packet(1, 0, Some(r)), false).unwrap();
    let t = b.finish().unwrap();
    let a = t.lookup(k).unwrap();
    assert_eq!(a.reading.unwrap().year, Some(2018));
    assert!(t.findings().iter().any(|f| f.code == 32));
    let r = a.basis.reference.unwrap();
    assert!(
        r.input
            .year_basis
            .candidates()
            .iter()
            .any(|c| c.year == 2017)
    );
}
