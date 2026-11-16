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
    let mut o = OfflinePolicy {
        gap_interval_ns: 20,
        gap_tolerance_ns: 3,
        ..Default::default()
    };
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
    clock.offline.channel = Some(7);
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
    let mut next_clock = p2.stream(StreamPolicy::new(2, caps)).unwrap();
    next_clock.offline.channel = Some(7);
    let b = s.set_policy(next_clock, 9).unwrap();
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
    let dated = b.observe(packet(1, 0, Some(r)), false).unwrap();
    let k = b.observe(packet(2, 1, Some(input())), false).unwrap();
    let t = b.finish().unwrap();
    assert_eq!(t.lookup(dated).unwrap().reading.unwrap().year, Some(2017));
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

// Clause checks: shared setup/era provenance, not the entire ANS/POL contract.
#[test]
fn governing_setup_and_session_era_replace_provided_context_without_erasing_it() {
    use irig106_time::{edition::PacketEdition, format2::*, leap::*, ntp::*};
    // RFC5905 Fig4: era1 offset63104 -> 2036-02-08; LE 0xf680.
    let bytes = [0x01, 0, 0, 0, 0x80, 0xf6, 0, 0, 0, 0, 0, 0];
    let o = Format2Input {
        payload: &bytes,
        data_length: 12,
        capture: ReferenceCapture {
            session: 1,
            packet: 1,
            channel: 7,
            rtc: Rtc::from_raw(0),
            byte_offset: None,
        },
        declared_edition: Some(PacketEdition::V24R1),
        data_type_version: 8,
        declared_timein: false,
    }
    .decode()
    .unwrap();
    let r =
        ReferenceInput::format2(o, NtpContext::new(1), LeapSecondTable::builtin(), false).unwrap();
    let setup = SetupDeclarations {
        range: GoverningRange::new(1, SignedRtcTicks::new(0), None).unwrap(),
        setup_packet: 90,
        provenance: "caller setup90".into(),
        declared_version: Some("106-24R1".into()),
        ri4: Some("2036-02-08".into()),
        recording_year: Some(2036),
        channels: vec![ChannelDeclaration {
            id: 7,
            cdt: Some("TIMEIN".into()),
            tsrc: Some("E".into()),
            ..Default::default()
        }],
    };
    let mut settings = SharedSettings {
        declarations: Some(Declarations::present(vec![setup.clone()]).unwrap()),
        use_tfmt_default: true,
        ..Default::default()
    };
    let mut era = NtpContext::new(1);
    era.declared = Some(EraDeclaration {
        era: 1,
        id: "caller era1".into(),
    });
    settings.ntp.insert(1, era);
    let p = SharedPolicy::new(1, settings.clone()).unwrap();
    let core = p.offline(OfflinePolicy::default()).unwrap();
    assert!(p.declaration_policy(&core).use_tfmt_default);
    let mut t = TimelineBuilder::new(core).unwrap();
    let wire_packet = PacketObservation {
        declared_timein: false,
        ..packet(1, 0, Some(r.clone()))
    };
    let k = t.observe(wire_packet.clone(), false).unwrap();
    let a = t.finish().unwrap().lookup(k).unwrap();
    let reading = a.reading.unwrap();
    assert_eq!((reading.year, reading.day_of_year), (Some(2036), 39));
    let effective = a.basis.reference.unwrap().input;
    let q = effective.qualification.as_ref().unwrap();
    assert_eq!(
        q.declarations()
            .unwrap()
            .governing
            .as_ref()
            .unwrap()
            .setup_packet,
        setup.setup_packet
    );
    assert_eq!(
        q.declarations().unwrap().governing.as_ref().unwrap().ri4,
        setup.ri4
    );
    assert_eq!(
        q.declarations().unwrap().channel.as_ref().unwrap().tfmt,
        None
    );
    let NetworkConversion::Ntp(prior) = q.provided_network().unwrap().reference().conversion()
    else {
        panic!()
    };
    assert_eq!(prior.basis().era, Some(0));
    let NetworkConversion::Ntp(actual) =
        effective.network.as_ref().unwrap().reference().conversion()
    else {
        panic!()
    };
    assert_eq!(actual.basis().era, Some(1));
    // The same centralized conversion runs under the actual stream session.
    let mut clock = p
        .stream(StreamPolicy::new(
            1,
            BufferCaps {
                packets: 2,
                bytes: 100,
                references: 3,
            },
        ))
        .unwrap();
    clock.offline.channel = Some(7);
    let mut stream = StreamTimeline::new(clock, None).unwrap();
    stream.observe(wire_packet.clone(), 0, 12, false).unwrap();
    let a = stream
        .observe(packet(2, 1, None), 1, 12, false)
        .unwrap()
        .answers
        .remove(0);
    assert_eq!(
        (a.reading.unwrap().year, a.reading.unwrap().day_of_year),
        (Some(2036), 39)
    );
    assert_eq!(
        a.basis
            .preceding
            .unwrap()
            .input
            .qualification
            .unwrap()
            .policy()
            .id(),
        1
    );
    // Explicit absence overrides setup evidence; no default admission is invented.
    settings.declarations = Some(Declarations::absent());
    let mut t = TimelineBuilder::new(
        SharedPolicy::new(2, settings)
            .unwrap()
            .offline(OfflinePolicy::default())
            .unwrap(),
    )
    .unwrap();
    let k = t.observe(wire_packet.clone(), false).unwrap();
    assert!(t.finish().unwrap().lookup(k).unwrap().reading.is_none());
}

#[test]
fn gap_and_hold_are_independently_executed_and_timeout_edge_is_inclusive() {
    for (gap, expected) in [(20, true), (200, false)] {
        let mut p = shared(1).offline(OfflinePolicy::default()).unwrap();
        p.gap_interval_ns = gap;
        let mut t = TimelineBuilder::new(p).unwrap();
        t.observe(packet(1, 0, Some(input())), false).unwrap();
        t.observe(packet(2, 1, Some(input())), false).unwrap();
        assert_eq!(
            t.finish().unwrap().findings().iter().any(|f| f.code == 16),
            expected
        );
    }
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
    p.mode = StreamMode::Interpolate;
    p.offline.channel = Some(7);
    p.hold_interval_ns = 7;
    p.hold_tolerance_ns = 3;
    let mut s = StreamTimeline::new(p, None).unwrap();
    s.observe(packet(1, 0, Some(input())), 0, 20, false)
        .unwrap();
    assert!(
        s.observe(packet(2, 1, None), 1, 20, false)
            .unwrap()
            .answers
            .is_empty()
    );
    assert!(s.advance(11).unwrap().answers.is_empty());
    let a = s.advance(12).unwrap().answers.remove(0);
    assert_eq!(a.basis.release, ReleaseReason::Timeout);
    assert!(a.basis.extrapolated);
}

#[test]
fn semantic_change_drains_then_clears_and_invalid_change_is_atomic() {
    let caps = BufferCaps {
        packets: 2,
        bytes: 100,
        references: 3,
    };
    let mut p = shared(1).stream(StreamPolicy::new(1, caps)).unwrap();
    p.mode = StreamMode::Interpolate;
    p.offline.channel = Some(7);
    let mut s = StreamTimeline::new(p.clone(), None).unwrap();
    s.observe(packet(1, 0, Some(input())), 0, 20, false)
        .unwrap();
    s.observe(packet(2, 1, None), 1, 20, false).unwrap();
    p.id = 2; // shared snapshot still1: rejected without draining.
    assert!(s.set_policy(p, 2).is_err());
    assert_eq!(s.usage().packets, 1);
    let mut settings = shared(2).settings().clone();
    settings.scales.irig = TimeScale::Gps;
    let mut next = SharedPolicy::new(2, settings)
        .unwrap()
        .stream(StreamPolicy::new(2, caps))
        .unwrap();
    next.offline.channel = Some(7);
    let old = s.set_policy(next, 3).unwrap();
    assert_eq!(old.answers[0].basis.policy.id, 1);
    assert!(old.answers[0].reading.is_some());
    let after = s.observe(packet(3, 2, None), 4, 20, false).unwrap();
    assert!(after.answers[0].reading.is_none());
}

#[test]
fn no_year_inference_is_explicit_and_dated_packets_remain_dated() {
    let mut settings = shared(1).settings().clone();
    settings.year_order = YearOrder::new(vec![]).unwrap();
    let mut t = TimelineBuilder::new(
        SharedPolicy::new(1, settings)
            .unwrap()
            .offline(OfflinePolicy::default())
            .unwrap(),
    )
    .unwrap();
    let k = t.observe(packet(1, 0, Some(input())), false).unwrap();
    let a = t.finish().unwrap().lookup(k).unwrap();
    assert_eq!(a.reading.unwrap().year, None);
    assert!(
        a.basis
            .reference
            .unwrap()
            .input
            .year_basis
            .candidates()
            .iter()
            .any(|c| c.year == 2024)
    );
}

#[test]
fn shared_provenance_validation_rejects_bad_session_and_empty_citations() {
    use irig106_time::{ntp::*, secondary_packet::TimeApplicability};
    let mut s = SharedSettings::default();
    s.ntp.insert(2, NtpContext::new(1));
    assert!(SharedPolicy::new(1, s).is_err());
    let mut s = SharedSettings::default();
    let mut c = NtpContext::new(1);
    c.declared = Some(EraDeclaration {
        era: 1,
        id: " ".into(),
    });
    s.ntp.insert(1, c);
    assert!(SharedPolicy::new(1, s).is_err());
    let mut s = SharedSettings::default();
    s.own.counter_applicability = TimeApplicability::CitedException {
        citation: " ".into(),
        data_type: 0x11,
    };
    assert!(SharedPolicy::new(1, s).is_err());
    assert!(shared(1).ntp_context(0).is_err());
}

#[test]
fn shared_scale_applies_before_decode_and_normalizes_a_real_leap_label() {
    use irig106_time::format1::Format1Input;
    // Fig11-13: DOY366 23:59:60, LY=1, SRC1/FMT0; real 2016 insertion.
    let bytes = [1, 1, 0, 0, 0, 0x60, 0x59, 0x23, 0x66, 3];
    let mut settings = shared(1).settings().clone();
    settings.caller_year = Some(2016);
    let p = SharedPolicy::new(1, settings).unwrap();
    let make_input = || Format1Input {
        payload: &bytes,
        data_length: 10,
        layout: TimeF1Layout::SourceWithIts17,
        // Caller prepared a wrong assumption; shared UTC must govern DECODE too.
        scales: ScalePolicy {
            irig: TimeScale::Gps,
            ..Default::default()
        },
    };
    let mut t = TimelineBuilder::new(p.offline(OfflinePolicy::default()).unwrap()).unwrap();
    let k = t
        .observe_format1(
            PacketObservation {
                kind: PacketKind::Time,
                ..packet(1, 0, None)
            },
            make_input(),
            false,
        )
        .unwrap();
    let timeline = t.finish().unwrap();
    assert!(timeline.lookup(k).unwrap().reading.is_none()); // legacy second60 stays partial
    let r = &timeline.sessions()[0].references[0];
    assert_eq!(r.input.year_basis.selected().unwrap().year, 2016);
    assert!(r.input.normalization.is_some());
    assert_eq!(r.input.wire.unwrap().1.time.unwrap().seconds, 60);
    assert_eq!(r.input.scale, ScaleEvidence::assumed(TimeScale::Utc));
    let mut clock = p
        .stream(StreamPolicy::new(
            1,
            BufferCaps {
                packets: 2,
                bytes: 100,
                references: 3,
            },
        ))
        .unwrap();
    clock.offline.channel = Some(7);
    let mut stream = StreamTimeline::new(clock, None).unwrap();
    let batch = stream
        .observe_format1(
            PacketObservation {
                kind: PacketKind::Time,
                ..packet(1, 0, None)
            },
            make_input(),
            0,
            10,
            false,
        )
        .unwrap();
    let a = stream
        .observe(packet(2, 0, None), 1, 10, false)
        .unwrap()
        .answers
        .remove(0);
    assert!(a.reading.is_none());
    assert!(batch.findings.iter().any(|f| f.code == 22));
    assert!(!batch.findings.iter().any(|f| f.code == 4));
}
