//! Partial ANS101/105 and POL101/102 replay clauses, ADR0010/0011.
//! Native replay is retained under ADR-0019; JSON output belongs to the CLI.
use irig106_time::{answer::*, policy::*, scale::*, stream::*, timeline::*, *};
fn packet(id: u64, ticks: u64, reference: Option<ReferenceInput>) -> PacketObservation {
    PacketObservation {
        id,
        channel: 7,
        rtc: Rtc::from_raw(ticks),
        kind: if reference.is_some() {
            PacketKind::Time
        } else {
            PacketKind::Data
        },
        declared_timein: true,
        declared_external: true,
        flags: 0,
        reference,
    }
}
fn reference() -> ReferenceInput {
    // Fig11-14 (11-25/PDF33): DMY2017-01-01, DATE1, SRC1/FMT4.
    ReferenceInput::format1(
        &[0x41, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap()
}
fn offline() -> OfflineAnswer {
    let p = SharedPolicy::new(7, SharedSettings::default()).unwrap();
    let mut b = TimelineBuilder::new(p.offline(OfflinePolicy::default()).unwrap()).unwrap();
    b.observe(packet(1, 0, Some(reference())), false).unwrap();
    let key = b.observe(packet(2, 10, None), false).unwrap();
    b.finish().unwrap().lookup(key).unwrap()
}
#[test]
fn offline_replay_owns_basis_and_typed_duration_without_timeline() {
    let a = offline();
    let v = AnswerView::offline(&a, &[]).unwrap();
    assert_eq!(a.counter_duration(), SignedNanos::new(1000));
    assert_eq!(v.value().counter_duration, Some(SignedNanos::new(1000)));
    assert_eq!(v.value().calendar.unwrap().nanoseconds, 1000);
    let AnswerBasis::Counter(c) = v.basis() else {
        panic!()
    };
    let ClockBasis::Offline(b) = c.clock() else {
        panic!()
    };
    assert_eq!(b.session_origin.ticks, 0);
    assert_eq!(b.policy.shared.as_ref().unwrap().id(), 7);
    drop(a);
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
#[test]
fn public_native_result_tampering_is_rejected_not_copied_into_replay() {
    let mut a = offline();
    a.counter_ns += 1;
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.reading.as_mut().unwrap().nanoseconds = 999;
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.basis.reference.as_mut().unwrap().key.session = 2;
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.basis.reference.as_mut().unwrap().admitted = false;
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.basis.channel = Some((8, ChannelChoice::Caller));
    assert!(AnswerView::offline(&a, &[]).is_err());
    let mut a = offline();
    a.basis.session_origin.ticks = i128::MIN;
    assert!(AnswerView::offline(&a, &[]).is_err());
}
#[test]
fn negative_counter_only_duration_is_checked_signed_and_replayable() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let k = b.observe(packet(1, 5, None), false).unwrap();
    let a = b
        .finish()
        .unwrap()
        .lookup(PacketKey {
            ticks: 4,
            packet: 2,
            ..k
        })
        .unwrap();
    let v = AnswerView::offline(&a, &[]).unwrap();
    assert_eq!(v.value().counter_duration, Some(SignedNanos::new(-100)));
    assert!(v.value().calendar.is_none());
    assert_eq!(
        v.value().issue,
        Some(TimeError::NeedsContext {
            context: "no admitted reference on selected channel"
        })
    );
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
fn network(id: u64, ticks: u64, seconds: u32, nanos: u32) -> ReferenceInput {
    use irig106_time::{edition::PacketEdition, format2::*, leap::*, ntp::*};
    let mut bytes = [0; 12];
    bytes[0] = 0x11;
    bytes[4..8].copy_from_slice(&seconds.to_le_bytes());
    bytes[8..].copy_from_slice(&nanos.to_le_bytes());
    let o = Format2Input {
        payload: &bytes,
        data_length: 12,
        capture: ReferenceCapture {
            session: 1,
            packet: id,
            channel: 7,
            rtc: Rtc::from_raw(ticks),
            byte_offset: Some(id * 100),
        },
        declared_edition: Some(PacketEdition::V24R1),
        data_type_version: 8,
        declared_timein: true,
    }
    .decode()
    .unwrap();
    ReferenceInput::format2(o, NtpContext::new(1), LeapSecondTable::builtin(), false).unwrap()
}
#[test]
fn interpolated_replay_uses_both_normalized_endpoints_and_floor_after_engine_drop() {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 100,
            references: 3,
        },
    );
    p.mode = StreamMode::Interpolate;
    p.offline.channel = Some(7);
    p.offline.jump_threshold_ns = Some(20_000); // Network resolution1ns would flag10us as a jump.
    let mut s = StreamTimeline::new(p, None).unwrap();
    // Ch11 Fig11-17: native TAI. Absolute span1.000010s / RTC span1s;
    // midpoint displacement0.500005s, correction5000ns. S04 offset37s.
    s.observe(
        packet(1, 0, Some(network(1, 0, 1_483_228_837, 0))),
        0,
        12,
        false,
    )
    .unwrap();
    s.observe(packet(2, 5_000_000, None), 1, 12, false).unwrap();
    let a = s
        .observe(
            packet(
                3,
                10_000_000,
                Some(network(3, 10_000_000, 1_483_228_838, 10000)),
            ),
            2,
            12,
            false,
        )
        .unwrap()
        .answers
        .remove(0);
    assert_eq!(a.counter_duration(), SignedNanos::new(500_000_000));
    let v = AnswerView::stream(&a, &[]).unwrap();
    assert_eq!(v.value().calendar.unwrap().nanoseconds, 500_005_000);
    assert_eq!(
        v.value().calendar.unwrap().scale.effective(),
        TimeScale::Tai
    );
    let AnswerBasis::Counter(c) = v.basis() else {
        panic!()
    };
    let ClockBasis::Stream(b) = c.clock() else {
        panic!()
    };
    assert_eq!(b.drift_correction_ns, Some(5000));
    assert_eq!(b.release, ReleaseReason::Interpolated);
    assert!(b.following.as_ref().unwrap().input.network.is_some());
    drop((a, s));
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}

use irig106_time::{extracted::*, own_time::*, secondary_packet::*};
fn context() -> OwnContext {
    OwnContext {
        ieee: Some(EpochContext {
            origin: format1::CalendarReading {
                year: Some(2017),
                day_of_year: 1,
                hours: 0,
                minutes: 0,
                seconds: 0,
                nanoseconds: 0,
                scale: ScaleEvidence::standard(TimeScale::Utc),
            },
            counting: EpochCounting::Continuous,
            source: "explicit caller epoch".into(),
        }),
        ..Default::default()
    }
}
fn origin() -> ExtractionOrigin {
    ExtractionOrigin {
        key: PacketKey {
            session: 1,
            ticks: 10,
            packet: 2,
        },
        channel: 7,
        byte_offset: Some(123),
        field: "caller measurement".into(),
    }
}
#[test]
fn own_envelope_retains_raw_location_independent_counter_and_comparison() {
    let a = offline();
    let mut ctx = context();
    let mut bytes = [0; 12]; // Fig11-5: nanoseconds first, seconds second.
    bytes[..4].copy_from_slice(&1000u32.to_le_bytes());
    let checksum: u16 = bytes[..10].iter().map(|n| u16::from(*n)).sum();
    bytes[10..].copy_from_slice(&checksum.to_le_bytes());
    let mut p = OwnPolicy {
        tolerance: Some(NanosDuration::new(17)),
        ..Default::default()
    };
    p.levels.insert(14, Severity::Information);
    for (valid, nanos) in [(true, 1000u32), (false, 1000), (true, 20000)] {
        let mut raw = bytes;
        raw[..4].copy_from_slice(&nanos.to_le_bytes());
        let checksum: u16 = raw[..10].iter().map(|n| u16::from(*n)).sum();
        raw[10..].copy_from_slice(&checksum.to_le_bytes());
        if !valid {
            raw[10] ^= 1;
        }
        let own = answer_own(
            CounterAnswer::Offline(&a),
            SecondaryPacket {
                key: a.basis.key,
                channel: 7,
                data_type: 0x19,
                rtc: Rtc::from_raw(10),
                flags: 0x84,
                bytes: &raw,
                byte_offset: Some(24),
                applicability: TimeApplicability::FirstDataBit,
            },
            &ctx,
            &p,
        )
        .unwrap();
        let v = AnswerView::own(&own).unwrap();
        assert_eq!(v.value().point.as_ref(), own.own());
        assert_eq!(v.value().own_minus_counter, own.delta_ns());
        assert_eq!(v.value().tolerance, Some(NanosDuration::new(17)));
        assert_eq!(v.value().counter_duration, Some(SignedNanos::new(1000)));
        assert_eq!(
            v.value().counter.as_ref().unwrap().as_counter().key(),
            a.basis.key
        );
        assert_eq!(v.value().findings, own.findings());
        assert_eq!(v.value().point.is_some(), valid);
        if valid {
            assert_eq!(
                v.value().own_minus_counter,
                Some(SignedNanos::new(i128::from(nanos) - 1000))
            );
            if nanos > 1000 {
                assert!(
                    v.value()
                        .findings
                        .iter()
                        .any(|f| f.code().number() == 14 && f.severity() == Severity::Information)
                );
            }
        } else {
            assert!(v.value().findings.iter().any(|f| f.code().number() == 12));
        }
        let AnswerBasis::Own(b) = v.basis() else {
            panic!()
        };
        assert_eq!(b.header().data_type(), 0x19);
        assert_eq!(b.header().byte_offset(), Some(24));
        assert_eq!(b.header().raw_bytes(), raw);
        assert_eq!(b.policy(), &p);
        assert_eq!(b.context(), &ctx);
        drop(own);
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
    // A frozen envelope is independent of later caller changes.
    let own = answer_own(
        CounterAnswer::Offline(&a),
        SecondaryPacket {
            key: a.basis.key,
            channel: 7,
            data_type: 0x19,
            rtc: Rtc::from_raw(10),
            flags: 0x84,
            bytes: &bytes,
            byte_offset: Some(24),
            applicability: TimeApplicability::FirstDataBit,
        },
        &ctx,
        &p,
    )
    .unwrap();
    let v = AnswerView::own(&own).unwrap();
    drop(own);
    drop(a);
    ctx.ieee = None;
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
#[test]
fn stamp_envelope_replays_released_query_and_subtick_duration_without_engine() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 0, Some(reference())), false).unwrap();
    let timeline = b.finish().unwrap();
    for (flags, bytes, want) in [
        (0, 20u64.to_le_bytes(), 2000),
        (0xc8, 2050u64.to_le_bytes(), 2050),
    ] {
        // §11.2.1.3b: padded RTC20ticks=2000ns; ERTC2050ns=20ticks+50ns.
        let stamp = inspect_stamp(
            StampInput {
                origin: origin(),
                query_id: 3,
                flags,
                bytes: &bytes,
                order: rtc::CounterOrder::Nearest,
            },
            &SecondaryPolicy::default(),
        )
        .unwrap()
        .resolve(&context(), |k| {
            Ok(ResolvedCounter::offline(timeline.lookup(k)?))
        })
        .unwrap();
        let v = AnswerView::stamp(&stamp).unwrap();
        assert_eq!(v.value().calendar, stamp.calendar());
        assert_eq!(v.value().counter_duration, Some(SignedNanos::new(want)));
        assert_eq!(v.value().findings, stamp.inspection().findings());
        let AnswerBasis::Stamp(b) = v.basis() else {
            panic!()
        };
        assert_eq!(b.inspection().origin(), &origin());
        assert_eq!(b.inspection().raw(), bytes);
        assert!(b.counter().is_some());
        drop(stamp);
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
    drop(timeline);
}
#[test]
fn absolute_stamp_and_partial_word_do_not_invent_counters_or_context() {
    for flags in [0xc0, 0xc4, 0xcc] {
        // Chapter4, IEEE, undefined.
        let bytes = [0; 8];
        let stamp = inspect_stamp(
            StampInput {
                origin: origin(),
                query_id: 3,
                flags,
                bytes: &bytes,
                order: rtc::CounterOrder::Nearest,
            },
            &SecondaryPolicy::default(),
        )
        .unwrap()
        .resolve(&OwnContext::default(), |_| {
            panic!("absolute/invalid stamps never query")
        })
        .unwrap();
        let v = AnswerView::stamp(&stamp).unwrap();
        assert!(v.value().calendar.is_none());
        assert!(v.value().counter_duration.is_none());
        assert_eq!(v.value().issue.as_ref(), stamp.issue());
        assert_eq!(v.value().findings, stamp.inspection().findings());
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
    // IEEE elapsed1s from2016-12-31 23:59:59 is the explicit UTC second60.
    // Replaying a resolved inspection must not append TF-022 a second time.
    let mut ctx = context();
    let e = ctx.ieee.as_mut().unwrap();
    e.origin.year = Some(2016);
    e.origin.day_of_year = 366;
    e.origin.hours = 23;
    e.origin.minutes = 59;
    e.origin.seconds = 59;
    let bytes = [0, 0, 0, 0, 1, 0, 0, 0];
    let a = inspect_stamp(
        StampInput {
            origin: origin(),
            query_id: 3,
            flags: 0xc4,
            bytes: &bytes,
            order: rtc::CounterOrder::Nearest,
        },
        &SecondaryPolicy::default(),
    )
    .unwrap()
    .resolve(&ctx, |_| panic!("absolute"))
    .unwrap();
    let v = AnswerView::stamp(&a).unwrap();
    assert_eq!(v.value().calendar.unwrap().seconds, 60);
    assert_eq!(
        v.value()
            .findings
            .iter()
            .filter(|f| f.code().number() == 22)
            .count(),
        1
    );
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
    let w = answer_word(
        origin(),
        TimeWord::Binary(ch4::Ch4Words {
            high: 0,
            low: 0x8000,
            microseconds: 0,
        }),
        &WordContext::new(1),
        &SecondaryPolicy::default(),
    )
    .unwrap();
    let v = AnswerView::word(&w).unwrap();
    assert!(v.value().calendar.is_none());
    assert!(v.value().counter_duration.is_none());
    assert_eq!(v.value().chapter4.as_ref(), w.chapter4());
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}
#[test]
fn word_envelope_keeps_network_era_zero_ambiguity_stale_table_and_overrides() {
    use irig106_time::{network_time::*, ntp::*};
    let mut c = WordContext::new(1);
    c.ntp.declared = Some(EraDeclaration {
        era: 1,
        id: "caller era1".into(),
    });
    let mut p = SecondaryPolicy::default();
    p.levels.insert(29, Severity::Information);
    p.levels.insert(30, Severity::Information);
    p.levels.insert(21, Severity::Information);
    let cases = [
        NetworkWord::Ntp(NtpTime {
            seconds: 63_104,
            fraction: 0,
        }),
        NetworkWord::Ntp(NtpTime {
            seconds: 0,
            fraction: 0,
        }),
        NetworkWord::Ptp(PtpTime {
            seconds: 2_145_916_837,
            nanoseconds: 123,
        }),
        NetworkWord::Ptp(PtpTime {
            seconds: 1_483_228_836,
            nanoseconds: 0,
        }),
        NetworkWord::NtpFraction(0x80000000),
    ];
    for word in cases {
        let a = answer_word(origin(), TimeWord::Network(word), &c, &p).unwrap();
        let v = AnswerView::word(&a).unwrap();
        assert_eq!(v.value().network.as_ref(), a.network());
        assert_eq!(v.value().calendar, a.calendar());
        assert_eq!(v.value().findings, a.findings());
        match word {
            NetworkWord::Ntp(NtpTime {
                seconds: 0,
                fraction: 0,
            }) => {
                assert!(v.value().calendar.is_none());
                assert!(
                    v.value()
                        .findings
                        .iter()
                        .any(|f| f.code().number() == 29 && f.severity() == Severity::Information)
                );
            }
            NetworkWord::Ptp(t) => {
                assert_eq!(
                    v.value().calendar.unwrap().scale.effective(),
                    TimeScale::Tai
                );
                let code = if t.seconds == 1_483_228_836 { 22 } else { 21 };
                assert!(v.value().findings.iter().any(|f|f.code().number()==code && f.severity()==Severity::Information));
            }
            NetworkWord::Ntp(NtpTime {
                seconds: 63_104, ..
            }) => {
                let cal = v.value().calendar.unwrap();
                assert_eq!((cal.year, cal.day_of_year), (Some(2036), 39));
            }
            _ => {}
        }
        let AnswerBasis::Word(b) = v.basis() else {
            panic!()
        };
        assert_eq!(b.context(), &c);
        assert_eq!(b.policy(), &p);
        assert_eq!(b.word(), TimeWord::Network(word));
        drop(a);
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
    for pivot in [false, true] {
        let mut ctx = WordContext::new(1);
        if pivot {
            ctx.ntp.pivots.push(
                NtpPivot::approximate(1, "caller2036", SignedUnixSeconds::new(2_086_041_600), 0)
                    .unwrap(),
            );
        }
        let a = answer_word(
            origin(),
            TimeWord::Network(NetworkWord::Ntp(NtpTime {
                seconds: 63_104,
                fraction: 0,
            })),
            &ctx,
            &p,
        )
        .unwrap();
        let v = AnswerView::word(&a).unwrap();
        let format2::NetworkConversion::Ntp(n) = v.value().network.as_ref().unwrap() else {
            panic!()
        };
        assert_eq!(
            n.basis().era_basis,
            Some(if pivot {
                EraBasis::PivotDerived
            } else {
                EraBasis::AssumedZero
            })
        );
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
    c.matching = Some(ComponentBinding {
        target: origin(),
        other: ExtractionOrigin {
            field: "paired seconds".into(),
            byte_offset: Some(127),
            ..origin()
        },
        word: NetworkWord::NtpSeconds(63_104),
    });
    let paired = answer_word(
        origin(),
        TimeWord::Network(NetworkWord::NtpFraction(0x80000000)),
        &c,
        &p,
    )
    .unwrap();
    let v = AnswerView::word(&paired).unwrap();
    assert_eq!(v.value().calendar.unwrap().nanoseconds, 500_000_000);
    let AnswerBasis::Word(b) = v.basis() else {
        panic!()
    };
    assert_eq!(b.context(), &c);
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
    c.matching = None;
    c.ntp.declared.as_mut().unwrap().era = 0;
    let a = answer_word(
        origin(),
        TimeWord::Network(NetworkWord::Ntp(NtpTime {
            seconds: 3_692_217_600,
            fraction: 0,
        })),
        &c,
        &p,
    )
    .unwrap(); // 2017 insertion ambiguity window
    let v = AnswerView::word(&a).unwrap();
    assert!(
        v.value()
            .findings
            .iter()
            .any(|f| f.code().number() == 30 && f.severity() == Severity::Information)
    );
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}

#[test]
fn stream_fallback_replay_retains_old_policy_release_and_session_origin() {
    for reason in [
        ReleaseReason::Eof,
        ReleaseReason::Timeout,
        ReleaseReason::PacketCap,
        ReleaseReason::ByteCap,
        ReleaseReason::ModeChange,
        ReleaseReason::SessionBoundary,
    ] {
        let mut p = SharedPolicy::new(1, SharedSettings::default())
            .unwrap()
            .stream(StreamPolicy::new(
                1,
                BufferCaps {
                    packets: if reason == ReleaseReason::ByteCap {
                        2
                    } else {
                        1
                    },
                    bytes: 10,
                    references: 3,
                },
            ))
            .unwrap();
        p.mode = StreamMode::Interpolate;
        p.offline.channel = Some(7);
        let mut s = StreamTimeline::new(p, None).unwrap();
        s.observe(packet(1, 100, Some(reference())), 0, 10, false)
            .unwrap();
        assert!(
            s.observe(packet(2, 150, None), 1, 10, false)
                .unwrap()
                .answers
                .is_empty()
        );
        let batch = match reason {
            ReleaseReason::Eof => s.finish(2).unwrap(),
            ReleaseReason::Timeout => s.advance(1_100_000_002).unwrap(),
            ReleaseReason::PacketCap => s.observe(packet(3, 160, None), 2, 10, false).unwrap(),
            ReleaseReason::ByteCap => s.observe(packet(3, 160, None), 2, 11, false).unwrap(),
            ReleaseReason::ModeChange => {
                let mut next = SharedPolicy::new(2, SharedSettings::default())
                    .unwrap()
                    .stream(StreamPolicy::new(
                        2,
                        BufferCaps {
                            packets: 1,
                            bytes: 10,
                            references: 3,
                        },
                    ))
                    .unwrap();
                next.offline.channel = Some(7);
                s.set_policy(next, 2).unwrap()
            }
            ReleaseReason::SessionBoundary => s
                .observe(packet(3, 0, Some(reference())), 2, 10, true)
                .unwrap(),
            _ => unreachable!(),
        };
        let a = &batch.answers[0];
        assert_eq!(a.basis.release, reason);
        assert!(a.basis.extrapolated);
        assert_eq!(
            (a.basis.session_origin.session, a.basis.session_origin.ticks),
            (1, 100)
        );
        let v = AnswerView::stream(a, &[]).unwrap();
        assert_eq!(v.value().counter_duration, Some(SignedNanos::new(5000)));
        let AnswerBasis::Counter(b) = v.basis() else {
            panic!()
        };
        let ClockBasis::Stream(b) = b.clock() else {
            panic!()
        };
        assert_eq!(b.release, reason);
        assert_eq!(b.policy.id, 1);
        assert_eq!(b.policy.offline.shared.as_ref().unwrap().id(), 1);
        drop((batch, s));
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
}
#[test]
fn caller_selected_counter_findings_remain_separate_from_own_findings() {
    use irig106_time::findings::*;
    let a = offline();
    let f = Finding::new(
        FindingCode::from_number(24).unwrap(),
        Some(Severity::Information),
        FindingEvidence::new(
            FindingLocation {
                session: Some(1),
                packet: Some(2),
                channel: Some(7),
                byte_offset: Some(10),
            },
            ValueValidity::Kept,
        ),
    )
    .unwrap();
    let c = AnswerView::offline(&a, std::slice::from_ref(&f)).unwrap();
    assert_eq!(c.value().findings, vec![f.clone()]);
    assert_eq!(c.basis().replay().unwrap().value(), c.value());
    let own = answer_own(
        CounterAnswer::Offline(&a),
        SecondaryPacket {
            key: a.basis.key,
            channel: 7,
            data_type: 0x19,
            rtc: Rtc::from_raw(10),
            flags: 0,
            bytes: &[],
            byte_offset: Some(24),
            applicability: TimeApplicability::FirstDataBit,
        },
        &context(),
        &OwnPolicy::default(),
    )
    .unwrap();
    let view = AnswerView::own(&own)
        .unwrap()
        .with_counter_findings(std::slice::from_ref(&f))
        .unwrap();
    let AnswerBasis::Own(b) = view.basis() else {
        panic!()
    };
    assert_eq!(b.counter().supplied_findings(), [f]);
    assert_eq!(view.value().findings, own.findings());
    assert_eq!(view.basis().replay().unwrap().value(), view.value());
}

#[test]
fn typed_counter_projection_covers_missing_invalid_undeclared_pre_first_and_wrap() {
    for case in 0..5 {
        let first = if case == 4 { (1u64 << 48) - 2 } else { 10 };
        let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
        let mut r = reference();
        if case == 1 {
            r.value = ReferenceValue::Invalid;
        }
        let mut p = packet(
            1,
            first,
            if case == 0 || case == 3 {
                None
            } else {
                Some(r)
            },
        );
        if case == 2 {
            p.declared_timein = false;
        }
        b.observe(p, false).unwrap();
        let query = if case == 4 { 1 } else { 11 };
        let key = b.observe(packet(2, query, None), false).unwrap();
        if case == 3 {
            b.observe(packet(3, 12, Some(reference())), false).unwrap();
        }
        let a = b.finish().unwrap().lookup(key).unwrap();
        let v = AnswerView::offline(&a, &[]).unwrap();
        assert_eq!(
            v.value().counter_duration,
            Some(SignedNanos::new(if case == 4 { 300 } else { 100 }))
        );
        assert_eq!(v.value().calendar.is_none(), case != 4);
        assert_eq!(v.basis().replay().unwrap().value(), v.value());
    }
}

#[test]
fn negative_interpolation_replay_floors_instead_of_truncating() {
    let mut p = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 100,
            references: 3,
        },
    );
    p.mode = StreamMode::Interpolate;
    p.offline.channel = Some(7);
    // Deliberately permissive reader threshold, so a -1ns absolute span is
    // eligible across1s RTC. At midpoint, floor(-1/2)=-1ns, not0ns.
    p.offline.jump_threshold_ns = Some(2_000_000_000);
    let mut s = StreamTimeline::new(p, None).unwrap();
    s.observe(
        packet(1, 0, Some(network(1, 0, 1_483_228_837, 0))),
        0,
        12,
        false,
    )
    .unwrap();
    s.observe(packet(2, 5_000_000, None), 1, 12, false).unwrap();
    let a = s
        .observe(
            packet(
                3,
                10_000_000,
                Some(network(3, 10_000_000, 1_483_228_836, 999_999_999)),
            ),
            2,
            12,
            false,
        )
        .unwrap()
        .answers
        .remove(0);
    assert_eq!(a.basis.release, ReleaseReason::Interpolated);
    assert_eq!(a.basis.drift_correction_ns, Some(-500_000_001));
    let v = AnswerView::stream(&a, &[]).unwrap();
    assert_eq!(v.value().calendar.unwrap().nanoseconds, 999_999_999);
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}

#[test]
fn replay_keeps_stated_gps_native_calendar() {
    // Fig11-12 FMT5 GPS; Fig11-14 DATE1 calendar. Never report this as UTC.
    let r = ReferenceInput::format1(
        &[0x51, 2, 0, 0, 0, 0, 0, 0, 1, 1, 0x17, 0x20],
        12,
        TimeF1Layout::SourceWithIts17,
        ScalePolicy::default(),
    )
    .unwrap();
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 0, Some(r)), false).unwrap();
    let k = b.observe(packet(2, 1, None), false).unwrap();
    let a = b.finish().unwrap().lookup(k).unwrap();
    let v = AnswerView::offline(&a, &[]).unwrap();
    assert_eq!(
        v.value().calendar.unwrap().scale,
        ScaleEvidence::standard(TimeScale::Gps)
    );
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}

#[test]
fn review_stamp_replay_keeps_explicit_backward_order() {
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    b.observe(packet(1, 0, Some(reference())), false).unwrap();
    let timeline = b.finish().unwrap();
    let bytes = 20u64.to_le_bytes();
    let inspect = |order| {
        inspect_stamp(
            StampInput {
                origin: origin(),
                query_id: 3,
                flags: 0,
                bytes: &bytes,
                order,
            },
            &SecondaryPolicy::default(),
        )
        .unwrap()
    };
    // OriginRTC10 -> raw stamp20. Nearest is+10ticks; caller-guaranteed
    // backward within one wrap is10−2^48ticks, query20−2^48.
    assert_eq!(
        inspect(rtc::CounterOrder::Nearest).query().unwrap().ticks,
        20
    );
    let order = rtc::CounterOrder::BackwardWithinOneWrap;
    let expected = 20 - (1i128 << 48);
    let inspection = inspect(order);
    assert_eq!(inspection.query().unwrap().ticks, expected);
    let stamp = inspection
        .resolve(&context(), |k| {
            Ok(ResolvedCounter::offline(timeline.lookup(k)?))
        })
        .unwrap();
    let v = AnswerView::stamp(&stamp).unwrap();
    assert_eq!(
        v.value().counter.as_ref().unwrap().as_counter().key().ticks,
        expected
    );
    assert_eq!(
        v.value().counter_duration,
        Some(SignedNanos::new(expected * 100))
    );
    let AnswerBasis::Stamp(b) = v.basis() else {
        panic!()
    };
    assert_eq!(b.inspection().order(), order);
    drop((stamp, timeline));
    assert_eq!(v.basis().replay().unwrap().value(), v.value());
}

fn review_interpolated_control() -> StreamAnswer {
    let mut policy = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 100,
            references: 3,
        },
    );
    policy.mode = StreamMode::Interpolate;
    policy.offline.channel = Some(7);
    let mut stream = StreamTimeline::new(policy, None).unwrap();
    stream
        .observe(
            packet(1, 0, Some(network(1, 0, 1_483_228_837, 0))),
            0,
            12,
            false,
        )
        .unwrap();
    stream
        .observe(packet(2, 5_000_000, None), 1, 12, false)
        .unwrap();
    stream
        .observe(
            packet(
                3,
                10_000_000,
                Some(network(3, 10_000_000, 1_483_228_838, 0)),
            ),
            2,
            12,
            false,
        )
        .unwrap()
        .answers
        .remove(0)
}
fn review_endpoint(id: u64, ticks: u64, nanos: u32) -> TimelineReference {
    let mut builder = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    builder
        .observe(
            packet(id, ticks, Some(network(id, ticks, 1_483_228_837, nanos))),
            false,
        )
        .unwrap();
    builder.finish().unwrap().sessions()[0].references[0].clone()
}
fn review_changed<T: Clone>(original: &T, change: impl FnOnce(&mut T)) -> T {
    let mut copy = original.clone();
    change(&mut copy);
    copy
}
#[test]
fn review_capture_consistency_rejects_single_field_tampering() {
    let off = offline();
    let interp = review_interpolated_control();
    assert_eq!(interp.basis.release, ReleaseReason::Interpolated);
    assert_eq!(interp.basis.drift_correction_ns, Some(0));
    let mut b = TimelineBuilder::new(OfflinePolicy::default()).unwrap();
    let key = b.observe(packet(1, 0, None), false).unwrap();
    let empty_off = b.finish().unwrap().lookup(key).unwrap();
    let mut policy = StreamPolicy::new(
        1,
        BufferCaps {
            packets: 2,
            bytes: 100,
            references: 3,
        },
    );
    policy.offline.channel = Some(7);
    let empty_stream = StreamTimeline::new(policy, None)
        .unwrap()
        .observe(packet(1, 0, None), 0, 10, false)
        .unwrap()
        .answers
        .remove(0);
    // Controls are native answers; every one is accepted before tampering.
    for a in [&off, &empty_off] {
        assert!(AnswerView::offline(a, &[]).is_ok());
    }
    for a in [&interp, &empty_stream] {
        assert!(AnswerView::stream(a, &[]).is_ok());
    }
    // Ch11 Fig11-17 hand arithmetic: this control is exactly100ns/tick.
    // Replacing ONE whole endpoint with another native, internally coherent
    // endpoint preserves midpoint time and zero correction. Only its position
    // relative to the query becomes invalid, avoiding a masked guard test.
    let after_query = review_endpoint(1, 5_000_001, 500_000_100);
    let before_query = review_endpoint(3, 4_999_999, 499_999_900);
    let cases = [
        (
            "A2 offline distance",
            ResolvedCounter::offline(review_changed(&off, |a| a.basis.distance_ns = Some(1001))),
        ),
        (
            "A9 deferred offline drift",
            ResolvedCounter::offline(review_changed(&off, |a| a.basis.drift_corrected = true)),
        ),
        (
            "A15 counter-only position",
            ResolvedCounter::offline(review_changed(&empty_off, |a| {
                a.basis.position = ReferencePosition::Exact
            })),
        ),
        (
            "A15 counter-only distance",
            ResolvedCounter::offline(review_changed(&empty_off, |a| {
                a.basis.distance_ns = Some(0)
            })),
        ),
        (
            "A18 offline reading with reason",
            ResolvedCounter::offline(review_changed(&off, |a| {
                a.basis.unresolved = Some("corrupt reason")
            })),
        ),
        (
            "A18 offline missing reason",
            ResolvedCounter::offline(review_changed(&empty_off, |a| a.basis.unresolved = None)),
        ),
        (
            "A19 invalid offline policy",
            ResolvedCounter::offline(review_changed(&off, |a| a.basis.policy.version = 0)),
        ),
        (
            "A26 invalid stream policy",
            ResolvedCounter::stream(review_changed(&interp, |a| a.basis.policy.caps.packets = 0)),
        ),
        (
            "A8 interpolated without endpoints",
            ResolvedCounter::stream(review_changed(&empty_stream, |a| {
                a.basis.release = ReleaseReason::Interpolated
            })),
        ),
        (
            "A8 following with standard release",
            ResolvedCounter::stream(review_changed(&interp, |a| {
                a.basis.release = ReleaseReason::Standard
            })),
        ),
        (
            "A16 preceding follows query",
            ResolvedCounter::stream(review_changed(&interp, |a| {
                a.basis.preceding = Some(after_query)
            })),
        ),
        (
            "A17 following precedes query",
            ResolvedCounter::stream(review_changed(&interp, |a| {
                a.basis.following = Some(before_query)
            })),
        ),
        (
            "A25 interpolation in Standard mode",
            ResolvedCounter::stream(review_changed(&interp, |a| {
                a.basis.policy.mode = StreamMode::Standard
            })),
        ),
        (
            "A18 stream reading with reason",
            ResolvedCounter::stream(review_changed(&interp, |a| {
                a.basis.unresolved = Some("corrupt reason")
            })),
        ),
        (
            "A18 stream missing reason",
            ResolvedCounter::stream(review_changed(&empty_stream, |a| a.basis.unresolved = None)),
        ),
    ];
    for (name, a) in cases {
        let rejected = match &a {
            ResolvedCounter::Offline(a) => AnswerView::offline(a, &[]).is_err(),
            ResolvedCounter::Stream(a) => AnswerView::stream(a, &[]).is_err(),
        };
        assert!(rejected, "{name}");
    }
}
